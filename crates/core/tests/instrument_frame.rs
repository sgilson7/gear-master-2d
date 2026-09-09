//! An instrument has a frame of its own, and a save from before it opens.
//!
//! Reported from play: *"the implementation for the surveying should not
//! require a weapon ... it makes any fight you would reach on the other side
//! impossible"*. The trade `PLAN-M11.md` §8 row 4 asked for — surveying costs
//! your sword arm — was a real decision and it was the wrong one, because what
//! is through the Reach is a map you have to fight on.

mod common;

use gm2d_core::character::Character;
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;
use gm2d_core::save;

/// A file written when instruments lived in the weapon grid opens without
/// leaving map shards among the blades.
///
/// **The loader is where a field that arrives wrong is caught** — the rule
/// `World::repair` is written down under, one system across. Built by hand
/// rather than checked in, because what is being tested is the *shape* of that
/// file and not one particular character.
#[test]
fn a_save_with_an_instrument_in_the_weapon_grid_is_repaired() {
    let mut g = Game::new(11, "td");
    g.character = Character::with_all_pieces();
    g.character.loadout.name_seed = 11;
    g.character.loadout.naming = gm2d_core::theme::by_id("td").naming;
    for k in SlotKind::EVERY {
        g.character.loadout.slot_mut(k).clear();
    }

    // A weapon, and then a compass forced in beside it the way the old build's
    // file would have it. `Slot::place` does not validate, which is exactly why
    // this can happen and exactly why the loader has to answer for it.
    let handle = common::piece(&g.character, "Oak Handle");
    g.character.equip(handle, SlotKind::Weapon, 0, 0).unwrap();
    let reg = g.character.registry.clone();
    for (n, x, y) in [("Map Shard", 3, 0), ("Glass Lens", 3, 1), ("Magnet", 4, 1)] {
        let id = common::piece(&g.character, n);
        g.character.loadout.slot_mut(SlotKind::Weapon).place(&reg, id, x, y);
    }
    assert_eq!(
        g.character.loadout.slot(SlotKind::Weapon).pieces().len(),
        4,
        "the fixture did not build the old build's board",
    );

    let back = save::load(&save::save(&g)).expect("a save this build wrote loads");

    // The strays are off the weapon grid...
    let left: Vec<&str> = back
        .character
        .loadout
        .slot(SlotKind::Weapon)
        .pieces()
        .into_iter()
        .map(|p| back.character.registry.def(p).name)
        .collect();
    assert_eq!(left, vec!["Oak Handle"], "the weapon grid still holds instrument parts");

    // ...and still owned, so nothing was taken from the player.
    for n in ["Map Shard", "Glass Lens", "Magnet"] {
        assert!(
            back.character.owned.iter().any(|&p| back.character.registry.def(p).name == n),
            "{n} was lifted off the board and lost",
        );
    }
    // Loose, so the instrument frame is where they go next.
    assert!(
        back.character.instrument().is_none(),
        "a repaired board reassembled an instrument by itself",
    );
}

/// A grid the file does not name comes back at the height a player's frames are.
#[test]
fn a_save_from_before_the_frame_gets_one() {
    let g = Game::new(4, "td");
    let text = save::save(&g);
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let boards = v["state"]["character"]["boards"].as_array_mut().unwrap();
    boards.retain(|b| b[0] != "instrument");
    assert_eq!(boards.len(), 5, "the fixture did not make an old-shaped file");

    let back = save::load(&v.to_string()).expect("a five-board file still opens");
    assert_eq!(
        back.character.loadout.slot(SlotKind::Instrument).rows(),
        gm2d_core::progression::STARTING_ROWS,
        "the instrument frame came back at the engine's full height",
    );
}

/// Surveying costs no gear now, which is the whole of the report.
#[test]
fn an_instrument_leaves_the_weapon_grid_alone() {
    let mut ch = Character::starting();
    ch.apply_preset();
    let armed = ch.combat_items().len();
    assert!(armed > 0, "the fixture packs no items at all");

    for n in ["Map Shard", "Glass Lens", "Magnet"] {
        ch.give(n);
    }
    for (n, x, y) in [("Map Shard", 0, 0), ("Glass Lens", 0, 1), ("Magnet", 1, 1)] {
        let id = common::spare(&ch, n);
        ch.equip(id, SlotKind::Instrument, x, y).unwrap_or_else(|e| panic!("{n}: {e}"));
    }
    assert_eq!(ch.instrument(), Some("compass"), "the compass did not come together");
    assert_eq!(
        ch.combat_items().len(),
        armed,
        "building an instrument changed what walks into a fight",
    );
}
