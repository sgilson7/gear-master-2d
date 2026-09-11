//! The glossary is complete, and it says numbers rather than adjectives.
//!
//! Reported from play: *"can you also add a game glossary that explains
//! everything you need to play the game, with stuff like what mana
//! empowerment does"*.

use gm2d_core::glossary;

/// **Every class a player can be is in it**, which is the half that goes stale
/// on its own.
///
/// The shelves' prose is written by a person and will be read by one. The
/// class shelf is *derived* — `ClassPower::describe` writes each promise — so
/// what this guards is that the derivation still covers the roster: an expert
/// added without a glossary entry would be a thing a player can become and
/// cannot look up.
#[test]
fn every_class_a_player_can_be_is_in_the_glossary() {
    let shelf = glossary::shelves()
        .into_iter()
        .find(|s| s.name == "What you can become")
        .expect("there is a class shelf");
    let terms: Vec<&str> = shelf.entries.iter().filter_map(|e| e.key).collect();

    let mut missing = Vec::new();
    for c in gm2d_core::class::OFFERED {
        if !terms.contains(c) {
            missing.push(c.to_string());
        }
    }
    for e in gm2d_core::expert::EXPERTS {
        if !terms.contains(&e.name) {
            missing.push(e.name.to_string());
        }
    }
    assert!(missing.is_empty(), "not in the glossary: {missing:?}");
    assert_eq!(
        terms.len(),
        gm2d_core::class::OFFERED.len() + gm2d_core::expert::EXPERTS.len(),
        "the class shelf has entries for things nobody can become"
    );
}

/// **Nothing in it is blank**, which is the failure an empty box cannot be
/// told apart from.
#[test]
fn no_entry_says_nothing() {
    for sh in glossary::shelves() {
        assert!(!sh.entries.is_empty(), "{}: an empty shelf", sh.name);
        for e in &sh.entries {
            assert!(!e.term.is_empty(), "{}: an entry with no term", sh.name);
            assert!(!e.body.is_empty(), "{}: {:?} says nothing", sh.name, e.term);
            for line in &e.body {
                assert!(
                    !line.trim().is_empty(),
                    "{}: {:?} has a blank line in it",
                    sh.name,
                    e.term
                );
            }
        }
    }
}

/// **The numbers in it are the engine's**, checked where it is cheap to check.
///
/// The whole design of this file is that a figure is read from the constant
/// that decides it rather than typed — a glossary with its numbers written out
/// by hand is a second rulebook with a slower feedback loop than the first.
/// That cannot be asserted in general, so it is asserted where it would hurt
/// most: move a constant and the sentence about it must move too.
#[test]
fn the_numbers_are_read_and_not_typed() {
    let entries: Vec<gm2d_core::glossary::Entry> =
        glossary::shelves().into_iter().flat_map(|s| s.entries).collect();
    let body_of = |term: &str| -> String {
        entries
            .iter()
            .find(|e| e.term == term)
            .unwrap_or_else(|| panic!("no glossary entry called {term:?}"))
            .body
            .join(" ")
    };

    // **In its own entry, as a whole number.** Two weaker versions of this
    // both passed with the fatigue figure hardcoded to *nine*: `contains("4")`
    // because the cart's forty-Fnorp fare has a 4 in it, and then whole-number
    // matching over the *whole* glossary because a class promise says "every 4
    // seconds". A figure has to be checked where it is said, or the check is
    // asking whether the digit exists anywhere in the game — which it always
    // does. Third time lucky, and the first two were only ever found by
    // breaking it.
    for (term, what, n) in [
        ("Fatigue", "what a fight costs", gm2d_core::fatigue::PER_FIGHT.to_string()),
        ("Fatigue", "the cap", gm2d_core::fatigue::CAP.to_string()),
        ("Fatigue", "the hard cap", gm2d_core::fatigue::HARD_CAP.to_string()),
        ("Casting", "a cast's mana", gm2d_core::combat::SPELL_MANA_COST.to_string()),
        ("Resistance", "the resist cap", gm2d_core::stats::RESIST_CAP.to_string()),
        ("Resistance", "the lane cap", gm2d_core::stats::LANE_CAP.to_string()),
        ("Instant battle", "wins before a mark", gm2d_core::fight::INSTANT_AFTER.to_string()),
        ("The long cart", "the fare", gm2d_core::game::CART_FARE.to_string()),
        ("Rarity", "what Rare costs", gm2d_core::rating::RARE_AT.to_string()),
        ("Rarity", "what Epic costs", gm2d_core::rating::EPIC_AT.to_string()),
        ("Grids", "the tallest a grid gets", gm2d_core::progression::MAX_ROWS.to_string()),
        ("Enchs", "the licence", gm2d_core::ench::LICENCE_PRICE.to_string()),
        ("The spin", "a turn", gm2d_core::combat::SPIN_PCT_PER_TURN.to_string()),
    ] {
        let body = body_of(term);
        let said: Vec<&str> = body
            .split(|c: char| !c.is_ascii_digit())
            .filter(|t| !t.is_empty())
            .collect();
        assert!(
            said.contains(&n.as_str()),
            "{term:?} should say {what} is {n} and says {said:?} - a number that is \
             typed rather than read is a number that goes stale"
        );
    }
}

/// **And the pools are the same answer the standing panel gives.**
///
/// `Combatant::pool_pays` through `Stats::parts` is the one sentence, and a
/// first draft of this file listed the fields by hand instead — which dropped
/// **rage**, the pool every player meets first. Two answers to *what does a
/// pool pay* is exactly the shape this repository keeps paying for.
#[test]
fn the_pools_shelf_is_the_panels_answer() {
    let entry = glossary::shelves()
        .into_iter()
        .flat_map(|s| s.entries)
        .find(|e| e.term == "Pools")
        .expect("there is a pools entry");
    let body = entry.body.join(" ");
    for what in gm2d_core::combat::Combatant::pools_worth_holding() {
        let pays = gm2d_core::combat::Combatant::pool_pays(what);
        for (line, _) in pays.parts() {
            assert!(
                body.contains(&line),
                "{:?} pays {line:?} and the glossary does not say so",
                what
            );
        }
    }
}
