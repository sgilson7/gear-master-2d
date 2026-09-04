//! The licence, and turning the barrel and the ledger over.

mod common;

use gm2d_core::combat::Difficulty;
use gm2d_core::game::Game;
use gm2d_core::shop::{self, REROLL_BARREL, REROLL_LEDGER};

const D: Difficulty = Difficulty::Easy;
const PIT: &str = "the-end-of-all-gears";

fn rich() -> Game {
    let mut g = Game::new(11, "td");
    g.character.gold = 100_000;
    g
}

// ------------------------------------------------------------------ licence

/// **A class is still the free way in, and the van sells the same permission.**
///
/// The fork is permanent. Without this, a player who took Gorillathon at level
/// five can be paid an ench by an errand for the rest of the game and never
/// once use one.
#[test]
fn the_van_sells_a_licence_to_anybody_who_has_not_got_one() {
    let mut g = rich();
    g.character.class = Some("Berserker".into());
    assert!(!g.character.licensed(), "an unlicensed class starts unlicensed");

    let before = g.character.gold;
    let paid = g.buy_licence().expect("he sells one");
    assert_eq!(paid, gm2d_core::ench::LICENCE_PRICE);
    assert_eq!(paid, 5_000, "the licence moved with everything else");
    assert_eq!(g.character.gold, before - paid, "it was not paid for");
    assert!(g.character.licensed(), "paid for a licence and is not licensed");

    // Nobody sells a second, and the refusal says why.
    let why = g.buy_licence().expect_err("a second is refused");
    assert!(why.to_lowercase().contains("second"), "{why:?}");
    assert_eq!(g.character.gold, before - paid, "a refused sale still charged");
}

#[test]
fn a_licensed_class_is_not_sold_one_and_does_not_need_it() {
    let mut g = rich();
    g.character.class = Some(gm2d_core::ench::LICENSED_CLASS.into());
    assert!(g.character.licensed());
    let why = g.buy_licence().expect_err("he can tell");
    assert!(!why.is_empty());
}

#[test]
fn no_money_no_licence_and_the_refusal_names_the_price() {
    let mut g = Game::new(4, "td");
    g.character.class = Some("Berserker".into());
    g.character.gold = 10;
    let why = g.buy_licence().expect_err("ten Fnorp is not a thousand");
    assert!(why.contains("Fnorp"), "{why:?}");
    assert!(!g.character.licensed());
}

#[test]
fn a_bought_licence_survives_the_save() {
    let mut g = rich();
    g.character.class = Some("Berserker".into());
    g.buy_licence().expect("bought");
    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it loads");
    assert!(back.character.licensed(), "the licence did not survive the save");
}

#[test]
fn an_ench_costs_two_thousand() {
    for e in &gm2d_core::data::enchs().enchs {
        if let Some(p) = e.price {
            assert_eq!(p, 2_000, "{} is on a table at {p}", e.id);
        }
    }
}

// ------------------------------------------------------------------ rerolls

/// **1, 4, 9, 16** — `n * n` for the nth.
#[test]
fn a_reroll_costs_n_squared() {
    assert_eq!(shop::reroll_price(0), 1, "the first");
    assert_eq!(shop::reroll_price(1), 4);
    assert_eq!(shop::reroll_price(2), 9);
    assert_eq!(shop::reroll_price(3), 16);
    assert_eq!(shop::reroll_price(7), 64);
    let mut g = rich();
    for want in [1, 4, 9, 16] {
        assert_eq!(g.reroll_price(REROLL_BARREL), want);
        assert_eq!(g.reroll_barrel().expect("he turns it over"), want);
    }
}

/// **The two counters are separate**, which is the whole of the ask.
#[test]
fn turning_the_barrel_over_does_not_price_the_ledger() {
    // **Both are exercised, or this proves nothing.** The first version of
    // this rerolled only the barrel and then asserted the ledger's counter was
    // zero — which it is whether or not they share a tally, because nothing
    // had touched the ledger. Making the ledger bump the barrel's counter
    // passed it cleanly.
    let mut g = rich();
    for _ in 0..5 {
        g.reroll_barrel().expect("rolled");
    }
    assert_eq!(g.rerolls_done(REROLL_BARREL), 5);
    assert_eq!(g.rerolls_done(REROLL_LEDGER), 0, "the ledger paid for the barrel's impatience");
    assert_eq!(g.reroll_price(REROLL_LEDGER), 1, "the ledger's first is still the first");

    // Now turn the ledger over twice and check it landed on the ledger.
    for _ in 0..2 {
        g.reroll_ledger(PIT).expect("turned over");
    }
    assert_eq!(g.rerolls_done(REROLL_LEDGER), 2, "the ledger's own rerolls were not counted");
    assert_eq!(
        g.rerolls_done(REROLL_BARREL), 5,
        "turning the ledger over moved the barrel's counter"
    );
    assert_eq!(g.reroll_price(REROLL_BARREL), 36);
    assert_eq!(g.reroll_price(REROLL_LEDGER), 9);
}

/// Every ten levels, everywhere, the counters go back to one.
#[test]
fn ten_levels_resets_every_counter() {
    let mut g = rich();
    for _ in 0..4 {
        g.reroll_barrel().expect("rolled");
    }
    assert_eq!(g.reroll_price(REROLL_BARREL), 25);
    assert_eq!(shop::reroll_band(1), 0);
    assert_eq!(shop::reroll_band(10), 1, "ten is a new band");

    g.character.gain_xp(gm2d_core::progression::xp_to_reach(10));
    assert!(g.character.level() >= 10, "the character actually crossed a band");
    assert_eq!(g.rerolls_done(REROLL_BARREL), 0, "the band moved and nothing reset");
    assert_eq!(g.reroll_price(REROLL_BARREL), 1, "it is the first one again");
}

/// A rolled barrel obeys every rule the authored one obeys.
#[test]
fn a_rolled_barrel_is_still_a_barrel() {
    let mut g = rich();
    let shops = gm2d_core::data::shops();
    let on_a_shelf: Vec<&str> =
        shops.towns.iter().flat_map(|t| t.stock.iter().map(|s| s.as_str())).collect();
    for _ in 0..25 {
        g.reroll_barrel().expect("rolled");
        let b = g.barrel_now();
        assert!(b.len() >= 9, "a rolled barrel of {}", b.len());
        for o in &b {
            assert!(o.price <= shop::BARREL_CEILING, "{} is in the barrel at {}", o.def.name, o.price);
            assert!(o.price > 0);
            assert!(o.def.cells.len() <= 4, "{} is {} cells", o.def.name, o.def.cells.len());
            assert_ne!(o.def.kind, gm2d_core::piece::PieceKind::Quest);
            assert!(
                !gm2d_core::piece::EVENT_ONLY.contains(&o.def.name),
                "{} is off a creature and a reroll shook it out",
                o.def.name
            );
            assert!(!on_a_shelf.contains(&o.def.name), "{} is on a shelf", o.def.name);
        }
        // Every grid still assembles out of it.
        let kinds: Vec<gm2d_core::piece::PieceKind> = b.iter().map(|o| o.def.kind).collect();
        for want in [
            gm2d_core::piece::PieceKind::Handle,
            gm2d_core::piece::PieceKind::Damaging,
            gm2d_core::piece::PieceKind::Frame,
            gm2d_core::piece::PieceKind::Base,
            gm2d_core::piece::PieceKind::Layer,
        ] {
            assert!(kinds.contains(&want), "a rolled barrel has no {want:?}");
        }
    }
}

/// **The one you are waiting for is not rerolled.**
///
/// You paid for it and its clock is running. A reroll that took it away would
/// be a way to lose something you had bought.
#[test]
fn a_reroll_never_takes_away_the_order_you_are_waiting_for() {
    let mut g = rich();
    let first = g.ledger_at(PIT).into_iter().next().expect("the pit takes orders");
    let want = first.def.name.to_string();
    g.order(PIT, first.index).expect("ordered");
    assert_eq!(g.order_at(PIT).map(|c| c.piece.clone()), Some(want.clone()));

    let mut moved = 0;
    for _ in 0..12 {
        g.reroll_ledger(PIT).expect("turned over");
        let book: Vec<String> =
            g.ledger_at(PIT).into_iter().map(|o| o.def.name.to_string()).collect();
        assert!(
            book.contains(&want),
            "the order being made fell out of the book: {book:?}"
        );
        assert_eq!(
            g.order_at(PIT).map(|c| c.piece.clone()),
            Some(want.clone()),
            "the order itself was rerolled"
        );
        if book.iter().any(|n| *n != want) {
            moved += 1;
        }
    }
    assert!(moved > 0, "nothing else on the counter ever changed, so nothing was rerolled");

    // And it still arrives.
    let left = g.order_at(PIT).expect("still on order").fights_left;
    for _ in 0..left {
        g.encounter =
            Some(gm2d_core::fight::Encounter { enemy: "Cave Rat".into(), at: g.world.at });
        let log = gm2d_core::fight::run(&g, D).expect("a fight");
        gm2d_core::fight::settle(&mut g, &log, D).expect("settles");
    }
    assert_eq!(g.collect(PIT).expect("it arrived"), want);
    assert!(g.character.holds(&want));
}

/// A reroll is paid for, and a refused one costs nothing.
#[test]
fn a_reroll_is_paid_for_and_a_refusal_spends_nothing() {
    let mut g = Game::new(3, "td");
    g.character.gold = 0;
    let before = g.world.rolled_barrel.clone();
    let why = g.reroll_barrel().expect_err("no money, no reroll");
    assert!(why.contains("Fnorp"), "{why:?}");
    assert_eq!(g.world.rolled_barrel, before, "a refused reroll rolled anyway");
    assert_eq!(g.rerolls_done(REROLL_BARREL), 0, "and counted anyway");

    g.character.gold = 50;
    let paid = g.reroll_barrel().expect("now he will");
    assert_eq!(g.character.gold, 50 - paid);
}

/// The rolled stock survives the save, or a reload undoes what was paid for.
#[test]
fn a_rolled_barrel_and_ledger_survive_the_save() {
    let mut g = rich();
    g.reroll_barrel().expect("rolled");
    g.reroll_ledger(PIT).expect("rolled");
    let barrel: Vec<String> = g.barrel_now().into_iter().map(|o| o.def.name.into()).collect();
    let ledger: Vec<String> = g.ledger_at(PIT).into_iter().map(|o| o.def.name.into()).collect();

    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it loads");
    let b2: Vec<String> = back.barrel_now().into_iter().map(|o| o.def.name.into()).collect();
    let l2: Vec<String> = back.ledger_at(PIT).into_iter().map(|o| o.def.name.into()).collect();
    assert_eq!(barrel, b2, "the rolled barrel did not survive the save");
    assert_eq!(ledger, l2, "the rolled ledger did not survive the save");
    assert_eq!(back.rerolls_done(REROLL_BARREL), g.rerolls_done(REROLL_BARREL));
}

/// A save from before any of this opens on the barrel it always had.
#[test]
fn an_older_save_opens_on_the_authored_barrel() {
    let g = Game::new(7, "td");
    assert!(g.world.rolled_barrel.is_empty(), "a fresh game has rolled nothing");
    let authored: Vec<&str> =
        shop::barrel(&gm2d_core::data::shops()).into_iter().map(|o| o.def.name).collect();
    let now: Vec<&str> = g.barrel_now().into_iter().map(|o| o.def.name).collect();
    assert_eq!(authored, now, "an unrolled barrel is not the authored one");
}
