//! Where an ench comes from.
//!
//! Every trading town kept a bench until M10, and every one of them sold every
//! priced ench to any licensee. That was right when enching was one class's
//! whole identity and the worry was stranding a licensee from their own class.
//! It is wrong for the reason the shelves stopped rolling in M7: **a thing
//! every town sells is not a thing you went and got.**
//!
//! Three sources now, and a lint per source. A tree awards one, an errand pays
//! one, and one man on the Verge road sells the rest — and he is not there
//! until level ten.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::{Allowances, PlaceKind};

const D: Difficulty = Difficulty::Easy;

fn benches() -> Vec<gm2d_core::world::PlaceDef> {
    data::all_maps(D)
        .iter()
        .flat_map(|w| w.places.clone())
        .filter(|p| p.kind == PlaceKind::Bench)
        .collect()
}

// ------------------------------------------------------------------ the ask

/// **No town sells an ench.** The ask, stated as a lint.
#[test]
fn no_town_sells_an_ench() {
    for w in data::all_maps(D) {
        for p in w.places.iter().filter(|p| p.kind == PlaceKind::Town) {
            assert!(
                p.sells.is_empty(),
                "{} sells {:?}, and a town selling an ench is what M10 took out",
                p.id,
                p.sells
            );
        }
    }
    // And the shelves themselves hold components, not enchs — nothing in
    // `shops.json` names one.
    let all = data::enchs();
    let ids: Vec<&str> = all.enchs.iter().map(|e| e.id.as_str()).collect();
    for t in &data::shops().towns {
        for line in &t.stock {
            assert!(!ids.contains(&line.as_str()), "{} stocks the ench {line}", t.id);
        }
    }
}

/// **Every ench comes from somewhere**, and that is the point of narrowing it.
///
/// An ench on no bench, in no node and paid by no errand is an orphan — the
/// same lint the errands and the drops each have, and for the same reason:
/// nothing else in the game would say so.
#[test]
fn every_ench_comes_from_somewhere() {
    let enchs = data::enchs();
    let tree = data::skills();
    let quests = data::quests();
    let sold: Vec<String> = benches().iter().flat_map(|p| p.sells.clone()).collect();
    let granted: Vec<String> = tree
        .trees
        .iter()
        .flat_map(|t| &t.nodes)
        .flat_map(|n| &n.effects)
        .filter_map(|e| match e {
            gm2d_core::skills::Effect::GivesEnch { ench } => Some(ench.clone()),
            _ => None,
        })
        .collect();
    let paid: Vec<String> = quests.quests.iter().flat_map(|q| q.enchs.clone()).collect();

    for e in &enchs.enchs {
        let from: Vec<&str> = [
            sold.contains(&e.id).then_some("a bench"),
            granted.contains(&e.id).then_some("a node"),
            paid.contains(&e.id).then_some("an errand"),
        ]
        .into_iter()
        .flatten()
        .collect();
        assert!(!from.is_empty(), "{} comes from nowhere at all", e.id);
    }
    // And nothing is handed out twice: a node granting what a bench sells makes
    // one of the two pointless, and which one is not obvious from either end.
    for id in &granted {
        assert!(!sold.contains(id), "{id} is both awarded and for sale");
        assert!(!paid.contains(id), "{id} is both awarded and paid for an errand");
    }
    for id in &paid {
        assert!(!sold.contains(id), "{id} is both paid and for sale");
    }
}

/// A bench is refused at load if it sells nothing, or something that is not an
/// ench, or one nobody prices.
#[test]
fn a_bench_that_sells_nothing_real_is_refused() {
    let terrain = data::TERRAIN_JSON;
    let mut map: serde_json::Value = serde_json::from_str(data::TILES_JSON).unwrap();
    let places = map["places"].as_array_mut().unwrap();
    let base = places.iter().find(|p| p["kind"] == "bench").cloned().expect("one ships");

    let load = |places: serde_json::Value| {
        let mut m: serde_json::Value = serde_json::from_str(data::TILES_JSON).unwrap();
        m["places"] = places;
        gm2d_core::world::World::load(terrain, &m.to_string(), D)
    };

    let mut junk = base.clone();
    junk["sells"] = serde_json::json!(["a-thing-nobody-invented"]);
    let e = load(serde_json::json!([junk])).unwrap_err();
    assert!(e.contains("a-thing-nobody-invented"), "{e}");

    let mut empty = base.clone();
    empty["sells"] = serde_json::json!([]);
    assert!(load(serde_json::json!([empty])).unwrap_err().contains("nothing on it"));

    // **A priceless ench is on nobody's bench**, which is the errands' half of
    // the rule and has been since M8: a reward you could have bought makes the
    // errand a slow way to shop.
    let priceless = data::enchs()
        .enchs
        .iter()
        .find(|e| e.price.is_none())
        .map(|e| e.id.clone())
        .expect("an errand pays one");
    let mut freebie = base.clone();
    freebie["sells"] = serde_json::json!([priceless]);
    assert!(load(serde_json::json!([freebie])).unwrap_err().contains("not for sale"));

    // And only a bench sells anything.
    let mut town = base;
    town["kind"] = serde_json::json!("town");
    assert!(load(serde_json::json!([town])).unwrap_err().contains("only a bench"));
}

// ---------------------------------------------------------------- the vendor

/// He is not there below the level, and he is there at it.
#[test]
fn the_vendor_is_not_there_before_the_level() {
    let w = data::world(D);
    let van = benches().into_iter().next().expect("one ships");
    let need = van.hidden_until_level.expect("he waits for a level");
    assert!(need > 5, "a vendor you meet before the class fork is not much of a wait");

    let state = gm2d_core::world::WorldState::at_start(&w);
    let below = Allowances { level: need - 1, ..Allowances::default() };
    let at = Allowances { level: need, ..Allowances::default() };

    assert!(w.place_now(&state, van.at[0], van.at[1], &below).is_none());
    assert!(!w.places_now(&state, &below).iter().any(|p| p.id == van.id));
    assert!(w.place_now(&state, van.at[0], van.at[1], &at).is_some());
    assert!(w.places_now(&state, &at).iter().any(|p| p.id == van.id));

    // **Not drawn is not steppable.** A place absent from the map and walked
    // onto is half hidden, which is the rule the door in the wall set.
    let mut rng = gm2d_core::rng::Rng::new(4);
    let beside = gm2d_core::world::WorldState { at: [van.at[0], van.at[1] + 1], ..state.clone() };
    for (allowed, want) in [(below, None), (at, Some(van.id.clone()))] {
        let mut s = beside.clone();
        let step = gm2d_core::world::step(
            &w, &mut s, &mut rng, D, gm2d_core::world::Dir::North, &allowed,
        );
        assert!(step.moved, "could not reach the van's tile at all");
        assert_eq!(step.bench, want);
    }
}

/// He stands past the last crossing, so the walk to him is a walk.
#[test]
fn the_vendor_is_past_what_the_map_already_gates() {
    let w = data::world(D);
    let van = benches().into_iter().next().unwrap();
    let region = w.region_at(van.at[0], van.at[1]).expect("he stands in a region");
    let guard = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Crossing && p.guards.as_deref() == Some(region.id.as_str()))
        .unwrap_or_else(|| panic!("{} is not behind a crossing", region.id));
    assert!(
        guard.needs_level.unwrap_or(0) < van.hidden_until_level.unwrap_or(0),
        "the crossing asks for as much as the van does, so he is not a second thing to earn"
    );
    assert!(w.passable(van.at[0], van.at[1]), "he is standing in the scenery");
}

// ----------------------------------------------------------------- the tree

/// **The Patent is not inert before the vendor.**
///
/// Three of its eight nodes tune the spin, and the spin is not a stat — it is
/// The Ponkey Turn, which was 90 Fnorp off a bench. With the benches closed, a
/// class offered at level five would have had an identity and three eighths of
/// a tree that did nothing until ten. The node at the root of the spin spine is
/// called Bench Rights, and it hands you the bench.
#[test]
fn the_patent_is_not_inert_before_the_vendor() {
    let tree = data::skills();
    let patent = tree.tree_for_class(gm2d_core::ench::LICENSED_CLASS).expect("it has a tree");
    let granted: Vec<String> = patent
        .nodes
        .iter()
        .flat_map(|n| &n.effects)
        .filter_map(|e| match e {
            gm2d_core::skills::Effect::GivesEnch { ench } => Some(ench.clone()),
            _ => None,
        })
        .collect();
    assert!(!granted.is_empty(), "the licensed class's tree awards no ench at all");

    // And it is reachable with one point, on the day the class is taken.
    let root = patent
        .nodes
        .iter()
        .filter(|n| n.requires.is_empty() && n.cost <= 1)
        .find(|n| {
            n.effects.iter().any(|e| matches!(e, gm2d_core::skills::Effect::GivesEnch { .. }))
        })
        .expect("the ench is behind a prerequisite, so the class is inert until it is bought");

    let mut c = Character::starting();
    c.class = Some(gm2d_core::ench::LICENSED_CLASS.to_string());
    c.skill_points = 1;
    c.take_skill(&tree, &root.id).expect("takeable at level five with one point");
    assert!(!c.enchs().is_empty(), "took the node and the rack is empty");

    // Which is the whole of it: a licensee can bolt one on straight away.
    let piece = c.owned[0];
    let id = c.enchs()[0].clone();
    c.attach_ench(&id, piece).expect("a licensee with an ench can use it");
}

/// **Derived, never banked.** The tree's answer moves when the tree does.
#[test]
fn a_granted_ench_is_derived_and_not_banked() {
    let tree = data::skills();
    let node = tree
        .trees
        .iter()
        .flat_map(|t| &t.nodes)
        .find(|n| n.effects.iter().any(|e| matches!(e, gm2d_core::skills::Effect::GivesEnch { .. })))
        .expect("something grants one");

    let mut c = Character::starting();
    assert!(c.enchs().is_empty());
    c.skills_taken.push(node.id.clone());
    assert!(!c.enchs().is_empty(), "taking the node handed over nothing");

    // Nothing was written down: the save carries the node, not the ench.
    assert!(
        c.enchs_owned.is_empty(),
        "the ench was banked into the save, so retuning the node would not reach a \
         character who had already taken it"
    );
}

/// A node naming an ench nobody has heard of is a point spent on nothing.
#[test]
fn a_node_that_names_no_ench_is_refused() {
    let mut raw: serde_json::Value = serde_json::from_str(data::SKILLS_JSON).unwrap();
    raw["trees"][0]["nodes"][0]["effect"] =
        serde_json::json!({ "gives_ench": { "ench": "the-nothing-at-all" } });
    let why = gm2d_core::skills::SkillsData::parse(&raw.to_string()).unwrap_err();
    assert!(why.contains("the-nothing-at-all"), "{why}");
}

// ----------------------------------------------------------- what you own

/// An ench that is bolted on is still one you have.
///
/// `enchs_owned` meant *loose* and `attach` moved an entry out of it. That
/// stopped working the moment a tree could grant one, because there is nothing
/// to take an entry out of a derived list — so it means *banked* now and
/// `enchs_loose` subtracts what is on the board.
#[test]
fn attaching_does_not_spend_the_ench() {
    let mut c = Character::with_all_pieces();
    c.class = Some(gm2d_core::ench::LICENSED_CLASS.to_string());
    let id = data::enchs().enchs[0].id.clone();
    c.give_ench(&id);
    assert_eq!(c.enchs_loose(&id), 1);

    let piece = c.owned[0];
    c.attach_ench(&id, piece).unwrap();
    assert_eq!(c.enchs_loose(&id), 0, "it is on something");
    assert_eq!(c.enchs().iter().filter(|e| **e == id).count(), 1, "and it is still yours");

    c.detach_ench(piece);
    assert_eq!(c.enchs_loose(&id), 1, "taking it off gives it back");
    assert_eq!(
        c.enchs().iter().filter(|e| **e == id).count(),
        1,
        "and does not hand out a second one"
    );
}

/// A save written before `enchs_owned` changed meaning comes back whole.
#[test]
fn a_save_from_before_the_benches_closed_still_opens() {
    let mut g = Game::new(11, "td");
    g.character = Character::with_all_pieces();
    g.character.class = Some(gm2d_core::ench::LICENSED_CLASS.to_string());
    let id = data::enchs().enchs[0].id.clone();
    let piece = g.character.owned[0];
    // The old shape, written by hand: bolted on, and absent from `enchs_owned`
    // because attaching used to take it out.
    g.character.enchanted.push(gm2d_core::ench::Ench { on: piece, id: id.clone(), active: true });
    assert_eq!(g.character.enchs_loose(&id), 0, "the fixture is the old shape");

    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it reloads");
    assert_eq!(
        back.character.enchs().iter().filter(|e| **e == id).count(),
        1,
        "the bolted ench was not carried across"
    );
    assert_eq!(back.character.enchs_loose(&id), 0, "and it is still on the component");
    let mut after = back;
    after.character.detach_ench(piece);
    assert_eq!(after.character.enchs_loose(&id), 1, "taking it off would have lost it");
}
