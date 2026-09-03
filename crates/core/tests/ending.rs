//! The door in the wall, and what is on the other side of it.
//!
//! The key from the bottom of the Cave had nothing to open until M8.7, which
//! gave it a door and gave the demo an ending. **M11.1 took the ending off it.**
//! The door is a `Gate` now and it crosses onto the Treyway, which is the map
//! the anthology has been drawing as the edge of the paper for eight
//! milestones. Everything else about it is unchanged and still first of its
//! kind: a place that is not there until it is, an errand gated on something
//! that is not another errand, and a lock whose key falls off one tile.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::quest;
use gm2d_core::world::{self, Allowances, Dir, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;

/// The one place in the game that is hidden until something has happened.
const DOOR: &str = "the-door-in-the-wall";

fn overworld() -> world::World {
    data::world(D)
}

fn the_door(w: &world::World) -> &world::PlaceDef {
    w.places.iter().find(|p| p.id == DOOR).expect("the door in the western wall")
}

/// **Not drawn, not steppable, absent from `place_now`.**
///
/// Spawning a place at runtime was the other option and is rejected for the
/// reason the map is not in the save: places are content, and content is not
/// state.
#[test]
fn a_hidden_place_is_not_there_until_its_condition_holds() {
    let w = overworld();
    let door = the_door(&w).clone();
    let key = door.hidden_until.clone().expect("the door is conditional");

    let mut g = Game::new(7, "td");
    g.world = WorldState::at_start(&w);

    // Shut: nowhere in the list, nothing on the tile.
    assert!(
        !w.places_now(&g.world, &Allowances::default()).iter().any(|p| p.id == door.id),
        "a hidden place is in the list the map draws from"
    );
    assert!(w.place_now(&g.world, door.at[0], door.at[1], &Allowances::default()).is_none());
    // But it is in the file, which is what makes it content rather than state.
    assert!(w.place_at(door.at[0], door.at[1]).is_some());

    // And walking onto it finds nothing there.
    g.world.at = [door.at[0] + 1, door.at[1]];
    let mut rng = g.rng.clone();
    let s = world::step(&w, &mut g.world, &mut rng, D, Dir::West, &Allowances::default());
    assert!(s.moved, "the tile itself is walkable");
    assert_eq!(s.gate, None, "a hidden door opened");

    // Open, once the thing that opens it has happened.
    g.world.answered.push(key);
    assert!(w.places_now(&g.world, &Allowances::default()).iter().any(|p| p.id == door.id));
    g.world.at = [door.at[0] + 1, door.at[1]];
    let s = world::step(&w, &mut g.world, &mut rng, D, Dir::West, &Allowances::default());
    assert_eq!(s.gate.as_deref(), Some(door.id.as_str()), "the door is not answering");
}

/// **The door wants the key the boss drops.**
///
/// The same shape as `the_witchs_key_is_the_key_the_cave_wants`: a lock whose
/// key nothing hands out is a door nobody opens.
#[test]
fn the_door_wants_the_key_the_boss_drops() {
    let w = overworld();
    let door = the_door(&w);
    let wants = door.needs.as_deref().expect("the door wants something");

    let dropped: Vec<String> = data::MAPS
        .iter()
        .flat_map(|(id, _)| {
            data::map(id, D).places.iter().flat_map(|p| p.drops.clone()).collect::<Vec<_>>()
        })
        .collect();
    assert!(
        dropped.iter().any(|d| d == wants),
        "the door wants {wants:?} and nothing on any map hands one out: {dropped:?}"
    );

    // And what makes it appear is the tile that hands the key over.
    let opens = door.hidden_until.as_deref().expect("the door is conditional");
    let dropper = data::MAPS
        .iter()
        .flat_map(|(id, _)| data::map(id, D).places.clone())
        .find(|p| p.drops.iter().any(|d| d == wants))
        .expect("something drops it");
    assert_eq!(
        opens, dropper.id,
        "the door appears on {opens:?} and the key comes off {:?}",
        dropper.id
    );
}

/// **It crosses now, and it says so on the way through.**
///
/// M8.7 ended the demo here and the prose said as much in as many words. That
/// sentence was true for two blocks and is a lie the moment there is a map on
/// the far side, so the test that used to demand the word "demo" now demands
/// the two things that make it a crossing: somewhere real to arrive, and a
/// paragraph about arriving there.
#[test]
fn the_door_opens_onto_a_map_that_exists() {
    let w = overworld();
    let door = the_door(&w);
    assert_eq!(door.kind, PlaceKind::Gate, "the door stopped being a way through");
    let to = door.to.as_deref().expect("it opens onto nothing");
    assert!(
        data::MAPS.iter().any(|(m, _)| *m == to),
        "the door opens onto {to:?}, which is not a map"
    );
    assert_ne!(to, w.id, "the door opens onto the map it is in");
    assert!(!door.prose.is_empty(), "you cross out of Bambulon and nobody says anything");
    let said = door.prose.join(" ").to_lowercase();
    assert!(
        !said.contains("demo ends"),
        "the door still ends the demo, and there is a map behind it: {said:?}"
    );
    assert!(!door.shut.is_empty(), "a locked door that says nothing");
}

/// **A border lands you where you left off; a dungeon's mouth names its tile.**
///
/// The difference is `at_to`, and it is written in the map file rather than
/// decided in the shim. The Cave names one because a corridor has one door; the
/// door in the wall does not, because coming back to the Treyway's southern
/// corner every time would make it a chute.
#[test]
fn a_gate_without_a_landing_tile_is_a_border() {
    let w = overworld();
    assert!(the_door(&w).at_to.is_none(), "the door names a landing tile, so it is a chute");

    let cave = data::map("the-great-gear-cave", D);
    let up = cave
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Gate)
        .expect("the way back up");
    assert!(up.at_to.is_some(), "the Cave's mouth stopped naming its tile on the far side");
}

/// **An errand gated on something that is not another errand.**
#[test]
fn the_ending_errand_waits_for_the_boss() {
    let quests = data::quests();
    let q = quests
        .quests
        .iter()
        .find(|q| !q.requires_answered.is_empty())
        .expect("an errand gated on a flag");
    let mut g = Game::new(8, "td");
    g.world = WorldState::at_start(&overworld());

    assert_eq!(quest::stage(&g, q), quest::Stage::Locked, "it is on offer before the boss");
    for k in &q.requires_answered {
        g.world.answered.push(k.clone());
    }
    assert_eq!(quest::stage(&g, q), quest::Stage::Offered, "the boss is down and it is still shut");

    // It sends you to the door, and standing there is the whole of the doing.
    let place = q.goal.place().expect("it sends you somewhere");
    assert_eq!(place, DOOR, "the errand sends you somewhere that is not the door");
    quest::take(&mut g, &q.id).expect("the clerk gives it out");
    quest::on_arrival(&mut g, place);
    assert_eq!(quest::stage(&g, q), quest::Stage::Ready, "standing at the door was not enough");
}

/// Every conditional place names something that can actually happen.
///
/// A `hidden_until` nobody ever writes is a place nobody ever sees, and
/// nothing else in the game would say so.
#[test]
fn every_hidden_place_names_something_that_happens() {
    let events = data::events();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        // Everything any map can write into `answered`: a boss tile's own id,
        // an event's id, a gate's own id where it carries a paragraph, and an
        // errand's word marker.
        let mut writable: Vec<String> = Vec::new();
        for (other, _) in data::MAPS {
            for p in data::map(other, D).places {
                writable.push(p.id.clone());
            }
        }
        writable.extend(events.events.iter().map(|e| e.id.clone()));
        writable.extend(data::quests().quests.iter().map(|q| quest::spoken(&q.id)));
        for p in &w.places {
            let Some(k) = &p.hidden_until else { continue };
            assert!(
                writable.contains(k),
                "{}: hidden until {k:?}, which nothing ever writes",
                p.id
            );
            assert_ne!(k, &p.id, "{}: is hidden until itself", p.id);
        }
    }
}
