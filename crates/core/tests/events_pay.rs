//! Events that pay something, and say what they pay. M12.5.
//!
//! The measurement that started this milestone: 56 event tiles, **9** offering
//! a choice, **47** prose and nothing else, **0** events opening another
//! event, **0** choices saying what they pay.

mod common;

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::tile_event::{Outcome, Requirement};

const D: Difficulty = Difficulty::Easy;

fn flags_in(o: &Outcome, out: &mut Vec<String>) {
    match o {
        Outcome::All(list) => list.iter().for_each(|i| flags_in(i, out)),
        Outcome::Flag(f) => out.push(f.clone()),
        _ => {}
    }
}

fn warps_in(o: &Outcome, out: &mut Vec<(String, [u8; 2])>) {
    match o {
        Outcome::All(list) => list.iter().for_each(|i| warps_in(i, out)),
        Outcome::Warp { map, at } => out.push((map.clone(), *at)),
        _ => {}
    }
}

/// Every flag an event sets is read by something.
///
/// **The `every_ench_comes_from_somewhere` shape.** A flag nobody reads is a
/// chain that was started and never finished — the data format has been able
/// to open one event from another since M2 and had never once been used, so
/// eighteen `flag` outcomes existed and three of them were read.
#[test]
fn every_flag_an_event_sets_is_read_by_something() {
    let events = data::events();
    let mut set: Vec<String> = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            flags_in(&c.outcome, &mut set);
        }
    }
    set.sort();
    set.dedup();
    assert!(!set.is_empty(), "no event sets a flag, so this checks nothing");

    // Read by a choice's requirement, by a place that is hidden until it, or
    // by an errand that waits on it.
    let mut read: Vec<String> = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            if let Requirement::Flag(f) = &c.requires {
                read.push(f.clone());
            }
        }
    }
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in &w.places {
            read.extend(p.hidden_until.iter().cloned());
        }
    }
    let orphans: Vec<&String> = set.iter().filter(|f| !read.contains(f)).collect();
    assert!(
        orphans.is_empty(),
        "{} flags are set by an event and read by nothing: {:?}",
        orphans.len(),
        orphans
    );
}

/// A map is not mostly wallpaper.
///
/// **A `note` is a legitimate event and stays one** — read once, then quiet;
/// it is the map's furniture. But a map whose events are *only* furniture is a
/// map where reading pays nothing, and the Kettleworks field was 0 decisions
/// against 41 notes.
#[test]
fn a_map_is_not_mostly_wallpaper() {
    let events = data::events();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        let mut asks = 0;
        let mut notes = 0;
        for p in &w.places {
            if !matches!(p.kind, gm2d_core::world::PlaceKind::Event) {
                continue;
            }
            // A place's id *is* its event's id — `tiles.json` places an event
            // and `events.json` never says where it is.
            match events.get(&p.id) {
                Some(e) if !e.choices.is_empty() => asks += 1,
                Some(_) => notes += 1,
                None => {}
            }
        }
        // A map with no events at all is not wallpaper; it is a map with no
        // events. Only maps that read to you are asked this.
        if asks + notes == 0 {
            continue;
        }
        assert!(
            asks >= notes,
            "{id}: {asks} events ask something and {notes} do not, so reading it pays nothing"
        );
    }
}

/// Every warp lands somewhere you can stand.
///
/// **The exact shape of `every_gate_leads_somewhere_you_can_stand`**, which
/// exists because a gate whose far side is a wall strands a player and nothing
/// else in the game would say so. A warp is a gate that did not ask.
#[test]
fn every_warp_lands_somewhere_you_can_stand() {
    let events = data::events();
    let mut warps = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            warps_in(&c.outcome, &mut warps);
        }
    }
    assert!(!warps.is_empty(), "no event warps anybody, so this checks nothing");
    let allowed = gm2d_core::world::Allowances::of(&[]);
    for (map, at) in &warps {
        assert!(
            data::MAPS.iter().any(|(id, _)| id == map),
            "a warp goes to {map:?}, which is not a map this build ships"
        );
        let w = data::map(map, D);
        assert!(
            w.walkable(at[0], at[1], &allowed),
            "a warp lands on {map} ({}, {}), which cannot be stood on",
            at[0],
            at[1]
        );
    }
}

/// A warp moves you out, and never home.
///
/// `Rule::Homeward` is the thing that takes you back and it costs a tin. A
/// warp that landed you in a town would be free fast travel with a paragraph
/// on it.
#[test]
fn a_warp_is_never_a_way_home() {
    let events = data::events();
    let mut warps = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            warps_in(&c.outcome, &mut warps);
        }
    }
    for (map, at) in &warps {
        let w = data::map(map, D);
        let here = w.place_at(at[0], at[1]);
        assert!(
            !matches!(here.map(|p| &p.kind), Some(gm2d_core::world::PlaceKind::Town)),
            "a warp lands in a town on {map}, which is a free ride home"
        );
    }
}

/// Every choice says what it pays, and says it in the engine's words.
///
/// **TONE 13a, and the same lint `no_mechanical_line_speaks_the_theme` makes
/// over skill nodes.** An outcomes box that said "you feel the Nut Freeze
/// lift" would be that bug in a new room.
#[test]
fn every_choice_says_what_it_pays_and_does_not_speak_the_theme() {
    let events = data::events();
    let themed = ["fnorp", "cork", "funny", "roast", "nut freeze", "semuta", "idiot mode"];
    let mut boxes = 0;
    for e in &events.events {
        for c in &e.choices {
            let lines = c.outcome.describe();
            // A choice that pays nothing at all still says so.
            assert!(
                !lines.is_empty() || matches!(c.outcome, Outcome::Flag(_)),
                "{}: the choice {:?} describes itself as nothing",
                e.id,
                c.label
            );
            boxes += lines.len();
            for l in &lines {
                // **An errand's name is a proper noun and is exempt.** TONE
                // 13a's carve-out: a name somebody wrote is not translated,
                // which is why a set's name is not themed either. "THE CORK
                // YOU TOOK" is about boundary cork, the material, and not
                // about the theme's word for armour.
                if l.starts_with("Begins an errand:") {
                    continue;
                }
                let low = l.to_lowercase();
                for t in themed {
                    // "Fnorp" is the theme's word for gold and is the one
                    // exception: the economy's unit is what the player counts
                    // in, and every other screen in the game says it.
                    if t == "fnorp" {
                        continue;
                    }
                    assert!(!low.contains(t), "{}: an outcomes box says {l:?}", e.id);
                }
            }
        }
    }
    assert!(boxes > 20, "only {boxes} outcome lines in the whole game");
}

/// The three new outcome kinds each *do* something.
///
/// **Calls rather than declares**, the `every_offered_class_reaches_something`
/// rule: a payout that is described and not wired is the fifth promise this
/// project has shipped that reached nothing.
#[test]
fn the_new_outcomes_reach_something() {
    // A tin.
    let mut g = Game::new(3, "td");
    let tin = data::supplies().supplies[0].id.clone();
    let before = g.character.supply_count(&tin);
    let mut r = Vec::new();
    g.apply_outcome_for_test(&Outcome::Supply { id: tin.clone(), n: 2 }, &mut r, D);
    assert_eq!(g.character.supply_count(&tin), before + 2, "Supply handed over nothing");
    assert!(!r.is_empty());

    // Tiredness.
    let mut g = Game::new(3, "td");
    let before = g.character.fatigue;
    let mut r = Vec::new();
    g.apply_outcome_for_test(&Outcome::Tire(8), &mut r, D);
    assert!(g.character.fatigue > before, "Tire cost nothing");

    // A warp.
    let mut g = Game::new(3, "td");
    let was = (g.world.map_id(), g.world.at);
    let mut r = Vec::new();
    g.apply_outcome_for_test(
        &Outcome::Warp { map: "the-treyway".into(), at: [4, 13] },
        &mut r,
        D,
    );
    assert_eq!(g.world.map_id(), "the-treyway", "Warp did not move anybody");
    assert!(
        g.world.positions.iter().any(|(m, at)| *m == was.0 && *at == was.1),
        "a warp did not write down where you were standing"
    );
}

/// A chain completes end to end, and each step needs the one before it.
///
/// **The data format has been able to open one event from another since M2 and
/// had never once been used.** Three chains now, and this walks the longest:
/// the reach's plate, its tenth cairn, and the four acres every survey agrees
/// about — each step refused until the one before it is taken, and the last
/// one paying a component rather than Fnorp, because a chain that pays Fnorp
/// is a longer way to earn Fnorp.
#[test]
fn a_chain_runs_end_to_end_and_no_step_can_be_skipped() {
    let chain = [
        ("the-wextreen-reach", "Read the five"),
        ("the-trig-stone", "Sign the plate"),
        ("the-nine-surveys", "Build the tenth cairn"),
        ("the-common-ground", "Take the four acres' measure"),
    ];
    let events = data::events();
    let index = |id: &str, label: &str| {
        events
            .get(id)
            .unwrap_or_else(|| panic!("no event {id}"))
            .choices
            .iter()
            .position(|c| c.label == label)
            .unwrap_or_else(|| panic!("{id} has no choice {label:?}"))
    };

    // **No step can be skipped.** Each one refused on a fresh game, which is
    // what makes it a chain rather than four events that happen to be near
    // each other.
    for (id, label) in &chain[1..] {
        let mut g = Game::new(9, "td");
        let n = index(id, label);
        assert!(
            g.answer_event(id, n, D).is_err(),
            "{id}: {label:?} can be taken without the step before it"
        );
    }

    // And end to end it runs.
    let mut g = Game::new(9, "td");
    let mut paid_a_component = false;
    for (id, label) in &chain {
        let n = index(id, label);
        let receipt = g
            .answer_event(id, n, D)
            .unwrap_or_else(|e| panic!("{id}: {label:?} was refused mid-chain: {e}"));
        if receipt.iter().any(|l| l.starts_with("Gained:")) {
            paid_a_component = true;
        }
    }
    assert!(paid_a_component, "the chain paid no gear, so it is a longer way to earn Fnorp");
}

/// An event is answered once, and the second visit is quiet.
///
/// **Check the second visit** — the rule three post-M11 faults shared. An
/// event with choices is spent once its choices are taken.
#[test]
fn a_choice_is_taken_once() {
    let mut g = Game::new(4, "td");
    let id = "the-counted-heap";
    g.answer_event(id, 0, D).expect("the first time");
    let again = g.answer_event(id, 1, D).expect_err("the second time");
    assert!(again.contains("already"), "{again:?}");
}

fn errands_in(o: &Outcome, out: &mut Vec<String>) {
    match o {
        Outcome::All(l) => l.iter().for_each(|i| errands_in(i, out)),
        Outcome::Errand(id) => out.push(id.clone()),
        _ => {}
    }
}

/// **A choice at a chain root is never just a smaller number.**
///
/// The complaint this exists for, in the words it arrived in: at The Shallows
/// Marker one half paid 12 experience and the other 20, and both opened a
/// different chain — *so they'd always take the 20, cause why would you take
/// the smaller number*. From where the player sits there was no decision,
/// because the thing that made it one was invisible.
///
/// So: **every choice at an event that roots a chain hands over an errand, and
/// no two of them hand over the same one.** The errand is what makes the
/// branch legible — it lands in the log, the log points at the map, and the
/// outcomes box says so before the choice is taken.
#[test]
fn every_choice_at_a_chain_root_starts_its_own_errand() {
    let events = data::events();
    let quests = data::quests();

    // A root is an event where at least one choice starts an errand.
    let roots: Vec<&gm2d_core::tile_event::TileEvent> = events
        .events
        .iter()
        .filter(|e| {
            e.choices.iter().any(|c| {
                let mut v = Vec::new();
                errands_in(&c.outcome, &mut v);
                !v.is_empty()
            })
        })
        .collect();
    assert!(roots.len() >= 8, "only {} events root a chain", roots.len());

    for e in &roots {
        let mut seen: Vec<String> = Vec::new();
        for c in &e.choices {
            let mut mine = Vec::new();
            errands_in(&c.outcome, &mut mine);
            // **Starts one, or continues one.** An event can be a root and a
            // step at once — The Standing Frame begins two chains and is the
            // second rung of a third, and its cork choice is gated on the cork
            // you took at the boundary. What must never happen is the third
            // kind: a choice at a root that goes nowhere, which is the
            // "smaller number" this test exists to refuse.
            if mine.is_empty() {
                assert!(
                    !matches!(c.requires, Requirement::None),
                    "{}: the choice {:?} starts no chain and continues none, so it is only a \
                     smaller number than the one beside it",
                    e.id,
                    c.label
                );
                continue;
            }
            assert_eq!(
                mine.len(), 1,
                "{}: {:?} starts {} errands at once", e.id, c.label, mine.len()
            );
            let id = mine.remove(0);
            assert!(
                quests.get(&id).is_some(),
                "{}: {:?} starts {id:?}, which is not an errand",
                e.id,
                c.label
            );
            assert!(
                !seen.contains(&id),
                "{}: two choices both start {id:?}, so one of them is the smaller number",
                e.id
            );
            seen.push(id);
        }
    }
}

/// Every errand a choice hands over is granted, never offered.
///
/// The branch you did not take must not be sitting on the same tile a moment
/// later, offering itself at a counter.
#[test]
fn a_chain_errand_is_handed_over_and_never_sits_on_a_counter() {
    let events = data::events();
    let quests = data::quests();
    let mut started = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            errands_in(&c.outcome, &mut started);
        }
    }
    assert!(started.len() >= 20, "only {} chain errands", started.len());
    for id in &started {
        let q = quests.get(id).expect("it exists");
        assert!(q.granted, "{id} is started by a choice and also offered at {}", q.giver);
        assert!(
            matches!(q.goal, gm2d_core::quest::Goal::Word { .. }),
            "{id} is a chain errand and does not send you anywhere"
        );
        assert!(
            !q.reward.is_empty() || !q.enchs.is_empty() || q.gold > 0,
            "{id} pays nothing at the end of it"
        );
    }
    // And none of them can be picked up by walking to the tile that gave it.
    let mut g = Game::new(2, "td");
    for id in &started {
        let q = quests.get(id).expect("it exists");
        assert_eq!(
            gm2d_core::quest::stage(&g, q),
            gm2d_core::quest::Stage::Locked,
            "{id} is offered before anybody chose it"
        );
    }
    // Taking one puts it in the log.
    let first = &started[0];
    let mut r = Vec::new();
    g.apply_outcome_for_test(&Outcome::Errand(first.clone()), &mut r, D);
    assert!(g.world.quests_taken.contains(first), "starting an errand did not log it");
    assert!(r.iter().any(|l| !l.is_empty()), "and said nothing about it");
}

/// A chain's payout is not only experience.
///
/// The whole complaint: rewards that are all experience are rewards a player
/// reads as one number against another. Across the chains there has to be a
/// spread — gear, an ench, an instrument's part, Fnorp.
#[test]
fn the_chains_pay_more_than_experience() {
    let quests = data::quests();
    let chain: Vec<_> = quests.quests.iter().filter(|q| q.granted).collect();
    assert!(chain.len() >= 20, "only {} chain errands", chain.len());
    let gear = chain.iter().filter(|q| !q.reward.is_empty()).count();
    let enchs = chain.iter().filter(|q| !q.enchs.is_empty()).count();
    let instrument = chain
        .iter()
        .filter(|q| {
            q.reward.iter().any(|r| {
                ["Map Shard", "Glass Lens", "Magnet", "Cosmic Orb", "Cosmic Alignment",
                 "Living Earth"]
                    .contains(&r.as_str())
            })
        })
        .count();
    assert!(gear >= 15, "only {gear} chains pay a component");
    assert!(enchs >= 2, "only {enchs} chains pay an ench");
    assert!(instrument >= 3, "only {instrument} chains pay an instrument's part");
    assert!(chain.iter().all(|q| q.gold > 0), "a chain that pays no Fnorp at all");
}
