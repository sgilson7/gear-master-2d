//! A key is spent opening its lock, and the lock stays open.
//!
//! Two keys exist — the Witch's Key at the Cave mouth and the Deep Gate Key at
//! the door in the wall — and both used to sit in the bag for ever, because the
//! bag was only ever *asked* about them. Reported by the human.
//!
//! **The second half of the rule is what stops a soft-lock.** The door in the
//! wall is the only way to the back half of the game and a defeat in the
//! Treyway walks you home to West Bambulon; a key that were spent *and*
//! re-locked would end the run there, and there is no second key.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::{Game, Unlocked};
use gm2d_core::world::{PlaceKind, WorldState};

mod common;

const D: Difficulty = Difficulty::Easy;

/// Every place in the game that wants a key, by map.
fn locks() -> Vec<(String, gm2d_core::world::PlaceDef)> {
    let mut out = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            if p.needs.is_some() {
                out.push((id.to_string(), p));
            }
        }
    }
    out
}

fn at_start() -> Game {
    let mut g = Game::new(0xC0FFEE, "td");
    g.world = WorldState::at_start(&data::world(D));
    g
}

/// The premise: there are locks, and they want components that exist.
#[test]
fn every_lock_wants_something_the_catalogue_has() {
    let locks = locks();
    assert!(!locks.is_empty(), "no place in the game wants a key any more");
    for (map, p) in &locks {
        let key = p.needs.as_deref().expect("filtered on needs");
        assert!(
            gm2d_core::piece::CATALOG.iter().any(|d| d.name == key),
            "{map}/{} wants {key}, which is not in the catalogue",
            p.id
        );
        assert!(
            matches!(p.kind, PlaceKind::Gate | PlaceKind::Door),
            "{map}/{} is a {:?} and wants a key",
            p.id,
            p.kind
        );
    }
}

/// **The key leaves the bag, once, and the lock stays open.**
#[test]
fn a_key_is_spent_and_the_lock_stays_open() {
    for (map, p) in locks() {
        let key = p.needs.clone().expect("filtered on needs");
        let mut g = at_start();

        // Shut, before you have it, and holding nothing is not spending
        // anything.
        assert_eq!(g.unlock(&p), Unlocked::Shut, "{map}/{} opened with no key", p.id);
        assert_eq!(gm2d_core::quest::holding(&g, &key), 0);

        g.character.give(&key).unwrap_or_else(|| panic!("{key} is in the catalogue"));
        assert_eq!(gm2d_core::quest::holding(&g, &key), 1);

        // It turns, and it is gone.
        assert_eq!(
            g.unlock(&p),
            Unlocked::Spent { key: key.clone() },
            "{map}/{} did not spend the key",
            p.id
        );
        assert_eq!(
            gm2d_core::quest::holding(&g, &key),
            0,
            "{map}/{} kept {key} in the bag",
            p.id
        );

        // And it never wants one again. This is the half that stops the run
        // ending: there is no second key anywhere in the game.
        assert_eq!(g.unlock(&p), Unlocked::Already, "{map}/{} re-locked itself", p.id);
        assert_eq!(g.unlock(&p), Unlocked::Already, "and again");
    }
}

/// **A key is carried, never worn**, so turning it only ever touches the bag.
///
/// Both keys are `PieceKind::Quest`, which `can_equip` refuses outright — *"that
/// is a quest item - it is carried, not worn"*. Pinned because it is the reason
/// spending one cannot leave a cell occupied by something that is gone, and
/// because a key that became wearable would need that to be true again.
///
/// `Character::spend_one` takes it off a board first regardless, which is what
/// an errand's tally needs: those *are* seatable, and handing one in over the
/// counter while it sits in a cell would be a component in two places.
#[test]
fn a_key_is_carried_and_not_worn() {
    for (map, p) in locks() {
        let key = p.needs.clone().expect("filtered on needs");
        let mut g = at_start();
        g.character.grow_boards(20);
        for k in gm2d_core::piece::SlotKind::ALL {
            g.character.loadout.slot_mut(k).clear();
        }
        let id = g.character.give(&key).expect("in the catalogue");
        for k in gm2d_core::piece::SlotKind::ALL {
            assert!(
                g.character.equip(id, k, 0, 0).is_err(),
                "{map}: {key} sat down on a {k:?} board"
            );
        }
        assert!(matches!(g.unlock(&p), Unlocked::Spent { .. }));
        assert_eq!(gm2d_core::quest::holding(&g, &key), 0);
        assert!(
            !g.character.loadout.slots.iter().any(|s| s.contains(id)),
            "{map}: the key turned and something is still holding it"
        );
    }
}


/// **An instrument is asked for every time and is never spent.**
///
/// The Reach's whole design is that what you carry changes what you read, so a
/// survey gate must never be remembered — and there is nothing in the bag to
/// take, because an instrument is assembled on a board.
#[test]
fn a_survey_gate_is_a_standing_question_and_not_a_key() {
    let mut found = 0;
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            if !p.needs_survey {
                continue;
            }
            found += 1;
            assert!(p.needs.is_none(), "{id}/{} wants a key as well as an instrument", p.id);
            let mut g = at_start();
            assert_eq!(g.unlock(&p), Unlocked::Shut, "{id}/{} opened with no instrument", p.id);
            assert!(
                !g.world.answered.iter().any(|a| *a == p.id),
                "a refused survey gate remembered itself"
            );
        }
    }
    assert!(found > 0, "no gate wants an instrument any more");
}

/// A place that wants nothing is open and is not written down.
#[test]
fn a_lock_that_wants_nothing_remembers_nothing() {
    let mut g = at_start();
    let plain = data::map(&gm2d_core::world::overworld(), D)
        .places
        .into_iter()
        .find(|p| matches!(p.kind, PlaceKind::Gate) && p.needs.is_none() && !p.needs_survey)
        .expect("an unlocked gate somewhere");
    assert_eq!(g.unlock(&plain), Unlocked::Open);
    assert!(g.world.answered.is_empty(), "an open gate wrote itself down");
}

/// Spending a key does not spend anything else.
#[test]
fn turning_a_key_leaves_the_rest_of_the_bag_alone() {
    let (_, p) = locks().into_iter().next().expect("a lock");
    let key = p.needs.clone().expect("filtered on needs");
    let mut g = at_start();
    g.character.give(&key).expect("in the catalogue");
    g.character.give(&key).expect("a second one");
    let before = g.character.owned.len();
    assert!(matches!(g.unlock(&p), Unlocked::Spent { .. }));
    assert_eq!(g.character.owned.len(), before - 1, "it took more than one thing");
    // A second copy is still yours. The lock is what stops wanting one, not
    // the bag that stops holding one.
    assert_eq!(gm2d_core::quest::holding(&g, &key), 1);
}

/// The starting character opens nothing.
#[test]
fn a_new_character_carries_no_keys() {
    let mut ch = gm2d_core::character::Character::starting();
    ch.apply_preset();
    let mut g = Game::new(1, "td");
    g.character = ch;
    for (map, p) in locks() {
        let key = p.needs.as_deref().expect("filtered on needs");
        assert_eq!(
            gm2d_core::quest::holding(&g, key),
            0,
            "a new character starts holding {key} for {map}/{}",
            p.id
        );
    }
}
