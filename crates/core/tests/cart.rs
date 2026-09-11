//! The long cart: a way between towns you have stood in.
//!
//! Reported from play: *"there should be a way to teleport between towns"*.

mod common;

use gm2d_core::combat::Difficulty;
use gm2d_core::game::{CartStop, Game, CART_FARE};

const D: Difficulty = Difficulty::Easy;

fn in_the_pit() -> Game {
    let mut g = Game::new(5, "td");
    g.world.map = "west-bambulon".into();
    g.character.gold = 1_000;
    g
}

/// **The cart runs to towns you have been, and nowhere else.**
///
/// The requirement is the whole design: a coach line to a counter you have
/// never seen would be a map handed to you rather than walked. Derived off the
/// counters, so nothing writes a second copy of *where you have been*.
#[test]
fn the_cart_runs_only_where_you_have_been() {
    let mut g = in_the_pit();
    assert!(g.cart_stops(D).is_empty(), "a fresh run can already ride somewhere");

    g.arrive_in_town("the-end-of-all-gears");
    // Standing in the only town you have been is nowhere to go.
    g.world.at = pit(&g);
    assert!(
        g.cart_stops(D).is_empty(),
        "the cart offers the counter you are leaning on"
    );

    // Walked on to the second town, which is where you would board.
    g.arrive_in_town("kettleworks");
    g.world.map = "kettleworks-field".into();
    g.world.at = [0, 0];
    let stops: Vec<String> = g.cart_stops(D).iter().map(|s| s.id.clone()).collect();
    assert!(
        stops.contains(&"the-end-of-all-gears".to_string()),
        "stood in two towns and the cart offers {stops:?}"
    );
}

fn pit(g: &Game) -> [u8; 2] {
    gm2d_core::data::map("west-bambulon", D)
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Town)
        .expect("the pit has a town")
        .at
        .clone()
}

/// **It charges, and a refusal spends nothing.**
///
/// The first thing anybody does with a refused button is press it again —
/// which is the reroll's rule and the bank's, pinned here for the third time.
#[test]
fn the_fare_is_taken_once_and_a_refusal_takes_nothing() {
    let mut g = in_the_pit();
    g.arrive_in_town("the-end-of-all-gears");
    g.arrive_in_town("kettleworks");
    g.world.map = "kettleworks-field".into();

    g.character.gold = CART_FARE - 1;
    let before = g.character.gold;
    let why = g
        .take_the_cart("the-end-of-all-gears", D)
        .expect_err("rode with no fare");
    assert!(why.contains(&CART_FARE.to_string()), "the refusal does not say the fare: {why}");
    assert_eq!(g.character.gold, before, "a refused fare was taken anyway");

    g.character.gold = CART_FARE + 7;
    let stop: CartStop = g.take_the_cart("the-end-of-all-gears", D).expect("the fare was there");
    assert_eq!(g.character.gold, 7, "the fare was not {CART_FARE}");
    assert_eq!(g.world.map_id(), stop.map);
    assert_eq!(g.world.at, stop.at, "the cart put you somewhere else");
}

/// **A town the cart does not serve is refused by name.**
#[test]
fn the_cart_refuses_a_town_you_have_not_seen() {
    let mut g = in_the_pit();
    g.arrive_in_town("the-end-of-all-gears");
    g.arrive_in_town("kettleworks");
    g.world.map = "kettleworks-field".into();
    let why = g.take_the_cart("high-wick", D).expect_err("rode to a town on no map");
    assert!(!why.is_empty(), "it refused in silence");
    assert_eq!(g.character.gold, 1_000, "a refused ride was charged for");
}

/// **It does not undercut the Drover's Stride, and this is why.**
///
/// The Stride's job is getting you *out of the wilderness*: it works from
/// anywhere and costs a tin. The cart runs town to town, so you have to
/// already be somewhere safe to take it — it can never be the thing that saves
/// a run, which is the whole reason a second kind of travel is allowed to
/// exist at all. `a_warp_is_never_a_way_home` makes the same argument about
/// events and this is its third reader.
///
/// Stated as a property rather than as prose: every stop the cart offers is a
/// town, so every ride begins and ends at a counter.
#[test]
fn every_ride_begins_and_ends_at_a_counter() {
    let mut g = in_the_pit();
    for t in ["the-end-of-all-gears", "kettleworks"] {
        g.arrive_in_town(t);
    }
    // Out in the wilds, where the Stride is what you want.
    g.world.map = "the-treyway".into();
    g.world.at = [8, 8];

    // The cart still lists its stops — it is a timetable, not a rule about
    // where you are — and what stops it being a rescue is that you board it in
    // a town. The shim only offers the button on the town screen, and the
    // engine's half is that every stop is a town tile.
    for stop in g.cart_stops(D) {
        let w = gm2d_core::data::map(&stop.map, D);
        let p = w
            .place_at(stop.at[0], stop.at[1])
            .unwrap_or_else(|| panic!("{}: the cart stops at a tile with nothing on it", stop.id));
        assert_eq!(
            p.kind,
            gm2d_core::world::PlaceKind::Town,
            "{}: the cart stops somewhere that is not a town",
            stop.id
        );
    }
}

/// **No save seam.** Where you have been is a counter, and counters already
/// round-trip — so a file written before the cart existed opens with no stops,
/// which is what that character had.
#[test]
fn where_you_have_been_is_a_counter_and_needs_no_field() {
    let mut g = in_the_pit();
    g.arrive_in_town("the-end-of-all-gears");
    let json = gm2d_core::save::SaveFile::of(&g).to_json();
    let back = gm2d_core::save::load(&json).expect("loads");
    assert_eq!(
        back.world.count(&gm2d_core::game::stood_key("the-end-of-all-gears")),
        1,
        "the ride home was forgotten across a save"
    );
}
