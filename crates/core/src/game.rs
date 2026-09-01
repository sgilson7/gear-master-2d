//! Everything one save file holds.
//!
//! `Character` is who you are and what is on your frames. `Game` is that plus
//! the things a *session* owns: the random stream every encounter is drawn
//! from, and which theme is doing the talking.
//!
//! It is deliberately small right now — the world arrives in M2, levels and
//! skills in M4, a class in M5 — and every one of those lands here rather than
//! on `Character`, because the question this type answers is "what has to
//! survive being written to a file and read back".
//!
//! # Adding a field
//!
//! Add it here, and `save.rs` stops compiling until you have said what happens
//! to it. That is on purpose: see `save::SaveState`. A field added to `Game`
//! and forgotten in the save is a save that loads a subtly different game, and
//! nothing about it looks wrong until a player loses a run to it.

use crate::character::Character;
use crate::rng::Rng;
use crate::world::WorldState;

/// The whole of a session's state.
#[derive(Clone, Debug)]
pub struct Game {
    /// The one random stream. Encounter rolls, shop stock, drops — all of it,
    /// so that a seeded walk replays and a save can put the stream back
    /// exactly where it was.
    pub rng: Rng,
    /// Which theme is talking, by id. The theme's *contents* are content and
    /// live in `data/`; a save carries the id and nothing else.
    pub theme: String,
    pub character: Character,
    /// Where you are standing and what you have answered. **Not the map** —
    /// the map is `data/tiles.json` and is derived, never stored.
    pub world: WorldState,
}

impl Game {
    /// A new session from a seed.
    ///
    /// The seed does double duty: it starts the random stream and it seeds the
    /// item-name hash, so the same seed names the same arrangement the same
    /// way for the life of the save.
    pub fn new(seed: u64, theme: &str) -> Self {
        Game {
            rng: Rng::new(seed),
            theme: theme.to_string(),
            character: Character::seeded(seed),
            // Left at the default rather than reading the map: `Game` must not
            // depend on content loading, or a broken data file becomes a
            // panic in every constructor. Whoever has a `World` places the
            // player with `WorldState::at_start`.
            world: WorldState::default(),
        }
    }
}

impl Game {
    /// A creature's name in the theme that is talking.
    ///
    /// Falls through to the canonical name, like every other theme lookup, so
    /// an unthemed creature is one untranslated word rather than a crash.
    pub fn theme_name(&self, canonical: &'static str) -> String {
        crate::theme::by_id(&self.theme).monster(canonical).to_string()
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(0x5EED_1234_ABCD_0001, "td")
    }
}

/// Two games are equal when a player could not tell them apart.
///
/// Hand-written rather than derived because `Character` holds an undo stack
/// that a save deliberately does not carry, and `Loadout` holds a pointer into
/// a theme's word tables. Neither is part of the game; both would make a
/// derived `PartialEq` answer "different" about two identical positions.
///
/// This is the equality M1's round-trip property is stated in, so what it
/// counts is exactly what a save is required to preserve.
impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        let a = &self.character;
        let b = &other.character;
        self.rng.state() == other.rng.state()
            && self.theme == other.theme
            && a.gold == b.gold
            && a.grown_health == b.grown_health
            && a.owned == b.owned
            && a.loadout.name_seed == b.loadout.name_seed
            && a.loadout.locks == b.loadout.locks
            && a.loadout.assembly_pct == b.loadout.assembly_pct
            && a.loadout.slots == b.loadout.slots
            && a.registry == b.registry
            && self.world == other.world
    }
}

impl Eq for Game {}
