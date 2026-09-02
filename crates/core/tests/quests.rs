//! The errands, walked from both ends.
//!
//! The first one is the shape every later one is cut from: a town asks for
//! five of something, the something drops off the creature that has it, and
//! the reward is two components that only work as a pair. Each of those three
//! is a place it can go wrong quietly, so each is checked here.

use gm2d_core::combat::{Difficulty, MonsterSpec, Outcome};
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::quest::{self, Goal, Stage};

const D: Difficulty = Difficulty::Easy;

fn creature(name: &str) -> &'static MonsterSpec {
    gm2d_core::combat::LADDER.iter().find(|s| s.name == name).expect("a creature by that name")
}

/// Stand the player in a town so `town_here`-shaped rules apply.
fn at_town(g: &mut Game, id: &str) {
    let w = data::world(D);
    let p = w.places.iter().find(|p| p.id == id).expect("a town by that id");
    g.world.at = p.at;
}

#[test]
fn the_shipped_errands_parse_and_name_things_that_exist() {
    // `QuestsData::parse` checks the creature and every component; this is the
    // check that there is an errand at all and that it is somebody's.
    let q = data::quests();
    assert!(!q.quests.is_empty(), "no errands anywhere");
    let w = data::world(D);
    let shops = data::shops();
    let places: Vec<&str> = w.places.iter().map(|p| p.id.as_str()).collect();
    for e in &q.quests {
        // A giver is a town or an event tile now, and either is fine — but it
        // has to be somewhere. A staged town counts: its map is coming.
        let known = |id: &str| places.contains(&id) || shops.town(id).is_some();
        assert!(known(&e.giver), "{} is given out by {:?}, which is nowhere", e.id, e.giver);
        let back = e.turn_in.as_deref().unwrap_or(&e.giver);
        assert!(known(back), "{} is handed in at {:?}, which is nowhere", e.id, back);
        if let Some(p) = e.goal.place() {
            assert!(known(p), "{} sends you to {p:?}, which is nowhere", e.id);
        }
        for r in &e.requires {
            assert!(q.get(r).is_some(), "{} requires {r}, which is not an errand", e.id);
        }
        assert!(!e.brief.is_empty() && !e.thanks.is_empty(), "{} says nothing", e.id);
        assert!(!e.reward.is_empty() || e.gold != 0, "{} pays nothing", e.id);
    }
}

/// **No two errands share a tally.**
///
/// `holding` counts a token by name across the whole bag, so two errands
/// asking for the same one would each see the other's — take both, kill five
/// toads, and hand in twice for one walk. Distinct tokens is what makes the
/// count mean anything.
#[test]
fn every_errand_counts_something_of_its_own() {
    use std::collections::HashMap;
    let q = data::quests();
    let mut seen: HashMap<&str, &str> = HashMap::new();
    for e in &q.quests {
        let Some(t) = e.goal.token() else { continue };
        if let Some(other) = seen.insert(t, &e.id) {
            panic!("{} and {} both tally {t:?}", other, e.id);
        }
    }
}

/// Every town on the map asks for something.
///
/// A town is a shelf and an errand. One with nothing to want is a room you
/// walk into, spend in, and leave — which is a shop, not a place.
#[test]
fn every_town_has_an_errand() {
    use gm2d_core::world::PlaceKind;
    let q = data::quests();
    let w = data::world(D);
    let mut bare: Vec<&str> = w
        .places
        .iter()
        .filter(|p| p.kind == PlaceKind::Town)
        .map(|p| p.id.as_str())
        .filter(|id| q.quests.iter().all(|e| e.giver != *id))
        .collect();
    bare.sort();
    assert!(bare.is_empty(), "towns that want nothing: {bare:?}");
}

/// What every errand asks for lives somewhere a player can reach it.
///
/// An errand naming a creature that is in no region's pool is an errand that
/// cannot be finished, and nothing else in the game would say so.
#[test]
fn every_errand_names_a_creature_that_is_actually_out_there() {
    let q = data::quests();
    let w = data::world(D);
    for e in &q.quests {
        let Some(c) = e.goal.creature() else { continue };
        let regions: Vec<&str> = w
            .regions
            .iter()
            .filter(|r| r.enemies.iter().any(|m| m.name == c))
            .map(|r| r.id.as_str())
            .collect();
        assert!(!regions.is_empty(), "{}: nothing anywhere on the map is a {c}", e.id);
    }
}

/// The starter town's errand, from offered to done.
#[test]
fn five_toads_and_a_walk_home() {
    let mut g = Game::new(7, "td");
    at_town(&mut g, "the-end-of-all-gears");
    let quests = data::quests();
    let q = quests.get("the-eyes-have-it").expect("the toad errand");
    let Goal::Slay { count, token, .. } = &q.goal else { panic!("the toad errand is a slaying") };
    let (count, token) = (*count, token.clone());

    assert_eq!(quest::stage(&g, q), Stage::Offered);
    // Nothing drops before it is asked for. A bag filling with eyes nobody
    // wants is litter, and this is the line that keeps it out.
    assert!(quest::on_victory(&mut g, "Bog Toad").is_empty(), "a toad paid out before being asked");

    quest::take(&mut g, &q.id).expect("the errand can be taken");
    assert_eq!(quest::stage(&g, q), Stage::Carrying { have: 0, want: count });
    assert!(quest::take(&mut g, &q.id).is_err(), "taken twice");

    for i in 1..=count {
        let got = quest::on_victory(&mut g, "Bog Toad");
        assert_eq!(got, vec![token.clone()], "toad {i} dropped nothing");
        if i < count {
            assert_eq!(quest::stage(&g, q), Stage::Carrying { have: i, want: count });
            assert!(quest::hand_in(&mut g, &q.id).is_err(), "handed in {i} of {count}");
        }
    }
    assert_eq!(quest::stage(&g, q), Stage::Ready);
    // And it stops paying out. Five is five: a sixth eye would be a thing in
    // the bag that means nothing and cannot be handed in.
    assert!(quest::on_victory(&mut g, "Bog Toad").is_empty(), "a sixth toad still paid");

    let purse = g.character.gold;
    let given = quest::hand_in(&mut g, &q.id).expect("five eyes is five eyes");
    assert_eq!(given, q.reward, "the reward is not what was written down");
    assert_eq!(quest::stage(&g, q), Stage::Done);
    assert_eq!(g.character.gold, purse + q.gold);
    assert_eq!(quest::holding(&g, &token), 0, "the eyes are still in the bag");
    for name in &q.reward {
        assert_eq!(quest::holding(&g, name), 1, "{name} was not handed over");
    }
    assert!(quest::hand_in(&mut g, &q.id).is_err(), "handed in twice");
    assert!(quest::on_victory(&mut g, "Bog Toad").is_empty(), "still dropping after it is done");
}

/// **The reward has to be usable, and the two halves only work together.**
///
/// A book with no spell assembles nothing, so handing over one without the
/// other would be a reward you carry around and cannot fit anywhere. The
/// errand pays both; this is the check that both is enough.
#[test]
fn what_the_errand_pays_assembles_into_a_weapon() {
    use gm2d_core::piece::SlotKind;
    let mut g = Game::new(11, "td");
    at_town(&mut g, "the-end-of-all-gears");
    let quests = data::quests();
    let q = quests.get("the-eyes-have-it").unwrap();

    quest::take(&mut g, &q.id).unwrap();
    for _ in 0..q.goal.count() {
        quest::on_victory(&mut g, "Bog Toad");
    }
    quest::hand_in(&mut g, &q.id).unwrap();

    // Clear the frame the starting kit is on and seat what the errand paid.
    let ids: Vec<_> = g.character.owned.clone();
    for id in &ids {
        g.character.loadout.remove_anywhere(*id);
    }
    // First fit, scanning the frame. Enough for the question being asked,
    // which is only whether the two of them go on at all.
    let mut seated = 0;
    for name in &q.reward {
        let id = ids
            .iter()
            .copied()
            .find(|&p| g.character.registry.def(p).name == *name)
            .expect("the reward is owned");
        let rows = g.character.loadout.slot(SlotKind::Weapon).rows();
        let mut done = false;
        for y in 0..rows {
            for x in 0..gm2d_core::slot::SLOT_W {
                if g.character.equip(id, SlotKind::Weapon, x, y).is_ok() {
                    seated += 1;
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
        }
    }
    assert_eq!(seated, q.reward.len(), "the reward does not fit on a starting frame");

    let items = g.character.combat_items();
    assert!(
        items.iter().any(|i| i.slot == SlotKind::Weapon),
        "a book and its spell assembled nothing: {:?}",
        g.character.report(SlotKind::Weapon).items.iter().map(|i| &i.status).collect::<Vec<_>>()
    );
}

/// A starting character can reach the errand's target.
///
/// The kit is a handle and a blade. If it cannot beat a Bog Toad then the
/// first errand in the game is one you have to grind past before you can
/// start it, which is the wrong way round.
#[test]
fn the_starting_kit_can_beat_what_the_first_errand_asks_for() {
    use gm2d_core::character::Character;
    let mut c = Character::starting();
    c.apply_preset();
    let log = gm2d_core::combat::simulate_at(
        c.player_stats(),
        &c.combat_items(),
        creature("Bog Toad"),
        D,
    );
    assert_eq!(log.outcome, Outcome::Victory, "the starting kit cannot beat a toad");
}
