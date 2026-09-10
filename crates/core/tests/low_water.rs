//! M14.1 — the Treyway's south, and the tide that goes out on the tenth cairn.
//!
//! **It is its own map, and for one milestone it was not.** M14.1 drew the
//! shore into `the-treyway.tiles.json` at sixteen by twenty-six on the *one
//! country, one file* principle — the note at the top of that map already says
//! West Bambulon is a tile of it, and two files are two places to keep
//! identical. That instinct is right and it was the wrong call here, for a
//! reason that has nothing to do with content: **`#map` has been a fixed square
//! in CSS since the first map**, so a grid half again as tall as it is wide came
//! out squashed. Reported from play as *"the resolution for the overworld looks
//! all messed up"*.
//!
//! So the shore is `the-low-water`, and it is reached the way it always was —
//! over a bar of shingle the tide leaves at column 8, which is one tile of
//! `tide` on the Treyway's last row that becomes `coast` when the tenth cairn
//! goes up. **What changed is that the bar is a gate rather than two tiles of
//! the same grid.**

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const TREYWAY: &str = "the-treyway";
const SHORE: &str = "the-low-water";

/// **Every gate into the Treyway lands where it always did.**
///
/// Rows 0 to 14 are byte-for-byte what they were through both shapes of this
/// map, so no `at_to` on any road into it ever had to move — and the way to say
/// that is to walk every gate on every map and check the tile it names is a
/// tile you can stand on.
///
/// Negative-tested by inserting a row at the *top* of the file rather than the
/// bottom: the door back into Bambulon landed in the sea, and this named it.
#[test]
fn no_gate_into_the_treyway_moved() {
    let treyway = data::map(TREYWAY, D);
    let mut roads = 0;
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places.iter().filter(|p| p.kind == PlaceKind::Gate) {
            if p.to.as_deref() != Some(TREYWAY) {
                continue;
            }
            roads += 1;
            let Some([x, y]) = p.at_to else { continue };
            assert!(
                treyway.ever_walkable(x, y),
                "{id}/{}: lands at ({x}, {y}) on the Treyway, which is {:?}",
                p.id,
                treyway.terrain_name(x, y)
            );
        }
    }
    assert!(roads >= 2, "only {roads} roads into the Treyway, so this proves nothing");

    // And the two tiles the country is entered on are still what they were:
    // the door back in from Bambulon, and the way off the Reach.
    assert_eq!(treyway.terrain_name(6, 1), "plain", "the Reach's landing tile moved");
    assert!(
        treyway.place_at(13, 13).is_some_and(|p| p.id == "the-door-back"),
        "the door back into Bambulon moved"
    );
}

/// **The tide goes out on the tenth cairn, and not before.**
///
/// One tile at column 8 on the Treyway's last row, and it is the only ground
/// between the coast at row 14 and a map nobody can otherwise reach. Drawn from
/// the first visit — a player standing at the water has been able to see the
/// bar the whole time — and impassable until `built-the-tenth`, which is the
/// last thing anybody does on the Reach.
///
/// Negative-tested by drawing it `coast` in the file: the crossing was walkable
/// from the first afternoon and `the_shore_is_shut_until_the_reach_is_finished`
/// said so.
#[test]
fn the_tide_goes_out_on_the_tenth_cairn() {
    let mut st = WorldState::default();
    st.map = TREYWAY.into();
    let dry = Allowances { wade: false, level: 99 };

    let before = data::map_now(TREYWAY, D, &st);
    assert_eq!(before.terrain_name(8, 15), "tide", "(8, 15) is not the tide");
    assert!(!before.walkable(8, 15, &dry), "the tide was out before the tenth cairn");
    // **Drawn, though.** A wall you cannot see is a map that ends; a tide you
    // can see is a map with a far side you have not earned.
    assert!(before.ever_walkable(8, 15), "the tide is a wall nothing ever opens");

    // A Toad set does not open it either. `Rule::Wade` opens `water`, and the
    // sea and the tide are their own terrains for exactly this reason.
    let wading = Allowances { wade: true, level: 99 };
    assert!(!before.walkable(8, 15, &wading), "a frame walked out onto the tide");

    st.flags.push("built-the-tenth".into());
    let after = data::map_now(TREYWAY, D, &st);
    assert_eq!(after.terrain_name(8, 15), "coast", "the bar did not come out");
    assert!(after.walkable(8, 15, &dry), "the tide went out and is still a wall");
    // And it took nothing else with it: it is a bar of shingle, not a coastline.
    assert_eq!(after.terrain_name(7, 15), "sea", "the whole row went out");
    assert_eq!(after.terrain_name(9, 15), "sea", "the whole row went out");
}

/// **The shore is shut until the Reach is finished, and it is a gate that
/// shuts it.**
///
/// The crossing stands on the bar, so it is not a *hidden* place and not a
/// refusal — it is a tile you cannot stand on until the tide is out, which is
/// the whole of what a land bridge is. Asked the way M11.7's failure taught
/// this project to ask it: not *are there tiles*, but *can a walker get to them
/// from the door they come in through*.
#[test]
fn the_shore_is_shut_until_the_reach_is_finished() {
    let reach = |flags: &[&str]| -> bool {
        let mut st = WorldState::default();
        st.map = TREYWAY.into();
        st.flags = flags.iter().map(|s| s.to_string()).collect();
        let w = data::map_now(TREYWAY, D, &st);
        let a = Allowances { wade: true, level: 99 };
        let mut seen = std::collections::BTreeSet::new();
        let mut queue = vec![[13u8, 13u8]];
        seen.insert([13u8, 13u8]);
        while let Some([x, y]) = queue.pop() {
            for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let (nx, ny) = (nx as u8, ny as u8);
                if w.walkable(nx, ny, &a) && seen.insert([nx, ny]) {
                    queue.push([nx, ny]);
                }
            }
        }
        seen.contains(&[8, 15])
    };

    assert!(!reach(&[]), "the bar was walkable before the tide went out");
    assert!(reach(&["built-the-tenth"]), "the tenth cairn went up and the bar is still a wall");

    // And the crossing stands on it, which is what makes the far side a map
    // rather than a rumour.
    let w = data::map(TREYWAY, D);
    let bar = w.place_at(8, 15).expect("nothing on the bar");
    assert_eq!(bar.id, "the-tide-crossing");
    assert_eq!(bar.to.as_deref(), Some(SHORE));
    assert!(!bar.prose.is_empty(), "you cross to another country and nothing is said");
}

/// **The shore is a map, a band of its own, and it goes back where it came
/// from.**
///
/// `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` says the
/// winnable half over every map; what is here is that it is *harder than the
/// tile you cross from*, and that the way back lands on the coast rather than
/// in the sea.
#[test]
fn the_shore_is_a_band_of_its_own() {
    let w = data::map(SHORE, D);
    assert_eq!((w.width, w.height), (16, 11), "the shore is not the shape it was drawn");
    assert_eq!(w.regions.len(), 1, "the shore is one band");
    let low = &w.regions[0];
    assert_eq!(low.id, "the-low-water");

    let first = data::map(TREYWAY, D)
        .regions
        .iter()
        .find(|r| r.id == "the-first-treyway")
        .map(|r| r.danger)
        .expect("the door's band");
    assert!(
        low.danger > first,
        "the shore ({}) is no harder than the coast you cross from ({first})",
        low.danger
    );

    // Every walkable tile of it is in that band — the check that would have
    // caught a box drawn one row short.
    for y in 0..w.height {
        for x in 0..w.width {
            if w.passable(x, y) {
                assert_eq!(
                    w.region_at(x, y).map(|r| r.id.as_str()),
                    Some("the-low-water"),
                    "({x}, {y}) is walkable and is in no band"
                );
            }
        }
    }

    // And the way back is on the bar, landing on the coast it came from.
    let back = w.place_at(8, 1).expect("no way back over the tide");
    assert_eq!(back.to.as_deref(), Some(TREYWAY));
    assert_eq!(back.at_to, Some([8, 14]), "the way back lands somewhere else");
    let t = data::map(TREYWAY, D);
    assert_eq!(t.terrain_name(8, 14), "plain", "it lands in the sea");
    // **Not on the bar itself.** Arriving on the gate you came through is a
    // tile you have to step off before you can step back.
    assert_ne!(w.start, (8, 1), "the shore starts you standing on the way off it");
}

/// **The lip of the Wextreen Sump is on the shore, and it is a stack of four.**
#[test]
fn the_lip_is_a_stack_and_it_wants_an_instrument() {
    let w = data::map(SHORE, D);
    let lip = w.place_at(7, 6).expect("nothing at the lip");
    assert_eq!(lip.id, "the-lip-of-the-sump");
    assert!(lip.needs_survey, "the lip opens for anybody");
    assert!(lip.to.is_none(), "the lip names one map as well as four floors");
    assert_eq!(lip.floors.len(), 4, "the Sump is four floors");
    assert!(!lip.shut.is_empty(), "a shut door that says nothing");

    // **Top down, and each floor is done when its own thing is done.** Three of
    // the four are flags, because three of the four have no boss on them.
    let mut st = WorldState::default();
    assert_eq!(lip.opens_onto(&st), Some("the-sump-1"));
    st.flags.push("sluice-b".into());
    assert_eq!(lip.opens_onto(&st), Some("the-sump-2"), "a solved floor is still the way in");
    st.flags.push("the-shelf-is-open".into());
    st.flags.push("cairn-9".into());
    assert_eq!(lip.opens_onto(&st), Some("the-sump-4"));
    st.answered.push("the-ninth-surveyor".into());
    assert_eq!(lip.opens_onto(&st), None, "the Sump is finished and still has a floor in it");
    assert_eq!(lip.floors_cleared(&st), 4);

    // And the way back up out of the first floor lands beside it.
    let up = data::map("the-sump-1", D)
        .places
        .iter()
        .find(|p| p.id == "the-sump-1-up")
        .map(|p| (p.to.clone(), p.at_to))
        .expect("the Sump's first floor has no way up");
    assert_eq!(up.0.as_deref(), Some(SHORE));
    assert_eq!(up.1, Some([7, 6]), "coming up out of the Sump does not put you at its lip");
}

// -------------------------------------------- a shut door that says so

/// **A place you can be refused at has to say why.**
///
/// Reported from play, standing at the shore: *"i cannot go to the southern
/// area in my save, the land is pink and it says no way through"*. It is one
/// tile of `tide` and `world::step` refuses on `walkable` **before** anything
/// asks the place, so the sentence a player got was the sentence a cliff gets
/// — no tide, no cairn, no Reach, and nothing that could be acted on.
///
/// **"Standing on impassable ground" is not the condition, and the first
/// version of this check used it.** Two gates in this game are on ground nobody
/// can walk on, and only one of them is a place anybody can be *refused* at:
///
/// ```text
/// west-bambulon: the-way-under-the-lake at [8, 11] on water -- standable neighbours: []
/// the-treyway:   the-tide-crossing      at [8, 15] on tide  -- standable neighbours: [[8, 14]]
/// ```
///
/// The grating is in **open water in the middle of the lake**: without
/// `Rule::Wade` you cannot reach a tile beside it, and with `Rule::Wade` the
/// water is walkable and the gate simply opens. So a `shut` on it is a sentence
/// no player can ever read — dead content, which is the thing M15.5 had just
/// caught in a skill node's hover. It was written and is deleted, and this
/// check is what would notice the day somebody drew land at [8, 12].
///
/// The refusal itself is content — `shut`, in the map file, in the world's
/// words, TONE 12. What the engine owns is only that it is a *place's* refusal
/// and therefore goes on the strip rather than in the one-second flash.
#[test]
fn a_place_you_can_be_refused_at_says_why() {
    let plain = Allowances::default();
    let mut refusable = 0;
    let mut unreachable = Vec::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in &w.places {
            if w.walkable(p.at[0], p.at[1], &plain) {
                continue;
            }
            // Somewhere a player could be standing when the step is refused.
            let (x, y) = (p.at[0] as i32, p.at[1] as i32);
            let beside = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                w.in_bounds(nx, ny) && w.walkable(nx as u8, ny as u8, &plain)
            });
            if !beside {
                unreachable.push(format!("{id}: {}", p.id));
                assert!(
                    p.shut.is_empty(),
                    "{id}: {} is in open ground nobody can reach and carries a refusal \
                     no player can ever read",
                    p.id
                );
                continue;
            }
            refusable += 1;
            assert!(
                !p.shut.is_empty(),
                "{id}: {} stands on {} with somewhere to be refused from, and has nothing \
                 to say about it, so the only sentence a player can get there is the cliff's",
                p.id,
                w.terrain_name(p.at[0], p.at[1])
            );
        }
    }
    assert!(refusable >= 1, "nothing in the game can refuse a step this way, so this is vacuous");
    // **The exception is asserted rather than skipped**, the same way
    // `common::UNWRITTEN` and `avail.rs`'s `STAGED` are: a list that quietly
    // grew is a list that has gone stale.
    assert_eq!(
        unreachable,
        vec!["west-bambulon: the-way-under-the-lake".to_string()],
        "the set of places nobody can be refused at has changed"
    );
}

/// And the sentence actually comes back out of the step, on the strip.
///
/// **The half a data check cannot see.** `shut` being written is one thing;
/// `walkable` returning before anything reads it is what the report was.
#[test]
fn the_tide_says_what_is_over_the_bar() {
    let mut g = gm2d_core::game::Game::new(3, "td");
    g.world = WorldState::at_start(&data::map(TREYWAY, D));
    g.world.map = TREYWAY.into();
    g.world.at = [8, 14];
    let allowed = g.character.allowances();
    let live = data::map_now(TREYWAY, D, &g.world);
    let s = gm2d_core::world::step(
        &live,
        &mut g.world,
        &mut g.rng,
        D,
        gm2d_core::world::Dir::South,
        &allowed,
    );
    assert!(!s.moved, "the bar is under water and the step went through anyway");
    let said = s.blocked.clone().unwrap_or_default();
    assert!(
        said.contains("tenth"),
        "the refusal does not name what opens it: {said:?}"
    );
    assert_eq!(
        s.refused_by.as_deref(),
        Some("the-tide-crossing"),
        "the sentence would go in the flash rather than on the strip"
    );

    // **And it stops saying it once the tide is out**, which is the second
    // visit this project keeps forgetting to check.
    g.world.at = [8, 14];
    g.world.flags.push("built-the-tenth".into());
    let live = data::map_now(TREYWAY, D, &g.world);
    let s = gm2d_core::world::step(
        &live,
        &mut g.world,
        &mut g.rng,
        D,
        gm2d_core::world::Dir::South,
        &allowed,
    );
    assert!(s.moved, "the tenth cairn went up and the bar is still a wall");
    assert_eq!(s.gate.as_deref(), Some("the-tide-crossing"));
}

/// **Nobody can be refused at the grating**, which is why it says nothing.
///
/// The old version of this test planted the character on `[8, 10]` and stepped
/// south. That tile is **water** — it is not a position a player can be in, and
/// `World::repair` moves anybody who loads into it. So the check was asserting a
/// refusal that only a character standing illegally in a lake could ever
/// produce, which is the *a planted check is about what was planted* family
/// with the plant itself wrong.
///
/// What is true is stated instead, and it is the reason the sentence came back
/// out of the map file.
#[test]
fn the_grating_is_in_open_water_and_refuses_nobody() {
    let w = data::map("west-bambulon", D);
    let gate = w
        .places
        .iter()
        .find(|p| p.id == "the-way-under-the-lake")
        .expect("the way under the lake is on the map");
    let plain = Allowances::default();
    assert!(!w.walkable(gate.at[0], gate.at[1], &plain), "the grating is on dry land now");
    let (x, y) = (gate.at[0] as i32, gate.at[1] as i32);
    for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
        let (nx, ny) = (x + dx, y + dy);
        if !w.in_bounds(nx, ny) {
            continue;
        }
        assert!(
            !w.walkable(nx as u8, ny as u8, &plain),
            "[{nx}, {ny}] is beside the grating and standable, so the refusal is reachable \
             and the gate needs a `shut` again"
        );
    }
    assert!(gate.shut.is_empty(), "a refusal nobody can read is dead content");
}
