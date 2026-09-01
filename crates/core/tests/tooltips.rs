//! Everything on the road explains itself, in the engine's own words.
//!
//! Three small systems with one idea behind them: **the engine owns the
//! sentence**. A greyed choice, a rumour in the tray and a receipt after a
//! resolution are all the game saying what just happened or what would have to
//! be true - and if the interface writes those sentences, the CLI has to write
//! them again, the theme layer has nothing to swap, and the three drift.
//!
//! So `Requirement::describe`, `Outcome::describe`, `TownVisit::receipt` and
//! `rumour::conditions_line` all return canonical prose, and the display layer
//! swaps the nouns on the way out.
//!
//! The distinction that matters, and the one that is easy to lose: `unmet` is
//! flavour for the moment *after* you have tried a door - "Merrik does not move
//! the rope, and has not moved it in eleven years" - and `describe` is the
//! plain statement *before* you try. Both ship, and neither replaces the other.

mod common;

use gm2d_core::event::{Outcome, Requirement, EVENTS, TABLE_THREE};
use gm2d_core::character::Character;


// --------------------------------------------------------- what a door wants

#[test]
fn a_requirement_says_what_would_open_it() {
    assert_eq!(Requirement::None.describe(), "");
    assert_eq!(
        Requirement::LooseItemOfSize { w: 2, h: 2 }.describe(),
        "Requires: a loose component 2 by 2"
    );
    assert!(Requirement::Holding("Platinum Chip").describe().contains("Platinum Chip"));
    assert!(Requirement::Took("Ask how he does it").describe().contains("Ask how he does it"));
}

#[test]
fn every_locked_choice_in_the_game_can_say_why() {
    // The lint the tooltip buys. A door with a requirement and no sentence is
    // a door somebody has to guess at.
    for e in EVENTS {
        for c in e.choices {
            if c.requires == Requirement::None {
                continue;
            }
            let said = c.requires.describe();
            assert!(!said.is_empty(), "{}: {} cannot say what it wants", e.id, c.label);
            // "Requires: X" for a thing you have to hold or have done, and
            // "Costs X" for a price - because a price is not a condition, it
            // is a transaction, and a tooltip that called it a requirement
            // would not say that the gold goes.
            assert!(
                said.starts_with("Requires: ")
                    || said.starts_with("Costs ")
                    || said.starts_with("Name a figure"),
                "{}: {}",
                e.id,
                said
            );
            // And the flavour line is still there, which is a different job.
            assert!(!c.unmet.is_empty(), "{}: {} lost its prose", e.id, c.label);
        }
    }
}

// ------------------------------------------------------ what a choice hands over

#[test]
fn every_outcome_in_the_game_lists_its_deltas() {
    for e in EVENTS {
        for c in e.choices {
            let lines = c.outcome.describe();
            assert!(!lines.is_empty(), "{}: {} resolves into silence", e.id, c.label);
            for l in &lines {
                assert!(!l.trim().is_empty(), "{}: {} has an empty receipt line", e.id, c.label);
            }
        }
    }
}

#[test]
fn a_fight_an_event_arranges_says_what_is_at_stake_and_what_losing_costs() {
    let lines = Outcome::Step(&TABLE_THREE).describe();
    let all = lines.join(" | ");
    assert!(all.contains("Bone Archer") && all.contains("Frost Wisp"));
    assert!(all.contains("Platinum Chip"), "the reason to step in is not stated");
    assert!(all.contains("no life"), "the reason it is safe to step in is not stated");
}

#[test]
fn a_curated_shop_names_what_is_on_the_table_and_what_it_costs() {
    let vip = EVENTS.iter().find(|e| e.id == "the-vip-area").expect("authored");
    let keep = vip.choices.iter().find(|c| c.label == "Keep your face still").expect("authored");
    let all = keep.outcome.describe().join(" | ");
    assert!(all.contains("Overseer's Circlet"));
    assert!(all.contains("Immense Guilt"), "a cost that is not on the receipt is a cost nobody sees");
}

// -------------------------------------------------------------- the receipt









// ------------------------------------------------------ what a rumour is for






// ------------------------------------------------- whose activation, exactly

/// A watcher's description says *whose* activation it counts.
///
/// The Ratchet Cog said "every 8 activations, gain 1 spellblade". It counts
/// activations by your **other** items - `notify_watchers` skips the item that
/// acted and walks only its own side - so the one reading the words invited
/// was the one thing it does not do. Thirty pieces carried a watcher and every
/// one of them said it the same wrong way, because they all go through
/// `Trigger::describe`.
#[test]
fn a_watcher_says_whose_activation_it_is_counting() {
    use gm2d_core::piece::{Trigger, CATALOG};
    let mut checked = 0;
    for d in CATALOG {
        for t in d.triggers {
            let Trigger::Watch { .. } = t else { continue };
            checked += 1;
            let said = t.describe();
            assert!(
                said.contains("by your other items")
                    || said.contains("by another of your items")
                    || said.contains("by items ")
                    || said.contains("by a neighbour")
                    || said.contains("by an item ")
                    || said.contains("by a corner-neighbour")
                    || said.contains("curse"),
                "{} does not say whose: {:?}",
                d.name,
                said
            );
        }
    }
    assert!(checked > 20, "only {} watchers found - did the catalogue lose some?", checked);
}

/// And the plural lands on the right word.
///
/// The old line was `name() + "s"`, which is why the name had to be one word:
/// the plural of "activation by another of your items" is not that phrase with
/// an s on the end.
#[test]
fn a_watchers_phrase_pluralises_where_the_plural_belongs() {
    use gm2d_core::piece::Watched;
    for w in [
        Watched::AnyActivation,
        Watched::AdjacentActivation,
        Watched::DiagonalActivation,
        Watched::AlignedActivation,
        Watched::CurseApplied,
    ] {
        let one = w.counted(1);
        let many = w.counted(8);
        assert!(one.starts_with("1 "), "{:?} says {:?} for one", w, one);
        assert!(many.starts_with("8 "), "{:?} says {:?} for eight", w, many);
        assert!(!one.contains("ss"), "{:?} doubled an s: {:?}", w, one);
        assert!(!many.contains("itemss"), "{:?} bolted a plural on a phrase: {:?}", w, many);
        // The singular is singular and the plural is not.
        assert!(!one.contains("activations"), "{:?} pluralised one: {:?}", w, one);
    }
}
