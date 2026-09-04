//! An event that asks nothing is read once, and its experience lands.
//!
//! Two faults with one cause: **nothing had ever checked what a tile event does
//! after you have read it**, because until M11 every event in the game asked a
//! question and answering one is what marks it.
//!
//! 1. `answer(id, n)` takes the index of the choice you picked, so an event
//!    with no choices could never reach `answered` — it re-opened its modal on
//!    every step onto the tile, for ever. Invisible while the only events were
//!    seven on the overworld that all ask something; M11 put 41 on the
//!    Kettleworks field and 6 on the Reach, none of which do.
//! 2. The experience a choice pays was written into `world.counters["xp"]`,
//!    which nothing has ever read. Reported by the M11.8 playtest, which is a
//!    run by somebody who was not allowed to read this file.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::rng::Rng;
use gm2d_core::world::{self, PlaceKind, WorldState};

mod common;

const D: Difficulty = Difficulty::Easy;

/// Every event tile in the game, as (map, place id, does it ask anything).
fn events() -> Vec<(String, String, bool)> {
    let defs = data::events();
    let mut out = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            if matches!(p.kind, PlaceKind::Event) {
                let asks = defs.get(&p.id).map(|e| !e.choices.is_empty()).unwrap_or(true);
                out.push((id.to_string(), p.id.clone(), asks));
            }
        }
    }
    out
}

/// The premise: there are events that ask nothing, and this suite is theirs.
///
/// **It used to say *most* events ask nothing, and M12.5 retires that clause
/// on purpose.** Its own message said "if this drops, re-read what the suite
/// is for" — so: the suite is about what an examinable *does*, which is to be
/// read once and then go quiet, and that has not changed and is still checked
/// below. What changed is the balance. M12.0 measured what the old one cost —
/// 0 decisions against 41 notes on the Kettleworks field, and events paying no
/// gear anywhere in the game — and `events_pay.rs::a_map_is_not_mostly_
/// wallpaper` now wants the other answer.
///
/// The half that matters is kept: **there are still examinables**, because a
/// map with none is a map with no furniture, and a suite about furniture with
/// nothing to test would pass by being empty.
#[test]
fn there_are_events_that_ask_nothing() {
    let all = events();
    let quiet = all.iter().filter(|(_, _, asks)| !asks).count();
    assert!(quiet > 0, "every event asks something; this suite is about the ones that do not");
    assert!(quiet >= 10, "only {quiet} of {} are furniture", all.len());
}

/// **Read once, then quiet.**
///
/// Walk onto the tile twice. The first step reports the event; the second
/// reports nothing at all, because a page cannot decline to draw a card it was
/// handed — so the not-drawing is core's.
#[test]
fn an_event_that_asks_nothing_is_read_once() {
    for (map, id, asks) in events() {
        if asks {
            continue;
        }
        let w = data::map(&map, D);
        let Some(place) = w.places.iter().find(|p| p.id == id) else { continue };
        let at = place.at;

        let mut state = WorldState::at_start(&w);
        let mut rng = Rng::new(7);
        let allowed = world::Allowances::default();

        // Stand next to it and step on.
        let first = step_onto(&w, &mut state, &mut rng, &allowed, at);
        assert_eq!(
            first.as_deref(),
            Some(id.as_str()),
            "{map}/{id} did not report itself the first time"
        );
        assert!(
            state.answered.iter().any(|a| *a == id),
            "{map}/{id} was read and not written down"
        );

        // Walk off and back on.
        let again = step_onto(&w, &mut state, &mut rng, &allowed, at);
        assert_eq!(again, None, "{map}/{id} opened itself again on a second visit");
    }
}

/// An event that *does* ask something still comes back, so you can re-read it
/// and see that it is spent. That is nine events on two hand-built maps and it
/// is deliberate — the change is about the ones with nothing to answer.
#[test]
fn an_event_that_asks_something_still_reopens() {
    let mut checked = 0;
    for (map, id, asks) in events() {
        if !asks {
            continue;
        }
        let w = data::map(&map, D);
        let Some(place) = w.places.iter().find(|p| p.id == id) else { continue };
        let at = place.at;
        let mut state = WorldState::at_start(&w);
        // Pretend it was answered.
        state.answered.push(id.clone());
        let mut rng = Rng::new(7);
        let allowed = world::Allowances::default();
        let seen = step_onto(&w, &mut state, &mut rng, &allowed, at);
        assert_eq!(
            seen.as_deref(),
            Some(id.as_str()),
            "{map}/{id} asks something and stopped reporting itself"
        );
        checked += 1;
    }
    assert!(checked > 0, "no event asks anything any more");
}

/// **What an event pays lands where a fight's pay lands.**
///
/// `carry`, not `gain_xp`: nothing on the road spends, and a town is the only
/// thing that turns carried into a level.
#[test]
fn the_experience_an_event_promises_is_carried() {
    let defs = data::events();
    let mut promised = 0;
    for e in &defs.events {
        for c in &e.choices {
            promised += xp_in(&c.outcome);
        }
    }
    assert!(
        promised > 0,
        "no event pays experience any more; this test is about the ones that do"
    );

    // And the number a player would actually collect is not zero, which is what
    // it was: the old code put it in `world.counters[\"xp\"]` and nothing has
    // ever read that.
    let mut ch = gm2d_core::character::Character::starting();
    ch.apply_preset();
    assert_eq!(ch.carried, 0);
    ch.carry(promised);
    assert_eq!(ch.carried, promised, "carrying is what an event's payout does now");
    // Spending it is a town's, and it crosses levels.
    let crossed = ch.bank();
    assert!(!crossed.is_empty(), "{promised} experience banked no level at all");
}

fn xp_in(o: &gm2d_core::tile_event::Outcome) -> i32 {
    use gm2d_core::tile_event::Outcome;
    match o {
        Outcome::Xp(n) => *n,
        Outcome::All(list) => list.iter().map(xp_in).sum(),
        _ => 0,
    }
}

/// Put the player next to `at`, then step onto it. Returns the event reported.
fn step_onto(
    w: &world::World,
    state: &mut WorldState,
    rng: &mut Rng,
    allowed: &world::Allowances,
    at: [u8; 2],
) -> Option<String> {
    // Stand on a neighbour that is walkable, and step in from it.
    let (x, y) = (at[0] as i32, at[1] as i32);
    for (dx, dy, dir) in [
        (-1i32, 0i32, world::Dir::East),
        (1, 0, world::Dir::West),
        (0, -1, world::Dir::South),
        (0, 1, world::Dir::North),
    ] {
        let (nx, ny) = (x + dx, y + dy);
        if nx < 0 || ny < 0 || nx >= w.width as i32 || ny >= w.height as i32 {
            continue;
        }
        if !w.walkable(nx as u8, ny as u8, allowed) {
            continue;
        }
        state.at = [nx as u8, ny as u8];
        let s = world::step(w, state, rng, D, dir, allowed);
        assert_eq!(state.at, at, "the step did not land on the tile under test");
        return s.event;
    }
    panic!("nothing walkable next to {at:?}");
}
