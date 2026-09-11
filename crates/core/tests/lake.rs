//! The lake drains, and there was always something under it.
//!
//! M11.4, and one new idea: **terrain that is derived from what has happened.**
//! The grid is still content and content is still not state — the *rule* is in
//! the map file and what it reads is `answered`, exactly the way a hidden place
//! does. The tiles are not in the save; they come out of it.
//!
//! The other half of the milestone is that `Rule::Wade` widened from the rim to
//! the whole body of water, which is `tests/sets.rs` and `tests/rules.rs`. What
//! is here is the emptying, the way down, and the fact that both ways in reach
//! the same thing.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::world::{Allowances, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;
const UNDER: &str = "under-the-lake";
/// What has to have happened for anything to empty. There is no
/// `tower_dropped` flag and there does not need to be one: the Stack's bottom
/// floor is reachable only once the four above it are gone.
const TOWER_DOWN: &str = "the-drambus-stack-1-boss";

fn bambulon() -> World {
    data::world(D)
}

fn lake_of(w: &World) -> Vec<[u8; 2]> {
    (0..w.height)
        .flat_map(|y| (0..w.width).map(move |x| (x, y)))
        .filter(|&(x, y)| w.terrain_name(x, y) == "water")
        .map(|(x, y)| [x, y])
        .collect()
}

/// **The flag empties it, and nothing else does.**
#[test]
fn the_lake_is_a_lake_until_the_stack_comes_down() {
    let before = bambulon();
    assert_eq!(lake_of(&before).len(), 28, "the lake stopped being twenty-eight tiles");

    let mut fresh = WorldState::at_start(&before);
    let dry_run = data::map_now(&gm2d_core::world::overworld(), D, &fresh);
    assert_eq!(lake_of(&dry_run).len(), 28, "it emptied before anybody did anything");

    fresh.answered.push(TOWER_DOWN.into());
    let after = data::map_now(&gm2d_core::world::overworld(), D, &fresh);
    assert!(lake_of(&after).is_empty(), "the Stack is down and the lake is still full");
    for at in lake_of(&before) {
        assert_eq!(after.terrain_name(at[0], at[1]), "lakebed");
        assert!(after.passable(at[0], at[1]), "a drained bed you cannot walk on");
    }
}

/// **`data::map` is the file and `data::map_now` is the game.**
///
/// The same division `place_at` and `place_now` make, and it matters for the
/// same reason: a lint that could only see the drained lake could not see the
/// undrained one, and the undrained one is what everybody plays for two blocks.
#[test]
fn the_file_still_has_a_lake_in_it() {
    let mut done = WorldState::at_start(&bambulon());
    done.answered.push(TOWER_DOWN.into());
    assert_eq!(lake_of(&data::map(&gm2d_core::world::overworld(), D)).len(), 28);
    assert!(lake_of(&data::map_now(&gm2d_core::world::overworld(), D, &done)).is_empty());
}

/// A drain names terrain the terrain table has, on both sides.
///
/// The guard `Rule::check` and `DropsData::parse` get: a drain naming nothing
/// is a lake that never empties, and nothing else in the game would say so.
#[test]
fn every_drain_names_terrain_that_exists() {
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for d in &w.drains {
            assert!(
                w.terrain_named(&d.from).is_some(),
                "{id}: drains {:?}, which is not terrain",
                d.from
            );
            let to = w
                .terrain_named(&d.to)
                .unwrap_or_else(|| panic!("{id}: into {:?}, which is not terrain", d.to));
            // **A drain may take ground away, and only if a later one gives it
            // back.** Every drain in the game before M14 opened something: a
            // lake into bed, a tide into coast, a channel into silt, and a
            // drain into a wall would have been a lake that empties into rock.
            // The Drowned Gallery is the other kind — chain A floods a road on
            // purpose, and what makes that safe is that chain B is in the same
            // list and turns the water it made into silt. Flags only grow, so a
            // player who pulled A can always pull B; the pairing is the
            // monotone half of `PLAN-M14.md` §1.1 written where it can be
            // checked.
            let given_back = w.drains.iter().skip_while(|x| *x != d).skip(1).any(|later| {
                later.from == d.to
                    && w.terrain_named(&later.to).is_some_and(|t| t.passable)
            });
            assert!(
                to.passable || given_back,
                "{id}: {:?} drains into something you cannot walk on, and nothing after it \
                 turns that back into ground",
                d.from
            );
            // **Only when the drain is the whole map**, which is the question
            // this clause was written to ask and is not the question it was
            // asking. A drain with no `tiles` is every cell of that terrain,
            // and the corner of every map in the game is its wall — so a
            // whole-map drain naming the corner's terrain rewrites the border
            // and the map falls open. A drain that *names its cells* has
            // already said which three it means: the Assay's three doors are
            // one tile of rock each, opening one gap in one wall, which is a
            // door and not a demolition.
            //
            // The clause predates `Drain.tiles` by a block and went on asking
            // the older question. Same shape as the three narrow lints M14.1
            // retired: right for the reason it was written and wrong about the
            // thing in front of it.
            assert!(
                d.tiles.is_some() || w.terrain_name(0, 0) != d.from.as_str(),
                "{id}: the corner of the map drains and the drain names no cells, \
                 which is a whole-map rewrite"
            );
            // **And what it waits for is asked somewhere else now.** The
            // clause that stood here counted only place ids, because when it
            // was written a drain waited on a boss going down. The tide on the
            // Treyway waits on `built-the-tenth`, which is a *flag* an event
            // raises, so this refused it for the right reason and the wrong
            // question — the third narrow lint in this block to do that.
            // `primitives_m14.rs::no_flag_is_waited_on_forever` asks it over
            // every mark and every reader at once, flags included, and two
            // lints asking one question is how they drift.
        }
    }
}

// ------------------------------------------------------------- the way down

fn grating(w: &World) -> &gm2d_core::world::PlaceDef {
    w.places.iter().find(|p| p.id == "the-way-under-the-lake").expect("the grating")
}

/// **The way down is in the middle of the lake, and that is the whole design.**
///
/// Before the tower falls it is under water, and the only thing that reaches it
/// is a whole Toad set — which is the early way in. After the tower falls the
/// water is bed and anybody walks out to it.
#[test]
fn the_way_down_is_reachable_two_ways_and_neither_is_by_accident() {
    let w = bambulon();
    let g = grating(&w);
    assert_eq!(g.kind, PlaceKind::Gate);
    assert_eq!(g.to.as_deref(), Some(UNDER));
    assert_eq!(w.terrain_name(g.at[0], g.at[1]), "water", "the grating is not in the lake");

    // Dry, before: a wall.
    assert!(!w.walkable(g.at[0], g.at[1], &Allowances::default()));
    // Wet, before: the set opens it.
    let wading = Allowances { wade: true, level: 99 };
    assert!(w.walkable(g.at[0], g.at[1], &wading), "the Toad set does not reach the grating");

    // After: everybody.
    let mut done = WorldState::at_start(&w);
    done.answered.push(TOWER_DOWN.into());
    let drained = data::map_now(&gm2d_core::world::overworld(), D, &done);
    assert!(
        drained.walkable(g.at[0], g.at[1], &Allowances::default()),
        "the lake drained and the way down is still shut"
    );
}

/// **One map, read twice.**
///
/// `PLAN-M11.md` §M11.4 asks for a flooded variant with a harder frame for the
/// same boss. It is the same map: its own `drains` block turns the two middle
/// rows to bed when the Stack falls, so entered early the straight run is shut
/// and the only way to the bottom is the long way round the slag. What the
/// early way costs is the walk, which is the only currency a dungeon here has —
/// not a second boss, and not a second map.
#[test]
fn under_the_lake_is_longer_when_it_is_wet() {
    let wet = data::map(UNDER, D);
    let mut done = WorldState::at_start(&wet);
    done.answered.push(TOWER_DOWN.into());
    let dry = data::map_now(UNDER, D, &done);

    let boss = wet
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Boss)
        .expect("something at the bottom of it");

    let walk = |w: &World| -> Option<usize> {
        let start = (w.start.0, w.start.1);
        let goal = (boss.at[0], boss.at[1]);
        let mut seen = vec![start];
        let mut edge = vec![(start, 0usize)];
        while let Some((at, d)) = edge.pop() {
            if at == goal {
                return Some(d);
            }
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (at.0 as i32 + dx, at.1 as i32 + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let n = (nx as u8, ny as u8);
                if w.passable(n.0, n.1) && !seen.contains(&n) {
                    seen.push(n);
                    edge.insert(0, (n, d + 1));
                }
            }
        }
        None
    };

    let wet_walk = walk(&wet).expect("the bottom is unreachable while it is flooded");
    let dry_walk = walk(&dry).expect("the bottom is unreachable once it has drained");
    assert!(
        wet_walk > dry_walk + 5,
        "wet is {wet_walk} tiles and dry is {dry_walk}, which is not a harder frame"
    );
    // And the long way is the dear way: slag at 260 per mille against road at 30.
    assert!(wet.terrain_name(2, 1) == "slag", "the long way round is not the rough way");
}

/// **Both ways in reach the same thing, and it can only be done once.**
#[test]
fn there_is_one_boss_under_the_lake_and_one_door_behind_it() {
    let w = data::map(UNDER, D);
    let bosses: Vec<_> = w.places.iter().filter(|p| p.kind == PlaceKind::Boss).collect();
    assert_eq!(bosses.len(), 1);
    let boss = bosses[0];
    assert_eq!(boss.id, "the-bottom-of-the-lake");
    assert!(!boss.drops.is_empty(), "it carries nothing");
    // Harder than anything in the tower, which is what makes it the bottom.
    let rated = |name: &str| {
        gm2d_core::rating::creature_rating(
            gm2d_core::combat::creature(name).expect("in the ladder"),
            D,
        )
    };
    let here = rated(boss.creature.as_deref().expect("a creature"));
    let tower = data::map("the-drambus-stack-1", D)
        .places
        .iter()
        .find_map(|p| p.creature.clone())
        .expect("the Stack's bottom floor");
    assert!(here > rated(&tower), "the lake is easier than the tower it drained");

    // **The door behind it is a way on, and until M14.3 it was a screen that
    // said nobody had decided.** *Nothing is behind the door* was true and is
    // not; it is a gate onto the Silt Stair, still not there until the boss is
    // down, and still wanting no key — the whole of what it wants is that the
    // thing in front of it has stopped.
    let door = w
        .places
        .iter()
        .find(|p| p.id == "the-door-under-the-lake")
        .expect("nothing behind it at all");
    assert_eq!(door.kind, PlaceKind::Gate, "the door under the lake still ends the writing");
    assert_eq!(door.to.as_deref(), Some("the-silt-stair-1"));
    assert_eq!(door.hidden_until.as_deref(), Some("the-bottom-of-the-lake"));
    assert!(door.needs.is_none(), "the last door in the game wants a key");
    assert!(!door.prose.is_empty(), "it opens onto nothing and says nothing");
    // **And the sentence that said nobody had decided is gone from here**,
    // because somebody has: it is four floors and a country. What it said
    // moved one map down onto the third town, which is the thing that has
    // nothing in it now — `the_third_town_has_no_shelves_and_says_so`.
    let said = door.prose.join(" ").to_lowercase();
    assert!(
        !said.contains("nobody has decided") && !said.contains("saved for later"),
        "the door is a way on and still says the writing stops at it: {said:?}"
    );
    assert!(said.contains("stair"), "it opens onto a stair and does not say so: {said:?}");
}

/// **Under the lake is a dungeon, not a sitting.**
///
/// The Drambus Stack's floors say `outside` and this does not, and the
/// difference is the whole of what makes one a sitting and the other a place:
/// you walk out of a dungeon, and a save taken in one reopens in it. Two
/// hundred and six steps down and two hundred and six back up is the budget.
#[test]
fn a_save_under_the_lake_reopens_under_the_lake() {
    let w = data::map(UNDER, D);
    assert!(w.outside.is_none(), "the bottom of the lake became one sitting");
    let mut st = WorldState::at_start(&w);
    st.map = UNDER.into();
    st.at = [6, 7];
    assert_eq!(gm2d_core::world::leave_the_sitting(&mut st, D), None);
    assert_eq!(st.map, UNDER, "a save under the lake reopened somewhere else");
    assert_eq!(st.at, [6, 7]);
    // And there is a way back up, which is what a dungeon has.
    let up = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Gate)
        .expect("no way back up the steps");
    assert_eq!(up.to.as_deref(), Some(&gm2d_core::world::overworld()[..]));
    // It lands you on the grating, which is where you came in.
    assert_eq!(up.at_to, Some(grating(&bambulon()).at));
}
