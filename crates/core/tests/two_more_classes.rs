//! M16.4 — the sixth and seventh classes, and the first two GM2D wrote.
//!
//! Every class before these is upstream's with the theme talking, which was the
//! right call while the powers were already tuned and already tested. These two
//! are asked for by name and neither has an upstream to borrow from: one burns
//! held pools into mana empowerment, and one makes mind damage a way to finish
//! rather than a way to whittle.

mod common;

use gm2d_core::character::Character;
use gm2d_core::class::{ClassPower, CLASSES, OFFERED};
use gm2d_core::combat::{self, Combatant, Difficulty, Outcome, Side};
use gm2d_core::data;
use gm2d_core::piece::Resource;

const D: Difficulty = Difficulty::Easy;

/// **The plan assumes three pools and one call, and both are true.**
///
/// `PROMPT-M16.md` asks for this to be checked before M16.4 and reported if it
/// is not: *the plan assumes `pools_worth_holding` returns rage, faith and
/// nature.* It does, and it is derived rather than listed — a pool is in it
/// only if something in the catalogue grants it **and** it pays a wearer for
/// sitting on a pile. Mana and insight fail the second and the three fusions
/// fail the first.
///
/// So the Stoker burns whatever that function returns, and the sentence about
/// *"the three pools"* stands.
#[test]
fn the_furnace_burns_the_pools_that_pay_for_being_held() {
    let pools = Combatant::pools_worth_holding();
    assert_eq!(pools, vec![Resource::Rage, Resource::Faith, Resource::Nature]);
}

/// **A Stoker burns the largest pool, and only that one.**
#[test]
fn a_stoker_burns_the_largest_pool_and_only_that_one() {
    let log = burn_for(&[("rage", 4), ("faith", 60), ("nature", 9)], 12_000);
    let burned: Vec<&'static str> = log
        .entries
        .iter()
        .filter_map(|e| match e.event {
            combat::Event::Burned { what, .. } => Some(what),
            _ => None,
        })
        .collect();
    assert!(!burned.is_empty(), "the furnace never lit");
    assert!(
        burned.iter().all(|w| *w == "faith"),
        "it burned {burned:?} and faith was the biggest of the three"
    );
}

/// **And it moves on once that pool is no longer the largest.**
///
/// The class is a decision about which hopper to fill, which only means
/// anything if *largest* is asked every time rather than chosen once.
#[test]
fn the_furnace_follows_the_pile() {
    // Thirty faith and twenty-five rage. Ten a shovelful, so faith is the
    // biggest for one turn and rage is the biggest after it.
    let log = burn_for(&[("faith", 30), ("rage", 25)], 20_000);
    let burned: Vec<&'static str> = log
        .entries
        .iter()
        .filter_map(|e| match e.event {
            combat::Event::Burned { what, .. } => Some(what),
            _ => None,
        })
        .collect();
    assert!(burned.contains(&"faith") && burned.contains(&"rage"), "it burned only {burned:?}");
}

/// **A burned pool pays no held bonus**, which is the trade the class *is*.
///
/// What you give up is the standing bonus you were holding, and a node that
/// took the cost away entirely would be a node that deleted the class.
/// `Rule::BurnKeepsBonus` is the one thing that softens it and it is a node in
/// the tree rather than the power.
#[test]
fn a_burned_pool_pays_no_held_bonus() {
    let mut c = Combatant::player(gm2d_core::stats::Stats::new(100, 0, 0, 0), &[]);
    c.rage = 40;
    let before = c.held_bonus().physical_damage;
    c.rage = 10;
    c.burned = [30, 0, 0];
    assert_eq!(c.held_bonus().physical_damage, before - 30, "the burned thirty still pays");
    // And with the rule on, half of it comes back.
    c.burn_keeps_pct = 50;
    assert_eq!(c.held_bonus().physical_damage, before - 15, "half of thirty is not fifteen");
}

/// **An unmaking is a kill and pays like one.**
///
/// `settle` is untouched: `is_down` has fired at `max_health <= 0` since the
/// fork, and the class moves where that line is rather than inventing a second
/// way to die. So the bounty, the drops and the tally all arrive through the
/// path they always did.
#[test]
fn an_unmaking_is_a_kill_and_pays_like_one() {
    let spec = combat::creature("Bone Archer").expect("a creature");
    let mut stats = gm2d_core::stats::Stats::new(200, 0, 0, 4_000);
    stats.mind = 60;
    let profiles = common::one_item_dealing_mind();
    let bare = combat::simulate_at(stats, &profiles, spec, D);
    let whispered = combat::simulate_with_class(
        stats,
        &profiles,
        spec,
        D,
        &[gm2d_core::class::ClassDef {
            name: "Whisperer",
            blurb: "",
            requires: &[],
            power: ClassPower::Whisperer { third: 90 },
        }],
    );
    assert_eq!(whispered.outcome, Outcome::Victory, "a ninety percent threshold ends it early");
    assert!(
        whispered.duration_ms < bare.duration_ms || bare.outcome != Outcome::Victory,
        "the unmaking bought nothing: {}ms against {}ms",
        whispered.duration_ms,
        bare.duration_ms
    );
    let said = whispered
        .entries
        .iter()
        .any(|e| matches!(e.event, combat::Event::Unmade { side: Side::Enemy, .. }));
    assert!(said, "nothing in the log says how it ended");
    // The purse does not know the difference, and must not.
    let at = gm2d_core::reward::AtTheBell::default();
    assert_eq!(
        gm2d_core::reward::bounty_with_class(
            whispered.outcome,
            spec.bounty,
            &[],
            whispered.duration_ms,
            at
        ),
        gm2d_core::reward::bounty_with_class(Outcome::Victory, spec.bounty, &[], 1_000, at),
    );
}

/// **`mind_resist` is still the only answer.**
///
/// An unmaking at the cap is slow, not impossible — which is a sentence that
/// only became true in M16.0, when `LANE_CAP` stopped the lane shutting
/// completely. A threshold does not walk through a resistance; it changes where
/// the fight ends.
#[test]
fn mind_resist_is_still_the_only_answer() {
    use gm2d_core::curse::mind_damage_after_resist;
    assert_eq!(mind_damage_after_resist(1000, 0), 1000);
    assert_eq!(mind_damage_after_resist(1000, 90), 100);
    // And at the ceiling it is a twentieth rather than nothing.
    assert_eq!(mind_damage_after_resist(1000, 400), 50);
    // A mind pierce walks through a share of it, and it is a rule three expert
    // trees will grant rather than a knob one of them owns.
    let r = gm2d_core::rule::Rule::MindPierce { pct: 20 };
    assert!(r.check().is_ok());
    assert!(gm2d_core::rule::Rule::MindPierce { pct: 0 }.check().is_err());
}

/// **Seven on the fork, and every one of them reaches something.**
#[test]
fn the_roster_is_seven_and_all_seven_are_real() {
    assert_eq!(OFFERED.len(), 7);
    for name in OFFERED {
        let d = CLASSES.iter().find(|c| c.name == *name).unwrap_or_else(|| panic!("no {name}"));
        assert!(!d.power.describe().is_empty(), "{name} promises nothing");
        // And the promise reaches a player in the world's words rather than
        // the engine's canonical, which is `every_class_name_is_one_a_player
        // _could_read`'s question asked of the two new ones.
        let themed = gm2d_core::theme::by_id("td").class(name);
        assert_ne!(themed, *name, "{name} reaches the fork as its own canonical");
    }
}

/// **The two new powers have knobs, and their own trees turn them.**
///
/// M13.6's finding, one level down: a promise printed from `CLASSES` is the
/// promise as *written*, and thirty-eight expert nodes cost points and changed
/// nothing because of it. A base class with knobs has the same hole, so
/// `Character::class_defs` tunes a base power the way it tunes an expert's.
#[test]
fn a_stokers_own_tree_moves_its_own_promise() {
    let tree = data::skills();
    let mut ch = Character::starting();
    ch.class = Some("Stoker".into());
    let before = ch.class_defs()[0].power.describe();
    // The two nodes that tune it, and their prerequisites — **and two nodes
    // out of somebody else's tree that tune a knob of the same name.**
    // `per_stack` is the Stoker's and the Patented Funnel's, so a reader that
    // walked every taken node rather than every node of *this* tree would move
    // the furnace by seven more, and this is where that shows.
    for id in [
        "ks-damper",
        "ks-hotter-draught",
        "ks-firebox",
        "ks-hundredweight",
        "ks-clinker-rake",
        "pf-the-tap",
        "pf-funnel-patented",
    ] {
        assert!(tree.node(id).is_some(), "{id} is not in the tree");
        ch.skills_taken.push(id.to_string());
    }
    let after = ch.class_defs()[0].power.describe();
    assert_ne!(before, after, "twelve points of tuning and the promise did not move");
    match ch.class_defs()[0].power {
        ClassPower::Stoker { every_ms, per_stack } => {
            assert_eq!(every_ms, 3_200, "a hotter draught is eight hundred milliseconds");
            assert_eq!(per_stack, 7, "the clinker rake is three points a stack");
        }
        p => panic!("a Stoker is not a {p:?}"),
    }
}

/// **And a tuning does not cross from one tree into another.**
///
/// `per_stack` is the Stoker's **and** the Patented Funnel's, which is exactly
/// the collision the parse-time check was written to make impossible — *`carry`
/// is Standing Fact's and Overwound Arm's* — arriving from the other side:
/// the check reads one tree's knobs and `tunings_from` read every tree's
/// tunings. Safe while there was one expert at a time and no base knobs at all,
/// which is why nothing caught it before there were.
#[test]
fn a_tuning_stays_in_its_own_tree() {
    let tree = data::skills();
    let stoker: Vec<String> = tree
        .trees
        .iter()
        .find(|t| t.id == "kettle-stoker")
        .expect("the Stoker's tree")
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect();
    let funnel: Vec<String> = tree
        .trees
        .iter()
        .find(|t| t.id == "patented-funnel")
        .expect("the Funnel's tree")
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect();
    let all: Vec<String> = stoker.iter().chain(funnel.iter()).cloned().collect();
    let mine = tree.tunings_for("Stoker", &all);
    assert!(!mine.is_empty(), "the Stoker's own tree tunes nothing");
    for (knob, _) in &mine {
        assert!(
            ["every_ms", "per_stack"].contains(&knob.as_str()),
            "a {knob:?} reached the Stoker out of somebody else's tree"
        );
    }
    // And both trees do tune `per_stack`, or this proves nothing.
    let both: usize = tree
        .trees
        .iter()
        .filter(|t| {
            matches!(t.id.as_str(), "kettle-stoker" | "patented-funnel")
                && tree
                    .tunings_for(t.class.as_deref().unwrap_or(""), &all)
                    .iter()
                    .any(|(k, _)| k == "per_stack")
        })
        .count();
    assert_eq!(both, 2, "only {both} tree tunes per_stack, so the collision is hypothetical");
}

/// A Stoker's fight, with `held` pools planted.
fn burn_for(pools: &[(&str, i32)], _ms: u32) -> combat::CombatLog {
    let mut held = combat::Held::default();
    for (what, n) in pools {
        match *what {
            "rage" => held.rage = *n,
            "faith" => held.faith = *n,
            "nature" => held.nature = *n,
            other => panic!("{other} is not a pool"),
        }
    }
    let spec = combat::creature("The Iron Warden").expect("something that lasts");
    combat::simulate_holding(
        gm2d_core::stats::Stats::new(400, 0, 0, 3_000),
        &common::one_item_dealing_mind(),
        spec,
        D,
        &[gm2d_core::class::ClassDef {
            name: "Stoker",
            blurb: "",
            requires: &[],
            power: ClassPower::Stoker { every_ms: 4_000, per_stack: 10 },
        }],
        0,
        held,
    )
}

/// **The furnace reaches a board that swings, and for a milestone it did not.**
///
/// Reported from play: *"for the kettle stoker, mana empowerment ... seemingly
/// does nothing for my attacks"*, and it was exactly right. Empowerment is
/// upstream's **caster** mechanic — `magic_empower` scales a magic hit and
/// nothing else — so every stack the furnace bought on a board holding a blade
/// was a number that could never be read.
///
/// Measured against `common::geared_from`, which is what this repository means
/// by *the board a player actually has*: a Stoker dealt **746** and a
/// classless character dealt **746**, over one burn. Not close — identical.
///
/// `every_offered_class_reaches_something` passed it, and the reason is the
/// failure this file has recorded before: **a fixture that casts is measuring
/// a game the reporter is not playing.** So this asks the question of the
/// shopped board instead, which is the one that found it.
#[test]
fn the_furnace_reaches_a_board_that_swings() {
    use gm2d_core::class::ClassDef;
    use gm2d_core::combat::{Event, Side};
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let spec = combat::creature("The Curator").expect("a mid creature").clone();
    let stoker = ClassDef {
        name: "Stoker",
        blurb: "",
        requires: &[],
        power: ClassPower::Stoker { every_ms: 4_000, per_stack: 10 },
    };

    let run = |classes: &[ClassDef]| {
        combat::simulate_holding(
            ch.player_stats(),
            &ch.combat_items(),
            &spec,
            D,
            classes,
            0,
            ch.start_with(),
        )
    };
    let without = run(&[]);
    let with = run(std::slice::from_ref(&stoker));

    let burns = with
        .entries
        .iter()
        .filter(|e| matches!(e.event, Event::Burned { .. }))
        .count();
    assert!(burns > 0, "the furnace never lit on the board a player has");

    // **The fight has to differ**, which is the same question
    // `every_offered_class_reaches_something` asks and the same answer it
    // wants — asked here of a board that does not cast.
    let hits = |log: &combat::CombatLog| -> i32 {
        log.entries
            .iter()
            .filter_map(|e| match &e.event {
                Event::Hit { by, damage, .. } if *by == Side::Player => Some(*damage),
                _ => None,
            })
            .sum()
    };
    assert!(
        with.duration_ms < without.duration_ms || hits(&with) > hits(&without),
        "the furnace changed nothing on a board that swings: {}ms dealing {} against \
         {}ms dealing {}",
        with.duration_ms,
        hits(&with),
        without.duration_ms,
        hits(&without)
    );
}
