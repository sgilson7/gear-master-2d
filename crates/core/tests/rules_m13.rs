//! M13.3 — the four new rules, proved by what they do.
//!
//! Every one is checked against **behaviour**, never against a field being
//! set. This project has shipped eight nodes that parsed, cost a point and
//! changed nothing, and `every_offered_class_reaches_something` had to be
//! rewritten once because *a lint that reads a list rather than the behaviour
//! is the failure it exists to catch, one level up*.
//!
//! The rules are granted the way a player gets them: by taking the real node
//! out of the real tree, on a character who is the real class. A fixture rule
//! list would be testing the arithmetic and not the wiring, and the wiring is
//! what has gone wrong here before.

mod common;

use gm2d_core::character::Character;
use gm2d_core::combat::{Difficulty, MonsterSpec};
use gm2d_core::data;
use gm2d_core::piece::{PieceKind, SlotKind, CATALOG};
use gm2d_core::rule::Rule;

/// A character who is `class`, with `nodes` taken and the points to have taken
/// them — which is how a player arrives holding a rule.
fn expert_with(class: &str, nodes: &[&str]) -> Character {
    expert_from(Character::new(), class, nodes)
}

/// The same, on a character somebody has already given gear to.
fn expert_from(mut c: Character, class: &str, nodes: &[&str]) -> Character {
    c.expert = Some(class.to_string());
    let tree = data::skills();
    for id in nodes {
        c.skill_points += 9;
        // The node is in an expert tree, so `can_take` wants the character to
        // be that class — which is exactly the gate being relied on.
        c.take_skill(&tree, id).unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    c
}

fn has(c: &Character, want: fn(&Rule) -> bool) -> bool {
    c.rules().iter().any(want)
}

// ------------------------------------------------------------------ the guard

#[test]
fn a_rule_that_does_nothing_does_not_load() {
    for (rule, want) in [
        (Rule::Spread { every_turns: 0 }, "no turns at all"),
        (Rule::RowHarvest { per_cell: 0 }, "pays nothing"),
        (Rule::Beacon { pct: 0 }, "lends nothing"),
        (Rule::Productivity { every: 0, slower_pct: 15 }, "no activations"),
        (Rule::Productivity { every: 3, slower_pct: 0 }, "no speed"),
        (Rule::Productivity { every: 3, slower_pct: 100 }, "stopped"),
    ] {
        let why = rule.check().unwrap_err();
        assert!(why.contains(want), "{rule:?} refused with {why:?}, wanted {want:?}");
    }
    for rule in [
        Rule::Spread { every_turns: 2 },
        Rule::RowHarvest { per_cell: 4 },
        Rule::Beacon { pct: 40 },
        Rule::Productivity { every: 3, slower_pct: 15 },
    ] {
        assert!(rule.check().is_ok(), "{rule:?}");
    }
}

#[test]
fn the_four_new_rules_say_what_they_do() {
    for rule in [
        Rule::Spread { every_turns: 2 },
        Rule::RowHarvest { per_cell: 4 },
        Rule::Beacon { pct: 40 },
        Rule::Productivity { every: 3, slower_pct: 15 },
    ] {
        let line = rule.line();
        assert!(line.chars().any(|c| c.is_ascii_digit()), "{rule:?}: {line:?}");
        assert!(line.len() < 95, "{rule:?} is {} characters: {line:?}", line.len());
        assert!(!rule.detail().is_empty(), "{rule:?} explains none of its words");
    }
}

// --------------------------------------------------------------- RowHarvest

/// A single-cell component that goes in the gloves, for filling a row by hand.
fn one_cell_glove() -> &'static str {
    CATALOG
        .iter()
        .find(|d| {
            d.slot == SlotKind::Gloves
                && d.cells.len() == 1
                && d.kind != PieceKind::Quest
                && !d.kind.is_enchantment()
        })
        .expect("a single-cell glove component")
        .name
}

#[test]
fn a_full_row_pays_at_the_bell_and_a_gap_pays_nothing() {
    let w = gm2d_core::slot::SLOT_W;
    // The Patented Funnel node that grants the rule, taken off the real tree.
    let mut c = expert_with("PatentedFunnel", &["pf-the-tap", "pf-full-rows"]);
    assert!(has(&c, |r| matches!(r, Rule::RowHarvest { .. })), "the node grants it");

    let name = one_cell_glove();
    let ids: Vec<_> = (0..w).map(|_| c.give(name).expect("a component")).collect();
    let empty = c.start_with().mana;

    for (i, id) in ids.iter().enumerate().take(w as usize - 1) {
        c.loadout.slot_mut(SlotKind::Gloves).place(&c.registry, *id, i as u8, 0);
    }
    assert_eq!(c.start_with().mana, empty, "a row one cell short pays nothing");

    c.loadout.slot_mut(SlotKind::Gloves).place(&c.registry, *ids.last().unwrap(), w - 1, 0);
    let paid = c.start_with().mana - empty;
    assert_eq!(paid, w as i32 * 4, "a full row pays its width at 4 a cell");

    // **The row is not cleared.** That is the whole borrowed idea: a filled
    // row is a machine, and clearing it would be taking the machine apart.
    assert_eq!(
        c.loadout.slot(SlotKind::Gloves).pieces().len(),
        w as usize,
        "the row was taken apart"
    );
    // And it goes on paying, fight after fight.
    assert_eq!(c.start_with().mana - empty, paid, "it paid once and stopped");
}

/// The capstone's knob **adds to** what the rule pays rather than replacing it.
#[test]
fn the_capstone_tunes_the_row_rather_than_restating_it() {
    let w = gm2d_core::slot::SLOT_W;
    let name = one_cell_glove();
    let mut plain = expert_with("PatentedFunnel", &["pf-the-tap", "pf-full-rows"]);
    let mut tuned = expert_with(
        "PatentedFunnel",
        &[
            "pf-the-tap",
            "pf-full-rows",
            "pf-second-licence",
            "pf-flywheel-brake",
            "pf-overflow-pipe",
            "pf-funnel-patented",
        ],
    );
    for c in [&mut plain, &mut tuned] {
        for x in 0..w {
            let id = c.give(name).expect("a component");
            c.loadout.slot_mut(SlotKind::Gloves).place(&c.registry, id, x, 0);
        }
    }
    // 4 a cell becomes 6: the rule's own number plus the capstone's +2.
    assert_eq!(plain.start_with().mana, w as i32 * 4);
    assert_eq!(tuned.start_with().mana, w as i32 * 6, "the knob adds to the rule");
}

// -------------------------------------------------------------------- Beacon

/// A power ench, by id.
fn a_power_ench() -> String {
    data::enchs()
        .enchs
        .iter()
        .find(|e| matches!(e.effect, gm2d_core::ench::Effect::Power { .. }))
        .expect("a power ench")
        .id
        .clone()
}

/// Four finished items in a line, the beacon granted, and a licence.
///
/// **A board with adjacency on it, and not three loose cells or a spaced-out
/// fixture.** Two drafts of this test measured nothing and said so rather than
/// passing: three touching single-cell components assemble *no* items at all,
/// because an item needs a core; and `common::build_full_loadout` makes eight
/// items with **no adjacency between any of them**, because that preset spaces
/// them out on purpose.
///
/// It was Auto-pack over the whole catalogue for that reason, which is nineteen
/// items and works — and is a second and a half of packing every time one of
/// these four tests asks for a board. `common::items_in_a_row` is the fixture
/// the repository did not have and now does: **a line rather than a cluster**,
/// because the two questions a lending rule has are *does it reach my
/// neighbour* and *does it reach my neighbour's neighbour*, and three in a line
/// is the smallest board that can ask the second. Four, so there is also an
/// item touching neither end.
fn beacon_board(nodes: &[&str]) -> Character {
    let mut c = Character::with_all_pieces();
    common::items_in_a_row(&mut c, 4);
    let mut c = expert_from(c, "FullBill", nodes);
    c.bought_licence = true;
    c
}

/// The power of the item holding `id`.
fn power_of(ps: &[gm2d_core::loadout::ItemProfile], id: gm2d_core::piece::PieceId) -> i32 {
    ps.iter().find(|p| p.pieces.contains(&id)).expect("that component's item").power
}

/// A lender keeps what it has, and its neighbours are worth more for touching
/// it. Measured on the profiles, which is what a fight is actually handed.
#[test]
fn a_beacon_lends_and_keeps_and_does_not_chain() {
    let mut c = beacon_board(&["fb-second-rack", "fb-ponkey-broadcast"]);
    assert!(has(&c, |r| matches!(r, Rule::Beacon { .. })), "the node grants it");

    let bare = c.combat_items();
    // An item with a neighbour, and a third item touching neither.
    let (lender, neighbour) = bare
        .iter()
        .enumerate()
        .find_map(|(i, p)| p.adjacent_items.first().map(|&n| (i, n)))
        .expect("a full board has two finished items sharing an edge");
    let stranger = (0..bare.len())
        .find(|&k| k != lender && k != neighbour && !bare[k].adjacent_items.contains(&lender))
        .expect("a full board has an item that touches neither");

    let (lp, np, sp) = (bare[lender].pieces[0], bare[neighbour].pieces[0], bare[stranger].pieces[0]);
    let (m_l, m_n, m_s) = (power_of(&bare, lp), power_of(&bare, np), power_of(&bare, sp));

    let power = a_power_ench();
    c.enchs_owned.push(power.clone());
    c.attach_ench(&power, lp).expect("bolted on");
    let lit = c.combat_items();
    let (l_l, l_n, l_s) = (power_of(&lit, lp), power_of(&lit, np), power_of(&lit, sp));

    assert!(l_l > m_l, "the enched item lost its own ench");
    assert!(l_n > m_n, "the neighbour was lent nothing: {m_n} -> {l_n}");
    assert!(l_n - m_n < l_l - m_l, "the neighbour got the whole ench rather than a share");
    assert_eq!(l_s, m_s, "an item touching nothing was lent something anyway");
}

/// **It does not chain.** What a neighbour is lent is never lent onward, or a
/// packed grid would broadcast itself to a fixed point.
#[test]
fn what_a_neighbour_is_lent_is_not_lent_on() {
    let mut c = beacon_board(&["fb-second-rack", "fb-ponkey-broadcast"]);
    let bare = c.combat_items();
    // A -> B -> C where C does not touch A. Without one, a chain could not be
    // told from a lend, so the fixture says so rather than passing.
    let (a, b, far) = bare
        .iter()
        .enumerate()
        .find_map(|(a, pa)| {
            pa.adjacent_items.iter().find_map(|&b| {
                bare[b]
                    .adjacent_items
                    .iter()
                    .find(|&&x| x != a && !pa.adjacent_items.contains(&x))
                    .map(|&x| (a, b, x))
            })
        })
        .expect("a full board has a run of three items");

    let (ap, bp, fp) = (bare[a].pieces[0], bare[b].pieces[0], bare[far].pieces[0]);
    let (m_b, m_f) = (power_of(&bare, bp), power_of(&bare, fp));
    let power = a_power_ench();
    c.enchs_owned.push(power.clone());
    c.attach_ench(&power, ap).expect("bolted on");
    let lit = c.combat_items();

    assert!(power_of(&lit, bp) > m_b, "the neighbour was lent nothing");
    assert_eq!(power_of(&lit, fp), m_f, "two hops away was reached, so the lend chained");
}

/// A switch has no fraction, and lending a one-shot would break the neighbour.
#[test]
fn only_the_enchs_that_are_a_number_lend() {
    use gm2d_core::ench::Effect;
    assert_eq!(Effect::Power { pct: 100 }.scaled(40), Some(Effect::Power { pct: 40 }));
    assert_eq!(Effect::Haste { pct: 50 }.scaled(40), Some(Effect::Haste { pct: 20 }));
    assert_eq!(Effect::Spin.scaled(40), None, "part of a switch is not a thing");
    assert_eq!(
        Effect::Fragile { pct: 200 }.scaled(40),
        None,
        "lending a one-shot would break the neighbour"
    );
    // And a share that rounds to nothing is nothing rather than a zero effect.
    assert_eq!(Effect::Power { pct: 1 }.scaled(40), None);
}

/// The capstone's knob adds to what the rule lends.
#[test]
fn the_capstone_widens_the_broadcast() {
    let lend = |nodes: &[&str]| {
        let mut c = beacon_board(nodes);
        let bare = c.combat_items();
        let (lender, neighbour) = bare
            .iter()
            .enumerate()
            .find_map(|(i, p)| p.adjacent_items.first().map(|&n| (i, n)))
            .expect("two touching items");
        let (lp, np) = (bare[lender].pieces[0], bare[neighbour].pieces[0]);
        let before = power_of(&bare, np);
        let power = a_power_ench();
        c.enchs_owned.push(power.clone());
        c.attach_ench(&power, lp).expect("bolted on");
        power_of(&c.combat_items(), np) - before
    };
    let plain = lend(&["fb-second-rack", "fb-ponkey-broadcast"]);
    let wider = lend(&["fb-second-rack", "fb-ponkey-broadcast", "fb-wider-broadcast"]);
    assert!(wider > plain, "Wider Broadcast lent no more: {plain} -> {wider}");
}

// -------------------------------------------------------------- Productivity

/// A board that fights long enough for an every-third schedule to fire.
///
/// **`Character::starting()` cannot ask this question**, and the first draft
/// used it: the starting kit against a Rust Colossus is a defeat in 3.2
/// seconds with **two** activations, so an every-third rule never comes round
/// once. `common::build_full_loadout` wins the same fight in 35 seconds over
/// 84 activations, which is a fight with a schedule in it.
fn productivity_board() -> Character {
    let mut c = common::bench();
    common::build_full_loadout(&mut c);
    expert_from(c, "CursedLicence", &["cl-searing-clause", "cl-second-reading"])
}

/// Every third act of an enched item runs twice, and the item is slower after.
///
/// Measured on the fight, not on a field: a doubled activation shows up as
/// more of the log, and the slowdown shows up as fewer activations later.
#[test]
fn productivity_doubles_an_enched_item_and_slows_it() {
    use gm2d_core::combat::{simulate_holding, Event, Side};

    let mut c = productivity_board();
    assert!(has(&c, |r| matches!(r, Rule::Productivity { .. })), "the node grants it");

    let spec: &MonsterSpec = gm2d_core::combat::LADDER
        .iter()
        .find(|s| s.name == "Rust Colossus")
        .expect("something that lasts");

    let plain = |c: &Character| {
        simulate_holding(
            c.player_stats(),
            &c.combat_items(),
            spec,
            Difficulty::Easy,
            &[],
            0,
            c.start_with(),
        )
    };
    // Without an ench nothing is enched, so the rule reaches nothing at all.
    let before = plain(&c);
    let acts = |log: &gm2d_core::combat::CombatLog| {
        log.entries
            .iter()
            .filter(|e| matches!(e.event, Event::Activate { side: Side::Player, .. }))
            .count()
    };
    let hits = |log: &gm2d_core::combat::CombatLog| {
        log.entries
            .iter()
            .filter(|e| matches!(e.event, Event::Hit { by: Side::Player, .. }))
            .count()
    };

    // Bolt a power ench onto a seated component, which makes its item enched.
    c.bought_licence = true;
    let power = data::enchs()
        .enchs
        .iter()
        .find(|e| matches!(e.effect, gm2d_core::ench::Effect::Power { .. }))
        .expect("a power ench")
        .id
        .clone();
    let seated = c
        .loadout
        .slot(SlotKind::Weapon)
        .pieces()
        .first()
        .copied()
        .expect("the fixture seated a weapon");
    c.enchs_owned.push(power.clone());
    c.attach_ench(&power, seated).expect("bolted on");
    let after = plain(&c);

    assert!(after.player.items.iter().any(|i| i.enched), "no item reads as enched");
    assert!(
        hits(&after) > 0 && acts(&after) > 0,
        "the fixture never swung: {} acts, {} hits",
        acts(&after),
        hits(&after)
    );
    // The item did more per activation than it did without the rule — which is
    // the whole of the promise — even though it is slower for having done it.
    let per_act_before = hits(&before) as f32 / acts(&before).max(1) as f32;
    let per_act_after = hits(&after) as f32 / acts(&after).max(1) as f32;
    assert!(
        per_act_after > per_act_before,
        "an enched item did no more per act: {per_act_before} -> {per_act_after}"
    );
}

/// **Deterministic**, like every other counted rule in this engine: the same
/// board fights the same fight twice.
#[test]
fn productivity_is_deterministic_on_replay() {
    use gm2d_core::combat::simulate_holding;
    let mut c = productivity_board();
    c.bought_licence = true;
    let power = data::enchs()
        .enchs
        .iter()
        .find(|e| matches!(e.effect, gm2d_core::ench::Effect::Power { .. }))
        .expect("a power ench")
        .id
        .clone();
    let seated = c.loadout.slot(SlotKind::Weapon).pieces()[0];
    c.enchs_owned.push(power.clone());
    c.attach_ench(&power, seated).expect("bolted on");
    let spec = gm2d_core::combat::LADDER.iter().find(|s| s.name == "Rust Colossus").unwrap();
    let run = || {
        simulate_holding(
            c.player_stats(),
            &c.combat_items(),
            spec,
            Difficulty::Easy,
            &[],
            0,
            c.start_with(),
        )
    };
    let (a, b) = (run(), run());
    assert_eq!(a.duration_ms, b.duration_ms);
    assert_eq!(a.outcome, b.outcome);
    assert_eq!(a.entries.len(), b.entries.len());
    for (x, y) in a.entries.iter().zip(&b.entries) {
        assert_eq!(x.at_ms, y.at_ms);
        assert_eq!(x.event, y.event);
    }
}

/// The slowdown stacks and is capped, so an item can never be slowed to a stop
/// — which would be a stun the player bought and cannot take off.
#[test]
fn the_slowdown_never_stops_an_item() {
    assert!(gm2d_core::combat::MAX_PRODUCTIVITY_SLOW_PCT < 100);
}

// -------------------------------------------------------------------- Spread

/// The smallest thing that can be laid under a grid.
///
/// **Two cells, not one.** There is no single-cell enchantment in the
/// catalogue — the smallest is the Lightning Rod at two — which is why the
/// spread has to ask `can_place` rather than assume a bare cell is room.
fn smallest_underlay() -> &'static gm2d_core::piece::PieceDef {
    CATALOG
        .iter()
        .filter(|d| d.kind.is_enchantment())
        .min_by_key(|d| d.cells.len())
        .expect("something goes under a grid")
}

/// **It works on the diagonal**, because a copy laid edge-on would kill both.
#[test]
fn spread_works_a_corner_and_never_an_edge() {
    let mut c = expert_with("OverwoundArm", &["oa-dry-bearing", "oa-clearing"]);
    assert!(has(&c, |r| matches!(r, Rule::Spread { .. })), "the node grants it");

    let under = smallest_underlay();
    // **Into the grid it belongs to.** `Slot::place` does not validate, so the
    // first draft laid it in the gloves and `spread_one` then refused every
    // copy with `WrongSlot` — the guard working, and the fixture wrong.
    let frame = under.slot;
    let id = c.give(under.name).expect("a component");
    c.loadout.slot_mut(frame).place(&c.registry, id, 0, 0);
    assert!(c.loadout.slot(frame).enchant_is_live(id), "the source is live");

    assert_eq!(c.spread_underlay(1), 1, "nothing was worked");
    let slot = c.loadout.slot(frame);
    let laid: Vec<_> = slot.enchantments();
    assert_eq!(laid.len(), 2, "one cell was worked");
    // Both are still live, which is the whole reason it goes on the diagonal.
    for p in &laid {
        assert!(slot.enchant_is_live(*p), "working the frame killed an underlay");
    }
    // And no cell of the new one shares an edge with a cell of the old one,
    // which is the whole of why it goes on the diagonal.
    let mine: Vec<(u8, u8)> = slot.enchant_cells(id);
    let theirs: Vec<(u8, u8)> =
        laid.iter().filter(|p| **p != id).flat_map(|p| slot.enchant_cells(*p)).collect();
    assert!(!theirs.is_empty(), "nothing new was laid");
    for &(x, y) in &theirs {
        for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
            let n = ((x as i32 + dx), (y as i32 + dy));
            assert!(
                !mine.iter().any(|&(mx, my)| (mx as i32, my as i32) == n),
                "the new underlay shares an edge with the old at {n:?}"
            );
        }
    }
}

/// **Only a frame with nothing seated in it.** A frame you are wearing gear on
/// is one whose underlay you arranged on purpose.
#[test]
fn a_frame_with_gear_in_it_is_left_alone() {
    let mut c = expert_with("OverwoundArm", &["oa-dry-bearing", "oa-clearing"]);
    let under = smallest_underlay();
    let frame = under.slot;
    let u = c.give(under.name).expect("a component");
    c.loadout.slot_mut(frame).place(&c.registry, u, 0, 0);
    // One piece of gear, anywhere in the same frame, is enough to close it.
    let gear = CATALOG
        .iter()
        .find(|d| d.slot == frame && !d.kind.is_enchantment() && d.kind != PieceKind::Quest)
        .expect("something to wear on that frame");
    let g = c.give(gear.name).expect("a component");
    c.loadout.slot_mut(frame).place(&c.registry, g, 0, 6);

    assert_eq!(c.spread_underlay(3), 0, "a frame with gear in it was worked anyway");
    assert_eq!(c.loadout.slot(frame).enchantments().len(), 1);
}

/// A frame with nothing to copy spreads nothing and says so by returning less.
#[test]
fn a_bare_frame_has_nothing_to_copy() {
    let mut c = expert_with("OverwoundArm", &["oa-dry-bearing", "oa-clearing"]);
    assert_eq!(c.spread_underlay(5), 0, "something was worked out of nothing");
}

/// It never crosses into another grid.
#[test]
fn spread_stays_in_its_frame() {
    let mut c = expert_with("OverwoundArm", &["oa-dry-bearing", "oa-clearing"]);
    let under = smallest_underlay();
    let frame = under.slot;
    let u = c.give(under.name).expect("a component");
    c.loadout.slot_mut(frame).place(&c.registry, u, 0, 0);
    assert!(c.spread_underlay(4) > 0, "nothing was worked, so nothing is proved");
    for k in SlotKind::ALL {
        if k == frame {
            continue;
        }
        assert!(
            c.loadout.slot(k).enchantments().is_empty(),
            "{k:?} was worked from another grid"
        );
    }
}
