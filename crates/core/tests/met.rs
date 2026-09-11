//! The bestiary: what you have seen, and what a glossary entry is allowed to be.
//!
//! Asked for as *"once you have encountered an enemy once, you should be able to
//! see a bestiary glossary entry for them where you can see their loadout and
//! stats."*
//!
//! **Meeting is not beating**, which is why this is a second counter beside
//! `beat:` rather than a reading of it: a creature that has killed you four
//! times is one you have met four times and beaten none, and a glossary is
//! about what you have seen.

use gm2d_core::combat::Difficulty;
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;

mod common;

const D: Difficulty = Difficulty::Easy;

fn a_game() -> Game {
    let mut g = Game::new(9, "td");
    g.world = gm2d_core::world::WorldState::at_start(&gm2d_core::data::world(D));
    g
}

// ------------------------------------------------------------- the counting

/// Meeting one writes it down, and the encounter is set in the same move.
///
/// **`encounter_with` is the one door.** An encounter was set in two places in
/// the shim — the ground rolling one and a boss standing on a tile — and a
/// bestiary populated at one of them is a bestiary with no bosses in it.
#[test]
fn meeting_a_creature_writes_it_down() {
    let mut g = a_game();
    assert!(!g.has_met("Cave Rat"));
    g.encounter_with("Cave Rat", [4, 17]);
    assert_eq!(g.met("Cave Rat"), 1);
    assert!(g.has_met("Cave Rat"));
    assert_eq!(g.encounter.as_ref().map(|e| e.enemy.as_str()), Some("Cave Rat"));
    assert_eq!(g.met("Bog Toad"), 0, "meeting one wrote down another");

    // And it counts every time, because the entry says how many.
    g.encounter_with("Cave Rat", [4, 17]);
    assert_eq!(g.met("Cave Rat"), 2);
}

/// **Meeting is not beating**, and the two counters do not read each other.
#[test]
fn losing_to_something_still_puts_it_in_the_book() {
    let mut g = a_game();
    g.character = common::bench();
    g.character.carry(40);
    g.world.last_town = "the-end-of-all-gears".into();
    g.encounter_with("Rust Colossus", g.world.at);
    let log = fight::run(&g, D).expect("there is something to fight");
    assert_eq!(
        log.outcome,
        gm2d_core::combat::Outcome::Defeat,
        "an empty board beat the deepest thing on the first map"
    );
    fight::settle(&mut g, &log, D).expect("a fight that happened settles");
    assert_eq!(g.met("Rust Colossus"), 1, "it killed you and is not in the book");
    assert_eq!(g.beaten("Rust Colossus"), 0, "a defeat counted as a win");
    assert!(g.bestiary().iter().any(|l| l.canonical == "Rust Colossus"));
}

/// A creature that gave up was still met.
#[test]
fn a_routed_creature_is_still_in_the_book() {
    let mut g = a_game();
    g.character = common::bench();
    for (name, x, y) in
        [("Ratskin Material", 0u8, 0u8), ("Ratskin Mold", 2, 0), ("Rat Signet", 4, 0)]
    {
        let id = common::spare(&g.character, name);
        g.character.registry.set_rotation(id, 0);
        g.character.equip(id, SlotKind::Gloves, x, y).expect("the Mandate seats");
    }
    gm2d_core::loadout::lock_assembled_in(
        &mut g.character.loadout,
        &g.character.registry,
        SlotKind::Gloves,
    );
    g.encounter_with("Cave Rat", g.world.at);
    fight::rout(&mut g).expect("the Mandate routs a Cave Rat");
    assert_eq!(g.met("Cave Rat"), 1, "it walked away and left no entry");
}

/// A name this build has not got is not written into the book.
///
/// The encounter is still set — `fight::spec` answers `None` and the caller
/// handles it, which is what happens today — but there is nothing to look up,
/// so there is no entry to make.
#[test]
fn a_creature_this_build_has_not_got_makes_no_entry() {
    let mut g = a_game();
    g.encounter_with("A Thing From Another Build", [1, 1]);
    assert!(g.encounter.is_some(), "the encounter was dropped as well");
    assert_eq!(g.met("A Thing From Another Build"), 0);
    assert!(g.bestiary().is_empty());
}

// --------------------------------------------------------------- the index

/// The book holds what you have met and nothing else, in the ladder's order.
///
/// **Ordered by the ladder rather than by when you met them**, because a
/// glossary is a thing you look things up in: a list that reorders itself as
/// you fight is a list you cannot find anything in twice.
#[test]
fn the_book_is_what_you_have_met_in_the_ladders_order() {
    let mut g = a_game();
    assert!(g.bestiary().is_empty(), "a new game has met nothing");

    // Deliberately met out of ladder order.
    for who in ["Rust Colossus", "Cave Rat", "Bog Toad"] {
        g.encounter_with(who, [1, 1]);
    }
    let book = g.bestiary();
    assert_eq!(book.len(), 3);
    let order: Vec<&str> = book.iter().map(|l| l.canonical.as_str()).collect();
    let ladder: Vec<&str> = gm2d_core::combat::LADDER
        .iter()
        .map(|m| m.name)
        .filter(|n| order.contains(n))
        .collect();
    assert_eq!(order, ladder, "the book is in the order they were met, not the ladder's");

    for l in &book {
        assert!(!l.name.is_empty(), "{} has no themed name to print", l.canonical);
        assert!(l.met >= 1);
        assert_eq!(l.beaten, 0, "nothing has been beaten and the book says otherwise");
    }
    assert!(
        book.len() < gm2d_core::combat::LADDER.len(),
        "three meetings put the whole ladder in the book"
    );
}

/// The counts beside an entry are the two different questions.
#[test]
fn the_book_counts_meetings_and_wins_separately() {
    let mut g = a_game();
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    for _ in 0..3 {
        g.encounter_with("Cave Rat", g.world.at);
        let log = fight::run(&g, D).expect("something to fight");
        fight::settle(&mut g, &log, D).expect("it settles");
    }
    // Met three times and beaten three, then met once more and walked away.
    g.encounter_with("Cave Rat", g.world.at);
    g.encounter = None;

    let l = g.bestiary().into_iter().find(|l| l.canonical == "Cave Rat").expect("in the book");
    assert_eq!(l.met, 4, "walking away from one did not count as meeting it");
    assert_eq!(l.beaten, 3);
}

// ------------------------------------------------------------------ the save

/// The book survives a round trip, and it makes no seam.
#[test]
fn the_book_survives_a_round_trip_and_makes_no_seam() {
    let before = gm2d_core::save::catalog_fingerprint();
    let mut g = a_game();
    g.encounter_with("Cave Rat", [4, 17]);
    g.encounter = None;

    let back = gm2d_core::save::load(&gm2d_core::save::save(&g)).expect("it reads back");
    assert_eq!(back.met("Cave Rat"), 1);
    assert_eq!(back.bestiary().len(), 1);
    assert_eq!(
        before,
        gm2d_core::save::catalog_fingerprint(),
        "the bestiary moved the catalogue, and it has no business doing that"
    );
}

/// A save written before any of this opens with an empty book.
///
/// **`counters` has been `#[serde(default)]` since M2**, so this is the no-seam
/// claim as a check rather than as a sentence in a commit: a file with no
/// counters at all still opens, and the character has met nothing, which is
/// what that character had.
#[test]
fn a_save_from_before_the_book_opens_with_it_empty() {
    let mut g = a_game();
    g.encounter_with("Cave Rat", [4, 17]);
    g.encounter = None;
    let mut v: serde_json::Value =
        serde_json::from_str(&gm2d_core::save::save(&g)).expect("is json");
    let w = v
        .get_mut("state")
        .and_then(|s| s.get_mut("world"))
        .and_then(|w| w.as_object_mut())
        .expect("the save has a world");
    assert!(w.remove("counters").is_some(), "there is no `counters` key, so this proves nothing");
    let back = gm2d_core::save::load(&serde_json::to_string(&v).unwrap())
        .expect("a file with no counters still opens");
    assert!(back.bestiary().is_empty());
    assert!(!back.has_met("Cave Rat"));
}

// ---------------------------------------------------------- what an entry is

/// **Every creature in the game can be looked up**, and its entry has something
/// in it.
///
/// The ask is *"their loadout and stats"*, and this is the half `cargo test` can
/// answer: that every one of the sixty has a board or a body to describe, so no
/// entry is a name and an empty box. A creature with neither would be a glossary
/// page that says nothing, and `Encounter` is not the place to find that out.
#[test]
fn every_creature_has_something_to_say_about_itself() {
    for m in gm2d_core::combat::LADDER {
        let (stats, _) = m.outfit_at(D);
        let (reg, lo) = m.loadout_at(D);
        let items = lo.combat_items(&reg);
        assert!(stats.health > 0, "{} has no health", m.name);
        assert!(
            !items.is_empty() || !m.attacks.is_empty(),
            "{} wears nothing and has no attacks, so its entry would be blank",
            m.name
        );
    }
}

/// **Sixty creatures carry a defence, and nothing has ever shown one.**
///
/// `physical_resist`, `magic_resist` and the rest have been on `Stats` since the
/// fork; resist cuts the blow, pierce cuts the resistance, hardening cancels the
/// piercing. This is the measurement behind *"currently you cannot see enemies
/// stats / resists"* — the numbers are real and were on no screen.
#[test]
fn the_ladder_is_carrying_defences_worth_printing() {
    let with_any = gm2d_core::combat::LADDER
        .iter()
        .filter(|m| {
            let (s, _) = m.outfit_at(D);
            s.physical_resist != 0 || s.magic_resist != 0 || s.mind_resist != 0
        })
        .count();
    assert!(
        with_any > gm2d_core::combat::LADDER.len() / 2,
        "only {with_any} of {} creatures resist anything, so a defences box is decoration",
        gm2d_core::combat::LADDER.len()
    );
}

/// **A defence is printed at what the fight will use, not what the block says.**
///
/// Found by hand-checking the deployed page: the Iron Abbot's entry read *"144%
/// mind resist"* and *"99% physical resist"*, and the simulation clamps those
/// lanes at 100 and 95. The page was drawing what core sent it, which is the
/// rule; **core was sending a number it does not itself believe**, which is the
/// rule one level up — and a glossary that disagrees with the thing it is a
/// glossary of is worse than no glossary.
#[test]
fn no_defence_is_printed_above_what_the_fight_will_use() {
    use gm2d_core::stats::{LANE_CAP, MIND_CAP, RESIST_CAP};
    let mut capped = 0;
    for m in gm2d_core::combat::LADDER {
        let (s, _) = m.outfit_at(D);
        for d in gm2d_core::explain::defences_of(&s) {
            let cap = match d.what {
                "physical resist" | "magic resist" => RESIST_CAP,
                "mind resist" | "curse resist" => LANE_CAP,
                "reflect" => i32::MAX,
                _ => MIND_CAP,
            };
            assert!(
                d.value <= cap,
                "{}: {} is printed at {} and the fight caps it at {cap}",
                m.name,
                d.what,
                d.value
            );
            if let Some(raw) = d.raw {
                capped += 1;
                assert!(raw > d.value, "{}: {} claims a cap that took nothing off", m.name, d.what);
            }
        }
    }
    // **And something in the shipped ladder is actually over a cap**, or this
    // check is comparing every number with a ceiling none of them reach.
    assert!(capped > 0, "no creature in the game exceeds a defence cap, so this proves nothing");
}

/// The clamp the bestiary prints is the clamp the fight applies, read from the
/// same constant rather than from a second copy of the number.
#[test]
fn the_printed_cap_is_the_fights_own() {
    use gm2d_core::stats::{Stats, LANE_CAP, RESIST_CAP};
    let s = Stats { physical_resist: 400, mind_resist: 400, ..Stats::new(10, 0, 0, 100) };
    let rows = gm2d_core::explain::defences_of(&s);
    let by = |w: &str| rows.iter().find(|d| d.what == w).expect("row").value;
    assert_eq!(by("physical resist"), RESIST_CAP);
    assert_eq!(by("mind resist"), LANE_CAP);
    // And the fight agrees, asked directly. **Five, not nothing** — see
    // `stats::LANE_CAP`: a lane you can commit to must never be one you can be
    // shut out of, which is `RESIST_CAP`'s argument arriving at the two lanes
    // that now have classes behind them.
    assert_eq!(gm2d_core::curse::mind_damage_after_resist(100, 400), 5);
    let dealt = gm2d_core::stats::after_defences(1000, 400, 0, 0);
    assert_eq!(dealt, 1000 * (100 - RESIST_CAP) / 100, "the fight's own clamp is not {RESIST_CAP}");
}
