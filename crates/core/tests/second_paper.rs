//! M13.0 — the countable fact, and three classes on one character.
//!
//! A class is **finished** when every node of its tree is taken. That fact is
//! what Spike's second paper is gated on, and it is the whole reason this file
//! exists before any of the ten experts do: a gate nobody can measure is a
//! gate that has to be argued about rather than tested.

use gm2d_core::character::Character;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::progression;
use gm2d_core::save;

fn at_level(n: u32) -> Game {
    let mut g = Game::new(0x5EED_1234_ABCD_0013, "td");
    g.character.gain_xp(progression::xp_to_reach(n));
    g.character.resize_boards([0; 5]);
    g.character.apply_preset();
    g
}

/// Every node of the named class's tree, taken, for free.
///
/// It writes `skills_taken` directly rather than spending points, because what
/// is under test is the *reading* of a finished tree and not the ledger that
/// pays for it. `a_finished_tree_is_every_node` is where the ledger is asked.
fn finish(ch: &mut Character, class: &str) {
    let tree = data::skills();
    let t = tree.tree_for_class(class).expect("that class has a tree");
    for n in &t.nodes {
        ch.skills_taken.push(n.id.clone());
    }
}

// ------------------------------------------------------------ finished

/// **Every node, not most of them.** Nine of ten does not unlock a thing.
#[test]
fn a_finished_tree_is_every_node() {
    let tree = data::skills();
    for class in gm2d_core::class::OFFERED {
        let t = tree.tree_for_class(class).expect("every offered class has a tree");
        // One short is not finished, whichever one is missing.
        for skip in 0..t.nodes.len() {
            let taken: Vec<String> =
                t.nodes.iter().enumerate().filter(|(i, _)| *i != skip).map(|(_, n)| n.id.clone()).collect();
            assert!(
                !tree.tree_finished(class, &taken),
                "{class} read as finished with {} of {} taken",
                taken.len(),
                t.nodes.len()
            );
        }
        let all: Vec<String> = t.nodes.iter().map(|n| n.id.clone()).collect();
        assert!(tree.tree_finished(class, &all), "{class} is not finished by all of it");
    }
}

/// The base tree is nobody's class, so finishing it finishes no class.
#[test]
fn the_base_tree_is_nobodys_class() {
    let tree = data::skills();
    let base = tree.base().expect("there is a base tree");
    let all: Vec<String> = base.nodes.iter().map(|n| n.id.clone()).collect();
    for class in gm2d_core::class::OFFERED {
        assert!(
            !tree.tree_finished(class, &all),
            "{class} was finished by walking the base tree"
        );
    }
}

/// A class with no tree is never finished — otherwise it would be finished by
/// having no work in it, and hand over the paper the moment it was taken.
#[test]
fn a_class_with_no_tree_is_never_finished() {
    let tree = data::skills();
    assert!(tree.tree_for_class("Wanderer").is_none(), "the fixture needs a treeless class");
    assert!(!tree.tree_finished("Wanderer", &[]));
    assert!(!tree.tree_finished("Wanderer", &["anything".to_string()]));
}

/// The refusal can count: six of the eight, and he can say so.
#[test]
fn the_progress_is_a_pair_the_refusal_can_read() {
    let tree = data::skills();
    let t = tree.tree_for_class("Berserker").unwrap();
    let some: Vec<String> = t.nodes.iter().take(3).map(|n| n.id.clone()).collect();
    assert_eq!(tree.tree_progress("Berserker", &some), (3, t.nodes.len()));
    assert_eq!(tree.tree_progress("Berserker", &[]), (0, t.nodes.len()));
}

// ------------------------------------------------------- three classes

#[test]
fn a_character_holds_none_one_two_or_three() {
    let mut g = at_level(6);
    assert_eq!(g.character.classes().count(), 0);
    g.character.choose_class("Berserker").unwrap();
    assert_eq!(g.character.classes().collect::<Vec<_>>(), vec!["Berserker"]);
    g.character.second_paper = true;
    g.character.choose_second_class("Hexweaver").unwrap();
    assert_eq!(g.character.classes().collect::<Vec<_>>(), vec!["Berserker", "Hexweaver"]);
    g.character.expert = Some("Recycler".into());
    assert_eq!(g.character.classes().count(), 3);
    assert_eq!(g.character.class_defs().len(), 3);
}

/// A class named in a file this build has not got is skipped, not fatal.
#[test]
fn a_class_this_build_has_not_got_is_skipped() {
    let mut g = at_level(6);
    g.character.class = Some("Berserker".into());
    g.character.second_class = Some("Something From Later".into());
    assert_eq!(g.character.classes().count(), 2, "the save says two");
    assert_eq!(g.character.class_defs().len(), 1, "and one of them resolves");
}

#[test]
fn the_second_class_is_permanent_and_wants_a_paper() {
    let mut g = at_level(6);
    g.character.choose_class("Berserker").unwrap();
    // No paper, no class.
    assert!(g.character.choose_second_class("Hexweaver").is_err());
    assert!(!g.character.owed_a_second_class());
    g.character.second_paper = true;
    assert!(g.character.owed_a_second_class());
    // Not the one you already are.
    assert!(g.character.choose_second_class("Berserker").is_err());
    assert!(g.character.second_paper, "a refusal spends nothing");
    // And not a class that does not exist.
    assert!(g.character.choose_second_class("Nobody").is_err());
    assert!(g.character.second_paper, "a refusal spends nothing");

    g.character.choose_second_class("Hexweaver").unwrap();
    assert!(!g.character.second_paper, "the paper is spent on the choice");
    assert!(!g.character.owed_a_second_class());
    // And it does not come off.
    assert!(g.character.choose_second_class("Bloodletter").is_err());
    assert_eq!(g.character.second_class.as_deref(), Some("Hexweaver"));
}

#[test]
fn finished_trees_counts_only_the_classes_you_are() {
    let mut g = at_level(6);
    g.character.choose_class("Berserker").unwrap();
    assert_eq!(g.character.finished_trees(), 0);
    finish(&mut g.character, "Hexweaver");
    assert_eq!(g.character.finished_trees(), 0, "somebody else's tree is not yours");
    finish(&mut g.character, "Berserker");
    assert_eq!(g.character.finished_trees(), 1);
    g.character.second_paper = true;
    g.character.choose_second_class("Hexweaver").unwrap();
    assert_eq!(g.character.finished_trees(), 2, "and now the nodes already taken count");
}

// ------------------------------------------------------------------ the save

#[test]
fn the_three_fields_round_trip() {
    let mut g = at_level(7);
    g.character.choose_class("Berserker").unwrap();
    g.character.second_paper = true;
    g.character.choose_second_class("Hexweaver").unwrap();
    g.character.expert = Some("Recycler".into());
    // **The expert is written raw here and the re-derivation is asked for by
    // hand**, because `take_expert` does not exist until M13.4. It is not
    // decoration: `Loadout::assembly_pct` is banked, the loader re-derives it,
    // and a class field written without this comes back from a round trip with
    // a *different* number than it went in with — which is what this test
    // caught on its first run. Every writer of a class field owes the
    // re-derivation; `choose_class` and `choose_second_class` both pay it.
    g.character.refresh_assembly_pct();
    let text = save::save(&g);
    let back = save::load(&text).expect("it loads");
    assert_eq!(back, g, "the round trip is the operator");
    assert_eq!(back.character.second_class.as_deref(), Some("Hexweaver"));
    assert_eq!(back.character.expert.as_deref(), Some("Recycler"));
}

/// An unspent paper survives a reload, which is what makes it a thing you may
/// sleep on rather than a screen you had to answer.
#[test]
fn an_unspent_paper_survives_a_reload() {
    let mut g = at_level(7);
    g.character.choose_class("Berserker").unwrap();
    g.character.second_paper = true;
    let back = save::load(&save::save(&g)).expect("it loads");
    assert!(back.character.second_paper);
    assert!(back.character.owed_a_second_class());
    assert_eq!(back, g);
}

/// A file written before M13 opens with none of them — which is what those
/// characters had. No seam: nothing here moved the catalogue.
#[test]
fn a_save_from_before_this_block_opens() {
    let mut g = at_level(7);
    g.character.choose_class("Berserker").unwrap();
    let text = save::save(&g);
    assert!(
        !text.contains("second_class") && !text.contains("second_paper") && !text.contains("\"expert\""),
        "an unset field should not be written at all"
    );
    let back = save::load(&text).expect("it loads");
    assert_eq!(back.character.second_class, None);
    assert_eq!(back.character.expert, None);
    assert!(!back.character.second_paper);
}

// ------------------------------------------------- M13.9: derived, never banked

/// **Every door that sets a class re-derives the bonus.**
///
/// `Loadout::assembly_pct` is the last banked derived number in the game. It is
/// on the loadout because `report` is called from a hundred and eight places
/// and every one of them — the sheet, each item card, the shop's comparison,
/// the fight — has to see the same figure; a parameter through all of them is a
/// parameter somebody forgets in one place, and the bug that makes is an item
/// card that disagrees with the fight.
///
/// What it must not be is *stale*. Since M13.9 the save no longer carries it,
/// so a file cannot arrive with an old one — and what is left is four mutators
/// that have to remember `refresh_assembly_pct`. `SECOND-ORDER-M13.md` row 1
/// asks for a guard that a class field cannot be set without it. This is that
/// guard: it walks every door in the game that sets one, and compares the field
/// against `Character::assembly_pct_of`, which is the one place the sum is
/// done.
///
/// **Called, not listed.** A test naming the three mutators it knows about is a
/// test that goes quiet the moment a fourth is added — so the Kaklon Licensee
/// is on both sides of every pairing here, which is the class whose whole power
/// is this number.
#[test]
fn a_class_taken_any_way_re_derives_the_bonus() {
    let tree = data::skills();
    let agrees = |c: &Character, door: &str| {
        assert_eq!(
            c.loadout.assembly_pct,
            c.assembly_pct_of(&tree),
            "{door} left the assembly bonus at {} where the tree and the classes say {}",
            c.loadout.assembly_pct,
            c.assembly_pct_of(&tree),
        );
    };

    // **The Kaklon Licensee on both sides of the pairing, and the run is done
    // twice for that reason alone.** It is the only class whose power moves
    // this number, so a door it is not standing at is a door that could forget
    // the re-derivation and change nothing — the check would pass on a game
    // that was broken. Taking it first exercises `choose_class`; taking it
    // second exercises `choose_second_class`.
    for (first, second) in [("Recycler", "Showstopper"), ("Showstopper", "Recycler")] {
        // Level twenty, because the fork refuses below five — what is under
        // test is the re-derivation and not the ledger that gates it.
        let mut c = at_level(20).character;
        let before = c.loadout.assembly_pct;
        c.choose_class(first).expect("the fork");
        agrees(&c, "choose_class");
        if first == "Recycler" {
            assert!(c.loadout.assembly_pct > before, "the Licensee's own number is nothing");
        }

        // The second paper.
        let mid = c.loadout.assembly_pct;
        c.second_paper = true;
        c.choose_second_class(second).expect("the paper");
        agrees(&c, "choose_second_class");
        if second == "Recycler" {
            assert!(c.loadout.assembly_pct > mid, "a second Licensee added nothing");
        }

        // A point spent, which is the tree's half of the same number.
        c.skill_points = 60;
        for class in [first, second] {
            for n in tree.tree_for_class(class).expect("a tree").nodes.iter() {
                let _ = c.take_skill(&tree, &n.id);
            }
        }
        agrees(&c, "take_skill");

        // The expert, which is a third class. **It cannot move this number and
        // is walked anyway**: `expert_nodes_touch_only_the_expert` forbids a
        // bare `assembly_pct` in an expert tree and no `ExpertPower` is a
        // `Recycler`, so what this proves is that taking one does not *disturb*
        // it. The day an expert does move it, this line starts asking a
        // question rather than confirming an absence.
        c.take_expert().expect("two finished trees");
        agrees(&c, "take_expert");

        // And the way in from a file, which is the door that used to carry a
        // stale figure rather than forget to write a fresh one.
        let mut g = at_level(20);
        g.character = c.clone();
        let back = save::load(&save::save(&g)).expect("it loads");
        agrees(&back.character, "the loader");
        assert_eq!(back.character.loadout.assembly_pct, c.loadout.assembly_pct);
    }
}

/// **The file does not carry it.** A number that is stored and then thrown away
/// on the way in is a number somebody will one day believe.
#[test]
fn the_assembly_bonus_is_not_in_the_file() {
    let mut g = at_level(20);
    g.character.choose_class("Recycler").expect("the fork");
    assert!(g.character.loadout.assembly_pct > 0);
    let json = save::save(&g);
    assert!(
        !json.contains("assembly_pct"),
        "the save writes a figure it derives on the way in"
    );
    // And a file that *does* carry one — every file written before M13.9 —
    // opens with the derived figure rather than the written one.
    let doctored = json.replace(
        "\"name_seed\"",
        "\"assembly_pct\":9999,\"name_seed\"",
    );
    let back = save::load(&doctored).expect("an older file opens");
    assert_eq!(
        back.character.loadout.assembly_pct,
        g.character.loadout.assembly_pct,
        "a stale figure in an older file was believed"
    );
}
