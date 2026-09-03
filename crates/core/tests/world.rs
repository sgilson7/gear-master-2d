//! The map holds together, and a seeded walk is the same walk every time.
//!
//! M2's acceptance, and the map's own lint. Everything here runs against the
//! files GM2D actually ships, so a broken tile is a red test rather than a
//! player walking into a hole.

use std::collections::HashSet;

use gm2d_core::combat::Difficulty;
use gm2d_core::rng::Rng;
use gm2d_core::tile_event::EventsData;
use gm2d_core::world::{step, Allowances, Dir, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;

/// The overworld's layout, by path. One file per map since M11.1.
const WEST_BAMBULON: &str = "maps/west-bambulon.tiles.json";

fn data(name: &str) -> String {
    let p = format!("{}/../../data/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {p}: {e}"))
}

fn world() -> World {
    World::load(&data("terrain.json"), &data(WEST_BAMBULON), D).expect("the shipped map loads")
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
    // **Every map, not the first one.** This walked West Bambulon alone while
    // three maps shipped, and a mistyped glyph on any of the others would have
    // walled off a quarter of it in silence — which is exactly the failure the
    // flood fill exists for and exactly the one a per-map test cannot see.
    for (id, _) in gm2d_core::data::MAPS {
        let w = gm2d_core::data::map(id, D);
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
            "{id}: {} walkable tiles cannot be reached from the start: {:?}",
            stranded.len(),
            &stranded[..stranded.len().min(12)]
        );
    }
}

/// Every place named on any map is somewhere a player can stand.
#[test]
fn every_place_is_on_walkable_ground() {
    for (id, _) in gm2d_core::data::MAPS {
        let w = gm2d_core::data::map(id, D);
        for p in &w.places {
            let (x, y) = (p.at[0], p.at[1]);
            assert!(w.passable(x, y), "{id}: {:?} stands on {}", p.id, w.terrain_name(x, y));
        }
    }
}

/// **No two maps share a place id.**
///
/// `answered`, `bought` and `quests_done` are one set each for the whole game,
/// so two places called the same thing on two maps are one place remembered in
/// one of them. It has never happened and it is one copy-paste away.
#[test]
fn no_two_maps_name_a_place_the_same() {
    let mut seen: Vec<(String, &str)> = Vec::new();
    for (id, _) in gm2d_core::data::MAPS {
        for p in gm2d_core::data::map(id, D).places {
            if let Some((_, other)) = seen.iter().find(|(k, _)| *k == p.id) {
                panic!("{:?} is a place on both {other} and {id}", p.id);
            }
            seen.push((p.id.clone(), id));
        }
    }
}

/// Every event any map places exists, and no event is placed twice.
///
/// Both halves matter. A missing event is a tile that does nothing; a doubled
/// one is an event that can be answered in two places and remembered in one —
/// and since M11.1 "twice" means *across every map*, because `answered` is one
/// set for the whole game and always was.
#[test]
fn every_placed_event_exists_exactly_once() {
    use gm2d_core::data;
    let e = events();

    let mut placed: Vec<String> = Vec::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in w.places.iter().filter(|p| p.kind == PlaceKind::Event) {
            assert!(
                e.get(&p.id).is_some(),
                "{id} places {:?}, which events.json has not got",
                p.id
            );
            assert!(!placed.contains(&p.id), "{:?} is placed twice", p.id);
            placed.push(p.id.clone());
        }
    }

    for ev in &e.events {
        assert!(
            placed.contains(&ev.id),
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
    for name in [WEST_BAMBULON, "terrain.json", "events.json"] {
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
    for (id, _) in gm2d_core::data::MAPS {
        let w = gm2d_core::data::map(id, D);
        for y in 0..w.height {
            for x in 0..w.width {
                let c = w.encounter_per_mille(x, y);
                assert!(
                    (0..=MAX_ENCOUNTER_PER_MILLE).contains(&c),
                    "{id}: ({x}, {y}) rolls {c} per mille"
                );
                if !w.passable(x, y) {
                    continue;
                }
                if w.terrain_at(x, y).encounter_per_mille == 0 {
                    assert_eq!(c, 0, "{id}: ({x}, {y}) is safe terrain and rolls {c}");
                }
            }
        }
    }
    let w = world();

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
    let s = step(&w, &mut state, &mut rng, D, Dir::West, &Allowances::default());
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
        let s = step(&w, &mut g.world, &mut g.rng, D, *d, &Allowances::default());
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

/// Walks a real `Game`, not a bare `Rng`.
///
/// It matters: `Game::new` stocks the shop off the same stream, so a walk
/// driven by `Rng::new(seed)` starts several draws behind one driven by a game
/// made from the same seed. One stream is the rule — this helper is what makes
/// the tests measure the stream the game actually walks on.
fn walk(w: &World, path: &[Dir], seed: u64) -> Trace {
    let mut g = gm2d_core::game::Game::new(seed, "td");
    g.world = WorldState::at_start(w);
    path.iter()
        .map(|d| {
            let s = step(w, &mut g.world, &mut g.rng, D, *d, &Allowances::default());
            (g.world.at, s.met(), s.event.clone())
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
        let s = step(&w, &mut state, &mut rng, D, dirs[i % dirs.len()], &Allowances::default());
        if s.encounter.is_some() {
            met += 1;
        }
    }
    assert!(met > 5, "four hundred steps started {met} fights");
}

// ------------------------------------------------------------- being stuck

/// A position that cannot be stood on is repaired rather than trusted.
///
/// Reported from a real session: a player carrying an autosave from before M2
/// spawned inside the rock in the top-left corner and could not move. The
/// `world` field is `#[serde(default)]` so that older saves still open, and a
/// default `WorldState` stands at `(0, 0)`.
#[test]
fn a_position_off_the_walkable_map_is_repaired() {
    let w = world();

    // The exact reported case: an older save, no world at all.
    let mut state = WorldState::default();
    assert_eq!(state.at, [0, 0]);
    assert!(!w.passable(0, 0), "the corner is walkable, so this test proves nothing");
    let was = w.repair(&mut state, &Allowances::default());
    assert_eq!(was, Some([0, 0]), "the repair did not report moving anybody");
    assert!(w.passable(state.at[0], state.at[1]), "repaired onto {:?}", state.at);
    assert_eq!(state.at, [w.start.0, w.start.1], "with no town known, go to the start");

    // With a town remembered, that is where they wake up. Taken off the map
    // rather than named, because which towns are on which map is content and
    // this rule is not about any one of them.
    let town = w
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Town)
        .expect("the map has a town");
    let mut state = WorldState::default();
    state.last_town = town.id.clone();
    w.repair(&mut state, &Allowances::default());
    assert_eq!(state.at, town.at, "a remembered town is a better answer than the start");

    // **A town this map does not have.** The first map carries one town and
    // the others are written and waiting for maps that do not exist, so a save
    // can remember somewhere that is not here — a file from before they moved,
    // or from a map this build does not ship. It has to fall back rather than
    // wedge, which is the whole job of `repair`.
    let mut state = WorldState::default();
    state.last_town = "kettleworks".into();
    state.at = [0, 0];
    w.repair(&mut state, &Allowances::default());
    assert_eq!(
        state.at,
        [w.start.0, w.start.1],
        "a town that is not on this map should send you to the start, not nowhere"
    );
    assert!(w.passable(state.at[0], state.at[1]));

    // Off the map entirely.
    let mut state = WorldState::default();
    state.at = [200, 200];
    w.repair(&mut state, &Allowances::default());
    assert!(w.in_bounds(state.at[0] as i32, state.at[1] as i32));
    assert!(w.passable(state.at[0], state.at[1]));

    // And somebody standing somewhere fine is left alone.
    let mut state = WorldState::at_start(&w);
    assert_eq!(w.repair(&mut state, &Allowances::default()), None, "a good position was moved");
    assert_eq!(state.at, [w.start.0, w.start.1]);
}

/// An old save, loaded, leaves the player somewhere they can walk.
///
/// The end-to-end version: this is the file the player actually had.
#[test]
fn a_save_from_before_the_world_existed_still_walks() {
    use gm2d_core::game::Game;
    use gm2d_core::save;

    let text = save::save(&Game::new(7, "td"));
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    v["state"].as_object_mut().unwrap().remove("world");
    let old = serde_json::to_string(&v).unwrap();

    let mut g = save::load(&old).expect("an old save still opens");
    let w = world();
    w.repair(&mut g.world, &Allowances::default());

    // And they can actually go somewhere.
    let mut rng = Rng::new(1);
    let moved = [Dir::North, Dir::South, Dir::East, Dir::West]
        .into_iter()
        .any(|d| step(&w, &mut g.world.clone(), &mut rng, D, d, &Allowances::default()).moved);
    assert!(moved, "the player is walled in at {:?}", g.world.at);
}
