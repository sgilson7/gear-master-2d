//! A creature leaves something behind, and how often.
//!
//! M9.1 is the roll, the rate and the eight components it hands out. The
//! components are gear here and a *set* in M9.2 — what makes a set a set is
//! `AssemblyBonus::names` and `AssemblyBonus::grants`, and both arrive with the
//! rules they hand out. The split is the plan's and the reason is that the rate
//! is the thing that gets retuned and the gear is the thing that gets added to.

mod common;

use gm2d_core::combat::{Difficulty, Outcome};
use gm2d_core::data;
use gm2d_core::drops::{self, DropsData, MAX_PER_MILLE};
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::rng::Rng;

const D: Difficulty = Difficulty::Easy;

fn shipped() -> DropsData {
    data::drops()
}

// ------------------------------------------------------------------ the file

/// A drop off a creature that is nowhere is content nobody can reach.
///
/// The same lint the errands got in M8.1, and for the same reason: nothing else
/// in the game would say so. Walks every shipped map, because a creature can
/// live in a dungeon.
#[test]
fn every_drop_names_a_creature_some_region_holds() {
    let maps = data::all_maps(D);
    for e in &shipped().drops {
        let anywhere = maps.iter().any(|m| !m.regions_holding(&e.creature).is_empty())
            || maps.iter().any(|m| {
                m.places.iter().any(|p| p.creature.as_deref() == Some(e.creature.as_str()))
            });
        assert!(anywhere, "{:?} drops {:?} and lives on no map", e.creature, e.piece);
    }
}

/// **A drop you could buy is worse than a quest reward you could buy.**
///
/// A quest reward on a shelf makes the errand a slow way to shop; a drop on a
/// shelf makes the creature that owns it a formality. Every one of them is in
/// `EVENT_ONLY`, which does the four jobs that block already names — off the
/// shelves, out of `melt` in both directions, out of `dearer_than`, and out of
/// every footprint family the creature stepper walks.
#[test]
fn every_drop_is_off_every_shelf() {
    let shops = data::shops();
    for name in shipped().every_piece() {
        assert!(
            gm2d_core::piece::is_event_only(name),
            "{name} drops off a creature and is not EVENT_ONLY, so the ladder can deal it"
        );
        for town in &shops.towns {
            assert!(
                !town.stock.iter().any(|s| s == name),
                "{} sells {name}, which a creature is supposed to be the only source of",
                town.id
            );
        }
    }
}

/// The rates the file may hold at all, and the reason there is a ceiling.
#[test]
fn a_rate_is_a_chance_and_not_a_certainty() {
    for e in &shipped().drops {
        assert!(e.per_mille > 0 && e.per_mille <= MAX_PER_MILLE, "{e:?}");
    }
    // And the loader refuses the ones outside it rather than letting somebody
    // discover a 0% drop by fighting for an hour.
    let bad = |text: &str| DropsData::parse(text).unwrap_err();
    let wrap = |row: &str| {
        format!(r#"{{"format":"gm2d-drops","version":1,"drops":[{row}]}}"#)
    };
    assert!(bad(&wrap(
        r#"{"creature":"Cave Rat","piece":"Ratskin Mold","per_mille":0}"#
    ))
    .contains("per mille"));
    assert!(bad(&wrap(
        r#"{"creature":"A. Rat","piece":"Ratskin Mold","per_mille":50}"#
    ))
    .contains("ladder"), "the themed name is not what the engine matches on");
    assert!(bad(&wrap(
        r#"{"creature":"Cave Rat","piece":"Cheese Touch","per_mille":50}"#
    ))
    .contains("component"));
}

// ------------------------------------------------------------------ the roll

/// A drop only falls for the creature that owns it.
#[test]
fn a_drop_only_falls_for_the_creature_that_owns_it() {
    let d = shipped();
    let rats: Vec<&str> = d.of("Cave Rat").iter().map(|e| e.piece.as_str()).collect();
    let toads: Vec<&str> = d.of("Bog Toad").iter().map(|e| e.piece.as_str()).collect();
    assert!(!rats.is_empty() && !toads.is_empty(), "the pit's creatures drop nothing");
    for r in &rats {
        assert!(!toads.contains(r), "{r} falls off two creatures, so it is not either one's");
    }
    // Walked, not reasoned: a thousand rats never leave a toad's hide.
    let mut rng = Rng::new(0xD0FF);
    for _ in 0..1000 {
        for got in drops::roll(&d, &mut rng, "Cave Rat") {
            assert!(rats.contains(&got.as_str()), "a Cave Rat left a {got}");
        }
    }
    // And a creature with no entry leaves nothing at all, without drawing.
    let mut rng = Rng::new(7);
    let before = rng.state();
    assert!(drops::roll(&d, &mut rng, "The Hollow King").is_empty());
    assert_eq!(rng.state(), before, "a creature that drops nothing must not move the stream");
}

/// **The roll happens whether or not you can keep it.**
///
/// Skipping the draw for a piece already in the bag would make the stream a
/// function of what the player is carrying rather than of the fights they had —
/// which is a save that replays for the person who wrote it and not for the
/// person they sent it to.
#[test]
fn the_stream_is_a_function_of_the_fights_and_not_of_the_bag() {
    let d = shipped();
    let mut a = Rng::new(0xBEEF);
    let mut b = Rng::new(0xBEEF);
    for _ in 0..200 {
        let _ = drops::roll(&d, &mut a, "Cave Rat");
        let _ = drops::roll(&d, &mut b, "Cave Rat");
    }
    assert_eq!(a.state(), b.state());
    // One entry, one draw, every time — so the number of draws a win costs is
    // a property of the file and not of the outcome.
    let n = d.of("Cave Rat").len() as u32;
    let mut r = Rng::new(3);
    let mut count = Rng::new(3);
    let _ = drops::roll(&d, &mut r, "Cave Rat");
    for _ in 0..n {
        count.below(1000);
    }
    assert_eq!(r.state(), count.state(), "a roll is one draw per entry, in file order");
}

// ------------------------------------------------------------------ the rate

/// **The rate is set by a test, and this is the test.**
///
/// `XP_DIVISOR` is 5 because a test says so and `PER_FIGHT` is 4 because a test
/// walks twelve fights. This is the same: a whole set has to be a few hours of
/// meeting a creature and not a lifetime of it, so it is walked with the real
/// roll and the real generator over a hundred seeds and the mean is held
/// between fifteen and a hundred and twenty wins.
///
/// **Wins against the creature that owns the set**, which is what a player
/// counts. How often the pit deals that creature is the map's question and not
/// the rate's — `draw_enemy` weights a pool so its hardest member is its rarest,
/// and the Bone Archer is one encounter in ninety-seven down there. That is a
/// real problem and it is not this number's: see the note this milestone leaves
/// for M9.4.
#[test]
fn a_set_is_a_few_hours_and_not_a_lifetime() {
    let d = shipped();
    let mut creatures: Vec<&str> = d.drops.iter().map(|e| e.creature.as_str()).collect();
    creatures.dedup();
    for who in creatures {
        let want: Vec<&str> = d.of(who).iter().map(|e| e.piece.as_str()).collect();
        let mut total = 0u32;
        let trials = 100;
        for seed in 0..trials {
            let mut rng = Rng::new(0x5E7_0000 + seed as u64);
            let mut have: Vec<String> = Vec::new();
            let mut wins = 0u32;
            while have.len() < want.len() {
                wins += 1;
                assert!(wins < 100_000, "{who}'s set never completed at all");
                for got in drops::roll(&d, &mut rng, who) {
                    if !have.contains(&got) {
                        have.push(got);
                    }
                }
            }
            total += wins;
        }
        let mean = total / trials;
        assert!(
            (15..=120).contains(&mean),
            "{who}: {} pieces take a mean of {mean} wins. Fewer than fifteen is a set \
             you get by accident; more than a hundred and twenty is a set nobody finishes.",
            want.len()
        );
        println!("{who}: {} pieces, mean {mean} wins", want.len());
    }
}

// ------------------------------------------------------------- and the fight

/// A win pays it, a loss does not, and it only ever falls once.
#[test]
fn a_settled_win_hands_the_drop_over_and_nothing_drops_twice() {
    let want = "Ratskin Mold";
    let mut g = Game::new(0xD10B_5EED, "td");
    // A board that beats a Cave Rat four hundred times running, which a
    // starting kit does not: `build_full_loadout` is the known-good fixture and
    // it needs the whole catalogue and the full frames to seat.
    g.character = common::bench();
    common::build_full_loadout(&mut g.character);
    // And it must not already own what it is waiting for: `with_all_pieces` is
    // the whole catalogue, drops included, and a bag that already holds the
    // mold refuses it for ever — which is the refusal working, and would have
    // read here as a roll that never fires.
    g.character.owned.retain(|&p| !shipped().every_piece().contains(&g.character.registry.def(p).name));

    let mut fights = 0;
    let mut got = 0;
    while got == 0 && fights < 400 {
        fights += 1;
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).unwrap();
        assert_eq!(log.outcome, Outcome::Victory, "the test board lost to a Cave Rat");
        let s = fight::settle(&mut g, &log, D).unwrap();
        if s.receipt.iter().any(|l| l.contains(&g.theme_piece(want))) {
            got += 1;
        }
        // Fatigue would eventually make even a Cave Rat interesting.
        g.character.fatigue = 0;
    }
    assert_eq!(got, 1, "{fights} won fights and the Cave Rat never left a {want}");
    assert!(g.character.holds(want));

    // **Refused when you already have it.** A set is three specific pieces and
    // not three of a kind; a bag filling with molds is the litter an errand's
    // tally is already refused for.
    for _ in 0..400 {
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).unwrap();
        fight::settle(&mut g, &log, D).unwrap();
        g.character.fatigue = 0;
    }
    let held = g.character.owned.iter().filter(|&&p| g.character.registry.def(p).name == want).count();
    assert_eq!(held, 1, "four hundred more rats left {held} molds");
}

/// A loss leaves nothing behind, because nothing was beaten.
#[test]
fn a_lost_fight_drops_nothing() {
    let mut g = Game::new(0x105E, "td");
    // A starting character against the top of the ladder: this is a defeat.
    g.encounter = Some(fight::Encounter { enemy: "Bone Archer".into(), at: [1, 18] });
    let log = fight::run(&g, D).unwrap();
    assert_ne!(log.outcome, Outcome::Victory, "the whole point of this fight is losing it");
    let before = g.character.owned.len();
    fight::settle(&mut g, &log, D).unwrap();
    assert_eq!(g.character.owned.len(), before, "a defeat handed something over");
}

/// A dropped piece is a registry entry like any other, so it should survive a
/// save. Asserted rather than assumed.
#[test]
fn a_dropped_piece_survives_a_round_trip() {
    let mut g = Game::new(0xA11, "td");
    for name in shipped().every_piece() {
        g.character.give(name).unwrap_or_else(|| panic!("{name} is not in the catalogue"));
    }
    let text = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&text).expect("a save with drops in it reloads");
    assert_eq!(g, back);
    for name in shipped().every_piece() {
        assert!(back.character.holds(name), "{name} did not survive the trip");
    }
}
