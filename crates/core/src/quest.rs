//! Errands a town gives you, and what finishing one pays.
//!
//! **A quest is content; taking one and finishing it is state.** The errands
//! are `data/quests.json` and the save carries three short lists — taken, done,
//! and what has been counted so far. Same division as the map and the shelves,
//! and it buys the same thing: an errand can be retuned without touching
//! anybody's file.
//!
//! This is not upstream's `quest.rs`, which went with the campaign in `48203ee`
//! and was a different idea — a chain of receipts along a road. It is also not
//! `piece::Quest`, which is a *component* that transforms after it has gone off
//! enough times. Three things called quest, and they are three different
//! things; this one is the only one a town hands out.
//!
//! The tally is deliberately a **bag item and not a counter**. Killing a toad
//! gives you a Toad Eye, and the eyes sit in your bag taking up nothing until
//! you carry them back. A counter would be simpler and would also mean the
//! errand had no middle: with eyes there is a walk home holding something.

use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "gm2d-quests";
pub const VERSION: u32 = 1;

/// What finishing an errand asks for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
    /// Beat `count` of a creature; each win drops one `token`, and the tokens
    /// are what is handed in.
    Slay {
        /// The creature's **canonical** name, as `enemies.json` spells it. The
        /// theme is what a player reads; this is what the engine matches.
        creature: String,
        count: u32,
        /// The canonical name of the catalogue piece each win drops.
        token: String,
    },
}

impl Goal {
    /// The creature this errand is about, if it is about one.
    pub fn creature(&self) -> Option<&str> {
        match self {
            Goal::Slay { creature, .. } => Some(creature),
        }
    }

    pub fn token(&self) -> Option<&str> {
        match self {
            Goal::Slay { token, .. } => Some(token),
        }
    }

    pub fn count(&self) -> u32 {
        match self {
            Goal::Slay { count, .. } => *count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    /// The place id that gives it out and takes it back.
    pub town: String,
    pub name: String,
    /// Why somebody is asking. The world's words.
    pub brief: String,
    /// What the town says when you bring it back.
    pub thanks: String,
    pub goal: Goal,
    /// Canonical catalogue names handed over on finishing.
    #[serde(default)]
    pub reward: Vec<String>,
    #[serde(default)]
    pub gold: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestsData {
    pub format: String,
    pub version: u32,
    pub quests: Vec<Quest>,
}

impl QuestsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: QuestsData =
            serde_json::from_str(text).map_err(|e| format!("quests.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these errands are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        // Everything an errand names has to exist, checked once here rather
        // than discovered by a player who cannot hand one in.
        for q in &d.quests {
            if let Some(c) = q.goal.creature() {
                if !crate::combat::LADDER.iter().any(|s| s.name == c) {
                    return Err(format!("{}: no creature called {c:?}", q.id));
                }
            }
            for name in q.goal.token().into_iter().chain(q.reward.iter().map(|s| s.as_str())) {
                if crate::shop::def_named(name).is_none() {
                    return Err(format!("{}: {name:?} is not in the catalogue", q.id));
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&Quest> {
        self.quests.iter().find(|q| q.id == id)
    }

    /// The errands a given town hands out.
    pub fn at(&self, town: &str) -> Vec<&Quest> {
        self.quests.iter().filter(|q| q.town == town).collect()
    }
}

/// Where an errand stands for this character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// On the board and not yet taken.
    Offered,
    /// Taken, and short of what it asked for.
    Carrying { have: u32, want: u32 },
    /// Taken, and everything it asked for is in the bag.
    Ready,
    Done,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Offered => "offered",
            Stage::Carrying { .. } => "carrying",
            Stage::Ready => "ready",
            Stage::Done => "done",
        }
    }
}

// ---------------------------------------------------------------- the rules
//
// Four verbs and nothing else: where an errand stands, taking one, what a win
// drops, and handing one in. All of it here rather than in the page, for the
// reason every rule is: a rule the test suite cannot reach in seconds is a
// rule with two rulebooks.

use crate::game::Game;

/// How many of a named component the character is holding, seated or not.
pub fn holding(game: &Game, name: &str) -> u32 {
    game.character
        .owned
        .iter()
        .filter(|&&id| game.character.registry.def(id).name == name)
        .count() as u32
}

/// Where this errand stands right now.
pub fn stage(game: &Game, q: &Quest) -> Stage {
    if game.world.quests_done.iter().any(|d| *d == q.id) {
        return Stage::Done;
    }
    if !game.world.quests_taken.iter().any(|t| *t == q.id) {
        return Stage::Offered;
    }
    let want = q.goal.count();
    let have = q.goal.token().map(|t| holding(game, t)).unwrap_or(0);
    if have >= want {
        Stage::Ready
    } else {
        Stage::Carrying { have, want }
    }
}

/// Take an errand on. Returns why not.
pub fn take(game: &mut Game, id: &str) -> Result<(), String> {
    let quests = crate::data::quests();
    let Some(q) = quests.get(id) else { return Err("there is no such errand".into()) };
    match stage(game, q) {
        Stage::Done => Err("you have already done that one".into()),
        Stage::Offered => {
            game.world.quests_taken.push(q.id.clone());
            Ok(())
        }
        _ => Err("you are already carrying that one".into()),
    }
}

/// What a win drops, if anything.
///
/// Called from `fight::settle` on a victory and nowhere else. Gated on the
/// errand being **taken and unfinished**, which is deliberate: a bag filling
/// with toad eyes before anybody asked for one is litter, and a bag still
/// filling after the fifth is worse — you would hand in five and walk out
/// holding four more that mean nothing.
pub fn on_victory(game: &mut Game, creature: &str) -> Vec<String> {
    let quests = crate::data::quests();
    let mut dropped = Vec::new();
    for q in &quests.quests {
        if q.goal.creature() != Some(creature) {
            continue;
        }
        let Some(token) = q.goal.token() else { continue };
        if !matches!(stage(game, q), Stage::Carrying { .. }) {
            continue;
        }
        if game.character.give(token).is_some() {
            dropped.push(token.to_string());
        }
    }
    dropped
}

/// Hand one in. Returns what was handed over, or why not.
pub fn hand_in(game: &mut Game, id: &str) -> Result<Vec<String>, String> {
    let quests = crate::data::quests();
    let Some(q) = quests.get(id) else { return Err("there is no such errand".into()) };
    match stage(game, q) {
        Stage::Offered => return Err("you have not taken that one".into()),
        Stage::Done => return Err("you have already handed that in".into()),
        Stage::Carrying { have, want } => {
            return Err(format!(
                "{have} of {want}. She will not write down a thing she has not been handed."
            ));
        }
        Stage::Ready => {}
    }

    // The tokens go over the counter. Taken off the board first if any of them
    // are seated — a component that is handed in and still occupying a cell is
    // a component in two places.
    if let Some(token) = q.goal.token() {
        for _ in 0..q.goal.count() {
            let Some(id) = game
                .character
                .owned
                .iter()
                .copied()
                .find(|&p| game.character.registry.def(p).name == token)
            else {
                break;
            };
            game.character.loadout.remove_anywhere(id);
            game.character.owned.retain(|&p| p != id);
        }
    }

    let mut given = Vec::new();
    for name in &q.reward {
        if game.character.give(name).is_some() {
            given.push(name.clone());
        }
    }
    if q.gold != 0 {
        game.character.gold += q.gold;
    }
    game.world.quests_taken.retain(|t| *t != q.id);
    game.world.quests_done.push(q.id.clone());
    Ok(given)
}
