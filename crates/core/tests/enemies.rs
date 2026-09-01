//! Every creature wears the catalogue, and every one of them assembles.
//!
//! The original's rule and the original's test, carried across: a monster's
//! difficulty is set by giving it better equipment, so a monster is a loadout,
//! and **a typo in a component name would silently leave one harmless.** That
//! is the failure this file exists for. It is not a hypothetical — it is why
//! upstream's README bragged about the test.

use gm2d_core::combat::{Difficulty, LADDER};
use gm2d_core::enemy_data::{BestiaryData, FORMAT};

const D: Difficulty = Difficulty::Easy;

fn path() -> String {
    format!("{}/../../data/enemies.json", env!("CARGO_MANIFEST_DIR"))
}

/// The file on disk is the ladder, written out.
#[test]
fn the_bestiary_file_matches_the_ladder() {
    let want = serde_json::to_string_pretty(&BestiaryData::of(D)).unwrap() + "\n";
    if std::env::var("REBASELINE_ENEMIES").as_deref() == Ok("1") {
        std::fs::write(path(), &want).unwrap();
        return;
    }
    let got = std::fs::read_to_string(path())
        .unwrap_or_else(|e| panic!("cannot read {}: {e}\nRegenerate with REBASELINE_ENEMIES=1", path()));
    assert_eq!(
        got, want,
        "data/enemies.json has drifted from the ladder.\n\
         If a creature changed on purpose, regenerate with REBASELINE_ENEMIES=1."
    );
}

/// **Every creature that wears gear assembles it into at least one item.**
///
/// A component name that does not exist places nothing, so the creature turns
/// up to the fight in an empty frame — alive, harmless, and indistinguishable
/// from a balance decision. Nothing else in the suite would notice.
#[test]
fn every_geared_creature_assembles_something() {
    let mut bare = Vec::new();
    for m in LADDER {
        let (reg, lo) = m.loadout_at(D);
        let items = lo.combat_items(&reg);
        if !m.gear.is_empty() && items.is_empty() {
            bare.push(m.name);
        }
    }
    assert!(
        bare.is_empty(),
        "{} creatures wear gear that assembles into nothing: {bare:?}",
        bare.len()
    );
}

/// Every component every creature names is in the catalogue.
///
/// Checked directly as well as through assembly, because a creature with two
/// items and one mistyped component still assembles — it just fights with less
/// than it was written to have.
#[test]
fn every_named_component_exists() {
    use gm2d_core::piece::CATALOG;
    let mut missing = Vec::new();
    for m in LADDER {
        for (name, ..) in m.gear_at(D) {
            if !CATALOG.iter().any(|d| d.name == name) {
                missing.push((m.name, name));
            }
        }
    }
    assert!(missing.is_empty(), "components no catalogue entry matches: {missing:?}");
}

/// Nothing fights for nothing: every creature can hurt the player somehow.
#[test]
fn every_creature_can_do_something() {
    let mut inert = Vec::new();
    for m in LADDER {
        let (reg, lo) = m.loadout_at(D);
        let armed = !lo.combat_items(&reg).is_empty() || !m.attacks.is_empty();
        if !armed {
            inert.push(m.name);
        }
    }
    assert!(inert.is_empty(), "creatures with no way to act at all: {inert:?}");
}

/// The file round-trips and refuses what it cannot read.
#[test]
fn the_bestiary_file_reads_back() {
    let text = std::fs::read_to_string(path()).unwrap();
    let d = BestiaryData::parse(&text).expect("the shipped bestiary parses");
    assert_eq!(d.enemies.len(), LADDER.len());
    assert_eq!(d.format, FORMAT);
    assert_eq!(d.difficulty, "easy");

    let rat = d.get("Cave Rat").expect("the ladder starts with a rat");
    assert!(rat.rating > 0 && rat.rating < 100, "a rat rates {}", rat.rating);
    assert!(!rat.attacks.is_empty(), "the rat fights with its teeth and has none");

    let mut wrong = d.clone();
    wrong.format = "gm2d-save".into();
    let e = BestiaryData::parse(&serde_json::to_string(&wrong).unwrap()).unwrap_err();
    assert!(e.contains(FORMAT) && e.contains("gm2d-save"), "{e}");
}
