//! Becoming something, and the two ways the bench pulls apart.
//!
//! **The Apothecary shipped unreachable.** `Character::specialization` was
//! written by exactly one line in the repository — the save loader — so a
//! tree, a power, a theme name and five honoured arms sat behind no door at
//! all. `every_offered_class_reaches_something` could not see it, because a
//! specialization is deliberately outside `class::OFFERED`: *a lint that reads
//! a list rather than the behaviour is the failure it exists to catch*, and the
//! list it reads had a hole exactly the shape of the new feature.
//!
//! So the first test here is the one that would have caught it, and it asks the
//! maps rather than the roster.

use gm2d_core::class::SPECIALIZATIONS;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::PlaceKind;


const D: Difficulty = Difficulty::Easy;

/// **Every specialization has a trainer standing somewhere a player can reach.**
///
/// The lint the Apothecary needed and did not have. Asked over
/// `SPECIALIZATIONS` rather than a list written here, for the reason every
/// completeness lint in this repository is: a list of two written by hand is a
/// list that can be one.
#[test]
fn every_specialization_has_a_trainer() {
    let mut taught: Vec<(String, String)> = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            if let Some(c) = p.teaches.clone() {
                assert_eq!(p.kind, PlaceKind::Bench, "{}: only a bench teaches", p.id);
                taught.push((c, id.to_string()));
            }
        }
    }
    for c in SPECIALIZATIONS {
        assert!(
            taught.iter().any(|(t, _)| t == c),
            "{c} is something you can be and nobody teaches it"
        );
    }
    // And exactly one each: *one trainer you can find somewhere on the map* is
    // the ask, and two would make the second one dead content.
    for c in SPECIALIZATIONS {
        let n = taught.iter().filter(|(t, _)| t == c).count();
        assert_eq!(n, 1, "{c} has {n} trainers");
    }
    // Nobody teaches something that is not one.
    for (c, map) in &taught {
        assert!(SPECIALIZATIONS.contains(&c.as_str()), "{map} teaches {c:?}, which is nothing");
    }
}

/// One only, it does not come off, and the refusal names what is in the way.
#[test]
fn one_only_and_it_does_not_come_off() {
    let mut g = Game::new(0x5EED_0000_5EC1_0001, "td");
    assert!(g.character.specialization.is_none(), "nobody starts as anything");

    assert_eq!(g.train("Apothecary").unwrap(), "Apothecary");
    assert_eq!(g.character.specialization.as_deref(), Some("Apothecary"));

    // The same one again, and the other one.
    assert!(g.train("Apothecary").unwrap_err().contains("already"));
    let why = g.train("Chef").unwrap_err();
    assert!(why.contains("Apothecary") && why.contains("two things"), "{why}");
    assert_eq!(g.character.specialization.as_deref(), Some("Apothecary"), "a refusal changed it");

    // And nothing that is not one.
    assert!(g.train("Berserker").unwrap_err().contains("not something anybody trains"));
}

/// **The tree is spendable, which is the other half of the same bug.**
///
/// The tree screen filters on the classes you are, and a specialization is not
/// one of the three — so even a character who somehow had it set could not
/// spend a point in its tree.
#[test]
fn a_specialization_can_spend_in_its_own_tree() {
    let mut g = Game::new(0x5EED_0000_5EC1_0001, "td");
    let sees = |g: &gm2d_core::game::Game| -> Vec<String> {
        g.character.spendable_trees().map(|s| s.to_string()).collect()
    };
    assert!(!sees(&g).contains(&"Chef".to_string()));
    g.train("Chef").unwrap();
    assert!(sees(&g).contains(&"Chef".to_string()), "a Chef cannot reach the Chef tree");
    // And it is still not one of the classes, which every pairing question asks.
    assert!(!g.character.classes().any(|c| c == "Chef"), "a specialization joined the classes");
}

/// **A Chef holds more than one, and everybody else holds one.**
#[test]
fn a_chef_drinks_more_than_one() {
    let mut g = Game::new(0x5EED_0000_5EC1_0001, "td");
    assert_eq!(g.character.draughts(), 1, "everybody starts at one");
    g.train("Chef").unwrap();
    assert!(g.character.draughts() > 1, "a Chef holds {}", g.character.draughts());
    // The tree only ever raises it.
    let base = g.character.draughts();
    let tree = data::skills();
    for n in tree.trees.iter().filter(|t| t.class.as_deref() == Some("Chef")).flat_map(|t| &t.nodes)
    {
        g.character.skills_taken.push(n.id.clone());
    }
    assert!(g.character.draughts() > base, "a finished Chef tree bought nothing");
}

/// **An ink is never half a pair**, which is what makes a trainer's ingredient
/// cost the brew table nothing.
#[test]
fn a_trainers_ingredient_is_an_ink_and_not_a_pair() {
    let b = data::brews();
    let inks: Vec<&str> =
        b.ingredients.iter().filter(|i| i.ink_only).map(|i| i.id.as_str()).collect();
    assert!(!inks.is_empty(), "no trainer sells anything");
    for i in &inks {
        for j in b.ingredients.iter().map(|x| x.id.as_str()) {
            assert!(b.pair(i, j).is_none(), "{i} is half of a pair with {j}");
        }
    }
}

/// A trainer sells to their own kind and says so to everybody else, and **a
/// refusal spends nothing**.
#[test]
fn a_trainer_sells_to_their_own() {
    let ink = data::brews()
        .ingredients
        .iter()
        .find(|i| i.ink_only)
        .expect("a trainer sells something")
        .clone();
    let mut g = Game::new(0x5EED_0000_5EC1_0001, "td");
    g.character.gold = 10_000;

    let before = g.character.gold;
    let why = g.buy_ingredient(&ink.id, 400, "Apothecary").unwrap_err();
    assert!(why.contains("Apothecary"), "{why}");
    assert_eq!(g.character.gold, before, "a refusal spent money");
    assert_eq!(g.character.in_larder(&ink.id), 0);

    g.train("Apothecary").unwrap();
    assert_eq!(g.buy_ingredient(&ink.id, 400, "Apothecary").unwrap(), ink.name);
    assert_eq!(g.character.gold, before - 400);
    assert_eq!(g.character.in_larder(&ink.id), 1);

    // And no money is no purchase.
    g.character.gold = 399;
    let why = g.buy_ingredient(&ink.id, 400, "Apothecary").unwrap_err();
    assert!(why.contains("399"), "{why}");
    assert_eq!(g.character.in_larder(&ink.id), 1, "a refusal handed one over");
}

/// **A trainer is never on a table**, because a counter does not catch the ball.
///
/// The Chef's trainer was authored onto the Undercountry, which is a shot map:
/// the arrow keys aim a cue there rather than taking a step, and `catches` is
/// **gates and bosses** — a town stands in the middle of the Undercountry's one
/// fast lane and catching every shot down it would be a road nobody can use. So
/// a bench on a table is a bench you reach with a hole in one.
///
/// `every_place_on_a_shot_map_is_reachable_by_shots` passes it, and is right
/// to: the tile *is* landable, which is the question that lint asks. This is
/// the sharper one, and it is narrow on purpose — a specialization is taken
/// once, permanently, from one person in the whole world, and putting that
/// behind an exact landing is putting it behind a dice roll.
#[test]
fn a_trainer_is_on_ground_you_walk() {
    use gm2d_core::world::Traversal;
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        if w.traversal != Traversal::Shot {
            continue;
        }
        for p in &w.places {
            assert!(
                p.teaches.is_none() && p.stocks.is_empty(),
                "{} is a counter on a table, which is a hole in one",
                p.id
            );
        }
    }
}

/// **`an Apothecary`, `a Chef`.** One of the two starts with a vowel, and the
/// sentence is on the screen where a permanent choice is confirmed.
///
/// Over `SPECIALIZATIONS` rather than the two written here — *a glossary is a
/// proofreading surface*, and so is a receipt.
#[test]
fn nobody_is_a_apothecary() {
    for c in SPECIALIZATIONS {
        let said = gm2d_core::class::an(c);
        let want = if c.starts_with(['A', 'E', 'I', 'O', 'U']) { "an " } else { "a " };
        assert!(said.starts_with(want), "{said:?} is not how you say it");
    }
    // And the refusal a trainer gives, which is the one a player reads most.
    let mut g = Game::new(0x5EED_0000_5EC1_0002, "td");
    g.train("Apothecary").unwrap();
    let why = g.train("Chef").unwrap_err();
    assert!(why.contains("an Apothecary"), "{why}");
}
