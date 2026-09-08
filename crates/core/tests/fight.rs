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

mod common;

const D: Difficulty = Difficulty::Easy;

fn facing(enemy: &str) -> Game {
    let mut g = Game::new(0x5EED_1234_ABCD_0001, "td");
    g.character = Character::with_all_pieces();
    g.character.loadout.name_seed = 0x5EED_1234_ABCD_0001;
    g.character.loadout.naming = gm2d_core::theme::by_id("td").naming;
    // **The fixture's board, not the button's.** These tests want a board that
    // beats a Cave Rat and loses to Francis, which is a statement about one
    // particular arrangement. Auto-pack stopped being an arrangement in M8.8
    // and became a packer, and a packer handed the whole catalogue produces a
    // board that beats everything — which is correct of the packer and useless
    // as a fixture.
    common::build_full_loadout(&mut g.character);
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
    // **Carried, not banked.** A fight pays into your pocket; only a town
    // turns it into a level, so `xp` — which is what has been *spent* — has
    // not moved.
    assert!(won.character.carried > 0, "a win carried no experience");
    assert_eq!(won.character.xp, 0, "a fight levelled the character on the road");
    assert_eq!(won.character.level(), 1, "a fight crossed a level by itself");

    // And a town turns it into levels.
    let held = won.character.carried;
    let b = fight::bank(&mut won);
    assert_eq!(b.spent, held);
    assert_eq!(won.character.carried, 0, "banking left something in the pocket");
    assert_eq!(won.character.xp, held, "banking spent a different number");

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

/// **A defeat takes everything you were carrying, and nothing you had spent.**
///
/// The whole of the souls rule in one test: what you have become is safe, what
/// you were going to become is not.
#[test]
fn dying_costs_what_is_in_your_pocket_and_not_what_you_are() {
    let mut g = facing("Cave Rat");
    let log = fight::run(&g, D).unwrap();
    fight::settle(&mut g, &log, D).unwrap();
    fight::bank(&mut g);
    let spent = g.character.xp;
    let level = g.character.level();
    assert!(spent > 0, "nothing was banked, so this test proves nothing");

    // Win once more, carry it, and then lose.
    g.encounter = Some(gm2d_core::fight::Encounter {
        enemy: "Cave Rat".into(),
        at: g.world.at,
    });
    let log = fight::run(&g, D).unwrap();
    fight::settle(&mut g, &log, D).unwrap();
    let carried = g.character.carried;
    assert!(carried > 0, "the second win carried nothing");

    g.encounter = Some(gm2d_core::fight::Encounter {
        enemy: "Francis".into(),
        at: g.world.at,
    });
    let log = fight::run(&g, D).unwrap();
    let s = fight::settle(&mut g, &log, D).unwrap();
    assert_ne!(s.outcome, Outcome::Victory);
    assert_eq!(g.character.carried, 0, "a defeat left experience in the pocket");
    assert_eq!(g.character.xp, spent, "a defeat took banked experience too");
    assert_eq!(g.character.level(), level, "a defeat cost a level");
    assert!(
        s.receipt.iter().any(|l| l.contains(&carried.to_string())),
        "the receipt does not say how much was lost: {:?}",
        s.receipt
    );
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

/// **The combat log screen's two questions, both of them core's.**
///
/// Ported from the original, where the same two functions answer them:
/// `CombatLog::describe` writes the line and `combat::tally_items` says which
/// lines belong to which item. Both have been in this engine since the fork
/// and neither was read by anything — the interface is the whole of what M13
/// added, so this is what stops it drifting.
#[test]
fn every_line_of_a_fight_has_a_sentence_and_an_owner() {
    use gm2d_core::combat::{self, Event, Side};

    let ch = common::preset_board();
    let spec = combat::LADDER.iter().find(|m| m.name == "Rust Colossus").expect("a long fight");
    let log = combat::simulate_at(ch.player_stats(), &ch.combat_items(), spec, Difficulty::Easy);
    assert!(log.entries.len() > 20, "too short a fight to say anything: {}", log.entries.len());

    // Every entry reads as something. `describe` matches the event enum
    // exhaustively, so a new variant is a compile error there rather than a
    // blank line here — this is the other half: that what it writes is not
    // empty and carries its moment.
    for e in &log.entries {
        let line = log.describe(e);
        assert!(!line.trim().is_empty(), "an entry at {}ms reads as nothing", e.at_ms);
        assert!(
            line.contains('s') || line.starts_with("--"),
            "a line with no moment in it: {line:?}"
        );
    }

    // And every item's own lines are its own. `entries` is what the screen
    // filters on, so an index out of range or an activation missing from it is
    // a rail that lies about what a piece did.
    for side in [Side::Player, Side::Enemy] {
        for t in combat::tally_items(&log, side, 0) {
            for &i in &t.entries {
                assert!(i < log.entries.len(), "{} points at entry {i} of {}", t.name, log.entries.len());
            }
            let acts: Vec<usize> = log
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    matches!(&e.event, Event::Activate { side: s, index, .. }
                             if *s == side && *index == t.index)
                })
                .map(|(i, _)| i)
                .collect();
            for i in &acts {
                assert!(
                    t.entries.contains(i),
                    "{} activated at entry {i} and its own list does not have it",
                    t.name
                );
            }
            assert_eq!(
                t.activations as usize,
                acts.len(),
                "{} counts {} activations and the log holds {}",
                t.name,
                t.activations,
                acts.len()
            );
        }
    }
}
