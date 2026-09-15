//! An errand for every bench.
//!
//! M21.16. Asked for: *a quest chain for each new system, that the quests are
//! predicated upon using the system.*

use gm2d_core::data;
use gm2d_core::quest::{Goal, Shown};

mod common;

/// Every bench in the game has a chain about it.
///
/// **Asked over the benches rather than over a list of four**, which is the
/// `C(8,2)` argument in its smallest form: a list of four written by hand is a
/// list that can be three, and the day somebody adds a fifth bench this is what
/// says it has nothing pointing at it.
#[test]
fn every_bench_has_a_chain() {
    let quests = data::quests();
    // The benches, as `Shown::bench` names them — one answer, so a bench that
    // is renamed is renamed here too.
    let benches: Vec<&str> = {
        let mut v: Vec<&str> = Vec::new();
        for s in every_shown() {
            let b = s.bench();
            if !v.contains(&b) {
                v.push(b);
            }
        }
        v
    };
    assert_eq!(benches.len(), 4, "four benches: {benches:?}");
    for b in &benches {
        let about: Vec<&str> = quests
            .quests
            .iter()
            .filter(|q| matches!(&q.goal, Goal::Show { what } if what.bench() == *b))
            .map(|q| q.id.as_str())
            .collect();
        assert!(
            about.len() >= 3,
            "{b} has {} errands about it and a chain is three: {about:?}",
            about.len()
        );
        // **A chain and not three loose errands.** Every rung past the first
        // requires the one before it, which is what makes the log draw them as
        // a tree rather than as three things standing side by side.
        let rungs: Vec<&gm2d_core::quest::Quest> = quests
            .quests
            .iter()
            .filter(|q| matches!(&q.goal, Goal::Show { what } if what.bench() == *b))
            .collect();
        let roots = rungs.iter().filter(|q| q.requires.is_empty()).count();
        assert_eq!(roots, 1, "{b}'s chain has {roots} roots: {about:?}");
        for q in rungs.iter().filter(|q| !q.requires.is_empty()) {
            assert!(
                q.requires.iter().all(|r| about.contains(&r.as_str())),
                "{} requires something outside its own chain: {:?}",
                q.id,
                q.requires
            );
            assert!(q.granted, "{} is a chain rung and is not granted", q.id);
        }
    }
}

/// **Every rung is a thing the bench already does.**
///
/// The constraint the milestone is written to: an errand that needed a new
/// mechanic would be a milestone about the mechanic. So every figure a rung
/// asks for is one this game can actually produce — a brew that reaches the
/// stat, a crop that reaches the count, creatures the kennel will hold, a sale
/// the counter can make.
#[test]
fn every_rung_is_a_thing_the_bench_already_does() {
    let quests = data::quests();
    let brews = data::brews();
    let mut bad = Vec::new();
    for q in &quests.quests {
        let Goal::Show { what } = &q.goal else { continue };
        match what {
            Shown::Brew { stat, at_least } => {
                // Some brew in the game reaches it. A threshold no pair can
                // meet is a wall with a sentence on it.
                let best = brews.brews.iter().map(|b| b.gives.of(stat)).max().unwrap_or(0);
                if best < *at_least {
                    bad.push(format!(
                        "{}: wants {at_least} {stat} and the best brew gives {best}",
                        q.id
                    ));
                }
            }
            Shown::BrewOf { cells, each } => {
                let big: Vec<u32> = brews
                    .ingredients
                    .iter()
                    .map(|i| i.shape().cells().len() as u32)
                    .filter(|n| n >= each)
                    .collect();
                // Two of them have to exist *and* fit the glass together.
                let fits = big.len() >= 2 && big.iter().take(2).sum::<u32>() >= *cells;
                let room = gm2d_core::brew::RETORT.len() as u32;
                if !fits || *cells > room + 1 {
                    bad.push(format!(
                        "{}: wants {cells} cells of {each}s; the glass is {room} and there are \
                         {} ingredients that big",
                        q.id,
                        big.len()
                    ));
                }
            }
            Shown::Grown { crop, n } => {
                if brews.get(crop).is_none() {
                    bad.push(format!("{}: {crop:?} is no ingredient", q.id));
                }
                if data::plot().seeds.iter().all(|s| s.crop != *crop) {
                    bad.push(format!("{}: nothing grows into {crop:?}", q.id));
                }
                // One harvest has to be able to reach it, or it is several
                // rows and the errand does not say so.
                if *n > gm2d_core::plot::HARVEST_YIELD * 2 {
                    bad.push(format!("{}: wants {n} and a row pays {}", q.id,
                        gm2d_core::plot::HARVEST_YIELD));
                }
            }
            Shown::Kennelled { creatures } | Shown::Together { creatures } => {
                let fams = data::art_families();
                let mut seen: Vec<String> = Vec::new();
                for c in creatures {
                    let Some(spec) = gm2d_core::combat::creature(c) else {
                        bad.push(format!("{}: {c:?} is no creature", q.id));
                        continue;
                    };
                    // **A boss is never kennelled**, which is the block's own
                    // standing constraint — so an errand asking for one is an
                    // errand nobody can finish.
                    if spec.rank == gm2d_core::combat::Rank::Boss {
                        bad.push(format!("{}: {c} is a boss and a boss is never kennelled", q.id));
                    }
                    // **One a family**, which is what the kennel holds — so two
                    // creatures of one family can never both be in it.
                    let Some(f) = fams.get(c) else {
                        bad.push(format!("{}: {c} has no family", q.id));
                        continue;
                    };
                    if seen.contains(f) {
                        bad.push(format!(
                            "{}: {c} is family {f} and so is something else it asks for — \
                             the kennel holds one a family",
                            q.id
                        ));
                    }
                    seen.push(f.clone());
                }
                if let Shown::Together { creatures } = what {
                    // And they have to fit the run at once.
                    let (_, mouths, _, _, _) = gm2d_core::character::Character::new().handler();
                    if creatures.len() as u32 > mouths.max(2) {
                        bad.push(format!(
                            "{}: wants {} out at once and a run holds {mouths}",
                            q.id,
                            creatures.len()
                        ));
                    }
                }
            }
            Shown::Sold { at_least } => {
                // Something in the catalogue is worth it at the shelf's own
                // figure, or the counter can never make the sale at a fair ask.
                let best = gm2d_core::piece::CATALOG
                    .iter()
                    .map(gm2d_core::shop::shelf_price)
                    .max()
                    .unwrap_or(0);
                if *at_least > best {
                    bad.push(format!(
                        "{}: wants {at_least} and the dearest thing in the game is {best}",
                        q.id
                    ));
                }
            }
            Shown::SoldHigh => {}
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("; "));
}

/// **Every rung says what it wants, in the engine's own words.**
///
/// TONE 13a: somebody reading *brew one that gives at least 300 max health* is
/// working out what to go and do, so the figure is unthemed and the sentence is
/// core's. What this refuses is a sentence with a raw id in it.
#[test]
fn every_rung_says_what_it_wants() {
    for s in every_shown() {
        let line = s.ask();
        assert!(!line.is_empty(), "{s:?} says nothing");
        assert!(
            !line.contains('-') || line.contains(" - "),
            "{s:?} reads out a raw id: {line:?}"
        );
        assert!(
            line.chars().next().is_some_and(|c| c.is_lowercase()),
            "an ask is a phrase, not a sentence: {line:?}"
        );
    }
}

/// Every `Shown` a shipped errand carries.
fn every_shown() -> Vec<Shown> {
    data::quests()
        .quests
        .iter()
        .filter_map(|q| match &q.goal {
            Goal::Show { what } => Some(what.clone()),
            _ => None,
        })
        .collect()
}

/// **A rung is `Ready` the moment the bench has done it, and not before.**
///
/// Read fresh off the character, the way `Clear` reads `answered` — so an
/// errand taken after the fact is `Ready` the moment it is taken, which is
/// right: you did the thing.
#[test]
fn a_rung_reads_the_bench_and_not_a_counter() {
    let mut g = gm2d_core::game::Game::new(4, "td");
    g.character = common::bench();
    let quests = data::quests();
    let q = quests
        .quests
        .iter()
        .find(|q| matches!(&q.goal, Goal::Show { what: Shown::Sold { .. } }))
        .expect("a selling rung");
    g.world.quests_taken.push(q.id.clone());
    assert!(
        matches!(gm2d_core::quest::stage(&g, q), gm2d_core::quest::Stage::Carrying { .. }),
        "ready before anything was sold"
    );
    g.character.ledger.push(gm2d_core::stall::Sale {
        buyer: "the-drover".into(),
        item: "Iron Blade".into(),
        paid: 5_000,
        worth: 75,
    });
    assert!(
        matches!(gm2d_core::quest::stage(&g, q), gm2d_core::quest::Stage::Ready),
        "sold something and the rung is still carrying"
    );
}
