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
        ch.equip(id, SlotKind::Instrument, 0, y)?;
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
        for k in SlotKind::EVERY {
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
            ch.equip(id, SlotKind::Instrument, x, y)
                .unwrap_or_else(|e| panic!("{want}: {n} at ({x}, {y}): {e}"));
        }
        let report = ch.report(SlotKind::Instrument);
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

/// **An instrument has a frame of its own, and gear cannot go in it.**
///
/// This replaces `a_grid_holds_gear_or_an_instrument_and_says_which`, which
/// asserted the trade M13 removes: an instrument used to be built in the weapon
/// grid, and `RuleError::MixedGrid` refused a blade beside a shard. That was
/// `PLAN-M11.md` §8 row 4 — *surveying costs your sword arm* — and it was
/// wrong, because what is through the Reach is a map you have to fight on.
/// Reported as *"it makes any fight you would reach on the other side
/// impossible"*.
///
/// A test that pins the behaviour a change removes is a test to delete in the
/// commit that removes it, with the reason in the message. This is the reason.
#[test]
fn an_instrument_has_a_frame_of_its_own() {
    let mut ch = Character::with_all_pieces();
    for k in SlotKind::EVERY {
        ch.loadout.slot_mut(k).clear();
    }

    // A blade in the weapon grid and a shard in the instrument frame, at the
    // same time — which is the whole point, and was refused before.
    for n in ["Oak Handle", "Iron Blade", "Iron Blade", "Map Shard"] {
        ch.give(n);
    }
    let handle = common::spare(&ch, "Oak Handle");
    ch.equip(handle, SlotKind::Weapon, 0, 0).expect("a handle goes in the weapon grid");
    let blade = common::spare(&ch, "Iron Blade");
    ch.equip(blade, SlotKind::Weapon, 1, 0).expect("a blade goes beside it");
    let shard = common::spare(&ch, "Map Shard");
    ch.equip(shard, SlotKind::Instrument, 0, 0)
        .expect("a shard goes in the instrument frame while a weapon is built");

    // Gear is refused there, and by the slot rather than by a special case.
    let spare_blade = common::spare(&ch, "Iron Blade");
    let why = ch
        .can_equip(spare_blade, SlotKind::Instrument, 3, 0)
        .expect_err("a blade went into the instrument frame");
    assert!(
        matches!(why, RuleError::Place(gm2d_core::slot::PlaceError::WrongSlot)),
        "{why:?}",
    );

    // And a shard is refused in the weapon grid, the same way.
    let spare_shard = common::spare(&ch, "Map Shard");
    assert!(
        ch.can_equip(spare_shard, SlotKind::Weapon, 3, 2).is_err(),
        "a shard went into the weapon grid",
    );
}

/// The instrument frame is three rows and nothing grows it.
///
/// **That is what "one instrument" is made of.** A golem is three shards and
/// two of the ground — twelve cells — so one fits and the frame is the cost
/// that replaced the sword arm. `resize_boards` walks `ALL`, and no node or
/// errand names this frame, so there is no path that makes it bigger.
#[test]
fn the_instrument_frame_never_grows() {
    let mut ch = Character::starting();
    assert_eq!(
        ch.loadout.slot(SlotKind::Instrument).rows(),
        gm2d_core::progression::STARTING_ROWS,
        "the instrument frame did not start at the base height",
    );
    // Every row any grid can be granted, handed out at once.
    ch.resize_boards([8, 8, 8, 8, 8]);
    assert_eq!(
        ch.loadout.slot(SlotKind::Instrument).rows(),
        gm2d_core::progression::STARTING_ROWS,
        "something grew the instrument frame",
    );
}

/// Two instruments on the frame read as one, and it is the first.
///
/// Geometry cannot enforce "one at a time": the frame has to hold a golem at
/// twelve cells, and two compasses are ten. So the rule is stated instead —
/// `Character::instrument` answers with one — and the screen at the Reach
/// prints which, because a player carrying two must not be left guessing.
#[test]
fn two_instruments_read_as_one() {
    let mut ch = Character::with_all_pieces();
    for k in SlotKind::EVERY {
        ch.loadout.slot_mut(k).clear();
    }
    for n in ["Map Shard", "Glass Lens", "Magnet"] {
        ch.give(n);
    }
    // Two compasses, not touching, so they are two items rather than one.
    // Columns 0-1 and 3-4, so column 2 keeps them apart: touching would make
    // them one group of six pieces, which satisfies no recipe at all.
    for (n, x, y) in [
        ("Map Shard", 0, 0), ("Glass Lens", 0, 1), ("Magnet", 1, 1),
        ("Map Shard", 3, 0), ("Glass Lens", 3, 1), ("Magnet", 4, 1),
    ] {
        let id = common::spare(&ch, n);
        ch.equip(id, SlotKind::Instrument, x, y).unwrap_or_else(|e| panic!("{n} at ({x},{y}): {e}"));
    }
    let made = ch
        .report(SlotKind::Instrument)
        .items
        .iter()
        .filter(|i| i.assembled)
        .count();
    assert_eq!(made, 2, "the fixture did not build two instruments");
    assert_eq!(ch.instrument(), Some("compass"), "two instruments did not read as one");
    assert_eq!(
        ch.rules(),
        vec![Rule::Survey { kind: "compass".into() }],
        "two instruments granted two readings",
    );
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
        // **Where it goes**, which since M13 is a frame of its own rather than
        // the weapon grid. The old assertion read "weapon grid" and was about
        // the cost that made the far side of the Reach unwinnable.
        assert!(
            survey.iter().any(|l| l.contains("instrument frame")),
            "{name} does not say where it goes: {survey:?}"
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
