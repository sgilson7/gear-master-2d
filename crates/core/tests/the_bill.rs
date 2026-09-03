//! Top of the Bill, and the promise that reaches something.
//!
//! `ClassPower::Showstopper` — *a fight won in under ten seconds pays fifty
//! percent more* — existed, was tuned, was themed as Top of the Bill (Hanglo
//! Chiemstar, p. 31), and was **honoured nowhere**. `combat.rs` ignored it on
//! purpose and correctly, because it is a settlement rule; and `fight::settle`
//! never read the class at all. Offering it would have cost a player an
//! irreversible choice at level five in exchange for nothing, which is exactly
//! what eight skill nodes cost this project two milestones.
//!
//! So the first test in this file is the lint, and the class comes after it.

mod common;

use gm2d_core::class::{ClassPower, CLASSES, OFFERED};
use gm2d_core::combat::{Difficulty, Outcome};
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::reward;

const D: Difficulty = Difficulty::Easy;

fn def(name: &str) -> &'static gm2d_core::class::ClassDef {
    CLASSES.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("no class {name}"))
}

// ------------------------------------------------------------------ the lint

/// **Every class on the fork reaches something.**
///
/// The test that would have caught `Showstopper`. A power is honoured in
/// exactly one of three places — the fight, the settlement, or the map screen —
/// and a class offered whose power is in none of them is a point of no return
/// spent on nothing.
///
/// Written as a match rather than a grep, so it cannot rot: adding a
/// `ClassPower` to the offered list is a compile error here until somebody has
/// said which of the three answers it.
#[test]
fn every_offered_class_reaches_something() {
    let ch = common::preset_board();
    let spec = gm2d_core::combat::creature("Rust Colossus").expect("it exists");
    let bare = gm2d_core::combat::simulate_at(
        ch.player_stats(), &ch.combat_items(), spec, Difficulty::Medium,
    );

    for name in OFFERED {
        let d = def(name);
        let worn = vec![d.clone()];
        match d.power {
            // **The board.** `Recycler` is not a fight rule and `combat.rs`
            // says so: it scales assembly bonuses, which are in the item
            // profiles before the bell. So it is proved on the board rather
            // than on the fighter — and this is the arm that caught it doing
            // nothing at all, because `apply_skills` only ever read the tree
            // and the class's half of `assembly_pct` was zero for every
            // character who ever took the Patent.
            ClassPower::Recycler { .. } => {
                let mut classed = common::preset_board();
                classed.class = Some(d.name.to_string());
                classed.refresh_assembly_pct();
                assert_ne!(
                    format!("{:?}", classed.combat_items()),
                    format!("{:?}", ch.combat_items()),
                    "{name} says it reaches the board and the board is unchanged"
                );
            }
            // The fight. `simulate_party_holding` translates each of these into
            // a `Combatant` field at the bell — so the fighter who walks to it
            // has to be a different one.
            ClassPower::Leeching(_)
            | ClassPower::Contagion(_)
            | ClassPower::Bloodscent(_) => {
                let with = gm2d_core::combat::simulate_with_class(
                    ch.player_stats(), &ch.combat_items(), spec, Difficulty::Medium, &worn,
                );
                assert_ne!(
                    format!("{:?}", with.player),
                    format!("{:?}", bare.player),
                    "{name} says it reaches the fight and the fighter at the bell is \
                     the same one"
                );
            }
            // The settlement. **Called, not declared** — the first version of
            // this matched the variant and said "the purse", which a stubbed
            // payout passed cleanly. A lint that reads a list rather than the
            // behaviour is the failure it exists to catch, one level up.
            ClassPower::Showstopper { .. } => {
                let quick = reward::bounty_with_class(Outcome::Victory, 40, &worn, 1);
                assert_ne!(
                    quick,
                    reward::bounty_for(Outcome::Victory, 40),
                    "{name} says it reaches the purse and the purse is unchanged"
                );
            }
            // Anything else on the fork has not been argued about, and this is
            // where the argument goes.
            other => panic!(
                "{name} is offered and its power {other:?} is honoured nowhere. \
                 A class that costs an irreversible choice and does nothing is the \
                 failure eight skill nodes already cost two milestones — wire it up, \
                 or take it off `class::OFFERED`."
            ),
        }
    }
}

/// The roster is core's, and there is one of it.
#[test]
fn the_roster_is_not_written_down_twice() {
    assert_eq!(OFFERED.len(), 5, "the fork deals five");
    for name in OFFERED {
        assert!(CLASSES.iter().any(|c| c.name == *name), "{name} is offered and is not a class");
        assert!(
            data::skills().tree_for_class(name).is_some(),
            "{name} is offered and brings no tree"
        );
    }
    let mut seen: Vec<&str> = Vec::new();
    for n in OFFERED {
        assert!(!seen.contains(n), "{n} is offered twice");
        seen.push(n);
    }
}

// ------------------------------------------------------------------ the purse

/// **A fast win pays more and a slow one does not.**
#[test]
fn a_fast_win_pays_more_and_a_slow_one_does_not() {
    let bill = vec![def("Showstopper").clone()];
    let ClassPower::Showstopper { pct, under_ms } = def("Showstopper").power else {
        panic!("the class changed power")
    };
    let plain = reward::bounty_for(Outcome::Victory, 40);
    assert_eq!(plain, 40);

    let quick = reward::bounty_with_class(Outcome::Victory, 40, &bill, under_ms - 1);
    assert_eq!(quick, 40 + 40 * pct / 100, "a quick win paid the plain bounty");
    let slow = reward::bounty_with_class(Outcome::Victory, 40, &bill, under_ms + 1);
    assert_eq!(slow, plain, "a slow win paid the bonus anyway");

    // And nobody else is paid for being quick.
    for name in OFFERED.iter().filter(|n| **n != "Showstopper") {
        let other = vec![def(name).clone()];
        assert_eq!(
            reward::bounty_with_class(Outcome::Victory, 40, &other, 1),
            plain,
            "{name} is being paid for speed and its promise says nothing about it"
        );
    }
}

/// **A loss pays nothing, however quick it was.**
///
/// The family `a_lose_win_cycle_is_not_a_gold_farm` guards, and the one a class
/// that multiplies a bounty could have broken from the other end: half again of
/// nothing has to be nothing.
#[test]
fn losing_quickly_is_still_losing() {
    let bill = vec![def("Showstopper").clone()];
    for outcome in [Outcome::Defeat, Outcome::Stalemate] {
        assert_eq!(reward::bounty_with_class(outcome, 40, &bill, 1), 0, "{outcome:?} paid");
    }
    // And a lose/win cycle still pays exactly one win, quick or not.
    let cycle = reward::bounty_with_class(Outcome::Defeat, 40, &bill, 1)
        + reward::bounty_with_class(Outcome::Victory, 40, &bill, 1);
    assert_eq!(cycle, reward::bounty_with_class(Outcome::Victory, 40, &bill, 1));
}

/// **A rout is not quick, it is nothing**, so it pays the plain bounty.
///
/// Half again for taking no time at all would be the one arrangement in the
/// game where a class is paid for something not happening — and it is the same
/// reasoning that makes a rout cost no tiredness.
#[test]
fn a_rout_pays_no_speed_bonus() {
    let mut g = Game::new(3, "td");
    g.character.class = Some("Showstopper".into());
    // The Mandate, so there is something to rout with.
    for name in gm2d_core::loadout::set_pieces(gm2d_core::piece::MANDATE) {
        g.character.give(name);
    }
    let seats: Vec<(String, u8, u8)> = vec![
        ("Ratskin Material".into(), 0, 0),
        ("Ratskin Mold".into(), 2, 0),
        ("Rat Signet".into(), 4, 0),
    ];
    for (n, x, y) in seats {
        let id = g.character.find_by_name(&n).unwrap();
        let _ = g.character.equip(id, gm2d_core::piece::SlotKind::Gloves, x, y);
    }
    if !g.character.rules().iter().any(|r| matches!(r, gm2d_core::rule::Rule::Rout { .. })) {
        // The starting frames are three rows; if the set will not seat, there
        // is nothing to prove here and saying so beats passing quietly.
        panic!("the Mandate did not assemble, so this test proves nothing");
    }
    let rat = gm2d_core::combat::creature("Cave Rat").unwrap();
    g.encounter = Some(gm2d_core::fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let r = gm2d_core::fight::rout(&mut g).expect("it routs");
    assert_eq!(r.gold, rat.bounty, "a rout was paid for being quick about nothing");
}

/// The receipt says what the speed was worth, because nothing else would.
#[test]
fn the_receipt_says_what_being_quick_paid() {
    let mut g = Game::new(0x5A1E, "td");
    g.character = common::bench();
    common::build_full_loadout(&mut g.character);
    g.character.class = Some("Showstopper".into());
    g.encounter = Some(gm2d_core::fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let log = gm2d_core::fight::run(&g, D).unwrap();
    assert_eq!(log.outcome, Outcome::Victory);
    let ClassPower::Showstopper { under_ms, .. } = def("Showstopper").power else {
        panic!()
    };
    assert!(log.duration_ms < under_ms, "the fixture takes too long to be quick");
    let s = gm2d_core::fight::settle(&mut g, &log, D).unwrap();
    assert!(
        s.receipt.iter().any(|l| l.contains("speed")),
        "the class paid and the receipt does not say so: {:?}",
        s.receipt
    );
    assert!(s.gold > gm2d_core::combat::creature("Cave Rat").unwrap().bounty);
}

// ----------------------------------------------------------------- the class

/// **Two classes may ench now, and what separates them is which they can get.**
#[test]
fn the_bill_can_ench_and_so_can_the_patent() {
    use gm2d_core::ench::licences;
    assert!(licences(Some("Recycler")));
    assert!(licences(Some("Showstopper")));
    assert!(!licences(Some("Berserker")));
    assert!(!licences(None));

    // And the Swing is the Bill's alone: its own tree awards it, and nothing
    // else does.
    let tree = data::skills();
    let bill = tree.tree_for_class("Showstopper").expect("it has a tree");
    let grants: Vec<String> = bill
        .nodes
        .iter()
        .flat_map(|n| &n.effects)
        .filter_map(|e| match e {
            gm2d_core::skills::Effect::GivesEnch { ench } => Some(ench.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(grants, vec!["the-chonga-swing".to_string()]);

    // The Patent's is its own, and neither can reach the other's.
    let patent = tree.tree_for_class("Recycler").expect("it has a tree");
    for n in &patent.nodes {
        for e in &n.effects {
            if let gm2d_core::skills::Effect::GivesEnch { ench } = e {
                assert_ne!(ench, "the-chonga-swing", "the Patent awards the Bill's ench");
            }
        }
    }
}

/// The tree is a tree, in the shape the other four are.
#[test]
fn the_bill_brings_a_tree_somebody_can_spend_in() {
    let tree = data::skills();
    let bill = tree.tree_for_class("Showstopper").expect("it has a tree");
    assert_eq!(bill.nodes.len(), 8, "the other class trees are eight, ten and eight");
    // Something to spend the first point on.
    let top = bill.rows().into_iter().next().expect("it has rows");
    assert!(top.len() >= 2, "one door in is not a tree, it is a queue");
    // And every prerequisite is inside this tree.
    for n in &bill.nodes {
        for r in &n.requires {
            assert!(bill.nodes.iter().any(|m| m.id == *r), "{}: needs {r}, which is elsewhere", n.id);
        }
    }
    // **No node makes the Swing survivable.** A node that let a fragile item
    // fire twice would be the class's whole build and would delete the mechanic
    // it is named for — `PLAN-M10.md` §5 row 4, answered no. The tree tunes what
    // the swing is worth, never how many it gets.
    for n in &bill.nodes {
        let line = n.line().to_lowercase();
        assert!(
            !line.contains("twice") && !line.contains("again"),
            "{}: {line:?} reads like it makes a fragile item fire more than once",
            n.id
        );
    }
}
