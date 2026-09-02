//! Enchs: bolted on, switched off, and carried across a save.
//!
//! The word is the book's (the ench economy, p. 119) and the distinction from
//! `PieceKind::Enchantment` — thirteen catalogue pieces laid *under* the grid —
//! is the whole reason it is a second word. Two mechanics, two names, no
//! rename and no migration.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::ench::{self, Refusal};
use gm2d_core::game::Game;
use gm2d_core::piece::{PieceId, SlotKind};
use gm2d_core::save;

const D: Difficulty = Difficulty::Easy;

/// A licensee with a packed board and both enchs in the rack.
fn licensee() -> Character {
    let mut c = Character::starting();
    c.apply_preset();
    c.gain_xp(500);
    c.choose_class(ench::LICENSED_CLASS).expect("the class the licence belongs to");
    for e in &data::enchs().enchs {
        c.give_ench(&e.id);
    }
    c
}

/// The component the starting weapon is built round.
fn blade(c: &Character) -> PieceId {
    c.owned
        .iter()
        .copied()
        .find(|&p| c.registry.def(p).name == "Iron Blade")
        .expect("the starting blade")
}

#[test]
fn the_shipped_enchs_parse_and_say_what_they_do() {
    let d = data::enchs();
    assert!(!d.enchs.is_empty(), "no enchs at all");
    for e in &d.enchs {
        let line = e.effect.line();
        assert!(line.chars().any(|c| c.is_ascii_digit()), "{}: {line:?} names no number", e.id);
        // TONE 13a: the spec is the engine's words, the blurb is the world's.
        for w in ["fnorp", "the funny", "cork", "fury", "devotion", "harvest"] {
            assert!(!line.to_lowercase().contains(w), "{}: the spec speaks the theme", e.id);
        }
        assert!(!e.effect.detail().is_empty(), "{}: explains nothing on hover", e.id);
    }
}

/// **The licence is the gate.** Enching is what the class is.
#[test]
fn nobody_unlicensed_bolts_anything_to_anything() {
    let mut c = Character::starting();
    c.apply_preset();
    c.give_ench("plug-energy-tap");
    let b = blade(&c);
    assert_eq!(c.attach_ench("plug-energy-tap", b), Err(Refusal::NoLicence));
    assert!(c.ench_on(b).is_none());
}

#[test]
fn nothing_may_be_enched_twice() {
    let mut c = licensee();
    let b = blade(&c);
    c.attach_ench("plug-energy-tap", b).expect("the first one goes on");
    let second = c.attach_ench("grungo-elastic-band", b);
    assert!(
        matches!(second, Err(Refusal::AlreadyEnched(_))),
        "a second ench went onto the same component: {second:?}"
    );
    // And it did not quietly spend the ench that was refused.
    assert_eq!(c.enchs_loose("grungo-elastic-band"), 1, "the refused ench was eaten");
}

/// An ench you have not got is one you cannot bolt on.
#[test]
fn you_cannot_bolt_on_what_you_have_not_got() {
    let mut c = licensee();
    let b = blade(&c);
    c.attach_ench("plug-energy-tap", b).expect("the one you have");
    let other = c
        .owned
        .iter()
        .copied()
        .find(|&p| p != b)
        .expect("a second component");
    assert_eq!(c.attach_ench("plug-energy-tap", other), Err(Refusal::NotYours));
    assert_eq!(c.attach_ench("no-such-thing", other), Err(Refusal::NoSuchEnch));
}

/// **An ench toggled off changes nothing.**
///
/// The whole promise of the toggle: trying an arrangement has to be free, or
/// nobody tries one.
#[test]
fn an_ench_toggled_off_changes_nothing() {
    let mut c = licensee();
    let b = blade(&c);
    let plain: Vec<i32> = c.combat_items().iter().map(|i| i.power).collect();

    c.attach_ench("plug-energy-tap", b).expect("it goes on");
    let on: Vec<i32> = c.combat_items().iter().map(|i| i.power).collect();
    assert_ne!(on, plain, "bolted on and nothing changed");

    assert_eq!(c.toggle_ench(b), Some(false));
    let off: Vec<i32> = c.combat_items().iter().map(|i| i.power).collect();
    assert_eq!(off, plain, "switched off and it is still doing something");

    assert_eq!(c.toggle_ench(b), Some(true));
    assert_eq!(c.combat_items().iter().map(|i| i.power).collect::<Vec<_>>(), on);
}

/// Haste is the other one, and it is the cadence rather than the power.
#[test]
fn haste_moves_the_cadence_and_nothing_else() {
    let mut c = licensee();
    let b = blade(&c);
    let before = c.combat_items();
    let was: Vec<(u32, i32)> = before.iter().map(|i| (i.cooldown_ms, i.power)).collect();
    c.attach_ench("grungo-elastic-band", b).expect("it goes on");
    let after = c.combat_items();
    let now: Vec<(u32, i32)> = after.iter().map(|i| (i.cooldown_ms, i.power)).collect();
    assert!(
        was.iter().zip(&now).any(|(a, b)| a.0 > b.0),
        "the band went on and nothing came round faster"
    );
    for (a, b) in was.iter().zip(&now) {
        assert_eq!(a.1, b.1, "a haste ench moved an item's power");
    }
}

/// **An ench follows its component, not its cell.**
///
/// Detach it from the board, turn it, seat it somewhere else — the ench is
/// still on it. Storing a cell would have meant it falling off every repack,
/// which is the one thing a player does constantly.
#[test]
fn an_ench_follows_its_component() {
    let mut c = licensee();
    let b = blade(&c);
    c.attach_ench("plug-energy-tap", b).expect("it goes on");
    let was = c.combat_items().iter().map(|i| i.power).max().unwrap_or(0);

    c.unequip(b).expect("off the board");
    assert!(c.ench_on(b).is_some(), "taking it off the board took the ench off");
    // Twice, back to lying down: a blade is one cell wide and four tall, and a
    // starting frame is three rows — upright it fits nowhere, which is the M4
    // soft-lock and not what this test is about.
    c.rotate(b).expect("turned in hand");
    assert!(c.ench_on(b).is_some(), "turning it took the ench off");
    c.rotate(b).expect("and back");
    // Back on, wherever it fits.
    let rows = c.loadout.slot(SlotKind::Weapon).rows();
    let mut seated = false;
    'find: for y in 0..rows {
        for x in 0..gm2d_core::slot::SLOT_W {
            if c.equip(b, SlotKind::Weapon, x, y).is_ok() {
                seated = true;
                break 'find;
            }
        }
    }
    assert!(seated, "the blade would not go back on the frame");
    assert!(c.ench_on(b).is_some(), "reseating it took the ench off");
    assert_eq!(
        c.combat_items().iter().map(|i| i.power).max().unwrap_or(0),
        was,
        "the ench stopped paying once the component moved"
    );
}

/// Handing a component over a counter takes the ench back rather than leaving
/// an attachment pointing at something you have not got.
#[test]
fn an_ench_comes_back_when_its_component_goes_away() {
    let mut c = licensee();
    let b = blade(&c);
    c.attach_ench("plug-energy-tap", b).expect("it goes on");
    c.owned.retain(|&p| p != b);
    c.loadout.remove_anywhere(b);
    c.tidy_enchs();
    assert!(c.ench_on(b).is_none(), "the ench is still bolted to a component you have not got");
    assert_eq!(c.enchs_loose("plug-energy-tap"), 1, "it did not come back to the rack");
}

/// **An ench survives a round trip**, on the right component.
#[test]
fn an_ench_survives_a_round_trip() {
    let mut g = Game::new(11, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.character = licensee();
    let b = blade(&g.character);
    g.character.attach_ench("plug-energy-tap", b).expect("it goes on");
    g.character.toggle_ench(b);

    let text = save::save(&g);
    let back = save::load(&text).expect("a save with an ench on it loads");

    assert_eq!(back.character.enchs_owned, g.character.enchs_owned, "the rack");
    assert_eq!(back.character.enchanted, g.character.enchanted, "what is bolted where");
    let e = back.character.ench_on(b).expect("the ench came back on the same component");
    assert_eq!(e.id, "plug-energy-tap");
    assert!(!e.active, "the switch came back the other way round");
    assert_eq!(back, g, "and the whole game, by the game's own equality");
}

/// A file written before enchs existed opens with an empty rack.
#[test]
fn a_save_from_before_enchs_still_opens() {
    let mut g = Game::new(12, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.character.apply_preset();
    let text = save::save(&g);
    // The fields are `skip_serializing_if` empty, so a game with no enchs
    // writes exactly the file an older build wrote.
    assert!(!text.contains("enchanted\":"), "an empty rack is being written into every save");
    let back = save::load(&text).expect("it loads");
    assert!(back.character.enchs_owned.is_empty());
    assert!(back.character.enchanted.is_empty());
}
