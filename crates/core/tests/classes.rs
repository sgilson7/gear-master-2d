//! Level five forks the character.
//!
//! M5's acceptance, and the MVP's. The last test in this file walks the
//! checklist in `PLANNING-BRIEF.md` §0 line by line, because "the MVP is
//! finished when all of these are true" is a claim that should be checked by
//! something other than a person remembering to.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::progression;
use gm2d_core::save;
use gm2d_core::skills::Refusal;

const D: Difficulty = Difficulty::Easy;

/// The three GM2D ships. Canonical names — the theme renames them on the way
/// to the screen, like every other name a player reads.
const THREE: [&str; 3] = ["Berserker", "Hexweaver", "Bloodletter"];

fn at_level(n: u32) -> Game {
    let mut g = Game::new(0x5EED_1234_ABCD_0001, "td");
    g.character.gain_xp(progression::xp_to_reach(n));
    g.character.resize_boards([0; 5]);
    g.character.apply_preset();
    g
}

// ------------------------------------------------------------------ the fork

/// A class is offered at five, and not before.
#[test]
fn a_class_is_offered_at_five_and_not_before() {
    for level in 1..5 {
        let g = at_level(level);
        assert!(!g.character.owed_a_class(), "a class was offered at level {level}");
    }
    for level in 5..9 {
        let g = at_level(level);
        assert!(g.character.owed_a_class(), "no class was offered at level {level}");
    }
}

/// **A save made before level 5 loads and is still asked at level 5.**
///
/// The plan's acceptance in its own words. The question is asked because it has
/// never been answered, not because the save was made at a particular moment —
/// so a character who reached level 9 without choosing is still asked.
#[test]
fn a_pre_level_five_save_is_still_asked() {
    let before = at_level(3);
    assert!(!before.character.owed_a_class());

    let mut after = save::load(&save::save(&before)).expect("a level-three save loads");
    assert_eq!(after.character.level(), 3);
    assert!(!after.character.owed_a_class());

    after.character.gain_xp(progression::xp_to_reach(5) - after.character.xp);
    assert_eq!(after.character.level(), 5);
    assert!(after.character.owed_a_class(), "level five and no class was offered");

    // And a save made *at* nine with no class is still asked.
    let late = save::load(&save::save(&at_level(9))).unwrap();
    assert!(late.character.owed_a_class());
}

/// **A class choice is permanent within a save.**
///
/// There is no path that clears one. The test cannot prove a negative about
/// every future edit, so it does the two things it can: it shows that choosing
/// twice is refused, and it walks the whole public surface of `Character`
/// looking for something that puts the class back.
#[test]
fn a_class_is_permanent() {
    let mut g = at_level(5);
    g.character.choose_class("Berserker").expect("a class");
    assert_eq!(g.character.class.as_deref(), Some("Berserker"));

    let e = g.character.choose_class("Hexweaver").unwrap_err();
    assert!(e.contains("Berserker"), "refusing should name what you already are: {e}");
    assert_eq!(g.character.class.as_deref(), Some("Berserker"), "the class changed anyway");

    // Everything else a player can do to a character, and none of it undoes it.
    let tree = data::skills();
    g.character.gain_xp(5000);
    g.character.resize_boards([1; 5]);
    g.character.apply_preset();
    g.character.apply_skills(&tree);
    g.character.clear_all();
    let _ = g.character.take_skill(&tree, "frame-sense");
    g.character.undo();
    g.character.forget_undo();
    assert_eq!(g.character.class.as_deref(), Some("Berserker"), "something cleared the class");

    // Nor does a round trip.
    let after = save::load(&save::save(&g)).unwrap();
    assert_eq!(after.character.class.as_deref(), Some("Berserker"));
}

/// Choosing below five is refused, and so is a class nobody wrote.
#[test]
fn a_class_cannot_be_taken_early_or_invented() {
    let mut early = at_level(4);
    let e = early.character.choose_class("Berserker").unwrap_err();
    assert!(e.contains("level 5") && e.contains("level 4"), "{e}");
    assert!(early.character.class.is_none());

    let mut g = at_level(5);
    let e = g.character.choose_class("Wumpus Hunter").unwrap_err();
    assert!(e.contains("Wumpus Hunter"), "{e}");
    assert!(g.character.class.is_none());
}

/// All three ship, all three are named by the theme, and each promises
/// something a player can read.
#[test]
fn the_three_classes_are_real() {
    let theme = gm2d_core::theme::by_id("td");
    let tree = data::skills();
    for canonical in THREE {
        let def = gm2d_core::class::CLASSES
            .iter()
            .find(|c| c.name == canonical)
            .unwrap_or_else(|| panic!("no class called {canonical}"));

        let shown = theme.class(canonical);
        assert_ne!(shown, canonical, "{canonical} has no themed name");
        assert!(!def.blurb.is_empty(), "{canonical} has no blurb");

        // The one-line mechanical promise the plan asks each class to make.
        let promise = def.power.describe();
        assert!(!promise.is_empty(), "{canonical} promises nothing mechanical");

        let t = tree
            .tree_for_class(canonical)
            .unwrap_or_else(|| panic!("{canonical} has no tree"));
        assert!(
            (8..=12).contains(&t.nodes.len()),
            "{canonical}'s tree has {} nodes and the plan asks for 8 to 12",
            t.nodes.len()
        );
    }
}

// ------------------------------------------------------------------ the locks

/// A class tree is shut to everybody but its class.
#[test]
fn class_trees_are_locked() {
    let tree = data::skills();
    let mut g = at_level(5);
    g.character.skill_points = 9;

    // No class yet: every class node refuses, and says why.
    assert_eq!(g.character.take_skill(&tree, "g-deadlift"), Err(Refusal::NoClassYet));

    g.character.choose_class("Berserker").unwrap();

    // Its own tree opens.
    g.character.take_skill(&tree, "g-deadlift").expect("a Gorillathon node");

    // The other two stay shut, and name whose they are.
    match g.character.take_skill(&tree, "f-issue") {
        Err(Refusal::WrongClass(whose)) => assert_eq!(whose, "Funnel Sergeant"),
        other => panic!("expected a wrong-class refusal, got {other:?}"),
    }
    match g.character.take_skill(&tree, "w-the-fact") {
        Err(Refusal::WrongClass(whose)) => assert_eq!(whose, "Worm-Fact Keeper"),
        other => panic!("expected a wrong-class refusal, got {other:?}"),
    }

    // And the base tree is open to everybody, always.
    g.character.take_skill(&tree, "thick-skull").expect("a base node");
}

/// A class changes the fight, and an unclassed fight is the fight it always was.
#[test]
fn a_class_reaches_combat() {
    use gm2d_core::combat::LADDER;
    use gm2d_core::fight::{self, Encounter};

    let spec = LADDER.iter().find(|m| m.name == "Rust Golem").unwrap();
    let mut plain = at_level(5);
    plain.encounter = Some(Encounter { enemy: spec.name.into(), at: [1, 18] });

    let before = fight::run(&plain, D).expect("a fight");

    let mut classed = plain.clone();
    classed.character.choose_class("Berserker").unwrap();
    let after = fight::run(&classed, D).expect("a fight");

    assert_ne!(
        format!("{:?}", before.entries),
        format!("{:?}", after.entries),
        "taking a class changed nothing about the fight"
    );
}

// ------------------------------------------------------------------ the run

/// **Level 1 to a class choice, on the shipped map, on one seed.**
///
/// The plan asks for a full playthrough scripted as an integration test so it
/// is reproducible. This is it: a real character, real encounter rolls, real
/// fights, real levels, arriving at the fork and taking it.
#[test]
fn a_whole_run_reaches_the_fork() {
    use gm2d_core::combat::{simulate_with_class, Outcome};
    use gm2d_core::world::{step, Allowances, Dir, WorldState};

    let w = data::world(D);
    let mut g = Game::new(0xC0FF_EE00_1234_5678, "td");
    g.world = WorldState::at_start(&w);
    g.character.apply_preset();
    let tree = data::skills();

    let patrol = [
        Dir::East, Dir::East, Dir::East, Dir::East, Dir::East, Dir::East,
        Dir::West, Dir::West, Dir::West, Dir::West, Dir::West, Dir::West,
    ];

    let (mut fights, mut wins) = (0, 0);
    for i in 0..20_000 {
        if g.character.owed_a_class() {
            break;
        }
        let s = step(&w, &mut g.world, &mut g.rng, D, patrol[i % patrol.len()], &Allowances::default());
        let Some(m) = s.encounter else { continue };
        fights += 1;

        let worn: Vec<gm2d_core::class::ClassDef> =
            g.character.class_def().into_iter().cloned().collect();
        let log = simulate_with_class(
            g.character.player_stats(),
            &g.character.combat_items(),
            m,
            D,
            &worn,
        );
        if log.outcome == Outcome::Victory {
            wins += 1;
            g.character
                .gain_xp(progression::xp_for_rating(gm2d_core::rating::creature_rating(m, D)));
            g.character.resize_boards([0; 5]);
            g.character.apply_preset();
            // Spend every point as it arrives, on whatever is takeable.
            while g.character.skill_points > 0 {
                let next = tree
                    .base()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|n| {
                        tree.can_take(&n.id, &g.character.skills_taken, g.character.skill_points, None)
                            .is_ok()
                    })
                    .map(|n| n.id.clone());
                match next {
                    Some(id) => g.character.take_skill(&tree, &id).map(|_| ()).unwrap_or(()),
                    None => break,
                }
            }
        }
    }

    assert!(g.character.owed_a_class(), "the run never reached level five");
    assert_eq!(g.character.level(), 5, "arrived at level {}", g.character.level());
    assert!(wins > 0 && fights >= wins, "{wins} wins in {fights} fights");
    assert!(
        !g.character.skills_taken.is_empty(),
        "four levels and nothing was spent"
    );

    // Take the fork, and it holds across a save.
    g.character.choose_class("Hexweaver").expect("the fork");
    let after = save::load(&save::save(&g)).expect("the run saves");
    assert_eq!(after.character.class.as_deref(), Some("Hexweaver"));
    assert_eq!(after.character.level(), 5);
    assert_eq!(after, g, "the run did not survive being written down");
}

// ------------------------------------------------------------------ the MVP

/// **The checklist in `PLANNING-BRIEF.md` §0, walked line by line.**
///
/// "MVP is finished when all of these are true in the deployed browser build."
/// Six of the seven lines are facts about core and are checked here; the
/// seventh is the deploy itself, which `testing/drive.py` walks in three
/// browsers on every push.
#[test]
fn the_mvp_checklist() {
    use gm2d_core::piece::SlotKind;
    let tree = data::skills();

    // 1. A levelling system exists, and levelling pays a **point**.
    //
    // **It used to say levelling adds a row, and M12.3 retired that.** The
    // MVP's rotation handed a frame a row every level; M12.0 measured what
    // that cost once components were not scarce — fill falling as you level —
    // so a row is earned now, with the point this level just paid. What a
    // level still does, and what this checks, is hand you the point.
    let mut c = Character::starting();
    let before = c.slot_rows();
    let points_before = c.skill_points;
    c.gain_xp(progression::xp_to_reach(2));
    c.resize_boards([0; 5]);
    assert_eq!(c.level(), 2);
    assert_eq!(c.slot_rows(), before, "a level grew a board, which M12.3 retired");
    assert!(c.skill_points > points_before, "a level paid no point");
    let tree_grows = tree
        .trees
        .iter()
        .flat_map(|t| t.nodes.iter())
        .any(|n| n.effects.iter().any(|e| matches!(e, gm2d_core::skills::Effect::GrowSlotRows { .. })));
    assert!(tree_grows, "and there is nowhere to spend it on a row");

    // 2. A base skill tree exists and the player gets one point per level.
    let base = tree.base().expect("a base tree");
    assert!(!base.nodes.is_empty());
    let mut d = Character::starting();
    let levels = d.gain_xp(progression::xp_to_reach(6));
    assert_eq!(levels.len(), 5, "five levels crossed");
    assert_eq!(d.skill_points, 5, "one point a level");

    // 3. At level 5 the player chooses a class, which unlocks a class tree.
    let mut e = at_level(5);
    assert!(e.character.owed_a_class());
    e.character.choose_class("Bloodletter").unwrap();
    assert!(tree.tree_for_class("Bloodletter").is_some());
    e.character.skill_points = 1;
    e.character.take_skill(&tree, "w-the-fact").expect("the class tree opened");

    // 4. A save round-trips, including the RNG.
    let mut g = at_level(5);
    for _ in 0..9 {
        g.rng.next_u64();
    }
    let back = save::load(&save::save(&g)).expect("a save loads");
    assert_eq!(back, g, "the game did not round-trip");
    assert_eq!(back.rng.state(), g.rng.state(), "the RNG did not round-trip");

    // 5. A 20×20 map with terrain, tile-bound events, and random combat whose
    //    chance is a function of terrain and danger.
    let w = data::world(D);
    assert_eq!((w.width, w.height), (20, 20));
    assert!(w.places.iter().any(|p| p.kind == gm2d_core::world::PlaceKind::Event));
    let quiet = w.regions.iter().map(|r| r.danger).min().unwrap();
    let loud = w.regions.iter().map(|r| r.danger).max().unwrap();
    assert!(loud > quiet, "the map has no danger gradient");

    // 6. It is hosted on GitHub Pages in a separate repo — the deploy, which
    //    `testing/drive.py` walks in three browsers on every push. Nothing here
    //    can check that, and pretending to would be worse than saying so.
}

/// **No offered class promises a mechanic this game has not got.**
///
/// Upstream handed the same class out over and over, so a promise had to say
/// what a second one bought. GM2D asks once, at level five, and the answer
/// does not come off — so a sentence about carrying five of something
/// describes a game the player is not in. It reached the screen: the Kaklon
/// Patent's promise said "for each stack of Recycler you are carrying. Five
/// stacks is half again on all five slots", on the one screen in the game
/// somebody reads before an irreversible choice.
#[test]
fn no_class_on_offer_promises_a_stack() {
    // The four the fork deals. Named here rather than read from the shim,
    // because the shim is wasm and this is the list it holds.
    const OFFERED: [&str; 4] = ["Berserker", "Hexweaver", "Bloodletter", "Recycler"];
    for name in OFFERED {
        let def = gm2d_core::class::CLASSES
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("{name} is offered and is not a class"));
        let said = def.power.describe().to_lowercase();
        assert!(
            !said.contains("stack"),
            "{name} promises {said:?}, and nobody carries two of anything here"
        );
        // A number, in digits or spelled out. Spelling small ones out is the
        // house style — TONE rule 12's lint had to learn the same thing after
        // it failed "Forty Fnorp" for naming no number.
        const SPELT: &[&str] = &[
            "once", "twice", "one", "two", "three", "four", "five", "six", "seven",
            "eight", "nine", "ten", "half", "double",
        ];
        let counted = said.chars().any(|c| c.is_ascii_digit())
            || said.split(|c: char| !c.is_alphanumeric()).any(|w| SPELT.contains(&w));
        assert!(counted, "{name} promises {said:?}, which names no number");
        // And it has a tree to spend points in, or the promise is the whole
        // class.
        assert!(
            gm2d_core::data::skills().tree_for_class(name).is_some(),
            "{name} is offered and has no tree"
        );
    }
}
