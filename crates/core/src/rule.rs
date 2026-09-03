//! What something can grant that is not a number.
//!
//! # Why this is not in `skills.rs` any more
//!
//! [`Rule`] arrived in M8.3 as the skill tree's fifth effect kind, and lived
//! in `skills.rs` because the tree was the only thing that could hand one out.
//! M9.0 widens that door to **an assembled item**, and a type two systems grant
//! is a type neither of them owns. `skills::Effect::Grants` still names it and
//! `skills` re-exports it, so nothing about `data/skills.json` changes.
//!
//! # The guard, and why it is three things
//!
//! This project has shipped eight nodes that cost a point and did nothing,
//! because serde drops a key it does not know without a word. So a rule is
//! guarded three ways and every one of them is load-bearing:
//!
//! 1. **An enum, and every match on it is exhaustive.** A rule nobody wires up
//!    is a compile error rather than a granted nothing.
//! 2. **`deny_unknown_fields`**, which is a container attribute — the exact
//!    reason it could not be put on `Effect::Stat`, which is what let the
//!    original failure through.
//! 3. **[`Rule::check`]**, run by `SkillsData::parse`, which refuses a rule
//!    naming a grid, a curse or a creature the engine has not got, or a tuning
//!    that tunes nothing.
//!
//! # Why the strings are `Cow`
//!
//! A rule now arrives from two places: parsed out of JSON, where a name has to
//! be owned, and written into `CATALOG`, where it has to be a compile-time
//! constant. `Cow<'static, str>` is the one type that is both, and it costs
//! nothing at either end — `Cow::Borrowed` in a static, `Cow::Owned` from
//! serde. The alternative was a second enum for the catalogue's half, which is
//! two rulebooks and is the thing every rule in `CLAUDE.md` is about.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// A name a rule carries: a slot, a curse or a creature.
///
/// See the module header for why it is a `Cow`.
pub type Name = Cow<'static, str>;

/// What a node — or an assembled item — can grant that is not a number.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Rule {
    /// Every activation of an item in this slot lands a curse on them.
    ///
    /// The slot and the curse are named as strings because both vocabularies
    /// already live elsewhere — `SlotKind` and `CurseKind` — and duplicating
    /// either here would be two lists to keep in step. [`Rule::check`] is what
    /// stops a misspelling reaching a player, and `SkillsData::parse` runs it.
    CurseOnActivate { slot: Name, curse: Name },
    /// Every turn of a spinning item banks this many extra stacks.
    ///
    /// The tree tuning a rule it did not invent, which is what M8.3's plumbing
    /// was for: the spin exists, so the Patent's nodes move its numbers rather
    /// than each inventing a mechanic of its own.
    SpinExtra { per_turn: u32 },
    /// A spinning item keeps this many stacks through an activation instead of
    /// starting again from nothing.
    SpinKeep { stacks: u32 },
    /// A spinning item turns every this many milliseconds instead of every
    /// thousand.
    SpinEvery { ms: u32 },
    /// The danger of a region and the odds on a tile become readable.
    ///
    /// Not a combat rule: nothing in a fight reads it. It gates what the map
    /// screen is allowed to print, which is why it is a rule and not a stat —
    /// there is no number to add.
    Scout,
    /// Meeting this creature is a win that never becomes a fight.
    ///
    /// **Not a combat rule, and it must not become one.** A fight decided
    /// before its first tick is a fight the replay has to draw, and there is
    /// nothing to draw. It is answered where the encounter is settled, by
    /// [`crate::fight::rout`], which is the only place in the game that can
    /// honestly say "nothing was fought" and still pay.
    Rout { creature: Name },
    /// Water one tile from dry land is ground.
    ///
    /// Answered in `world::step`, because a step is where a wall is refused —
    /// and reaching it through [`crate::world::Allowances`] rather than through
    /// the character, because **a map does not know about bags.**
    ///
    /// **All of it, since M11.4.** See `world::WADE_DEPTH` for the measurement
    /// that used to bound it and the reason the bound went.
    Wade,
    /// An assembled instrument is on the board, and it is this one.
    ///
    /// **The third granter of rules is an item again**, and this is the first
    /// rule that is *about* a map rather than about a fight or a step. What
    /// each kind does to the map it opens is M11.6's; what this does is say
    /// which instrument you built, once, in one place — so nothing downstream
    /// has to re-derive it from what is sitting in the weapon grid.
    ///
    /// The kind is a `Name` for the reason every other name here is one: the
    /// vocabulary lives in `piece.rs`'s recipes, and a second enum listing
    /// compass-atlas-golem would be two lists to keep in step. [`Rule::check`]
    /// refuses one the recipes have not got.
    Survey { kind: Name },
    /// Assembled and whole, it takes you back to your last town.
    ///
    /// **The block's one piece of new travel, and it is priced in a tin.** The
    /// walk home is what makes a restorative worth drinking and a town worth
    /// reaching, so a teleport that cost nothing would re-price both — the fare
    /// is one restorative, consumed on departure, and with nothing in the bag
    /// the set refuses and says so.
    ///
    /// **Not a combat rule and not a step rule.** It is the first rule in the
    /// game that is a *gesture*: the player asks for it, on the standing panel,
    /// and `character::go_home` answers. That is why it carries no number —
    /// there is nothing to tune but the fare, and the fare is a tin.
    ///
    /// It may fire from inside the Drambus Stack (`PLAN-M11.md` §8 row 9: the
    /// tower is five entries by design and the kick already moves you) and not
    /// from under the lake — a dungeon you can post yourself out of is not
    /// under a lake.
    Homeward,
}

/// The three instruments, by name, in the order their recipes are written.
///
/// **A list, and it is the only one.** `Rule::check` reads it, `Rule::line`
/// reads it, and M11.6's modifiers read it — so an instrument that is not here
/// is an instrument that does not exist, which is what stops a fourth being
/// half-added.
pub const INSTRUMENTS: &[&str] = &["compass", "atlas", "golem"];

impl Rule {
    /// Refuse a rule that names something the engine has not got.
    ///
    /// The parse-time half of the guard. The compile-time half is that every
    /// match on `Rule` is exhaustive.
    pub fn check(&self) -> Result<(), String> {
        match self {
            Rule::CurseOnActivate { slot, curse } => {
                if crate::skills::slot_of(slot).is_none() {
                    return Err(format!("there is no {slot:?} grid"));
                }
                if crate::curse::CurseKind::by_name(curse).is_none() {
                    return Err(format!("there is no curse of {curse:?}"));
                }
                Ok(())
            }
            // A rule that grants nothing is a node that costs a point and
            // does nothing, which is the exact failure this file exists to
            // stop shipping twice.
            Rule::SpinExtra { per_turn } => {
                (*per_turn > 0).then_some(()).ok_or_else(|| "no extra stacks at all".into())
            }
            Rule::SpinKeep { stacks } => {
                (*stacks > 0).then_some(()).ok_or_else(|| "keeps nothing".into())
            }
            // Slower than a second is not a tuning, it is a downgrade, and
            // zero is a division by nothing.
            Rule::SpinEvery { ms } => (*ms > 0 && *ms < crate::combat::SPIN_EVERY_MS)
                .then_some(())
                .ok_or_else(|| format!("{ms}ms is not faster than a second")),
            Rule::Scout => Ok(()),
            // The same guard `CurseOnActivate` gets, for the same reason: an
            // instrument nobody wrote a recipe for is a rule that can never
            // fire, and nothing else in the game would say so.
            Rule::Survey { kind } => INSTRUMENTS
                .contains(&kind.as_ref())
                .then_some(())
                .ok_or_else(|| format!("there is no instrument called {kind:?}")),
            Rule::Homeward => Ok(()),
            // A creature nothing in the game is called is a set bonus that can
            // never fire, which is the granted-nothing failure wearing a
            // creature's name.
            Rule::Rout { creature } => crate::combat::creature(creature)
                .map(|_| ())
                .ok_or_else(|| format!("nothing in the ladder is called {creature:?}")),
            Rule::Wade => Ok(()),
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
            Rule::SpinExtra { per_turn } => format!(
                "a turning item banks {} more per turn, on top of the 1 it banks anyway",
                per_turn
            ),
            // Short on purpose: this line sits under a node's name and shares
            // the button with a second effect, and
            // `a_mechanical_line_stays_short_enough_to_read_at_a_glance` is
            // what caught the first draft at 121 characters. What it means is
            // the hover's job.
            Rule::SpinKeep { stacks } => {
                format!("a turning item keeps {stacks} of its turns when it goes off")
            }
            Rule::SpinEvery { ms } => format!(
                "a turning item turns every {:.1}s instead of every {:.1}s",
                *ms as f32 / 1000.0,
                crate::combat::SPIN_EVERY_MS as f32 / 1000.0,
            ),
            Rule::Scout => format!(
                "read a region's danger and every tile's odds out of {}, on the map",
                1000,
            ),
            // The number is the one a player will want to check, and it is the
            // surprising one: a fight costs 4% of you whatever happens in it,
            // and this is a win that costs none because nothing was fought.
            Rule::Rout { creature } => format!(
                "every {creature} you meet gives up: paid like a win, and {}% tiring",
                0,
            ),
            Rule::Wade => "walk onto water".to_string(),
            Rule::Survey { kind } => format!("survey a map with 1 {kind}"),
            Rule::Homeward => {
                "go back to your last town from anywhere, for 1 restorative".to_string()
            }
        }
    }

    /// What the words in [`Rule::line`] mean, for the hover.
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
            Rule::SpinExtra { .. } | Rule::SpinKeep { .. } | Rule::SpinEvery { .. } => vec![
                format!(
                    "A turning item banks {}% of its own power a turn and spends the lot \
                     the moment it activates. It only turns where it has room to: an item \
                     packed flush against its neighbours never moves and never banks.",
                    crate::combat::SPIN_PCT_PER_TURN,
                ),
                "This does nothing at all without something on the board that turns.".into(),
            ],
            Rule::Scout => vec![
                format!(
                    "Danger is the mean rating of what a region holds. The odds are \
                     per-mille per step — {} at the very worst, whatever the ground.",
                    crate::world::MAX_ENCOUNTER_PER_MILLE,
                ),
                "Nothing shows either until this is taken.".into(),
            ],
            Rule::Rout { creature } => vec![
                format!(
                    "A {creature} that meets you does not fight: the bounty and the \
                     experience are paid on the spot and the encounter is over."
                ),
                format!(
                    "It costs no tiredness, because a fight is what costs {}% of your \
                     maximum health and there was no fight.",
                    crate::fatigue::PER_FIGHT,
                ),
                "Nothing else is routed. Everything else in that region still fights."
                    .into(),
            ],
            Rule::Wade => vec![
                "Water is impassable to everybody else. This opens all of it, edge to \
                 middle, on every map that has any."
                    .into(),
                "Nothing lives in water, so a waded tile never starts a fight.".into(),
            ],
            Rule::Survey { kind } => vec![
                format!(
                    "An assembled {kind} in the weapon grid. A surveyable map reads \
                     differently through it — see what the reach says when you get there."
                ),
                "The weapon grid holds gear or an instrument and never both.".into(),
            ],
            Rule::Homeward => vec![
                "Asked for on the standing panel, and answered wherever you are \
                 standing except under the lake."
                    .into(),
                "One restorative, spent on departure. With nothing in the bag it \
                 refuses, because the fare is the whole of what makes it a decision."
                    .into(),
            ],
        }
    }
}

/// Does this set of rules rout that creature?
///
/// A free function over a rule list rather than a method on a character,
/// because both callers already have the list and neither should have to build
/// a character to ask. Matched on the **canonical** name, like a `Slay` goal
/// and like a drop, because that is what the engine matches on everywhere.
pub fn routs(rules: &[Rule], creature: &str) -> bool {
    rules.iter().any(|r| matches!(r, Rule::Rout { creature: c } if c == creature))
}
