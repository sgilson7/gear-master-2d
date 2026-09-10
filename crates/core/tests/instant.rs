//! A fight you have already had — the tally, and what may be marked.
//!
//! M15.0. Nothing here is a screen: this file is about whether the engine can
//! *count* the thing the menu is going to list, which is the order the block
//! runs in for a reason — a screen listing something the engine cannot count is
//! a screen built on a guess.

use gm2d_core::combat::Difficulty;
use gm2d_core::fight::{self, Encounter, INSTANT_AFTER};
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;

mod common;

const D: Difficulty = Difficulty::Easy;

/// A game standing on the pit's own tile with an encounter rolled.
fn meeting(enemy: &str) -> Game {
    let mut g = Game::new(9, "td");
    g.world = gm2d_core::world::WorldState::at_start(&gm2d_core::data::world(D));
    g.encounter = Some(Encounter { enemy: enemy.into(), at: g.world.at });
    g
}

// ------------------------------------------------------------- the counting

/// A win counts, and it counts under the creature that lost it.
///
/// **Through `settle`, not through a hand-written bump**, because the counter's
/// whole legitimacy is that `pay_a_win` is the one place a win is paid. A test
/// that bumped the counter itself would be testing `WorldState::bump`.
#[test]
fn a_win_is_counted_by_creature() {
    let mut g = meeting("Cave Rat");
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    let log = fight::run(&g, D).expect("there is an encounter to run");
    assert_eq!(
        log.outcome,
        gm2d_core::combat::Outcome::Victory,
        "the board this test is built on cannot beat a Cave Rat, so it proves nothing"
    );
    fight::settle(&mut g, &log, D).expect("a fight that happened settles");
    assert_eq!(g.beaten("Cave Rat"), 1);
    assert_eq!(g.beaten("Bog Toad"), 0, "the win went into somebody else's column");
}

/// A rout counts, because it is a win.
///
/// A player who has routed a rat five times has met the rat five times, and
/// the whole of the feature is *a fight you have already had*. Both paths go
/// through `pay_a_win`, which is why this is one line rather than a second
/// mechanism.
#[test]
fn a_rout_counts_too() {
    let mut g = meeting("Cave Rat");
    g.character = common::bench();
    for (name, x, y) in
        [("Ratskin Material", 0u8, 0u8), ("Ratskin Mold", 2, 0), ("Rat Signet", 4, 0)]
    {
        let id = common::spare(&g.character, name);
        g.character.registry.set_rotation(id, 0);
        g.character.equip(id, SlotKind::Gloves, x, y).expect("the Mandate seats");
    }
    gm2d_core::loadout::lock_assembled_in(
        &mut g.character.loadout,
        &g.character.registry,
        SlotKind::Gloves,
    );
    assert!(
        gm2d_core::rule::routs(&g.character.rules(), "Cave Rat"),
        "the Mandate is not assembled, so nothing routs and this proves nothing"
    );
    fight::rout(&mut g).expect("the Mandate routs a Cave Rat");
    assert_eq!(g.beaten("Cave Rat"), 1, "a rout is a win and pays what a win pays");
}

// --------------------------------------------------------------- the marking

/// Nothing may be marked that has not been beaten five times, and the refusal
/// says how far off you are.
///
/// **The count is in the sentence**, because a button that greys with no reason
/// is a button reported as a bug — this project has written that down six
/// times.
#[test]
fn nothing_under_five_can_be_marked() {
    let mut g = meeting("Cave Rat");
    for had in 0..INSTANT_AFTER {
        let why = g.mark_instant("Cave Rat").expect_err("marked with too few wins");
        assert!(
            why.contains(&had.to_string()) && why.contains(&INSTANT_AFTER.to_string()),
            "the refusal does not count: {why}"
        );
        assert!(!g.is_instant("Cave Rat"), "a refused mark went on anyway");
        g.world.bump(&fight::beat_key("Cave Rat"));
    }
    g.mark_instant("Cave Rat").expect("five wins is the price and it has been paid");
    assert!(g.is_instant("Cave Rat"));
    g.unmark_instant("Cave Rat");
    assert!(!g.is_instant("Cave Rat"), "taking the mark off left it on");
}

/// The list is the count and nothing else, and it is worked out fresh.
#[test]
fn the_menu_lists_what_has_been_beaten_five_times() {
    let mut g = meeting("Cave Rat");
    assert!(g.instant_candidates().is_empty(), "a new game has beaten nothing");
    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER - 1);
    g.world.add(&fight::beat_key("Bog Toad"), INSTANT_AFTER + 3);
    let lines = g.instant_candidates();
    assert_eq!(lines.len(), 1, "four wins is not five: {lines:?}");
    assert_eq!(lines[0].canonical, "Bog Toad");
    assert_eq!(lines[0].beaten, INSTANT_AFTER + 3);
    assert!(!lines[0].marked);
    assert!(!lines[0].name.is_empty(), "the line has no themed name to print");

    // **A counter naming a creature this build has not got is dropped**, not
    // listed: there is nothing to fight and nothing to theme it with.
    g.world.add(&fight::beat_key("A Thing From Another Build"), 40);
    assert_eq!(g.instant_candidates().len(), 1, "a stranger reached the menu");
}

/// Marking is idempotent, because the switch is a switch.
#[test]
fn marking_twice_marks_once() {
    let mut g = meeting("Cave Rat");
    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
    g.mark_instant("Cave Rat").unwrap();
    g.mark_instant("Cave Rat").unwrap();
    assert_eq!(g.world.instant.len(), 1, "one creature, two marks");
}

// ------------------------------------------------------------------ the save

/// An empty mark list and a full one both survive a round trip, and neither
/// moves the fingerprint.
#[test]
fn the_mark_survives_a_round_trip_and_makes_no_seam() {
    let before = gm2d_core::save::catalog_fingerprint();

    let mut g = meeting("Cave Rat");
    g.encounter = None;
    let plain = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&plain).expect("and reads");
    assert!(back.world.instant.is_empty());

    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
    g.mark_instant("Cave Rat").unwrap();
    let marked = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&marked).expect("and reads");
    assert_eq!(back.world.instant, vec!["Cave Rat".to_string()]);
    assert_eq!(back.beaten("Cave Rat"), INSTANT_AFTER, "the counter did not come back");

    assert_eq!(
        before,
        gm2d_core::save::catalog_fingerprint(),
        "M15 moved the catalogue, and it has no business doing that"
    );
}

/// A file written before the field existed opens with nobody marked.
///
/// **This is the no-seam claim stated as a check** rather than as a sentence in
/// a commit: `#[serde(default)]` is only a promise until something plants a
/// file without the key.
#[test]
fn a_save_written_before_the_mark_opens_with_nobody_marked() {
    let mut g = meeting("Cave Rat");
    g.encounter = None;
    let json = gm2d_core::save::save(&g);
    let mut v: serde_json::Value = serde_json::from_str(&json).expect("is json");
    let w = v.get_mut("state").and_then(|s| s.get_mut("world")).expect("the save has a world");
    assert!(
        w.as_object_mut().expect("an object").remove("instant").is_some(),
        "there is no `instant` key to take out, so this check proves nothing"
    );
    let back = gm2d_core::save::load(&serde_json::to_string(&v).unwrap())
        .expect("a file with no `instant` key still opens");
    assert!(back.world.instant.is_empty());
}
