//! The Wextreen Sands — the second surveyable map, and the first that reads a
//! compass badly.
//!
//! Asked for as *"one more surveyable map that can be accessed from the southern
//! openworld area."*
//!
//! **`survey::mods_for` has taken a `map` argument since M11.6 and never read
//! it.** Its own doc says why it was kept: *a second surveyable map is meant to
//! be a data drop plus an arm here, and a signature that could not tell two maps
//! apart would have to change to become one that could.* This is that, and the
//! whole return on it is that the instrument which reads the Reach best reads
//! this worst — so **which** instrument you built becomes a question about where
//! you are going.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::survey;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const SANDS: &str = "the-wextreen-sands";
const SHORE: &str = "the-low-water";

// ------------------------------------------------------------------ the map

/// It is reached from the southern shore, and it wants an instrument.
#[test]
fn the_sands_are_reached_from_the_southern_shore() {
    let shore = data::map(SHORE, D);
    let door = shore
        .places
        .iter()
        .find(|p| p.to.as_deref() == Some(SANDS))
        .expect("nothing on the shore leads to the sands");
    assert_eq!(door.kind, PlaceKind::Gate);
    assert!(door.needs_survey, "the sands can be walked into with nothing to read them with");
    assert!(!door.shut.is_empty(), "the door refuses in silence");
    // And you can stand next to it, or the refusal is one nobody reads —
    // which is the fault the grating taught this project one commit ago.
    let (x, y) = (door.at[0] as i32, door.at[1] as i32);
    let plain = Allowances::default();
    assert!(
        [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|(dx, dy)| {
            let (nx, ny) = (x + dx, y + dy);
            shore.in_bounds(nx, ny) && shore.walkable(nx as u8, ny as u8, &plain)
        }),
        "the door onto the sands has no standable neighbour"
    );
}

/// The way back, and it lands where the door is.
#[test]
fn the_way_back_lands_on_the_shore() {
    let sands = data::map(SANDS, D);
    let back = sands
        .places
        .iter()
        .find(|p| p.to.as_deref() == Some(SHORE))
        .expect("there is no way off the sands");
    let at = back.at_to.expect("the way back names no tile");
    let shore = data::map(SHORE, D);
    assert!(
        shore.walkable(at[0], at[1], &Allowances::default()),
        "the way back lands on {}, which nobody can stand on",
        shore.terrain_name(at[0], at[1])
    );
}

/// Every tile of it is reachable from where you arrive.
///
/// The M11.7 lesson: a content block needs a reachability *measurement*, and
/// the measurement names the tile it starts from.
#[test]
fn every_tile_of_the_sands_is_reachable() {
    let w = data::map(SANDS, D);
    let allowed = Allowances::default();
    let start = [w.start.0, w.start.1];
    let mut seen = vec![start];
    let mut queue = vec![start];
    while let Some(at) = queue.pop() {
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = (at[0] as i32 + dx, at[1] as i32 + dy);
            if !w.in_bounds(nx, ny) {
                continue;
            }
            let (nx, ny) = (nx as u8, ny as u8);
            if w.walkable(nx, ny, &allowed) && !seen.contains(&[nx, ny]) {
                seen.push([nx, ny]);
                queue.push([nx, ny]);
            }
        }
    }
    let walkable = (0..w.height)
        .flat_map(|y| (0..w.width).map(move |x| (x, y)))
        .filter(|(x, y)| w.walkable(*x, *y, &allowed))
        .count();
    assert_eq!(seen.len(), walkable, "{} of {walkable} tiles are reachable", seen.len());
    for p in &w.places {
        assert!(seen.contains(&p.at), "{} is on a tile nobody can walk to", p.id);
    }
}

// ------------------------------------------------------------ the difference

/// **The instrument that reads the Reach best reads this worst.**
///
/// This is the whole reason a second surveyable map is worth having rather than
/// being a second map with the same numbers on it. On the Reach a compass
/// quiets the ground and an atlas is loud; here it is the other way round,
/// because there is iron under the sand.
#[test]
fn the_sands_read_the_other_way_round() {
    let reach_compass = survey::mods_for("the-reach", "compass", 0);
    let reach_atlas = survey::mods_for("the-reach", "atlas", 0);
    let sands_compass = survey::mods_for(SANDS, "compass", 0);
    let sands_atlas = survey::mods_for(SANDS, "atlas", 0);

    assert!(reach_compass.encounter_pct < 0, "the Reach's compass is not the quiet one");
    assert!(reach_atlas.encounter_pct > 0, "the Reach's atlas is not the loud one");
    assert!(sands_compass.encounter_pct > 0, "the needle is no use here and it should be loud");
    assert!(sands_atlas.encounter_pct < 0, "the paper survey is right here and should be quiet");

    // And the atlas still pays, because that is what an atlas is — the map
    // changes which one is quiet, not what either of them is for.
    assert!(sands_atlas.drops_per_mille > 0 && sands_atlas.xp_pct > 0);
    assert_eq!(sands_atlas.drops_per_mille, reach_atlas.drops_per_mille);
}

/// A compass gets no quieter here however packed the board is.
///
/// On the Reach the quiet scales with how much of a board you gave up to carry
/// the instrument. Here there is nothing to scale: the needle is wrong, and a
/// fuller board does not make it right.
#[test]
fn packing_does_not_fix_the_needle() {
    let bare = survey::mods_for(SANDS, "compass", 0);
    let packed = survey::mods_for(SANDS, "compass", 12);
    assert_eq!(bare.encounter_pct, packed.encounter_pct);
    // Whereas the atlas's quiet does scale, exactly as the Reach's compass does.
    assert!(
        survey::mods_for(SANDS, "atlas", 12).encounter_pct
            < survey::mods_for(SANDS, "atlas", 0).encounter_pct,
        "the paper read does not reward a packed board the way the Reach's compass does"
    );
    // And it has the same floor, so it cannot switch the map off.
    assert!(survey::mods_for(SANDS, "atlas", 40).encounter_pct >= survey::COMPASS_FLOOR_PCT);
}

/// **The golem reads every map the same**, because it is a thing that walked in
/// with you and has no opinion about magnetism.
#[test]
fn the_golem_does_not_care_where_it_is() {
    assert_eq!(survey::mods_for(SANDS, "golem", 3), survey::mods_for("the-reach", "golem", 3));
    assert!(survey::mods_for(SANDS, "golem", 0).golem);
}

/// It is still pure, asked twice.
#[test]
fn reading_the_sands_is_a_pure_function() {
    for kind in ["compass", "atlas", "golem", "nothing"] {
        for n in [0usize, 3, 9] {
            assert_eq!(survey::mods_for(SANDS, kind, n), survey::mods_for(SANDS, kind, n));
        }
    }
}

/// **Nothing about surveying is in the map file**, which is the architectural
/// half of M11.6 and is the thing a second map could most easily break.
#[test]
fn the_sands_file_says_nothing_about_surveying() {
    let raw = data::SANDS_JSON.to_lowercase();
    for word in ["compass", "atlas", "encounter_pct", "drops_per_mille", "xp_pct"] {
        assert!(
            !raw.contains(&format!("\"{word}\"")),
            "the map file carries a surveying key {word:?}, and an instrument is the \
             character's while a map is the world's"
        );
    }
}
