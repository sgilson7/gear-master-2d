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

mod common;


#[test]
fn every_town_stocks_something_and_stocks_it_from_the_catalogue() {
    // `ShopsData::parse` already refuses an unknown name; this is the check
    // that a town exists at all and is not an empty room.
    let shops = data::shops();
    assert!(!shops.towns.is_empty(), "nowhere sells anything");
    for t in &shops.towns {
        // **An unwritten town is an empty room on purpose**, and the list is
        // where that is said. See `UNWRITTEN`.
        //
        // **And a wing need not sell anything, because a wing is a counter
        // rather than a town.** The clerk's desk came down to keep an
        // inventory: it has errands and no stock at all, and a desk with
        // nothing on it is what that looks like. What it may *not* be is a
        // shelf with nothing and nobody — so a wing that sells nothing has to
        // want something, which is the same question this asks of a town one
        // field along.
        if t.wing_of.is_some() {
            assert!(
                !t.stock.is_empty() || !data::quests().at(&t.id).is_empty(),
                "{} is a wing that sells nothing and wants nothing, which is a counter \
                 nobody stands at",
                t.id
            );
            continue;
        }
        // **A town's shelf is the town's and its wings'.** The third town sells
        // nothing under its own id and has two counters standing in it — the
        // clerk's desk and what came down from High Wick — which is what a
        // player walks up to. `UNWRITTEN` is empty and asserted empty since
        // M22.3, so the exemption that used to carry this town is gone and the
        // question is asked of every town there is.
        assert!(
            !stocked_between_them(&shops, &t.id).is_empty()
                || UNWRITTEN.contains(&t.id.as_str()),
            "{} sells nothing, and neither does anything standing in it",
            t.id
        );
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
/// **Emptied in M22.3 and asserted empty rather than deleted.** High Wick came
/// down as a wing of the third town, so there is nothing staged — and the
/// assertion is what keeps that a claim rather than an absence. A shelf that
/// quietly grew no ground under it would read as an oversight again.
const STAGED: &[&str] = &[];

/// What a town and everything standing in it sell between them.
///
/// **Whether or not a wing has arrived**, because this is a question about what
/// the town *is* rather than about a particular afternoon. The third town sells
/// nothing under its own id and has two counters in it, which is what a player
/// walks up to and is not a thing a state can be planted to hide.
fn stocked_between_them(shops: &gm2d_core::shop::ShopsData, town: &str) -> Vec<String> {
    shops
        .towns
        .iter()
        .filter(|t| t.id == town || t.wing_of.as_deref() == Some(town))
        .flat_map(|t| t.stock.clone())
        .collect()
}

// `UNWRITTEN` lives in `common`, because `quests.rs` asks the same question of
// the same town — see the const.
use common::UNWRITTEN;

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
    // **Everything that can ever stand in this town**, which is the town's own
    // shelf and every wing of it — *whether or not it has arrived*. A wing that
    // arrives on the last rung of a chain is still what this town trades in;
    // asking with an empty state would say the third town sells nothing, which
    // is true on the first afternoon and is not what this lint is about.
    for t in &towns {
        if UNWRITTEN.contains(&t.as_str()) {
            // And the exception is asserted rather than skipped: an unwritten
            // town that quietly grew a shelf is a list that has gone stale.
            assert!(
                shops.town(t).is_some_and(|s| s.stock.is_empty()),
                "{t} is listed as unwritten and has a shelf"
            );
            assert!(quests.at(t).is_empty(), "{t} is listed as unwritten and wants something");
            continue;
        }
        assert!(
            !stocked_between_them(&shops, t).is_empty(),
            "{t} is on a map and nothing standing in it sells anything"
        );
        let counters: Vec<String> = std::iter::once(t.clone())
            .chain(shops.towns.iter().filter(|w| w.wing_of.as_deref() == Some(t.as_str()))
                   .map(|w| w.id.clone()))
            .collect();
        assert!(
            counters.iter().any(|c| !quests.at(c).is_empty()),
            "{t} is on a map and nothing standing in it wants anything"
        );
    }
    let mut written: Vec<&str> =
        UNWRITTEN.iter().copied().filter(|id| !towns.contains(*id)).collect();
    written.sort();
    assert!(written.is_empty(), "{written:?} are listed as unwritten and are on no map");

    // Anything shelved for a place that is on no map is staged, and has to be
    // named as such.
    //
    // **A wing is neither a town nor staged**, and before M22 there was no
    // third thing to be: a wing is a counter standing inside somebody else's
    // town, so where it is written down is its host. A wing whose host is on no
    // map is *still* an orphan, and this is what says so — the host is checked
    // rather than assumed, in `a_wing_has_a_host_on_a_map`.
    let shelves: HashSet<&str> = shops
        .towns
        .iter()
        .filter(|t| t.wing_of.is_none())
        .map(|t| t.id.as_str())
        .collect();
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
            // Nought of nought is not one shop in two costumes.
            if UNWRITTEN.contains(&a.id.as_str()) || UNWRITTEN.contains(&b.id.as_str()) {
                continue;
            }
            // **Nor is a desk.** A wing may carry errands and no stock — the
            // clerk came down to keep an inventory rather than to trade — and
            // an empty shelf shares nought of nought with everybody. A wing
            // that *does* stock something is asked the question, because two
            // counters in one town dealing the same components is the same
            // failure one street along.
            if a.stock.is_empty() || b.stock.is_empty() {
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
/// **The guard is the same and the thing guarded moved.** The kit is two
/// components, so somewhere in the first town has to sell the third — a first
/// shop a new character cannot reach is the M4 soft-lock wearing a different
/// hat, and that has not changed.
///
/// What changed is *which counter*. This asked it of the shelf, because until
/// M12.1 the shelf was the only counter there was and its prices had to be
/// small enough to be reachable. There is a floor under it now, and M12.1a
/// charges the shelf five times the catalogue precisely so that it can stop
/// being the place a beginner shops. So the question is asked of the barrel,
/// which is what the beginner's counter is now.
///
/// The shelf keeps a weaker guard of its own — one line of it is affordable on
/// arrival and two are not — in `barrel.rs`, which is where the tier that set
/// the number lives.
#[test]
fn the_first_shop_buys_something_at_the_starting_town() {
    let shops = data::shops();
    let c = Character::starting();
    assert_eq!(c.gold, STARTING_GOLD);
    let barrel = gm2d_core::shop::barrel(&shops);
    assert!(!barrel.is_empty(), "there is no barrel, so there is no floor");
    let affordable: Vec<_> = barrel.iter().filter(|o| o.price <= c.gold).collect();
    assert!(
        affordable.len() >= 4,
        "{} of {} barrel entries are within {} Fnorp; the first counter has to be reachable",
        affordable.len(),
        barrel.len(),
        c.gold
    );
    // And enough of it to finish a second item, not just to own more scrap.
    let cheapest_total: i32 = {
        let mut prices: Vec<i32> = barrel.iter().map(|o| o.price).collect();
        prices.sort();
        prices.iter().take(4).sum()
    };
    assert!(
        cheapest_total <= c.gold,
        "the four cheapest things in the barrel cost {cheapest_total} and you arrive with {}",
        c.gold
    );
}

/// The first shop can actually finish a piece of gear.
///
/// Affording four components means nothing if no four of them assemble. The
/// helmet recipe is the one a new character can reach, so it is the one
/// checked: a frame and a plating, both for sale, both inside the purse.
///
/// Asked of the barrel for the reason above — and it is a stronger question
/// there than it ever was of the shelf, because the barrel is the same in
/// every town, so this holds at whichever one a player reaches first.
#[test]
fn the_first_shop_can_finish_a_helmet() {
    let shops = data::shops();
    let barrel = gm2d_core::shop::barrel(&shops);
    let cheapest = |k: PieceKind| {
        barrel
            .iter()
            .filter(|o| o.def.kind == k && o.def.slot == SlotKind::Helmet)
            .map(|o| o.price)
            .min()
    };
    let frame = cheapest(PieceKind::Frame).expect("no helmet frame in the barrel, so no helmet");
    let plating = cheapest(PieceKind::Plating).expect("a frame with nothing to plate it");
    assert!(
        frame + plating <= STARTING_GOLD,
        "the cheapest helmet a beginner can buy costs {}",
        frame + plating
    );
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
