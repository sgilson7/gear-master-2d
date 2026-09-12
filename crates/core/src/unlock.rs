//! What a shut thing wants, all the way down.
//!
//! Reported from play, standing at the bar of shingle: *"the event outside of
//! the diamond to get to the southern area should say very specifically the
//! mechanical necessities to unlock the area; so the specific events and
//! errands that have to be complete. anything that is locked should have such
//! an explanation."*
//!
//! Which is the third time this project has been told the same thing, each
//! time one level further in. The shore said what a cliff says, and was given
//! its own sentence. Then the sentence named the notch and not where the notch
//! is cut, and was given the event that opens it — one hop. This is the rest of
//! the hops: the tenth cairn wants the trig stone, which wants the reach read,
//! which wants a map you can only walk into with an instrument on your frame,
//! and a refusal that names the last of those four and none of the other three
//! is still a wall with a better sentence on it.
//!
//! **Derived, and never a list.** Every step is looked up — which choice raises
//! a flag, which map that choice's card stands on, which gate opens that map,
//! and what *that* gate wants — so a chain somebody re-authors cannot leave
//! this pointing at the wrong place. It is the engine's register, unthemed,
//! TONE 13a: a player reading it is working out what to go and do, and a plan
//! wearing a joke has to be translated first.

use crate::combat::Difficulty;
use crate::tile_event::{EventsData, Requirement};
use crate::world::{PlaceKind, World};

/// How far a chain is followed.
///
/// The longest in the shipped game is four. A cap rather than trust: a map file
/// can name a mark whose source wants the mark itself, and a refusal that hangs
/// the page is worse than one that stops early.
const DEPTH: usize = 8;

/// Every step between here and a mark being made, in the order they have to be
/// done, with anything already done left out.
///
/// `done` is the marks this character has — `WorldState::marks` — so a chain
/// three quarters walked prints the quarter that is left rather than the whole
/// thing again.
pub fn steps_to(mark: &str, done: &[String], difficulty: Difficulty) -> Vec<String> {
    let maps = crate::data::all_maps(difficulty);
    let events = crate::data::events();
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    walk(mark, done, &maps, &events, &mut out, &mut seen, 0);
    out
}

/// The same, rendered as one sentence, or nothing if there is nothing to say.
pub fn sentence(mark: &str, done: &[String], difficulty: Difficulty) -> Option<String> {
    let steps = steps_to(mark, done, difficulty);
    if steps.is_empty() {
        return None;
    }
    Some(format!("To open it: {}.", steps.join(", then ")))
}

fn walk(
    mark: &str,
    done: &[String],
    maps: &[World],
    events: &EventsData,
    out: &mut Vec<String>,
    seen: &mut Vec<String>,
    depth: usize,
) {
    if depth > DEPTH || done.iter().any(|d| d == mark) || seen.iter().any(|s| s == mark) {
        return;
    }
    seen.push(mark.to_string());

    // A boss's tile id, which is what beating it writes down.
    if let Some((map, p)) = find_place(maps, mark) {
        if p.kind == PlaceKind::Boss {
            reach_the_map(map, done, maps, events, out, seen, depth);
            out.push(format!("beat what stands at {}", place_words(p)));
            return;
        }
    }

    // A choice that raises it. Its own requirement comes first, because that
    // is the order somebody has to do them in.
    if let Some(e) = events.events.iter().find(|e| e.choices.iter().any(|c| raises(&c.outcome, mark))) {
        let c = e.choices.iter().find(|c| raises(&c.outcome, mark)).expect("just found");
        want(&c.requires, done, maps, events, out, seen, depth + 1);
        if let Some((map, _)) = find_place(maps, &e.id) {
            reach_the_map(map, done, maps, events, out, seen, depth);
        }
        out.push(format!("{} — {}", e.title, c.label));
        return;
    }

    // A card that is only read. Answering one writes its own id down, so a mark
    // that *is* an event id is a card somebody has to stand on.
    if let Some(e) = events.events.iter().find(|e| e.id == mark) {
        if let Some((map, _)) = find_place(maps, &e.id) {
            reach_the_map(map, done, maps, events, out, seen, depth);
        }
        out.push(e.title.clone());
        return;
    }

    // An errand's own mark, which `Goal::Word` writes when you arrive.
    if let Some(id) = mark.strip_prefix(crate::quest::SPOKEN) {
        if let Some(q) = crate::data::quests().get(id) {
            out.push(format!("the errand {}", q.name));
            return;
        }
    }

    out.push(mark.replace('-', " "));
}

/// What a gate onto `map` wants, and then the going in.
///
/// One hop of geography per step, and the gate's own locks are walked with the
/// same function that got here — so a door behind a door behind a survey reads
/// as three lines rather than as one that mentions the last of them.
fn reach_the_map(
    map: &str,
    done: &[String],
    maps: &[World],
    events: &EventsData,
    out: &mut Vec<String>,
    seen: &mut Vec<String>,
    depth: usize,
) {
    if depth > DEPTH {
        return;
    }
    let Some(gate) = maps
        .iter()
        .flat_map(|w| w.places.iter())
        .find(|p| p.kind == PlaceKind::Gate && leads_to(p, map))
    else {
        return;
    };
    // **A door you have already been through is not a step.** Turning a key
    // writes the gate's own id into `answered`, so this is the same question
    // `Game::unlock` asks — and without it a player standing on the Treyway is
    // told to go and find a key they spent to get there.
    if done.iter().any(|d| *d == gate.id) {
        return;
    }
    let key = format!("gate:{}", gate.id);
    if seen.iter().any(|s| s == &key) {
        return;
    }
    seen.push(key);
    for m in &gate.needs_all {
        walk(m, done, maps, events, out, seen, depth + 1);
    }
    if let Some(item) = &gate.needs {
        out.push(format!("carry {item}"));
    }
    if gate.needs_survey {
        out.push("assemble a survey instrument on its own frame".to_string());
    }
    out.push(format!("go in through {}", place_words(gate)));
}

/// What a choice wants, flattened into steps.
fn want(
    r: &Requirement,
    done: &[String],
    maps: &[World],
    events: &EventsData,
    out: &mut Vec<String>,
    seen: &mut Vec<String>,
    depth: usize,
) {
    match r {
        Requirement::None => {}
        Requirement::All(list) => {
            for one in list {
                want(one, done, maps, events, out, seen, depth);
            }
        }
        Requirement::Flag(f) => walk(f, done, maps, events, out, seen, depth),
        Requirement::Gold(n) => out.push(format!("{n} Fnorp in the purse")),
        Requirement::Holding(item) => out.push(format!("carry {item}")),
        Requirement::LooseItemOfSize { w, h } => {
            out.push(format!("a loose component {w} by {h}"))
        }
        Requirement::AssembledOfRarity(r) => {
            out.push(format!("an assembled item of {r} or better"))
        }
        Requirement::Surveying(kind) => out.push(format!("carry the {kind}")),
    }
}

/// Whether a gate opens onto this map — its far side, or one of its floors.
///
/// Not `opens_onto`, which answers *which floor you get* and needs the run's
/// own marks. This is a question about the file: is this the door.
fn leads_to(p: &crate::world::PlaceDef, map: &str) -> bool {
    p.to.as_deref() == Some(map) || p.floors.iter().any(|f| f.map == map)
}

fn find_place<'a>(maps: &'a [World], id: &str) -> Option<(&'a str, &'a crate::world::PlaceDef)> {
    maps.iter()
        .find_map(|w| w.places.iter().find(|p| p.id == id).map(|p| (w.id.as_str(), p)))
}

/// A place in words, falling back to its id with the hyphens taken out.
fn place_words(p: &crate::world::PlaceDef) -> String {
    if p.name.is_empty() {
        p.id.replace('-', " ")
    } else {
        p.name.clone()
    }
}

fn raises(o: &crate::tile_event::Outcome, flag: &str) -> bool {
    crate::tile_event::sets_flag(o, flag)
}
