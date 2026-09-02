//! The north is a decision, not a slope.
//!
//! Nothing stopped a level-one character walking fifteen tiles north into a
//! region of two-thousand-rated creatures. The gradient was a gradient and not
//! a gate, and this is the gate.
//!
//! **A crossing guards a region, not its own tile**, which is a divergence from
//! `PLAN-M9.md` §M9.3 and the map is the reason: rows four to fifteen are open
//! ground twelve tiles wide, so a crossing that refused only the square it
//! stands on would need a dozen of them across a row — which is the wall the
//! plan rejected, drawn in places instead of in rock. The place itself stands
//! on the near side of what it guards, so it is a milestone you can walk up to
//! and read rather than an invisible rule.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::rng::Rng;
use gm2d_core::world::{self, Allowances, Dir, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;

fn overworld() -> World {
    data::world(D)
}

fn crossings(w: &World) -> Vec<&gm2d_core::world::PlaceDef> {
    w.places.iter().filter(|p| p.kind == PlaceKind::Crossing).collect()
}

fn at(w: &World, x: u8, y: u8) -> WorldState {
    WorldState { at: [x, y], ..WorldState::at_start(w) }
}

// ------------------------------------------------------------------ the file

/// Every crossing guards a region that exists, asks for a level, and says why.
#[test]
fn every_crossing_guards_something_and_says_why() {
    let w = overworld();
    let cs = crossings(&w);
    assert_eq!(cs.len(), 2, "the plan places two");
    for c in &cs {
        let guards = c.guards.as_deref().unwrap_or_else(|| panic!("{} guards nothing", c.id));
        assert!(
            w.regions.iter().any(|r| r.id == guards),
            "{} guards {guards:?}, which is not a region on this map",
            c.id
        );
        let need = c.needs_level.unwrap_or_else(|| panic!("{} asks for nothing", c.id));
        assert!(need > 1, "{} asks for level {need}, which everybody already is", c.id);
        assert!(!c.shut.is_empty(), "{} refuses in silence", c.id);
        // TONE 12: the sentence is the world's and carries no figure, because
        // the figure is `needs_level` and the engine appends it. Two copies in
        // one file is one copy too many.
        assert!(
            !c.shut.chars().any(|ch| ch.is_ascii_digit()),
            "{}: the prose states a number, and `needs_level` is two lines above it",
            c.id
        );
        // **It stands on the near side of what it guards**, or it is a
        // signpost you can only read from the far side of the thing it warns
        // about.
        let here = w.region_at(c.at[0], c.at[1]).map(|r| r.id.as_str());
        assert_ne!(here, Some(guards), "{} stands inside the region it guards", c.id);
    }
    // And the two of them are not the same crossing twice.
    assert_ne!(cs[0].guards, cs[1].guards);
}

/// **Two regions at level one, five by level nine.**
///
/// The number the plan's table names, walked rather than declared: flood-fill
/// the map from the start at each level and count which regions are reachable.
#[test]
fn the_north_opens_a_region_at_a_time() {
    let w = overworld();
    let reach = |level: u32| -> Vec<String> {
        let allowed = Allowances { level, ..Allowances::default() };
        let start = WorldState::at_start(&w);
        let mut seen = vec![start.at];
        let mut queue = vec![start.at];
        while let Some(at) = queue.pop() {
            for d in [Dir::North, Dir::South, Dir::East, Dir::West] {
                let mut state = WorldState { at, ..start.clone() };
                let mut rng = Rng::new(1);
                // A step that is refused for any reason is a step not taken;
                // a fight rolled on the way in does not stop the walking.
                let s = world::step(&w, &mut state, &mut rng, D, d, &allowed);
                if s.moved && !seen.contains(&state.at) {
                    seen.push(state.at);
                    queue.push(state.at);
                }
            }
        }
        let mut regions: Vec<String> = seen
            .iter()
            .filter_map(|c| w.region_at(c[0], c[1]).map(|r| r.id.clone()))
            .collect();
        regions.sort();
        regions.dedup();
        regions
    };
    assert_eq!(
        reach(1),
        vec!["the-end-of-all-gears".to_string(), "the-slag-flats".to_string()],
        "a level-one character can reach more than the pit and the flats"
    );
    assert_eq!(reach(5).len(), 3, "level five opens exactly one more: {:?}", reach(5));
    assert_eq!(reach(9).len(), 5, "level nine opens the rest: {:?}", reach(9));
}

// ------------------------------------------------------------------ the step

/// A crossing refuses, and the refusal names the number.
#[test]
fn a_crossing_refuses_and_says_why() {
    let w = overworld();
    let c = crossings(&w)
        .into_iter()
        .find(|c| c.guards.as_deref() == Some("the-burnwarp-shallows"))
        .expect("something guards the shallows");
    let need = c.needs_level.unwrap();

    // Standing on the crossing itself, which is in the Slag Flats, one step
    // south of the boundary.
    let mut rng = Rng::new(3);
    let mut low = at(&w, c.at[0], c.at[1]);
    let s = world::step(&w, &mut low, &mut rng, D, Dir::North,
                        &Allowances { level: need - 1, ..Allowances::default() });
    assert!(!s.moved, "a level {} character walked north", need - 1);
    assert_eq!(low.at, c.at, "a refused step moved somebody");
    assert_eq!(s.crossing.as_deref(), Some(c.id.as_str()), "the page cannot tell what refused");
    let why = s.blocked.expect("a refusal with no sentence");
    assert!(why.contains(&c.shut), "the world's half is missing: {why:?}");
    assert!(why.contains(&need.to_string()), "the number is missing: {why:?}");
    assert!(why.contains(&(need - 1).to_string()), "it does not say what you are: {why:?}");

    // And at the level it asks for, it is a road.
    let mut high = at(&w, c.at[0], c.at[1]);
    let s = world::step(&w, &mut high, &mut rng, D, Dir::North,
                        &Allowances { level: need, ..Allowances::default() });
    assert!(s.moved, "{why}");
    assert_eq!(high.at, [c.at[0], c.at[1] - 1]);
}

/// **A refused crossing does not move the stream.**
///
/// The same rule a cliff obeys, and for the same reason: a replay must depend
/// on the path rather than on the player's mistakes.
#[test]
fn a_refused_crossing_draws_nothing() {
    let w = overworld();
    let c = crossings(&w)[0];
    let mut state = at(&w, c.at[0], c.at[1]);
    let mut rng = Rng::new(77);
    let before = rng.state();
    let s = world::step(&w, &mut state, &mut rng, D, Dir::North, &Allowances::default());
    assert!(!s.moved);
    assert_eq!(rng.state(), before, "a refused crossing rolled for an encounter");
    assert_eq!(state.count("tiles-walked"), 0, "a refused step counted as a step");
}

/// **A crossing is a threshold, not a cage.**
///
/// A step that stays inside the guarded region is never refused, and neither is
/// one out of it. Without that a save planted on the far side of a crossing —
/// or a build that lowered the number after somebody had crossed — would be a
/// character who cannot move in any direction, which is the position
/// `World::repair` exists to make impossible.
#[test]
fn a_crossing_never_shuts_somebody_in() {
    let w = overworld();
    let c = crossings(&w)
        .into_iter()
        .find(|c| c.guards.as_deref() == Some("the-burnwarp-shallows"))
        .unwrap();
    let inside = [c.at[0], c.at[1] - 1];
    let nobody = Allowances::default();
    for d in [Dir::North, Dir::South, Dir::East, Dir::West] {
        let mut state = at(&w, inside[0], inside[1]);
        let mut rng = Rng::new(5);
        let s = world::step(&w, &mut state, &mut rng, D, d, &nobody);
        // Some of these run into the lake or the map's edge; what must never
        // happen is a refusal *by the crossing*.
        assert_eq!(s.crossing, None, "planted inside, the crossing refused {d:?} as well");
    }
}

/// **Going home is never refused.**
///
/// A defeat walks the player to their last town, and it does it by putting them
/// there rather than by stepping — so a crossing between them and it cannot
/// strand anybody. Asserted rather than assumed, because the alternative is a
/// character who has lost a fight in the north and cannot get back.
#[test]
fn no_crossing_shuts_a_player_out_of_a_town_they_have_used() {
    let w = overworld();
    let towns: Vec<&gm2d_core::world::PlaceDef> =
        w.places.iter().filter(|p| p.kind == PlaceKind::Town).collect();
    assert!(!towns.is_empty(), "this map has no town and the rest of this proves nothing");

    for t in &towns {
        // **The walk home is a placement, not a walk.** `World::repair` puts
        // the player at their last town, and no crossing is consulted — which
        // is what makes a defeat in the north survivable at all.
        let mut wedged =
            WorldState { at: [0, 0], last_town: t.id.clone(), ..WorldState::at_start(&w) };
        w.repair(&mut wedged, &Allowances::default());
        assert_eq!(wedged.at, t.at, "a repair did not land on the town it was told about");

        // And the belt to that brace: no town stands behind a crossing at all.
        // A town you can only reach at level nine is a town a level-one player
        // sees on the map and cannot get to, and the one on this map is where
        // the game starts.
        let region = w.region_at(t.at[0], t.at[1]).map(|r| r.id.clone());
        for c in crossings(&w) {
            assert_ne!(
                c.guards, region,
                "{} is behind {}, which shuts a player out of a town",
                t.id, c.id
            );
        }
    }
}

/// A crossing is walked over, not stopped on.
#[test]
fn a_crossing_is_walked_over() {
    let w = overworld();
    let c = crossings(&w)[0];
    let mut state = at(&w, c.at[0], c.at[1] + 1);
    let mut rng = Rng::new(11);
    let open = Allowances { level: 99, ..Allowances::default() };
    let s = world::step(&w, &mut state, &mut rng, D, Dir::North, &open);
    assert!(s.moved, "could not reach the crossing's own tile");
    assert_eq!(state.at, c.at);
    assert!(s.town.is_none() && s.gate.is_none() && s.door.is_none() && s.event.is_none(),
            "a crossing stopped the walk like a gate");
}

// ------------------------------------------------------------- and the walker

/// A character's level travels with the allowances, so the map never reads one.
#[test]
fn the_level_reaches_the_step_from_the_character() {
    let c = Character::starting();
    assert_eq!(c.allowances().level, c.level());
    let mut grown = Character::starting();
    grown.gain_xp(100_000);
    assert!(grown.level() > c.level(), "the fixture did not level up");
    assert_eq!(grown.allowances().level, grown.level());
}
