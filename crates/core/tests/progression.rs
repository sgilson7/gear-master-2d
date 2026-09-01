//! Levels, rows and points.
//!
//! M4's acceptance. The last test in this file is the one that matters most:
//! the plan committed to level 5 arriving in 25–35 fights, and that band is a
//! contract the shipped map has to hold rather than a hope about it.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;
use gm2d_core::progression::{self, MAX_LEVEL, STARTING_ROWS, XP_TO_NEXT};
use gm2d_core::save;
use gm2d_core::skills::Refusal;

const D: Difficulty = Difficulty::Easy;

// ------------------------------------------------------------------ the curve

/// The table is the formula. Generated rather than typed, and checked here so
/// the two cannot drift.
#[test]
fn the_table_matches_the_formula() {
    for level in 1..=MAX_LEVEL as u32 {
        let want = (20.0_f64 * 1.35_f64.powi(level as i32 - 1)).round() as i32;
        assert_eq!(
            progression::xp_to_next(level),
            want,
            "level {level} costs {} and the formula says {want}",
            progression::xp_to_next(level)
        );
    }
    assert_eq!(XP_TO_NEXT[0], 20);
}

/// Reaching level 5 costs what the plan says it costs.
#[test]
fn level_five_costs_a_hundred_and_thirty_two() {
    assert_eq!(progression::xp_to_reach(5), 132);
    assert_eq!(progression::level_for(131), 4);
    assert_eq!(progression::level_for(132), 5);
}

/// The level is a function of the total, and the total alone.
#[test]
fn a_level_is_derived_from_experience() {
    let mut last = 1;
    for xp in 0..3000 {
        let l = progression::level_for(xp);
        assert!(l >= last, "the level went backwards at {xp}");
        assert!(l >= 1);
        last = l;
    }
    let (into, needed) = progression::progress(progression::xp_to_reach(4) + 5);
    assert_eq!(into, 5);
    assert_eq!(needed, progression::xp_to_next(4));
}

// ------------------------------------------------------------------ the rows

/// **Board dimensions are a pure function of level plus granted rows.**
///
/// The acceptance criterion in the plan's own words. Checked across every level
/// the table covers, because the failure it guards is a board that agrees with
/// the formula at level 5 and disagrees at level 11.
#[test]
fn board_size_is_a_function_of_level() {
    for level in 1..=MAX_LEVEL as u32 {
        let total: u8 = SlotKind::ALL.iter().map(|&k| progression::rows_for(k, level)).sum();
        let expected = (5 * STARTING_ROWS as u32 + (level - 1)).min(5 * 8) as u8;
        assert_eq!(
            total, expected,
            "at level {level} the five grids total {total} rows and should total {expected}"
        );
    }
    // The rotation, in the plan's order.
    assert_eq!(progression::grows_at(1), None, "level 1 is where you start");
    assert_eq!(progression::grows_at(2), Some(SlotKind::Weapon));
    assert_eq!(progression::grows_at(3), Some(SlotKind::Chest));
    assert_eq!(progression::grows_at(4), Some(SlotKind::Helmet));
    assert_eq!(progression::grows_at(5), Some(SlotKind::Gloves));
    assert_eq!(progression::grows_at(6), Some(SlotKind::Greaves));
    assert_eq!(progression::grows_at(7), Some(SlotKind::Weapon), "and round again");
}

/// A character's real boards match what the level implies.
#[test]
fn a_levelled_character_has_the_boards_its_level_implies() {
    let mut c = Character::starting();
    for k in SlotKind::ALL {
        assert_eq!(c.loadout.slot(k).rows(), STARTING_ROWS, "{k:?} did not start at three");
    }
    c.gain_xp(progression::xp_to_reach(6));
    assert_eq!(c.level(), 6);
    c.resize_boards([0; 5]);
    for k in SlotKind::ALL {
        assert_eq!(
            c.loadout.slot(k).rows(),
            progression::rows_for(k, 6),
            "{k:?} is the wrong height at level 6"
        );
    }
}

/// Growing a board never shrinks one.
///
/// A board that got shorter would drop whatever was seated in the rows it lost,
/// silently, and the player would find out in a fight.
#[test]
fn boards_only_ever_grow() {
    let mut c = Character::starting();
    c.gain_xp(progression::xp_to_reach(9));
    c.resize_boards([2, 2, 2, 2, 2]);
    let tall: Vec<u8> = SlotKind::ALL.iter().map(|&k| c.loadout.slot(k).rows()).collect();
    // Re-apply with nothing granted: the rows a skill gave must not be taken
    // back, because the skill is still taken.
    c.resize_boards([0; 5]);
    let after: Vec<u8> = SlotKind::ALL.iter().map(|&k| c.loadout.slot(k).rows()).collect();
    assert_eq!(tall, after, "a board shrank");
}

// ------------------------------------------------------------------ the tree

/// **No node can be bought twice, without its prerequisite, or without a
/// point.** The three refusals the plan names.
#[test]
fn the_three_refusals() {
    let tree = data::skills();
    let mut c = Character::starting();

    // Without a point.
    assert_eq!(
        c.take_skill(&tree, "frame-sense"),
        Err(Refusal::NotEnoughPoints { need: 1, have: 0 })
    );

    c.skill_points = 5;

    // Without its prerequisite.
    match c.take_skill(&tree, "second-frame") {
        Err(Refusal::Missing(what)) => assert_eq!(what, "Frame Sense"),
        other => panic!("expected a missing prerequisite, got {other:?}"),
    }

    // And once it is met, it works.
    c.take_skill(&tree, "frame-sense").expect("frame-sense");
    c.take_skill(&tree, "second-frame").expect("second-frame");

    // Twice.
    assert_eq!(c.take_skill(&tree, "frame-sense"), Err(Refusal::AlreadyTaken));

    // A node nobody wrote.
    assert_eq!(c.take_skill(&tree, "no-such-thing"), Err(Refusal::NoSuchNode));

    assert_eq!(c.skill_points, 3, "points were spent wrongly");
}

/// A node's effect reaches the character the moment it is bought.
#[test]
fn a_bought_node_does_something_immediately() {
    let tree = data::skills();
    let mut c = Character::starting();
    c.skill_points = 6;

    let rows = c.loadout.slot(SlotKind::Weapon).rows();
    c.take_skill(&tree, "frame-sense").unwrap();
    assert_eq!(
        c.loadout.slot(SlotKind::Weapon).rows(),
        rows + 1,
        "Frame Sense granted no row"
    );

    let hp = c.player_stats().health;
    c.take_skill(&tree, "cave-lungs").unwrap();
    assert_eq!(c.player_stats().health, hp + 60, "Cave Lungs granted no health");

    c.take_skill(&tree, "flush-fit").unwrap();
    assert_eq!(c.loadout.assembly_pct, 10, "Flush Fit changed no rule");
}

/// Every node in the shipped tree is reachable, and every prerequisite exists.
///
/// A node whose prerequisite is misspelled is a node no player can ever take,
/// and nothing else would notice.
#[test]
fn the_shipped_tree_is_coherent() {
    let tree = data::skills();
    let base = tree.base().expect("a base tree");
    assert!(
        (10..=15).contains(&base.nodes.len()),
        "the base tree has {} nodes and the plan asks for 10 to 15",
        base.nodes.len()
    );

    // Every tree, not only the base one: a class tree with a misspelled
    // prerequisite is a node no player can ever take, and it would sit there
    // through the whole of M5 without anything noticing.
    for t in &tree.trees {
        let ids: Vec<&str> = t.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut seen: Vec<&str> = Vec::new();
        for n in &t.nodes {
            assert!(!seen.contains(&n.id.as_str()), "{} appears twice in {}", n.id, t.id);
            seen.push(&n.id);
            assert!(!n.blurb.is_empty(), "{} has no blurb", n.id);
            for r in &n.requires {
                assert!(
                    ids.contains(&r.as_str()),
                    "{} requires {r:?}, which is not in its own tree {}",
                    n.id,
                    t.id
                );
            }
        }
        // And reachable by spending in some order.
        let mut taken: Vec<String> = Vec::new();
        let mut progress = true;
        while progress {
            progress = false;
            for n in &t.nodes {
                if !taken.contains(&n.id) && n.requires.iter().all(|r| taken.contains(r)) {
                    taken.push(n.id.clone());
                    progress = true;
                }
            }
        }
        assert_eq!(taken.len(), t.nodes.len(), "{} has unreachable nodes", t.id);
    }

    let ids: Vec<&str> = base.nodes.iter().map(|n| n.id.as_str()).collect();
    for n in &base.nodes {
        assert!(!n.blurb.is_empty(), "{} has no blurb", n.id);
        for r in &n.requires {
            assert!(ids.contains(&r.as_str()), "{} requires {r:?}, which is not in the tree", n.id);
            assert_ne!(r, &n.id, "{} requires itself", n.id);
        }
        for e in &n.effects {
            if let gm2d_core::skills::Effect::GrowSlotRows { slot, .. } = e {
                assert!(
                    gm2d_core::skills::slot_of(slot).is_some(),
                    "{} grows {slot:?}, which is not a slot",
                    n.id
                );
            }
        }
    }

    // Every node can be reached by spending points in some order.
    let mut taken: Vec<String> = Vec::new();
    let mut progress = true;
    while progress {
        progress = false;
        for n in &base.nodes {
            if taken.contains(&n.id) {
                continue;
            }
            if n.requires.iter().all(|r| taken.contains(r)) {
                taken.push(n.id.clone());
                progress = true;
            }
        }
    }
    assert_eq!(taken.len(), base.nodes.len(), "some nodes are unreachable: {taken:?}");
}

// ------------------------------------------------------------------ the save

/// A level-5 save reloads with the same boards, the same points and the same
/// gold. The plan's acceptance, word for word.
#[test]
fn a_level_five_save_comes_back_the_same() {
    let tree = data::skills();
    let mut g = Game::new(0x5EED_1234_ABCD_0001, "td");
    g.character.gain_xp(progression::xp_to_reach(5));
    g.character.resize_boards([0; 5]);
    g.character.take_skill(&tree, "frame-sense").unwrap();
    g.character.take_skill(&tree, "corked").unwrap();
    g.character.gold = 91;

    assert_eq!(g.character.level(), 5);
    let after = save::load(&save::save(&g)).expect("a level-five save loads");

    assert_eq!(after.character.level(), 5, "the level did not survive");
    assert_eq!(after.character.xp, g.character.xp);
    assert_eq!(after.character.skill_points, g.character.skill_points);
    assert_eq!(after.character.skills_taken, g.character.skills_taken);
    assert_eq!(after.character.gold, 91);
    assert_eq!(after.character.slot_rows(), g.character.slot_rows(), "the boards changed height");
    assert_eq!(
        after.character.player_stats(),
        g.character.player_stats(),
        "the character sheet moved across a save"
    );
    assert_eq!(after, g, "the game as a whole did not survive");
}

/// The level is derived on load rather than stored, so a hand-edited total
/// produces a consistent character rather than a contradictory one.
#[test]
fn the_level_is_never_stored() {
    let text = save::save(&Game::new(7, "td"));
    assert!(!text.contains("\"level\""), "the save stores a level, which can disagree with the xp");
    assert!(text.contains("\"xp\""));
}

// ------------------------------------------------------------------ the pit

/// **A starting character can win in the region it starts in.**
///
/// The test M4 needed and did not have. A loss pays nothing, so a character who
/// cannot beat anything has no income, no experience and no way to buy out of
/// it — the game is unwinnable from its own first tile, and every other test
/// still passes. It shipped that way for an afternoon: `PRESET` is an eight-row
/// arrangement and `Balanced Grip` is four cells tall, so on a three-row frame
/// the starting weapon had no handle and assembled nothing.
#[test]
fn a_starting_character_can_win_in_the_pit() {
    use gm2d_core::combat::{simulate_at, Outcome};

    let mut c = Character::starting();
    c.apply_preset();
    let items = c.combat_items();
    assert!(
        items.iter().any(|i| i.slot == SlotKind::Weapon),
        "the starting kit assembles no weapon, so it cannot hurt anything"
    );

    let w = data::world(D);
    let pit = w
        .regions
        .iter()
        .find(|r| r.id == "the-end-of-all-gears")
        .expect("the starting region");

    let stats = c.player_stats();
    let wins = pit
        .enemies
        .iter()
        .filter(|m| simulate_at(stats, &items, m, D).outcome == Outcome::Victory)
        .count();
    assert!(
        wins > 0,
        "a starting character beats none of the {} creatures in the region it starts in",
        pit.enemies.len()
    );
}

// ------------------------------------------------------------------ the band

/// **Level 5 arrives in 25 to 35 fights.**
///
/// The plan committed to this band, and `XP_DIVISOR` is what is tuned to hold
/// it. Not a hope about the map — a measurement of it. Retuning the map's
/// regions can move this, which is the point: the band is the contract, and
/// whoever moves the map has to look at what it did to the grind.
///
/// The walk is the real one, and so are the fights. An earlier version banked
/// experience for every encounter as though it were a win, which measured how
/// much the map *offers* rather than how much a player *gets* — and would have
/// gone on passing while the starting kit lost every fight it met. Losses are
/// fought and lost here, and pay nothing.
#[test]
fn level_five_lands_where_the_plan_says() {
    use gm2d_core::world::{step, Dir, WorldState};

    let w = data::world(D);
    // East and west along the pit's road, which is what grinding the first
    // region looks like. An earlier version patrolled north as well and
    // measured about three thousand fights — honestly, because a scrap board
    // loses everything in the Slag Flats. That is a fact about the map's
    // gradient rather than about its pacing, and pacing is what this measures.
    let patrol = [
        Dir::East, Dir::East, Dir::East, Dir::East, Dir::East, Dir::East,
        Dir::West, Dir::West, Dir::West, Dir::West, Dir::West, Dir::West,
    ];

    // Nine seeds, and the assertion is on the mean. A per-seed band would be a
    // band tuned to whichever walks happened to be checked; what is being
    // asserted is the pacing of the map, which is an average over players.
    let mut counts = Vec::new();
    for seed in [0xC0FF_EE00_1234_5678, 11, 22, 33, 44, 55, 66, 77, 88] {
        let mut g = Game::new(seed, "td");
        g.world = WorldState::at_start(&w);
        g.character.apply_preset();
        let mut fights = 0;
        for i in 0..20000 {
            let s = step(&w, &mut g.world, &mut g.rng, D, patrol[i % patrol.len()]);
            let Some(m) = s.encounter else { continue };
            fights += 1;
            let log = gm2d_core::combat::simulate_at(
                g.character.player_stats(),
                &g.character.combat_items(),
                m,
                D,
            );
            if log.outcome == gm2d_core::combat::Outcome::Victory {
                let rating = gm2d_core::rating::creature_rating(m, D);
                g.character.gain_xp(progression::xp_for_rating(rating));
                g.character.resize_boards([0; 5]);
                g.character.apply_preset();
            }
            if g.character.level() >= 5 {
                break;
            }
        }
        counts.push(fights);
    }

    let mean = counts.iter().sum::<i32>() as f64 / counts.len() as f64;
    assert!(
        (25.0..=35.0).contains(&mean),
        "level 5 arrives after {mean:.1} fights on average ({counts:?}), and the plan asks \
         for 25 to 35. XP_DIVISOR is {} — that is the dial, and moving the map's regions \
         moves this too.",
        progression::XP_DIVISOR
    );
    // And no single walk is wildly off, which a mean alone would hide.
    let wild: Vec<i32> = counts.iter().copied().filter(|n| !(15..=50).contains(n)).collect();
    assert!(wild.is_empty(), "some walks are nowhere near the band: {counts:?}");
}

#[test]
#[ignore = "prints what the starting kit actually beats"]
fn show_the_pit() {
    use gm2d_core::combat::{simulate_at, Outcome};
    let mut c = Character::starting();
    c.apply_preset();
    let items = c.combat_items();
    let stats = c.player_stats();
    println!("owns {} pieces, {} Fnorp", c.owned.len(), c.gold);
    for i in &items {
        println!("  item: {} ({:?}) hits {} every {}ms",
                 i.name, i.slot, i.hit_for(stats.strength), i.cooldown_ms);
    }
    for r in &data::world(D).regions {
        for m in &r.enemies {
            let log = simulate_at(stats, &items, m, D);
            if r.id == "the-end-of-all-gears" || log.outcome == Outcome::Victory {
                println!("  {:<24} {:<8} {:?} in {:.1}s", m.name, r.id,
                         log.outcome, log.duration_ms as f32 / 1000.0);
            }
        }
    }
}
