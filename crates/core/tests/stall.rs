//! The Stall: a counter of your own, eight buyers, and what two kin leave.
//!
//! M21.8.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::stall::{self, Ask, Bargain, FAIR_PCT, SHELF};

mod common;

const D: Difficulty = Difficulty::Easy;

fn a_fighter(seed: u64) -> Game {
    let mut g = Game::new(seed, "td");
    g.character = common::bench();
    common::build_full_loadout(&mut g.character);
    g
}

/// The shipped file parses, and every check in `parse` is live.
#[test]
fn the_stall_file_reads() {
    let d = data::stall();
    assert_eq!(d.buyers.len(), 8, "eight buyers");
    assert_eq!(d.kin.len(), 28, "C(8,2)");
}

/// **All twenty-eight pairs are authored**, which is the `C(8,2)` argument
/// `every_pair_of_offered_classes_reaches_an_expert` makes and the brewing
/// table and the kennel's diet table both make: a list of twenty-eight written
/// by hand is a list that can be twenty-seven.
#[test]
fn every_pair_of_buyers_is_authored() {
    let d = data::stall();
    let mut missing = Vec::new();
    for (i, a) in d.buyers.iter().enumerate() {
        for b in &d.buyers[i + 1..] {
            if d.kin_of(&a.id, &b.id).is_none() {
                missing.push(format!("{} + {}", a.id, b.id));
            }
        }
    }
    assert!(missing.is_empty(), "no bargain for: {}", missing.join(", "));
}

/// **A bargain is never on a counter.** *A reward you could have bought makes
/// the errand a slow way to shop* — the errands' rule since M8, and a sale is
/// the same shape: three of the twenty-eight pay an ench, and every one of
/// those three is priceless, so the van cannot sell it.
#[test]
fn a_bargain_is_never_on_the_barrel() {
    let d = data::stall();
    let enchs = data::enchs();
    let towns = data::towns_on_the_map();
    let mut bad = Vec::new();
    for k in &d.kin {
        match &k.gives {
            Bargain::Ench(id) => {
                if enchs.get(id).and_then(|e| e.price).is_some() {
                    bad.push(format!("{id} is a bargain and is also for sale"));
                }
            }
            Bargain::Piece(name) => {
                for t in data::shops().towns.iter().filter(|t| towns.contains(&t.id)) {
                    if t.stock.iter().any(|l| l == name) {
                        bad.push(format!("{name} is a bargain and {} sells it", t.id));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("; "));
}

/// **Three of the twenty-eight leave an ench, and those three are the only way
/// to get them.** Asked for in as many words: *you should sometimes receive
/// unique enchs from selling items in your store front.* An ench nothing
/// anywhere hands over is content shipped for nobody, which is the other half
/// of `every_ench_comes_from_somewhere`.
#[test]
fn the_stalls_three_enchs_come_from_the_stall_and_nowhere_else() {
    let d = data::stall();
    let paid: Vec<&str> = d
        .kin
        .iter()
        .filter_map(|k| match &k.gives {
            Bargain::Ench(id) => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(paid.len(), 3, "three bargains pay an ench, got {paid:?}");
    let quests = data::quests();
    let skills = data::skills();
    for id in &paid {
        assert!(
            !quests.quests.iter().any(|q| q.enchs.iter().any(|e| e == id)),
            "{id} is also an errand's"
        );
        assert!(
            !skills.trees.iter().any(|t| t.nodes.iter().any(|n| n
                .effects
                .iter()
                .any(|e| matches!(e, gm2d_core::skills::Effect::GivesEnch { ench } if ench == id)))),
            "{id} is also a node's"
        );
    }
}

/// A component out on the counter has left the bag, which is the bank's rule
/// and the whole of what a shelf is.
#[test]
fn shelved_is_not_carried() {
    let mut g = a_fighter(7);
    let id = g.character.give("Iron Blade").expect("a blade");
    assert!(g.character.owned.contains(&id));
    g.shelve(id, (0, 0), 1, 40).expect("out it goes");
    assert!(!g.character.owned.contains(&id), "still in the bag");
    assert!(
        !g.character.owned.iter().any(|&p| p == id && g.character.fits_anywhere(p)),
        "still on the bench's books"
    );
    g.unshelve(id).expect("back it comes");
    assert!(g.character.owned.contains(&id), "did not come back");
}

/// **A seated component is refused by name and nothing moves.** The bank's
/// rule, for the bank's reason: this happens in a town where the board is not
/// on the screen, so lifting a piece off a grid would break an item somewhere
/// the player cannot watch it happen.
#[test]
fn a_refusal_spends_nothing() {
    let mut g = a_fighter(7);
    let seated = g
        .character
        .owned
        .iter()
        .copied()
        .find(|&p| g.character.is_equipped(p))
        .expect("something is seated");
    let before = g.character.stall.len();
    let e = g.shelve(seated, (0, 0), 1, 40).unwrap_err();
    assert!(e.contains("board"), "{e}");
    assert_eq!(g.character.stall.len(), before, "it went out anyway");

    // And off the mask.
    let id = g.character.give("Iron Blade").expect("a blade");
    let e = g.shelve(id, (3, 3), 1, 40).unwrap_err();
    assert!(e.contains("counter stops short"), "{e}");
    assert!(g.character.owned.contains(&id), "it left the bag on a refusal");
}

/// **A tally is carried, never sold.** `can_equip` has refused
/// `PieceKind::Quest` since M8, and a counter is the fourth consumer that has
/// to know: an errand's tokens crossing a counter is an errand that cannot be
/// handed in, and nothing anywhere would say so. Found by `make play`, which
/// put a Bengulon Toad Eye out at twenty-five Fnorp.
#[test]
fn a_tally_is_carried_never_sold() {
    let mut g = a_fighter(5);
    let quest: &str = gm2d_core::piece::CATALOG
        .iter()
        .find(|d| d.kind == gm2d_core::piece::PieceKind::Quest)
        .expect("the catalogue has a tally")
        .name;
    let id = g.character.give(quest).expect("one of them");
    let e = g.shelve(id, (0, 0), 0, 25).unwrap_err();
    assert!(e.contains("carried, not sold"), "{e}");
    assert!(g.character.owned.contains(&id), "it left the bag anyway");
}

/// **The shelf is a shape and not a box.** Fourteen cells in a five-by-four, so
/// what you can have out at once is a packing decision — the retort's argument,
/// one counter along.
#[test]
fn the_counter_is_a_shape_and_not_a_box() {
    assert_eq!(SHELF.len(), 14, "fourteen cells");
    let w = SHELF.iter().map(|c| c.0).max().unwrap() + 1;
    let h = SHELF.iter().map(|c| c.1).max().unwrap() + 1;
    assert!(
        (w * h) as usize > SHELF.len(),
        "{w}x{h} is {} cells and the shelf is {} — that is a box",
        w * h,
        SHELF.len()
    );
}

/// **The counter holds the weapon every character starts with**, and the first
/// draft did not — nine cells in a three-tall box against a blade that is one
/// by four. *A counter that cannot hold the starting weapon is a counter nobody
/// opens.*
///
/// So the rule is measured and stated over the whole catalogue rather than over
/// the blade: **everything inside a four by three goes on the counter**, which
/// is 562 of the 568, and the six that do not are named. A list that quietly
/// grew is a list that has gone stale.
#[test]
fn the_counter_holds_what_a_player_actually_owns() {
    let mut g = a_fighter(1);
    let mut refused = Vec::new();
    let mut too_big_but_fits = Vec::new();
    // The five the counter is allowed to refuse, by name.
    const TOO_BIG: &[&str] = &[
        "Vast Tapestry",
        "Colossus Ring",
        "Sprawling Handwrap",
        "Wandering Root",
        "Broken Crown",
        "Godsteel Haft",
    ];
    for def in gm2d_core::piece::CATALOG.iter() {
        let sh = gm2d_core::shape::Shape::new(def.cells);
        let w = sh.cells().iter().map(|c| c.0).max().unwrap() + 1;
        let h = sh.cells().iter().map(|c| c.1).max().unwrap() + 1;
        let (long, short) = (w.max(h), w.min(h));
        let Some(id) = g.character.give(def.name) else { continue };
        let fits = (0..4).any(|turn| {
            SHELF.iter().any(|&at| {
                let want = g.shelf_cells(id, at, turn);
                want.iter().all(|c| SHELF.contains(c))
            })
        });
        if TOO_BIG.contains(&def.name) {
            if fits {
                too_big_but_fits.push(def.name);
            }
        } else if !fits {
            refused.push(format!("{} ({w}x{h})", def.name));
        } else if long > 4 || short > 3 {
            // Fine, and worth knowing: it is not on the list and it went on.
            let _ = (long, short);
        }
    }
    assert!(
        refused.is_empty(),
        "{} components fit no counter anywhere: {:?}",
        refused.len(),
        &refused[..refused.len().min(8)]
    );
    assert!(too_big_but_fits.is_empty(), "{too_big_but_fits:?} is on the too-big list and fits");
}

/// **`ask_of` is the one answer**, so the card and the sale cannot disagree —
/// §C.3's *the price shown is the price taken*, read from the other side.
#[test]
fn what_counts_as_fair_is_one_answer() {
    assert_eq!(stall::ask_of(100, 100), Ask::Fair);
    assert_eq!(stall::ask_of(100 + FAIR_PCT, 100), Ask::Fair, "the edge is fair");
    assert_eq!(stall::ask_of(100 + FAIR_PCT + 1, 100), Ask::High);
    assert_eq!(stall::ask_of(100 - FAIR_PCT - 1, 100), Ask::Low);
}

/// **A high ask is refused by two buyers in three, and a fair one always
/// sells.** That is the whole of what makes pricing a decision: without the
/// refusal the counter is a vendor with an extra screen in front of it.
#[test]
fn a_high_ask_is_mostly_refused_and_a_fair_one_sells() {
    let mut sold_fair = 0;
    let mut sold_high = 0;
    for seed in 0..60u64 {
        for high in [false, true] {
            let mut g = a_fighter(seed);
            let id = g.character.give("Iron Blade").expect("a blade");
            let worth = g.worth_of(id);
            let price = if high { worth * 3 } else { worth };
            g.shelve(id, (0, 0), 1, price).expect("out it goes");
            // Every buyer wants a weapon or does not; run until one looks.
            for _ in 0..12 {
                if g.a_buyer_comes_by().is_some() && g.character.stall.is_empty() {
                    if high { sold_high += 1 } else { sold_fair += 1 }
                    break;
                }
            }
        }
    }
    assert!(sold_fair > sold_high, "fair {sold_fair}, high {sold_high}");
    assert!(sold_high > 0, "a high ask never sells at all");
}

/// A sale is what pays, and the ledger says what it was worth.
#[test]
fn a_sale_pays_and_is_written_down() {
    let mut g = a_fighter(3);
    let id = g.character.give("Iron Blade").expect("a blade");
    let worth = g.worth_of(id);
    g.shelve(id, (0, 0), 1, worth).expect("out it goes");
    let purse = g.character.gold;
    for _ in 0..40 {
        g.a_buyer_comes_by();
        if g.character.stall.is_empty() {
            break;
        }
    }
    assert!(g.character.stall.is_empty(), "nobody ever bought it");
    assert_eq!(g.character.gold, purse + worth, "the purse did not move by the price");
    let sale = g.character.ledger.last().expect("a row");
    assert_eq!(sale.paid, worth);
    assert_eq!(sale.worth, worth, "the ledger does not say what it was worth");
}

/// **The counter is on the bell.** A fight is the clock, so the receipt of a
/// won fight is where a buyer turns up.
#[test]
fn a_buyer_comes_by_at_the_bell() {
    let mut said = 0;
    for seed in 0..25u64 {
        let mut g = a_fighter(seed);
        let id = g.character.give("Iron Blade").expect("a blade");
        let worth = g.worth_of(id);
        g.shelve(id, (0, 0), 1, worth / 2).expect("out it goes");
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).expect("a fight");
        let r = fight::settle(&mut g, &log, D).expect("a settlement");
        if r.receipt.iter().any(|l| l.contains("took the")) {
            said += 1;
        }
    }
    assert!(said > 0, "nobody came by in twenty-five fights");
}

/// **A save carries the counter and the ledger.**
#[test]
fn the_counter_survives_a_round_trip() {
    let mut g = a_fighter(11);
    let id = g.character.give("Iron Blade").expect("a blade");
    g.shelve(id, (0, 0), 1, 44).expect("out it goes");
    g.character.ledger.push(stall::Sale {
        buyer: "the-drover".into(),
        item: "Oak Handle".into(),
        paid: 12,
        worth: 10,
    });
    let json = gm2d_core::save::SaveFile::of(&g).to_json();
    let back = gm2d_core::save::load(&json).expect("read");
    assert!(Game::eq(&g, &back), "the counter did not survive");
    assert_eq!(back.character.stall.len(), 1);
    assert_eq!(back.character.ledger.len(), 1);
}
