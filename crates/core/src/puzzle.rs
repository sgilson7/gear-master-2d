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
//! # A card you cannot walk to is not a card
//!
//! **The first version counted every event on the floor whether or not the
//! solver could get to it**, which measures the flag chain and not the floor.
//! On the Lip that is the difference between eight and ten, and on a floor
//! drawn slightly worse it is the difference between *solvable* and *the wheel
//! that drains the channel is on the far side of the channel* — which is the
//! classic way to draw a room nobody can leave, and the whole reason this
//! module exists.
//!
//! So a round visits the events the solver can **reach**, on the map as the
//! flags so far have left it: the world is re-drained at every position and
//! flooded from the arrival tile. A floor with no drains answers the same
//! question once and costs nothing.
//!
//! # A card that wants something this floor cannot give is read once
//!
//! The Cairnfield's slab wants a golem to stand on it, or the nine heights off
//! a clipboard two hundred paces away on the shore. A solver with neither reads
//! it, learns that, and does not walk back to it every round — so counting it
//! as *remaining* nine times over made the field 54 card reads where
//! `PLAN-M14.md` §1.2 says 45.
//!
//! **A choice is ever-possible when this floor could satisfy it**: no
//! requirement, a flag something on this floor raises, or something the kit
//! either has or does not and never will. A card with no ever-possible choice
//! is scenery, and the model counts it the way a player would.
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

use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
                // A list asks for what its members ask for, so the fixture
                // walks into it.
                Requirement::All(list) => {
                    for r in list {
                        match r {
                            Requirement::LooseItemOfSize { w, h } => {
                                *out.of_size.entry(key(*w, *h)).or_insert(0) += 1;
                            }
                            Requirement::Holding(name) => {
                                out.holding.insert(name.clone());
                            }
                            _ => {}
                        }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

/// Every flag some choice on this floor raises.
fn raisable_here(all: &[&crate::tile_event::TileEvent]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for e in all {
        for c in &e.choices {
            let mut fs = Vec::new();
            flags_of(&c.outcome, &mut fs);
            out.extend(fs);
        }
    }
    out
}

/// Could this floor ever satisfy this requirement, for this solver?
///
/// A flag raised somewhere else in the world is a flag this floor cannot hand
/// you, which is exactly what the Cairnfield's slab is about: it wants a golem
/// or a clipboard from the shore, and a solver with neither reads it once.
fn ever_possible(r: &Requirement, raisable: &BTreeSet<String>, kit: &Carrying) -> bool {
    match r {
        Requirement::Flag(f) => raisable.contains(f),
        Requirement::All(list) => list.iter().all(|r| ever_possible(r, raisable, kit)),
        // Everything else is a fact about the solver, and a fact about the
        // solver does not change while they are on one floor.
        other => takeable(other, &Position::default(), kit),
    }
}

fn takeable(r: &Requirement, pos: &Position, kit: &Carrying) -> bool {
    match r {
        Requirement::None => true,
        Requirement::Gold(n) => kit.gold >= *n,
        Requirement::Flag(f) => pos.met(f),
        Requirement::Holding(name) => kit.holding.contains(name),
        Requirement::LooseItemOfSize { w, h } => {
            // **The kit when the position has nothing to say.** A default
            // `Position` is what `ever_possible` asks with, and it carries no
            // tray — so fall through to what the solver walked in with.
            let tray = if pos.of_size.is_empty() { &kit.of_size } else { &pos.of_size };
            tray.get(&key(*w, *h)).copied().unwrap_or(0) > 0
        }
        Requirement::AssembledOfRarity(r) => crate::rating::Rarity::by_name(r)
            .is_some_and(|want| kit.best_rarity.is_some_and(|have| have >= want)),
        Requirement::Surveying(kind) => kit.instrument.as_deref() == Some(kind.as_str()),
        Requirement::All(list) => list.iter().all(|r| takeable(r, pos, kit)),
    }
}

/// The fewest steps that put a stone on every mark, or `None` if nothing does.
///
/// **A second solver, because there is a second kind of puzzle.**
/// [`solvable_blind`] models somebody reading cards and is the right model for
/// every floor built out of flags; a floor built out of stones is a different
/// question with a different answer, and asking the first one about the second
/// gets *"this floor has no puzzle on it"*.
///
/// **In core for the reason that one is**: a number a design stakes itself on,
/// worked out by the thing measuring it, is the page recomputing a total one
/// level up. The Gallery's twenty-five steps are quoted in its own map file and
/// this is what makes that quotable.
///
/// The model is the rule in `world::step` and not a copy of it in spirit: you
/// walk, a stone in the way moves one further if the tile beyond is walkable,
/// empty of stones **and not a place** — a chain under a boulder is a chain
/// nobody can pull.
///
/// **The world must be handed over as the puzzle is played on it.** The
/// Gallery's stones are under water until it is drained, so a caller asks
/// `data::map_now` with the flag already raised; a solver given the flooded
/// room would correctly report that there is nowhere to stand.
pub fn stones_solvable(world: &World) -> Option<usize> {
    let d = &world.blocks;
    if d.at.is_empty() || d.marks.is_empty() {
        return None;
    }
    let allowed = crate::world::Allowances::default();
    let free = |x: i32, y: i32| {
        world.in_bounds(x, y) && world.walkable(x as u8, y as u8, &allowed)
    };
    let start = (world.start.0, world.start.1);
    let sorted = |v: &[[u8; 2]]| {
        let mut v = v.to_vec();
        v.sort();
        v
    };
    let done = |at: &[[u8; 2]]| d.marks.iter().all(|m| at.contains(m));

    let first = sorted(&d.at);
    if done(&first) {
        return Some(0);
    }
    let mut seen: BTreeSet<((u8, u8), Vec<[u8; 2]>)> = BTreeSet::new();
    seen.insert((start, first.clone()));
    let mut queue: VecDeque<((u8, u8), Vec<[u8; 2]>, usize)> = VecDeque::new();
    queue.push_back((start, first, 0));
    while let Some((at, stones, steps)) = queue.pop_front() {
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = (at.0 as i32 + dx, at.1 as i32 + dy);
            if !free(nx, ny) {
                continue;
            }
            let (nx, ny) = (nx as u8, ny as u8);
            let mut moved = stones.clone();
            if let Some(i) = stones.iter().position(|s| *s == [nx, ny]) {
                let (bx, by) = (nx as i32 + dx, ny as i32 + dy);
                if !free(bx, by) {
                    continue;
                }
                let (bx, by) = (bx as u8, by as u8);
                if stones.iter().any(|s| *s == [bx, by]) || world.place_at(bx, by).is_some() {
                    continue;
                }
                moved[i] = [bx, by];
                moved.sort();
            }
            if done(&moved) {
                return Some(steps + 1);
            }
            let key = ((nx, ny), moved.clone());
            if seen.insert(key) {
                queue.push_back(((nx, ny), moved, steps + 1));
            }
        }
    }
    None
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

// **`solvable_blind_with` is not here, and it was written first.** The idea was
// a blind sweep done carrying an instrument, so that *with* and *without* would
// be the same unit. It measured the Cairnfield at **54** with a golem against
// **45** without one — which is true and useless: carrying a golem puts a tenth
// card on the floor worth walking to, and a solver who does not know that is a
// solver who walks nine cairns first. A number that moves the wrong way is a
// number somebody will one day quote.
//
// The comparison that means something is **shortest against shortest**, which
// is what [`solvable_knowing`] does with `None` and with `Some(kind)`: nine
// moves without the golem and one with it, which is the pair `PLAN-M14.md` §1.2
// is actually about.

/// The fewest moves this floor can be solved in, optionally reading it through
/// an instrument.
///
/// **The shortest honest path rather than the worst one**, because an
/// instrument is *information*: it tells you which tile to walk to, so the
/// sweep collapses to the solution itself. `None` is the same question asked of
/// somebody carrying nothing, and the pair is what §1.2's *an instrument makes
/// a floor short, never possible* means — nine moves across the Cairnfield
/// against one on the slab.
pub fn solvable_knowing(
    world: &World,
    events: &EventsData,
    instrument: Option<&str>,
) -> Result<usize, Unsolvable> {
    let mut kit = Carrying::for_floor(world, events);
    if let Some(k) = instrument {
        kit = kit.with(k);
    }
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
    let mut floors: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    worst(&start, &goal, &all, kit, world, &mut floors, 0, &mut seen)
}

/// Which event tiles the solver can walk to, on the map these marks have left.
///
/// **Wading is not assumed.** The solver is somebody who came down a hole with
/// an instrument, not somebody wearing a set of greaves ground off a Bog Toad;
/// a floor that is only solvable in the Toad's Own Frame is a floor, and one
/// that needs it to be solvable at all is a wall.
fn reachable(world: &World, marks: &[String], events: &[&crate::tile_event::TileEvent]) -> BTreeSet<String> {
    let mut w = world.clone();
    w.drain_by(marks);
    let allowed = crate::world::Allowances::default();
    let mut seen: BTreeSet<(u8, u8)> = BTreeSet::new();
    let mut queue = vec![w.start];
    seen.insert(w.start);
    while let Some((x, y)) = queue.pop() {
        for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if !w.in_bounds(nx, ny) {
                continue;
            }
            let (nx, ny) = (nx as u8, ny as u8);
            if w.walkable(nx, ny, &allowed) && seen.insert((nx, ny)) {
                queue.push((nx, ny));
            }
        }
    }
    events
        .iter()
        .filter(|e| {
            world
                .places
                .iter()
                .any(|p| p.id == e.id && seen.contains(&(p.at[0], p.at[1])))
        })
        .map(|e| e.id.clone())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn worst(
    pos: &Position,
    goal: &[&PlaceDef],
    all: &[&crate::tile_event::TileEvent],
    kit: &Carrying,
    world: &World,
    floors: &mut BTreeMap<Vec<String>, BTreeSet<String>>,
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
    let raisable = raisable_here(all);
    let marks: Vec<String> = pos.flags.iter().chain(pos.answered.iter()).cloned().collect();
    let here = floors
        .entry(marks.clone())
        .or_insert_with(|| reachable(world, &marks, all))
        .clone();
    for (ei, e) in all.iter().enumerate() {
        if !e.repeats && pos.answered.contains(&e.id) {
            continue;
        }
        // **A card on the far side of a channel is not a card yet.**
        if !here.contains(&e.id) {
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
            // **A card this floor can never open is read once.** See the module
            // header: it is the difference between 45 and 54 on the Cairnfield,
            // and it is what a person actually does with a slab that says it
            // wants a golem.
            if !ever_possible(&c.requires, &raisable, kit) {
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
        let cost = worst(&next, goal, all, kit, world, floors, depth + 1, seen)?;
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
    let mut floors: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for taken in 0..MAX_ROUNDS {
        if frontier.iter().any(|p| goal.iter().all(|g| p.there(g))) {
            return Ok(taken);
        }
        let mut next = Vec::new();
        for pos in &frontier {
            let marks: Vec<String> =
                pos.flags.iter().chain(pos.answered.iter()).cloned().collect();
            let here = floors
                .entry(marks.clone())
                .or_insert_with(|| reachable(world, &marks, &all))
                .clone();
            for e in &all {
                if !e.repeats && pos.answered.contains(&e.id) {
                    continue;
                }
                if !here.contains(&e.id) {
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
