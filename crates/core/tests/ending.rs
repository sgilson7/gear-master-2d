//! The door in the wall, and the one screen that is not a loop.
//!
//! The key from the bottom of the Cave had nothing to open. This gives it a
//! door and the demo an ending — and each of the three things it needs is a
//! first: a place that is not there until it is, an errand gated on something
//! that is not another errand, and a screen that does not loop.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::quest;
use gm2d_core::world::{self, Allowances, Dir, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;

fn overworld() -> world::World {
    data::world(D)
}

fn the_door(w: &world::World) -> &world::PlaceDef {
    w.places.iter().find(|p| p.kind == PlaceKind::Door).expect("a door somewhere")
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
        !w.places_now(&g.world).iter().any(|p| p.id == door.id),
        "a hidden place is in the list the map draws from"
    );
    assert!(w.place_now(&g.world, door.at[0], door.at[1]).is_none());
    // But it is in the file, which is what makes it content rather than state.
    assert!(w.place_at(door.at[0], door.at[1]).is_some());

    // And walking onto it finds nothing there.
    g.world.at = [door.at[0] + 1, door.at[1]];
    let mut rng = g.rng.clone();
    let s = world::step(&w, &mut g.world, &mut rng, D, Dir::West, &Allowances::default());
    assert!(s.moved, "the tile itself is walkable");
    assert_eq!(s.door, None, "a hidden door opened");

    // Open, once the thing that opens it has happened.
    g.world.answered.push(key);
    assert!(w.places_now(&g.world).iter().any(|p| p.id == door.id));
    g.world.at = [door.at[0] + 1, door.at[1]];
    let s = world::step(&w, &mut g.world, &mut rng, D, Dir::West, &Allowances::default());
    assert_eq!(s.door.as_deref(), Some(door.id.as_str()), "the door is not answering");
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
            data::map(id, D).places.iter().filter_map(|p| p.drops.clone()).collect::<Vec<_>>()
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
        .find(|p| p.drops.as_deref() == Some(wants))
        .expect("something drops it");
    assert_eq!(
        opens, dropper.id,
        "the door appears on {opens:?} and the key comes off {:?}",
        dropper.id
    );
}

/// The ending says something, and says the demo is over.
#[test]
fn the_door_says_what_is_behind_it() {
    let w = overworld();
    let door = the_door(&w);
    assert!(!door.prose.is_empty(), "the door opens onto nothing at all");
    let said = door.prose.join(" ").to_lowercase();
    assert!(said.contains("demo"), "the ending does not say the demo is over: {said:?}");
    assert!(!door.shut.is_empty(), "a locked door that says nothing");
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
    let w = overworld();
    assert_eq!(
        w.place_at(
            w.places.iter().find(|p| p.id == place).expect("the place exists").at[0],
            w.places.iter().find(|p| p.id == place).expect("the place exists").at[1],
        )
        .map(|p| p.kind),
        Some(PlaceKind::Door),
        "the errand sends you somewhere that is not the door"
    );
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
        // an event's id, a door's own id, and an errand's word marker.
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
