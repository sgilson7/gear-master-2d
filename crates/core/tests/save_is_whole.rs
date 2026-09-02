//! **A save carries everything, and this is the proof.**
//!
//! The *structure* has always been safe: `SaveFile::of` and `into_game`
//! destructure exhaustively, so a new field on `Character` or `Game` is a
//! compile error until the save carries it. What was missing is proof the
//! **values** survive — nothing asserted `fatigue`, `supplies`, `carried`,
//! `quests_taken`, `quests_done`, `bought`, `answered` or `map`, all of which
//! were added after the round-trip suite was written.
//!
//! So the game round-tripped here is a **thoroughly used** one: walked, events
//! answered, errands taken and finished, gear bought off a shelf, tired,
//! carrying experience, standing in a dungeon. A fresh game is the shape that
//! passes while proving least — every list is empty, so every list compares
//! equal however badly it is handled.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::{fight, quest, save};

const D: Difficulty = Difficulty::Easy;

/// A game with something in every field a save has to carry.
fn a_used_game() -> Game {
    let mut g = Game::new(21, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.character.apply_preset();
    // A locked item, because locks are *state* and not geometry — re-deriving
    // them gives a different board, which is the first thing this fork learned
    // the expensive way. A save that dropped them would still load.
    let seated = g.character.owned.iter().copied().find(|&id| g.character.is_equipped(id));
    if let Some(id) = seated {
        g.character.toggle_lock_item(id);
    }

    // Walked, and standing somewhere that is not the start.
    g.world.at = [5, 16];
    g.world.bump("tiles-walked");
    g.world.bump("wins");
    g.world.last_town = "the-end-of-all-gears".into();

    // An event answered, and a flag off it.
    g.world.answered.push("the-cork-boundary".into());
    g.world.flags.push("has-cork".into());

    // Something bought off a shelf, which is a town and an index.
    g.world.bought.push(("the-end-of-all-gears".into(), 1));
    g.character.give("Tin Plating");

    // An errand taken, one finished, and a tally in the bag.
    quest::take(&mut g, "the-eyes-have-it").expect("the toad errand");
    g.character.give("Toad Eye");
    g.world.quests_done.push("word-with-the-fencecutter".into());

    // Worn out, and carrying something for it.
    g.character.tire(12);
    g.character.give_supply("cork-tea", 2);

    // Experience in the pocket, and enough spent to be somebody: a class is
    // only choosable at level five, so the spend has to reach it.
    g.character.gain_xp(400);
    g.character.carry(17);

    // A skill, a class, and a mid-fight encounter on a dungeon tile.
    g.character.skill_points += 2;
    g.character.take_skill(&data::skills(), "corked").expect("a base node");
    g.character.choose_class("Berserker").expect("a class");
    g.world.map = "the-great-gear-cave".into();
    g.world.at = [2, 1];
    g.encounter = Some(fight::Encounter { enemy: "Iron Sentinel".into(), at: g.world.at });
    g
}

/// Every field, by name. The list is the test: adding a field to the save
/// without adding a line here is what this is meant to catch.
#[test]
fn a_used_game_survives_a_round_trip() {
    let before = a_used_game();
    let text = save::save(&before);
    let after = save::load(&text).expect("a used save loads");

    // --- the world ---------------------------------------------------------
    assert_eq!(after.world.map, before.world.map, "which map");
    assert_eq!(after.world.at, before.world.at, "where");
    assert_eq!(after.world.last_town, before.world.last_town, "the town remembered");
    assert_eq!(after.world.answered, before.world.answered, "events answered");
    assert_eq!(after.world.flags, before.world.flags, "flags");
    assert_eq!(after.world.counters, before.world.counters, "counters");
    assert_eq!(after.world.bought, before.world.bought, "what has been bought");
    assert_eq!(after.world.quests_taken, before.world.quests_taken, "errands taken");
    assert_eq!(after.world.quests_done, before.world.quests_done, "errands finished");

    // --- the character -----------------------------------------------------
    let (a, b) = (&after.character, &before.character);
    assert_eq!(a.gold, b.gold, "the purse");
    assert_eq!(a.grown_health, b.grown_health, "health earned off the boards");
    assert_eq!(a.xp, b.xp, "experience spent");
    assert_eq!(a.carried, b.carried, "experience carried");
    assert_eq!(a.fatigue, b.fatigue, "how worn out");
    assert_eq!(a.supplies, b.supplies, "restoratives carried");
    assert_eq!(a.skill_points, b.skill_points, "points");
    assert_eq!(a.skills_taken, b.skills_taken, "nodes taken");
    assert_eq!(a.class, b.class, "the class");
    assert_eq!(a.owned.len(), b.owned.len(), "how much is owned");
    assert_eq!(a.registry, b.registry, "the registry, in order");
    assert_eq!(a.loadout.slots, b.loadout.slots, "the boards");
    assert_eq!(a.loadout.locks, b.loadout.locks, "the locks");
    assert_eq!(a.loadout.name_seed, b.loadout.name_seed, "the name seed");
    assert_eq!(a.loadout.assembly_pct, b.loadout.assembly_pct, "the assembly percentage");

    // --- the rest ----------------------------------------------------------
    assert_eq!(after.theme, before.theme, "the theme");
    assert_eq!(after.encounter, before.encounter, "the fight in progress");
    assert_eq!(after, before, "and the whole thing, by the game's own equality");
}

/// The values are not zero, or the assertions above are comparing nothing.
///
/// This is the guard against the trap the old suite fell into: a fresh game
/// round-trips perfectly while proving almost nothing, because every list is
/// empty and every empty list compares equal.
#[test]
fn the_used_game_is_actually_used() {
    let g = a_used_game();
    assert!(!g.world.map.is_empty(), "not on a second map");
    assert!(!g.world.answered.is_empty(), "no event answered");
    assert!(!g.world.flags.is_empty(), "no flag set");
    assert!(!g.world.counters.is_empty(), "nothing counted");
    assert!(!g.world.bought.is_empty(), "nothing bought");
    assert!(!g.world.quests_taken.is_empty(), "no errand taken");
    assert!(!g.world.quests_done.is_empty(), "no errand finished");
    assert!(g.character.fatigue > 0, "not tired");
    assert!(!g.character.supplies.is_empty(), "carrying no tins");
    assert!(g.character.carried > 0, "carrying no experience");
    assert!(g.character.xp > 0, "no experience spent");
    assert!(!g.character.skills_taken.is_empty(), "no node taken");
    assert!(g.character.class.is_some(), "no class");
    assert!(g.encounter.is_some(), "no fight in progress");
    assert!(!g.character.loadout.locks.is_empty(), "nothing locked, so the board is loose");
}

/// **A place with an errand on it can be gone back to.**
///
/// Reported from a real session: *"i could not revisit marbulon to submit the
/// quest."* Answering her card once put her id in `answered`, and `world::step`
/// opened an event only when its id was absent — so the tile went inert and her
/// two errands, the questline that unlocks the cave, could never be handed in.
#[test]
fn an_event_with_an_errand_reopens_once_its_choices_are_spent() {
    use gm2d_core::world::Dir;

    let w = data::world(D);
    let mut g = Game::new(4, "td");
    g.world = gm2d_core::world::WorldState::at_start(&w);
    let door = w.places.iter().find(|p| p.id == "marbulons-door").expect("Marbulon");

    // Stand beside her and step on.
    g.world.at = [door.at[0] - 1, door.at[1]];
    let mut rng = g.rng.clone();
    let s = gm2d_core::world::step(&w, &mut g.world, &mut rng, D, Dir::East);
    assert_eq!(s.event.as_deref(), Some("marbulons-door"), "her card did not open");
    assert!(!s.spent, "her choices were spent before they were answered");

    // Answer it, walk off, and come back.
    g.world.answered.push("marbulons-door".into());
    g.world.at = [door.at[0] - 1, door.at[1]];
    let s = gm2d_core::world::step(&w, &mut g.world, &mut rng, D, Dir::East);
    assert_eq!(
        s.event.as_deref(),
        Some("marbulons-door"),
        "an answered event went inert, so its errands are unreachable"
    );
    assert!(s.spent, "the choices should be spent the second time");

    // And the errands are still hers to give.
    let quests = data::quests();
    let first = quests.get("marbulon-asks-first").expect("her first errand");
    assert_eq!(quest::stage(&g, first), quest::Stage::Offered);
    quest::take(&mut g, &first.id).expect("she can still be asked");
}
