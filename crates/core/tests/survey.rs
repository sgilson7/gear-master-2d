//! Map shards and the three instruments — the block's first save seam.
//!
//! M11.5. Six components join the catalogue, which moves the fingerprint and
//! refuses every save written before it; that half is
//! `sets.rs::a_save_from_before_this_block_is_refused_by_name`, which is where
//! the number has always been said out loud.
//!
//! What is here is the shape of the thing: three recipes on the weapon board, a
//! grid that holds gear or an instrument and never both, and a rule that says
//! which instrument you built. **What each one *does* is M11.6's** — this
//! milestone is the object and not yet the effect, and the split is deliberate:
//! a seam is a bad thing to combine with new behaviour.

use gm2d_core::character::{Character, RuleError};
use gm2d_core::piece::{PieceKind, SlotKind, CATALOG};
use gm2d_core::rule::{Rule, INSTRUMENTS};

mod common;

/// The six, and what makes each of them reachable.
const PARTS: &[&str] =
    &["Map Shard", "Glass Lens", "Magnet", "Cosmic Orb", "Cosmic Alignment", "Living Earth"];

fn def(name: &str) -> &'static gm2d_core::piece::PieceDef {
    CATALOG.iter().find(|d| d.name == name).unwrap_or_else(|| panic!("no {name}"))
}

/// **Nothing here is for sale, and everything here comes from somewhere.**
///
/// The rule an errand reward and a set piece both follow: what an instrument is
/// worth is the walk that assembled it, so a shelf that sold one would make the
/// walk a slow way to shop. And the other direction, which is the one that
/// actually bites: a part nothing hands out is a recipe nobody can finish, and
/// nothing else in the game would say so.
#[test]
fn every_part_of_an_instrument_comes_from_somewhere_and_no_shelf() {
    use gm2d_core::data;
    let drops = data::drops();
    let quests = data::quests();
    let shops = data::shops();

    let off_a_tile: Vec<String> = data::MAPS
        .iter()
        .flat_map(|(id, _)| {
            data::map(id, gm2d_core::combat::Difficulty::Easy)
                .places
                .iter()
                .flat_map(|p| p.drops.clone())
                .collect::<Vec<_>>()
        })
        .collect();
    let paid: Vec<String> =
        quests.quests.iter().flat_map(|q| q.reward.iter().cloned()).collect();
    let rolled: Vec<&str> = drops.every_piece();

    for name in PARTS {
        assert!(
            gm2d_core::piece::is_event_only(name),
            "{name} is not EVENT_ONLY, so the ladder can deal one"
        );
        for t in &shops.towns {
            assert!(
                !t.stock.iter().any(|s| s == name),
                "{} sells {name}, which makes the instrument a slow way to shop",
                t.id
            );
        }
        let from = off_a_tile.iter().any(|d| d == name)
            || paid.iter().any(|r| r == name)
            || rolled.contains(name);
        assert!(from, "{name} is in no drop table, off no tile and paid by no errand");
    }
}

/// **The tower is the shards' faucet, and there are enough of them.**
///
/// One compass, one atlas and one golem want six shards between them. Five come
/// off the Drambus Stack's floors, one off the thing under the lake and one off
/// an errand — so all three can be built, once, by somebody who did everything.
#[test]
fn there_are_enough_shards_to_build_all_three() {
    use gm2d_core::data;
    let mut shards = 0;
    for (id, _) in data::MAPS {
        for p in data::map(id, gm2d_core::combat::Difficulty::Easy).places {
            shards += p.drops.iter().filter(|d| *d == "Map Shard").count();
        }
    }
    shards += data::quests()
        .quests
        .iter()
        .flat_map(|q| q.reward.iter())
        .filter(|r| *r == "Map Shard")
        .count();
    let wanted: usize = INSTRUMENTS
        .iter()
        .map(|k| match *k {
            "compass" => 1,
            "atlas" => 2,
            _ => 3,
        })
        .sum();
    assert_eq!(wanted, 6);
    assert!(
        shards >= wanted,
        "{shards} shards in the world and the three instruments want {wanted}"
    );
}

// ------------------------------------------------------------- the recipes

fn seat(ch: &mut Character, names: &[&str]) -> Result<(), RuleError> {
    // Down the left edge of the weapon grid, one row apart, so nothing is
    // touching until it is meant to be. Placement is what is under test here,
    // not packing.
    let mut y = 0;
    for n in names {
        let id = common::spare(ch, n);
        ch.equip(id, SlotKind::Weapon, 0, y)?;
        y += 2;
    }
    Ok(())
}

/// **Each of the three assembles, and says which it is.**
///
/// Packed touching, because an item is a group of components that touch — the
/// recipe is the second half of the question and the first half is geometry.
#[test]
fn each_instrument_assembles_and_names_itself() {
    // **Laid out by hand, not packed.** An item is a group of components that
    // touch, and a greedy first-free-cell walk puts a four-cell orb somewhere
    // that touches nothing — which is a board that assembles nothing and says
    // nothing about the recipe. These three arrangements are the smallest
    // blocks that make each instrument one group.
    let layouts: [(&str, &[(&str, u8, u8)]); 3] = [
        ("compass", &[("Map Shard", 0, 0), ("Glass Lens", 2, 0), ("Magnet", 0, 1)]),
        (
            "atlas",
            &[
                ("Map Shard", 0, 0),
                ("Map Shard", 0, 1),
                ("Glass Lens", 2, 0),
                ("Cosmic Orb", 2, 1),
                ("Cosmic Alignment", 2, 3),
            ],
        ),
        (
            "golem",
            &[
                ("Map Shard", 0, 0),
                ("Map Shard", 0, 1),
                ("Map Shard", 0, 2),
                ("Living Earth", 2, 0),
                ("Living Earth", 2, 2),
            ],
        ),
    ];

    for (want, layout) in layouts {
        let mut ch = Character::with_all_pieces();
        ch.grow_boards(20);
        for k in SlotKind::ALL {
            ch.loadout.slot_mut(k).clear();
        }
        // **`with_all_pieces` owns one of each**, and an atlas wants two shards
        // and a golem three. The extras are given rather than the fixture
        // widened: everything else in the suite depends on one-of-each.
        for (n, ..) in layout {
            ch.give(n);
        }
        for &(n, x, y) in layout {
            let id = common::spare(&ch, n);
            ch.equip(id, SlotKind::Weapon, x, y)
                .unwrap_or_else(|e| panic!("{want}: {n} at ({x}, {y}): {e}"));
        }
        let report = ch.report(SlotKind::Weapon);
        let made: Vec<_> = report.items.iter().filter(|i| i.assembled).collect();
        assert_eq!(made.len(), 1, "{want}: the grid came to {} items", made.len());
        assert_eq!(
            gm2d_core::loadout::instrument_of(&ch.registry, &made[0].pieces),
            Some(want),
            "{want}: the assembled item is not the instrument its parts spell"
        );
        // And it grants exactly one rule, which is the one that names it.
        let rules = ch.rules();
        assert_eq!(
            rules,
            vec![Rule::Survey { kind: want.into() }],
            "{want}: the board grants {rules:?}"
        );
    }
}

/// **The weapon grid holds gear or an instrument, and never both.**
///
/// `PLAN-M11.md` §8 row 4, and the reason a sixth board was refused: surveying
/// costs your sword arm. Refused in `can_equip` and not in a recipe, because a
/// recipe governs one item and a grid holds several — a compass and a blade in
/// the same grid satisfy two recipes perfectly well.
#[test]
fn a_grid_holds_gear_or_an_instrument_and_says_which() {
    let mut ch = Character::with_all_pieces();
    ch.grow_boards(20);
    for k in SlotKind::ALL {
        ch.loadout.slot_mut(k).clear();
    }
    seat(&mut ch, &["Oak Handle"]).expect("a handle goes in an empty grid");
    let shard = common::spare(&ch, "Map Shard");
    let why = ch.can_equip(shard, SlotKind::Weapon, 3, 0).expect_err("a shard went in beside gear");
    assert!(matches!(why, RuleError::MixedGrid { instrument: false }), "{why:?}");
    // TONE 12: the refusal names the thing that is in the way.
    let said = why.to_string();
    assert!(said.contains("gear"), "{said:?}");

    // And the other way round.
    let mut ch = Character::with_all_pieces();
    ch.grow_boards(20);
    for k in SlotKind::ALL {
        ch.loadout.slot_mut(k).clear();
    }
    seat(&mut ch, &["Map Shard"]).expect("a shard goes in an empty grid");
    let handle = common::spare(&ch, "Oak Handle");
    let why =
        ch.can_equip(handle, SlotKind::Weapon, 3, 0).expect_err("a handle went in beside a shard");
    assert!(matches!(why, RuleError::MixedGrid { instrument: true }), "{why:?}");
    assert!(why.to_string().contains("instrument"), "{why}");

    // Every other grid is untouched: this is the weapon's trade and nobody
    // else's.
    let mut ch = Character::with_all_pieces();
    ch.grow_boards(20);
    let frame = common::spare(&ch, "Steel Frame");
    assert!(ch.can_equip(frame, SlotKind::Helmet, 0, 0).is_ok());
}

/// A survey part is a survey part, and an orb is still an orb.
///
/// `Cosmic Orb` and `Cosmic Alignment` are deliberately the kinds a crystal
/// ball already uses, so that one of them set into a ball is a good ball. That
/// only works if `is_survey` does *not* claim them — otherwise every ball part
/// in the game would refuse to sit beside a blade.
#[test]
fn the_cosmic_pieces_are_still_crystal_ball_parts() {
    use gm2d_core::piece::is_survey;
    assert!(is_survey(PieceKind::Shard));
    assert!(is_survey(PieceKind::Lens));
    assert!(is_survey(PieceKind::Magnet));
    assert!(is_survey(PieceKind::Earth));
    assert!(!is_survey(PieceKind::Orb), "an orb is a crystal ball's core");
    assert!(!is_survey(PieceKind::Alignment));
    assert_eq!(def("Cosmic Orb").kind, PieceKind::Orb);
    assert_eq!(def("Cosmic Alignment").kind, PieceKind::Alignment);
    assert!(def("Cosmic Orb").power_bonus > 0, "an orb that scales nothing");
    assert!(def("Cosmic Alignment").power_bonus > 0);
}

/// **Every part says what it is a part of.**
///
/// The lesson the set line learned one block earlier, and the reason it is
/// derived: a player handed a Map Shard off a tower floor has a two-cell
/// component with three mind damage on it, and no way to find out that three of
/// them and two handfuls of earth make a golem.
#[test]
fn a_part_says_which_instruments_want_it() {
    for name in PARTS {
        let lines = gm2d_core::explain::piece_lines(def(name));
        let survey: Vec<&String> =
            lines.iter().filter(|(k, _)| *k == "survey").map(|(_, v)| v).collect();
        if !gm2d_core::piece::is_survey(def(name).kind) {
            // The two cosmic pieces are ball parts that an atlas happens to
            // want; they carry no survey line and that is correct.
            continue;
        }
        assert!(!survey.is_empty(), "{name} says nothing about being an instrument's part");
        assert!(
            survey.iter().any(|l| l.contains("weapon grid")),
            "{name} does not say what it costs: {survey:?}"
        );
        assert!(
            survey.len() > 1,
            "{name} names no recipe it belongs to: {survey:?}"
        );
    }
}

/// The rule refuses an instrument nobody wrote a recipe for.
#[test]
fn a_survey_rule_names_an_instrument_that_exists() {
    for k in INSTRUMENTS {
        Rule::Survey { kind: (*k).into() }.check().expect("a shipped instrument");
        let r = Rule::Survey { kind: (*k).into() };
        assert!(r.line().contains(k), "{k}: the line does not name it");
        assert!(!r.detail().is_empty(), "{k} explains nothing on hover");
    }
    assert!(Rule::Survey { kind: "sextant".into() }.check().is_err());
}
