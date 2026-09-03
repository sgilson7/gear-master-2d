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
    let shops = data::shops();
    // **Every map.** An errand's giver, its turn-in and the place it sends you
    // to may each be on a different one since M11.2 — that is the point of the
    // two lines the field map added, and a check that only walked the first map
    // would have called all three of them nowhere.
    let places: Vec<String> = data::MAPS
        .iter()
        .flat_map(|(id, _)| data::map(id, D).places.iter().map(|p| p.id.clone()).collect::<Vec<_>>())
        .collect();
    for e in &q.quests {
        // A giver is a town or an event tile now, and either is fine — but it
        // has to be somewhere. A staged town counts: its map is coming.
        let known = |id: &str| places.iter().any(|p| p == id) || shops.town(id).is_some();
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
        assert!(
            !e.reward.is_empty() || !e.enchs.is_empty() || e.gold != 0,
            "{} pays nothing",
            e.id
        );
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
    let mut bare: Vec<String> = data::MAPS
        .iter()
        .flat_map(|(id, _)| {
            data::map(id, D)
                .places
                .iter()
                .filter(|p| p.kind == PlaceKind::Town)
                .map(|p| p.id.clone())
                .collect::<Vec<_>>()
        })
        .filter(|id| q.quests.iter().all(|e| &e.giver != id))
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
    for e in &q.quests {
        let Some(c) = e.goal.creature() else { continue };
        // Any map's pools, not the first map's. An errand handed out on the
        // field map is about what lives in the field.
        let regions: Vec<String> = data::MAPS
            .iter()
            .flat_map(|(id, _)| {
                data::map(id, D)
                    .regions
                    .iter()
                    .filter(|r| r.enemies.iter().any(|m| m.name == c))
                    .map(|r| r.id.clone())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(!regions.is_empty(), "{}: nothing anywhere in the world is a {c}", e.id);
    }
}

/// **An errand never asks for the rarest thing in the room.**
///
/// `draw_enemy` weights a pool by `(max + 1 − rating)`, so the hardest creature
/// in a region is the rarest one in it — which is right for fights and is a
/// *content* decision the moment something is farmed off it. `PLAN.md` §6b row
/// 1 is this problem seen from the drops' end: measured per creature the three
/// sets looked fine, and measured the way a player counts, one of them was
/// three and a half thousand fights away.
///
/// So: somewhere in the world there is a region where the creature an errand
/// names is drawn at least a fifth of the time. Not a statement about the pool
/// being fair — a statement about the errand being finishable in an evening.
#[test]
fn every_slaying_errand_asks_for_something_you_actually_meet() {
    let quests = data::quests();
    for e in &quests.quests {
        let Some(c) = e.goal.creature() else { continue };
        let mut best = 0;
        let mut where_ = String::new();
        for (id, _) in data::MAPS {
            for r in data::map(id, D).regions {
                if !r.enemies.iter().any(|m| m.name == c) {
                    continue;
                }
                let rated: Vec<i32> = r
                    .enemies
                    .iter()
                    .map(|m| gm2d_core::rating::creature_rating(m, D))
                    .collect();
                let max = rated.iter().copied().max().unwrap_or(0);
                let weights: Vec<i32> = rated.iter().map(|v| (max + 1 - v).max(1)).collect();
                let total: i32 = weights.iter().sum();
                let mine = r
                    .enemies
                    .iter()
                    .zip(&weights)
                    .find(|(m, _)| m.name == c)
                    .map(|(_, w)| *w)
                    .unwrap_or(0);
                let pct = mine * 100 / total.max(1);
                if pct > best {
                    best = pct;
                    where_ = format!("{id}/{}", r.id);
                }
            }
        }
        // An evening, not a lifetime. The number is the *expected wins* to
        // finish, which is what a player counts — see `drops.rs`'s rate, which
        // learned the same lesson from the other end.
        let wins = e.goal.count() as i32 * 100 / best.max(1);
        assert!(
            best > 0 && wins <= 60,
            "{}: {c} is drawn {best}% of the time at best ({where_}), so {} of them is \
             about {wins} wins",
            e.id,
            e.goal.count()
        );
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

// ------------------------------------------------------------------ the guide
//
// An errand that cannot say where to go is an errand a player has to be told
// about somewhere else. The log points at the map, and what it points at is
// core's answer — a page working it out would be a second copy of the rules
// about stages and goals.

/// **Every slaying errand names a creature some region actually holds.**
///
/// `QuestsData::parse` already refuses a creature that is not in the ladder.
/// This is the stronger claim the highlight depends on: it has to be *placed*.
/// A creature in the catalogue and in no region's pool cannot be met, so the
/// errand cannot be finished and nothing else in the game would say so.
#[test]
fn every_slaying_errand_names_a_creature_some_region_holds() {
    let quests = data::quests();
    let maps = data::all_maps(D);
    for q in &quests.quests {
        let Some(c) = q.goal.creature() else { continue };
        let regions: Vec<&str> = maps
            .iter()
            .flat_map(|w| w.regions_holding(c))
            .map(|r| r.id.as_str())
            .collect();
        assert!(
            !regions.is_empty(),
            "{}: {c:?} is in no region's pool, so the errand cannot be finished",
            q.id
        );
    }
}

/// Where each stage of an errand points.
#[test]
fn a_guide_points_where_the_stage_says() {
    let maps = data::all_maps(D);
    let quests = data::quests();
    let mut g = Game::new(9, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));

    let toads = quests.get("the-eyes-have-it").expect("the toad errand");

    // Untaken: go and be asked.
    let go = quest::guide(&g, toads, &maps);
    assert_eq!(go.places, vec!["the-end-of-all-gears".to_string()]);
    assert!(go.regions.is_empty(), "nothing to hunt until it is taken");

    // Taken: the regions the toads are in.
    quest::take(&mut g, &toads.id).unwrap();
    let go = quest::guide(&g, toads, &maps);
    assert!(go.places.is_empty(), "a hunt does not point at a person");
    assert!(
        go.regions.iter().any(|r| r == "the-end-of-all-gears"),
        "the pit holds Bog Toads and the guide does not say so: {go:?}"
    );

    // Full: back to whoever asked.
    for _ in 0..toads.goal.count() {
        g.character.give("Toad Eye");
    }
    assert_eq!(quest::stage(&g, toads), Stage::Ready);
    let go = quest::guide(&g, toads, &maps);
    assert_eq!(go.places, vec!["the-end-of-all-gears".to_string()]);

    // A word points at the tile you have to stand on, and then at whoever
    // wants to be told — which is the whole of what makes "go and tell them"
    // one errand rather than two.
    let heap = quests.get("the-count-is-wrong").expect("the heap errand");
    quest::take(&mut g, &heap.id).unwrap();
    let go = quest::guide(&g, heap, &maps);
    assert_eq!(go.places, vec!["the-end-of-all-gears".to_string()], "go and tell the office");
    assert_ne!(heap.giver, "the-end-of-all-gears", "or this is testing nothing");
}

/// A pin is one at a time, comes off by being pinned again, and is dropped
/// when the errand it names is finished.
#[test]
fn a_pin_is_one_errand_and_survives_being_walked_away_from() {
    let mut g = Game::new(3, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    quest::take(&mut g, "the-eyes-have-it").unwrap();
    quest::take(&mut g, "word-with-the-fencecutter").unwrap();

    assert_eq!(quest::pin(&mut g, "the-eyes-have-it"), Ok(true));
    assert_eq!(g.world.pinned.as_deref(), Some("the-eyes-have-it"));
    // A second pin replaces the first. Two is a map with two answers.
    assert_eq!(quest::pin(&mut g, "word-with-the-fencecutter"), Ok(true));
    assert_eq!(g.world.pinned.as_deref(), Some("word-with-the-fencecutter"));
    // Pinning the pinned one takes it off.
    assert_eq!(quest::pin(&mut g, "word-with-the-fencecutter"), Ok(false));
    assert_eq!(g.world.pinned, None);

    // And handing one in drops its pin, rather than ringing a place that has
    // nothing at it any more.
    quest::pin(&mut g, "the-eyes-have-it").unwrap();
    for _ in 0..5 {
        g.character.give("Toad Eye");
    }
    at_town(&mut g, "the-end-of-all-gears");
    quest::hand_in(&mut g, "the-eyes-have-it").unwrap();
    assert_eq!(g.world.pinned, None, "a finished errand is still pinned");
}
