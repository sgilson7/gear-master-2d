//! Can a floor be solved by somebody who does not know the answer?
//!
//! **Not a game rule.** Nothing in a fight or on a map reads this module; it is
//! a harness, and it is in core rather than in `tests/` for the reason
//! `pressure.rs` is: *a number a design stakes itself on that is worked out in
//! Python by the thing measuring it is the page recomputing a total, one level
//! up.* `PLAN-M14.md` §1.2 stakes a design on this number for six floors.
//!
//! # The two properties, and they are different
//!
//! **Monotone.** `WorldState::flags` and `answered` only ever grow, so a puzzle
//! whose wrong move locks the right one is a puzzle the save cannot come back
//! from. That is not something this module has to check, it is something the
//! engine cannot do — and it is why every puzzle in M14 is solved by
//! *discovering* something rather than by *avoiding* something.
//!
//! **Bounded.** A floor only a person can solve is a floor `make play` cannot
//! walk, and a walker that stops at floor two is a gate that never sees floor
//! four. So every floor has a blind solution, and this counts it.
//!
//! # What a "visit" is, and why the model is a sweep
//!
//! A player standing on a floor can *see* which of a card's choices are
//! refused — the card draws them greyed with the `unmet` line under them. What
//! they cannot see is **which tile to walk to next**. So the expensive thing on
//! a floor is not guessing a button, it is crossing the room, and a visit is
//! one card read.
//!
//! The model is therefore a sweep against an adversary who arranges the floor
//! as badly as it can be arranged:
//!
//! 1. Everything that could still raise a flag is *remaining*.
//! 2. The solver walks all of them; the one that opens is visited **last**, so
//!    a round costs one visit per remaining event.
//! 3. Whichever open choice is taken, the adversary picks the worst one.
//!
//! Nine cairns is `9 + 8 + … + 1`, which is **45** — the number `PLAN-M14.md`
//! §1.2 builds its ceiling out of, reproduced by the model rather than assumed
//! by it. That agreement is the reason to trust the other five floors' numbers.
//!
//! # What the fixture is for
//!
//! Three of the six doors want something off the board — a loose component of a
//! footprint, an assembled item of a rarity, a component in hand. **Having the
//! thing is the player's problem and finding the door that wants it is the
//! puzzle's**, so [`Carrying`] hands the solver one of each and the count is
//! about the floor rather than about a character.
//!
//! A `GiveUp` **spends** one, which is why the fixture counts rather than
//! answering yes: a floor with two doors wanting the same footprint costs two
//! components, and a model that could not tell would call it free.

use std::collections::{BTreeMap, BTreeSet};

use crate::tile_event::{EventsData, Outcome, Requirement};
use crate::world::{PlaceDef, World};

/// The most rounds a floor may take before this gives up on it.
///
/// A bound rather than a proof of termination: flags are monotone so a sweep
/// always terminates, but a floor with forty events and a long chain would take
/// a long time to say so, and a harness that hangs is a harness nobody runs.
const MAX_ROUNDS: usize = 64;

/// Why a floor could not be solved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unsolvable {
    /// Everything that could be opened has been, and the way on is still shut.
    /// **The failure this module exists for.**
    Stuck { still_hidden: Vec<String>, raised: Vec<String> },
    /// It went on longer than [`MAX_ROUNDS`], which is a floor nobody would
    /// walk even if it does finish.
    TooLong,
}

impl std::fmt::Display for Unsolvable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unsolvable::Stuck { still_hidden, raised } => write!(
                f,
                "nothing else on this floor can be opened, and {} is still not there \
                 (raised: {})",
                still_hidden.join(", "),
                if raised.is_empty() { "nothing".to_string() } else { raised.join(", ") }
            ),
            Unsolvable::TooLong => write!(f, "it takes more than {MAX_ROUNDS} sweeps of the floor"),
        }
    }
}

/// What the imaginary solver walked in with.
///
/// **One of each thing the floor asks for**, built by [`Carrying::for_floor`].
/// It is deliberately not a `Character`: a character has a board, and a board
/// is what the *other* half of this game is about. What is being measured here
/// is the floor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Carrying {
    /// Loose components by footprint, normalised so `(3, 2)` and `(2, 3)` are
    /// one key — a slot does not care which way up a rectangle goes in.
    pub of_size: BTreeMap<(u8, u8), usize>,
    /// Components in hand, by canonical name.
    pub holding: BTreeSet<String>,
    /// The best assembled item on the board, if the solver has one.
    pub best_rarity: Option<crate::rating::Rarity>,
    /// The instrument on the frame, if any. `None` is the blind read.
    pub instrument: Option<String>,
    pub gold: i32,
}

fn key(w: u8, h: u8) -> (u8, u8) {
    if w <= h { (w, h) } else { (h, w) }
}

impl Carrying {
    /// One of everything this floor's doors ask for, and nothing else.
    ///
    /// **Derived from the floor rather than written down**, so a door that
    /// starts wanting something new is a door the harness already knows about.
    /// The one thing it never supplies is an instrument: that is the whole
    /// question §1.2 asks, and handing one over would answer it.
    pub fn for_floor(world: &World, events: &EventsData) -> Self {
        let mut out = Carrying { gold: 100_000, ..Carrying::default() };
        for c in choices_on(world, events) {
            match &c.requires {
                Requirement::LooseItemOfSize { w, h } => {
                    *out.of_size.entry(key(*w, *h)).or_insert(0) += 1;
                }
                Requirement::Holding(name) => {
                    out.holding.insert(name.clone());
                }
                Requirement::AssembledOfRarity(r) => {
                    let want = crate::rating::Rarity::by_name(r);
                    if want > out.best_rarity {
                        out.best_rarity = want;
                    }
                }
                Requirement::None | Requirement::Gold(_) | Requirement::Flag(_) => {}
                // Never supplied. See the doc above.
                Requirement::Surveying(_) => {}
            }
        }
        out
    }

    /// The same, reading the floor through one instrument.
    pub fn with(mut self, instrument: &str) -> Self {
        self.instrument = Some(instrument.to_string());
        self
    }
}

/// Every choice on every event standing on this map.
fn choices_on<'a>(
    world: &World,
    events: &'a EventsData,
) -> Vec<&'a crate::tile_event::Choice> {
    events_on(world, events).into_iter().flat_map(|e| e.choices.iter()).collect()
}

/// Every event standing on this map, in the order the file places them.
fn events_on<'a>(
    world: &World,
    events: &'a EventsData,
) -> Vec<&'a crate::tile_event::TileEvent> {
    world
        .places
        .iter()
        .filter(|p| p.kind == crate::world::PlaceKind::Event)
        .filter_map(|p| events.get(&p.id))
        .collect()
}

/// Every place on this map that is not there until something has happened.
///
/// **The goal is all of them.** A floor has one way down and it is hidden until
/// the puzzle is solved; a floor that grew a second hidden place would be a
/// floor whose puzzle is now two puzzles, and the harness noticing is better
/// than the harness assuming.
fn hidden_places(world: &World) -> Vec<&PlaceDef> {
    world
        .places
        .iter()
        .filter(|p| p.hidden_until.is_some() || !p.hidden_until_all.is_empty())
        .collect()
}

/// One position in the sweep.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    flags: BTreeSet<String>,
    answered: BTreeSet<String>,
    of_size: BTreeMap<(u8, u8), usize>,
}

impl Position {
    fn met(&self, k: &str) -> bool {
        self.flags.contains(k) || self.answered.contains(k)
    }

    fn there(&self, p: &PlaceDef) -> bool {
        p.hidden_until.as_ref().is_none_or(|k| self.met(k))
            && p.hidden_until_all.iter().all(|k| self.met(k))
    }
}

/// Every flag this outcome raises, however deep it is nested.
fn flags_of(o: &Outcome, out: &mut Vec<String>) {
    match o {
        Outcome::Flag(f) => out.push(f.clone()),
        Outcome::All(list) => {
            for x in list {
                flags_of(x, out);
            }
        }
        _ => {}
    }
}

/// What this outcome spends off the tray, if anything.
fn spends(o: &Outcome, out: &mut Vec<(u8, u8)>) {
    match o {
        Outcome::GiveUp { w, h } => out.push(key(*w, *h)),
        Outcome::All(list) => {
            for x in list {
                spends(x, out);
            }
        }
        _ => {}
    }
}

fn takeable(r: &Requirement, pos: &Position, kit: &Carrying) -> bool {
    match r {
        Requirement::None => true,
        Requirement::Gold(n) => kit.gold >= *n,
        Requirement::Flag(f) => pos.met(f),
        Requirement::Holding(name) => kit.holding.contains(name),
        Requirement::LooseItemOfSize { w, h } => {
            pos.of_size.get(&key(*w, *h)).copied().unwrap_or(0) > 0
        }
        Requirement::AssembledOfRarity(r) => crate::rating::Rarity::by_name(r)
            .is_some_and(|want| kit.best_rarity.is_some_and(|have| have >= want)),
        Requirement::Surveying(kind) => kit.instrument.as_deref() == Some(kind.as_str()),
    }
}

/// The worst-case number of card reads a blind solver needs on this floor.
///
/// `Ok(n)` is *at most n*, over every way the floor could be arranged against
/// the solver. `Err` is the thing this exists to catch.
///
/// The count is asserted **at its number** wherever it is checked, never as
/// `< 100`: a ceiling nobody can hit is a ceiling nobody checked, which is the
/// `(2..5).contains(&taken)` failure M11.7 lost a block to.
pub fn solvable_blind(world: &World, events: &EventsData) -> Result<usize, Unsolvable> {
    let kit = Carrying::for_floor(world, events);
    walk(world, events, &kit)
}

/// The same floor read through an instrument.
///
/// **The shortest honest path rather than the worst one**, because an
/// instrument is information: it tells you which tile to walk to, so the sweep
/// collapses to the solution itself. That is what `PLAN-M14.md` §1.2 means by
/// *an instrument makes a floor short, never possible* — a number worth
/// printing beside the blind one.
pub fn solvable_knowing(
    world: &World,
    events: &EventsData,
    instrument: &str,
) -> Result<usize, Unsolvable> {
    let kit = Carrying::for_floor(world, events).with(instrument);
    shortest(world, events, &kit)
}

/// The blind sweep. See the module header for the model.
fn walk(world: &World, events: &EventsData, kit: &Carrying) -> Result<usize, Unsolvable> {
    let goal = hidden_places(world);
    let all = events_on(world, events);
    let start = Position {
        flags: BTreeSet::new(),
        answered: BTreeSet::new(),
        of_size: kit.of_size.clone(),
    };
    let mut seen: BTreeMap<Position, Option<usize>> = BTreeMap::new();
    worst(&start, &goal, &all, kit, 0, &mut seen)
}

fn worst(
    pos: &Position,
    goal: &[&PlaceDef],
    all: &[&crate::tile_event::TileEvent],
    kit: &Carrying,
    depth: usize,
    seen: &mut BTreeMap<Position, Option<usize>>,
) -> Result<usize, Unsolvable> {
    if goal.iter().all(|p| pos.there(p)) {
        return Ok(0);
    }
    if depth >= MAX_ROUNDS {
        return Err(Unsolvable::TooLong);
    }
    if let Some(cached) = seen.get(pos) {
        return match cached {
            Some(n) => Ok(*n),
            None => Err(stuck(pos, goal)),
        };
    }

    // **Remaining** is everything that could still put a flag up. A card whose
    // every flag is already raised is a card nobody walks back to, and one
    // already answered is a card that will not open again — unless it repeats,
    // which is exactly what a chair in an empty room is.
    let mut remaining = 0usize;
    // (event index, choice index) pairs that would move the floor on.
    let mut progress: Vec<(usize, usize)> = Vec::new();
    for (ei, e) in all.iter().enumerate() {
        if !e.repeats && pos.answered.contains(&e.id) {
            continue;
        }
        let mut live = false;
        for (ci, c) in e.choices.iter().enumerate() {
            // **A choice raising only flags that are already up is a button
            // that does nothing.** It is not what brings a solver back across
            // the room, so it is not what makes a card *remaining* — and
            // counting it would make a floor with one shrug on it sweep for
            // ever.
            let mut raises = Vec::new();
            flags_of(&c.outcome, &mut raises);
            if !raises.iter().any(|f| !pos.met(f)) {
                continue;
            }
            live = true;
            if takeable(&c.requires, pos, kit) {
                progress.push((ei, ci));
            }
        }
        if live {
            remaining += 1;
        }
    }

    if progress.is_empty() {
        seen.insert(pos.clone(), None);
        return Err(stuck(pos, goal));
    }

    // **The one that opens is visited last**, which is the whole of the
    // adversary: a round costs one card read for every card still worth
    // reading.
    let mut best: Option<usize> = None;
    for (ei, ci) in progress {
        let next = after(pos, all[ei], ci);
        let cost = worst(&next, goal, all, kit, depth + 1, seen)?;
        let total = remaining + cost;
        best = Some(best.map_or(total, |b: usize| b.max(total)));
    }
    let answer = best.expect("progress is not empty");
    seen.insert(pos.clone(), Some(answer));
    Ok(answer)
}

/// The shortest number of choices taken, for a solver who knows where to walk.
fn shortest(world: &World, events: &EventsData, kit: &Carrying) -> Result<usize, Unsolvable> {
    let goal = hidden_places(world);
    let all = events_on(world, events);
    let start = Position {
        flags: BTreeSet::new(),
        answered: BTreeSet::new(),
        of_size: kit.of_size.clone(),
    };
    // Breadth-first over positions, so the first time the goal is met is the
    // fewest moves it can be met in.
    let mut frontier = vec![start.clone()];
    let mut seen: BTreeSet<Position> = BTreeSet::new();
    seen.insert(start);
    for taken in 0..MAX_ROUNDS {
        if frontier.iter().any(|p| goal.iter().all(|g| p.there(g))) {
            return Ok(taken);
        }
        let mut next = Vec::new();
        for pos in &frontier {
            for e in &all {
                if !e.repeats && pos.answered.contains(&e.id) {
                    continue;
                }
                for (ci, c) in e.choices.iter().enumerate() {
                    if !takeable(&c.requires, pos, kit) {
                        continue;
                    }
                    let p = after(pos, e, ci);
                    if seen.insert(p.clone()) {
                        next.push(p);
                    }
                }
            }
        }
        if next.is_empty() {
            return Err(stuck(&frontier[0], &goal));
        }
        frontier = next;
    }
    Err(Unsolvable::TooLong)
}

fn after(pos: &Position, e: &crate::tile_event::TileEvent, ci: usize) -> Position {
    let mut out = pos.clone();
    let c = &e.choices[ci];
    let mut raises = Vec::new();
    flags_of(&c.outcome, &mut raises);
    for f in raises {
        out.flags.insert(f);
    }
    let mut spent = Vec::new();
    spends(&c.outcome, &mut spent);
    for s in spent {
        if let Some(n) = out.of_size.get_mut(&s) {
            *n = n.saturating_sub(1);
        }
    }
    if !e.repeats {
        out.answered.insert(e.id.clone());
    }
    out
}

fn stuck(pos: &Position, goal: &[&PlaceDef]) -> Unsolvable {
    Unsolvable::Stuck {
        still_hidden: goal
            .iter()
            .filter(|p| !pos.there(p))
            .map(|p| p.id.clone())
            .collect(),
        raised: pos.flags.iter().cloned().collect(),
    }
}
