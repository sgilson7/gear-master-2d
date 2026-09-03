//! The dense map, and the budget that makes it dense.
//!
//! M11.2. Kettleworks stops being a shelf with no ground under it, the Drambus
//! Stack stands in the middle of a field, and two questlines run across three
//! maps.
//!
//! **The density budget is the only new number in this milestone**, and it is
//! here rather than in a comment because Look Outside's lesson is checkable: a
//! map this size carries a game when everything on it answers, and "everything
//! answers" is a count.

use std::collections::HashSet;

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::world::{PlaceKind, World};

const D: Difficulty = Difficulty::Easy;
const FIELD: &str = "kettleworks-field";

/// How many of the field's four hundred tiles have to carry something.
///
/// `PLAN-M11.md` §M11.2 sets it: forty of four hundred, one in ten. Not a
/// ceiling — a floor, and it is a floor on *this* map rather than on every map,
/// because the Treyway is a country and a country is mostly ground.
const DENSITY: usize = 40;

fn field() -> World {
    data::map(FIELD, D)
}

/// **One tile in ten answers.**
///
/// A place, a card, or an examinable. The last of those is M11.2's own
/// category: an event with no choices, which is a post or a pond or a wall
/// somebody built out of rind — you read it, there is nothing to spend, and it
/// says the same thing on the ninth crossing as on the first.
#[test]
fn the_field_is_dense() {
    let w = field();
    let tiles = w.width as usize * w.height as usize;
    let answering: HashSet<[u8; 2]> = w.places.iter().map(|p| p.at).collect();
    assert_eq!(tiles, 400, "the field stopped being twenty by twenty");
    assert!(
        answering.len() >= DENSITY,
        "{} of {tiles} tiles answer, and the budget is {DENSITY}",
        answering.len()
    );
    // And they are not all in one corner: something in every quarter of it.
    for (qx, qy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let n = answering
            .iter()
            .filter(|[x, y]| (*x as usize >= 10) as usize == qx && (*y as usize >= 10) as usize == qy)
            .count();
        assert!(n >= 4, "the {qx},{qy} quarter has {n} things in it");
    }
}

/// Every readable on it has something to read, and the examinables outnumber
/// the cards — which is what makes it a place rather than a row of dialogues.
#[test]
fn most_of_the_field_is_something_to_look_at_rather_than_something_to_answer() {
    let w = field();
    let events = data::events();
    let (mut cards, mut examinables) = (0, 0);
    for p in w.places.iter().filter(|p| p.kind == PlaceKind::Event) {
        let e = events
            .get(&p.id)
            .unwrap_or_else(|| panic!("{} is placed and written nowhere", p.id));
        assert!(!e.prose.is_empty(), "{}: nothing to read", p.id);
        assert!(!e.title.is_empty(), "{}: no title", p.id);
        if e.is_examinable() {
            examinables += 1;
        } else {
            cards += 1;
        }
    }
    assert!(examinables > cards, "{examinables} examinables against {cards} cards");
    assert!(examinables >= 30, "only {examinables} things to look at");
}

/// **The Stack is in the middle, it is not rock, and it has a door.**
#[test]
fn the_drambus_stack_stands_in_the_field() {
    let w = field();
    let mut curd = Vec::new();
    for y in 0..w.height {
        for x in 0..w.width {
            if w.terrain_name(x, y) == "curd" {
                assert!(!w.passable(x, y), "({x}, {y}) is curd and you can walk into it");
                curd.push((x, y));
            }
        }
    }
    assert_eq!(curd.len(), 16, "the Stack is {} tiles", curd.len());
    // Middling, rather than against a wall. A tower at the edge of a map is a
    // building; one in the middle is a thing the map is arranged around.
    let (cx, cy) = (
        curd.iter().map(|(x, _)| *x as usize).sum::<usize>() / curd.len(),
        curd.iter().map(|(_, y)| *y as usize).sum::<usize>() / curd.len(),
    );
    assert!((5..15).contains(&cx) && (5..15).contains(&cy), "the Stack is at ({cx}, {cy})");

    // And a way in at the foot of it, on ground, next to the curd.
    let door = w
        .places
        .iter()
        .find(|p| p.id == "the-way-into-the-stack")
        .expect("the way into the Stack");
    let [dx, dy] = door.at;
    assert!(w.passable(dx, dy), "the way in is inside the Stack");
    // And the board you read about it is beside it rather than on it: an
    // errand that sends you to the door must not put you on floor five of a
    // tower you have not agreed to enter.
    let board = w.places.iter().find(|p| p.id == "the-stack-door").expect("the board outside it");
    assert_eq!(board.kind, PlaceKind::Event);
    assert_ne!(board.at, door.at, "the board and the doorway are one tile");
    assert_eq!(
        (board.at[0] as i32 - dx as i32).abs() + (board.at[1] as i32 - dy as i32).abs(),
        1,
        "the board about the door is not beside the door"
    );
    let touching = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].iter().any(|(ox, oy)| {
        let (nx, ny) = (dx as i32 + ox, dy as i32 + oy);
        w.in_bounds(nx, ny) && w.terrain_name(nx as u8, ny as u8) == "curd"
    });
    assert!(touching, "the door in the Stack does not touch the Stack");
    // **Five floors behind it, top down.** M11.2 wrote it as an event, because
    // the maps did not exist; M11.3 built them. What the order encodes is the
    // whole condition — you always enter the current top, so a floor is
    // reachable only while every floor above it is cleared and it is not, and
    // that is the ordering rather than a rule stated anywhere.
    assert_eq!(door.kind, PlaceKind::Gate);
    assert_eq!(door.floors.len(), 5, "the Stack is {} floors", door.floors.len());
    assert!(door.to.is_none(), "the door names a map as well as a stack");
    let names: Vec<&str> = door.floors.iter().map(|f| f.map.as_str()).collect();
    assert_eq!(
        names,
        ["the-drambus-stack-5", "the-drambus-stack-4", "the-drambus-stack-3",
         "the-drambus-stack-2", "the-drambus-stack-1"],
        "the floors are out of order, so the tower comes down the wrong way"
    );
    assert!(!door.shut.is_empty(), "a stack that comes down and says nothing");
}

/// **Kettleworks is on the ground now, and it trades and it wants things.**
///
/// The shelf and two errands have existed since M8 with no map under them.
/// `towns_anywhere_in_the_world_all_trade_and_all_want_something` is the
/// general form; this is the specific one, because the whole milestone is about
/// this town in particular.
#[test]
fn kettleworks_is_a_place_you_can_walk_into() {
    let w = field();
    let town = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Town)
        .expect("the field has no town on it");
    assert_eq!(town.id, "kettleworks");
    assert!(data::shops().town("kettleworks").is_some());
    assert!(!data::quests().at("kettleworks").is_empty());
    // And the errand it has had since M8 is reachable now: the Bone Archer it
    // asks for lives on a map you can get to from here.
    let quests = data::quests();
    let asked = quests.get("a-quiver-of-nocks").expect("the fletcher's errand");
    let creature = asked.goal.creature().expect("it is a slaying");
    let anywhere = data::MAPS.iter().any(|(id, _)| {
        data::map(id, D).regions.iter().any(|r| r.enemies.iter().any(|m| m.name == creature))
    });
    assert!(anywhere, "the fletcher wants {creature} and nothing anywhere is one");
}

// ------------------------------------------------------------ the questlines

/// **Two lines, three deep each, and each one leaves the map it starts on.**
///
/// A questline that never crosses a border is a questline about one map, and
/// the whole reason there are three maps is that they are one world.
#[test]
fn the_field_runs_two_lines_and_both_of_them_cross_a_map() {
    let quests = data::quests();
    // Where every place in the world lives, by map.
    let mut home: Vec<(String, &str)> = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            home.push((p.id.clone(), id));
        }
    }
    let map_of = |place: &str| home.iter().find(|(k, _)| k == place).map(|(_, m)| *m);

    for head in ["what-the-door-smells-of", "the-count-at-the-pond"] {
        // Walk the line forwards from its first errand.
        let mut line = vec![quests.get(head).unwrap_or_else(|| panic!("{head} is not an errand"))];
        loop {
            let last = line.last().unwrap().id.clone();
            match quests.quests.iter().find(|q| q.requires.iter().any(|r| *r == last)) {
                Some(next) => line.push(next),
                None => break,
            }
        }
        assert!(line.len() >= 3, "{head}: the line is {} errands long", line.len());

        // At least one step of it happens on a map that is not the field.
        let elsewhere = line.iter().any(|q| {
            q.goal.place().and_then(map_of).is_some_and(|m| m != FIELD)
                || map_of(&q.giver).is_some_and(|m| m != FIELD)
        });
        assert!(elsewhere, "{head}: every step of the line is on the field map");

        // And every step of it is finishable: the tally is its own, the place
        // exists, and the pay is something.
        for q in &line {
            assert!(
                !q.reward.is_empty() || q.gold != 0 || !q.enchs.is_empty(),
                "{}: pays nothing",
                q.id
            );
            if let Some(p) = q.goal.place() {
                assert!(map_of(p).is_some(), "{}: sends you to {p:?}, which is nowhere", q.id);
            }
        }
    }
}

/// **The field's shelf is not the field's reward.**
///
/// Six new errands pay six components, and a reward you could have bought off
/// the counter in the same room makes the errand a slow way to shop. Checked
/// against every shelf in the game rather than against Kettleworks', because
/// the point is the walk and not the room.
#[test]
fn nothing_the_field_pays_is_on_a_shelf() {
    let shops = data::shops();
    let on_sale: HashSet<&str> =
        shops.towns.iter().flat_map(|t| t.stock.iter().map(|s| s.as_str())).collect();
    let quests = data::quests();
    for id in [
        "what-the-door-smells-of",
        "what-comes-off-it",
        "the-frame-in-the-shallows",
        "the-stack-is-shorter",
        "the-count-at-the-pond",
        "the-drawer-of-eight-hundred",
        "the-nine-and-the-eleven",
    ] {
        let q = quests.get(id).unwrap_or_else(|| panic!("{id} is not an errand"));
        for r in &q.reward {
            assert!(!on_sale.contains(r.as_str()), "{id} pays {r:?}, which is on a shelf");
            assert!(
                gm2d_core::piece::is_event_only(r),
                "{id} pays {r:?}, which the ladder could deal you"
            );
        }
    }
}
