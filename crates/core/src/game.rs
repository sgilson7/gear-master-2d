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
use crate::fight::Encounter;
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
    /// The fight the player is standing in, if any.
    ///
    /// No log and no seed: combat does not draw, so the situation is enough to
    /// reproduce the fight exactly. See `fight.rs`.
    pub encounter: Option<Encounter>,
}
// **There is no shelf here any more.** What a town sells is
// `data/shops.json` and never changes, so it is content and is derived where
// it is drawn; what a save carries is `WorldState::bought`, which is the short
// list of things already taken off a shelf. Same discipline as the map.

impl Game {
    /// A new session from a seed.
    ///
    /// The seed does double duty: it starts the random stream and it seeds the
    /// item-name hash, so the same seed names the same arrangement the same
    /// way for the life of the save.
    pub fn new(seed: u64, theme: &str) -> Self {
        let rng = Rng::new(seed);
        Game {
            rng,
            theme: theme.to_string(),
            character: Character::starting_seeded(seed),
            // Left at the default rather than reading the map: `Game` must not
            // depend on content loading, or a broken data file becomes a
            // panic in every constructor. Whoever has a `World` places the
            // player with `WorldState::at_start`.
            world: WorldState::default(),
            encounter: None,
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

    /// A component's name in the theme that is talking.
    ///
    /// Takes a `&str` where `theme_name` takes a `&'static str`, because the
    /// callers have one out of a data file rather than out of `CATALOG` — so
    /// the catalogue is what turns it back into a literal, and a name the
    /// catalogue does not know comes back unchanged.
    ///
    /// A receipt naming a component in the engine's words is a receipt about a
    /// thing no other screen in the game calls that. The three lines that do it
    /// are here: an errand's tally, a boss's drop, and a set piece.
    pub fn theme_piece(&self, canonical: &str) -> String {
        match crate::piece::CATALOG.iter().find(|d| d.name == canonical) {
            Some(d) => crate::theme::by_id(&self.theme).piece(d.name).to_string(),
            None => canonical.to_string(),
        }
    }

    /// Walking into a town takes the tiredness off. Returns how much came off.
    ///
    /// **This is not a rest, and there still is not one.** Health has reset at
    /// every bell since M0, so a rest would restore something that was never
    /// spent — the note at the top of `CLAUDE.md` has said so for five
    /// milestones. What a town undoes is the one thing a fight *does* spend.
    ///
    /// It does not make the tins decoration: a tin is what you drink four
    /// tiles into the Verge with a Rust Colossus on the next square, and the
    /// decision fatigue exists to create is that one. The town is what makes
    /// the walk home worth taking rather than a formality.
    ///
    /// In core rather than in the shim, because "a town mends you" is a rule
    /// and the shim decides nothing.
    pub fn arrive_in_town(&mut self, id: &str) -> i32 {
        self.world.last_town = id.to_string();
        let took = self.character.fatigue;
        self.character.fatigue = 0;
        took
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
            // **Who the character has become**, which this equality left out
            // until an ench needed adding to it. A level-one and a level-nine
            // character are extremely tellable apart, and the round-trip
            // property is stated in this operator — so anything a save has to
            // preserve and this did not compare was being asserted one field
            // at a time in one test and nowhere else.
            && a.xp == b.xp
            && a.carried == b.carried
            && a.fatigue == b.fatigue
            && a.supplies == b.supplies
            && a.skill_points == b.skill_points
            && a.skills_taken == b.skills_taken
            && a.class == b.class
            && a.enchs_owned == b.enchs_owned
            && a.enchanted == b.enchanted
            && self.world == other.world
            && self.encounter == other.encounter
    }
}

impl Eq for Game {}
