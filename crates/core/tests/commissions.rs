//! Commissions. M12.2.
//!
//! The deterministic answer to *I want the thing, not the lottery*: order a
//! piece, pay now, and it arrives on the world's clock rather than on a roll.

mod common;

use gm2d_core::combat::Difficulty;
use gm2d_core::game::Game;
use gm2d_core::piece::CATALOG;
use gm2d_core::shop;

const D: Difficulty = Difficulty::Easy;
const PIT: &str = "the-end-of-all-gears";

fn a_customer() -> Game {
    let mut g = Game::new(5, "td");
    g.character.gold = 5_000;
    g
}

/// Fight whatever is in front of you, so the clock ticks the way a fight ticks.
fn have_a_fight(g: &mut Game) {
    g.encounter = Some(gm2d_core::fight::Encounter {
        enemy: "Cave Rat".into(),
        at: g.world.at,
    });
    let log = gm2d_core::fight::run(g, D).expect("a fight to have");
    gm2d_core::fight::settle(g, &log, D).expect("it settles");
}

#[test]
fn every_commission_reaches_something() {
    // **The `every_offered_class_reaches_something` shape, and the reason that
    // lint works is that it CALLS rather than declares.** Its first version
    // matched a variant and named where the power was honoured, which a
    // stubbed payout passed cleanly. So this does not read the ledger and say
    // "yes, that piece exists": for every order in the game it places it,
    // ticks the clock the number of fights the ledger asks for, collects, and
    // asserts the component is in the bag.
    let shops = gm2d_core::data::shops();
    let mut any = 0;
    for town in &shops.towns {
        for o in shop::commissions(&shops, &town.id) {
            let mut g = a_customer();
            let before = g.character.gold;
            let placed = g.order(&town.id, o.index).unwrap_or_else(|e| {
                panic!("{} would not take an order for {}: {e}", town.id, o.def.name)
            });
            assert_eq!(placed.fights_left, o.fights);
            assert_eq!(g.character.gold, before - o.price, "the order was not paid for");
            assert!(!g.character.holds(o.def.name), "it arrived before it was made");

            for _ in 0..o.fights {
                have_a_fight(&mut g);
            }
            let got = g
                .collect(&town.id)
                .unwrap_or_else(|e| panic!("{} would not hand over {}: {e}", town.id, o.def.name));
            assert_eq!(got, o.def.name);
            assert!(
                g.character.holds(o.def.name),
                "{} collected {} and it is not in the bag",
                town.id,
                o.def.name
            );
            any += 1;
        }
    }
    assert!(any >= 6, "only {any} orders in the whole game");
}

#[test]
fn it_is_not_ready_one_fight_early() {
    // The clock is a cost, so it has to be countable and it has to bite.
    let shops = gm2d_core::data::shops();
    let o = shop::commissions(&shops, PIT).into_iter().next().expect("the pit takes orders");
    let mut g = a_customer();
    g.order(PIT, o.index).expect("the order is placed");
    for _ in 0..o.fights.saturating_sub(1) {
        have_a_fight(&mut g);
    }
    let why = g.collect(PIT).expect_err("it cannot be ready yet");
    assert!(why.contains('1') || why.to_lowercase().contains("not yet"), "{why:?} says nothing");
    assert!(!g.character.holds(o.def.name));
    have_a_fight(&mut g);
    assert_eq!(g.collect(PIT).expect("now it is ready"), o.def.name);
}

#[test]
fn a_rout_is_not_a_fight_and_does_not_tick() {
    // **The loophole this milestone had to be careful about.** `fight::rout`
    // settles an encounter with no fight in it and deliberately costs no
    // tiredness — nothing was fought. The clock ticks where the tiredness
    // does, so a routed creature must not bring an order closer: otherwise
    // the Rat King's Mandate and the survey golem pace an order out on
    // creatures that decline to fight, which is walking in circles by another
    // name.
    let shops = gm2d_core::data::shops();
    let o = shop::commissions(&shops, PIT).into_iter().next().expect("the pit takes orders");
    let mut g = a_customer();
    // **The whole Mandate, and all three down before anything locks.**
    // `common::seat` locks each item as it assembles — which is right, and is
    // what a player does — but it means the Material and the Mold lock as a
    // finished glove the moment they touch, and the Signet then arrives beside
    // a *locked* item and is its own group. Two items, and neither is the set:
    // `loadout::set_of` wants completeness, so two thirds of the Mandate
    // grants nothing. Seat the three, then lock once.
    g.character = common::bench();
    for (name, x, y) in
        [("Ratskin Material", 0u8, 0u8), ("Ratskin Mold", 2, 0), ("Rat Signet", 4, 0)]
    {
        let id = common::spare(&g.character, name);
        g.character.registry.set_rotation(id, 0);
        g.character
            .equip(id, gm2d_core::piece::SlotKind::Gloves, x, y)
            .unwrap_or_else(|e| panic!("{name} would not seat: {e}"));
    }
    gm2d_core::loadout::lock_assembled_in(
        &mut g.character.loadout,
        &g.character.registry,
        gm2d_core::piece::SlotKind::Gloves,
    );
    g.character.gold = 5_000;
    assert!(
        g.character.rules().iter().any(|r| matches!(r, gm2d_core::rule::Rule::Rout { .. })),
        "the Mandate is not assembled, so nothing would be routed and this proves nothing"
    );
    g.order(PIT, o.index).expect("the order is placed");
    let was = g.world.commissions[0].fights_left;

    g.encounter = Some(gm2d_core::fight::Encounter { enemy: "Cave Rat".into(), at: g.world.at });
    let routed = gm2d_core::fight::rout(&mut g).expect("the Mandate routs a Cave Rat");
    assert!(!routed.receipt.is_empty());
    assert_eq!(
        g.world.commissions[0].fights_left, was,
        "a rout moved the order along, and nothing was fought"
    );
}

#[test]
fn one_open_order_per_town_and_the_refusals_name_what_is_in_the_way() {
    let shops = gm2d_core::data::shops();
    let book = shop::commissions(&shops, PIT);
    let (a, b) = (&book[0], &book[1]);
    let mut g = a_customer();
    g.order(PIT, a.index).expect("the first order");
    let why = g.order(PIT, b.index).expect_err("a second is refused");
    assert!(why.to_lowercase().contains("one at a time"), "{why:?}");

    // A town that does not make it says so.
    assert!(g.order(PIT, 999).is_err());
    // And another town's counter is its own promise.
    let other = shops
        .towns
        .iter()
        .find(|t| t.id != PIT && !t.commissions.is_empty())
        .expect("a second town takes orders");
    let theirs = shop::commissions(&shops, &other.id);
    g.order(&other.id, theirs[0].index).expect("a second town takes its own order");
    assert_eq!(g.world.commissions.len(), 2, "one each, not one in total");

    // Not enough money says how much.
    let mut poor = Game::new(6, "td");
    poor.character.gold = 0;
    let why = poor.order(PIT, a.index).expect_err("no money, no order");
    assert!(why.contains("Fnorp"), "{why:?} does not name the price");
    assert!(poor.world.commissions.is_empty(), "a refused order was still written down");
}

#[test]
fn an_order_is_collected_where_it_was_placed() {
    let shops = gm2d_core::data::shops();
    let o = shop::commissions(&shops, PIT).into_iter().next().expect("the pit takes orders");
    let mut g = a_customer();
    g.order(PIT, o.index).expect("placed");
    for _ in 0..o.fights {
        have_a_fight(&mut g);
    }
    let other = shops
        .towns
        .iter()
        .find(|t| t.id != PIT)
        .expect("a second town");
    assert!(g.collect(&other.id).is_err(), "another town handed over the pit's order");
    assert_eq!(g.collect(PIT).expect("the pit hands it over"), o.def.name);
    assert!(g.collect(PIT).is_err(), "the pit handed it over twice");
}

#[test]
fn an_order_is_the_dearest_way_to_get_a_thing() {
    // The tier, from the order book's end: certainty costs more than luck.
    let shops = gm2d_core::data::shops();
    for town in &shops.towns {
        for o in shop::commissions(&shops, &town.id) {
            assert_eq!(o.price, shop::commission_price(o.def));
            assert!(o.price > shop::shelf_price(o.def), "{} is not dearer than the shelf", o.def.name);
            assert!(o.price > o.def.price);
        }
    }
}

#[test]
fn nothing_on_order_is_on_a_shelf_or_in_the_barrel() {
    // An order is a third source, not a slower way to shop. Refused at load;
    // this is that refusal proved rather than trusted.
    let shops = gm2d_core::data::shops();
    let barrel: Vec<&str> = shop::barrel(&shops).into_iter().map(|o| o.def.name).collect();
    for town in &shops.towns {
        for c in &town.commissions {
            assert!(!barrel.contains(&c.piece.as_str()), "{} is in the barrel", c.piece);
            for t in &shops.towns {
                assert!(!t.stock.iter().any(|n| *n == c.piece), "{} is on {}'s shelf", c.piece, t.id);
            }
            assert!(CATALOG.iter().any(|d| d.name == c.piece), "{} is not catalogued", c.piece);
            assert!(c.fights > 0);
        }
    }
}

#[test]
fn an_order_survives_the_save() {
    // `WorldState` is serialised whole rather than destructured, so a new
    // field on it is **not** a compile error the way one on `Character` or
    // `Game` is — `save_is_whole.rs` says so in its own opening. That makes
    // this the only thing standing between a new field and a save that
    // silently drops it.
    let shops = gm2d_core::data::shops();
    let o = shop::commissions(&shops, PIT).into_iter().next().expect("the pit takes orders");
    let mut g = a_customer();
    g.order(PIT, o.index).expect("placed");
    have_a_fight(&mut g);
    let mid = g.world.commissions.clone();
    assert_eq!(mid[0].fights_left, o.fights - 1);

    let text = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&text).expect("it loads");
    assert_eq!(back.world.commissions, mid, "the order did not survive the save");
}
