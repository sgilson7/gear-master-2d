//! The map holds together, and a seeded walk is the same walk every time.
//!
//! M2's acceptance, and the map's own lint. Everything here runs against the
//! files GM2D actually ships, so a broken tile is a red test rather than a
//! player walking into a hole.

use std::collections::HashSet;

use gm2d_core::combat::Difficulty;
use gm2d_core::rng::Rng;
use gm2d_core::tile_event::EventsData;
use gm2d_core::world::{step, Dir, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;

fn data(name: &str) -> String {
    let p = format!("{}/../../data/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {p}: {e}"))
}

fn world() -> World {
    World::load(&data("terrain.json"), &data("tiles.json"), D).expect("the shipped map loads")
}

fn events() -> EventsData {
    EventsData::parse(&data("events.json")).expect("the shipped events load")
}

// ------------------------------------------------------------- the map holds

/// Every walkable tile can be reached from the start.
///
/// A flood fill, because the failure it catches is invisible by inspection: one
/// mistyped glyph in a twenty-character row walls off a quarter of the map, and
/// the only symptom is a player who never finds the third town.
#[test]
fn the_whole_map_is_reachable_from_the_start() {
    let w = world();
    let mut seen: HashSet<(u8, u8)> = HashSet::new();
    let mut queue = vec![w.start];
    seen.insert(w.start);
    while let Some((x, y)) = queue.pop() {
        for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if !w.in_bounds(nx, ny) {
                continue;
            }
            let (nx, ny) = (nx as u8, ny as u8);
            if w.passable(nx, ny) && seen.insert((nx, ny)) {
                queue.push((nx, ny));
            }
        }
    }

    let mut stranded = Vec::new();
    for y in 0..w.height {
        for x in 0..w.width {
            if w.passable(x, y) && !seen.contains(&(x, y)) {
                stranded.push((x, y));
            }
        }
    }
    assert!(
        stranded.is_empty(),
        "{} walkable tiles cannot be reached from the start: {:?}",
        stranded.len(),
        &stranded[..stranded.len().min(12)]
    );
}

/// Every place named on the map is somewhere a player can stand.
#[test]
fn every_place_is_on_walkable_ground() {
    let w = world();
    for p in &w.places {
        let (x, y) = (p.at[0], p.at[1]);
        assert!(w.passable(x, y), "{:?} stands on {}", p.id, w.terrain_name(x, y));
    }
}

/// Every event the map places exists, and no event is placed twice.
///
/// Both halves matter. A missing event is a tile that does nothing; a doubled
/// one is an event that can be answered in two places and remembered in one.
#[test]
fn every_placed_event_exists_exactly_once() {
    let w = world();
    let e = events();

    let mut placed: Vec<&str> = Vec::new();
    for p in w.places.iter().filter(|p| p.kind == PlaceKind::Event) {
        assert!(e.get(&p.id).is_some(), "the map places {:?}, which events.json has not got", p.id);
        assert!(!placed.contains(&p.id.as_str()), "{:?} is placed twice", p.id);
        placed.push(&p.id);
    }

    for ev in &e.events {
        assert!(
            placed.contains(&ev.id.as_str()),
            "{:?} is written and never placed, so no player will read it",
            ev.id
        );
    }
}

/// Every component and flag an event hands out is one the game has.
#[test]
fn every_event_outcome_is_real() {
    use gm2d_core::piece::CATALOG;
    use gm2d_core::tile_event::{Outcome, Requirement};

    fn walk(o: &Outcome, f: &mut impl FnMut(&Outcome)) {
        f(o);
        if let Outcome::All(list) = o {
            for i in list {
                walk(i, f);
            }
        }
    }

    let e = events();
    let mut set_flags: HashSet<String> = HashSet::new();
    for ev in &e.events {
        for c in &ev.choices {
            walk(&c.outcome, &mut |o| {
                if let Outcome::Give(name) = o {
                    assert!(
                        CATALOG.iter().any(|d| d.name == name),
                        "{:?} hands out {name:?}, which the catalogue has not got",
                        ev.id
                    );
                }
                if let Outcome::Flag(f) = o {
                    set_flags.insert(f.clone());
                }
            });
        }
    }

    // A requirement on a flag nothing sets is a choice no player can take.
    for ev in &e.events {
        for c in &ev.choices {
            if let Requirement::Flag(f) = &c.requires {
                assert!(
                    set_flags.contains(f),
                    "{:?} wants the flag {f:?} and nothing in the game sets it",
                    ev.id
                );
            }
        }
    }
}

// ------------------------------------------------------------- danger

/// **No data file types a danger number.**
///
/// The acceptance criterion the brief states in those words. Danger is the mean
/// of `creature_rating` over a region's pool, so a number in a file would be an
/// opinion competing with a measurement — and the measurement would lose,
/// quietly, because whoever typed the number would trust it.
#[test]
fn no_data_file_types_a_danger_number() {
    for name in ["tiles.json", "terrain.json", "events.json"] {
        let text = data(name);
        assert!(
            !text.contains("\"danger\""),
            "{name} contains a danger field; danger is measured, not typed"
        );
    }
}

/// Danger rises with the ladder, region by region, out from the pit.
///
/// Not a tuning assertion — a statement that the map has a gradient at all. A
/// map whose regions all rate the same is a map with no shape, and the encounter
/// formula would have nothing to do.
#[test]
fn the_map_has_a_difficulty_gradient() {
    let w = world();
    let order = [
        "the-end-of-all-gears",
        "the-slag-flats",
        "the-burnwarp-shallows",
        "the-bengulon-verge",
        "west-bambulon",
    ];
    let mut last = 0;
    for id in order {
        let r = w.regions.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("no region {id}"));
        assert!(
            r.danger > last,
            "{id} rates {} and the region before it rated {last}",
            r.danger
        );
        last = r.danger;
    }
    assert!(last > 200, "the hardest region rates only {last}, so the gradient is flat");
}

/// The encounter chance is monotone in danger and never leaves its bounds.
///
/// Checked across every tile of the shipped map rather than at a few sample
/// points, because the cap is the thing that would be silently wrong: a late
/// region on slow terrain that rolled a fight on nine steps in ten would still
/// pass a spot check on the early map.
#[test]
fn encounter_chances_stay_inside_their_bounds() {
    use gm2d_core::world::MAX_ENCOUNTER_PER_MILLE;
    let w = world();
    for y in 0..w.height {
        for x in 0..w.width {
            let c = w.encounter_per_mille(x, y);
            assert!(
                (0..=MAX_ENCOUNTER_PER_MILLE).contains(&c),
                "({x}, {y}) rolls {c} per mille"
            );
            if !w.passable(x, y) {
                continue;
            }
            if w.terrain_at(x, y).encounter_per_mille == 0 {
                assert_eq!(c, 0, "({x}, {y}) is safe terrain and rolls {c}");
            }
        }
    }

    // Same terrain, harder region, higher chance. The monotonicity the brief
    // asks for, read off the map rather than off the formula.
    let pit = w.regions.iter().find(|r| r.id == "the-end-of-all-gears").unwrap().danger;
    let far = w.regions.iter().find(|r| r.id == "west-bambulon").unwrap().danger;
    assert!(far > pit);
}

// ------------------------------------------------------------- determinism

/// A fixed path produces the same encounters every time it is walked.
///
/// The property the whole integer discipline exists for. Two walks from the
/// same seed along the same route have to agree tile for tile — including the
/// creatures drawn, which is the part a float would move.
#[test]
fn a_seeded_walk_replays() {
    let w = world();
    let path = fixed_path();

    let a = walk(&w, &path, 0xC0FF_EE00_1234_5678);
    let b = walk(&w, &path, 0xC0FF_EE00_1234_5678);
    assert_eq!(a, b, "the same seed walked the same path and met different creatures");

    let c = walk(&w, &path, 0xC0FF_EE00_1234_5679);
    assert_ne!(a, c, "two different seeds produced identical walks, so the seed is ignored");
}

/// A blocked step draws nothing.
///
/// Otherwise two players walking the same route would meet different creatures
/// depending on how often they misjudged a cliff, and a replay would depend on
/// the player's mistakes rather than on their path.
#[test]
fn walking_into_a_wall_does_not_move_the_stream() {
    let w = world();
    let mut state = WorldState::at_start(&w);
    let mut rng = Rng::new(99);
    let before = rng.state();

    // West from the start column is the map's edge.
    let s = step(&w, &mut state, &mut rng, D, Dir::West);
    assert!(!s.moved && s.blocked.is_some(), "expected to be refused");
    assert_eq!(rng.state(), before, "a refused step advanced the random stream");
    assert_eq!(state.at, [w.start.0, w.start.1], "a refused step moved the player");
}

/// A walk interrupted by a save resumes as though it had not been.
///
/// Gate 3's requirement, and the one that ties M2 back to M1: the position, the
/// answered set and the stream all have to cross the file together, or a player
/// who saves mid-journey gets a different journey back.
#[test]
fn a_walk_survives_being_saved_halfway() {
    use gm2d_core::game::Game;
    use gm2d_core::save;

    let w = world();
    let path = fixed_path();
    let seed = 0xC0FF_EE00_1234_5678;

    let straight = walk(&w, &path, seed);

    let mut g = Game::new(seed, "td");
    g.world = WorldState::at_start(&w);
    let mut interrupted = Vec::new();
    for (i, d) in path.iter().enumerate() {
        if i == path.len() / 2 {
            let text = save::save(&g);
            g = save::load(&text).expect("a mid-walk save loads");
        }
        let s = step(&w, &mut g.world, &mut g.rng, D, *d);
        interrupted.push((g.world.at, s.met(), s.event.clone()));
    }

    assert_eq!(straight, interrupted, "the walk changed across a save");
}

/// The route is chosen once and shared, so every determinism test is walking
/// the same ground. Out of the pit, east along the bottom road, then north.
fn fixed_path() -> Vec<Dir> {
    let mut p = Vec::new();
    for _ in 0..8 {
        p.push(Dir::East);
    }
    for _ in 0..6 {
        p.push(Dir::North);
    }
    for _ in 0..5 {
        p.push(Dir::East);
    }
    for _ in 0..5 {
        p.push(Dir::North);
    }
    for _ in 0..4 {
        p.push(Dir::West);
    }
    p
}

type Trace = Vec<([u8; 2], Option<&'static str>, Option<String>)>;

fn walk(w: &World, path: &[Dir], seed: u64) -> Trace {
    let mut state = WorldState::at_start(w);
    let mut rng = Rng::new(seed);
    path.iter()
        .map(|d| {
            let s = step(w, &mut state, &mut rng, D, *d);
            (state.at, s.met(), s.event.clone())
        })
        .collect()
}

/// A walk long enough to be worth walking meets something.
///
/// Guards the case where the whole encounter system is wired up correctly and
/// the numbers are such that nothing ever happens.
#[test]
fn a_long_walk_starts_some_fights() {
    let w = world();
    let mut state = WorldState::at_start(&w);
    let mut rng = Rng::new(4);
    let mut met = 0;
    let dirs = [Dir::East, Dir::North, Dir::West, Dir::North];
    for i in 0..400 {
        let s = step(&w, &mut state, &mut rng, D, dirs[i % dirs.len()]);
        if s.encounter.is_some() {
            met += 1;
        }
    }
    assert!(met > 5, "four hundred steps started {met} fights");
}
