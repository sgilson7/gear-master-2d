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
    // **Shelves you can walk up to**, which is what the rule is about: the
    // barrel must not undercut a counter, and a counter on no map undercuts
    // nothing. See `nothing_in_the_barrel_is_on_a_shelf_you_can_reach`.
    let placed = gm2d_core::data::towns_on_the_map();
    let on_a_shelf: Vec<&str> = shops
        .towns
        .iter()
        .filter(|t| placed.iter().any(|p| *p == t.id))
        .flat_map(|t| t.stock.iter().map(|s| s.as_str()))
        .collect();
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
        // **Every recipe still finishes out of it**, which is a wider claim
        // than the five kinds this used to name: a weapon has three ways of
        // being built and a list of kinds written here would go stale the same
        // way the one in `roll_barrel` did. `shop::barrel_wants` is the
        // recipe table's answer, so this asks it.
        let kinds: Vec<gm2d_core::piece::PieceKind> = b.iter().map(|o| o.def.kind).collect();
        for (want, _) in shop::barrel_wants() {
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

/// **You get the line you were shown, and a reroll moves what you are buying.**
///
/// Reported from play: *"when you reroll the barrel, and you purchase a piece
/// of gear from the rerolled set of gear, you get a piece of gear from the
/// first version of the barrel gear items. So essentially the item you get is
/// not the item you buy and it seemingly never changes."*
///
/// Exactly that. The screen drew `Game::barrel_now` — rolled if it has been
/// rolled — and the shim's `buy_barrel` looked the index up in `shop::barrel`,
/// which is the authored list out of `shops.json`. **Two answers to what is in
/// the barrel, one drawn and one charged for.**
///
/// `Game::order` had the same question right from the day it was written and
/// says so in a comment; the ledger's buy went into core with the reroll and
/// the barrel's stayed in the shim. A rule decided in the shim is a rule the
/// fast suite cannot reach, and this is what that costs.
///
/// Negative-tested by putting `shop::barrel(&data::shops())` back in
/// `Game::buy_barrel`: the second assertion printed the authored name against
/// the rolled one.
#[test]
fn buying_out_of_the_barrel_hands_over_the_line_you_are_looking_at() {
    let mut g = Game::default();
    g.character.gold = 100_000;

    // Before any reroll, the authored barrel and what you are charged agree.
    let authored: Vec<String> =
        g.barrel_now().iter().map(|o| o.def.name.to_string()).collect();
    assert!(authored.len() >= 8, "the barrel is too short to prove anything");
    let (_, got) = g.buy_barrel(3).expect("the fourth line");
    assert_eq!(got, authored[3], "the authored barrel sold the wrong line");

    // Turn it over until the fourth line is a different component. It is
    // rolled, so one turn is not guaranteed to move any given line.
    let mut turns = 0;
    while g.barrel_now()[3].def.name == authored[3] && turns < 20 {
        g.reroll_barrel().expect("the purse is deep");
        turns += 1;
    }
    let rolled: Vec<String> = g.barrel_now().iter().map(|o| o.def.name.to_string()).collect();
    assert_ne!(rolled[3], authored[3], "twenty turns and the fourth line never moved");

    // **The line you are looking at.** This is the report, in one assertion.
    let (_, got) = g.buy_barrel(3).expect("the fourth line of the rolled barrel");
    assert_eq!(
        got, rolled[3],
        "the barrel showed {} and handed over {got}, which is the authored line at that index",
        rolled[3]
    );
    assert!(g.character.holds(&rolled[3]), "it is not in the bag");

    // And every line of it, because an off-by-one would pass the one above.
    for (i, want) in rolled.iter().enumerate() {
        let (_, got) = g.buy_barrel(i).unwrap_or_else(|why| panic!("line {i}: {why}"));
        assert_eq!(&got, want, "line {i} showed {want} and handed over {got}");
    }

    // A refusal spends nothing, and an index the barrel has not got is one.
    let purse = g.character.gold;
    assert!(g.buy_barrel(999).is_err());
    assert_eq!(g.character.gold, purse, "a refused line took the money anyway");
}
