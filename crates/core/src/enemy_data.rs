//! The bestiary, as data.
//!
//! `PLAN.md` M3 asks for `data/enemies.json` with every enemy written down as a
//! real loadout of catalogue components — the original's "monsters wear the
//! catalogue" rule, kept because it is the rule that makes a creature's
//! difficulty a thing you can read rather than a number somebody chose.
//!
//! The ladder already *is* that, in `combat.rs`, and it is the thing the golden
//! fixture was captured against. So this module does what `theme_data` does:
//! writes the shipped table out, and proves the file is lossless. The file is
//! the readable, editable record; the table is still what runs. M-later flips
//! the dependency, and the test here is what makes that safe to do without
//! rechecking anything.
//!
//! What the file is *for* today: it is the thing a person opens to ask what a
//! creature wears, and the thing `tests/enemies.rs` reads to prove every one of
//! them assembles.

use serde::{Deserialize, Serialize};

use crate::combat::{Difficulty, MonsterSpec};

/// One placement: component, slot, x, y, rotation.
pub type Placement = (String, String, u8, u8, u8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackData {
    pub name: String,
    pub cooldown_ms: u32,
    pub damage: i32,
    #[serde(default)]
    pub mind: i32,
    #[serde(default)]
    pub armor: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyData {
    pub name: String,
    pub health: i32,
    pub strength: i32,
    #[serde(default)]
    pub regen: i32,
    #[serde(default)]
    pub mind_resist: i32,
    #[serde(default)]
    pub curse_resist: i32,
    #[serde(default)]
    pub physical_resist: i32,
    #[serde(default)]
    pub magic_resist: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attacks: Vec<AttackData>,
    /// What it wears, in catalogue names. Empty means it fights with its body.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gear: Vec<Placement>,
    pub bounty: i32,
    pub rank: String,
    /// What this creature is worth on the shared scale, measured rather than
    /// typed. Written out because it is the number a person reading this file
    /// actually wants, and regenerated with the file so it cannot drift.
    pub rating: i32,
    /// How many assembled items its gear makes. Zero for a creature that
    /// fights bare; **anything with gear and no items is a typo**, which is
    /// what `tests/enemies.rs` is for.
    pub items: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BestiaryData {
    pub format: String,
    pub version: u32,
    /// The setting the ratings and gear below were measured at. A creature's
    /// equipment steps with difficulty, so a bestiary that did not say which
    /// one it was written at would be a bestiary of nothing in particular.
    pub difficulty: String,
    pub enemies: Vec<EnemyData>,
}

pub const FORMAT: &str = "gm2d-enemies";
pub const VERSION: u32 = 1;

fn slot_name(s: crate::piece::SlotKind) -> String {
    format!("{s:?}").to_lowercase()
}

impl EnemyData {
    pub fn of(spec: &'static MonsterSpec, difficulty: Difficulty) -> Self {
        let (reg, lo) = spec.loadout_at(difficulty);
        EnemyData {
            name: spec.name.to_string(),
            health: spec.health,
            strength: spec.strength,
            regen: spec.regen,
            mind_resist: spec.mind_resist,
            curse_resist: spec.curse_resist,
            physical_resist: spec.physical_resist,
            magic_resist: spec.magic_resist,
            attacks: spec
                .attacks
                .iter()
                .map(|a| AttackData {
                    name: a.name.to_string(),
                    cooldown_ms: a.cooldown_ms,
                    damage: a.damage,
                    mind: a.mind,
                    armor: a.armor,
                })
                .collect(),
            gear: spec
                .gear_at(difficulty)
                .into_iter()
                .map(|(n, s, x, y, r)| (n.to_string(), slot_name(s), x, y, r))
                .collect(),
            bounty: spec.bounty,
            rank: format!("{:?}", spec.rank).to_lowercase(),
            rating: crate::rating::creature_rating(spec, difficulty),
            items: lo.combat_items(&reg).len(),
        }
    }
}

impl BestiaryData {
    pub fn of(difficulty: Difficulty) -> Self {
        BestiaryData {
            format: FORMAT.to_string(),
            version: VERSION,
            difficulty: format!("{difficulty:?}").to_lowercase(),
            enemies: crate::combat::LADDER
                .iter()
                .map(|m| EnemyData::of(m, difficulty))
                .collect(),
        }
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let d: BestiaryData =
            serde_json::from_str(text).map_err(|e| format!("enemies.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "this bestiary is version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        Ok(d)
    }

    pub fn get(&self, name: &str) -> Option<&EnemyData> {
        self.enemies.iter().find(|e| e.name == name)
    }
}
