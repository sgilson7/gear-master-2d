//! The first dungeon, and the two keys either side of it.
//!
//! A dungeon here is a corridor with something certain at the bottom — the
//! only fight in the game that is not a draw against a region's pool. Its
//! shortness is deliberate: fatigue is what makes it a decision, so you walk
//! in with a budget and the walk back out is part of it.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::world::PlaceKind;

const D: Difficulty = Difficulty::Easy;

#[test]
fn every_shipped_map_loads_and_has_an_id_of_its_own() {
    let mut seen: Vec<String> = Vec::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        assert_eq!(&w.id, id, "{id} loads as {:?}", w.id);
        assert!(!seen.contains(&w.id), "two maps answer to {id}");
        seen.push(w.id.clone());
        assert!(w.width > 0 && w.height > 0, "{id} is empty");
        assert!(w.passable(w.start.0, w.start.1), "{id} starts you inside the scenery");
    }
    assert!(seen.len() >= 2, "there is only one map, so nothing here is being tested");
}

/// **A map this build has not got does not panic.**
///
/// A save can name one — a file from a later version, or a dungeon that was
/// renamed — and the overworld is a recoverable answer where a panic is not.
#[test]
fn an_unknown_map_falls_back_rather_than_panicking() {
    let w = data::map("a-map-that-was-never-written", D);
    assert_eq!(w.id, gm2d_core::world::overworld());
}

/// Every gate goes somewhere real, and lands you somewhere you can stand.
///
/// A gate whose far side is a wall is a way to strand a player on a map with
/// no way off it, and nothing else in the game would say so.
#[test]
fn every_gate_leads_somewhere_you_can_stand() {
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in w.places.iter().filter(|p| p.kind == PlaceKind::Gate) {
            // **A gate names one map or a stack of them.** The Drambus Stack's
            // door opens onto whichever of its five floors is still standing,
            // so every one of them has to be a map you can land on — the door
            // is walked through five times and each time it is a different one.
            let mut goes: Vec<&str> = p.floors.iter().map(|f| f.map.as_str()).collect();
            if goes.is_empty() {
                goes.push(p.to.as_deref().unwrap_or_else(|| panic!("{}: a gate to nowhere", p.id)));
            } else {
                assert!(p.to.is_none(), "{}: a stack that also names one map", p.id);
                // Each floor's mark is the boss standing on it, or the door
                // would open onto a floor that can never be cleared.
                for f in &p.floors {
                    let floor = data::map(&f.map, D);
                    assert_eq!(floor.id, f.map, "{}: {:?} is not a map", p.id, f.map);
                    assert!(
                        floor.places.iter().any(|b| b.id == f.cleared),
                        "{}: nothing on {} is called {:?}, so the floor never clears",
                        p.id,
                        f.map,
                        f.cleared
                    );
                }
            }
            for to in goes {
            assert!(
                data::MAPS.iter().any(|(m, _)| *m == to),
                "{}: opens onto {to:?}, which is not a map",
                p.id
            );
            // **A gate may name its landing tile or leave it to the far
            // side.** `at_to` is a dungeon's mouth: one door, one tile, the
            // trip round a constant. Leaving it out is a border, and the far
            // side lands you where you last stood on it — falling back to that
            // map's start the first time. Both have to be somewhere you can
            // stand, and the start already is, so this checks the one a gate
            // actually names.
            let dest = data::map(to, D);
            let at = p.at_to.unwrap_or([dest.start.0, dest.start.1]);
            assert!(
                dest.passable(at[0], at[1]),
                "{}: lands you on {:?} of {to}, which is not walkable",
                p.id,
                at
            );
            }
            // And what it wants, if it wants anything, is a real component.
            if let Some(n) = &p.needs {
                assert!(
                    gm2d_core::shop::def_named(n).is_some(),
                    "{}: wants {n:?}, which is not a component",
                    p.id
                );
                assert!(!p.shut.is_empty(), "{}: locked and says nothing about it", p.id);
            }
        }
    }
}

/// **The way in and the way out.**
///
/// A dungeon you can enter and not leave is a save nobody recovers from.
#[test]
fn the_cave_can_be_entered_and_left() {
    let over = data::map(&gm2d_core::world::overworld(), D);
    let door = over
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Gate && p.to.as_deref() == Some("the-great-gear-cave"))
        .expect("a way into the cave");
    assert_eq!(door.needs.as_deref(), Some("The Witch's Key"), "the cave is not locked");

    let cave = data::map("the-great-gear-cave", D);
    let back = cave
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Gate && p.to.as_deref() == Some(over.id.as_str()))
        .expect("a way back out of the cave");
    // You do not arrive standing on the way out, or the first step would be a
    // step back through it.
    assert_ne!(back.at, [cave.start.0, cave.start.1], "you arrive on the exit");
    assert!(back.needs.is_none(), "the way out is locked");
}

/// The boss is at the bottom, is a real creature, and leaves a real thing.
#[test]
fn the_bottom_of_the_cave_is_something_certain() {
    let cave = data::map("the-great-gear-cave", D);
    let bosses: Vec<_> = cave.places.iter().filter(|p| p.kind == PlaceKind::Boss).collect();
    assert_eq!(bosses.len(), 1, "a short dungeon has one thing at the end of it");
    let b = bosses[0];
    let who = b.creature.as_deref().expect("the boss is somebody");
    assert!(
        gm2d_core::combat::LADDER.iter().any(|s| s.name == who),
        "the boss is {who:?}, which is not a creature"
    );
    let drop = b.drops.as_deref().expect("the boss leaves something");
    assert!(gm2d_core::shop::def_named(drop).is_some(), "it leaves {drop:?}, which is nothing");

    // It is harder than what the cave throws at you on the way down, or the
    // corridor is the fight and the boss is a formality.
    let boss = gm2d_core::combat::LADDER.iter().find(|s| s.name == who).unwrap();
    let boss_rating = gm2d_core::rating::creature_rating(boss, D);
    for r in &cave.regions {
        for m in &r.enemies {
            assert!(
                gm2d_core::rating::creature_rating(m, D) < boss_rating,
                "{} rates as high as the thing at the bottom",
                m.name
            );
        }
    }
}

/// **The dungeon is short.** That is the design, not an accident of writing.
#[test]
fn the_first_dungeon_is_short() {
    let cave = data::map("the-great-gear-cave", D);
    let walkable = (0..cave.width)
        .flat_map(|x| (0..cave.height).map(move |y| (x, y)))
        .filter(|&(x, y)| cave.passable(x, y))
        .count();
    assert!(
        walkable <= 40,
        "{walkable} walkable tiles — a first dungeon that size is a second overworld"
    );
    assert!(walkable >= 8, "{walkable} walkable tiles is a room, not a corridor");
}

/// The key the cave wants is the key an errand pays, and the key it leaves is
/// paid by nothing else.
///
/// A lock whose key is not handed out anywhere is a door nobody opens.
#[test]
fn the_witchs_key_is_the_key_the_cave_wants() {
    let over = data::map(&gm2d_core::world::overworld(), D);
    let wants: Vec<&str> = over
        .places
        .iter()
        .filter(|p| p.kind == PlaceKind::Gate)
        .filter_map(|p| p.needs.as_deref())
        .collect();
    // **Two faucets, and a key wants exactly one of them.** An errand pays the
    // Cave's; the Cave's own boss pays the wall's, looked up by the tile. A
    // third kind of source would be a third place to look when a lock turns out
    // to open for nobody, so the list is closed and this is where it is stated.
    let mut paid: Vec<String> =
        data::quests().quests.iter().flat_map(|q| q.reward.iter().cloned()).collect();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places.iter() {
            if let Some(d) = &p.drops {
                paid.push(d.clone());
            }
        }
    }
    for w in &wants {
        assert!(paid.iter().any(|r| r == w), "{w:?} opens a gate and nothing hands it out");
    }
    // And the errand that pays the Cave's key is behind another one, which is
    // what makes it a questline rather than a fetch.
    let quests = data::quests();
    let giver = quests
        .quests
        .iter()
        .find(|q| q.reward.iter().any(|r| wants.contains(&r.as_str())))
        .expect("something pays a gate's key");
    assert!(!giver.requires.is_empty(), "{}: the key is one errand deep", giver.id);
}

/// **Beating the thing at the bottom leaves the key, and only there.**
///
/// Looked up by the tile rather than by the creature, because the same
/// creature stands in a region's pool as an ordinary encounter — beating a
/// Rust Colossus in a field must not hand over the way to the next map.
#[test]
fn the_boss_drops_its_key_and_a_field_fight_does_not() {
    use gm2d_core::character::Character;
    use gm2d_core::fight;
    use gm2d_core::game::Game;

    let cave = data::map("the-great-gear-cave", D);
    let boss = cave.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("a boss");
    let who = boss.creature.clone().unwrap();
    let key = boss.drops.clone().unwrap();

    let mut g = Game::new(3, "td");
    // Strong enough to win: what is being tested is the drop, not the fight.
    g.character = Character::with_all_pieces();
    g.character.apply_preset();
    g.world.map = cave.id.clone();

    // In a field first — same creature, wrong tile.
    g.world.at = [2, 1];
    g.encounter = Some(fight::Encounter { enemy: who.clone(), at: g.world.at });
    let log = fight::run(&g, D).unwrap();
    fight::settle(&mut g, &log, D).unwrap();
    assert_eq!(
        gm2d_core::quest::holding(&g, &key),
        0,
        "beating one in a field handed over the key"
    );

    // Now on the tile it is standing on.
    g.world.at = boss.at;
    g.encounter = Some(fight::Encounter { enemy: who.clone(), at: g.world.at });
    let log = fight::run(&g, D).unwrap();
    assert_eq!(log.outcome, gm2d_core::combat::Outcome::Victory, "the test board lost");
    let s = fight::settle(&mut g, &log, D).unwrap();
    assert_eq!(gm2d_core::quest::holding(&g, &key), 1, "the boss left nothing");
    // **In the theme's words.** The receipt used to name the key canonically,
    // which is the one screen in the game that called it that; M9.1 put the
    // three lines that hand a component over through `Game::theme_piece`.
    let said = g.theme_piece(&key);
    assert_ne!(said, key, "the theme has a name for the key and this is testing that it is used");
    assert!(
        s.receipt.iter().any(|l| l.contains(&said)),
        "the receipt does not mention it: {:?}",
        s.receipt
    );

    // And it does not drop twice.
    g.encounter = Some(fight::Encounter { enemy: who, at: g.world.at });
    let log = fight::run(&g, D).unwrap();
    fight::settle(&mut g, &log, D).unwrap();
    assert_eq!(gm2d_core::quest::holding(&g, &key), 1, "the boss left a second key");
}

