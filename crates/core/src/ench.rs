//! Enchs: what a Kaklon licensee bolts onto a component.
//!
//! # An ench is not an enchantment, and not a component
//!
//! `PieceKind::Enchantment` already exists — thirteen catalogue pieces, laid
//! *under* the grid so gear sits on top of them. That is upstream's terrain
//! model and it is a different mechanic entirely. The book's own word for the
//! other thing is **ench** (the ench economy, p. 119), so the two words stay
//! two words: no rename, no migration, and nobody has to remember which of two
//! meanings a sentence is using.
//!
//! An ench is also not a component, for the three reasons a restorative is not
//! one: **no shape, no grid, and it is attached rather than worn**. Forcing it
//! into `PieceDef` would make each of those a special case.
//!
//! # One ench a component
//!
//! Deliberately. Two is a bigger design space and a much bigger interface, and
//! neither has earned its place yet. The rule is enforced in [`attach`] rather
//! than assumed by the screens.
//!
//! # The attachment is to the piece, not to the cell
//!
//! `Ench::on` is a `PieceId`, which is an index into the registry — the same
//! identity `owned` and every board placement use. So an ench survives being
//! picked up, turned, moved to another grid and put back down, which is what a
//! player expects of a thing they bolted on. Storing a cell would have meant
//! the ench falling off every time the board was repacked.

use serde::{Deserialize, Serialize};

use crate::loadout::ItemProfile;
use crate::piece::PieceId;

/// The class that may bolt one on, by **canonical** name.
///
/// `Recycler` in the engine and the Kaklon Patent in the book — the theme
/// renames it on the way to the screen, like every other name. Nothing new was
/// invented in combat for it: its `ClassPower` is the one upstream already
/// wrote and already tuned.
pub const LICENSED_CLASS: &str = "Recycler";

pub const FORMAT: &str = "gm2d-enchs";
pub const VERSION: u32 = 1;

/// What an ench does to the item its component is part of.
///
/// **Both of these are things the engine already has.** `power` is what an item
/// multiplies its own damage by and `cooldown_ms` is how often it comes round;
/// `PieceDef::power_bonus` and `PieceDef::speed_bonus` move exactly these two
/// numbers. Nothing new was invented in combat, which is the rule this project
/// has held since M5 and the reason an ench can be added without retuning
/// anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    /// The item multiplies its own damage by this much more, in percent.
    Power { pct: i32 },
    /// The item comes round this much more often, in percent.
    Haste { pct: i32 },
}

impl Effect {
    /// One unthemed line with the number in it — TONE 13a, the same register a
    /// skill node's spec and a component's lines are written in.
    pub fn line(&self) -> String {
        match self {
            Effect::Power { pct } => {
                format!("+{pct}% power to the item this is on")
            }
            Effect::Haste { pct } => {
                format!("the item this is on comes round {pct}% more often")
            }
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Effect::Power { .. } => "Power is what an item multiplies its own damage and \
                 its own payouts by. It never reaches the wearer and never reaches \
                 another item."
                .into(),
            Effect::Haste { .. } => "Cadence is how long an item takes to come round. \
                 Faster is more activations in the same fight, and everything an item \
                 does, it does on its activation."
                .into(),
        }
    }

    /// Apply to one item profile.
    ///
    /// Exhaustive on purpose: a new effect is a compile error here until
    /// somebody has said what the fight does with it.
    pub fn apply(&self, p: &mut ItemProfile) {
        match self {
            Effect::Power { pct } => p.power += pct,
            Effect::Haste { pct } => {
                let faster = (100 + pct).max(10);
                p.cooldown_ms =
                    ((p.cooldown_ms as i64 * 100 / faster as i64) as u32).max(crate::curse::TICK_MS);
            }
        }
    }
}

/// One ench in the catalogue. Content, and it lives in `data/enchs.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnchDef {
    pub id: String,
    pub name: String,
    /// One line, in the world's words. The spec is derived; this is not.
    pub blurb: String,
    /// What a bench charges. **Every town that trades sells every ench**, the
    /// same rule the restoratives follow and for the same reason: a licensee
    /// who walked to the one town that stocks the thing their class is about
    /// would be a licensee who could be stranded from their own class.
    pub price: i32,
    pub effect: Effect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnchsData {
    pub format: String,
    pub version: u32,
    pub enchs: Vec<EnchDef>,
}

impl EnchsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: EnchsData =
            serde_json::from_str(text).map_err(|e| format!("enchs.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these enchs are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        for e in &d.enchs {
            if e.name.is_empty() || e.blurb.is_empty() {
                return Err(format!("{}: an ench that says nothing about itself", e.id));
            }
            if e.price <= 0 {
                return Err(format!("{}: an ench nobody charges for", e.id));
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&EnchDef> {
        self.enchs.iter().find(|e| e.id == id)
    }
}

/// One ench, bolted to one component.
///
/// `active` is the toggle. An ench toggled off is still attached and still
/// yours; it simply does nothing, which is what makes trying an arrangement
/// cheap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ench {
    pub on: PieceId,
    pub id: String,
    pub active: bool,
}

/// Why an ench could not be attached. Every one is a sentence for the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    NoSuchEnch,
    NotYours,
    /// This character is not licensed to bolt anything to anything.
    NoLicence,
    /// Something is already on that component.
    AlreadyEnched(String),
    /// This ench is already on something else.
    AlreadyPlaced,
    NoSuchPiece,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoSuchEnch => write!(f, "there is no such ench"),
            Refusal::NotYours => write!(f, "you have not got one of those"),
            Refusal::NoLicence => write!(f, "you are not licensed to bolt anything to anything"),
            Refusal::AlreadyEnched(what) => write!(f, "{what} is already on that component"),
            Refusal::AlreadyPlaced => write!(f, "that one is already bolted to something"),
            Refusal::NoSuchPiece => write!(f, "you do not own that component"),
        }
    }
}

/// Everything the enchs on a board do to the items on it.
///
/// Applied to the profiles the *fight* runs on, after they are built, because
/// an ench is a property of the character and a profile is the board's answer.
/// Folding it into `Loadout::combat_items` would mean the loadout knowing about
/// something only a licensed character has.
///
/// A component whose ench is toggled off is a component with nothing on it —
/// which is the whole of what the toggle promises.
pub fn apply(profiles: &mut [ItemProfile], enchanted: &[Ench], data: &EnchsData) {
    for e in enchanted {
        if !e.active {
            continue;
        }
        let Some(def) = data.get(&e.id) else { continue };
        for p in profiles.iter_mut() {
            if p.pieces.contains(&e.on) {
                def.effect.apply(p);
            }
        }
    }
}
