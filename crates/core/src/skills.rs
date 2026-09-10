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
    /// An ench, by id into `data/enchs.json`.
    ///
    /// The sixth kind, and the first that hands over a *thing* rather than
    /// changing a number or a rule. M10 took the ench bench off every town, so
    /// a tree is now one of the two places an ench comes from.
    ///
    /// **Derived, never banked**, like every other effect: the character's
    /// enchs are what they were given plus what their taken nodes grant, read
    /// fresh — so retuning which ench a node hands over retunes every save that
    /// took it. Banking it when the point was spent would be the one effect in
    /// this enum that wrote to the save, and `Character::enchs` is what makes
    /// that unnecessary.
    GivesEnch { ench: String },
    /// A named integer on the live power, moved by `by`.
    ///
    /// **The seventh effect kind, and the first that cannot be read without
    /// knowing which class you are in.** Every other effect changes the
    /// character; this one changes *the class's own sentence*, which is what
    /// an expert tree is for and what stops it becoming a second base tree.
    ///
    /// The knob is a plain name for the reason `Rule::CurseOnActivate`'s slot
    /// is one: the vocabulary lives with the power that declares it —
    /// [`crate::expert::ExpertPower::knobs`] — and a second enum listing
    /// rate-cap-floor-rebate would be two lists to keep in step.
    /// `SkillsData::parse` checks a tree's knobs against the power its class
    /// declares, so **a tree naming a knob its own class has not got does not
    /// load**. That guard is what makes
    /// [`crate::expert::ExpertPower::tune`] safe to no-op on an unknown name.
    ///
    /// Tenths where a knob is fractional: `rate: 20` is 2.0 strength a point.
    /// Everything a player reads is printed by `describe()` from the *tuned*
    /// value, so the promise re-reads itself after every point and cannot go
    /// stale — which was already the rule for the five base classes and is now
    /// the rule for twelve points of tuning.
    Tunes { knob: String, by: i32 },
}

/// What a node can grant that is not a number.
///
/// **Not the tree's any more.** M9.0 widened `Effect::Grants` to an assembled
/// item, so the type moved to [`crate::rule`] and this is the name the tree
/// knows it by. Nothing about `data/skills.json` changed.
pub use crate::rule::Rule;


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
            // **The ench's name, not its id, and not its spec.** A node that
            // named a key would be a node the player has to go and look up,
            // and one that quoted the whole spec put Bench Rights over the
            // ninety characters `a_mechanical_line_stays_short_enough_to_read_at_a_glance`
            // allows. What it does is the hover's job, below. `+1` because
            // that is the register every other effect on the button uses.
            Effect::GivesEnch { ench } => match crate::data::enchs().get(ench) {
                Some(e) => format!("+1 {} in the rack", e.name),
                None => format!("+1 {ench} in the rack"),
            },
            // **The knob's own name and the signed number, and nothing else.**
            // What the knob *means* is the class's promise, which the expert
            // tab prints at its head and `detail` repeats on the hover — so a
            // line here that re-explained the class would be the promise
            // written down sixty times, once a node, going stale sixty ways.
            // **Milliseconds are printed as seconds**, because that is what
            // the class's own promise prints and a line that disagreed with
            // the sentence it changes would be two answers to one question.
            // Everything else is the knob's own word with its underscores
            // opened out — engine words, unthemed, TONE 13a.
            Effect::Tunes { knob, by } => match knob.strip_suffix("_ms") {
                Some(name) => {
                    format!("{:+.1}s {}", *by as f32 / 1000.0, name.replace('_', " "))
                }
                None => format!("{by:+} {}", knob.replace('_', " ")),
            },
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
                        "Strength: added to every physical hit you land, then scaled by the power \
                         of the item landing it — so it is worth more on a strong weapon."
                            .into(),
                    );
                }
                if *regen != 0 {
                    out.push("Regeneration: health restored once a second, all fight.".into());
                }
                if *mind_resist != 0 {
                    out.push(
                        "Mind resist: cuts incoming mind damage by that percent. Mind damage takes \
                         maximum health rather than health, and nothing heals it back."
                            .into(),
                    );
                }
                if *curse_resist != 0 {
                    out.push(
                        "Curse resist: cuts how long a curse landed on you lasts by that percent. \
                         It does not stop the curse landing."
                            .into(),
                    );
                }
            }
            Effect::StartWith { armor, mana } => {
                if *armor != 0 {
                    out.push(
                        "Armor: absorbs damage before health does. Everybody starts a fight with \
                         none, and whatever is left is gone when the fight ends."
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
            // **There is no level rotation, and this sentence said there was.**
            // M12.3 retired it — `ROTATION`, `rows_for` and `grows_at` are all
            // gone, every frame starts at three rows and stays there — and this
            // hover went on telling players a row was *"granted out of turn, on
            // top of the row that grid gets when the level rotation reaches
            // it"* for three blocks after the turn stopped existing.
            //
            // The same failure as the `STARTER` comment this project's own
            // notes quoted as live fact for five blocks, and as the controls
            // blurb M12.B deleted for saying *every level adds a row to one
            // frame* — except player-facing, which is worse. Found by a lint
            // written for a formatting nit; see `SECOND-ORDER-M15.md` row 8.
            Effect::GrowSlotRows { slot, .. } => out.push(format!(
                "A row is {} more cells to pack the {slot} grid with. A level hands out none, so \
                 every row on every frame is one of these or an errand's. No grid goes past {} \
                 rows.",
                crate::slot::SLOT_W,
                crate::progression::MAX_ROWS,
            )),
            Effect::Grants { rule } => out.extend(rule.detail()),
            Effect::GivesEnch { ench } => {
                if let Some(e) = crate::data::enchs().get(ench) {
                    out.push(format!("{}: {}", e.name, e.effect.detail()));
                }
                out.push(
                    "An ench bolts onto one component and works on the item that component \
                     is part of. Bolting one on wants a class that may; being given one does \
                     not."
                        .into(),
                );
            }
            // The class's whole promise, **after this node is taken**, which
            // is the only way to say what moving a knob does: the number is
            // meaningless and the sentence it changes is not. Written by
            // `ExpertPower::describe` off the tuned value, so it cannot
            // disagree with what the node actually does.
            Effect::Tunes { knob, by } => {
                if let Some(e) = crate::expert::EXPERTS
                    .iter()
                    .find(|e| e.power.knobs().contains(&knob.as_str()))
                {
                    let mut after = e.power;
                    after.tune(knob, *by);
                    out.push(format!("{}: {}", e.name, after.describe()));
                }
            }
            Effect::AssemblyPct { .. } => out.push(
                "An assembly bonus is the lump a component pays only when the item it is part of is \
                 complete. This raises every one of them, on all five grids — so it pays a board \
                 that finishes what it seats and nothing at all to one that does not."
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
    ///
    /// **A node that grows every frame says so once.** Listing the five
    /// separately came to a hundred and thirty-three characters — half again
    /// over what `a_mechanical_line_stays_short_enough_to_read_at_a_glance`
    /// allows, and a line nobody reads is a line that is not there. It is
    /// collapsed rather than hand-written, so a tier that stops covering all
    /// five goes back to naming them and cannot quietly claim the set.
    pub fn line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        let grows: Vec<(&str, u8)> = self
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::GrowSlotRows { slot, rows } => Some((slot.as_str(), *rows)),
                _ => None,
            })
            .collect();
        // Every worn frame, all by the same amount, and none of them twice.
        let every = crate::piece::SlotKind::ALL.len();
        let same = grows.first().map(|(_, r)| *r);
        // Through `slot_of`, which is the one place a slot's written name is
        // turned into a slot — a second mapping here would be a second answer
        // to what "greaves" means.
        let covers_all = grows.len() == every
            && grows.iter().all(|(_, r)| Some(*r) == same)
            && crate::piece::SlotKind::ALL
                .iter()
                .all(|&k| grows.iter().any(|(s, _)| slot_of(s) == Some(k)));
        if covers_all {
            let rows = same.unwrap_or(0);
            parts.push(format!("+{rows} row{} on every grid", if rows == 1 { "" } else { "s" }));
        }
        for e in &self.effects {
            if covers_all && matches!(e, Effect::GrowSlotRows { .. }) {
                continue;
            }
            parts.push(e.line());
        }
        parts.join(", ")
    }

    /// Every effect's [`Effect::detail`], for the hover.
    pub fn detail(&self) -> Vec<String> {
        self.effects.iter().flat_map(|e| e.detail()).collect()
    }

    /// Every knob this node moves, and by how much.
    ///
    /// On the node rather than only on `SkillsData`, because the lints ask a
    /// node directly and a second walk over `effects` in a test file would be
    /// the "second copy of a list" this project keeps paying for.
    pub fn tunings(&self) -> Vec<(String, i32)> {
        self.effects
            .iter()
            .filter_map(|e| match e {
                Effect::Tunes { knob, by } => Some((knob.clone(), *by)),
                _ => None,
            })
            .collect()
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
        let enchs = crate::data::enchs();
        for t in &d.trees {
            for n in &t.nodes {
                for e in &n.effects {
                    if let Effect::Grants { rule } = e {
                        rule.check().map_err(|why| format!("{}: {why}", n.id))?;
                    }
                    // A node handing over an ench nobody has heard of is a
                    // point spent on nothing, which is the failure this file
                    // exists to stop shipping. Same guard `Rule::check` is.
                    if let Effect::GivesEnch { ench } = e {
                        if enchs.get(ench).is_none() {
                            return Err(format!("{}: there is no ench called {ench:?}", n.id));
                        }
                    }
                    // **A knob is checked against the power its own tree
                    // belongs to.** Not against every knob in the game: `carry`
                    // is Standing Fact's *and* Overwound Arm's, so a global
                    // vocabulary would let a Standing Fact node tune a knob it
                    // has not got and `ExpertPower::tune` would silently do
                    // nothing — which is the serde-drops-a-key failure
                    // wearing a new coat. This is what makes that no-op safe.
                    if let Effect::Tunes { knob, by } = e {
                        let owner = t.class.as_deref().unwrap_or("");
                        let Some(power) = crate::expert::by_name(owner).map(|e| e.power) else {
                            return Err(format!(
                                "{}: {:?} tunes {knob:?}, and only an expert class has knobs",
                                n.id, t.id
                            ));
                        };
                        if !power.knobs().contains(&knob.as_str()) {
                            return Err(format!(
                                "{}: there is no {knob:?} on {owner} — it has {:?}",
                                n.id,
                                power.knobs()
                            ));
                        }
                        // A tuning that tunes nothing is a point spent on
                        // nothing, exactly as `Rule::check` refuses a spin
                        // that banks no stacks.
                        let step = crate::expert::ExpertPower::step(knob);
                        if *by == 0 {
                            return Err(format!("{}: moves {knob:?} by nothing at all", n.id));
                        }
                        if by.rem_euclid(step) != 0 {
                            return Err(format!(
                                "{}: moves {knob:?} by {by}, which is less than the {step} it \
                                 takes to change what the class says",
                                n.id
                            ));
                        }
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

    /// Is this class's tree **finished** — every node of it taken?
    ///
    /// **Every node, not most of them, and not the base tree.** The base is
    /// nobody's class; the class trees are eight to ten nodes, so a player can
    /// count them and the game may too. M13 hangs the second paper off this,
    /// and a threshold like "most of it" would be a number nobody could check
    /// from inside the game — TONE rule 4.
    ///
    /// *Why not a level gate.* Levels measure walking and a tree measures the
    /// class. A level-forty save that hoarded its points has mastered nothing,
    /// and would be handed a second class for having walked about.
    ///
    /// **A class with no tree is never finished.** That is deliberate rather
    /// than an oversight: the five offered classes all have one, and a sixth
    /// added without a tree would otherwise unlock the second paper the moment
    /// it was taken — a class finished by having no work in it.
    pub fn tree_finished(&self, class: &str, taken: &[String]) -> bool {
        let Some(t) = self.tree_for_class(class) else { return false };
        !t.nodes.is_empty() && t.nodes.iter().all(|n| taken.iter().any(|x| x == &n.id))
    }

    /// How much of a class's tree is taken, out of how much there is.
    ///
    /// **The refusal names the count.** *"You have finished six of the eight,
    /// and he can count"* — a shelf line that greys out saying only "not yet"
    /// is a line that reads as broken, which is TONE rule 12 and the reason
    /// this returns a pair rather than a bool the caller has to phrase around.
    pub fn tree_progress(&self, class: &str, taken: &[String]) -> (usize, usize) {
        let Some(t) = self.tree_for_class(class) else { return (0, 0) };
        let have = t.nodes.iter().filter(|n| taken.iter().any(|x| x == &n.id)).count();
        (have, t.nodes.len())
    }

    /// Can this node be taken right now?
    ///
    /// The three refusals the plan names, in one place: bought twice, without
    /// its prerequisite, or without a point. A screen that greyed a button out
    /// for its own reasons would be a fourth rule nobody tested.
    /// **`classes` is every class the character is, not the first one.** It
    /// was `Option<&str>` and that was right while a character could only be
    /// one thing; M13 gives them up to three, and a ledger that read only the
    /// level-five fork refused every node of a tree somebody had *bought* —
    /// which is the shape of failure this file exists to stop, arriving from
    /// the other side. Found by the first M13.3 test that tried to take one.
    pub fn can_take(
        &self,
        id: &str,
        taken: &[String],
        points: u32,
        classes: &[&str],
    ) -> Result<&Node, Refusal> {
        let node = self.node(id).ok_or(Refusal::NoSuchNode)?;
        // A class tree is shut to everybody but its class. Checked before
        // anything else, because "you would need X first" about a node you can
        // never take is a worse answer than "that is not yours".
        if let Some(owner) = self.tree_of(id).and_then(|t| t.class.clone()) {
            if classes.is_empty() {
                return Err(Refusal::NoClassYet);
            }
            if !classes.iter().any(|c| *c == owner) {
                let name = self
                    .trees
                    .iter()
                    .find(|t| t.class.as_deref() == Some(owner.as_str()))
                    .map(|t| t.name.clone())
                    .unwrap_or(owner);
                return Err(Refusal::WrongClass(name));
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
    /// Every ench a set of taken nodes hands over, by id.
    ///
    /// A list rather than a set: two nodes granting the same ench is two of
    /// them, the same way two of anything else is.
    pub fn enchs_from(&self, taken: &[String]) -> Vec<String> {
        taken
            .iter()
            .filter_map(|id| self.node(id))
            .flat_map(|n| &n.effects)
            .filter_map(|e| match e {
                Effect::GivesEnch { ench } => Some(ench.clone()),
                _ => None,
            })
            .collect()
    }

    /// Every knob a set of taken nodes moves, and by how much.
    ///
    /// A list of `(knob, by)` rather than a resolved power, because *which*
    /// power is the character's question and this file only knows about trees.
    /// `Character::expert_power` folds it over the right one, and a tuning
    /// naming a knob that power has not got moves nothing — which is safe only
    /// because `parse` refuses that tree outright.
    pub fn tunings_from(&self, taken: &[String]) -> Vec<(String, i32)> {
        taken
            .iter()
            .filter_map(|id| self.node(id))
            .flat_map(|n| &n.effects)
            .filter_map(|e| match e {
                Effect::Tunes { knob, by } => Some((knob.clone(), *by)),
                _ => None,
            })
            .collect()
    }

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
