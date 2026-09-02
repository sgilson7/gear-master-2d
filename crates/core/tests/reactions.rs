//! Gear that reacts to other gear: touching neighbours, cross-slot alignment,
//! and the rule that unassembled gear never acts.

mod common;

use common::equip;
use gm2d_core::combat::{simulate, Event, MonsterSpec, Side};
use gm2d_core::loadout::ItemProfile;
use gm2d_core::piece::{Action, SlotKind, Trigger};
use gm2d_core::character::Character;
use gm2d_core::stats::Stats;

const DUMMY: MonsterSpec = MonsterSpec {
    name: "Dummy",
    health: 100_000,
    strength: 0,
    regen: 0,
    mind_resist: 0,    physical_resist: 0,
    magic_resist: 0,
    curse_resist: 0,
    attacks: &[],
    gear: &[],
    gear_offset: 0,
    bounty: 0,
    sprite: gm2d_core::combat::MonsterSprite::Rat,
    rank: gm2d_core::combat::Rank::Ordinary,
    drops: &[],
    items: &[],
};

fn item(name: &str, slot: SlotKind, cooldown_ms: u32, stats: Stats) -> ItemProfile {
    ItemProfile {
        sigil_seed: 0,
        pieces: Vec::new(),
        name: name.to_string(),
        full_name: name.to_string(),
        core: name.to_string(),
        slot,
        cooldown_ms,
        stats,
        triggers: Vec::new(),
        adjacent_assembled_same_slot: 0,
        diagonal_items: Vec::new(),
        open_cells: 0,
        turn_cycle: Vec::new(),
        spins: false,
        fragile: false,
        attracts_curses: false,
        steady: false,
        overtakes: false,
        wrong_sense: false,
        power: 100,
        rating: 0,
        power_bonus: 0,
        casts: Vec::new(),
        adjacent_items: Vec::new(),
        aligned_items: Vec::new(),
    }
}

fn activations(log: &gm2d_core::combat::CombatLog, name: &str) -> Vec<u32> {
    log.entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Activate { side: Side::Player, item, .. } if item == name => Some(e.at_ms),
            _ => None,
        })
        .collect()
}

// -------------------------------------------------- reacting to a neighbour

#[test]
fn a_reactive_item_answers_the_neighbour_it_touches() {
    let driver = item("Driver", SlotKind::Weapon, 1000, Stats::physical(1));
    let mut reactor = item("Reactor", SlotKind::Helmet, 60_000, Stats::ZERO);
    reactor.triggers = vec![Trigger::OnAdjacentActivate(Action::GainMana(3))];
    reactor.adjacent_items = vec![0]; // touching the driver

    let log = simulate(Stats::new(1000, 0, 0, 100), &[driver, reactor], &DUMMY);

    let gains = log
        .entries
        .iter()
        .filter(|e| matches!(e.event, Event::GainMana { side: Side::Player, amount: 3, .. }))
        .count();
    let driver_fired = activations(&log, "Driver").len();
    assert!(driver_fired > 5);
    assert_eq!(gains, driver_fired, "one reaction per neighbour activation");
}

#[test]
fn a_reactive_item_ignores_gear_it_does_not_touch() {
    let stranger = item("Stranger", SlotKind::Weapon, 1000, Stats::physical(1));
    let mut reactor = item("Reactor", SlotKind::Helmet, 60_000, Stats::ZERO);
    reactor.triggers = vec![Trigger::OnAdjacentActivate(Action::GainMana(3))];
    // adjacent_items left empty: it touches nothing.

    let log = simulate(Stats::new(1000, 0, 0, 100), &[stranger, reactor], &DUMMY);

    assert!(
        !log.entries.iter().any(|e| matches!(e.event, Event::GainMana { .. })),
        "nothing is adjacent, so nothing reacts"
    );
}

#[test]
fn reducing_a_cooldown_makes_the_item_fire_sooner() {
    let driver = item("Driver", SlotKind::Weapon, 1000, Stats::physical(1));
    let mut charmed = item("Charmed", SlotKind::Helmet, 4000, Stats::armor(1));
    charmed.triggers = vec![Trigger::OnAdjacentActivate(Action::ReduceCooldown(1000))];
    charmed.adjacent_items = vec![0];

    let with_charm = simulate(Stats::new(1000, 0, 0, 100), &[driver.clone(), charmed], &DUMMY);
    let plain = item("Charmed", SlotKind::Helmet, 4000, Stats::armor(1));
    let without = simulate(Stats::new(1000, 0, 0, 100), &[driver, plain], &DUMMY);

    let fast = activations(&with_charm, "Charmed").len();
    let slow = activations(&without, "Charmed").len();
    assert!(
        fast > slow,
        "the charm should get more activations in ({} vs {})",
        fast,
        slow
    );
}

#[test]
fn two_items_reacting_to_each_other_do_not_loop() {
    // Both react to the other. A reaction must not itself count as an
    // activation, or this would recurse until the stack gives out.
    let mut a = item("A", SlotKind::Weapon, 1000, Stats::physical(1));
    a.triggers = vec![Trigger::OnAdjacentActivate(Action::GainMana(1))];
    a.adjacent_items = vec![1];
    let mut b = item("B", SlotKind::Helmet, 1000, Stats::armor(1));
    b.triggers = vec![Trigger::OnAdjacentActivate(Action::GainMana(1))];
    b.adjacent_items = vec![0];

    let log = simulate(Stats::new(1000, 0, 0, 100), &[a, b], &DUMMY);

    // Terminating at all is most of the point; the counts should also be sane.
    let gains = log
        .entries
        .iter()
        .filter(|e| matches!(e.event, Event::GainMana { side: Side::Player, .. }))
        .count();
    let fired = activations(&log, "A").len() + activations(&log, "B").len();
    assert_eq!(gains, fired, "exactly one reaction per activation, no cascade");
}

// ------------------------------------------------------- cross-slot alignment

#[test]
fn alignment_is_computed_from_the_rows_two_items_occupy() {
    let mut ch = Character::with_all_pieces();
    // A weapon across rows 0-3...
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    // ...and gloves on the same rows in a different grid.
    equip(&mut ch, "Leather Material", SlotKind::Gloves, 0, 0);
    equip(&mut ch, "Channeling Mold", SlotKind::Gloves, 2, 0);

    let items = ch.combat_items();
    assert_eq!(items.len(), 2);
    let gloves = items.iter().find(|i| i.slot == SlotKind::Gloves).unwrap();
    assert_eq!(gloves.aligned_items.len(), 1, "the weapon shares its rows");
    assert!(gloves.adjacent_items.is_empty(), "different grids never touch");
}

#[test]
fn moving_gear_out_of_line_breaks_the_alignment() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0); // rows 0-2
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0); // rows 0-3
    // Gloves pushed down to rows 5-6, clear of the weapon.
    equip(&mut ch, "Leather Material", SlotKind::Gloves, 0, 5);
    equip(&mut ch, "Channeling Mold", SlotKind::Gloves, 2, 5);

    let items = ch.combat_items();
    let gloves = items.iter().find(|i| i.slot == SlotKind::Gloves).unwrap();
    assert!(gloves.aligned_items.is_empty(), "rows 5-6 do not meet rows 0-3");
}


// ------------------------------------------- unassembled gear stays inert


#[test]
fn an_oversized_piece_pays_off_precisely_because_it_cannot_be_built() {
    let mut ch = Character::with_all_pieces();
    // The Vast Tapestry is a 5x4 slab: a base cannot fit beside it.
    equip(&mut ch, "Vast Tapestry", SlotKind::Chest, 0, 0);

    let loose = ch.report(SlotKind::Chest);
    assert_eq!(loose.assembled_count(), 0);
    assert_eq!(loose.stats.health, 580, "30 base + 550 while unbound");
    // Its unbound bonus is deliberately *not* armour: loose gear never
    // activates, and armour only accrues on activation, so armour on a piece
    // that can never be built would be worth nothing at all.
    assert_eq!(loose.stats.armor, 0);

    // Finish the chestpiece around it and the bonus switches off.
    equip(&mut ch, "Hide Base", SlotKind::Chest, 0, 4);
    let built = ch.report(SlotKind::Chest);
    assert_eq!(built.assembled_count(), 1);
    assert_eq!(built.stats.health, 30 + 70, "the unbound bonus is gone");
}

// -------------------------------------------------- the two mana buffs

#[test]
fn mana_empowerment_scales_power_with_the_mana_you_still_hold() {
    use gm2d_core::combat::Combatant;
    let mut c = Combatant::player(Stats::new(100, 10, 0, 100), &[]);
    c.mana = 20;
    assert_eq!(c.effective_power(), 100, "no stacks, no bonus");

    c.empowerment = 1;
    assert_eq!(c.effective_power(), 200, "0.05x per point of 20 mana = +1.00x");
    c.empowerment = 2;
    assert_eq!(c.effective_power(), 300);

    // Spending the mana that powers it cuts the bonus straight back down.
    c.mana = 5;
    assert_eq!(c.effective_power(), 150, "2 stacks against 5 mana");
}


#[test]
fn a_ward_that_cannot_pay_falls_back_instead_of_stacking() {
    let mut ward = item("Ward", SlotKind::Helmet, 600, Stats::ZERO);
    ward.triggers = vec![Trigger::SpendMana {
        cost: 3,
        on_success: Action::GainShield(1),
        on_failure: Action::GainArmor(5),
    }];
    // No mana income at all.
    let log = simulate(Stats::new(2000, 0, 0, 100), &[ward], &DUMMY);
    assert!(
        !log.entries.iter().any(|e| matches!(e.event, Event::Shielded { .. })),
        "nothing to spend, so nothing to stack"
    );
    assert!(log.entries.iter().any(|e| matches!(e.event, Event::GainArmor { .. })));
}

// --------------------------------------------------- Derail: a hand on the wire

/// A player who can stand there while a mechanic is measured.
///
/// `Combatant::player` starts the wall and every pool at zero whatever the
/// stats say, so the only thing `Stats` carries here is a body. Without one
/// the player dies on the first tick and every count is zero, which reads as
/// "the mechanic does nothing" off a fight that never happened.
const ALIVE: Stats = Stats { health: 20_000, ..Stats::ZERO };

/// A foe whose one attack comes round on a known bar.
const TICKER: MonsterSpec = MonsterSpec {
    name: "Ticker",
    health: 100_000,
    strength: 0,
    regen: 0,
    mind_resist: 0,
    physical_resist: 0,
    magic_resist: 0,
    curse_resist: 100,
    attacks: &[gm2d_core::combat::MonsterAttack::hit("swing", 2_000, 1)],
    gear: &[],
    gear_offset: 0,
    bounty: 0,
    sprite: gm2d_core::combat::MonsterSprite::Rat,
    rank: gm2d_core::combat::Rank::Ordinary,
    drops: &[],
    items: &[],
};

fn foe_swings(log: &gm2d_core::combat::CombatLog) -> Vec<u32> {
    log.entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Activate { side: Side::Enemy, .. } => Some(e.at_ms),
            _ => None,
        })
        .collect()
}

/// An item inside the window fires later than it would have, to the tick.
#[test]
fn derail_catches_an_item_inside_the_window() {
    let plain = simulate(ALIVE, &[item("Idle", SlotKind::Weapon, 60_000, Stats::ZERO)], &TICKER);

    // One derail, fired once, from an item whose own bar comes round while the
    // foe's is near the top: at 1,800 ms the foe has 200 ms to go.
    let mut wire = item("Wire", SlotKind::Gloves, 1_800, Stats::ZERO);
    wire.triggers =
        vec![Trigger::OnActivate(Action::Derail { window_ms: 1_000, back_ms: 600 })];
    let derailed = simulate(ALIVE, &[wire], &TICKER);

    let (a, b) = (foe_swings(&plain), foe_swings(&derailed));
    assert_eq!(a[0], 2_000, "the plain bar comes round on its cooldown");
    assert_eq!(b[0], 2_600, "600 ms of bar, taken off at 1,800 ms, to the tick");

    assert!(
        derailed.entries.iter().any(|e| matches!(
            &e.event,
            Event::Derailed { item, by_ms: 600, .. } if item == "swing"
        )),
        "nothing was logged"
    );
}

/// Outside the window, nothing happens and the log says so by staying quiet.
#[test]
fn derail_ignores_an_item_outside_the_window() {
    // Fires at 200 ms, when the foe's 2,000 ms bar has 1,800 ms to go - well
    // outside a 1,000 ms window.
    let mut wire = item("Wire", SlotKind::Gloves, 200, Stats::ZERO);
    wire.triggers =
        vec![Trigger::OnActivate(Action::Derail { window_ms: 1_000, back_ms: 600 })];
    let log = simulate(ALIVE, &[wire], &TICKER);

    let first = log
        .entries
        .iter()
        .find(|e| matches!(&e.event, Event::Derailed { .. }))
        .map(|e| e.at_ms);
    assert!(
        first.is_none_or(|t| t >= 1_000),
        "it caught something at {first:?}, which was not inside the window"
    );
}

/// It is not a curse, so curse resistance is not an answer to it.
///
/// Deliberate, and the point of the effect: it is what a board built entirely
/// out of curse resist has no reply to. `TICKER` sits at 100 curse resist and
/// is derailed anyway.
#[test]
fn derail_is_not_a_curse_and_curse_resist_does_not_answer_it() {
    let mut wire = item("Wire", SlotKind::Gloves, 1_800, Stats::ZERO);
    wire.triggers =
        vec![Trigger::OnActivate(Action::Derail { window_ms: 1_000, back_ms: 600 })];
    let log = simulate(ALIVE, &[wire], &TICKER);
    assert_eq!(TICKER.curse_resist, 100, "the fixture is the point of the test");
    assert!(log.entries.iter().any(|e| matches!(&e.event, Event::Derailed { .. })));
    assert!(
        !log.entries.iter().any(|e| matches!(&e.event, Event::Warded { .. })),
        "a ward would mean the curse machinery had been asked, and it must not be"
    );
}
