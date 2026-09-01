//! The wasm boundary. A shim, and nothing else.
//!
//! Every rule lives in `gm2d-core`. This crate exists to move strings and
//! numbers across the boundary, and it must never grow a decision of its own —
//! the moment a rule is decided here it is decided somewhere the test suite
//! cannot reach in a few seconds, and there are two rulebooks.
//!
//! M1 replaces most of this with `save_json` and `load_json`. What is here now
//! is the minimum that proves the pipeline: the page loads the module, asks
//! core a question only core can answer, and prints the answer.

use wasm_bindgen::prelude::*;

/// How many components the catalogue holds.
///
/// The M0 deploy gate, and deliberately a number nothing but core knows. A
/// page that can print it has compiled the engine to wasm, loaded it, and
/// called into it.
#[wasm_bindgen]
pub fn piece_count() -> usize {
    gm2d_core::piece::CATALOG.len()
}

/// How many creatures the ladder holds.
#[wasm_bindgen]
pub fn monster_count() -> usize {
    gm2d_core::combat::LADDER.len()
}

/// The engine's own preset board, as an item list: what a real character
/// assembles, named by the naming system, rated by the rating system.
///
/// One line per assembled item, `name\trating`. A placeholder for a board
/// screen, but not a placeholder for the engine: producing this runs piece
/// placement, item assembly, the naming hash and the rating scale, so a page
/// showing it has exercised most of what M3 will draw.
#[wasm_bindgen]
pub fn preset_items() -> String {
    let mut ch = gm2d_core::character::Character::with_all_pieces();
    ch.loadout.name_seed = 0x5EED_1234_ABCD_0001;
    ch.apply_preset();
    ch.combat_items()
        .iter()
        .map(|i| format!("{}\t{}\t{}", i.name, i.rating, i.rarity().name()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The version this build was cut from, for the page's footer.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
