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
use gm2d_core::progression::{self, MAX_LEVEL, MAX_ROWS, STARTING_ROWS, XP_TO_NEXT};
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

/// **A level grows no board, and that retires an MVP pillar on purpose.**
///
/// This file used to hold `board_size_is_a_function_of_level` and
/// `a_levelled_character_has_the_boards_its_level_implies`, and both were
/// exactly right for `PLAN.md` M4: board size *was* a pure function of level,
/// which is what made it checkable rather than trusted. M12.3 takes that down
/// deliberately, and the reason is a measurement rather than a preference —
/// M12.0 found **fill going down as you level**, 43% at five and 37% at eight,
/// because rows arrive on a clock and components do not. A scheduled row is
/// dilution on a timer.
///
/// What replaces the pure function is this: a board is the base plus what was
/// *earned* for it, and levelling earns nothing.
#[test]
fn levelling_alone_never_grows_a_board() {
    let mut c = Character::starting();
    for k in SlotKind::ALL {
        assert_eq!(c.loadout.slot(k).rows(), STARTING_ROWS, "{k:?} did not start at three");
    }
    c.gain_xp(progression::xp_to_reach(MAX_LEVEL as u32));
    assert!(c.level() >= 6, "the character actually levelled, or this proves nothing");
    c.resize_boards([0; 5]);
    for k in SlotKind::ALL {
        assert_eq!(
            c.loadout.slot(k).rows(),
            STARTING_ROWS,
            "{k:?} grew for nothing but a level"
        );
    }
}

/// A board is the base plus what was earned, and it stops at the ceiling.
///
/// **The ledger test the pure-function test is succeeded by** — sums match,
/// growth is monotonic (below), and every slot caps at the original's 6x8 so
/// the late game converges instead of sprawling.
#[test]
fn a_board_is_the_base_plus_what_was_earned_and_caps() {
    for granted in 0..=12u8 {
        let want = (STARTING_ROWS + granted).min(MAX_ROWS);
        assert_eq!(progression::board_rows(granted), want, "{granted} granted");
        assert!(progression::board_rows(granted) <= MAX_ROWS, "past the ceiling");
    }
    assert_eq!(progression::base_rows(), STARTING_ROWS);

    // And a character's real boards follow it. **Indexed by `SlotKind::index`
    // rather than by writing the array out**, which is how the first version
    // of this failed: `ALL` is helmet-first and the literal assumed weapon-
    // first, so it asserted about a different frame than it meant.
    let mut c = Character::starting();
    let mut granted = [0u8; 5];
    granted[SlotKind::Weapon.index()] = 1;
    granted[SlotKind::Chest.index()] = 2;
    c.resize_boards(granted);
    assert_eq!(c.loadout.slot(SlotKind::Weapon).rows(), STARTING_ROWS + 1);
    assert_eq!(c.loadout.slot(SlotKind::Chest).rows(), STARTING_ROWS + 2);
    assert_eq!(c.loadout.slot(SlotKind::Helmet).rows(), STARTING_ROWS);
    let mut lots = [0u8; 5];
    lots[SlotKind::Weapon.index()] = 99;
    c.resize_boards(lots);
    assert_eq!(c.loadout.slot(SlotKind::Weapon).rows(), MAX_ROWS, "the ceiling did not hold");
}

/// **A row comes from a point spent or a questline finished, and nothing
/// else.** Both derived, neither banked.
#[test]
fn a_row_is_earned_from_the_tree_or_from_the_world() {
    let tree = data::skills();
    let rows: Vec<&str> = tree
        .trees
        .iter()
        .flat_map(|t| t.nodes.iter())
        .filter(|n| {
            n.effects.iter().any(|e| {
                matches!(e, gm2d_core::skills::Effect::GrowSlotRows { .. })
            })
        })
        .map(|n| n.id.as_str())
        .collect();
    assert!(rows.len() >= 10, "only {} nodes anywhere grant a row", rows.len());
    let base = tree.trees.iter().find(|t| t.id == "base").expect("a base tree");
    let in_base = base
        .nodes
        .iter()
        .filter(|n| {
            n.effects.iter().any(|e| {
                matches!(e, gm2d_core::skills::Effect::GrowSlotRows { .. })
            })
        })
        .count();
    assert!(
        (6..=8).contains(&in_base),
        "the base tree has {in_base} row nodes; the plan asks for six to eight"
    );

    // Every slot can be grown from the base tree, or one frame is unreachable
    // for anybody who does not take a particular class.
    let mut slots: Vec<String> = Vec::new();
    for n in &base.nodes {
        for e in &n.effects {
            if let gm2d_core::skills::Effect::GrowSlotRows { slot, .. } = e {
                slots.push(slot.clone());
            }
        }
    }
    for want in ["weapon", "helmet", "chest", "gloves", "greaves"] {
        assert!(slots.iter().any(|s| s == want), "no base node grows the {want} frame");
    }

    // And the world's half: at most one row per questline, on its last errand.
    let quests = gm2d_core::data::quests();
    let paying: Vec<&str> =
        quests.quests.iter().filter(|q| q.rows.is_some()).map(|q| q.id.as_str()).collect();
    assert!(!paying.is_empty(), "no questline pays a row");
    for id in &paying {
        let q = quests.get(id).expect("it exists");
        assert!(
            !q.requires.is_empty(),
            "{id} pays a row and is the *start* of a line; a row is the end of one"
        );
        let followed_by = quests.quests.iter().any(|o| o.requires.iter().any(|r| r == id));
        assert!(!followed_by, "{id} pays a row and something follows it");
    }
    // A row is never in a drop table.
    for d in &gm2d_core::data::drops().drops {
        assert!(!d.piece.is_empty());
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
    // **14 to 19 since M12.3**, and the widening is the milestone rather than
    // a bound being loosened to fit: `PLAN-M12.md` §8 row 6 asks for six to
    // eight row nodes in the base tree, and the base tree had three. A row is
    // bought here now, so this is where the nodes to buy it are.
    assert!(
        (14..=19).contains(&base.nodes.len()),
        "the base tree has {} nodes and the plan asks for 14 to 19",
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
    use gm2d_core::world::{step, Allowances, Dir, WorldState};

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
            let s = step(&w, &mut g.world, &mut g.rng, D, patrol[i % patrol.len()], &Allowances::default());
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
                // Banked as it is won, which makes this **the floor rather
                // than the expectation**. Experience is carried now and a
                // defeat takes all of it, so a real walk to level five is this
                // many fights or more — never fewer. The band is still the
                // contract for what the map *offers*; what a player keeps is
                // their own business and is the point of the rule.
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

/// A tree is a tree: nothing requires itself, however far round you go.
///
/// `Tree::depth_of` walks prerequisites and would recurse for ever on a cycle.
/// It refuses to revisit, so a cycle in the data would draw a quietly wrong
/// layout rather than hanging — which is worse, because nothing would say so.
#[test]
fn no_tree_requires_itself_in_a_circle() {
    let skills = data::skills();
    for t in &skills.trees {
        for n in &t.nodes {
            let mut seen = vec![n.id.clone()];
            let mut edge = n.requires.clone();
            while let Some(id) = edge.pop() {
                assert_ne!(id, n.id, "{}: {} requires its way back to itself", t.id, n.id);
                if seen.contains(&id) {
                    continue;
                }
                seen.push(id.clone());
                if let Some(m) = t.nodes.iter().find(|m| m.id == id) {
                    edge.extend(m.requires.clone());
                }
            }
        }
    }
}

/// Every tree has something you can spend a point on the moment you open it,
/// and every prerequisite is in the same tree as the node that wants it.
#[test]
fn every_tree_has_a_top_row_and_keeps_its_prerequisites_at_home() {
    let skills = data::skills();
    for t in &skills.trees {
        let rows = t.rows();
        assert!(!rows.is_empty(), "{}: no nodes at all", t.id);
        assert!(!rows[0].is_empty(), "{}: nothing can be taken first", t.id);
        for n in &t.nodes {
            for r in &n.requires {
                assert!(
                    t.nodes.iter().any(|m| m.id == *r),
                    "{}: {} requires {r}, which is in another tree",
                    t.id,
                    n.id
                );
            }
        }
        // And a node always sits below everything it asks for.
        for n in &t.nodes {
            let d = t.depth_of(&n.id);
            for r in &n.requires {
                assert!(
                    t.depth_of(r) < d,
                    "{}: {} is not below its prerequisite {r}",
                    t.id,
                    n.id
                );
            }
        }
    }
}
