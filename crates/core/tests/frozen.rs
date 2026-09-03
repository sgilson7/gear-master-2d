//! The freeze a player reported, and the class of fault under it.
//!
//! **Reported symptom:** after clearing a floor of the Drambus Stack, the
//! character stopped moving. Every reload bought one action and then nothing
//! worked. A new game was fine, and loading the save appeared to put the
//! player back on the wrong map.
//!
//! **What it actually was:** `quest::guide` asked all eleven maps whether a
//! crossing stood between the player and an errand, handing each of them
//! `world.at` — a position that belongs to exactly one map. Standing at
//! (4, 16) on the 20x20 Kettleworks field, the 16x16 Treyway was asked about
//! index 260 of its 256-tile grid and the wasm module trapped. The page catches
//! the throw and logs it, so the whole failure reads as the word
//! `unreachable` on the strip and a game that will not move.
//!
//! Nothing about the tower was involved. What the tower did was give the
//! player the errand and the walk that put them at y = 16.

use gm2d_core::combat::Difficulty;
use gm2d_core::world::Allowances;

const D: Difficulty = Difficulty::Easy;

fn maps() -> Vec<gm2d_core::world::World> {
    gm2d_core::data::MAPS.iter().map(|(id, _)| gm2d_core::data::map(id, D)).collect()
}

#[test]
fn the_reported_save_can_read_its_own_quest_log() {
    // The player's file, as they sent it. It is here rather than reconstructed
    // because the shape of the state is the bug report.
    let text = include_str!("fixtures/frozen-on-the-field.json");
    let g = gm2d_core::save::load(text).expect("the save loads");
    assert_eq!(g.world.map, "kettleworks-field", "the save is on the field");
    assert_eq!(g.world.at, [4, 16], "at the position that did it");
    assert!(
        g.world.answered.iter().any(|a| a == "the-drambus-stack-5-boss"),
        "with one floor of the Stack cleared"
    );

    let all = gm2d_core::data::quests();
    let maps = maps();
    assert!(!g.world.quests_taken.is_empty(), "and errands in hand, or this proves nothing");
    for id in &g.world.quests_taken {
        let q = all.get(id).expect("a taken errand exists");
        // The call that trapped. Any answer will do; not trapping is the test.
        let _ = gm2d_core::quest::guide(&g, q, &maps);
    }
}

#[test]
fn a_tile_off_the_map_is_nowhere_rather_than_somewhere_else() {
    // **The half that did not panic, and was worse for it.** `idx` was
    // `y * width + x`, so a coordinate one past the right-hand edge wrapped
    // into the next row and returned a real tile from somewhere else. Only an
    // overflow in `y` ran off the end and trapped, which is why this went four
    // blocks without being found.
    let allowed = Allowances::of(&[]);
    for w in maps() {
        let inside = w.terrain_name(0, 0);
        assert!(!inside.is_empty(), "{}: (0,0) has terrain", w.id);

        // One past the right-hand edge: the tile that used to come back was
        // (0, 1), and on most maps that is a real, different, passable tile.
        assert_eq!(w.terrain_name(w.width, 0), "", "{}: past the right edge is nothing", w.id);
        assert!(!w.passable(w.width, 0), "{}: and nothing off it is passable", w.id);
        assert!(w.region_at(w.width, 0).is_none(), "{}: and in no region", w.id);
        assert_eq!(w.encounter_per_mille(w.width, 0), 0, "{}: and nothing happens there", w.id);
        assert!(!w.walkable(w.width, 0, &allowed), "{}: and you cannot stand there", w.id);

        // One past the bottom: this is the one that trapped.
        assert_eq!(w.terrain_name(0, w.height), "", "{}: past the bottom is nothing", w.id);
        assert!(w.region_at(0, w.height).is_none(), "{}: and in no region", w.id);

        // And the far corner of the largest map, asked of every map. This is
        // the exact shape of the reported fault: a coordinate that is real
        // somewhere handed to a map it is not real on.
        let _ = w.region_at(19, 19);
        let _ = w.terrain_name(255, 255);
        let _ = w.encounter_per_mille(255, 255);
        let _ = w.walkable(255, 255, &allowed);
    }
}

#[test]
fn a_crossing_says_nothing_about_a_map_you_are_not_standing_on() {
    // A crossing is a rule about walking, and walking happens on the map you
    // are on. Asked about any other map it has no position to reason from, and
    // the honest answer is that it does not know.
    let g = gm2d_core::game::Game::new(7, "td");
    let allowed = g.character.allowances();
    let here = g.world.map_id();
    let maps = maps();

    let crossings: Vec<_> = maps
        .iter()
        .flat_map(|w| {
            w.places
                .iter()
                .filter(|p| matches!(p.kind, gm2d_core::world::PlaceKind::Crossing))
                .map(move |p| (w.id.clone(), p.at))
        })
        .collect();
    assert!(!crossings.is_empty(), "there are crossings to ask about");

    let mut asked_elsewhere = 0;
    for w in &maps {
        for (map, at) in &crossings {
            if map != &w.id {
                continue;
            }
            // Every tile of the region a crossing guards, asked from a state
            // standing on a *different* map, must come back as no answer.
            let mut state = g.world.clone();
            for other in maps.iter().filter(|o| o.id != w.id) {
                state.map = other.id.clone();
                state.at = other.start.into();
                assert_eq!(
                    w.crossing_between(&state, (at[0], at[1]), &allowed).map(|c| c.id.clone()),
                    None,
                    "{} was asked about a crossing while the player stood on {}",
                    w.id,
                    other.id
                );
                asked_elsewhere += 1;
            }
        }
    }
    assert!(asked_elsewhere > 0, "the question was actually asked");
    assert_eq!(here, g.world.map_id(), "and nothing moved the player");
}

#[test]
fn every_errand_can_be_guided_from_anywhere_on_any_map() {
    // **The check that would have caught it.** The fault needed one errand in
    // one stage read from one position, and no test had ever crossed an errand
    // with a map. This crosses all of them, and includes positions that are
    // off the map on purpose — `World::repair` stops a player reaching one, and
    // a hand-edited save does not have to obey `World::repair`.
    let all = gm2d_core::data::quests();
    let maps = maps();
    let mut g = gm2d_core::game::Game::new(11, "td");
    // Take every errand, so each one is past `Offered` and reaches the arm
    // that reads the maps.
    for q in &all.quests {
        g.world.quests_taken.push(q.id.clone());
    }

    let mut asked = 0;
    for w in &maps {
        let spots = [
            (0u8, 0u8),
            (w.width.saturating_sub(1), w.height.saturating_sub(1)),
            (w.width, w.height),   // one past both corners
            (19, 19),              // in bounds on the biggest map, off most
            (255, 255),
        ];
        for (x, y) in spots {
            g.world.map = w.id.clone();
            g.world.at = [x, y];
            for q in &all.quests {
                let _ = gm2d_core::quest::guide(&g, q, &maps);
                asked += 1;
            }
        }
    }
    assert!(asked >= all.quests.len() * maps.len(), "every errand on every map: {asked} readings");
}

#[test]
fn the_log_still_says_when_a_road_is_shut() {
    // **The feature the fix could have deleted in silence.** `Guide::shut` is
    // M9.4's, and it exists because a walk pressed north into the first
    // crossing for nine thousand steps while the log went on pointing at an
    // errand behind it without a word. Nothing tested it — the `shut` searched
    // for elsewhere in this suite is `PlaceDef::shut`, the map's sentence,
    // which is a different field on a different type. So narrowing
    // `crossing_between` to the map the player is standing on could have taken
    // this with it and the suite would have stayed green.
    let all = gm2d_core::data::quests();
    let maps = maps();
    let mut g = gm2d_core::game::Game::new(3, "td");
    // A level-one character standing at the start of the overworld: both
    // crossings are shut to them, since they ask for five and nine.
    let over = gm2d_core::world::overworld();
    let w = maps.iter().find(|w| w.id == over).expect("the overworld");
    g.world.map = over.clone();
    g.world.at = w.start.into();
    assert_eq!(g.character.level(), 1, "and it is level one, or nothing is shut");
    for q in &all.quests {
        g.world.quests_taken.push(q.id.clone());
    }

    let shut: Vec<_> = all
        .quests
        .iter()
        .filter_map(|q| gm2d_core::quest::guide(&g, q, &maps).shut.map(|s| (q.id.clone(), s)))
        .collect();
    assert!(
        !shut.is_empty(),
        "no errand reads as shut to a level-one character standing in the pit, so the log has \
         stopped saying when a road is shut"
    );
    for (id, why) in &shut {
        assert!(why.contains("level"), "{id}: {why:?} does not name what the road wants");
    }
}
