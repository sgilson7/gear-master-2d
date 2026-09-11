//! The door a choice that did nothing could shut for good.
//!
//! Reported from play, standing in it:
//!
//! > at the weighted door i couldn't do anything and selected the bottom
//! > option, can i still open the door to the next floor using the lintel? how
//! > do i access that?
//!
//! The answer was **no**, and that is a soft-lock in shipped content. The
//! Wextreen Sump's Shelf has four choices — three open the door and the fourth
//! is *Try the slot as you are*, which costs a minute and tells you the shape.
//! Taking it wrote the event into `answered` and the stair out is
//! `hidden_until: the-shelf-is-open`.
//!
//! **It is the hole in the monotone argument.** `CLAUDE.md` says a puzzle is
//! safe because `flags` and `answered` only ever *grow* — and growing is
//! exactly what did it. The guarantee is that you cannot lose a flag; it was
//! read as a guarantee that you cannot lose a **door**.
//! `every_floor_in_the_game_can_be_solved_blind` could not see it either,
//! because a blind solver never takes a choice that does nothing: it is the one
//! move a model of a good player will not make.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::tile_event::Outcome;
use gm2d_core::world::{self, Allowances, WorldState};

const D: Difficulty = Difficulty::Easy;
const DOOR: &str = "the-weighed-door";
const OPEN: &str = "the-shelf-is-open";

fn at_the_door() -> Game {
    let mut g = Game::new(5, "td");
    g.world.map = "the-sump-2".into();
    g.world.at = [5, 1];
    g
}

/// The way through this character can actually take, found rather than written
/// down — the reporter's own route is the lintel, which wants `read-the-shelf`
/// rather than a three-by-two in the bag.
fn a_way_through(g: &Game) -> usize {
    let events = data::events();
    let e = events.get(DOOR).expect("the weighed door");
    e.choices
        .iter()
        .position(|c| {
            gm2d_core::tile_event::flags_set_by(&c.outcome).iter().any(|f| f == OPEN)
                && g.can_take(c)
        })
        .expect("nothing at this door is takeable")
}

/// Which choice does nothing, found rather than written down.
fn the_no_op() -> usize {
    let events = data::events();
    let e = events.get(DOOR).expect("the weighed door");
    e.choices
        .iter()
        .position(|c| matches!(c.outcome, Outcome::Nothing))
        .expect("the weighed door has no choice that does nothing, so this file is stale")
}

/// **Taking the one that does nothing leaves the door open.**
#[test]
fn the_bottom_option_does_not_shut_the_door() {
    let mut g = at_the_door();
    let n = the_no_op();
    g.answer_event(DOOR, n, D).expect("the bottom option can be taken");
    assert!(
        !g.world.answered.iter().any(|a| a == DOOR),
        "a choice that changed nothing spent the card"
    );
    // And the door still opens, by the way the reporter asked about.
    g.world.flags.push("read-the-shelf".into());
    let lintel = a_way_through(&g);
    g.answer_event(DOOR, lintel, D).expect("the door still opens after the no-op");
    assert!(g.world.flags.iter().any(|f| f == OPEN), "the door did not open");
}

/// And taking it twice is still not an answer.
///
/// **Check the second visit.** A no-op that did not spend the card must not
/// start spending it on the way round again.
#[test]
fn the_bottom_option_can_be_taken_twice() {
    let mut g = at_the_door();
    let n = the_no_op();
    for _ in 0..3 {
        g.answer_event(DOOR, n, D).expect("it can be taken again");
    }
    assert!(g.world.answered.is_empty());
}

/// A choice that *does* something still spends the card.
#[test]
fn a_choice_that_does_something_still_spends_it() {
    let mut g = at_the_door();
    g.world.flags.push("read-the-shelf".into());
    let lintel = a_way_through(&g);
    g.answer_event(DOOR, lintel, D).unwrap();
    assert!(g.world.answered.iter().any(|a| a == DOOR), "the card was not spent");
    assert_eq!(
        g.answer_event(DOOR, lintel, D),
        Err("already answered".into()),
        "a spent card answered twice"
    );
}

// -------------------------------------------------------------- the repair

/// **A save that is already stuck comes unstuck on load.**
///
/// Fixing `answer_event` stops it happening again and does nothing for a save
/// it has already happened to — and somebody is standing in one.
#[test]
fn a_save_shut_by_the_no_op_is_reopened() {
    let mut st = WorldState::default();
    st.map = "the-sump-2".into();
    st.answered.push(DOOR.to_string());

    let w = data::map("the-sump-2", D);
    let stair = w.places.iter().find(|p| p.id == "the-sump-2-stair").expect("the stair out");
    let allowed = Allowances::default();
    assert!(
        !world::place_is_there(stair, &st, &allowed),
        "the stair is there without the flag, so this proves nothing"
    );

    world::reopen_doors_a_no_op_shut(&mut st, &data::events());
    assert!(
        !st.answered.iter().any(|a| a == DOOR),
        "the door was left shut and the floor is still unfinishable"
    );
}

/// **And a door that was actually opened is left alone**, or the repair is a
/// way to take a card twice.
#[test]
fn a_door_that_was_opened_is_not_reopened() {
    let mut st = WorldState::default();
    st.answered.push(DOOR.to_string());
    st.flags.push(OPEN.to_string());
    world::reopen_doors_a_no_op_shut(&mut st, &data::events());
    assert!(
        st.answered.iter().any(|a| a == DOOR),
        "an answered door with its flag set was handed back"
    );
}

/// An ordinary card with no no-op choice is never touched.
#[test]
fn an_ordinary_card_is_never_reopened() {
    let events = data::events();
    let ordinary = events
        .events
        .iter()
        .find(|e| {
            !e.choices.is_empty()
                && !e.choices.iter().any(|c| matches!(c.outcome, Outcome::Nothing))
                && e.choices
                    .iter()
                    .any(|c| !gm2d_core::tile_event::flags_set_by(&c.outcome).is_empty())
        })
        .expect("no ordinary flag-setting card in the game");
    let mut st = WorldState::default();
    st.answered.push(ordinary.id.clone());
    world::reopen_doors_a_no_op_shut(&mut st, &events);
    assert!(
        st.answered.iter().any(|a| a == &ordinary.id),
        "{} was reopened and it has no way to be shut wrongly",
        ordinary.id
    );
}

// ----------------------------------------------------------------- the lint

/// **No choice that does nothing may spend a door**, over every event there is.
///
/// The rule rather than the instance, because a list of one written by hand is
/// a list that becomes two the next time somebody authors an escape hatch onto
/// a card that opens something.
#[test]
fn no_choice_that_does_nothing_spends_a_door() {
    let events = data::events();
    let mut doors = 0;
    for e in &events.events {
        let no_ops: Vec<&str> = e
            .choices
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Nothing))
            .map(|c| c.label.as_str())
            .collect();
        if no_ops.is_empty() {
            continue;
        }
        let opens: Vec<String> = e
            .choices
            .iter()
            .flat_map(|c| gm2d_core::tile_event::flags_set_by(&c.outcome))
            .collect();
        if opens.is_empty() {
            continue;
        }
        doors += 1;
        // Take the no-op and the door must still open.
        for label in &no_ops {
            let n = e.choices.iter().position(|c| c.label == *label).unwrap();
            let mut g = Game::new(1, "td");
            g.answer_event(&e.id, n, D).expect("the no-op can be taken");
            assert!(
                !g.world.answered.iter().any(|a| a == &e.id),
                "{}: {label:?} does nothing and spent the card, which opens {opens:?}",
                e.id
            );
        }
    }
    assert!(doors > 0, "no event in the game is a door with an escape hatch, so this is vacuous");
}
