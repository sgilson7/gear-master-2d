//! M14.4 — the country under the country, and the two doors that open together.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const HERE: &str = "the-undercountry";
const BOTH: [&str; 2] = ["the-ninth-surveyor", "the-bottom-of-the-bottom"];

/// **Neither bottom opens until both are down, and the first one you finish
/// says which is missing.**
///
/// `PLAN-M14.md` §1.6, and it is why `needs_all` is a different field from
/// `hidden_until_all`: a door you cannot see cannot show you a refusal, and
/// **a sealed door that says nothing is a bug report**.
///
/// Negative-tested by giving one bottom's door a single-boss `needs_all`: it
/// opened on the wrong dungeon and this named it.
#[test]
fn both_bottoms_open_together_and_not_before() {
    let doors = [("the-sump-4", "the-sump-4-way-on"), ("the-silt-stair-4", "the-silt-stair-4-way-on")];
    for (map, id) in doors {
        let w = data::map(map, D);
        let door = w.places.iter().find(|p| p.id == id).unwrap_or_else(|| panic!("no {id}"));
        assert_eq!(door.kind, PlaceKind::Gate);
        assert_eq!(door.to.as_deref(), Some(HERE), "{id} opens somewhere else");
        let mut want = door.needs_all.clone();
        want.sort();
        let mut both: Vec<String> = BOTH.iter().map(|s| s.to_string()).collect();
        both.sort();
        assert_eq!(want, both);
        // **Drawn from the first visit.** Not hidden: the whole of §1.6 is
        // that you see it and are told what it wants.
        assert!(door.hidden_until.is_none() && door.hidden_until_all.is_empty(), "{id} is hidden");
        assert!(!door.shut.is_empty(), "{id} refuses without a sentence of its own");
    }

    // Nothing down: shut, and the refusal names both.
    let mut g = Game::default();
    let sump = data::map("the-sump-4", D);
    let door = sump.places.iter().find(|p| p.id == "the-sump-4-way-on").expect("the door");
    assert_eq!(g.unlock(door), gm2d_core::game::Unlocked::Shut);
    let why = g.sealed_because(door, D).expect("it said nothing");
    assert!(why.contains(" and "), "one bottom named where two are wanted: {why}");

    // One down: still shut, and it names the *other* one by the name a player
    // would recognise rather than by a tile id.
    g.world.answered.push("the-ninth-surveyor".into());
    assert_eq!(g.unlock(door), gm2d_core::game::Unlocked::Shut);
    let why = g.sealed_because(door, D).expect("it said nothing");
    assert!(
        why.contains("bottom of the bottom"),
        "the refusal does not name the dungeon still standing: {why}"
    );
    assert!(!why.contains("Ninth"), "it is still asking for the one that is done: {why}");
    assert!(!why.contains("the-"), "the refusal is reading out tile ids: {why}");

    // Both down: open, and no sentence.
    g.world.answered.push("the-bottom-of-the-bottom".into());
    assert_eq!(g.unlock(door), gm2d_core::game::Unlocked::Open);
    assert_eq!(g.sealed_because(door, D), None);
}

/// **Marbulon's door opens onto the same map, and it is the one you will use.**
///
/// It is on the starting map, so it is the only one of the three ways in that
/// is not four floors down something. **Hidden rather than sealed**, and the
/// difference is which side you are standing on: a door in the shallows that is
/// there from the first afternoon is a secret with a signpost on it, and a door
/// at the bottom of a dungeon that is not there is a room you walk out of.
#[test]
fn marbulons_door_opens_onto_the_same_map() {
    let w = data::map("west-bambulon", D);
    let door = w
        .places
        .iter()
        .find(|p| p.id == "the-door-in-the-shallows")
        .expect("no door in the shallows");
    assert_eq!(door.to.as_deref(), Some(HERE));
    let mut want = door.hidden_until_all.clone();
    want.sort();
    let mut both: Vec<String> = BOTH.iter().map(|s| s.to_string()).collect();
    both.sort();
    assert_eq!(want, both);

    // She is sitting in front of it, one tile south.
    let her = w.place_at(3, 10).expect("Marbulon moved");
    assert_eq!(her.id, "marbulons-door");
    assert_eq!(door.at, [3, 9], "the door is not the one she is facing away from");

    // Not there until both are down, and there the moment they are.
    let mut st = WorldState::default();
    let a = Allowances::default();
    assert!(!gm2d_core::world::place_is_there(door, &st, &a));
    st.answered.push(BOTH[0].into());
    assert!(!gm2d_core::world::place_is_there(door, &st, &a), "one bottom opened it");
    st.answered.push(BOTH[1].into());
    assert!(gm2d_core::world::place_is_there(door, &st, &a), "both are down and it is not there");

    // **Her answer is the gate's paragraph and not a third choice on her
    // card.** §6 asks for the choice; her event is spent the moment you take
    // either of her errands — and her errands are the questline that unlocks
    // the Cave, so everybody has — which makes a third choice on it a choice
    // nobody can ever reach.
    let hers = data::events().get("marbulons-door").expect("Marbulon").choices.len();
    assert_eq!(hers, 2, "a third question was added to a card that is spent");
    assert!(!door.prose.is_empty(), "the door opens and she says nothing");
    assert!(
        door.prose.join(" ").contains("Forty-two"),
        "she counted forty-one the first time and does not count now"
    );
}

/// **Three ways in, three ways back, and each lands beside the door it came
/// from.**
///
/// A gate that puts you across the map from the one you walked through is a
/// gate you have to find twice.
#[test]
fn every_gate_lands_beside_its_door() {
    let here = data::map(HERE, D);
    let ways: Vec<&gm2d_core::world::PlaceDef> =
        here.places.iter().filter(|p| p.kind == PlaceKind::Gate).collect();
    assert_eq!(ways.len(), 3, "the Undercountry has {} ways off it", ways.len());

    for out in &ways {
        let to = out.to.as_deref().expect("a gate to nowhere");
        let far = data::map(to, D);
        let landing = out.at_to.expect("a way back that names no tile");
        // The gate on the far side that comes back here.
        let back = far
            .places
            .iter()
            .find(|p| p.kind == PlaceKind::Gate && p.to.as_deref() == Some(HERE))
            .unwrap_or_else(|| panic!("{to} has no way into the Undercountry"));
        let d = (landing[0] as i32 - back.at[0] as i32).abs()
            + (landing[1] as i32 - back.at[1] as i32).abs();
        assert!(
            d <= 1,
            "{}: coming back from {to} lands at {landing:?}, {d} tiles from the door at {:?}",
            out.id,
            back.at
        );
        // And the same in the other direction.
        let in_landing = back.at_to.expect("a way in that names no tile");
        let d = (in_landing[0] as i32 - out.at[0] as i32).abs()
            + (in_landing[1] as i32 - out.at[1] as i32).abs();
        assert!(
            d <= 1,
            "{}: coming in from {to} lands at {in_landing:?}, {d} tiles from the way back at {:?}",
            back.id,
            out.at
        );
    }
}

/// **The third town is empty and says so, and that is not a placeholder.**
///
/// `PLAN-M14.md` §1.5. A shelf invented to keep a lint quiet would be content
/// nobody asked for standing exactly where the content that *was* asked for has
/// to go — so the town ships with a name, a start tile, a region and nothing to
/// buy, and `avail.rs`'s `UNWRITTEN` is where that is written down.
#[test]
fn the_third_town_has_no_shelves_and_says_so() {
    let w = data::map(HERE, D);
    let town = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Town)
        .expect("the country under the country has no town on it");
    assert_eq!(town.id, "the-third-town");
    assert!(!town.name.is_empty(), "a town with no name at all");

    let shops = data::shops();
    let shelf = shops.town(&town.id).expect("no counter at all");
    assert!(shelf.stock.is_empty(), "the empty town sells {} things", shelf.stock.len());
    assert!(shelf.commissions.is_empty(), "the empty town takes orders");
    assert!(data::quests().at(&town.id).is_empty(), "the empty town wants something");

    // **And the sentence that says so is on the map**, one tile south of the
    // town, where the door under the lake used to carry it.
    let stop = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Door)
        .expect("nothing here says the writing stops");
    let said = stop.prose.join(" ").to_lowercase();
    assert!(said.contains("nobody has decided"), "the last screen does not say what it is");
    // It moved rather than being copied: the lake's door is a way on now.
    let lake = data::map("under-the-lake", D);
    let was = lake
        .places
        .iter()
        .find(|p| p.id == "the-door-under-the-lake")
        .expect("the door");
    assert!(
        !was.prose.join(" ").to_lowercase().contains("nobody has decided"),
        "two screens in the game say the writing stops"
    );
}

/// **Twelve maps became twenty-one, and every walkable tile of every one of
/// them is still reachable from where you arrive.**
///
/// The M11.7 lesson stated over the new maps: a content block needs a
/// reachability *measurement*, and the measurement has to name the tile it
/// starts from.
#[test]
fn reachability_derives_over_twenty_one_maps() {
    assert_eq!(data::MAPS.len(), 21, "the game ships {} maps", data::MAPS.len());
    for (id, _) in data::MAPS {
        let mut opened = WorldState::default();
        opened.map = (*id).to_string();
        opened.flags = data::map(id, D).drains.iter().map(|d| d.when.clone()).collect();
        let w = data::map_now(id, D, &opened);
        let a = Allowances::default();
        let mut seen: std::collections::BTreeSet<(u8, u8)> = Default::default();
        let mut q = vec![w.start];
        seen.insert(w.start);
        while let Some((x, y)) = q.pop() {
            for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let (nx, ny) = (nx as u8, ny as u8);
                if w.walkable(nx, ny, &a) && seen.insert((nx, ny)) {
                    q.push((nx, ny));
                }
            }
        }
        // Every place on the map is somewhere the walker can get to, which is
        // the half `the_whole_map_is_reachable_from_the_start` does not ask.
        for p in &w.places {
            assert!(
                seen.contains(&(p.at[0], p.at[1])),
                "{id}: {} at {:?} cannot be walked to from the arrival tile",
                p.id,
                p.at
            );
        }
    }
}
