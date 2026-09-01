//! A fight is the same fight however you arrive at it.
//!
//! M3's acceptance, and the one that ties the milestone to M1: an encounter is
//! state, so it survives a save, and combat draws nothing, so the fight it
//! reopens is the fight that was interrupted.

use gm2d_core::character::Character;
use gm2d_core::combat::{Difficulty, Outcome};
use gm2d_core::fight::{self, Encounter};
use gm2d_core::game::Game;
use gm2d_core::save;

const D: Difficulty = Difficulty::Easy;

fn facing(enemy: &str) -> Game {
    let mut g = Game::new(0x5EED_1234_ABCD_0001, "td");
    g.character = Character::with_all_pieces();
    g.character.loadout.name_seed = 0x5EED_1234_ABCD_0001;
    g.character.loadout.naming = gm2d_core::theme::by_id("td").naming;
    g.character.apply_preset();
    g.world.last_town = "the-end-of-all-gears".into();
    g.encounter = Some(Encounter { enemy: enemy.into(), at: [9, 17] });
    g
}

/// **The mid-fight save.** An encounter saved and reloaded is the same fight.
///
/// `PLAN.md` §6 proposed storing the pre-fight state and the seed for this.
/// Neither is needed and the reason is a property worth keeping: combat has no
/// RNG, so naming the creature and holding the board is enough to reproduce the
/// fight character for character.
#[test]
fn a_fight_saved_halfway_reopens_as_the_same_fight() {
    let before = facing("Rust Golem");
    let a = fight::run(&before, D).expect("a fight to run");

    let after = save::load(&save::save(&before)).expect("a mid-fight save loads");
    assert_eq!(after.encounter, before.encounter, "the creature was forgotten");

    let b = fight::run(&after, D).expect("the reopened fight runs");
    assert_eq!(a.outcome, b.outcome);
    assert_eq!(a.duration_ms, b.duration_ms);
    assert_eq!(format!("{:?}", a.entries), format!("{:?}", b.entries),
               "the reopened fight went differently");
}

/// A win pays the bounty and the rating; a loss pays nothing and walks you home.
#[test]
fn winning_pays_and_losing_does_not() {
    // The rat is beatable by the preset; Francis is not.
    let mut won = facing("Cave Rat");
    let log = fight::run(&won, D).unwrap();
    assert_eq!(log.outcome, Outcome::Victory, "the preset should beat a rat");
    let purse = won.character.gold;
    let s = fight::settle(&mut won, &log, D).expect("a settlement");
    assert!(s.gold > 0, "a win paid {}", s.gold);
    assert_eq!(won.character.gold, purse + s.gold);
    assert!(s.sent_home.is_none(), "a win sent the player home");
    assert!(won.encounter.is_none(), "the encounter outlived its settlement");
    assert!(won.character.xp > 0, "a win banked no experience");

    let mut lost = facing("Francis");
    let log = fight::run(&lost, D).unwrap();
    assert_ne!(log.outcome, Outcome::Victory, "the preset should not beat Francis");
    let purse = lost.character.gold;
    let s = fight::settle(&mut lost, &log, D).expect("a settlement");
    assert_eq!(s.gold, 0, "a loss paid {}", s.gold);
    assert_eq!(lost.character.gold, purse, "the purse moved on a loss");
    assert_eq!(s.sent_home.as_deref(), Some("the-end-of-all-gears"));
    assert!(lost.encounter.is_none());
}

/// Settling twice does not pay twice.
#[test]
fn a_fight_settles_once() {
    let mut g = facing("Cave Rat");
    let log = fight::run(&g, D).unwrap();
    let first = fight::settle(&mut g, &log, D).expect("the first settlement");
    assert!(first.gold > 0);
    assert!(fight::settle(&mut g, &log, D).is_none(), "a settled fight settled again");
}

/// The board the player is wearing is the board that fights.
///
/// Guards the failure where the fight reads a stale or default character —
/// which would be invisible, because a fight against a default board still
/// produces a perfectly valid log.
#[test]
fn the_fight_uses_the_board_that_is_on() {
    use gm2d_core::piece::SlotKind;
    let armed = facing("Bog Toad");
    let with_gear = fight::run(&armed, D).unwrap();

    let mut bare = facing("Bog Toad");
    for k in SlotKind::ALL {
        bare.character.loadout.slot_mut(k).clear();
    }
    let without = fight::run(&bare, D).unwrap();

    assert_ne!(
        with_gear.outcome == without.outcome && with_gear.duration_ms == without.duration_ms,
        true,
        "stripping every grid changed nothing about the fight"
    );
}

/// An encounter with a creature this build has not got is refused, not guessed.
#[test]
fn an_unknown_creature_does_not_produce_a_fight() {
    let mut g = facing("Cave Rat");
    g.encounter = Some(Encounter { enemy: "A Thing From Another Game".into(), at: [0, 0] });
    assert!(fight::run(&g, D).is_none());
}
