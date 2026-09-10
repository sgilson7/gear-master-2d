//! M13.4 — three papers on one counter, two of them refused.
//!
//! **Visible-but-refused is the whole point.** A locked line on a shelf you can
//! read is a goal; an absent line is a secret. The refusal names what is in the
//! way and counts, because a button that greys out with no sentence reads as
//! broken — which this project has now written down four times.

use gm2d_core::character::Character;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::progression;
use gm2d_core::shop::{Paper, StockGate};

fn at_level(n: u32) -> Game {
    let mut g = Game::new(0x5EED_0000_1313_0004, "td");
    g.character.gain_xp(progression::xp_to_reach(n));
    g.character.resize_boards([0; 5]);
    g
}

fn finish(ch: &mut Character, class: &str) {
    let tree = data::skills();
    for n in &tree.tree_for_class(class).expect("a tree").nodes {
        ch.skills_taken.push(n.id.clone());
    }
}

fn line(g: &Game, p: Paper) -> Option<gm2d_core::game::PaperLine> {
    g.papers().into_iter().find(|l| l.paper == p)
}

// ------------------------------------------------------------------ the gate

#[test]
fn a_gate_counts_and_says_so() {
    let g = StockGate::TreesFinished(1);
    assert!(!g.met(40, 0), "a level-forty hoarder has mastered nothing");
    assert!(g.met(1, 1));
    let why = g.refusal(6, 0, 1);
    assert!(why.contains('1') && why.contains("finished"), "{why}");
    // With no class at all it says so rather than counting to zero of zero.
    let why = g.refusal(6, 0, 0);
    assert!(why.contains("no class yet"), "{why}");
    // And a level gate is still a level gate.
    assert!(StockGate::Level(10).met(10, 0));
    assert!(!StockGate::Level(10).met(9, 9));
}

// ---------------------------------------------------------------- the counter

#[test]
fn all_three_papers_are_on_the_counter_from_the_first_visit() {
    let g = at_level(12);
    let ids: Vec<&str> = g.papers().iter().map(|l| l.paper.id()).collect();
    assert_eq!(ids, vec!["licence", "second-paper", "expert-paper"]);
    // **One of them refused, since M15.4**, and its refusal names the count.
    // The Patent has never had a gate and the Second Paper stopped having one
    // when the human took the tree off it; what is left on that line is five
    // thousand Fnorp, which is a price rather than a gate and is why `afford`
    // is a separate field from `why`.
    assert!(line(&g, Paper::Patent).unwrap().why.is_none(), "the Patent has never had a gate");
    assert!(
        line(&g, Paper::Second).unwrap().why.is_none(),
        "the second paper is refusing, and it is meant to be gated by its price"
    );
    let why = line(&g, Paper::Expert).unwrap().why.expect("refused");
    assert!(why.contains("finished"), "{why}");
}

/// **The third line prints the expert's own promise**, so a player choosing a
/// second class can see what the pairing eventually reaches. That is the
/// pairing decision made in daylight, and this fork has no screen.
#[test]
fn the_expert_line_names_what_the_pair_reaches() {
    let mut g = at_level(12);
    let vague = line(&g, Paper::Expert).unwrap();
    assert!(vague.says.contains("whichever"), "{}", vague.says);

    g.character.choose_class("Berserker").unwrap();
    g.character.second_paper = true;
    g.character.choose_second_class("Recycler").unwrap();
    let named = line(&g, Paper::Expert).unwrap();
    let want = gm2d_core::expert::for_pair("Berserker", "Recycler").unwrap();
    assert!(named.name.contains(want.name), "{}", named.name);
    assert_eq!(named.says, want.power.describe(), "the line is not the promise");
    assert_eq!(named.price, 0, "you have already paid twice");
}

/// A paper you have answered comes off the counter — which is deliberately
/// unlike the shelf, where the gap **is** the memory of what you took.
#[test]
fn an_answered_paper_leaves_the_counter() {
    let mut g = at_level(12);
    assert!(line(&g, Paper::Second).is_some());
    g.character.second_paper = true;
    assert!(line(&g, Paper::Second).is_none(), "a paper in the pack is still on the table");
    g.character.class = Some("Berserker".into());
    g.character.second_paper = false;
    g.character.second_class = Some("Hexweaver".into());
    assert!(line(&g, Paper::Second).is_none(), "a second class was sold a second paper");
}

// ------------------------------------------------------------------ buying

/// **Five thousand Fnorp, and nothing else.**
///
/// It wanted one finished tree until M15.4 and the human took that off in their
/// own words: *"the second class should no longer be gated behind finishing the
/// first one, instead you get it whenever you can afford the 2nd paper at spike
/// kaklons van."*
///
/// So this test is the old one with its first assertion turned round: a
/// character who has spent **no points at all** buys it, and the only refusal
/// left is the money. The version that pinned the tree gate is not repaired,
/// it is inverted — a test that pins the behaviour you are deliberately
/// changing is a test to rewrite in the commit that changes it, with the reason
/// in the message.
#[test]
fn the_second_paper_wants_five_thousand_and_nothing_else() {
    let mut g = at_level(12);
    g.character.choose_class("Berserker").unwrap();
    assert_eq!(g.character.finished_trees(), 0, "this character has finished a tree already");

    // **No gate on the line at all**, with nothing finished.
    assert!(
        line(&g, Paper::Second).unwrap().why.is_none(),
        "the second paper is still refusing somebody who can afford it"
    );

    // The money is the only thing left, and a refusal spends nothing.
    g.character.gold = 4_999;
    let why = g.buy_paper(Paper::Second).unwrap_err();
    assert!(why.contains("4999"), "{why}");
    assert!(!g.character.second_paper, "a refusal spends nothing");
    assert_eq!(g.character.gold, 4_999);

    g.character.gold = 5_000;
    assert_eq!(g.buy_paper(Paper::Second).unwrap(), 5_000);
    assert_eq!(g.character.gold, 0);
    assert!(g.character.owed_a_second_class());

    // **And finishing a tree changes nothing about it**, which is the other
    // direction and the one that would catch a gate coming back by accident.
    let mut h = at_level(12);
    h.character.choose_class("Berserker").unwrap();
    finish(&mut h.character, "Berserker");
    h.character.gold = 5_000;
    assert!(line(&h, Paper::Second).unwrap().why.is_none());
    assert_eq!(h.buy_paper(Paper::Second).unwrap(), 5_000);
}

/// **The expert paper kept its gate, and that is not an oversight.**
///
/// The ask names the second paper and not this one. Two finished trees is what
/// makes the expert *free* — the twenty-four points are the price — and a free
/// paper anybody can walk up to is not a paper, it is a fourth class on the
/// fork.
#[test]
fn the_expert_paper_still_wants_two_finished_trees() {
    let mut g = at_level(12);
    g.character.choose_class("Berserker").unwrap();
    let why = line(&g, Paper::Expert).unwrap().why.expect("refused with nothing finished");
    assert!(why.contains("finished"), "{why}");
    assert_eq!(
        gm2d_core::shop::Paper::Expert.gate(),
        Some(gm2d_core::shop::StockGate::TreesFinished(2)),
        "the expert paper lost its gate with the second paper's"
    );
    assert_eq!(
        gm2d_core::shop::Paper::Second.gate(),
        None,
        "the second paper is still gated on something"
    );
}

/// **The paper is spent on the choice, not on the purchase**, so it survives
/// being slept on — which is what makes the second fork one you may leave.
#[test]
fn a_bought_paper_waits_in_the_pack() {
    let mut g = at_level(12);
    g.character.choose_class("Berserker").unwrap();
    finish(&mut g.character, "Berserker");
    g.character.gold = 5_000;
    g.buy_paper(Paper::Second).unwrap();

    // A reload does not lose it.
    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it loads");
    assert!(back.character.owed_a_second_class());
    // He does not sell a second.
    assert!(g.buy_paper(Paper::Second).is_err());

    g.character.choose_second_class("Recycler").unwrap();
    assert!(!g.character.second_paper, "the paper outlived the choice");
}

#[test]
fn the_expert_paper_is_free_and_wants_two_finished_trees() {
    let mut g = at_level(14);
    g.character.choose_class("Berserker").unwrap();
    finish(&mut g.character, "Berserker");
    g.character.gold = 5_000;
    g.buy_paper(Paper::Second).unwrap();
    g.character.choose_second_class("Recycler").unwrap();
    assert_eq!(g.character.gold, 0, "and it is free, so this stays at nothing");

    // One of two is not two.
    let why = g.buy_paper(Paper::Expert).unwrap_err();
    assert!(why.contains("finished 1 of the 2"), "{why}");

    finish(&mut g.character, "Recycler");
    assert_eq!(g.character.finished_trees(), 2);
    assert_eq!(g.buy_paper(Paper::Expert).unwrap(), 0, "free");
    assert_eq!(g.character.gold, 0);
    let want = gm2d_core::expert::for_pair("Berserker", "Recycler").unwrap();
    assert_eq!(g.character.expert.as_deref(), Some(want.name));
    // Three classes, all live.
    assert_eq!(g.character.classes().count(), 3);
    // And it does not come off.
    assert!(g.character.take_expert().is_err());
}

/// **An expert is what a pair reaches**, so one class reaches nothing.
#[test]
fn one_class_reaches_no_expert() {
    let mut g = at_level(12);
    g.character.choose_class("Berserker").unwrap();
    finish(&mut g.character, "Berserker");
    assert!(g.character.expert_on_offer().is_none());
    let why = g.character.take_expert().unwrap_err();
    assert!(why.contains("pair"), "{why}");
}

/// The expert paper cannot be taken before the second class is chosen, even
/// with a paper sitting in the pack: a paper is not a class.
#[test]
fn a_paper_in_the_pack_is_not_a_second_class() {
    let mut g = at_level(12);
    g.character.choose_class("Berserker").unwrap();
    finish(&mut g.character, "Berserker");
    g.character.gold = 5_000;
    g.buy_paper(Paper::Second).unwrap();
    assert!(g.character.expert_on_offer().is_none(), "an unspent paper is not a class");
    assert!(g.buy_paper(Paper::Expert).is_err());
}

/// Taking the expert re-derives what the board is worth, like the other two
/// forks — the round trip is what proves it, because the loader re-derives.
#[test]
fn taking_the_expert_re_derives_the_board() {
    let mut g = at_level(14);
    g.character.choose_class("Showstopper").unwrap();
    finish(&mut g.character, "Showstopper");
    g.character.gold = 5_000;
    g.buy_paper(Paper::Second).unwrap();
    // The Kaklon Licensee is a `Recycler`, which moves every assembly bonus.
    g.character.choose_second_class("Recycler").unwrap();
    finish(&mut g.character, "Recycler");
    g.buy_paper(Paper::Expert).unwrap();
    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it loads");
    assert_eq!(back, g, "a class field was written without the re-derivation");
}

/// The two prices are the same on purpose, and the third is nothing.
#[test]
fn the_prices_are_what_the_plan_says() {
    assert_eq!(Paper::Patent.price(), gm2d_core::ench::LICENCE_PRICE);
    assert_eq!(Paper::Second.price(), gm2d_core::ench::LICENCE_PRICE);
    assert_eq!(Paper::Expert.price(), 0);
    assert!(Paper::Patent.gate().is_none(), "gating the way round the fork gates the exception");
}
