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

// ------------------------------------------------------------------ the spin

/// **An item with no room does not turn, and banks nothing.**
///
/// The whole trade: rotation is decided on the board, so leaving room to turn
/// costs you cells. Put one component in the way and the arrangement stops
/// spinning — which is the answer to "if they are blocked and cannot rotate,
/// then they do not move".
#[test]
fn an_item_with_no_room_does_not_turn() {
    let mut c = licensee();
    let b = blade(&c);
    c.attach_ench("the-ponkey-turn", b).expect("the turn goes on");

    // The starting weapon is an L — a handle three cells tall and a blade four
    // along the top — so the only orientation it can reach is the half turn.
    let mine = |c: &Character| -> gm2d_core::loadout::ItemProfile {
        c.combat_items()
            .into_iter()
            .find(|p| p.pieces.contains(&b))
            .expect("the blade is part of an item")
    };
    let cycle = mine(&c).turn_cycle.clone();
    assert_eq!(cycle.len(), 2, "the starting arrangement should have exactly one turn in it");

    // One cell, in the way of that half turn, touching nothing — so it is not
    // part of the item, it is simply standing where the item wanted to be.
    // Touching nothing of the item's, or it would *join* the item — two
    // components that touch are one item unless a lock says otherwise, and a
    // wedge that merged in would change the footprint instead of blocking it.
    let touches = |&(x, y): &(u8, u8)| {
        [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            nx >= 0 && ny >= 0 && cycle[0].contains(&(nx as u8, ny as u8))
        })
    };
    let block = cycle[1]
        .iter()
        .copied()
        .find(|c| !cycle[0].contains(c) && !touches(c))
        .expect("the half turn lands somewhere the item is not already touching");
    let wedge = c.give("Ruby Inlay").expect("a one-cell component");
    c.equip(wedge, SlotKind::Weapon, block.0, block.1).expect("one free cell");

    let after = mine(&c);
    assert_eq!(
        after.turn_cycle.len(),
        1,
        "a component stood in the way and the item still turns {} ways",
        after.turn_cycle.len()
    );

    // And over a whole fight it banks nothing, because it never turns.
    let spec = gm2d_core::combat::LADDER
        .iter()
        .find(|s| s.name == "Bog Toad")
        .expect("a toad");
    let log = gm2d_core::combat::simulate_at(c.player_stats(), &c.combat_items(), spec, D);
    let spun = log
        .entries
        .iter()
        .filter(|e| matches!(e.event, gm2d_core::combat::Event::Spun { .. }))
        .count();
    assert_eq!(spun, 0, "an item that cannot turn banked {spun} spins");
}

/// A blade with room turns, and the stacks are spent on the tick it fires.
#[test]
fn stacks_are_spent_on_activation() {
    use gm2d_core::combat::{Event, SPIN_PCT_PER_TURN};

    let mut c = licensee();
    let b = blade(&c);
    let cycle = c
        .combat_items()
        .iter()
        .find(|p| p.pieces.contains(&b))
        .map(|p| p.turn_cycle.len())
        .unwrap_or(0);
    assert!(cycle > 1, "the starting frame leaves the blade nowhere to turn ({cycle})");

    c.attach_ench("the-ponkey-turn", b).expect("the turn goes on");
    let spec = gm2d_core::combat::LADDER
        .iter()
        .find(|s| s.name == "Rust Golem")
        .expect("something that lasts a while");
    let log = gm2d_core::combat::simulate_at(c.player_stats(), &c.combat_items(), spec, D);

    let turns: Vec<_> = log
        .entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Turned { stacks, .. } => Some((e.at_ms, *stacks)),
            _ => None,
        })
        .collect();
    assert!(!turns.is_empty(), "nothing ever turned");
    let spends: Vec<_> = log
        .entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Spun { stacks, pct, .. } => Some((e.at_ms, *stacks, *pct)),
            _ => None,
        })
        .collect();
    assert!(!spends.is_empty(), "it turned and never spent anything");

    // What it spent is what it had banked, at the rate the constant names.
    for (_, stacks, pct) in &spends {
        assert_eq!(*pct, *stacks as i32 * SPIN_PCT_PER_TURN, "the spend is not the tally");
    }
    // And the tally starts again from one after every spend.
    let first_spend = spends[0].0;
    let after: Vec<u32> =
        turns.iter().filter(|(at, _)| *at > first_spend).map(|(_, n)| *n).collect();
    if let Some(&n) = after.first() {
        assert_eq!(n, 1, "the tally was not cleared when the item went off");
    }
}

/// The stack is worth something: the same fight lands harder with the turn on.
#[test]
fn a_spinning_item_hits_harder_for_having_waited() {
    let mut c = licensee();
    let b = blade(&c);
    let spec = gm2d_core::combat::LADDER
        .iter()
        .find(|s| s.name == "Rust Golem")
        .expect("something that lasts");
    let damage = |c: &Character| -> i64 {
        let log = gm2d_core::combat::simulate_at(c.player_stats(), &c.combat_items(), spec, D);
        log.entries
            .iter()
            .filter_map(|e| match e.event {
                gm2d_core::combat::Event::Hit {
                    by: gm2d_core::combat::Side::Player,
                    damage,
                    absorbed,
                    ..
                } => Some((damage + absorbed) as i64),
                _ => None,
            })
            .sum()
    };
    let plain = damage(&c);
    c.attach_ench("the-ponkey-turn", b).expect("the turn goes on");
    let spun = damage(&c);
    assert!(spun > plain, "the spin banked nothing worth having ({plain} -> {spun})");
}

/// **The feature is additive.** Nothing bolted on, nothing different.
#[test]
fn the_golden_fixture_is_unmoved() {
    let mut c = Character::starting();
    c.apply_preset();
    for p in c.combat_items() {
        assert!(!p.spins, "an item spins with nothing bolted to it");
    }
    let spec = gm2d_core::combat::LADDER
        .iter()
        .find(|s| s.name == "Bog Toad")
        .expect("a toad");
    let log = gm2d_core::combat::simulate_at(c.player_stats(), &c.combat_items(), spec, D);
    assert!(
        !log.entries.iter().any(|e| matches!(
            e.event,
            gm2d_core::combat::Event::Turned { .. } | gm2d_core::combat::Event::Spun { .. }
        )),
        "a fight with no ench in it produced spin events"
    );
}

/// Every orientation the cycle names is one the board would actually accept.
#[test]
fn a_turn_cycle_only_names_places_a_shape_can_stand() {
    let c = licensee();
    for p in c.combat_items() {
        let slot = c.loadout.slot(p.slot);
        let own: std::collections::BTreeSet<(u8, u8)> =
            p.pieces.iter().flat_map(|&id| slot.cells_of(id)).collect();
        assert!(!p.turn_cycle.is_empty(), "{}: no orientations at all", p.name);
        assert_eq!(
            p.turn_cycle[0].iter().copied().collect::<std::collections::BTreeSet<_>>(),
            own,
            "{}: the cycle does not start where the item is",
            p.name
        );
        for cells in &p.turn_cycle {
            assert_eq!(cells.len(), own.len(), "{}: an orientation changed size", p.name);
            for &(x, y) in cells {
                assert!(x < gm2d_core::slot::SLOT_W, "{}: off the right edge", p.name);
                assert!(y < slot.rows(), "{}: off the bottom", p.name);
                // Either its own cell, or an empty one.
                assert!(
                    own.contains(&(x, y)) || slot.get(x, y).is_none(),
                    "{}: an orientation stands on somebody else at ({x}, {y})",
                    p.name
                );
            }
        }
    }
}
