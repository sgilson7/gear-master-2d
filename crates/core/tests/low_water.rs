//! M14.1 — the Treyway's south, and the tide that goes out on the tenth cairn.
//!
//! **The Treyway is one file and is now sixteen by twenty-six.** The note at
//! the top of that file already says West Bambulon is a tile of it; a second
//! file for the shore would have been two places to keep identical everywhere
//! they are not deliberately different, which is how a map and its copy drift.
//!
//! So the thing this milestone has to prove is that **nothing above row 15
//! moved**, and that is what most of this file is.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const TREYWAY: &str = "the-treyway";

/// **Every gate into the Treyway lands where it always did.**
///
/// Rows 0 to 14 are byte-for-byte what they were, so no `at_to` on any road
/// into this map had to move — and the way to say that is to walk every gate
/// on every map and check the tile it names is the tile it named.
///
/// Negative-tested by inserting the eleven new rows at the *top* of the file
/// rather than the bottom: the door back into Bambulon landed in the sea, and
/// this named it.
#[test]
fn no_gate_into_the_treyway_moved() {
    let treyway = data::map(TREYWAY, D);
    let mut roads = 0;
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places.iter().filter(|p| p.kind == PlaceKind::Gate) {
            if p.to.as_deref() != Some(TREYWAY) {
                continue;
            }
            roads += 1;
            let Some([x, y]) = p.at_to else { continue };
            assert!(
                treyway.passable(x, y),
                "{id}/{}: lands at ({x}, {y}) on the Treyway, which is {:?}",
                p.id,
                treyway.terrain_name(x, y)
            );
        }
    }
    assert!(roads >= 2, "only {roads} roads into the Treyway, so this proves nothing");

    // And the two tiles the country is entered on are still what they were:
    // the door back in from Bambulon, and the way off the Reach.
    assert_eq!(treyway.terrain_name(6, 1), "plain", "the Reach's landing tile moved");
    assert!(
        treyway.place_at(13, 13).is_some_and(|p| p.id == "the-door-back"),
        "the door back into Bambulon moved"
    );
}

/// **The tide goes out on the tenth cairn, and not before.**
///
/// Two tiles at column 8, and they are the only ground between the coast at
/// row 14 and the shore at row 17. Drawn from the first visit — a player
/// standing at the water has been able to see the far side the whole time —
/// and impassable until `built-the-tenth`, which is the last thing anybody does
/// on the Reach.
///
/// Negative-tested by drawing them `coast` in the file: the south was walkable
/// from the first afternoon and `the_south_is_shut_until_the_reach_is_finished`
/// said so.
#[test]
fn the_tide_goes_out_on_the_tenth_cairn() {
    let mut st = WorldState::default();
    st.map = TREYWAY.into();
    let dry = Allowances { wade: false, level: 99 };

    let before = data::map_now(TREYWAY, D, &st);
    for y in [15u8, 16] {
        assert_eq!(before.terrain_name(8, y), "tide", "(8, {y}) is not the tide");
        assert!(!before.walkable(8, y, &dry), "the tide was out before the tenth cairn");
        // **Drawn, though.** A wall you cannot see is a map that ends; a tide
        // you can see is a map with a far side you have not earned.
        assert!(before.ever_walkable(8, y), "the tide is a wall nothing ever opens");
    }

    // A Toad set does not open it either. `Rule::Wade` opens `water`, and the
    // sea and the tide are their own terrains for exactly this reason.
    let wading = Allowances { wade: true, level: 99 };
    assert!(!before.walkable(8, 15, &wading), "a frame walked out onto the tide");

    st.flags.push("built-the-tenth".into());
    let after = data::map_now(TREYWAY, D, &st);
    for y in [15u8, 16] {
        assert_eq!(after.terrain_name(8, y), "coast", "(8, {y}) did not go out");
        assert!(after.walkable(8, y, &dry), "the tide went out and is still a wall");
    }
    // And it took nothing else with it.
    assert_eq!(after.terrain_name(7, 15), "sea", "the whole row went out");
    assert_eq!(after.terrain_name(9, 16), "sea", "the whole row went out");
}

/// **The shore is reachable once the tide is out, and not one tile of it
/// before.**
///
/// The reachability question asked the way M11.7's failure taught this project
/// to ask it: not *are there tiles*, but *can a walker get to them from the
/// door they come in through*.
#[test]
fn the_south_is_shut_until_the_reach_is_finished() {
    let flood = |flags: &[&str]| -> usize {
        let mut st = WorldState::default();
        st.map = TREYWAY.into();
        st.flags = flags.iter().map(|s| s.to_string()).collect();
        let w = data::map_now(TREYWAY, D, &st);
        let a = Allowances { wade: false, level: 99 };
        // From the door back into Bambulon, which is where a player arrives.
        let mut seen = std::collections::BTreeSet::new();
        let mut queue = vec![[13u8, 13u8]];
        seen.insert([13u8, 13u8]);
        while let Some([x, y]) = queue.pop() {
            for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let (nx, ny) = (nx as u8, ny as u8);
                if w.walkable(nx, ny, &a) && seen.insert([nx, ny]) {
                    queue.push([nx, ny]);
                }
            }
        }
        seen.iter().filter(|[_, y]| *y >= 15).count()
    };

    assert_eq!(flood(&[]), 0, "the shore was reachable before the tide went out");
    let open = flood(&["built-the-tenth"]);
    assert!(open > 100, "the tide went out onto {open} tiles, which is not a country");
}

/// **The Low Water is its own band, and the commonest fight in it is one you
/// can win.**
///
/// `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` says the
/// second half over every map; what is here is the first: the shore is not part
/// of the first Treyway, which brackets twelve to sixteen, and it is not part
/// of the Kolok Downs either.
#[test]
fn the_shore_is_a_band_of_its_own() {
    let w = data::map(TREYWAY, D);
    let low = w
        .regions
        .iter()
        .find(|r| r.id == "the-low-water")
        .expect("the shore has no region");
    let first = w.regions.iter().find(|r| r.id == "the-first-treyway").expect("the door's band");
    assert!(
        low.danger > first.danger,
        "the shore ({}) is no harder than the tile you come in on ({})",
        low.danger,
        first.danger
    );
    // Every walkable tile of the south is in it, which is the check that would
    // have caught a box drawn one row short.
    for y in 17..25u8 {
        for x in 0..16u8 {
            if w.passable(x, y) {
                assert_eq!(
                    w.region_at(x, y).map(|r| r.id.as_str()),
                    Some("the-low-water"),
                    "({x}, {y}) is walkable and is not the shore's"
                );
            }
        }
    }
}

/// **The lip of the Sump refuses the way the Reach's edge does**, and it is a
/// stack of four rather than one map.
#[test]
fn the_lip_is_a_stack_and_it_wants_an_instrument() {
    let w = data::map(TREYWAY, D);
    let lip = w.place_at(7, 21).expect("nothing at the lip");
    assert_eq!(lip.id, "the-lip-of-the-sump");
    assert!(lip.needs_survey, "the lip opens for anybody");
    assert!(lip.to.is_none(), "the lip names one map as well as four floors");
    assert_eq!(lip.floors.len(), 4, "the Sump is four floors");
    assert!(!lip.shut.is_empty(), "a shut door that says nothing");

    // **Top down, and each floor is done when its own thing is done.** Three of
    // the four are flags, because three of the four have no boss on them.
    let mut st = WorldState::default();
    assert_eq!(lip.opens_onto(&st), Some("the-sump-1"));
    st.flags.push("sluice-b".into());
    assert_eq!(lip.opens_onto(&st), Some("the-sump-2"), "a solved floor is still the way in");
    st.flags.push("the-shelf-is-open".into());
    st.flags.push("cairn-9".into());
    assert_eq!(lip.opens_onto(&st), Some("the-sump-4"));
    // M14.1 ships stubs, so the bottom clears on the stub's flag; M14.2 puts
    // the Ninth Surveyor on that tile and this becomes the boss's own id.
    st.flags.push("the-sump-is-bottomed".into());
    assert_eq!(lip.opens_onto(&st), None, "the Sump is finished and still has a floor in it");
    assert_eq!(lip.floors_cleared(&st), 4);
}
