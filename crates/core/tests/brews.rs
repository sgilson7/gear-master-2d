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
    // **The pair table is over the ingredients that pair**, which is every one
    // a creature drops and none of the ones a trainer sells. An ink-only
    // ingredient multiplies a pair rather than halving one — see
    // `IngredientDef::ink_only`, which carries the arithmetic that made that
    // the cheap shape: two pairing ingredients on top of eight want seventeen
    // new brews written, and the next two want nineteen more.
    let ids: Vec<&str> =
        b.ingredients.iter().filter(|i| !i.ink_only).map(|i| i.id.as_str()).collect();
    assert!(
        b.ingredients.iter().any(|i| i.ink_only),
        "no ingredient is ink-only, so the exemption above is untested"
    );
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
    //
    // **Unless a trainer sells it.** An ink-only ingredient is bought, not
    // dropped — `every_special_ingredient_is_on_a_counter` is that half, and it
    // asks the maps, which this test cannot: a lint that let an ink off this
    // hook without somebody else holding it would be the orphan rule with a
    // door in it.
    for i in b.ingredients.iter().filter(|i| !i.ink_only) {
        let dropped = gm2d_core::combat::LADDER
            .iter()
            .any(|m| families.get(m.name).map(|f| i.from.contains(f)).unwrap_or(false));
        assert!(dropped, "{} is dropped by nothing in the ladder", i.id);
    }
}

/// **An ingredient nothing drops is one somebody sells, and this is where that
/// is checked.**
///
/// The other half of the orphan rule. `BrewsData::parse` lets an `ink_only`
/// ingredient have no `from`, because a trainer's stock is not a drop — and
/// that would be a hole rather than a rule if nothing asked the maps whether
/// anybody actually stocks it. A special ingredient on no counter is content
/// reachable from nowhere, which is exactly the shape the Apothecary itself
/// shipped in.
#[test]
fn every_special_ingredient_is_on_a_counter() {
    let b = data::brews();
    let stocked: Vec<String> = data::all_maps(gm2d_core::combat::Difficulty::Easy)
        .iter()
        .flat_map(|w| w.places.clone())
        .flat_map(|p| p.stocks)
        .collect();
    let special: Vec<&str> =
        b.ingredients.iter().filter(|i| i.ink_only).map(|i| i.id.as_str()).collect();
    assert!(!special.is_empty(), "no ingredient is ink-only, so this checks nothing");
    for id in special {
        assert!(stocked.iter().any(|s| s == id), "{id} is on nobody's counter");
    }
    // And nothing that *does* drop is also for sale: a thing you can farm and
    // buy is a thing the farming is pointless for.
    for id in &stocked {
        let def = b.ingredients.iter().find(|i| i.id == *id).expect("a real ingredient");
        assert!(def.ink_only, "{id} is sold and also drops off a creature");
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

/// **Seating is the puzzle, and a refusal takes nothing.**
///
/// The glass is a board now: an ingredient goes in *somewhere*, turned some
/// way, and where the others are sitting decides whether it fits. Every rule
/// about that is core's, for the reason the packing board's green fit preview
/// is core's.
#[test]
fn a_refused_seating_spends_nothing() {
    let mut g = Game::new(5, "td");
    g.character.gather("kettle-scale");
    g.character.gather("rust-bloom");

    // Somewhere it will not go.
    let err = g.seat_ingredient("kettle-scale", 0, [9, 9]).unwrap_err();
    assert!(err.contains("will not go there"), "{err}");
    assert_eq!(g.character.in_larder("kettle-scale"), 1, "a refusal spent one anyway");

    // Something you have not got.
    assert!(g.seat_ingredient("reef-salt", 0, [0, 0]).is_err());
    // Something that is not an ingredient at all.
    assert!(g.seat_ingredient("a-turnip", 0, [0, 0]).is_err());

    // And the two that do go in.
    let glass = g.retort();
    let b = data::brews();
    let one = brew::legal_anchors(&glass, &b, &[], "kettle-scale", 0)[0];
    g.seat_ingredient("kettle-scale", 0, one).expect("it seats");
    assert_eq!(g.character.in_larder("kettle-scale"), 0);
    assert_eq!(g.character.retort.len(), 1);

    // Nothing may sit on top of it.
    assert!(
        !brew::legal_anchors(&glass, &b, &g.character.retort, "rust-bloom", 0).contains(&one),
        "a second ingredient was offered the cell the first is standing on"
    );

    // Lifting it puts it back, because a brew you have not made is a decision
    // you have not taken.
    g.lift_from_glass(one[0], one[1]).expect("it comes out");
    assert_eq!(g.character.in_larder("kettle-scale"), 1);
    assert!(g.character.retort.is_empty());
    assert!(g.lift_from_glass(one[0], one[1]).is_err(), "lifted nothing twice");
}

/// **The brew button is a moment, and what comes out of it is a thing.**
///
/// Asked for: *press a brew button to actually turn your ingredients into a
/// potion, that can be activated before any fight.* So the glass is not the
/// potion — brewing empties it and puts something in the pack, and drinking is
/// a second decision after that.
#[test]
fn the_button_makes_a_potion_and_drinking_it_is_a_second_decision() {
    let mut g = Game::new(5, "td");
    let b = data::brews();
    g.character.gather("kettle-scale");
    g.character.gather("rust-bloom");

    assert!(g.brew().is_err(), "an empty glass brewed something");
    let glass = g.retort();
    for id in ["kettle-scale", "rust-bloom"] {
        let at = brew::legal_anchors(&glass, &b, &g.character.retort, id, 0)[0];
        g.seat_ingredient(id, 0, at).expect("it seats");
    }
    // Two things in and nothing drunk: the fight is still unchanged.
    assert!(g.character.boon().is_nothing(), "the glass alone reached the bell");

    let name = g.brew().expect("it brews");
    assert_eq!(name, "Searing Draught");
    assert!(g.character.retort.is_empty(), "the glass was not emptied");
    assert_eq!(g.character.potions.len(), 1, "nothing went in the pack");
    assert!(g.character.boon().is_nothing(), "a potion in the pack is already on you");

    let id = g.character.potions[0].clone();
    g.drink(&id).expect("it drinks");
    assert!(g.character.potions.is_empty());
    assert!(!g.character.boon().is_nothing(), "drinking it reached nothing");
    // One at a time: a second is refused and spends nothing.
    g.character.potions.push(id.clone());
    assert!(g.drink(&id).is_err(), "drank two at once");
    assert_eq!(g.character.potions.len(), 1, "a refusal spent one anyway");
}

/// **A brew reaches the bell and does not outlive it.**
///
/// The first half is what a potion is for; the second is the whole of what
/// *temporary* means, and it is free because `Held` is translated into a
/// `Combatant` at the bell and nothing persists.
#[test]
fn a_brew_is_on_you_at_the_bell_and_gone_after() {
    let mut g = Game::new(11, "td");
    let b = data::brews();
    g.character.gather("kettle-scale");
    g.character.gather("toad-ichor");
    let before = g.character.start_with();
    let glass = g.retort();
    for id in ["kettle-scale", "toad-ichor"] {
        let at = brew::legal_anchors(&glass, &b, &g.character.retort, id, 0)[0];
        g.seat_ingredient(id, 0, at).expect("it seats");
    }
    g.brew().expect("it brews");
    let id = g.character.potions[0].clone();
    g.drink(&id).expect("it drinks");
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
