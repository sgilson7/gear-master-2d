//! The Cairnworks: four floors, four lanes.

mod common;

use gm2d_core::combat::{self, Difficulty, Outcome};

const D: Difficulty = Difficulty::Easy;
const FLOORS: [&str; 4] =
    ["The Unslaked Kiln", "Nine Courses of Brick", "The Cold Flue", "What Was Left Banked"];

/// **Every floor is a fight the board a player actually has can win**, and the
/// lane is what makes it quick rather than what makes it possible.
///
/// `stats::LANE_CAP` is 95 and is not moving — *a lane you can commit to is
/// never one you can be shut out of* — so the wrong lane does a twentieth of
/// its damage rather than none. Against a clock that is a loss at the buzzer,
/// which is what "you have to use the other lane" means in this game; what it
/// must never become is a boss nobody can beat, which is a wall with a
/// sentence on it and is the thing this project shipped a whole block of once.
#[test]
fn every_floor_of_the_cairnworks_can_be_beaten() {
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let mut lost = Vec::new();
    for name in FLOORS {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        let log = combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D);
        let dps = (m.health as f32 / (log.duration_ms.max(1) as f32 / 1000.0)) as i32;
        println!("  {name}: {:?} in {}ms (about {dps}/s into it)", log.outcome, log.duration_ms);
        if log.outcome != Outcome::Victory {
            lost.push(name);
        }
    }
    assert!(lost.is_empty(), "the board a player actually has cannot beat: {lost:?}");
}

/// **And none of them is quick for a board that brought the wrong lane.**
///
/// The other half of the design, and without it the block is four bosses with
/// large numbers on them. A generic board wins these at 33 to 42 seconds —
/// which is the buzzer's own neighbourhood — because the lane it is swinging in
/// is doing a twentieth of its damage. A board built for the floor should
/// finish long before that, and the gap between the two is the whole of what
/// "you have to use the other lane" buys in a game with a clock.
///
/// *A check that compares zero with zero is not a check*: at these healths a
/// fight that came in under twenty-five seconds would mean the shut lanes had
/// stopped mattering, and nothing else would say so.
#[test]
fn no_floor_is_quick_for_the_wrong_lane() {
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    for name in FLOORS {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        let log = combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D);
        assert!(
            log.duration_ms > 25_000,
            "{name} went down in {}ms to a board that brought no particular lane",
            log.duration_ms
        );
    }
}

/// **Each floor shuts every lane but one, and the open lane is a measurement.**
///
/// Written as a large negative rather than as a zero, because a board grants
/// resistances of its own: a boss with `curse_resist: 0` and forty pieces on it
/// is a boss with about forty curse resist, and the design would have been
/// undone by the costume.
#[test]
fn the_cairnworks_shuts_every_lane_but_one() {
    let cap = gm2d_core::stats::LANE_CAP;
    let of = |name: &str| {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        let (s, _) = m.outfit_at(D);
        (s.physical_resist, s.magic_resist, s.mind_resist, s.curse_resist)
    };

    let (p, mg, mi, c) = of("The Unslaked Kiln");
    assert!(p >= cap && mg >= cap && mi >= cap, "the kiln leaves a damage lane open: {p}/{mg}/{mi}");
    assert!(c <= 0, "the kiln resists curses at {c}, so searing it is not the answer");

    let (p, mg, mi, c) = of("Nine Courses of Brick");
    assert!(mg >= cap && mi >= cap && c >= cap, "the brick leaves something open: {mg}/{mi}/{c}");
    assert!(p <= 0, "the brick resists a blade at {p}");

    let (p, mg, mi, c) = of("The Cold Flue");
    assert!(p >= cap && mi >= cap && c >= cap, "the flue leaves something open: {p}/{mi}/{c}");
    assert!(mg <= 0, "the flue resists magic at {mg}");

    let (p, mg, mi, c) = of("What Was Left Banked");
    assert!(
        p >= cap && mg >= cap && mi >= cap && c >= cap,
        "the last one leaves a lane open: {p}/{mg}/{mi}/{c}"
    );
}

/// A floor deals its own boss, so clearing the plate is not the end of it.
///
/// Asked for: *make sure the previous bosses can be somehow refought and farmed
/// for gear that helps with the next boss in the chain.* A pool of one has no
/// weighting to get wrong — `draw_enemy` makes a pool's hardest member its
/// rarest, and a floor whose pool held its boss beside three ordinary creatures
/// dealt it about one time in a hundred.
#[test]
fn a_floor_deals_the_thing_that_stands_on_its_plate() {
    let drops = gm2d_core::data::drops();
    for (n, name) in FLOORS.iter().enumerate() {
        let w = gm2d_core::data::map(&format!("the-cairnworks-{}", n + 1), D);
        let pool: Vec<&str> = w.regions[0].enemies.iter().map(|m| m.name).collect();
        assert_eq!(pool, vec![*name], "floor {} deals {pool:?}", n + 1);
        assert!(
            w.places.iter().any(|p| p.creature.as_deref() == Some(*name)),
            "{name} is in the pool and on no plate"
        );
        assert!(!drops.of(name).is_empty(), "{name} leaves nothing to farm it for");
    }
}
