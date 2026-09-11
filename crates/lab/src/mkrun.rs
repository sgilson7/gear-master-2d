//! Write the block's fixture save: **the run**, from `PLAN-M16.md` §5.1.
//!
//! **This exists because the file did not arrive.** `PROMPT-M16.md` says to
//! place `testing/saves/the-run-20260910.json` in the repository before
//! starting, and to treat it as the human's own file — *do not edit it, do not
//! re-save it through the game.* It was not there. What **was** there is
//! `PLAN-M16.md` §5.1, which transcribes the board in full: thirty-eight
//! placements with slot, cell and rotation, and six enchs by gear index.
//!
//! So the fixture is **reconstructed from the transcription** rather than
//! uploaded, and this is the reconstruction, checked in so that the file is a
//! thing somebody can regenerate and diff rather than a blob nobody can
//! account for. The block's own tests are what make it honest:
//! `the_save_opens_at_forty_five` asserts the level, the seated count and the
//! ench count, and `the_tenth_surveyor_wears_the_run` compares the boss's
//! items against this character's, by name and count per slot — so if the
//! transcription and the reconstruction disagree, two tests say so.
//!
//! It is **not** part of the game: `crates/wasm` does not depend on this crate.
//!
//!     cargo run -q -p gm2d-lab --bin mkrun -- testing/saves/the-run-20260910.json

use std::env;

use gm2d_core::character::Character;
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;
use gm2d_core::save::SaveFile;

/// The board, verbatim from `PLAN-M16.md` §5.1: name, slot, x, y, rotation.
const GEAR: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
    ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
    ("Bronze Plating", SlotKind::Helmet, 4, 0, 0),
    ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
    ("Bronze Frame", SlotKind::Helmet, 4, 1, 0),
    ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
    ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
    ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
    ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
    ("Brigandine Base", SlotKind::Chest, 4, 1, 0),
    ("Chain Layer", SlotKind::Chest, 0, 3, 0),
    ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
    ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
    ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
    ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
    ("Tin Band", SlotKind::Gloves, 3, 1, 0),
    ("Oathring", SlotKind::Gloves, 1, 2, 0),
    ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
    ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
    ("Padded Mold", SlotKind::Gloves, 4, 2, 0),
    ("Plain Sole", SlotKind::Greaves, 0, 0, 0),
    ("Spun Material", SlotKind::Greaves, 2, 0, 1),
    ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
    ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
    ("Spun Material", SlotKind::Greaves, 2, 1, 3),
    ("Sapling Mold", SlotKind::Greaves, 4, 1, 0),
    ("Herbal", SlotKind::Weapon, 0, 0, 0),
    ("Chain Coil", SlotKind::Weapon, 1, 0, 2),
    ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
    ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
    ("Quicksilver Ink", SlotKind::Weapon, 5, 0, 1),
    ("Quicksilver Ink", SlotKind::Weapon, 2, 1, 1),
    ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
    ("Emberburst", SlotKind::Weapon, 2, 2, 0),
    ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
    ("Ratchet Cog", SlotKind::Weapon, 0, 4, 1),
    ("Quicksilver Ink", SlotKind::Weapon, 2, 4, 0),
    ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
];

/// Ench id, and the index into `GEAR` of the piece it is bolted to.
const ENCHS: &[(&str, usize)] = &[
    ("the-yodregar-index", 32),
    ("the-wextreen-correction", 33),
    ("the-chonga-swing", 9),
    ("sneel-bearing", 26),
    ("plug-energy-tap", 27),
    ("grungo-elastic-band", 31),
];

/// Level 45, and the purse the plan transcribes.
const GOLD: i32 = 33_904;
const WINS: u32 = 475;

/// Replay one grid under a locking mask and count how many of its pieces end
/// up inside an item that actually assembles.
///
/// A scratch registry and a scratch loadout, so the search never touches the
/// character being built — and only this grid's pieces, so the two to the n is
/// over one board rather than over thirty-eight.
fn assembled_pieces_under(on: &[usize], mask: u32) -> usize {
    let kind = GEAR[on[0]].1;
    let mut reg = gm2d_core::piece::PieceRegistry::new();
    let mut lo = gm2d_core::loadout::Loadout::new();
    lo.slot_mut(kind).grow(8);
    for (at, &i) in on.iter().enumerate() {
        let (name, _, x, y, rot) = GEAR[i];
        let Some(def) = gm2d_core::piece::CATALOG.iter().position(|d| d.name == name) else {
            return 0;
        };
        let id = reg.alloc(def);
        reg.set_rotation(id, rot);
        if lo.can_place(&reg, id, kind, x, y).is_err() {
            return 0;
        }
        lo.slot_mut(kind).place(&reg, id, x, y);
        if mask & (1 << at) != 0 {
            gm2d_core::loadout::lock_assembled_in(&mut lo, &reg, kind);
        }
    }
    lo.report(&reg, kind)
        .items
        .iter()
        .filter(|i| i.assembled)
        .map(|i| i.pieces.len())
        .sum()
}

fn main() {
    let out = env::args().nth(1).unwrap_or_else(|| {
        "testing/saves/the-run-20260910.json".to_string()
    });

    let mut game = Game::new(0x6D_75_6E_20_34_35, "td");
    let ch = &mut game.character;
    *ch = Character::starting_seeded(0x6D_75_6E_20_34_35);
    ch.clear_all();

    // **Level forty-five is spent experience, not a field.** The level is
    // derived from `xp` and nothing else, which is the rule since M4, so the
    // way to make a level-45 character is to hand it what forty-five levels
    // cost and let `level()` say so.
    let want = gm2d_core::progression::xp_to_reach(45);
    ch.xp = want;
    ch.gold = GOLD;
    // **Every worn frame at the ceiling, and the instrument frame left where
    // it is.** `resize_boards` walks `SlotKind::ALL` and clamps at `MAX_ROWS`,
    // which is what makes this eight and not eleven — a level-45 run has bought
    // the whole row ladder and the ladder stops at the original six by eight.
    // The instrument frame is six by three for ever and is deliberately not in
    // `ALL`, so it is not touched here either.
    ch.resize_boards([5, 5, 5, 5, 5]);

    // The three classes, in the order they were paid for.
    ch.class = Some("Berserker".into());
    ch.second_class = Some("Showstopper".into());
    ch.expert = Some("ShortProgramme".into());
    ch.buy_licence();
    ch.refresh_assembly_pct();

    // **The file does not carry the order the run built in, and the order is
    // what decides which items a board makes.** `Loadout::locks` is state and
    // not geometry — the first of the three things the fork learned the
    // expensive way — so §5.1's thirty-eight placements are a picture of where
    // the pieces sit and say nothing about when each item was fixed.
    //
    // Two disciplines were tried by hand and both are wrong on this board.
    // Locking each grid once it is full is what a fresh Auto-pack does, and it
    // merged eight weapon pieces into one group that **assembled nothing** —
    // the Ninth Surveyor's first draft arriving from the fixture side. Locking
    // greedily after every placement fixed the weapon and broke the helmet and
    // the gloves, because a piece locked early cannot join the item the next
    // piece completes.
    //
    // So the order is **searched for**, per grid, over every subset of the
    // lock points: two to the twelve on the widest board, which is four
    // thousand replays of one grid and costs nothing. What it optimises is the
    // count of *assembled pieces*, because a board with a piece in no item is
    // a board with a cell doing nothing — and the winner is written down in the
    // output so the reconstruction is a thing somebody can read.
    let mut ids: Vec<Option<gm2d_core::piece::PieceId>> = vec![None; GEAR.len()];
    for kind in SlotKind::ALL {
        let on: Vec<usize> =
            (0..GEAR.len()).filter(|&i| GEAR[i].1 == kind).collect();
        let n = on.len();
        let mut best = (0usize, 0u32);
        for mask in 0..(1u32 << n) {
            let got = assembled_pieces_under(&on, mask);
            if got > best.0 {
                best = (got, mask);
            }
        }
        println!("  {kind:?}: {} of {n} pieces assembled, locks after {:b}", best.0, best.1);
        for (slot_at, &i) in on.iter().enumerate() {
            let (name, _, x, y, rot) = GEAR[i];
            let id = ch.give(name).unwrap_or_else(|| panic!("{name} is not in the catalogue"));
            for _ in 0..rot {
                ch.rotate(id).expect("a loose piece turns");
            }
            ch.equip(id, kind, x, y).unwrap_or_else(|e| {
                panic!("{name} does not seat at {kind:?} {x},{y} rot {rot}: {e:?}")
            });
            ids[i] = Some(id);
            if best.1 & (1 << slot_at) != 0 {
                gm2d_core::loadout::lock_assembled_in(&mut ch.loadout, &ch.registry, kind);
            }
        }
        gm2d_core::loadout::lock_assembled_in(&mut ch.loadout, &ch.registry, kind);
    }

    for &(ench, at) in ENCHS {
        ch.give_ench(ench);
        ch.attach_ench(ench, ids[at].expect("a seated piece"))
            .unwrap_or_else(|e| panic!("{ench} will not go on gear[{at}]: {e:?}"));
    }

    // Four hundred and seventy-five wins, which is what the bestiary counts.
    // The counter is `beat:<canonical>`; one creature is enough to make the
    // number true and the file is not a claim about which.
    game.world.counters.push(("beat:Cave Rat".into(), WINS));
    game.world.counters.push(("met:Cave Rat".into(), WINS));

    let file = SaveFile::of(&game);
    std::fs::write(&out, file.to_json()).expect("the save writes");
    let ch = &game.character;
    println!(
        "wrote {out}: level {}, {} seated, {} enchs, {} Fnorp, {} items",
        ch.level(),
        SlotKind::ALL.iter().map(|k| ch.loadout.slot(*k).pieces().len()).sum::<usize>(),
        ch.enchanted.len(),
        ch.gold,
        ch.reports().iter().map(|r| r.items.len()).sum::<usize>(),
    );
}
