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
///
/// Every variant has to be describable in one unthemed line with a number in
/// it — see [`Effect::line`]. That is not a documentation rule, it is the
/// reason the vocabulary stays small: an effect nobody can state plainly is an
/// effect nobody can decide about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Flat stats, added to the character sheet.
    ///
    /// **Armour and mana are deliberately not here.** Everywhere else in the
    /// engine they are grants an item makes on its own tick, so a
    /// character-level total of them has no tick to hang off — and
    /// `Combatant::player` has always thrown that total away. Eight nodes
    /// shipped granting one or the other and did nothing at all. What they
    /// meant is [`Effect::StartWith`], which is a different rule and says so.
    ///
    /// serde ignores a key it does not know, so a node left saying `armor`
    /// here would go on quietly doing nothing — which is the exact failure
    /// this split exists to end. `tests/tone.rs` reads the raw JSON and
    /// refuses any effect key the vocabulary has never had.
    Stat {
        #[serde(default)]
        health: i32,
        #[serde(default)]
        strength: i32,
        #[serde(default)]
        regen: i32,
        #[serde(default)]
        mind_resist: i32,
        #[serde(default)]
        curse_resist: i32,
    },
    /// What the player is already holding when the bell goes.
    ///
    /// Armour soaks before health and is gone when the fight ends; mana is
    /// what a casting item spends. Both start at zero for everybody, so this
    /// is the only way to begin a fight with either.
    StartWith {
        #[serde(default)]
        armor: i32,
        #[serde(default)]
        mana: i32,
    },
    /// Rows on one grid, out of the level rotation's turn.
    GrowSlotRows { slot: String, rows: u8 },
    /// Percentage points added to every assembly bonus.
    ///
    /// The engine already has this as `Loadout::assembly_pct` — upstream's
    /// Recycler wrote it. A rule change rather than a stat, and the cheapest
    /// one to express, because the fight already reads it.
    AssemblyPct { pct: i32 },
    /// A rule, rather than arithmetic.
    ///
    /// The first effect kind that is not a number. Everything above adds to
    /// something the engine already totals up; this one says the game works
    /// differently for you now.
    Grants { rule: Rule },
}

/// What a node can grant that is not a number.
///
/// **An enum, not a string.** An exhaustive match is the only thing that keeps
/// a rule nobody reads from shipping — which has happened here before, and
/// silently: `Effect::Stat` carried `armor` and `mana` that nothing consumed,
/// and eight nodes cost a point and did nothing for two milestones. serde
/// drops a key it does not know without a word, so the guard has to be a match
/// the compiler checks and a validation the loader runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Rule {
    /// Every activation of an item in this slot lands a curse on them.
    ///
    /// The slot and the curse are named as strings because both vocabularies
    /// already live elsewhere — `SlotKind` and `CurseKind` — and duplicating
    /// either here would be two lists to keep in step. [`Rule::check`] is what
    /// stops a misspelling reaching a player, and `SkillsData::parse` runs it.
    CurseOnActivate { slot: String, curse: String },
    /// The danger of a region and the odds on a tile become readable.
    ///
    /// Not a combat rule: nothing in a fight reads it. It gates what the map
    /// screen is allowed to print, which is why it is a rule and not a stat —
    /// there is no number to add.
    Scout,
}

impl Rule {
    /// Refuse a rule that names something the engine has not got.
    ///
    /// The parse-time half of the guard. The compile-time half is that every
    /// match on `Rule` is exhaustive.
    pub fn check(&self) -> Result<(), String> {
        match self {
            Rule::CurseOnActivate { slot, curse } => {
                if slot_of(slot).is_none() {
                    return Err(format!("there is no {slot:?} grid"));
                }
                if crate::curse::CurseKind::by_name(curse).is_none() {
                    return Err(format!("there is no curse of {curse:?}"));
                }
                Ok(())
            }
            Rule::Scout => Ok(()),
        }
    }

    /// One unthemed line with the number in it, the same as every other
    /// effect. TONE 13a: the name carries the world and this carries the rule.
    pub fn line(&self) -> String {
        match self {
            Rule::CurseOnActivate { slot, curse } => {
                format!("every activation of a {slot} item lands 1 curse of {curse} on them")
            }
            // The odds are per-mille per step, which is the unit the engine
            // actually works in — a spec with no number in it is the vagueness
            // this register exists to remove.
            Rule::Scout => format!(
                "read a region's danger and every tile's odds out of {}, on the map",
                1000,
            ),
        }
    }

    pub fn detail(&self) -> Vec<String> {
        match self {
            Rule::CurseOnActivate { curse, .. } => {
                let k = crate::curse::CurseKind::by_name(curse);
                vec![
                    format!(
                        "Curse of {curse}: {}.",
                        k.map(|k| k.describe().to_string()).unwrap_or_default()
                    ),
                    "It lands on top of whatever the item already does, and it stacks with \
                     the same curse off your own gear."
                        .into(),
                ]
            }
            Rule::Scout => vec![
                format!(
                    "Danger is the mean rating of what a region holds. The odds are \
                     per-mille per step — {} at the very worst, whatever the ground.",
                    crate::world::MAX_ENCOUNTER_PER_MILLE,
                ),
                "Nothing shows either until this is taken.".into(),
            ],
        }
    }
}

/// One line saying exactly what taking this node does.
///
/// **No theme and no flavour.** The name carries the world; this carries the
/// arithmetic, and a player deciding where to spend a point is reading it to
/// compare two numbers. `blurb` is where the mine and the plaid suit live.
///
/// Every branch names a number, because a description without one is the
/// vagueness this exists to remove.
impl Effect {
    pub fn line(&self) -> String {
        match self {
            Effect::Stat { health, strength, regen, mind_resist, curse_resist } => join(&[
                num(*health, "max health", ""),
                num(*strength, "strength", ""),
                num(*regen, "health a second", ""),
                num(*mind_resist, "mind resist", "%"),
                num(*curse_resist, "curse resist", "%"),
            ]),
            Effect::StartWith { armor, mana } => join(&[
                (*armor != 0).then(|| format!("start every fight with {armor} armor")),
                (*mana != 0).then(|| format!("start every fight with {mana} mana")),
            ]),
            Effect::GrowSlotRows { slot, rows } => {
                format!("+{rows} row{} on the {slot} grid", if *rows == 1 { "" } else { "s" })
            }
            Effect::AssemblyPct { pct } => format!("+{pct}% to every assembly bonus"),
            Effect::Grants { rule } => rule.line(),
        }
    }

    /// What the words in [`Effect::line`] mean, for the hover.
    ///
    /// One entry per term the line actually used, so a node granting health
    /// does not explain curse resistance at somebody who did not ask.
    pub fn detail(&self) -> Vec<String> {
        let mut out = Vec::new();
        match self {
            Effect::Stat { health, strength, regen, mind_resist, curse_resist } => {
                if *health != 0 {
                    out.push(
                        "Max health: damage comes off health, and you lose at zero.".into(),
                    );
                }
                if *strength != 0 {
                    out.push(
                        "Strength: added to every physical hit you land, then scaled by the                          power of the item landing it — so it is worth more on a strong weapon."
                            .into(),
                    );
                }
                if *regen != 0 {
                    out.push("Regeneration: health restored once a second, all fight.".into());
                }
                if *mind_resist != 0 {
                    out.push(
                        "Mind resist: cuts incoming mind damage by that percent. Mind damage                          takes maximum health rather than health, and nothing heals it back."
                            .into(),
                    );
                }
                if *curse_resist != 0 {
                    out.push(
                        "Curse resist: cuts how long a curse landed on you lasts by that                          percent. It does not stop the curse landing."
                            .into(),
                    );
                }
            }
            Effect::StartWith { armor, mana } => {
                if *armor != 0 {
                    out.push(
                        "Armor: absorbs damage before health does. Everybody starts a fight                          with none, and whatever is left is gone when the fight ends."
                            .into(),
                    );
                }
                if *mana != 0 {
                    let cost = crate::combat::SPELL_MANA_COST;
                    out.push(format!(
                        "Mana: what a casting item spends, {cost} a cast. Everybody starts a \
                         fight with none, so this is {} casts before anything on the board has \
                         to earn them.",
                        mana / cost.max(1),
                    ));
                }
            }
            Effect::GrowSlotRows { slot, .. } => out.push(format!(
                "A row is {} more cells to pack the {slot} grid with, granted out of turn — on                  top of the row that grid gets when the level rotation reaches it. No grid goes                  past {} rows.",
                crate::slot::SLOT_W,
                crate::progression::MAX_ROWS,
            )),
            Effect::Grants { rule } => out.extend(rule.detail()),
            Effect::AssemblyPct { .. } => out.push(
                "An assembly bonus is the lump a component pays only when the item it is part                  of is complete. This raises every one of them, on all five grids — so it pays                  a board that finishes what it seats and nothing at all to one that does not."
                    .into(),
            ),
        }
        out
    }
}

/// `+3 strength`, or nothing at all when the field is zero.
fn num(n: i32, label: &str, unit: &str) -> Option<String> {
    (n != 0).then(|| format!("{n:+}{unit} {label}"))
}

fn join(parts: &[Option<String>]) -> String {
    parts.iter().flatten().cloned().collect::<Vec<_>>().join(", ")
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
    /// What it does.
    ///
    /// Written in the JSON as one object, or as an array for a node that does
    /// two things — most do one, and reading `"effect": {...}` is what a
    /// person writing a tree expects to be able to type.
    #[serde(
        rename = "effect",
        deserialize_with = "one_or_many",
        serialize_with = "many_or_one"
    )]
    pub effects: Vec<Effect>,
}

fn one_or_many<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Effect>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(Effect),
        Many(Vec<Effect>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::One(e) => vec![e],
        OneOrMany::Many(v) => v,
    })
}

fn many_or_one<S: serde::Serializer>(v: &[Effect], s: S) -> Result<S::Ok, S::Error> {
    match v {
        [one] => one.serialize(s),
        many => many.serialize(s),
    }
}

impl Node {
    /// Every effect's [`Effect::line`], in one unthemed sentence.
    pub fn line(&self) -> String {
        self.effects.iter().map(Effect::line).collect::<Vec<_>>().join(", ")
    }

    /// Every effect's [`Effect::detail`], for the hover.
    pub fn detail(&self) -> Vec<String> {
        self.effects.iter().flat_map(|e| e.detail()).collect()
    }
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

impl Tree {
    /// How far down this node sits: **0 when nothing has to be taken first**,
    /// otherwise one past the deepest thing it needs.
    ///
    /// A property of the prerequisite graph, so it lives here rather than in
    /// the page that draws it. A screen working its own layering out would be
    /// a second answer to "what has to come first", and the two would disagree
    /// the first time a node gained a second prerequisite.
    ///
    /// A cycle cannot deepen a node for ever: the walk refuses to revisit, and
    /// `no_tree_requires_itself_in_a_circle` refuses the data outright.
    pub fn depth_of(&self, id: &str) -> u32 {
        fn walk(t: &Tree, id: &str, seen: &mut Vec<String>) -> u32 {
            if seen.iter().any(|s| s == id) {
                return 0;
            }
            let Some(n) = t.nodes.iter().find(|n| n.id == id) else { return 0 };
            if n.requires.is_empty() {
                return 0;
            }
            seen.push(id.to_string());
            let d = n.requires.iter().map(|r| walk(t, r, seen)).max().unwrap_or(0) + 1;
            seen.pop();
            d
        }
        walk(self, id, &mut Vec::new())
    }

    /// The nodes grouped by depth, shallowest first.
    ///
    /// What you can spend a point on right now is the top row; everything that
    /// asks for something first is below whatever it asks for.
    pub fn rows(&self) -> Vec<Vec<&Node>> {
        let mut rows: Vec<Vec<&Node>> = Vec::new();
        for n in &self.nodes {
            let d = self.depth_of(&n.id) as usize;
            if rows.len() <= d {
                rows.resize_with(d + 1, Vec::new);
            }
            rows[d].push(n);
        }
        rows
    }
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
    /// The node is in a tree belonging to a class this character is not.
    WrongClass(String),
    /// The node is in a class tree and no class has been chosen yet.
    NoClassYet,
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
            Refusal::WrongClass(what) => write!(f, "that is {what}'s, and you are not one"),
            Refusal::NoClassYet => write!(f, "that wants a class, and you have not taken one"),
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
        // A rule naming a grid or a curse the engine has not got is a node that
        // costs a point and does nothing. Refused here rather than discovered
        // by whoever spent the point.
        for t in &d.trees {
            for n in &t.nodes {
                for e in &n.effects {
                    if let Effect::Grants { rule } = e {
                        rule.check().map_err(|why| format!("{}: {why}", n.id))?;
                    }
                }
            }
        }
        Ok(d)
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.trees.iter().flat_map(|t| &t.nodes).find(|n| n.id == id)
    }

    /// Which tree a node belongs to.
    pub fn tree_of(&self, id: &str) -> Option<&Tree> {
        self.trees.iter().find(|t| t.nodes.iter().any(|n| n.id == id))
    }

    /// The tree belonging to a class, if it has one.
    pub fn tree_for_class(&self, class: &str) -> Option<&Tree> {
        self.trees.iter().find(|t| t.class.as_deref() == Some(class))
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
    pub fn can_take(
        &self,
        id: &str,
        taken: &[String],
        points: u32,
        class: Option<&str>,
    ) -> Result<&Node, Refusal> {
        let node = self.node(id).ok_or(Refusal::NoSuchNode)?;
        // A class tree is shut to everybody but its class. Checked before
        // anything else, because "you would need X first" about a node you can
        // never take is a worse answer than "that is not yours".
        if let Some(owner) = self.tree_of(id).and_then(|t| t.class.clone()) {
            match class {
                None => return Err(Refusal::NoClassYet),
                Some(c) if c != owner => {
                    let name = self
                        .trees
                        .iter()
                        .find(|t| t.class.as_deref() == Some(owner.as_str()))
                        .map(|t| t.name.clone())
                        .unwrap_or(owner);
                    return Err(Refusal::WrongClass(name));
                }
                Some(_) => {}
            }
        }
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
            for e in &n.effects {
                if let Effect::Stat { health, strength, regen, mind_resist, curse_resist } = e {
                    out.health += health;
                    out.strength += strength;
                    out.regen += regen;
                    out.mind_resist += mind_resist;
                    out.curse_resist += curse_resist;
                }
            }
        }
        out
    }

    /// Rows granted out of the rotation's turn, indexed by `SlotKind::index`.
    pub fn granted_rows(&self, taken: &[String]) -> [u8; 5] {
        let mut out = [0u8; 5];
        for id in taken {
            let Some(n) = self.node(id) else { continue };
            for e in &n.effects {
                if let Effect::GrowSlotRows { slot, rows } = e {
                    if let Some(k) = slot_of(slot) {
                        out[k.index()] += rows;
                    }
                }
            }
        }
        out
    }

    /// Armour and mana the player begins every fight already holding.
    ///
    /// Separate from [`stats_from`](Self::stats_from) because it has to be:
    /// the character's stat total already carries the *per activation* armour
    /// and mana its items grant, and adding that to what a fight starts with
    /// would pay every item twice.
    pub fn start_with(&self, taken: &[String]) -> crate::combat::Held {
        let mut out = crate::combat::Held::default();
        for id in taken {
            let Some(n) = self.node(id) else { continue };
            for e in &n.effects {
                match e {
                    Effect::StartWith { armor, mana } => {
                        out.armor += armor;
                        out.mana += mana;
                    }
                    // A granted rule goes through the same door and for the
                    // same reason: it is a fight input rather than a mutable
                    // global, so it arrives beside the stats.
                    Effect::Grants { rule } => out.rules.push(rule.clone()),
                    _ => {}
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
            .flat_map(|n| &n.effects)
            .filter_map(|e| match e {
                Effect::AssemblyPct { pct } => Some(*pct),
                _ => None,
            })
            .sum()
    }
}

impl SkillsData {
    /// Every rule a set of taken nodes grants.
    ///
    /// One list rather than one accessor a rule, so adding a rule is adding a
    /// variant and nothing else. Whoever consumes it matches exhaustively.
    pub fn rules_from(&self, taken: &[String]) -> Vec<Rule> {
        taken
            .iter()
            .filter_map(|id| self.node(id))
            .flat_map(|n| &n.effects)
            .filter_map(|e| match e {
                Effect::Grants { rule } => Some(rule.clone()),
                _ => None,
            })
            .collect()
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
