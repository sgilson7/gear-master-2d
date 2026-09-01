//! The skill tree, as data.
//!
//! A node is a stat change or a rule change, expressed in terms the engine
//! already has. Three effect kinds at MVP — see [`Effect`] — because a small
//! vocabulary reused across four trees is what keeps M5 from being four times
//! M4's work. A new kind is added when a node needs one, not in advance.
//!
//! The tree itself is `data/skills.json`. Nothing about which nodes exist, what
//! they cost or what they require is in this file; what is here is the shape,
//! the rules about spending, and the arithmetic that turns a set of taken nodes
//! into something the board and the fight can read.

use serde::{Deserialize, Serialize};

use crate::piece::SlotKind;
use crate::stats::Stats;

/// What a node does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Flat stats, added to the character sheet.
    Stat {
        #[serde(default)]
        health: i32,
        #[serde(default)]
        strength: i32,
        #[serde(default)]
        armor: i32,
        #[serde(default)]
        mana: i32,
        #[serde(default)]
        regen: i32,
        #[serde(default)]
        mind_resist: i32,
        #[serde(default)]
        curse_resist: i32,
    },
    /// Rows on one grid, out of the level rotation's turn.
    GrowSlotRows { slot: String, rows: u8 },
    /// Percentage points added to every assembly bonus.
    ///
    /// The engine already has this as `Loadout::assembly_pct` — upstream's
    /// Recycler wrote it. A rule change rather than a stat, and the cheapest
    /// one to express, because the fight already reads it.
    AssemblyPct { pct: i32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    /// One line. What it does, in the world's words, not the engine's.
    pub blurb: String,
    #[serde(default = "one")]
    pub cost: u32,
    /// Node ids that must be taken first.
    #[serde(default)]
    pub requires: Vec<String>,
    pub effect: Effect,
}

fn one() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    pub id: String,
    pub name: String,
    /// `null` for the base tree everybody spends in; a class id for M5's trees.
    #[serde(default)]
    pub class: Option<String>,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillsData {
    pub format: String,
    pub version: u32,
    pub trees: Vec<Tree>,
}

pub const FORMAT: &str = "gm2d-skills";
pub const VERSION: u32 = 1;

/// Why a node could not be taken. Every one is a sentence for the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    NoSuchNode,
    AlreadyTaken,
    NotEnoughPoints { need: u32, have: u32 },
    Missing(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoSuchNode => write!(f, "there is no such skill"),
            Refusal::AlreadyTaken => write!(f, "you have taken that already"),
            Refusal::NotEnoughPoints { need, have } => {
                write!(f, "that costs {need} and you have {have}")
            }
            Refusal::Missing(what) => write!(f, "you would need {what} first"),
        }
    }
}

impl SkillsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: SkillsData =
            serde_json::from_str(text).map_err(|e| format!("skills.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "this tree is version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        Ok(d)
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.trees.iter().flat_map(|t| &t.nodes).find(|n| n.id == id)
    }

    /// The base tree — the one everybody spends in.
    pub fn base(&self) -> Option<&Tree> {
        self.trees.iter().find(|t| t.class.is_none())
    }

    /// Can this node be taken right now?
    ///
    /// The three refusals the plan names, in one place: bought twice, without
    /// its prerequisite, or without a point. A screen that greyed a button out
    /// for its own reasons would be a fourth rule nobody tested.
    pub fn can_take(&self, id: &str, taken: &[String], points: u32) -> Result<&Node, Refusal> {
        let node = self.node(id).ok_or(Refusal::NoSuchNode)?;
        if taken.iter().any(|t| t == id) {
            return Err(Refusal::AlreadyTaken);
        }
        for need in &node.requires {
            if !taken.iter().any(|t| t == need) {
                let name = self.node(need).map(|n| n.name.clone()).unwrap_or_else(|| need.clone());
                return Err(Refusal::Missing(name));
            }
        }
        if points < node.cost {
            return Err(Refusal::NotEnoughPoints { need: node.cost, have: points });
        }
        Ok(node)
    }

    /// Everything a set of taken nodes adds to the character sheet.
    pub fn stats_from(&self, taken: &[String]) -> Stats {
        let mut out = Stats::default();
        for id in taken {
            let Some(n) = self.node(id) else { continue };
            if let Effect::Stat {
                health,
                strength,
                armor,
                mana,
                regen,
                mind_resist,
                curse_resist,
            } = &n.effect
            {
                out.health += health;
                out.strength += strength;
                out.armor += armor;
                out.mana += mana;
                out.regen += regen;
                out.mind_resist += mind_resist;
                out.curse_resist += curse_resist;
            }
        }
        out
    }

    /// Rows granted out of the rotation's turn, indexed by `SlotKind::index`.
    pub fn granted_rows(&self, taken: &[String]) -> [u8; 5] {
        let mut out = [0u8; 5];
        for id in taken {
            let Some(n) = self.node(id) else { continue };
            if let Effect::GrowSlotRows { slot, rows } = &n.effect {
                if let Some(k) = slot_of(slot) {
                    out[k.index()] += rows;
                }
            }
        }
        out
    }

    /// Extra percent on every assembly bonus.
    pub fn assembly_pct(&self, taken: &[String]) -> i32 {
        taken
            .iter()
            .filter_map(|id| self.node(id))
            .filter_map(|n| match &n.effect {
                Effect::AssemblyPct { pct } => Some(*pct),
                _ => None,
            })
            .sum()
    }
}

pub fn slot_of(name: &str) -> Option<SlotKind> {
    Some(match name {
        "weapon" => SlotKind::Weapon,
        "helmet" => SlotKind::Helmet,
        "chest" => SlotKind::Chest,
        "gloves" => SlotKind::Gloves,
        "greaves" => SlotKind::Greaves,
        _ => return None,
    })
}
