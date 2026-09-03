//! Can you actually meet the catalogue?
//!
//! The old version of this file asked that about a randomised shop and went
//! with the campaign in `48203ee`, leaving four imports and no tests. The
//! question survives the shop that prompted it, and is sharper now: a fixed
//! shelf can be *checked*, where a random one could only be sampled.
//!
//! What has to hold is that the three towns are three places rather than one
//! shelf in three costumes, and that a character who owns a handle and a blade
//! and twenty-eight Fnorp can walk into the nearest one and buy something.

use gm2d_core::character::Character;
use gm2d_core::data;
use gm2d_core::piece::{PieceKind, SlotKind, CATALOG};
use gm2d_core::shop::{shelf, STARTING_GOLD};
use std::collections::HashSet;

#[test]
fn every_town_stocks_something_and_stocks_it_from_the_catalogue() {
    // `ShopsData::parse` already refuses an unknown name; this is the check
    // that a town exists at all and is not an empty room.
    let shops = data::shops();
    assert!(!shops.towns.is_empty(), "nowhere sells anything");
    for t in &shops.towns {
        assert!(!t.stock.is_empty(), "{} sells nothing", t.id);
        for name in &t.stock {
            assert!(
                CATALOG.iter().any(|d| d.name == *name),
                "{} stocks {name:?}, which is not a component",
                t.id
            );
        }
    }
}

/// **Every town on the map trades, and every shelf is meant for a town.**
///
/// This used to be a set equality, which held while the first map carried
/// every town there was. It cannot now: the first map has one town, and
/// Kettleworks and High Wick are written, shelved and given errands while the
/// maps that will carry them do not exist.
///
/// The direction that still has to hold is that a town you can walk into
/// trades — a room you can enter and leave no different is not a place.
///
/// The other direction is replaced by `STAGED`, which is the point of this
/// test now: content waiting for a map is fine and content waiting for nothing
/// is an orphan, and the only difference between them is somebody having
/// written the name down. A third stray shelf fails here.
///
/// **Kettleworks came off this list in M11.2**, which is the thing the list was
/// for: it had a shelf and two errands for three blocks and no ground under it,
/// and the field map is `PLAN.md` §6a row 1 finally paid. High Wick is still
/// waiting, on purpose.
const STAGED: &[&str] = &["high-wick"];

#[test]
fn towns_anywhere_in_the_world_all_trade_and_all_want_something() {
    use gm2d_core::world::PlaceKind;
    // **Every map**, since M11.2 puts a town on the second one. A check that
    // walked only the first map would have gone on calling Kettleworks staged
    // while a player was standing in it.
    let mut towns: HashSet<String> = HashSet::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, gm2d_core::combat::Difficulty::Easy);
        towns.extend(
            w.places.iter().filter(|p| p.kind == PlaceKind::Town).map(|p| p.id.clone()),
        );
    }
    assert!(!towns.is_empty(), "the world has no town at all");

    let shops = data::shops();
    let quests = data::quests();
    for t in &towns {
        assert!(shops.town(t).is_some(), "{t} is on a map and sells nothing");
        assert!(!quests.at(t).is_empty(), "{t} is on a map and wants nothing");
    }

    // Anything shelved for a place that is on no map is staged, and has to be
    // named as such.
    let shelves: HashSet<&str> = shops.towns.iter().map(|t| t.id.as_str()).collect();
    let mut orphans: Vec<&str> = shelves
        .iter()
        .copied()
        .filter(|id| !towns.contains(*id) && !STAGED.contains(id))
        .collect();
    orphans.sort();
    assert!(
        orphans.is_empty(),
        "shelves for nowhere: {orphans:?} — put the town on a map or add it to STAGED"
    );
    let mut waiting: Vec<&str> =
        STAGED.iter().copied().filter(|id| towns.contains(*id)).collect();
    waiting.sort();
    assert!(
        waiting.is_empty(),
        "{waiting:?} are on the map now and still listed as staged"
    );
}

/// **Three towns, three places.**
///
/// The whole reason a shelf is fixed is so that "the Kettleworks has the
/// plating" is a thing a player can learn. Two towns dealing mostly the same
/// components would make that false while looking fine.
#[test]
fn no_two_towns_are_the_same_shop() {
    let shops = data::shops();
    for a in &shops.towns {
        for b in &shops.towns {
            if a.id >= b.id {
                continue;
            }
            let sa: HashSet<&str> = a.stock.iter().map(|s| s.as_str()).collect();
            let sb: HashSet<&str> = b.stock.iter().map(|s| s.as_str()).collect();
            let shared = sa.intersection(&sb).count();
            let smaller = sa.len().min(sb.len());
            assert!(
                shared * 2 < smaller,
                "{} and {} share {shared} of {smaller} — that is one shop in two costumes",
                a.id,
                b.id
            );
        }
    }
}

/// A starting character can afford to leave the first town better than they
/// arrived.
///
/// The kit is two components now, so the shelf is not decoration — it is where
/// the second and third come from. A starter shelf a new character cannot
/// reach is the M4 soft-lock wearing a different hat.
#[test]
fn the_starting_purse_buys_something_at_the_starting_town() {
    let shops = data::shops();
    let c = Character::starting();
    assert_eq!(c.gold, STARTING_GOLD);
    let shelf = shelf(&shops, "the-end-of-all-gears", &[]);
    let affordable: Vec<_> = shelf.iter().filter(|o| o.price <= c.gold).collect();
    assert!(
        affordable.len() >= 4,
        "{} of {} entries are within {} Fnorp; the first shelf has to be reachable",
        affordable.len(),
        shelf.len(),
        c.gold
    );
    // And enough of it to finish a second item, not just to own more scrap.
    // A helmet wants a frame; gloves and greaves want a material and a mold.
    let cheapest_total: i32 = {
        let mut prices: Vec<i32> = shelf.iter().map(|o| o.price).collect();
        prices.sort();
        prices.iter().take(4).sum()
    };
    assert!(
        cheapest_total <= c.gold,
        "the four cheapest things in the pit cost {cheapest_total} and you arrive with {}",
        c.gold
    );
}

/// The starter shelf can actually finish a piece of gear.
///
/// Affording four components means nothing if no four of them assemble. The
/// helmet recipe is the one a new character can reach, so it is the one
/// checked: a frame and a plating, both on the shelf, both inside the purse.
#[test]
fn the_first_shelf_can_finish_a_helmet() {
    let shops = data::shops();
    let shelf = shelf(&shops, "the-end-of-all-gears", &[]);
    let has = |k: PieceKind| {
        shelf.iter().any(|o| o.def.kind == k && o.def.slot == SlotKind::Helmet)
    };
    assert!(has(PieceKind::Frame), "no helmet frame in the pit, so no helmet");
    assert!(has(PieceKind::Plating), "a frame with nothing to plate it");
    let cost: i32 = [PieceKind::Frame, PieceKind::Plating]
        .iter()
        .filter_map(|&k| {
            shelf
                .iter()
                .filter(|o| o.def.kind == k && o.def.slot == SlotKind::Helmet)
                .map(|o| o.price)
                .min()
        })
        .sum();
    assert!(cost <= STARTING_GOLD, "the cheapest helmet in the pit costs {cost}");
}

/// Buying takes an entry off the shelf and leaves the rest where they were.
#[test]
fn a_bought_entry_stays_where_it_was_and_is_marked_sold() {
    let shops = data::shops();
    let before = shelf(&shops, "the-end-of-all-gears", &[]);
    let after = shelf(&shops, "the-end-of-all-gears", &[("the-end-of-all-gears".into(), 1)]);
    assert_eq!(before.len(), after.len(), "a sold entry is marked, never dropped");
    for (a, b) in before.iter().zip(after.iter()) {
        assert_eq!(a.index, b.index);
        assert_eq!(a.def.name, b.def.name, "the shelf renumbered itself");
    }
    assert!(after[1].sold && !after[0].sold && !after[2].sold);
}
