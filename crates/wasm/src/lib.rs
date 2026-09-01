//! The wasm boundary. A shim, and nothing else.
//!
//! Every rule lives in `gm2d-core`. This crate moves strings and numbers
//! across the boundary and must never grow a decision of its own — the moment
//! a rule is decided here it is decided somewhere `cargo test` cannot reach in
//! a few seconds, and then there are two rulebooks.
//!
//! The save surface is exactly two functions, as `PLAN.md` 1.3 requires:
//! [`save_json`] and [`load_json`]. Everything else here is a getter the page
//! draws with, or a setter the page needs to prove the round trip.

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use gm2d_core::game::Game;
use gm2d_core::save;

// One game, because a page is one session. `RefCell` rather than a lock:
// wasm32-unknown-unknown is single-threaded, and a mutex here would be
// ceremony around a borrow that cannot be contended.
thread_local! {
    static GAME: RefCell<Game> = RefCell::new(Game::default());
}

fn with<T>(f: impl FnOnce(&Game) -> T) -> T {
    GAME.with(|g| f(&g.borrow()))
}

fn with_mut<T>(f: impl FnOnce(&mut Game) -> T) -> T {
    GAME.with(|g| f(&mut g.borrow_mut()))
}

// ---------------------------------------------------------------- the save

/// The whole game state as a save file.
#[wasm_bindgen]
pub fn save_json() -> String {
    with(save::save)
}

/// Replace the game with the one this text describes.
///
/// The error is the sentence core produced, unchanged. The page shows it to
/// the player as-is, because core is where the reason is known and a second
/// wording here would be a second, worse explanation.
#[wasm_bindgen]
pub fn load_json(text: &str) -> Result<(), JsValue> {
    match save::load(text) {
        Ok(g) => {
            with_mut(|slot| *slot = g);
            Ok(())
        }
        Err(why) => Err(JsValue::from_str(&why)),
    }
}

/// Start over from a seed.
#[wasm_bindgen]
pub fn new_game(seed: f64) -> () {
    with_mut(|g| *g = Game::new(seed as u64, "td"));
}

// ---------------------------------------------------------------- readings

#[wasm_bindgen]
pub fn gold() -> i32 {
    with(|g| g.character.gold)
}

/// Move the purse. The number a visitor changes before downloading.
#[wasm_bindgen]
pub fn add_gold(n: i32) {
    with_mut(|g| g.character.gold = (g.character.gold + n).max(0));
}

/// Draw from the run's random stream, returning what came out.
///
/// The stream every encounter will be rolled against in M2. Exposed now
/// because "the save restores the RNG" is the half of the gate that a gold
/// counter cannot demonstrate: a save that stored the seed rather than the
/// position would restore the purse perfectly and then hand the player the
/// same next draw they had already seen.
#[wasm_bindgen]
pub fn draw() -> u32 {
    with_mut(|g| g.rng.below(1000) as u32)
}

/// Where the stream is standing, as hex. Shown so the page can display the
/// thing being preserved rather than only its consequences.
#[wasm_bindgen]
pub fn rng_state() -> String {
    with(|g| format!("{:016x}", g.rng.state()))
}

/// How many draws have been taken. Kept on the page rather than in core: it is
/// a fact about this demonstration, not about the game.
#[wasm_bindgen]
pub fn theme_id() -> String {
    with(|g| g.theme.clone())
}

/// The assembled board, one item a line: `name\trating\trarity`.
#[wasm_bindgen]
pub fn items() -> String {
    with(|g| {
        g.character
            .combat_items()
            .iter()
            .map(|i| format!("{}\t{}\t{}", i.name, i.rating, i.rarity().name()))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Seat the engine's own preset, so there is a board to save.
#[wasm_bindgen]
pub fn apply_preset() {
    with_mut(|g| {
        g.character = gm2d_core::character::Character::with_all_pieces();
        g.character.loadout.name_seed = g.rng.state();
        g.character.loadout.naming = gm2d_core::theme::by_id(&g.theme).naming;
        g.character.apply_preset();
    });
}

// ---------------------------------------------------------------- the rest

#[wasm_bindgen]
pub fn piece_count() -> usize {
    gm2d_core::piece::CATALOG.len()
}

#[wasm_bindgen]
pub fn monster_count() -> usize {
    gm2d_core::combat::LADDER.len()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The save format this build reads and writes, for the page's footer. A
/// player comparing two builds should be able to see this without opening a
/// file.
#[wasm_bindgen]
pub fn save_version() -> u32 {
    save::VERSION
}
