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
    /// Carry `count` of a component to whoever is asking.
    ///
    /// No token and no drop: what it wants is a thing that already exists in
    /// the world, so the errand is a reason to go and find one rather than a
    /// counter that fills itself.
    Bring { item: String, count: u32 },
    /// Go and speak to somebody, then come back and say so.
    ///
    /// The one errand with nothing in the bag at the end of it. Standing on
    /// the tile is the whole of the doing; `answered` is where that is
    /// recorded, which is the same set a tile-event writes to, so a word and a
    /// door are remembered the same way.
    Word { place: String },
}

impl Goal {
    /// The creature this errand is about, if it is about one.
    pub fn creature(&self) -> Option<&str> {
        match self {
            Goal::Slay { creature, .. } => Some(creature),
            _ => None,
        }
    }

    /// The component it tallies, if it tallies one.
    pub fn token(&self) -> Option<&str> {
        match self {
            Goal::Slay { token, .. } => Some(token),
            Goal::Bring { item, .. } => Some(item),
            Goal::Word { .. } => None,
        }
    }

    /// Whether the token is one the errand *hands out* as you go.
    ///
    /// A slain toad drops its eye; a component somebody wants was already in
    /// the world. The difference decides whether `on_victory` pays.
    pub fn drops_its_own(&self) -> bool {
        matches!(self, Goal::Slay { .. })
    }

    pub fn count(&self) -> u32 {
        match self {
            Goal::Slay { count, .. } => *count,
            Goal::Bring { count, .. } => *count,
            Goal::Word { .. } => 1,
        }
    }

    /// The place it sends you to, if it sends you anywhere.
    pub fn place(&self) -> Option<&str> {
        match self {
            Goal::Word { place } => Some(place),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    /// The place that offers it — a town or an event tile.
    ///
    /// An errand is not a town's any more. Somebody standing in a field with a
    /// bread knife can ask you for something, and the difference between that
    /// and a clerk behind a counter should be where they are standing rather
    /// than which system they are in.
    pub giver: String,
    /// Where it is handed in. The giver, unless it says otherwise — which is
    /// what makes "go and tell them in town" an errand rather than two.
    #[serde(default)]
    pub turn_in: Option<String>,
    /// Errands that must be done first. A questline is this and nothing else.
    #[serde(default)]
    pub requires: Vec<String>,
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
            // A reward is always a component; a tally may be a component or a
            // restorative, because `Bring` names either.
            for name in &q.reward {
                if crate::shop::def_named(name).is_none() {
                    return Err(format!("{}: {name:?} is not in the catalogue", q.id));
                }
            }
            if let Some(t) = q.goal.token() {
                let known = crate::shop::def_named(t).is_some()
                    || crate::data::supplies().get(t).is_some();
                if !known {
                    return Err(format!("{}: {t:?} is neither a component nor a supply", q.id));
                }
            }
            if let Some(p) = q.goal.place() {
                if p == q.giver {
                    return Err(format!("{}: sends you to the person asking", q.id));
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&Quest> {
        self.quests.iter().find(|q| q.id == id)
    }

    /// Where an errand is handed in — the giver unless it says otherwise.
    pub fn turn_in_of(q: &Quest) -> &str {
        q.turn_in.as_deref().unwrap_or(&q.giver)
    }

    /// The errands a place is concerned with: the ones it gives out, and the
    /// ones somebody else sent you here to report.
    pub fn at(&self, place: &str) -> Vec<&Quest> {
        self.quests
            .iter()
            .filter(|q| q.giver == place || Self::turn_in_of(q) == place)
            .collect()
    }
}

/// Where an errand stands for this character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Behind an errand that has not been done.
    Locked,
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
            Stage::Locked => "locked",
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

/// How many of a named thing the character is holding.
///
/// **Components or restoratives**, because an errand asking for four of
/// something does not care which drawer of the game they live in — and the
/// alternative was a second goal kind that asked the same question of a
/// different list.
pub fn holding(game: &Game, name: &str) -> u32 {
    let gear = game
        .character
        .owned
        .iter()
        .filter(|&&id| game.character.registry.def(id).name == name)
        .count() as u32;
    gear + game.character.supply_count(name)
}

/// Where this errand stands right now.
/// Has this errand been finished?
pub fn done(game: &Game, id: &str) -> bool {
    game.world.quests_done.iter().any(|d| d == id)
}

/// The word-of-mouth marker: standing on the place an errand sent you to.
///
/// Written into `world.answered`, the same set a tile-event writes to, so a
/// word and a door are remembered the same way and a save carries one field
/// rather than two.
pub fn spoken(id: &str) -> String {
    format!("word:{id}")
}

pub fn stage(game: &Game, q: &Quest) -> Stage {
    if done(game, &q.id) {
        return Stage::Done;
    }
    if !q.requires.iter().all(|r| done(game, r)) {
        return Stage::Locked;
    }
    if !game.world.quests_taken.iter().any(|t| *t == q.id) {
        return Stage::Offered;
    }
    match &q.goal {
        Goal::Word { .. } => {
            if game.world.answered.iter().any(|a| *a == spoken(&q.id)) {
                Stage::Ready
            } else {
                Stage::Carrying { have: 0, want: 1 }
            }
        }
        _ => {
            let want = q.goal.count();
            let have = q.goal.token().map(|t| holding(game, t)).unwrap_or(0);
            if have >= want {
                Stage::Ready
            } else {
                Stage::Carrying { have, want }
            }
        }
    }
}

/// Standing somewhere: mark any errand that sent you here as spoken to.
///
/// Returns the errands that just advanced, so the page can say something.
/// Called on arriving at a place rather than on opening a screen, because the
/// errand is "go and talk to them" and the talking is the arriving.
pub fn on_arrival(game: &mut Game, place: &str) -> Vec<String> {
    let quests = crate::data::quests();
    let mut moved = Vec::new();
    for q in &quests.quests {
        if q.goal.place() != Some(place) {
            continue;
        }
        if !matches!(stage(game, q), Stage::Carrying { .. }) {
            continue;
        }
        let mark = spoken(&q.id);
        if !game.world.answered.iter().any(|a| *a == mark) {
            game.world.answered.push(mark);
            moved.push(q.name.clone());
        }
    }
    moved
}

/// Where an errand is asking you to go next, in ids a map can find.
///
/// **A rule, not a drawing decision.** Where to go depends on the errand's
/// stage and its goal — a slaying points at the regions the creature lives in,
/// a word points at the tile you have to stand on, and every errand whose
/// tally is full points at whoever takes it back. A page working that out for
/// itself would be a second copy of the errand rules, and it would disagree
/// with this one the first time a goal kind was added.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Guide {
    /// Place ids to ring.
    pub places: Vec<String>,
    /// Region ids to pulse.
    pub regions: Vec<String>,
}

impl Guide {
    pub fn is_empty(&self) -> bool {
        self.places.is_empty() && self.regions.is_empty()
    }
}

/// Where `q` points, given every map this build ships.
///
/// Takes the worlds rather than loading them, because loading a map wants a
/// difficulty and this question does not: which regions hold a creature is a
/// property of the pools, and the pools are the same at every difficulty.
pub fn guide(game: &Game, q: &Quest, worlds: &[crate::world::World]) -> Guide {
    let mut out = Guide::default();
    match stage(game, q) {
        Stage::Done | Stage::Locked => {}
        // Not taken yet: go and be asked.
        Stage::Offered => out.places.push(q.giver.clone()),
        // Everything it wanted is in the bag. One answer, and it is the same
        // one for all three goal kinds — which is why it is not inside the
        // match below.
        Stage::Ready => out.places.push(QuestsData::turn_in_of(q).to_string()),
        Stage::Carrying { .. } => match &q.goal {
            Goal::Word { place } => out.places.push(place.clone()),
            Goal::Slay { creature, .. } => {
                for w in worlds {
                    for r in w.regions_holding(creature) {
                        if !out.regions.iter().any(|x| *x == r.id) {
                            out.regions.push(r.id.clone());
                        }
                    }
                }
            }
            // A component points at the shelves that stock it; a restorative
            // points at every shelf, because every town sells tins — a place
            // that had run out of the only thing that undoes tiredness would
            // be a place you could strand yourself at, and that rule is what
            // makes this one true.
            Goal::Bring { item, .. } => {
                let shops = crate::data::shops();
                let is_supply = crate::data::supplies().get(item).is_some();
                for t in &shops.towns {
                    let stocks = is_supply || t.stock.iter().any(|n| n == item);
                    let placed = worlds.iter().any(|w| w.places.iter().any(|p| p.id == t.id));
                    if stocks && placed && !out.places.iter().any(|x| *x == t.id) {
                        out.places.push(t.id.clone());
                    }
                }
            }
        },
    }
    out
}

/// Pin an errand, or unpin it by naming the one already pinned.
///
/// One at a time. Two pins is a map with two answers to "where now", and the
/// whole point of a pin is that it is the answer.
pub fn pin(game: &mut Game, id: &str) -> Result<bool, String> {
    if game.world.pinned.as_deref() == Some(id) {
        game.world.pinned = None;
        return Ok(false);
    }
    let quests = crate::data::quests();
    let Some(q) = quests.get(id) else { return Err("there is no such errand".into()) };
    match stage(game, q) {
        Stage::Done => Err("that one is finished".into()),
        Stage::Locked => Err("there is something else they want doing first".into()),
        _ => {
            game.world.pinned = Some(q.id.clone());
            Ok(true)
        }
    }
}

/// Drop a pin that no longer points anywhere.
///
/// Called wherever an errand can finish. A pin on a done errand would ring a
/// place with nothing at it, which is worse than no pin at all.
pub fn tidy_pin(game: &mut Game) {
    if let Some(id) = game.world.pinned.clone() {
        if done(game, &id) {
            game.world.pinned = None;
        }
    }
}

/// Take an errand on. Returns why not.
pub fn take(game: &mut Game, id: &str) -> Result<(), String> {
    let quests = crate::data::quests();
    let Some(q) = quests.get(id) else { return Err("there is no such errand".into()) };
    match stage(game, q) {
        Stage::Done => Err("you have already done that one".into()),
        Stage::Locked => Err("there is something else they want doing first".into()),
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
        // Only an errand that hands out its own tally pays here. A errand
        // asking for a component that already exists in the world is a reason
        // to go and find one, not a counter that fills itself.
        if !q.goal.drops_its_own() {
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
        Stage::Locked => return Err("there is something else they want doing first".into()),
        Stage::Offered => return Err("you have not taken that one".into()),
        Stage::Done => return Err("you have already handed that in".into()),
        Stage::Carrying { have, want } => {
            return Err(match &q.goal {
                Goal::Word { .. } => "You have not been yet.".to_string(),
                _ => format!("{have} of {want}, and nobody writes down what they are not handed."),
            });
        }
        Stage::Ready => {}
    }

    // The tokens go over the counter. Taken off the board first if any of them
    // are seated — a component that is handed in and still occupying a cell is
    // a component in two places.
    if let Some(token) = q.goal.token() {
        for _ in 0..q.goal.count() {
            let seated = game
                .character
                .owned
                .iter()
                .copied()
                .find(|&p| game.character.registry.def(p).name == token);
            match seated {
                Some(id) => {
                    // Off the board first. A component handed over the counter
                    // and still occupying a cell is a component in two places.
                    game.character.loadout.remove_anywhere(id);
                    game.character.owned.retain(|&p| p != id);
                    // Anything bolted to it comes back to the rack rather
                    // than going over the counter with it.
                    game.character.tidy_enchs();
                }
                None => {
                    // A restorative, then. Spent the same way it would be if it
                    // were drunk, minus the drinking.
                    for (s, n) in game.character.supplies.iter_mut() {
                        if s == token && *n > 0 {
                            *n -= 1;
                            break;
                        }
                    }
                    game.character.supplies.retain(|(_, n)| *n > 0);
                }
            }
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
    tidy_pin(game);
    Ok(given)
}
