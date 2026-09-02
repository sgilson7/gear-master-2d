//! A rule an item grants, and an item with a name somebody wrote.
//!
//! M9.0 is the milestone with no content in it: it widens `Effect::Grants` from
//! the skill tree to an assembled item, moves `Rule` out of `skills.rs` because
//! it is no longer the tree's, and adds the two seams every set in this block
//! needs. Nothing a player can see changes, which is the point — it is what
//! makes the next milestone small.
//!
//! **What is tested here and what is not.** Two of the tests `PLAN-M9.md` names
//! for this milestone — `a_rule_from_an_item_reaches_the_fight` and
//! `an_unassembled_set_grants_nothing` — need a component in `CATALOG` that
//! grants one, and there is not one until M9.2. They land there, against the
//! Mandate, and this file covers every seam that can be reached without
//! content: the agreement a name needs, the allowance a step reads, and the
//! rule list itself.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::loadout::agreed_name;
use gm2d_core::rng::Rng;
use gm2d_core::rule::{self, Rule};
use gm2d_core::world::{self, Allowances, Dir, WorldState};

const D: Difficulty = Difficulty::Easy;

// ------------------------------------------------------- a name somebody wrote

/// **Agreement, not one piece deciding.**
///
/// A set is its pieces. If any single component could name the finished item,
/// two thirds of a set plus a stranger would answer to the set's name — and the
/// stranger is the ordinary case, because five hundred and thirty-four of the
/// catalogue's components have nothing to say about it.
#[test]
fn a_named_set_needs_every_piece_to_agree() {
    let m = Some("The Rat King's Mandate");
    assert_eq!(agreed_name([m, m, m]), m.map(|s| s), "three of a set is the set");
    assert_eq!(agreed_name([m]), m.map(|s| s), "and so is a set of one");
    assert_eq!(agreed_name([m, m, None]), None, "a stranger in the item is a disagreement");
    assert_eq!(agreed_name([m, m, Some("The Toad's Own Frame")]), None, "so is another set");
    assert_eq!(
        agreed_name(Vec::<Option<&str>>::new()),
        None,
        "an item of no pieces is not any set"
    );
    // The ordinary item: nothing in it says anything, so the generator keeps
    // its job. This is the branch that runs five hundred times a screen.
    assert_eq!(agreed_name([None, None]), None);
}

// ------------------------------------------------------------ what a rule says

/// Every rule says what it does, with a number in it, in the engine's words.
///
/// The sibling of `skills_read::every_rule_is_described`, which covers the five
/// the tree grants. This one covers the two M9 adds and is here rather than
/// there for the reason `Rule` moved module: they are not the tree's.
#[test]
fn the_new_rules_describe_themselves() {
    const THEMED: &[&str] = &["fnorp", "the funny", "cork", "fury", "devotion", "harvest"];
    for r in [Rule::Rout { creature: "Cave Rat".into() }, Rule::Wade] {
        let line = r.line();
        assert!(!line.is_empty(), "{r:?} says nothing");
        assert!(line.chars().any(|c| c.is_ascii_digit()), "{r:?}: {line:?} names no number");
        let low = line.to_lowercase();
        for w in THEMED {
            assert!(!low.contains(w), "{r:?}: {line:?} speaks the theme");
        }
        assert!(!r.detail().is_empty(), "{r:?} explains nothing on hover");
        r.check().unwrap_or_else(|e| panic!("{r:?} is not a rule the engine has: {e}"));
    }
}

/// A rout naming nothing is a set bonus that can never fire.
///
/// The same refusal `CurseOnActivate` gets for a grid the engine has not got,
/// and for the same reason: a rule that grants nothing is the failure this
/// whole enum exists to stop shipping.
#[test]
fn a_rout_that_names_no_creature_is_refused() {
    assert!(Rule::Rout { creature: "Cave Rat".into() }.check().is_ok());
    assert!(Rule::Rout { creature: "Lord Drabley Henpeck".into() }.check().is_err(),
            "the themed name is not what the engine matches on");
    assert!(Rule::Rout { creature: String::new().into() }.check().is_err());
}

// -------------------------------------------------------------- what it routs

/// A rout is one creature's, by canonical name, and nobody else's.
#[test]
fn nothing_is_routed_but_what_the_rule_names() {
    let rules = vec![Rule::Rout { creature: "Cave Rat".into() }];
    assert!(rule::routs(&rules, "Cave Rat"));
    assert!(!rule::routs(&rules, "Bog Toad"), "one set is not a pass for the region");
    assert!(!rule::routs(&rules, "A. Rat"), "matched canonically, like a Slay goal");
    assert!(!rule::routs(&[], "Cave Rat"), "and nothing routs without the rule");
}

/// A character with an empty board and an empty tree has no rules at all.
///
/// The negative half of `Character::rules`, and the one that would have caught
/// a `start_with` that handed something out to everybody.
#[test]
fn a_starting_character_has_no_rules() {
    let c = Character::starting();
    assert!(c.rules().is_empty(), "a starting character got {:?}", c.rules());
    assert!(c.start_with().rules.is_empty());
    // The level rides in with the allowances and is not a rule — the map has
    // no other way to be told what a crossing asks about.
    assert_eq!(c.allowances(), Allowances { level: c.level(), ..Allowances::default() });
    assert!(!c.allowances().wade);
    assert!(!c.scouting());
}

// ------------------------------------------------------------------ the wade

/// The allowance a step reads is filled from a rule list, and from nothing else.
#[test]
fn wading_is_an_allowance_and_not_a_character() {
    assert!(!Allowances::of(&[]).wade);
    assert!(Allowances::of(&[Rule::Wade]).wade);
    // Every other rule is somebody else's business, and a step must not
    // quietly acquire one of them.
    assert_eq!(
        Allowances::of(&[
            Rule::Scout,
            Rule::SpinExtra { per_turn: 2 },
            Rule::Rout { creature: "Cave Rat".into() },
        ]),
        Allowances::default(),
    );
}

/// **Depth one, measured on the map rather than described.**
///
/// A water tile is shallow when something you could already stand on is one
/// orthogonal step away. On the first map that is fourteen of the lake's
/// twenty-eight tiles — the rim — and the middle fourteen stay shut. A corner
/// does not count: a diagonal touch is not somewhere to put a foot down on the
/// way in.
#[test]
fn the_rim_is_shallow_and_the_middle_is_not() {
    let w = data::world(D);
    let mut shallow = Vec::new();
    let mut deep = Vec::new();
    for y in 0..w.height {
        for x in 0..w.width {
            if w.terrain_name(x, y) != "water" {
                continue;
            }
            if w.shallow(x, y) { shallow.push([x, y]) } else { deep.push([x, y]) }
        }
    }
    assert_eq!(shallow.len() + deep.len(), 28, "the lake is twenty-eight tiles");
    assert_eq!(shallow.len(), 14, "the rim: {shallow:?}");
    assert_eq!(deep.len(), 14, "the middle: {deep:?}");
    // Row 9 end to end, which is what makes the lake crossable at all.
    for x in 7..=10 {
        assert!(w.shallow(x, 9), "({x}, 9) is the top edge of the lake");
    }
    // And the four the plan names as the deepest.
    for x in 7..=10 {
        assert!(!w.shallow(x, 11), "({x}, 11) is the middle");
    }
    // No dry tile is ever asked, but the answer had better not be nonsense.
    assert!(!w.shallow(0, 0), "the rock in the corner touches nothing walkable");
}

/// The step is where the wall is refused, and the allowance is what un-refuses
/// it.
#[test]
fn a_step_into_water_is_refused_until_it_is_allowed() {
    let w = data::world(D);
    let mut rng = Rng::new(9);
    // Standing on (7, 8), which is grass, with the lake's rim due south.
    let mut state = WorldState { at: [7, 8], ..WorldState::at_start(&w) };
    assert_eq!(w.terrain_name(7, 9), "water");

    let mut dry = state.clone();
    let s = world::step(&w, &mut dry, &mut rng, D, Dir::South, &Allowances::default());
    assert!(!s.moved, "water is a wall to everybody else");
    assert_eq!(dry.at, [7, 8], "and a refused step does not move you");
    assert!(s.blocked.as_deref().unwrap().contains("frame"), "{:?}", s.blocked);

    let s = world::step(&w, &mut state, &mut rng, D, Dir::South, &Allowances { wade: true, ..Allowances::default() });
    assert!(s.moved, "and ground to somebody who is allowed to wade");
    assert_eq!(state.at, [7, 9]);
    // The middle is still shut, allowance or not: depth is a property of the
    // ground, and this is the half of the rule the map answers.
    let s = world::step(&w, &mut state, &mut rng, D, Dir::South, &Allowances { wade: true, ..Allowances::default() });
    assert!(!s.moved, "the middle of the lake is still the middle of the lake");
    assert_eq!(state.at, [7, 9]);
}

/// A repair must not walk a wading player home, and must not leave a
/// non-wading one standing in a lake.
///
/// Two directions and they are not the same rule: what counts as *standing
/// somewhere* reads the allowances, and where a repair *puts* you ignores them
/// — a rim tile is only a place to stand while the set is on the board, so a
/// repair that used one would be a repair you could unpack your way out of.
#[test]
fn a_repair_reads_the_allowance_going_in_and_ignores_it_coming_out() {
    let w = data::world(D);
    let mut wet = WorldState { at: [7, 9], last_town: String::new(), ..WorldState::at_start(&w) };
    assert_eq!(w.repair(&mut wet, &Allowances { wade: true, ..Allowances::default() }), None, "they are allowed to be there");
    assert_eq!(wet.at, [7, 9]);

    let mut same = WorldState { at: [7, 9], last_town: String::new(), ..WorldState::at_start(&w) };
    assert!(w.repair(&mut same, &Allowances::default()).is_some(), "and not allowed without it");
    assert_ne!(same.at, [7, 9]);
    assert!(w.passable(same.at[0], same.at[1]), "a repair lands on ground, never on the rim");
}

/// Wading only ever *adds*: the map every other test asserts about is unchanged.
#[test]
fn an_allowance_never_shuts_anything() {
    let w = data::world(D);
    let allowed = Allowances { wade: true, ..Allowances::default() };
    for y in 0..w.height {
        for x in 0..w.width {
            if w.passable(x, y) {
                assert!(w.walkable(x, y, &allowed), "({x}, {y}) was ground and stopped being it");
            }
        }
    }
}

// ------------------------------------------------------------- and the fight

/// A rule the tree grants still reaches the fight, through the same door.
///
/// The regression half of moving `Rule` out of `skills.rs`: the module changed
/// and nothing else was allowed to.
#[test]
fn a_rule_from_the_tree_still_reaches_the_fight() {
    let mut g = Game::default();
    let tree = data::skills();
    // Whichever node grants the scouting rule; found rather than named, so the
    // test is about the door and not about one node's id.
    let node = tree
        .trees
        .iter()
        .flat_map(|t| &t.nodes)
        .find(|n| {
            n.effects.iter().any(|e| {
                matches!(e, gm2d_core::skills::Effect::Grants { rule: Rule::Scout })
            })
        })
        .expect("something in the tree grants scouting");
    assert!(!g.character.scouting());
    g.character.skills_taken.push(node.id.clone());
    assert!(g.character.scouting(), "the tree's half of Character::rules");
    assert!(g.character.start_with().rules.contains(&Rule::Scout));
}
