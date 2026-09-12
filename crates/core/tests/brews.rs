//! The larder and the bench.

use gm2d_core::brew::{self, Gives};
use gm2d_core::data;
use gm2d_core::game::Game;

/// The shipped table parses and every pair of eight is in it.
///
/// **A list of twenty-eight written by hand is a list that can be twenty-seven**,
/// which is the argument `every_pair_of_offered_classes_reaches_an_expert`
/// makes for the ten experts and is why the table is complete rather than
/// representative.
#[test]
fn every_pair_of_ingredients_brews_to_something() {
    let b = data::brews();
    let ids: Vec<&str> = b.ingredients.iter().map(|i| i.id.as_str()).collect();
    let want = ids.len() * (ids.len() - 1) / 2;
    assert_eq!(b.brews.len(), want, "{} ingredients want {want} pairs", ids.len());
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
        let (a, c) = (ids[i], ids[j]);
        let got = b.pair(a, c).unwrap_or_else(|| panic!("{a} and {c} brew to nothing"));
        // Either order, because which one you dropped in first is a fact about
        // your hand and not about the brew.
        assert_eq!(b.pair(c, a).map(|d| &d.name), Some(&got.name), "{a}/{c} is order-sensitive");
        assert!(!got.gives.is_nothing(), "{} gives nothing", got.name);
        assert!(!got.name.is_empty() && !got.blurb.is_empty(), "{a}/{c} says nothing");
        }
    }
    // And no two pairs share a name, or a receipt naming one is a receipt that
    // is wrong about what you drank.
    let mut names: Vec<&str> = b.brews.iter().map(|d| d.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "two brews with one name");
}

/// **Every creature leaves an ingredient**, and it is keyed by the family it is
/// drawn from rather than by its name.
///
/// So a creature added to `enemies.json` cannot arrive without a drop: it
/// already fails a test without an art family, and this is the second half of
/// that. A list of seventy-five creature names would be the seventh
/// hand-written list this project has paid for.
#[test]
fn every_creature_leaves_something_for_the_larder() {
    let families = data::art_families();
    let b = data::brews();
    let mut bare = Vec::new();
    for m in gm2d_core::combat::LADDER {
        match families.get(m.name).and_then(|f| b.from_family(f)) {
            Some(_) => {}
            None => bare.push(m.name),
        }
    }
    assert!(bare.is_empty(), "creatures that leave nothing: {bare:?}");
    // And the other direction: an ingredient nothing drops is an ingredient
    // nobody can brew with, which is a pair that cannot be made.
    for i in &b.ingredients {
        let dropped = gm2d_core::combat::LADDER
            .iter()
            .any(|m| families.get(m.name).map(|f| i.from.contains(f)).unwrap_or(false));
        assert!(dropped, "{} is dropped by nothing in the ladder", i.id);
    }
}

/// **The glass is a constraint, not a label.**
///
/// A brewing window every pair fits in is an inventory slot with a different
/// name: the arrangement would carry nothing, and *a check that compares zero
/// with zero is not a check*. So this measures both ends — at two ingredients
/// it has to be generous, and at three it has to bite.
#[test]
fn the_retort_is_a_shape_and_not_a_box() {
    let b = data::brews();
    let ids: Vec<String> = b.ingredients.iter().map(|i| i.id.clone()).collect();
    let small = brew::RETORT.to_vec();
    let mut big = small.clone();
    big.extend_from_slice(brew::REBLOWN);

    let mut pairs_in = 0;
    let mut pairs_out = 0;
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            if brew::fits(&small, &b, &[ids[i].clone(), ids[j].clone()]) {
                pairs_in += 1;
            } else {
                pairs_out += 1;
            }
        }
    }
    let mut triples_in = 0;
    let mut triples_out = 0;
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            for k in (j + 1)..ids.len() {
                if brew::fits(&big, &b, &[ids[i].clone(), ids[j].clone(), ids[k].clone()]) {
                    triples_in += 1;
                } else {
                    triples_out += 1;
                }
            }
        }
    }
    println!(
        "{} cells: {pairs_in} pairs in, {pairs_out} out. \
         {} cells: {triples_in} triples in, {triples_out} out.",
        small.len(),
        big.len()
    );
    // **Generous at two and tight at three**, which is the trade the glass is
    // for: a bench you cannot brew a pair at is a bench nobody uses, and a
    // reblown glass every triple goes into is a free slot with a ceremony in
    // front of it.
    assert_eq!(pairs_out, 0, "{pairs_out} pairs will not go in the glass at all");
    assert!(triples_in > 0, "no ink can ever be added, so the reward is nothing");
    assert!(
        triples_out > 0,
        "every triple fits the reblown glass, so its shape carries nothing at all"
    );
    // The reblowing only ever adds. A glass that got smaller would drop
    // whatever was standing in it, which is `resize_boards`'s rule one system
    // along.
    for c in &small {
        assert!(big.contains(c), "reblowing the glass lost {c:?}");
    }
}

/// **An ink multiplies and does not add**, which is the ask in as many words.
#[test]
fn an_ink_is_a_percentage_on_the_pair() {
    let b = data::brews();
    let pair = &b.brews[0];
    let ink = b.ingredients.iter().find(|i| !pair.of.contains(&i.id)).expect("a third thing");
    let plain = pair.gives;
    let inked = plain.scaled(ink.potency);
    assert_ne!(plain, inked, "{}: an ink that changes nothing", ink.id);
    // Every non-zero figure moved the same way, and nothing that was zero grew
    // — a multiplier that invented a lane would be a third ingredient that is
    // a second pair.
    assert_eq!(plain.is_nothing(), inked.is_nothing());
    assert_eq!(
        Gives::default().scaled(ink.potency),
        Gives::default(),
        "an ink on nothing is still nothing"
    );
}

/// Brewing takes the ingredients, and a refusal takes nothing.
#[test]
fn a_refused_brew_spends_nothing() {
    let mut g = Game::new(5, "td");
    g.character.gather("kettle-scale");
    g.character.gather("rust-bloom");
    // Two you have not got.
    let err = g.brew(&["reef-salt".into(), "bone-meal".into()]).unwrap_err();
    assert!(err.contains("you have not got"), "{err}");
    assert_eq!(g.character.in_larder("kettle-scale"), 1, "a refusal spent one anyway");
    // One that is not an ingredient at all.
    assert!(g.brew(&["a-turnip".into(), "kettle-scale".into()]).is_err());
    // Three, before the glass has been reblown.
    let err = g
        .brew(&["kettle-scale".into(), "rust-bloom".into(), "kettle-scale".into()])
        .unwrap_err();
    assert!(err.contains("the glass holds two"), "{err}");
    // And the one that works.
    let name = g.brew(&["kettle-scale".into(), "rust-bloom".into()]).expect("it brews");
    assert_eq!(name, "Searing Draught");
    assert_eq!(g.character.in_larder("kettle-scale"), 0);
    assert_eq!(g.character.in_larder("rust-bloom"), 0);
    // Tipping it out puts them back, because a brew you have not drunk is a
    // decision you have not taken.
    g.tip_out();
    assert_eq!(g.character.in_larder("kettle-scale"), 1);
    assert!(g.character.brewed.is_empty());
}

/// **A brew reaches the bell and does not outlive it.**
///
/// The first half is what a potion is for; the second is the whole of what
/// *temporary* means, and it is free because `Held` is translated into a
/// `Combatant` at the bell and nothing persists.
#[test]
fn a_brew_is_on_you_at_the_bell_and_gone_after() {
    let mut g = Game::new(11, "td");
    g.character.gather("kettle-scale");
    g.character.gather("toad-ichor");
    let before = g.character.start_with();
    g.brew(&["kettle-scale".into(), "toad-ichor".into()]).expect("it brews");
    let after = g.character.start_with();
    assert_ne!(before.stats, after.stats, "the brew reached nothing at the bell");
    assert_eq!(after.stats.strength - before.stats.strength, 14, "Lamp-Oil Wash is +14 strength");
    // And the potion is not the character: `player_stats` is what you are, and
    // it must not move.
    assert_eq!(
        Game::new(11, "td").character.player_stats(),
        g.character.player_stats(),
        "a brew changed what the character is"
    );
}

/// Every ingredient says what it is, and the glass can hold each on its own.
#[test]
fn every_ingredient_says_something_and_goes_in_the_glass() {
    let b = data::brews();
    for i in &b.ingredients {
        assert!(!i.name.is_empty() && !i.blurb.is_empty(), "{} says nothing", i.id);
        assert!(
            brew::fits(brew::RETORT, &b, std::slice::from_ref(&i.id)),
            "{} does not go in the glass on its own",
            i.id
        );
        assert!(i.potency > 0, "{} is an ink that multiplies by nothing", i.id);
    }
}

/// What a brew is worth is a sentence, derived rather than typed.
#[test]
fn a_brew_says_what_it_does_in_numbers() {
    let b = data::brews();
    for d in &b.brews {
        let line = d.gives.line();
        assert_ne!(line, "nothing at all", "{}: says nothing", d.name);
        assert!(line.contains('+') || line.contains('-'), "{}: {line}", d.name);
    }
}
