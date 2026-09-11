//! What an item hits for, and how a screen is told.
//!
//! **A swing is not a constant.** Held fury is added to every one of them and
//! a spin lifts the item's own power, so what an item hits for at the tenth
//! second is not what it hit for at the first. The fight has always worked
//! that way; what was missing was any way for a screen to say so, because the
//! log did not record *which* item threw a swing and the replay's row printed
//! the estimate it opened with for the whole fight.
//!
//! These are the two halves of that: the log names the item, and the number it
//! names climbs.

mod common;

use common::seat;
use gm2d_core::character::Character;
use gm2d_core::combat::{simulate, Combatant, Event, MonsterAttack, MonsterSpec, Side};
use gm2d_core::piece::{Resource, SlotKind};

/// Something to hit that hits back gently, so the fight lasts long enough for
/// a pool to bank.
const NIBBLER: MonsterSpec = MonsterSpec {
    name: "Nibbler",
    health: 900_000,
    strength: 0,
    regen: 0,
    mind_resist: 0,
    physical_resist: 0,
    magic_resist: 0,
    curse_resist: 0,
    attacks: &[MonsterAttack::hit("nibble", 2000, 1)],
    gear: &[],
    gear_offset: 0,
    bounty: 0,
    sprite: gm2d_core::combat::MonsterSprite::Rat,
    rank: gm2d_core::combat::Rank::Ordinary,
    drops: &[],
    items: &[],
    enchs: &[],
};

/// A blade, and a helmet that banks fury every time it comes round.
///
/// The plating **spends** the pool as well as banking it — six for thirty
/// armour — which is why the blade's swing climbs and then drops back. That is
/// the mechanic working, and it is the reason the assertion below is "it
/// moved" rather than "it only ever rose".
fn a_board_that_banks_fury() -> Character {
    let mut ch = Character::with_all_pieces();
    ch.grow_slot(SlotKind::Weapon, 5);
    ch.grow_slot(SlotKind::Helmet, 5);
    seat(
        &mut ch,
        &[
            ("Oak Handle", SlotKind::Weapon, 0, 0, 0),
            ("Iron Blade", SlotKind::Weapon, 1, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 0, 0, 0),
            ("Scarred Plating", SlotKind::Helmet, 2, 0, 0),
        ],
    );
    ch
}

/// A weapon that swings on its own cooldown, and a glove that hits back when
/// the glove beside it goes off.
///
/// Two damage sources by two different roads: an item's own activation, and a
/// reaction resolved through `apply`. Both have to name the item that owns
/// them or a row cannot show either.
fn a_board_with_a_reaction() -> Character {
    let mut ch = Character::with_all_pieces();
    ch.grow_slot(SlotKind::Weapon, 5);
    ch.grow_slot(SlotKind::Gloves, 5);
    seat(
        &mut ch,
        &[
            ("Oak Handle", SlotKind::Weapon, 0, 0, 0),
            ("Iron Blade", SlotKind::Weapon, 1, 0, 0),
            // The reactor: five physical whenever the item under it fires.
            ("Leather Material", SlotKind::Gloves, 0, 0, 0),
            ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
            // And the neighbour it is answering.
            ("Ratskin Material", SlotKind::Gloves, 0, 2, 0),
            ("Padded Mold", SlotKind::Gloves, 2, 2, 0),
        ],
    );
    ch
}

#[test]
fn a_hit_names_the_item_that_threw_it() {
    let ch = a_board_with_a_reaction();
    let items = ch.combat_items();
    let log = simulate(ch.player_stats(), &items, &NIBBLER);

    // Every swing the player threw is attributed, and to an item that exists.
    let mut seen: Vec<usize> = Vec::new();
    for e in &log.entries {
        if let Event::Hit { by: Side::Player, by_item, .. } = &e.event {
            let idx = by_item.expect("a swing off a board belongs to one of its items");
            assert!(idx < items.len(), "item {idx} of {}", items.len());
            if !seen.contains(&idx) {
                seen.push(idx);
            }
        }
    }
    assert!(!seen.is_empty(), "the board never swung");

    // And the attribution tells items apart rather than naming one for
    // everything: this board hits by two roads — a weapon on its own cooldown
    // and a glove answering its neighbour — and they are different indices.
    assert!(
        seen.len() >= 2,
        "every swing was credited to the same item, so nothing is being told apart: {seen:?}"
    );
    let by_slot: Vec<_> = seen.iter().map(|&i| items[i].slot).collect();
    assert!(
        by_slot.contains(&SlotKind::Weapon) && by_slot.contains(&SlotKind::Gloves),
        "a reaction resolved through `apply` lost its owner: {by_slot:?}"
    );

    // The index is the one `Activate` reports, which is what lets a row put
    // the number beside itself.
    for idx in &seen {
        assert!(
            log.entries.iter().any(|e| matches!(
                &e.event,
                Event::Activate { side: Side::Player, index, .. } if index == idx
            )),
            "item {idx} hit without ever activating"
        );
    }
}

#[test]
fn what_an_item_hits_for_climbs_as_fury_banks() {
    let ch = a_board_that_banks_fury();
    let items = ch.combat_items();
    let weapon = items
        .iter()
        .position(|p| p.slot == SlotKind::Weapon)
        .expect("the board has a weapon");
    let opened = items[weapon].hit_for(ch.player_stats().strength);
    let log = simulate(ch.player_stats(), &items, &NIBBLER);

    let swings: Vec<i32> = log
        .entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Hit { by: Side::Player, by_item: Some(i), damage, .. } if *i == weapon => {
                Some(*damage)
            }
            _ => None,
        })
        .collect();

    assert!(swings.len() > 4, "not enough swings to see a trend: {swings:?}");
    // **The opening swing is what the card estimated**, which is what makes the
    // row's first number honest — and every one after it is the fight's.
    assert_eq!(swings[0], opened, "the first swing is not what the card estimated");
    let top = *swings.iter().max().unwrap();
    assert!(
        top > opened,
        "fury banked all fight and the blade never once hit harder than its estimate: {swings:?}"
    );
    // It moves rather than stepping once and staying: the plating spends the
    // pool it banks, so the number goes back down as well as up.
    assert!(
        swings.iter().any(|&n| n < top && n > 0),
        "the swing rose once and never moved again: {swings:?}"
    );
}

#[test]
fn the_panel_shows_the_pools_a_board_can_actually_bank() {
    let worth = Combatant::pools_worth_holding();

    // The three the catalogue grants and that pay for being held.
    for want in [Resource::Rage, Resource::Faith, Resource::Nature] {
        assert!(worth.contains(&want), "{} is missing from {worth:?}", want.name());
    }
    // **Mana and insight are not on it.** Both are granted all over the
    // catalogue and neither pays a wearer for sitting on a pile of it: mana
    // empowers and shields when it is spent, insight feeds the mind lane. A
    // panel listing them would be promising something `held_bonus` does not do.
    for not in [Resource::Mana, Resource::Insight] {
        assert!(!worth.contains(&not), "{} pays nothing held, and is listed", not.name());
    }
    // And nothing is listed that pays nothing, whatever the catalogue does.
    for r in &worth {
        assert_ne!(
            Combatant::pool_pays(*r),
            gm2d_core::stats::Stats::ZERO,
            "{} is on the panel and pays nothing",
            r.name()
        );
    }
}

#[test]
fn what_a_pool_pays_is_the_rulebooks_answer_and_not_a_second_copy() {
    // The rates the panel prints, asked of the same function the fight uses.
    // If `held_bonus` is retuned these move with it, which is the whole reason
    // the panel is derived rather than written down.
    assert_eq!(Combatant::pool_pays(Resource::Rage).physical_damage, 1);
    let faith = Combatant::pool_pays(Resource::Faith);
    assert_eq!((faith.physical_resist, faith.magic_resist), (2, 2));
    assert_eq!(Combatant::pool_pays(Resource::Nature).regen, 1);

    // And a point held is that point paid, in a fight rather than on a probe.
    let mut c = Combatant::player(gm2d_core::stats::Stats { health: 100, ..Default::default() }, &[]);
    let before = c.effective_physical_resist();
    c.set_pool(Resource::Faith, 10);
    assert_eq!(
        c.effective_physical_resist() - before,
        10 * Combatant::pool_pays(Resource::Faith).physical_resist,
        "ten points paid something other than ten times one point"
    );
}
