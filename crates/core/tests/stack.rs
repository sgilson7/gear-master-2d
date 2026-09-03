//! The Drambus Stack: five floors, one sitting each, and a tower that drops.
//!
//! M11.3. Two things are new and neither of them is in the save, which is the
//! point of both: **which floor the door opens onto is derived** from the boss
//! tiles already in `answered`, and **a floor is one sitting** because its map
//! says where a sitting on it ends.
//!
//! The counter `PLAN-M11.md` §M11.3 asks for is not here. It would have been a
//! second answer to a question `answered` already answers, and the first rule
//! this project keeps is that a derived number is never banked.

use gm2d_core::character::Character;
use gm2d_core::combat::{self, Difficulty, Outcome};
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::{self, PlaceKind, World, WorldState};

mod common;

const D: Difficulty = Difficulty::Easy;
const FIELD: &str = "kettleworks-field";
const FLOORS: [&str; 5] = [
    "the-drambus-stack-5",
    "the-drambus-stack-4",
    "the-drambus-stack-3",
    "the-drambus-stack-2",
    "the-drambus-stack-1",
];

fn field() -> World {
    data::map(FIELD, D)
}

fn door(w: &World) -> &world::PlaceDef {
    w.places.iter().find(|p| p.id == "the-way-into-the-stack").expect("the way in")
}

/// **The counter drives which floor the door opens, at all six values.**
///
/// The acceptance criterion, and it is written as a walk down the tower rather
/// than as six separate assertions because the sixth value is only reachable by
/// having passed the other five.
#[test]
fn the_door_opens_onto_the_floor_that_is_still_standing() {
    let w = field();
    let d = door(&w);
    let mut st = WorldState::at_start(&w);

    for (i, want) in FLOORS.iter().enumerate() {
        assert_eq!(d.floors_cleared(&st), i, "the tower thinks {i} floors are gone");
        assert_eq!(
            d.opens_onto(&st),
            Some(*want),
            "with {i} floors down the door should open onto {want}"
        );
        // Clear it, the way beating its boss does.
        st.answered.push(format!("{want}-boss"));
    }

    assert_eq!(d.floors_cleared(&st), 5);
    assert_eq!(d.opens_onto(&st), None, "the Stack came all the way down and the door is still there");
    assert!(!d.shut.is_empty(), "there is nothing left and the map says nothing about it");
}

/// **A floor you have cleared does not reopen.**
///
/// The tower is shorter, not deeper. Clearing floor three out of order is not
/// something a player can do, so the test does it by hand — which is the only
/// way to find out that the ordering is the condition rather than a coincidence
/// of how the floors happen to be reached.
#[test]
fn a_cleared_floor_never_opens_again() {
    let w = field();
    let d = door(&w);
    let mut st = WorldState::at_start(&w);
    st.answered.push("the-drambus-stack-5-boss".into());
    st.answered.push("the-drambus-stack-4-boss".into());
    assert_eq!(d.opens_onto(&st), Some("the-drambus-stack-3"));
    // And answering a floor twice does not count twice.
    st.answered.push("the-drambus-stack-5-boss".into());
    assert_eq!(d.opens_onto(&st), Some("the-drambus-stack-3"));
}

/// Every floor is a floor: one boss, one sitting, and no way out but through.
#[test]
fn every_floor_is_one_sitting_with_one_thing_on_it() {
    for id in FLOORS {
        let w = data::map(id, D);
        assert_eq!(w.id, id, "{id} is filed under the wrong name");
        assert_eq!((w.width, w.height), (10, 10), "{id} is not a floor, it is a map");
        assert_eq!(
            w.outside.as_deref(),
            Some(FIELD),
            "{id} is not one sitting, so a save inside it stays inside it"
        );
        let bosses: Vec<&world::PlaceDef> =
            w.places.iter().filter(|p| p.kind == PlaceKind::Boss).collect();
        assert_eq!(bosses.len(), 1, "{id} has {} things standing on it", bosses.len());
        let b = bosses[0];
        assert_eq!(b.id, format!("{id}-boss"));
        assert!(b.creature.is_some(), "{id}: a boss tile with nothing on it");
        assert!(!b.drops.is_empty(), "{id}: a floor that pays nothing");
        assert!(
            b.drops.iter().any(|d| d == "Map Shard"),
            "{id}: the tower is the shards' faucet and this floor pays none"
        );
        assert!(!b.prose.is_empty(), "{id}: the floor comes down and says nothing");
        // No gate out. The way out is the boss, or the tab.
        assert!(
            !w.places.iter().any(|p| p.kind == PlaceKind::Gate),
            "{id} has a way out, so it is a dungeon and not a sitting"
        );
    }
}

/// **It gets harder on the way down, and every floor's boss is the worst thing
/// on it.**
///
/// Measured off `creature_rating` rather than declared, the same way a region's
/// danger is. A floor whose pool is harder than its boss is a floor where the
/// thing at the end is a relief.
#[test]
fn the_stack_climbs_as_it_comes_down() {
    let mut last = 0;
    for id in FLOORS {
        let w = data::map(id, D);
        let region = &w.regions[0];
        let boss = w.places.iter().find_map(|p| p.creature.as_deref()).expect("a boss");
        let rated = gm2d_core::rating::creature_rating(
            gm2d_core::combat::creature(boss).expect("in the ladder"),
            D,
        );
        assert!(
            rated > region.danger,
            "{id}: the boss rates {rated} and the floor averages {}",
            region.danger
        );
        assert!(rated > last, "{id}: {rated} after {last}, so the tower gets easier");
        last = rated;
    }
    // And the bottom of it is harder than anything on the map it stands in.
    let hardest_outside = field().regions.iter().map(|r| r.danger).max().unwrap();
    assert!(last > hardest_outside * 3 / 2);
}

/// **Beating the floor's boss puts you outside, and the tower is a floor
/// shorter.**
///
/// The whole loop, walked: stand on the boss, settle a win, and find yourself
/// in the field with the door pointing one floor further down.
#[test]
fn clearing_a_floor_kicks_you_out_and_drops_the_tower() {
    let w = field();
    let d = door(&w).clone();
    let mut g = Game::new(11, "td");
    g.world = WorldState::at_start(&w);
    // Where you were standing in the field before you went in. `at_start`
    // leaves `map` empty, which resolves to the *first* map — so it is set
    // before `remember`, or the field's position is written down as
    // Bambulon's and the kick lands you in the pit.
    g.world.map = FIELD.into();
    g.world.at = [d.at[0], d.at[1] + 1];
    g.world.remember();

    let top = data::map(FLOORS[0], D);
    let boss = top.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("a boss").clone();
    g.world.map = FLOORS[0].into();
    g.world.at = boss.at;

    // Answer it the way `settle` does on a win, then end the sitting.
    g.world.answered.push(boss.id.clone());
    let out = world::leave_the_sitting(&mut g.world, D);
    assert_eq!(out.as_deref(), Some(FIELD), "clearing a floor left you inside it");
    assert_eq!(g.world.map, FIELD);
    // **Beside the door, not on it.** The shim writes down the tile the step
    // started from rather than the doorway it ended on, so being put out does
    // not leave you one keypress from walking straight back in — which is what
    // it did, and which the walker read as *the tower is where I already am*.
    assert_eq!(g.world.at, [d.at[0], d.at[1] + 1], "the kick put you back on the doorstep");
    assert_ne!(g.world.at, d.at);
    assert_eq!(d.floors_cleared(&g.world), 1);
    assert_eq!(d.opens_onto(&g.world), Some(FLOORS[1]), "the next entry is the same floor");
}

/// **A save taken inside a floor reopens outside it.**
///
/// `PLAN-M11.md` §8 row 3, answered yes. The same one function the kick uses,
/// which is why they cannot disagree.
#[test]
fn a_sitting_you_walked_away_from_is_over() {
    let mut st = WorldState::at_start(&field());
    st.map = FLOORS[2].into();
    st.at = [4, 1];
    let out = world::leave_the_sitting(&mut st, D);
    assert_eq!(out.as_deref(), Some(FIELD));
    assert_eq!(st.map, FIELD);
    assert!(field().passable(st.at[0], st.at[1]), "put outside and standing in the Stack");

    // And a map that is not a sitting is left alone.
    let mut on_the_road = WorldState::at_start(&data::world(D));
    let was = on_the_road.at;
    assert_eq!(world::leave_the_sitting(&mut on_the_road, D), None);
    assert_eq!(on_the_road.at, was);
}

/// **Going back into a floor starts it at the beginning.**
///
/// A gate with no landing tile lands you where you left off — which is right
/// for a border and wrong for a sitting, because where you left off on a floor
/// is the tile the boss is standing next to.
#[test]
fn a_sitting_begins_at_the_beginning() {
    let top = data::map(FLOORS[0], D);
    let mut st = WorldState::default();
    st.positions.push((FLOORS[0].to_string(), [8, 1]));
    assert_eq!(top.arrival(&st), [top.start.0, top.start.1]);
    // Where a border still remembers.
    let treyway = data::map("the-treyway", D);
    st.positions.push(("the-treyway".to_string(), [7, 8]));
    assert_eq!(treyway.arrival(&st), [7, 8]);
}

/// **The tower's last floor is the mark everything downstream reads.**
///
/// There is no `tower_dropped` flag, and there does not need to be one: floor
/// one is reachable only once the four above it are gone, so its boss being in
/// `answered` *is* the tower being down. M11.4's lake reads exactly that id.
#[test]
fn the_bottom_floor_is_the_whole_condition() {
    let w = field();
    let d = door(&w);
    let mut st = WorldState::at_start(&w);
    for f in &d.floors[..4] {
        st.answered.push(f.cleared.clone());
    }
    assert_ne!(d.opens_onto(&st), None, "four floors down and the tower is gone");
    st.answered.push("the-drambus-stack-1-boss".into());
    assert_eq!(d.opens_onto(&st), None, "the bottom floor is down and the door is still open");
    assert_eq!(d.floors_cleared(&st), 5);
}

/// **Somebody outside is waiting for it to come down.**
///
/// `PLAN-M11.md` asks the field's questline to touch the tower so that M11.3
/// has a witness. The witness is the clerk at Kettleworks, and what makes her
/// one is `requires_answered` on the bottom floor's boss — an errand gated on
/// something that is not another errand, which is the mechanism M8.7 built for
/// the door in the wall and this is its second user.
#[test]
fn the_field_has_a_witness_to_the_tower_coming_down() {
    let quests = data::quests();
    let q = quests.get("the-stack-is-shorter").expect("the fourth of the Stack line");
    assert_eq!(q.requires_answered, vec!["the-drambus-stack-1-boss".to_string()]);

    let mut g = Game::new(12, "td");
    g.world = WorldState::at_start(&field());
    assert_eq!(
        gm2d_core::quest::stage(&g, q),
        gm2d_core::quest::Stage::Locked,
        "she is asking for it before the tower is down"
    );
    // Every floor, because the line in front of it has to be done too.
    for id in ["what-the-door-smells-of", "what-comes-off-it", "the-frame-in-the-shallows"] {
        g.world.quests_done.push(id.into());
    }
    for f in FLOORS {
        g.world.answered.push(format!("{f}-boss"));
    }
    assert_eq!(
        gm2d_core::quest::stage(&g, q),
        gm2d_core::quest::Stage::Offered,
        "the Stack is down and she still has nothing to say"
    );
}

/// **The climb is possible, and the floors are what cost you.**
///
/// Not a golden transcript. `PLAN-M11.md` asks for fixtures per boss, and a
/// fixture would pin *how* five existing ladder creatures fight — which the
/// ladder's own tests already do and which nothing in this milestone changes.
///
/// **What this pins moved in M11.7.** The first version asked that the best
/// board the game hands out take two to four of the five bosses, on the theory
/// that a tower whose bosses all fall is a tower that is free. The measurement
/// said it took three — and that the *second* floor's boss beat it, which is
/// not a cost, it is a wall: the tower cannot be dropped, so the lake cannot be
/// drained, so the ending is unreachable, and five hundred and ninety-seven
/// tests were green through it.
///
/// So the boss half is a floor rather than a band now, and lives in
/// `reach.rs::every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten`
/// with everything else that stands on a tile. What is measured here is where
/// the cost actually is: **the walk to the boss.** Every floor's pool holds
/// something this board loses to, because `draw_enemy` makes the hardest member
/// the rarest and a region's teeth are the fight you sometimes lose.
#[test]
fn the_floors_cost_more_than_the_things_at_the_end_of_them() {
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let beats = |name: &str| {
        let m = combat::creature(name).expect("in the ladder");
        // Each floor is its own sitting, so each fight opens rested: the wear
        // is spent on the walk to the door and not carried down the tower.
        combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D).outcome
            == Outcome::Victory
    };

    for id in FLOORS {
        let w = data::map(id, D);
        let boss = w.places.iter().find_map(|p| p.creature.as_deref()).expect("a boss");
        assert!(beats(boss), "{id}: {boss} cannot be beaten, so the tower cannot be dropped");
        let pool = &w.regions[0].enemies;
        let lost = pool.iter().filter(|m| !beats(m.name)).count();
        assert!(
            lost >= 1,
            "{id}: every one of the {} things wandering this floor is a win, so the walk \
             to the boss costs nothing",
            pool.len()
        );
        assert!(
            lost < pool.len(),
            "{id}: nothing on this floor can be beaten, so it is a wall and not a floor"
        );
    }

    // And the other end: the kit you start with does not walk into a tower.
    let mut fresh = Character::starting();
    fresh.apply_preset();
    let top = data::map(FLOORS[0], D);
    let first = combat::creature(
        top.places.iter().find_map(|p| p.creature.as_deref()).expect("a boss"),
    )
    .expect("in the ladder");
    let log = combat::simulate_at(fresh.player_stats(), &fresh.combat_items(), first, D);
    assert_ne!(
        log.outcome,
        Outcome::Victory,
        "an Oak Handle and an Iron Blade cleared the top floor of the Drambus Stack"
    );
}
