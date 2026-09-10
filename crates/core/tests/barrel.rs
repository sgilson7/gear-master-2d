//! The bargain barrel. M12.1.
//!
//! **The block's thesis, and the cheapest lever on it.** M12.0 measured a
//! board about half covered for the whole playable game and a greaves grid at
//! 0% for fourteen levels. The barrel is a permanent, fixed, cheap bin under
//! every town's counter: it never runs out, it is the same everywhere, and
//! between them its entries assemble one complete item in each of the five
//! grids and nothing better than that.

mod common;

use gm2d_core::piece::{PieceKind, SlotKind, CATALOG};
use gm2d_core::shop;

fn barrel() -> Vec<&'static gm2d_core::piece::PieceDef> {
    shop::barrel(&gm2d_core::data::shops()).into_iter().map(|o| o.def).collect()
}

#[test]
fn the_barrel_charges_the_catalogue_price_and_nothing_else() {
    // **§C.3: the screen shows the price actually charged.** The rest of this
    // file reads `PieceDef::price`, which is what the barrel is *stocked*
    // with — and a check that only reads that is blind to `shop::barrel`
    // marking anything up. Found by breaking it: adding fifty Fnorp to every
    // offer passed all seven other checks in this file.
    //
    // There is no barrel discount and no barrel mark-up. The entries are cheap
    // because they are cheap components.
    for o in shop::barrel(&gm2d_core::data::shops()) {
        assert_eq!(
            o.price, o.def.price,
            "{} is stocked at {} and offered at {}",
            o.def.name, o.def.price, o.price
        );
        assert!(o.price <= shop::BARREL_CEILING, "{} is offered at {} Fnorp", o.def.name, o.price);
        assert!(o.price > 0, "{} is offered free", o.def.name);
    }
}

#[test]
fn the_barrel_is_stocked_and_small() {
    let b = barrel();
    assert!((9..=16).contains(&b.len()), "the barrel holds {} entries", b.len());
    for d in &b {
        // Cheap enough to buy without thinking about it. A starting purse is
        // 28 Fnorp and the pit pays 6 a fight, so this is the ceiling that
        // makes the first weapon affordable on the first afternoon.
        assert!(d.price <= shop::BARREL_CEILING,
            "{} costs {} Fnorp, which is not a bargain", d.name, d.price);
        assert!(d.price > 0, "{} is free, and free filler is not a decision", d.name);
        // **Four cells, not two.** The frame's "1x1 and 1x2 commons" could not
        // assemble anything: every recipe needs a core and no core in the
        // catalogue is smaller than three cells, because a core is the
        // item-split anchor and is large by construction.
        assert!(d.cells.len() <= 4, "{} is {} cells", d.name, d.cells.len());
    }
}

#[test]
fn every_grid_assembles_out_of_the_barrel_alone() {
    // **The acceptance that matters.** A barrel that fills cells and assembles
    // nothing is a barrel that makes a board look busy and fight worse: two
    // components that touch are one item, so packing more is not packing
    // better. M12.0 found the greaves grid empty for fourteen levels because
    // a Mold with no Material is not an item.
    let names: Vec<&str> = barrel().iter().map(|d| d.name).collect();
    for &kind in SlotKind::ALL.iter() {
        let mut ch = common::bench();
        ch.grow_boards(20);
        for k in SlotKind::ALL {
            ch.loadout.slot_mut(k).clear();
        }
        // Seat every barrel piece that fits this grid, greedily, and see
        // whether the recipe is satisfied.
        for name in &names {
            let Some(id) = ch.owned.iter().copied().find(|&p| {
                ch.registry.def(p).name == *name && !ch.is_equipped(p)
            }) else { continue };
            if !ch.registry.def(id).fits(kind) {
                continue;
            }
            'seat: for y in 0..ch.loadout.slot(kind).rows() {
                for x in 0..gm2d_core::slot::SLOT_W {
                    if ch.equip(id, kind, x, y).is_ok() {
                        gm2d_core::loadout::lock_assembled_in(
                            &mut ch.loadout, &ch.registry, kind);
                        break 'seat;
                    }
                }
            }
        }
        let report = ch.report(kind);
        assert!(
            report.items.iter().any(|i| i.assembled),
            "{kind:?} assembles nothing out of the barrel: {:?}",
            report.items.iter().map(|i| i.status.clone()).collect::<Vec<_>>()
        );
    }
}

/// **Every way of building a weapon, not just the one the list happened to
/// cover.**
///
/// `every_grid_assembles_out_of_the_barrel_alone` above asks whether each of
/// the five grids makes *something*, and the weapon grid always did: the barrel
/// held a handle and an edge. What it never held was a book, a spell or an orb,
/// so **the other two ways of building a weapon assembled nothing from any
/// counter in the game** — a hundred and six components, a third of the weapon
/// grid's design space, on no shelf, in no barrel and in no order book.
///
/// The check that was green through it was asking about grids, and `recipes`
/// has said there are three ways to build that one since before the fork. So
/// this asks the recipe table instead — which means it cannot go stale the next
/// time a grid grows a second way of being built.
#[test]
fn every_recipe_assembles_out_of_the_barrel_alone() {
    let names: Vec<&str> = barrel().iter().map(|d| d.name).collect();
    let mut missing = Vec::new();
    for &kind in SlotKind::ALL.iter() {
        for (r, recipe) in gm2d_core::piece::recipes(kind).iter().enumerate() {
            // Seat exactly what this recipe asks for, in the order it asks —
            // greedily, from the barrel and from nowhere else.
            let mut ch = common::bench();
            ch.grow_boards(20);
            for k in SlotKind::ALL {
                ch.loadout.slot_mut(k).clear();
            }
            let mut short = Vec::new();
            for &(want, least, _) in recipe.iter() {
                for _ in 0..least {
                    let Some(id) = ch.owned.iter().copied().find(|&p| {
                        let d = ch.registry.def(p);
                        d.kind == want && d.fits(kind) && names.contains(&d.name)
                            && !ch.is_equipped(p)
                    }) else {
                        short.push(format!("{want:?}"));
                        continue;
                    };
                    let mut seated = false;
                    'seat: for y in 0..ch.loadout.slot(kind).rows() {
                        for x in 0..gm2d_core::slot::SLOT_W {
                            if ch.equip(id, kind, x, y).is_ok() {
                                gm2d_core::loadout::lock_assembled_in(
                                    &mut ch.loadout, &ch.registry, kind);
                                seated = true;
                                break 'seat;
                            }
                        }
                    }
                    if !seated {
                        short.push(format!("{want:?} (no room)"));
                    }
                }
            }
            let made = ch.report(kind).items.iter().any(|i| i.assembled);
            if !made {
                missing.push(format!("{kind:?} way {r}: nothing, short of {short:?}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "a way of building a grid that no barrel can finish:\n  {}",
        missing.join("\n  ")
    );
}

/// **A rerolled barrel is the same barrel**, and that is the half the authored
/// one cannot prove.
///
/// *The rules are the rules whoever rolled it* is already written down for
/// what the barrel may **hold**; this is the same sentence about what it must
/// **do**. `shop::barrel_wants` is derived from the recipe table, so a roll
/// covers every way of building every grid — and if it ever stops, the authored
/// barrel would still pass the check above while every rerolled one quietly
/// lost a way to build a weapon.
#[test]
fn a_rolled_barrel_covers_every_recipe_too() {
    use gm2d_core::piece::PieceKind;
    let wants = shop::barrel_wants();
    for &kind in SlotKind::ALL.iter() {
        for recipe in gm2d_core::piece::recipes(kind).iter() {
            for &(want, least, _) in recipe.iter() {
                if least == 0 {
                    continue;
                }
                let asked = wants.iter().filter(|(k, s)| *k == want && *s == kind).count();
                assert!(
                    asked >= least,
                    "{kind:?} wants {least} of {want:?} and a rolled barrel takes {asked}"
                );
            }
        }
    }
    // And the pool can actually answer every one of them, which is the other
    // way this goes quiet: a want nothing in the catalogue can fill is a want
    // that rolls nothing and says nothing.
    let pool = shop::barrel_pool();
    for (k, s) in &wants {
        let n = pool.iter().filter(|d| d.kind == *k && d.fits(*s)).count();
        assert!(n > 0, "nothing cheap and small enough is a {k:?} for {s:?}");
    }
    // Five spells, three orbs and six books, at the barrel's price — which is
    // the number this whole change is about and the one that was zero.
    let casting = |k: PieceKind| pool.iter().filter(|d| d.kind == k).count();
    for k in [PieceKind::Book, PieceKind::Spell, PieceKind::Orb, PieceKind::Ink,
              PieceKind::Alignment] {
        assert!(casting(k) >= 2, "the barrel can only ever offer {} of {k:?}", casting(k));
    }
    // The orb's way needs two spells at once, so one is not enough however
    // many rolls you take.
    assert!(casting(PieceKind::Spell) >= 2, "a crystal ball needs two spells and the barrel has one");
}

/// **Nothing the cheap tiers sell is something an errand pays.**
///
/// *A reward you could have bought makes the errand a slow way to shop* is a
/// rule this project wrote down when the errands were built, and it was
/// enforced for the shelf only: `EVENT_ONLY` holds every set piece and every
/// chain reward off every counter, and **eighteen ordinary errand rewards were
/// never on that list.** So the barrel could sell you the Warding Sigil an
/// errand was about to hand over, and the order book could take a commission
/// for eleven more.
///
/// Found by nearly authoring it: the first draft of the barrel's spell line
/// *was* the Warding Sigil. Both pools read `quests.json` now.
#[test]
fn no_cheap_tier_sells_what_an_errand_pays() {
    let quests = gm2d_core::data::quests();
    let paid: Vec<&str> = quests.quests.iter().flat_map(|q| q.reward.iter().map(|r| r.as_str())).collect();
    assert!(paid.len() > 10, "the errands pay {} things, so this check is asleep", paid.len());
    let mut bad = Vec::new();
    for (tier, pool) in [("the barrel", shop::barrel_pool()), ("the order book", shop::ledger_pool())] {
        for d in pool {
            if paid.contains(&d.name) {
                bad.push(format!("{tier} can hold {}, which an errand pays", d.name));
            }
        }
    }
    // And the authored barrel, which is not rolled from that pool.
    for d in barrel() {
        if paid.contains(&d.name) {
            bad.push(format!("the barrel holds {}, which an errand pays", d.name));
        }
    }
    assert!(bad.is_empty(), "an errand made into a slow way to shop:\n  {}", bad.join("\n  "));
}

#[test]
fn the_barrel_is_the_same_in_every_town_and_on_every_visit() {
    // One list, not a list per town: the barrel is furniture rather than a
    // place's character. A regional barrel is a second designed curve to keep
    // tuned — `PLAN-M12.md` §8 row 2 — and this is what makes that true rather
    // than intended.
    let shops = gm2d_core::data::shops();
    let once: Vec<&str> = shop::barrel(&shops).into_iter().map(|o| o.def.name).collect();
    let twice: Vec<&str> = shop::barrel(&shops).into_iter().map(|o| o.def.name).collect();
    assert_eq!(once, twice, "the barrel turned over between two readings");
    assert!(!once.is_empty());
    // And nothing about a town reaches it.
    assert!(shops.towns.len() > 1, "there is more than one town to compare");
}

#[test]
fn nothing_in_the_barrel_is_on_a_shelf_you_can_reach() {
    // **The shelf is the ceiling and the barrel is the floor**, and a
    // component in both is a shelf line nobody would ever take: the barrel
    // charges the same price and never runs out. Refused at load, and this is
    // that refusal proved rather than trusted.
    //
    // **On the map**, and the qualifier is the whole of what changed. The rule
    // is about *undercutting*, and a shelf nobody can walk up to undercuts
    // nothing — `shops.json` carries a shelf for High Wick, which is on no map
    // and is named as staged in `avail.rs`. It is also the arcane shelf: a
    // book, an ink and three of the five spells the barrel could afford, all
    // held out of the cheap tier on behalf of a counter nobody has stood at.
    //
    // The day High Wick is placed its stock leaves the barrel on its own,
    // which is this rule working rather than an exception to it.
    let shops = gm2d_core::data::shops();
    let placed = gm2d_core::data::towns_on_the_map();
    assert!(!placed.is_empty(), "no town is on any map, so this check is asleep");
    for d in barrel() {
        for t in shops.towns.iter().filter(|t| placed.iter().any(|p| *p == t.id)) {
            assert!(
                !t.stock.iter().any(|n| n == d.name),
                "{} is in the barrel and on {}'s shelf",
                d.name,
                t.id
            );
        }
    }
}

#[test]
fn the_barrel_is_a_floor_and_not_a_ceiling() {
    // **Filler is meant to die**, so the barrel must lose on quality and is
    // allowed to win on completeness. That distinction is the whole test, and
    // the first version of it got it wrong: it compared whole-board stats and
    // failed, because a barrel board covers five grids and the pit's shelf
    // covers none of them completely. Winning on completeness is the barrel's
    // *job* — M12.0 measured a greaves grid empty for fourteen levels.
    // Winning on quality would make the designed curve decoration.
    //
    // So: per piece, against the shelf a new character actually stands in
    // front of.
    let shops = gm2d_core::data::shops();
    let pit: Vec<&'static gm2d_core::piece::PieceDef> = shops
        .town("the-end-of-all-gears")
        .expect("the pit sells things")
        .stock
        .iter()
        .map(|n| CATALOG.iter().find(|c| c.name == *n).expect("in the catalogue"))
        .collect();
    let rate = |d: &&'static gm2d_core::piece::PieceDef| gm2d_core::rating::piece_rating(d) as i64;

    let b = barrel();
    let barrel_top = b.iter().map(rate).max().expect("a stocked barrel");
    let pit_top = pit.iter().map(rate).max().expect("a stocked shelf");
    assert!(
        barrel_top <= pit_top,
        "the barrel's best piece rates {barrel_top} and the pit's best rates {pit_top}"
    );

    // Hundredths, because the two means are close on purpose: the barrel is
    // the floor *of the same building*, not a worse building.
    let barrel_mean = b.iter().map(rate).sum::<i64>() * 100 / b.len() as i64;
    let pit_mean = pit.iter().map(rate).sum::<i64>() * 100 / pit.len() as i64;
    assert!(
        barrel_mean <= pit_mean,
        "the barrel averages {barrel_mean} per hundred and the pit shelf {pit_mean}, so the \
         barrel is the better shop"
    );
}

#[test]
fn there_is_no_way_to_turn_a_component_back_into_fnorp() {
    // **The frame's acceptance was "buying and reselling the barrel must lose
    // money", and there is nothing to resell.** Recon: the game has no sell
    // and no discard — no `sell`, no `sell_price`, no `discard` anywhere in
    // core, the shim or the page. So the gold loop the frame was guarding
    // against cannot exist, and the honest check is the invariant rather than
    // the spread: nothing converts gear into money.
    //
    // This is what makes an infinite barrel safe. The moment something buys a
    // component back, an entry that never runs out at a fixed price is a
    // money printer, and this check is what will fail on that day.
    let src = concat!(
        include_str!("../src/shop.rs"),
        include_str!("../src/character.rs"),
        include_str!("../src/game.rs"),
        include_str!("../src/quest.rs"),
    );
    for banned in ["fn sell", "sell_price", "fn buy_back", "fn discard"] {
        assert!(
            !src.contains(banned),
            "{banned:?} exists now, so a component can become Fnorp and the barrel is a \
             faucet: price the spread before shipping it"
        );
    }
    // And filler dies for free, which is the other half: unseating a piece
    // costs nothing and puts it back in the bag.
    let mut ch = common::bench();
    ch.grow_boards(20);
    let id = common::piece(&ch, "Oak Handle");
    ch.equip(id, SlotKind::Weapon, 0, 0).expect("it seats");
    let purse = ch.gold;
    ch.unequip(id).expect("and it comes back off");
    assert_eq!(ch.gold, purse, "taking a component off cost something");
    assert!(!ch.is_equipped(id), "and it is off the board");
    assert!(ch.inventory().contains(&id), "and back in the bag");
}

#[test]
fn the_barrel_holds_no_quest_item_and_nothing_off_a_creature() {
    // A quest item is carried, never worn, so a barrel full of them would sell
    // cells nobody can use. And every set piece is `EVENT_ONLY` and off a
    // creature — a set you could buy is not a set you earned.
    for d in barrel() {
        assert_ne!(d.kind, PieceKind::Quest, "{} is carried, not worn", d.name);
        assert!(
            !gm2d_core::piece::EVENT_ONLY.contains(&d.name),
            "{} is off a creature and the barrel sells it",
            d.name
        );
        assert!(
            CATALOG.iter().any(|c| c.name == d.name),
            "{} is not in the catalogue",
            d.name
        );
    }
}

// ---------------------------------------------------------------- the tiers

#[test]
fn the_barrel_is_cheaper_than_the_shelf_for_the_same_thing() {
    // **The tier, stated over the whole catalogue rather than over the stock.**
    // Comparing the barrel's actual entries with the shelf's actual entries
    // compares two different sets of components and would pass on a shelf that
    // happened to stock dear things. This asks the question that is really
    // being asked: for one and the same component, what does each tier want?
    for d in CATALOG.iter().filter(|d| d.price > 0) {
        let barrel = d.price;
        let shelf = shop::shelf_price(d);
        let order = shop::commission_price(d);
        assert!(shelf > barrel, "{}: the shelf wants {shelf} and the barrel {barrel}", d.name);
        assert!(order > shelf, "{}: an order wants {order} and the shelf {shelf}", d.name);
    }
}

#[test]
fn the_opening_is_one_good_piece_or_a_frame_of_junk() {
    // **What the mark-up is set against.** A starting purse has to buy a
    // decision rather than a shopping list: one considered piece off the
    // shelf and a weapon out of the barrel, or a frame full of junk — and
    // *not* two shelf pieces, which is what made the shelf the only place
    // worth looking before there was a floor under it.
    let shops = gm2d_core::data::shops();
    let purse = shop::STARTING_GOLD;
    let mut pit: Vec<i32> = shops
        .town("the-end-of-all-gears")
        .expect("the pit sells things")
        .stock
        .iter()
        .map(|n| shop::shelf_price(CATALOG.iter().find(|c| c.name == *n).expect("catalogued")))
        .collect();
    pit.sort_unstable();
    let cheapest = pit[0];
    let two = pit[0] + pit[1];
    assert!(cheapest <= purse, "a new character cannot afford one shelf line ({cheapest})");
    assert!(two > purse, "a new character can afford two shelf lines ({two} of {purse})");

    // And the barrel still assembles a weapon out of what is left over.
    let b = barrel();
    let handle = b.iter().filter(|d| d.kind == PieceKind::Handle).map(|d| d.price).min();
    let edge = b.iter().filter(|d| d.kind == PieceKind::Damaging).map(|d| d.price).min();
    let (handle, edge) = (handle.expect("a handle"), edge.expect("an edge"));
    assert!(
        cheapest + handle + edge <= purse,
        "one shelf piece ({cheapest}) and a barrel weapon ({handle}+{edge}) is {} of {purse}",
        cheapest + handle + edge
    );
}

#[test]
fn a_mark_up_never_rounds_a_price_to_nothing() {
    // The dearest tier must not become the cheapest on a rounding error.
    for pct in [shop::SHELF_PCT, shop::COMMISSION_PCT] {
        assert!(shop::at_pct(1, pct) >= 1);
        assert!(shop::at_pct(0, pct) >= 1, "even a free component costs something to order");
    }
    assert_eq!(shop::at_pct(15, shop::SHELF_PCT), 75, "the pit's cheapest line");
    assert_eq!(shop::at_pct(15, shop::COMMISSION_PCT), 150);
}
