//! Shared fixtures for the integration tests.
//!
//! Upstream's version of this file was built around `Run`, and around three
//! things GM2D does not have: share codes, a dungeon fixture, and a run mode.
//! What is left is the two jobs the surviving suite actually asks for —
//! walking a piece's actions, and seating a board — and the second is now done
//! against [`Character`] instead of a campaign.
#![allow(dead_code)] // each test binary uses a different subset

use gm2d_core::character::Character;
use gm2d_core::piece::{Action, PieceDef, PieceId, SlotKind, Trigger, CATALOG};

/// Run `f` over every action a trigger can reach.
///
/// This was a copy of `piece::walk_actions` and is now a call to it. The
/// engine's own doc had already noticed: *"The test suite has carried a copy
/// of this for a while; `rating.rs` needs the same answer, and two of them
/// would drift."* They drifted the moment a trigger variant was added — the
/// engine's walker knew about `OnEnemyActivate` and this one did not, and the
/// only reason that was caught is that the match was exhaustive.
///
/// One walker. A lint over the catalogue that misses a payload is a lint that
/// reports a clean catalogue.
pub fn actions_of(t: &Trigger, f: &mut impl FnMut(&Action)) {
    gm2d_core::piece::walk_actions(t, f)
}

/// Does any action this piece can reach satisfy `want`?
pub fn does(def: &PieceDef, want: fn(&Action) -> bool) -> bool {
    let mut hit = false;
    for t in def.triggers {
        actions_of(t, &mut |a| hit |= want(a));
    }
    hit
}

/// Does this piece carry a trigger satisfying `want`?
pub fn has(def: &PieceDef, want: fn(&Trigger) -> bool) -> bool {
    def.triggers.iter().any(want)
}

// ------------------------------------------------------------ seating a board

/// A character owning one of every component, for tests that need to arrange
/// arbitrary pieces without shopping for them.
pub fn bench() -> Character {
    Character::with_all_pieces()
}

/// Look an owned component up by name.
pub fn piece(ch: &Character, name: &str) -> PieceId {
    ch.find_by_name(name)
        .unwrap_or_else(|| panic!("no piece named {name}"))
}

/// Look up the first *unworn* component with this name.
///
/// Distinct from [`piece`] because `with_all_pieces` owns exactly one of each,
/// and a test that seats the same name twice wants to know that rather than
/// silently move the piece it already placed.
pub fn spare(ch: &Character, name: &str) -> PieceId {
    ch.owned
        .iter()
        .copied()
        .find(|&id| ch.registry.def(id).name == name && !ch.is_equipped(id))
        .unwrap_or_else(|| panic!("no unworn piece named {name}"))
}

/// Equip by name, failing loudly with the reason if the placement is illegal.
pub fn equip(ch: &mut Character, name: &str, slot: SlotKind, ax: u8, ay: u8) {
    let id = piece(ch, name);
    ch.equip(id, slot, ax, ay)
        .unwrap_or_else(|e| panic!("failed to equip {name} at ({ax}, {ay}): {e}"));
}

/// Seat a whole board from `(name, slot, x, y, rotation)` rows.
///
/// **Locks as each item completes**, which is what a player does while
/// building and what upstream's `share.rs` learned to do the expensive way: a
/// densely packed board derived in one pass at the end asks which pieces are
/// connected, and the answer on a full grid is "most of them" — nineteen
/// weapon pieces came back as one item. Anything that wants a board *without*
/// locks should seat it by hand.
pub fn seat(ch: &mut Character, rows: &[(&str, SlotKind, u8, u8, u8)]) {
    for &(name, slot, x, y, rot) in rows {
        let id = spare(ch, name);
        ch.registry.set_rotation(id, rot);
        ch.equip(id, slot, x, y)
            .unwrap_or_else(|e| panic!("failed to seat {name} at {slot:?} ({x}, {y}): {e}"));
        gm2d_core::loadout::lock_assembled_in(&mut ch.loadout, &ch.registry, slot);
    }
}

/// The preset board: a complete, legal loadout that assembles all five slots
/// and lights every assembly bonus.
///
/// Lifted verbatim from upstream's `Run::apply_preset`, which was the
/// auto-build button and the test fixture at once so the two could not drift.
/// GM2D has no auto-build button yet; when it grows one it should call this
/// rather than grow a second arrangement.
///
/// Deliberately shows off the mechanics rather than maxing the numbers: chest,
/// gloves and greaves each carry two separate finished items, the weapon's
/// Runed Edge doubles the Ruby Inlay next to it, and the Hollow Weave sits out
/// in open space where its empty-cell bonus counts.
pub const PRESET: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
    ("Crest of Vigor", SlotKind::Helmet, 3, 0, 0),
    ("Iron Plating", SlotKind::Helmet, 0, 2, 0),
    ("Visor of Focus", SlotKind::Helmet, 0, 4, 0),
    ("Padded Base", SlotKind::Chest, 0, 0, 0),
    ("Keystone Base", SlotKind::Chest, 0, 0, 0),
    ("Hollow Weave", SlotKind::Chest, 5, 2, 1),
    ("Chain Layer", SlotKind::Chest, 0, 3, 0),
    ("Woven Underlayer", SlotKind::Chest, 0, 4, 0),
    ("Hide Base", SlotKind::Chest, 3, 6, 0),
    ("Leather Material", SlotKind::Gloves, 0, 0, 0),
    ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
    ("Steel Material", SlotKind::Gloves, 0, 4, 0),
    ("Gauntlet Mold", SlotKind::Gloves, 2, 4, 0),
    ("Runed Material", SlotKind::Greaves, 0, 0, 0),
    ("Greave Mold", SlotKind::Greaves, 2, 0, 0),
    ("Boiled Leather", SlotKind::Greaves, 0, 4, 0),
    ("Runner's Mold", SlotKind::Greaves, 3, 4, 0),
    ("Balanced Grip", SlotKind::Weapon, 0, 0, 0),
    ("Runed Edge", SlotKind::Weapon, 1, 0, 0),
    ("Ruby Inlay", SlotKind::Weapon, 2, 0, 0),
    ("Balance Weight", SlotKind::Weapon, 2, 2, 0),
];

/// A character wearing [`PRESET`], seated without locks — the arrangement the
/// golden fixture was captured from.
pub fn preset_board() -> Character {
    let mut ch = Character::with_all_pieces();
    for &(name, slot, x, y, rot) in PRESET {
        let Some(id) = ch.owned.iter().copied().find(|&i| {
            ch.registry.def(i).name == name && !ch.is_equipped(i)
        }) else {
            continue;
        };
        ch.registry.set_rotation(id, rot);
        if ch.can_equip(id, slot, x, y).is_ok() {
            let _ = ch.equip(id, slot, x, y);
        }
    }
    ch
}

/// Every catalogue index, for lints that walk the whole catalogue.
pub fn all_def_indices() -> Vec<usize> {
    (0..CATALOG.len()).collect()
}

/// Upstream's name for the preset. Kept so ported tests read as they did.
pub fn build_full_loadout(ch: &mut Character) {
    ch.apply_preset();
}
