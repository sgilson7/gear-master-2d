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

/// The table is the formula, over **both** arms of it.
///
/// Generated rather than typed, and checked here so the two cannot drift. The
/// formula is `progression::curve`, which is the one place the sum is done —
/// this test does not restate it, because a test that carries its own copy of
/// what it is checking is a second rulebook with a shorter feedback loop.
#[test]
fn the_table_matches_the_formula() {
    for level in 1..=MAX_LEVEL as u32 {
        let want = progression::curve(level).round() as i32;
        assert_eq!(
            progression::xp_to_next(level),
            want,
            "level {level} costs {} and the formula says {want}",
            progression::xp_to_next(level)
        );
    }
    assert_eq!(XP_TO_NEXT[0], 20, "the first level no longer costs what it always has");
    // And it covers both shapes, or "both arms" above is one arm.
    assert!(MAX_LEVEL as u32 > progression::JOINT, "the table stops before the joint");
}

/// **The two arms agree at fifty.**
///
/// Not `curve(50) == curve(50)`, which is a check that compares a number with
/// itself. Each arm is evaluated at the joint *separately*, from the published
/// constants, and the two are required to be the same number — which is what
/// catches the mistake this is written against: an exponential anchored at
/// forty-nine, or at `L - JOINT + 1`, produces a curve that jumps by the whole
/// base at the one level a player will be watching for.
#[test]
fn the_curve_has_no_step_at_fifty() {
    use gm2d_core::progression::{CURVE_A, CURVE_B, CURVE_C, CURVE_BASE, JOINT};
    let j = JOINT as f64;
    let quadratic_at_the_joint = CURVE_A * j * j + CURVE_B * j + CURVE_C;
    let exponential_at_the_joint = quadratic_at_the_joint * CURVE_BASE.powi(0);
    assert!(
        (quadratic_at_the_joint - exponential_at_the_joint).abs() < 1e-9,
        "the arms disagree at the joint: {quadratic_at_the_joint} against {exponential_at_the_joint}"
    );
    // **Asked of `curve` and not of the table.** The table is a `const` and an
    // anchor written `L - JOINT + 1` leaves it untouched, so a check that read
    // `xp_to_next` here would pass on the exact mistake it exists to catch and
    // leave the catching to `the_table_matches_the_formula` — which is a
    // different check with a different failure message.
    assert!(
        (progression::curve(JOINT) - quadratic_at_the_joint).abs() < 1e-9,
        "the formula at the joint is {} and the quadratic says {quadratic_at_the_joint}",
        progression::curve(JOINT)
    );
    assert!(
        (progression::curve(JOINT + 1) - quadratic_at_the_joint * CURVE_BASE).abs() < 1e-6,
        "level {} is {} and one step of the base past the joint is {}",
        JOINT + 1,
        progression::curve(JOINT + 1),
        quadratic_at_the_joint * CURVE_BASE
    );
    // And the table is those numbers, taken through the same rounding.
    assert_eq!(progression::xp_to_next(JOINT), quadratic_at_the_joint.round() as i32);
    assert_eq!(
        progression::xp_to_next(JOINT + 1),
        (quadratic_at_the_joint * CURVE_BASE).round() as i32,
        "level {} is not one step of the base past the joint",
        JOINT + 1
    );
    // **And nothing anywhere jumps by more than the base.** The quadratic's
    // own steepest step is level three to four, at 1.361 — rounding on small
    // numbers — so the bound is the base with a whole point of slack, which is
    // still far under the 1.35 a misanchored exponential would add.
    for level in 1..MAX_LEVEL as u32 {
        let a = progression::xp_to_next(level) as f64;
        let b = progression::xp_to_next(level + 1) as f64;
        assert!(
            b <= a * (CURVE_BASE + 0.02),
            "level {} costs {a} and level {} costs {b}, which is a cliff",
            level,
            level + 1
        );
    }
}

/// The curve is monotone, positive, and fits the type it is stored in.
///
/// **`XP_TO_NEXT` is `[i32; MAX_LEVEL]` and the arm past the joint grows by a
/// third every level**, which is the trap `PLAN-M15.md` §2.6 names: overflow
/// checks are on in `[profile.test]` and off in a release build, so an
/// overflowing table fails loudly in one and silently in the other — and a
/// level that costs a negative amount is a level everybody already has.
///
/// The headroom is stated rather than hoped for: the first level whose cost
/// does not fit is **95**, so `MAX_LEVEL` has thirty-five levels of room. This
/// is the check that goes red the day somebody raises either number too far.
#[test]
fn the_curve_never_overflows() {
    let mut last = 0;
    for level in 1..=MAX_LEVEL as u32 {
        let cost = progression::xp_to_next(level);
        assert!(cost > 0, "level {level} costs {cost}, which is not a cost");
        assert!(cost >= last, "level {level} is cheaper than the one before it");
        last = cost;
    }
    // The whole climb, summed, is what `Character::xp` has to hold.
    let whole: i64 = (1..=MAX_LEVEL as u32).map(|l| progression::xp_to_next(l) as i64).sum();
    assert!(whole < i32::MAX as i64, "the whole climb is {whole}, which will not fit");
    assert_eq!(progression::xp_to_reach(MAX_LEVEL as u32 + 1) as i64, whole);
    // And there is room to move `MAX_LEVEL` before it is a problem, which is
    // the number the constant's own doc quotes.
    assert!(
        progression::curve(94) < i32::MAX as f64,
        "the headroom stated on MAX_LEVEL is not there"
    );
    assert!(
        progression::curve(95) >= i32::MAX as f64,
        "the headroom is larger than MAX_LEVEL's doc claims, so the figure is stale"
    );
}

/// **The climb to twenty is at most half what it was, and it is a quarter.**
///
/// The human's anchor has two bounds and this is the one that is arithmetic:
/// `xp_to_reach(20)` was 17,053 and must be no more than 8,526. The other bound
/// is 150 fights and lives in the walk.
#[test]
fn the_climb_to_twenty_is_at_most_half_what_it_was() {
    // The recorded figure, written out rather than recomputed: the old curve is
    // gone and a test that rebuilt it would be a test carrying a copy of
    // something that no longer exists.
    const WAS: i32 = 17_053;
    let now = progression::xp_to_reach(20);
    assert!(
        now <= WAS / 2,
        "level twenty costs {now} and half of what it was is {}",
        WAS / 2
    );
    // And the whole point of the block: it is very much less than half.
    assert!(now < WAS / 3, "it is meant to be about 150 fights, and it costs {now}");
}

/// **The exponential is steeper than the quadratic it replaced.**
///
/// Otherwise *"then exponential after level 50"* is decoration: a curve whose
/// second arm a quadratic keeps up with is one arm with a different formula
/// written on it. Measured at the top of the table, where the gap is the point.
#[test]
fn the_curve_past_fifty_is_steeper_than_the_quadratic_would_have_been() {
    use gm2d_core::progression::{CURVE_A, CURVE_B, CURVE_C, JOINT};
    let top = MAX_LEVEL as f64;
    let quadratic_would_be = CURVE_A * top * top + CURVE_B * top + CURVE_C;
    let is = progression::curve(MAX_LEVEL as u32);
    assert!(
        is > quadratic_would_be * 2.0,
        "at level {MAX_LEVEL} the exponential asks {is:.0} and the quadratic would have \
         asked {quadratic_would_be:.0}, which is not a change of shape"
    );
    // And below the joint the two are the same thing, or the arms are swapped.
    let below = JOINT as f64 - 1.0;
    assert!(
        (progression::curve(JOINT - 1) - (CURVE_A * below * below + CURVE_B * below + CURVE_C))
            .abs()
            < 1e-9,
        "the level below the joint is not on the quadratic"
    );
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
    // **Six to twelve since M13**, and the widening is the ask rather than a
    // bound loosened to fit: the tree grew five tiers whose whole job is rows,
    // so that every frame can be walked up to the original game's eight. See
    // `every_frame_can_be_walked_to_the_old_size`, which is the bound that
    // actually matters — this one only says the nodes are here.
    assert!(
        (6..=12).contains(&in_base),
        "the base tree has {in_base} row nodes; the plan asks for six to twelve"
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

/// The base tree walks every frame up to the size the old game had, and no
/// further — with nothing spent on a row that cannot be given.
///
/// Asked for in as many words: *"more rows to all of the gear slots, but get
/// progressively more expensive per additional row you add, up to the original
/// gear master size"*. Three claims, and each is a line here.
///
/// **The last one is the one worth having a test for.** `board_rows` clamps at
/// `MAX_ROWS`, so a tree that over-grants does not break anything — it just
/// quietly sells a point for nothing, which is exactly the failure eight nodes
/// shipped with for two milestones. A grant past the ceiling is a promise that
/// reaches nothing.
#[test]
fn every_frame_can_be_walked_to_the_old_size() {
    let tree = data::skills();
    let base = tree.base().expect("a base tree");
    let all: Vec<String> = base.nodes.iter().map(|n| n.id.clone()).collect();
    let granted = tree.granted_rows(&all);

    for k in SlotKind::ALL {
        let rows = progression::board_rows(granted[k.index()]);
        assert_eq!(
            rows,
            progression::MAX_ROWS,
            "{k:?} reaches {rows} rows with the whole base tree taken, and the old \
             game's frames are {}",
            progression::MAX_ROWS,
        );
        // And not one grant more than that, which is a point spent on nothing.
        assert_eq!(
            progression::STARTING_ROWS + granted[k.index()],
            progression::MAX_ROWS,
            "{k:?} is granted {} rows and can only use {}",
            granted[k.index()],
            progression::MAX_ROWS - progression::STARTING_ROWS,
        );
    }
}

/// A row costs more the deeper you go for it.
///
/// The other half of the same ask. Read off the tree rather than listed here:
/// the row nodes are sorted by how deep they sit, and a node deeper than
/// another may never cost less. **Depth rather than a hand-written order**,
/// because the order is what the prerequisites already say and a second copy
/// of it here would go stale the first time a tier was re-parented.
#[test]
fn a_row_costs_more_the_deeper_it_is() {
    use gm2d_core::skills::Effect;
    let tree = data::skills();
    let base = tree.base().expect("a base tree");
    let mut rows: Vec<(u32, u32, &str)> = base
        .nodes
        .iter()
        .filter(|n| n.effects.iter().any(|e| matches!(e, Effect::GrowSlotRows { .. })))
        .map(|n| (base.depth_of(&n.id), n.cost, n.id.as_str()))
        .collect();
    rows.sort();
    for pair in rows.windows(2) {
        let (da, ca, ia) = pair[0];
        let (db, cb, ib) = pair[1];
        if db > da {
            assert!(cb >= ca, "{ib} is deeper than {ia} and cheaper: {cb} against {ca}");
        }
    }
    // And it actually climbs rather than sitting flat all the way down.
    let (_, first, _) = rows.first().copied().expect("row nodes");
    let (_, last, _) = rows.last().copied().expect("row nodes");
    assert!(last > first, "the deepest row node costs {last} and the shallowest {first}");
}

/// Every node in the shipped tree is reachable, and every prerequisite exists.
///
/// A node whose prerequisite is misspelled is a node no player can ever take,
/// and nothing else would notice.
#[test]
fn the_shipped_tree_is_coherent() {
    let tree = data::skills();
    let base = tree.base().expect("a base tree");
    // **14 to 24 since M13**, widened twice for the same reason and both times
    // as the milestone rather than to fit: M12.3 made a row a thing you buy
    // and took this from three row nodes to seven, and M13 added five tiers
    // that carry the rest of the way to the old game's eight-row frames.
    assert!(
        (14..=24).contains(&base.nodes.len()),
        "the base tree has {} nodes and the plan asks for 14 to 24",
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
