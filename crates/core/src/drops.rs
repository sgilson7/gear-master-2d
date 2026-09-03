//! What a creature leaves behind.
//!
//! **Content, and content is not state**, so it is `data/drops.json` and not a
//! table in `piece.rs` — the same division the map, the shelves and the errands
//! make. A rate is the thing about this that will be retuned, and a rate in a
//! data file is a rate somebody can move without a compiler.
//!
//! # Keyed canonically
//!
//! By the creature's canonical name, like a `Slay` goal's, because that is what
//! the engine matches on everywhere. `Ladder` names are what the player reads;
//! these are what the game compares.
//!
//! # Integer per-mille, and rolled every time
//!
//! Every roll in this game is integer per-mille — a seeded walk has to produce
//! the same fights in every browser, and float rounding is the one thing that
//! breaks that silently.
//!
//! And **every entry is rolled whether or not the piece is already in the
//! bag.** Skipping the roll would make the stream a function of what the player
//! is carrying rather than of the fights they had, which is a save that replays
//! differently for the person you sent it to. The refusal happens after the
//! roll, where it costs nothing.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

pub const FORMAT: &str = "gm2d-drops";
pub const VERSION: u32 = 1;

/// The most a drop may be, in per-mille.
///
/// A certainty is not a drop, it is a reward, and the game already has a word
/// for that: a boss's `drops` field, looked up by the tile. Something that
/// falls off every one of a creature is content the player cannot fail to get,
/// and it should be written where that is obvious.
pub const MAX_PER_MILLE: i32 = 500;

/// One creature, one component, one rate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DropDef {
    /// The creature's **canonical** name, as `enemies.json` spells it.
    pub creature: String,
    /// The component's canonical name, as `CATALOG` spells it.
    pub piece: String,
    /// Chance per win, in per-mille.
    pub per_mille: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropsData {
    pub format: String,
    pub version: u32,
    /// Free text in the file, so the rate can carry its own argument. Ignored.
    #[serde(default, rename = "_note")]
    pub note: String,
    pub drops: Vec<DropDef>,
}

impl DropsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: DropsData =
            serde_json::from_str(text).map_err(|e| format!("drops.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these drops are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        // Refused at load rather than discovered by whoever fought the creature
        // for an hour. Same guard `Rule::check` gets and for the same reason: a
        // drop naming something the engine has not got is content nobody can
        // reach, and nothing else in the game would say so.
        for e in &d.drops {
            if crate::combat::creature(&e.creature).is_none() {
                return Err(format!("nothing in the ladder is called {:?}", e.creature));
            }
            if !crate::piece::CATALOG.iter().any(|c| c.name == e.piece) {
                return Err(format!("there is no component called {:?}", e.piece));
            }
            if e.per_mille <= 0 || e.per_mille > MAX_PER_MILLE {
                return Err(format!(
                    "{:?} drops {:?} at {} per mille, which is outside 1..={MAX_PER_MILLE}",
                    e.creature, e.piece, e.per_mille
                ));
            }
        }
        Ok(d)
    }

    /// Everything this creature can leave behind, in file order.
    ///
    /// File order and not sorted: the order decides how many draws the stream
    /// takes before each roll, so it is part of what a seeded walk replays.
    pub fn of(&self, creature: &str) -> Vec<&DropDef> {
        self.drops.iter().filter(|d| d.creature == creature).collect()
    }

    /// Every component any creature drops.
    pub fn every_piece(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.drops.iter().map(|d| d.piece.as_str()).collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// What this creature left behind this time.
///
/// One `below(1000)` per entry, in file order, always — see the module header.
/// Whether the player can *keep* it is the caller's question, because that is
/// about the bag and this is about the corpse.
pub fn roll(data: &DropsData, rng: &mut Rng, creature: &str) -> Vec<String> {
    roll_with(data, rng, creature, 0)
}

/// The same, with a survey's thumb on the scale.
///
/// **The draw is taken either way and the bonus moves the threshold**, which is
/// the same discipline as *every entry is rolled whether or not the piece is
/// already in the bag*: a survey that skipped or added draws would make the
/// stream a function of what you were carrying rather than of the fights you
/// had, and a seeded walk would stop replaying.
pub fn roll_with(
    data: &DropsData,
    rng: &mut Rng,
    creature: &str,
    extra_per_mille: i32,
) -> Vec<String> {
    data.of(creature)
        .into_iter()
        .filter(|d| (rng.below(1000) as i32) < d.per_mille + extra_per_mille)
        .map(|d| d.piece.clone())
        .collect()
}
