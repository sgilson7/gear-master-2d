//! The map behind the door, and the position that comes back with you.
//!
//! M11.1. Two things are new and one of them is in the save: a third map at a
//! different scale, and `WorldState::positions` — where you were standing on
//! every map you have left.
//!
//! The map's own well-formedness is `tests/world.rs`, which walks every map in
//! `data::MAPS` since this block. What is here is the things that are only
//! true of *this* map, and the field.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::{self, Allowances, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;
const TREYWAY: &str = "the-treyway";

fn treyway() -> World {
    data::map(TREYWAY, D)
}

/// It is a map, it is a different shape, and it is not the overworld under
/// another name.
#[test]
fn the_treyway_is_its_own_map() {
    let w = treyway();
    assert_eq!(w.id, TREYWAY, "the file's id is not the id it is filed under");
    let over = data::world(D);
    assert_ne!((w.width, w.height), (over.width, over.height), "the same map twice");
    assert_eq!((w.width, w.height), (16, 16));
    assert_eq!(w.regions.len(), 3, "three bands, easiest at the door");
}

/// **Its own terrain vocabulary, and the sea is not the lake.**
///
/// `Rule::Wade` opens the rim of anything drawn `water`, so an ocean drawn with
/// it would be an ocean a Toad set walks the edge of. The Treyway's edges are
/// `sea`, which no rule opens, and a test says so because the two look
/// identical on the screen and differ only in what a set does to them.
#[test]
fn nothing_wades_the_treyway_sea() {
    let w = treyway();
    let wading = Allowances { wade: true, level: 99 };
    let mut sea = 0;
    for y in 0..w.height {
        for x in 0..w.width {
            if w.terrain_name(x, y) == "sea" {
                sea += 1;
                assert!(!w.walkable(x, y, &wading), "({x}, {y}) is sea and a toad walks on it");
            }
        }
    }
    assert!(sea > 20, "only {sea} tiles of sea, so the check is nearly vacuous");
    // And the map uses the vocabulary it was given a vocabulary for.
    let mut kinds: Vec<&str> = Vec::new();
    for y in 0..w.height {
        for x in 0..w.width {
            let t = w.terrain_name(x, y);
            if !kinds.contains(&t) {
                kinds.push(t);
            }
        }
    }
    for want in ["plain", "coast", "range", "sea"] {
        assert!(kinds.contains(&want), "the Treyway never draws {want:?}: {kinds:?}");
    }
}

/// **The band the door opens onto is where West Bambulon leaves off.**
///
/// Measured, not declared. `PLAN-M11.md` asks for the overworld to bracket
/// levels five to nine; that number was written before recon and is wrong in
/// the honest direction — the door is behind the Cave, the Cave is behind a
/// crossing that asks for nine, so a player who reaches the door is level
/// twelve at the earliest. Bracketing this at five would have put a continent
/// below the map it opens off. What the test pins is the *shape*: the arrival
/// band is no easier than the map it arrived from, and the far band is harder.
#[test]
fn the_treyway_carries_on_where_bambulon_stops() {
    let w = treyway();
    let over = data::world(D);
    let hardest_at_home = over.regions.iter().map(|r| r.danger).max().expect("regions");

    let door = w
        .places
        .iter()
        .find(|p| p.id == "the-door-back")
        .expect("the way back into Bambulon");
    let arrival = w.region_at(door.at[0], door.at[1]).expect("the door is in a region");
    assert!(
        arrival.danger >= hardest_at_home * 9 / 10,
        "you arrive in {:?} at {} against {hardest_at_home} at home, which is a step down",
        arrival.id,
        arrival.danger
    );

    let hardest = w.regions.iter().map(|r| r.danger).max().unwrap();
    assert!(
        hardest > arrival.danger,
        "every band of the Treyway is the arrival band, so it has no gradient"
    );
}

/// Its three places are the door back and two promises, and the promises are
/// events with something to read.
#[test]
fn the_treyway_promises_two_maps_it_has_not_got() {
    let w = treyway();
    let events = data::events();
    let promises: Vec<&world::PlaceDef> =
        w.places.iter().filter(|p| p.kind == PlaceKind::Event).collect();
    assert_eq!(promises.len(), 2, "the two roads that are not built yet");
    for p in promises {
        let e = events.get(&p.id).unwrap_or_else(|| panic!("{} has nothing to read", p.id));
        assert!(!e.prose.is_empty(), "{}: a promise that says nothing", p.id);
    }
    assert_eq!(w.places.iter().filter(|p| p.kind == PlaceKind::Gate).count(), 1);
}

// ------------------------------------------------------- where you were

/// **A gate with no landing tile lands you where you left off.**
///
/// The whole of `positions`. Walked rather than asserted on the field: what
/// matters is the arrival, and a test that set `positions` by hand and read it
/// back would prove the field round-trips and nothing about the door.
#[test]
fn a_border_puts_you_back_where_you_left() {
    let w = treyway();
    let mut st = WorldState::at_start(&w);
    st.map = TREYWAY.into();

    // Somewhere that is not the start, and somewhere you can stand.
    let elsewhere = [7, 8];
    assert!(w.passable(elsewhere[0], elsewhere[1]), "the fixture stands in the sea");
    assert_ne!(elsewhere, [w.start.0, w.start.1], "the fixture never moved");
    st.at = elsewhere;

    // Leave, which is what a gate does.
    st.remember();
    st.map = "west-bambulon".into();
    st.at = [1, 10];

    assert_eq!(st.recall(TREYWAY), Some(elsewhere), "the map forgot where you were on it");
    assert_eq!(st.recall("the-great-gear-cave"), None, "it remembers a map you have never been on");
}

/// The remembered position moves with you rather than piling up.
#[test]
fn one_row_a_map() {
    let mut st = WorldState::default();
    st.at = [3, 3];
    st.remember();
    st.at = [4, 4];
    st.remember();
    assert_eq!(st.positions.len(), 1, "two rows for one map");
    assert_eq!(st.recall(&world::overworld()), Some([4, 4]));
}

/// **`remember` writes the map you are on, with the empty default resolved.**
///
/// `WorldState::map` is `""` for the first map, because a save written before
/// there was a second one has to open. If `remember` wrote the empty string,
/// `recall("west-bambulon")` would answer nothing and the door would land
/// everybody at the start for ever — silently, and only on the first map.
#[test]
fn the_empty_default_is_resolved_before_it_is_written_down() {
    let mut st = WorldState::default();
    assert_eq!(st.map, "", "the default stopped being the empty string");
    st.at = [2, 9];
    st.remember();
    assert_eq!(st.positions[0].0, world::overworld());
    assert_eq!(st.recall(&world::overworld()), Some([2, 9]));
}

/// A save carrying a position on a map this build has not got is not a crash.
#[test]
fn a_position_on_a_map_that_is_gone_is_ignored() {
    let mut g = Game::new(3, "td");
    g.world = WorldState::at_start(&data::world(D));
    g.world.positions.push(("a-map-nobody-wrote".into(), [200, 200]));
    let text = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&text).expect("it loads");
    assert_eq!(back.world.recall("a-map-nobody-wrote"), Some([200, 200]));
    // And it is never consulted, because no gate opens onto that map.
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            assert_ne!(p.to.as_deref(), Some("a-map-nobody-wrote"));
        }
    }
}
