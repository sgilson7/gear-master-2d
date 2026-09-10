//! The cart that is somewhere else tomorrow.
//!
//! Asked for as *"there should also be a traveling caravan in the kettleworks
//! that appears in a random area for 5 movements then telepots to another spot
//! in the kettleworks (ones not already covered in something else) that sells
//! survey gear like magnets, lenses, etc."*
//!
//! **A place that moves cannot be a place spawned at runtime.** `places are
//! content and content is not state` is why `PlaceDef::hidden_until` exists at
//! all, so the cart is authored as eight stops on empty ground and
//! `WorldState::caravan` says which one it is at. That is the whole design and
//! every test here is a consequence of it.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::rng::Rng;
use gm2d_core::shop;
use gm2d_core::world::{self, Allowances, Dir, PlaceKind, World, WorldState, CARAVAN_STAY};

mod common;

const D: Difficulty = Difficulty::Easy;
const FIELD: &str = "kettleworks-field";

fn field() -> World {
    data::map(FIELD, D)
}

fn stops(w: &World) -> Vec<&gm2d_core::world::PlaceDef> {
    w.places.iter().filter(|p| p.kind == PlaceKind::Caravan).collect()
}

fn on_the_field() -> (World, WorldState) {
    let w = field();
    let mut st = WorldState::at_start(&w);
    st.map = FIELD.into();
    // Somewhere in the middle with room to walk.
    st.at = [10, 10];
    (w, st)
}

// ------------------------------------------------------------------ the file

/// **The stops parsed, and they are on ground nothing else is using.**
///
/// *"ones not already covered in something else"* is the ask, and it is a fact
/// about the map file rather than about the movement rule — so it is checked
/// where it can be, over every stop rather than over the ones I happened to
/// pick.
#[test]
fn every_stop_is_on_empty_walkable_ground() {
    let w = field();
    let all = stops(&w);
    assert!(all.len() >= 4, "{} stops is not a cart that travels", all.len());
    for s in &all {
        assert!(
            w.walkable(s.at[0], s.at[1], &Allowances::default()),
            "{} is on {}, which nobody can stand on",
            s.id,
            w.terrain_name(s.at[0], s.at[1])
        );
        let others: Vec<&str> = w
            .places
            .iter()
            .filter(|p| p.at == s.at && p.id != s.id)
            .map(|p| p.id.as_str())
            .collect();
        assert!(others.is_empty(), "{} shares its tile with {others:?}", s.id);
    }
    // And no two stops are the same tile, or the cart has fewer places to be
    // than the file claims.
    let mut seen: Vec<[u8; 2]> = all.iter().map(|p| p.at).collect();
    seen.sort();
    let before = seen.len();
    seen.dedup();
    assert_eq!(before, seen.len(), "two stops are on one tile");
}

/// **Only one map has a cart**, so nothing else on the map list quietly grew one.
#[test]
fn the_cart_travels_one_country() {
    let mut carrying = Vec::new();
    for (id, _) in data::MAPS {
        if !stops(&data::map(id, D)).is_empty() {
            carrying.push(id);
        }
    }
    assert_eq!(carrying, vec![&FIELD], "the cart is on {carrying:?}");
}

// -------------------------------------------------------------- the movement

/// It is placed on the first step onto the field, and only one stop is there.
#[test]
fn the_cart_is_somewhere_and_only_somewhere() {
    let (w, mut st) = on_the_field();
    assert!(st.caravan.is_none(), "a fresh state already has the cart placed");
    let mut rng = Rng::new(7);
    world::step(&w, &mut st, &mut rng, D, Dir::East, &Allowances::default());
    let at = st.caravan.clone().expect("the first step did not place the cart");
    assert_eq!(at.moves_left, CARAVAN_STAY);

    // **Exactly one of the eight is there.** The others are bare ground, which
    // is what makes a moving place content.
    let allowed = Allowances::default();
    let there: Vec<&str> = stops(&w)
        .into_iter()
        .filter(|p| world::place_is_there(p, &st, &allowed))
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(there, vec![at.place.as_str()]);
}

/// **Five of your steps, then it is somewhere else.**
#[test]
fn it_stays_five_steps_and_then_moves() {
    let (w, mut st) = on_the_field();
    let mut rng = Rng::new(11);
    let allowed = Allowances::default();
    // The placing step, then it has its full stay ahead of it.
    world::step(&w, &mut st, &mut rng, D, Dir::East, &allowed);
    let first = st.caravan.clone().unwrap();
    assert_eq!(first.moves_left, CARAVAN_STAY);

    let mut moved_on = None;
    for i in 1..=CARAVAN_STAY {
        let d = if i % 2 == 0 { Dir::East } else { Dir::West };
        world::step(&w, &mut st, &mut rng, D, d, &allowed);
        let now = st.caravan.clone().unwrap();
        if now.place != first.place {
            moved_on = Some(i);
            break;
        }
        assert_eq!(
            now.moves_left,
            CARAVAN_STAY - i,
            "step {i} did not take one off the clock"
        );
    }
    assert_eq!(
        moved_on,
        Some(CARAVAN_STAY),
        "the cart moved on step {moved_on:?} and the stay is {CARAVAN_STAY}"
    );
    assert_eq!(st.caravan.unwrap().moves_left, CARAVAN_STAY, "it arrived part-way through a stay");
}

/// **It never moves to where it already is**, or a teleport is a thing nobody
/// can tell happened.
#[test]
fn it_never_teleports_onto_itself() {
    let w = field();
    for seed in [1u64, 2, 3, 5, 8, 13, 21] {
        let mut st = WorldState::at_start(&w);
        st.map = FIELD.into();
        st.at = [10, 10];
        let mut rng = Rng::new(seed);
        let allowed = Allowances::default();
        let mut last: Option<String> = None;
        for i in 0..40 {
            let d = if i % 2 == 0 { Dir::East } else { Dir::West };
            world::step(&w, &mut st, &mut rng, D, d, &allowed);
            let now = st.caravan.clone().unwrap().place;
            if let Some(prev) = &last {
                if st.caravan.as_ref().unwrap().moves_left == CARAVAN_STAY && *prev == now {
                    panic!("seed {seed}: the cart moved from {prev} to itself");
                }
            }
            last = Some(now);
        }
    }
}

/// **And never onto the tile you are standing on**, which would open its screen
/// without being walked to.
///
/// **The first version of this was vacuous and a negative test found it.**
/// It walked the player back and forth over one stop for sixty steps on one
/// seed and asserted the cart was never on their tile — and with the guard
/// deleted it still passed, because over that walk the cart never happened to
/// pick that stop. A check that never gives the fault a chance to happen is a
/// check that compares zero with zero.
///
/// So it counts the chances it gave, and refuses to pass without enough of
/// them: a move that lands while the player is standing on a stop is the only
/// moment this rule is about.
#[test]
fn it_never_lands_under_your_feet() {
    let w = field();
    let all = stops(&w);
    let allowed = Allowances::default();
    let mut chances = 0;
    for seed in [4u64, 9, 16, 25, 36, 49, 64, 81, 100, 121] {
        // Stand *on* a stop and step off and back, so half of every arrival
        // happens while the player is on a tile the cart could pick.
        let stand = all[seed as usize % all.len()].at;
        let mut st = WorldState::at_start(&w);
        st.map = FIELD.into();
        st.at = stand;
        let mut rng = Rng::new(seed);
        for i in 0..80 {
            let d = if i % 2 == 0 { Dir::East } else { Dir::West };
            world::step(&w, &mut st, &mut rng, D, d, &allowed);
            let Some(c) = &st.caravan else { continue };
            // A full clock means it arrived on this step, which is the only
            // moment the rule is about.
            if c.moves_left != CARAVAN_STAY {
                continue;
            }
            if all.iter().any(|p| p.at == st.at) {
                chances += 1;
            }
            let at = all.iter().find(|p| p.id == c.place).map(|p| p.at);
            assert_ne!(at, Some(st.at), "seed {seed}: the cart teleported under the player");
        }
    }
    assert!(
        chances >= 10,
        "the cart only arrived {chances} times while the player stood on a stop, so this \
         check never gave the fault a chance to happen"
    );
}

/// A save naming a stop this build has not got is repaired rather than left
/// dangling — the answer `World::repair` gives a position nobody can stand on.
#[test]
fn a_stop_this_build_has_not_got_is_repaired() {
    let (w, mut st) = on_the_field();
    st.caravan = Some(gm2d_core::world::CaravanAt {
        place: "the-caravan-stop-from-another-build".into(),
        moves_left: 3,
    });
    let mut rng = Rng::new(2);
    world::step(&w, &mut st, &mut rng, D, Dir::East, &Allowances::default());
    let now = st.caravan.clone().expect("the cart was dropped rather than repaired");
    assert!(
        stops(&w).iter().any(|p| p.id == now.place),
        "repaired onto {}, which is not a stop",
        now.place
    );
}

/// **Its clock does not run while you are somewhere else.** Five movements is
/// five you could have watched.
#[test]
fn the_clock_only_runs_on_the_field() {
    let (w, mut st) = on_the_field();
    let mut rng = Rng::new(6);
    world::step(&w, &mut st, &mut rng, D, Dir::East, &Allowances::default());
    let before = st.caravan.clone().unwrap();

    // Walk a long way about on a map with no stops on it.
    let other = data::map("west-bambulon", D);
    st.map = "west-bambulon".into();
    st.at = [4, 16];
    for i in 0..20 {
        let d = if i % 2 == 0 { Dir::East } else { Dir::West };
        world::step(&other, &mut st, &mut rng, D, d, &Allowances::default());
    }
    assert_eq!(st.caravan.unwrap(), before, "the cart moved while you were two maps away");
}

// ------------------------------------------------------------------ the stock

/// **Everything on the cart goes on the instrument frame.**
///
/// The ask names *survey gear*, and the whole reason it exists is that the
/// cheapest instrument had a part nothing in the game sold. A cart that had
/// drifted into selling greaves would be a fourth counter rather than the
/// answer to that.
#[test]
fn the_cart_sells_survey_gear_and_only_that() {
    let shops = data::shops();
    let rows = shop::caravan_shelf(&shops, &[]);
    assert!(!rows.is_empty(), "the cart is empty");
    for o in &rows {
        assert!(
            o.def.fits(gm2d_core::piece::SlotKind::Instrument),
            "the cart is selling {}, which does not go on an instrument",
            o.def.name
        );
        assert!(o.price > 0, "{} is free", o.def.name);
    }
}

/// **The compass can be finished off the cart alone.**
///
/// This is the measurement the cart exists for. Before it, the Glass Lens was
/// off an Ember Wisp and the Magnet off a Slag Warden and **nothing in the game
/// sold either** — twenty wins in the Kolok Downs for a Magnet, and the Kolok
/// Downs is behind the door the instrument opens.
#[test]
fn an_instrument_can_be_finished_off_the_cart_alone() {
    let shops = data::shops();
    let carried: Vec<&str> =
        shop::caravan_shelf(&shops, &[]).iter().map(|o| o.def.name).collect();
    let mut ch = common::bench();
    for name in &carried {
        ch.give(name);
    }
    // **Seated by hand on the instrument frame, and Auto-pack cannot do it.**
    // `SlotKind::Instrument` is deliberately outside `SlotKind::ALL` so that
    // nothing which asks what a board is *worth* ever counts it — and
    // `pack_what_you_own` walks `ALL`, so it packed the orb and the alignment
    // into the weapon grid and made nineteen items and no compass. That is the
    // sixth grid working as designed, and it is why the door opens the frame
    // for you rather than expecting the button to.
    for (name, x, y) in [("Map Shard", 0u8, 0u8), ("Glass Lens", 2, 0), ("Magnet", 0, 1)] {
        assert!(carried.contains(&name), "the cart no longer sells a {name}");
        let id = common::spare(&ch, name);
        ch.registry.set_rotation(id, 0);
        ch.equip(id, gm2d_core::piece::SlotKind::Instrument, x, y)
            .unwrap_or_else(|e| panic!("{name} would not seat on the frame: {e}"));
    }
    gm2d_core::loadout::lock_assembled_in(
        &mut ch.loadout,
        &ch.registry,
        gm2d_core::piece::SlotKind::Instrument,
    );
    assert!(
        ch.rules().iter().any(|r| matches!(r, gm2d_core::rule::Rule::Survey { .. })),
        "the three parts of a compass, all off the cart, and no instrument came out: {carried:?}"
    );
}

/// A line is spent once each, and the cart moving is not restocking it.
#[test]
fn what_you_bought_off_it_follows_it() {
    let shops = data::shops();
    let sold = vec![(shop::CARAVAN.to_string(), 0u16)];
    let rows = shop::caravan_shelf(&shops, &sold);
    assert!(rows[0].sold, "the first line is not spent");
    assert!(!rows[1].sold, "buying one line spent another");
    // Keyed by the cart and not by a stop, so it is the same cart wherever it
    // is standing.
    let by_stop = vec![("the-caravan-stop-1".to_string(), 0u16)];
    assert!(
        !shop::caravan_shelf(&shops, &by_stop)[0].sold,
        "purchases are keyed by the stop, so moving the cart restocks it"
    );
}
