//! An item that fires once and is finished.
//!
//! **The first new rule in the fight since the fork**, and the smallest one the
//! game could be given: `RunningItem` already carried `has_fired` for Overtake
//! and a per-item `stun_ms` checked once in the tick, so this is one field, one
//! branch beside it, and one event.
//!
//! It is invented for an *ench* rather than for a class — the class M10.2 adds
//! reuses a power that was already tuned — but the distinction is thin and is
//! worth saying out loud rather than letting somebody find it.

mod common;

use gm2d_core::character::Character;
use gm2d_core::combat::{simulate_at, Difficulty, Event, Side};
use gm2d_core::data;
use gm2d_core::ench::{Ench, Effect};
use gm2d_core::piece::SlotKind;

const D: Difficulty = Difficulty::Medium;

/// The shipped ench that breaks, and the one thing that must be true of it.
fn fragile() -> gm2d_core::ench::EnchDef {
    data::enchs()
        .enchs
        .into_iter()
        .find(|e| matches!(e.effect, Effect::Fragile { .. }))
        .expect("one ships")
}

/// A board with one weapon on it, and the ench bolted to a component of it.
fn wearing_it(on: bool) -> Character {
    let mut ch = common::bench();
    ch.class = Some(gm2d_core::ench::LICENSED_CLASS.to_string());
    common::build_full_loadout(&mut ch);
    if on {
        let weapon = ch
            .report(SlotKind::Weapon)
            .items
            .into_iter()
            .find(|i| i.assembled)
            .expect("the fixture assembles a weapon");
        let e = fragile();
        ch.give_ench(&e.id);
        ch.attach_ench(&e.id, weapon.pieces[0]).expect("a licensee can bolt it on");
    }
    ch
}

fn fight(ch: &Character) -> gm2d_core::combat::CombatLog {
    // Something with enough health to outlast a board, so the fight is long
    // enough for a second activation to have happened if one were coming.
    let spec = gm2d_core::combat::creature("Rust Colossus").expect("it exists");
    simulate_at(ch.player_stats(), &ch.combat_items(), spec, D)
}

fn activations(log: &gm2d_core::combat::CombatLog, item: &str) -> usize {
    log.entries
        .iter()
        .filter(|e| {
            matches!(&e.event, Event::Activate { side: Side::Player, item: n, .. }
                     if n.starts_with(item))
        })
        .count()
}

/// **It fires once**, and the rest of the board plays on around it.
#[test]
fn an_item_that_breaks_fires_once() {
    let ch = wearing_it(true);
    let name = ch
        .combat_items()
        .into_iter()
        .find(|p| p.fragile)
        .expect("the ench reached the profile")
        .name;
    let log = fight(&ch);

    assert_eq!(activations(&log, &name), 1, "{name} fired more than once");
    let broke: Vec<&gm2d_core::combat::LogEntry> = log
        .entries
        .iter()
        .filter(|e| matches!(&e.event, Event::Broke { .. }))
        .collect();
    assert_eq!(broke.len(), 1, "it broke {} times", broke.len());

    // The rest of the kit is untouched: this stops one item, not a fighter.
    let others: usize = log
        .entries
        .iter()
        .filter(|e| matches!(&e.event, Event::Activate { side: Side::Player, item: n, .. }
                             if !n.starts_with(&name)))
        .count();
    assert!(others > 3, "the whole board stopped, not one item ({others} other activations)");

    // And without the ench it fires plenty.
    let plain = wearing_it(false);
    let log = fight(&plain);
    assert!(
        activations(&log, &name) > 1,
        "the fixture's weapon only ever fires once, so this test proves nothing"
    );
}

/// **The activation that breaks it pays in full**, which is the whole bargain.
#[test]
fn the_activation_that_breaks_it_pays_in_full() {
    let ch = wearing_it(true);
    let log = fight(&ch);
    let at = log
        .entries
        .iter()
        .position(|e| matches!(&e.event, Event::Broke { .. }))
        .expect("it broke");
    let fired = log.entries[..at]
        .iter()
        .rposition(|e| matches!(&e.event, Event::Activate { side: Side::Player, .. }))
        .expect("it activated before it broke");
    // Everything between the activation and the break is what that activation
    // paid, and it has to include the blow itself.
    let paid = log.entries[fired..at]
        .iter()
        .any(|e| matches!(&e.event, Event::Hit { .. } | Event::GainResource { .. }));
    assert!(paid, "it broke without paying for the swing that broke it");
    assert_eq!(
        log.entries[at].at_ms,
        log.entries[fired].at_ms,
        "the break landed on a different tick from the swing"
    );
}

/// **Three times the power**, which is the half that sells it.
#[test]
fn the_ench_is_worth_three_of_it() {
    let Effect::Fragile { pct } = fragile().effect else { panic!("not a fragile ench") };
    assert_eq!(pct, 200, "power starts at 100, so 3x is +200 percentage points");

    let with = wearing_it(true);
    let without = wearing_it(false);
    let hit = |ch: &Character| -> i32 {
        ch.combat_items()
            .into_iter()
            .filter(|p| p.slot == SlotKind::Weapon)
            .map(|p| p.hit_for(ch.player_stats().strength))
            .max()
            .unwrap_or(0)
    };
    let (a, b) = (hit(&with), hit(&without));
    assert!(a > b, "the ench did not make the swing bigger at all ({a} vs {b})");
}

/// A broken bar does not turn, the sibling of the stun's rule.
#[test]
fn a_broken_item_does_not_turn() {
    let mut ch = wearing_it(true);
    // The spin as well as the break, on two different components, so the item
    // both turns and is finished.
    let weapon = ch
        .report(SlotKind::Weapon)
        .items
        .into_iter()
        .find(|i| i.assembled)
        .expect("assembles");
    if weapon.pieces.len() > 1 {
        ch.give_ench("the-ponkey-turn");
        let _ = ch.attach_ench("the-ponkey-turn", weapon.pieces[1]);
    }
    let log = fight(&ch);
    let broke = log
        .entries
        .iter()
        .position(|e| matches!(&e.event, Event::Broke { .. }));
    if let Some(at) = broke {
        let after = log.entries[at..]
            .iter()
            .filter(|e| matches!(&e.event, Event::Turned { side: Side::Player, .. }))
            .count();
        // The weapon is the only thing enched, so any turn after the break is
        // the broken item still running its own clock.
        assert_eq!(after, 0, "a broken item kept turning");
    }
}

/// An ench does not move an item's rating, in either direction.
///
/// `PLAN-M10.md` called an unrated `fragile` the risk that would cost a day —
/// it would make a Chonga'd blade the best item in the game by the shop's
/// reckoning, which prices every rarity mark on the board and picks what an
/// aimed stun goes for. It does not, and the reason is that **no** ench reaches
/// the rating: `item_rating` prices the pieces and the cadence, and `ench::apply`
/// runs over the profiles afterwards. Pinned here so the day is not spent
/// twice, and so that an ench that starts moving it has to say so.
#[test]
fn an_ench_does_not_move_what_an_item_is_worth() {
    let with = wearing_it(true);
    let without = wearing_it(false);
    let ratings = |ch: &Character| -> Vec<i32> {
        let mut v: Vec<i32> = ch.combat_items().iter().map(|p| p.rating).collect();
        v.sort_unstable();
        v
    };
    assert_eq!(ratings(&with), ratings(&without));
}

/// **A stun does not aim at something that is already finished.**
///
/// The code's own comment says stunning what is already stopped is the one
/// outcome an aimed stun must not have — and a broken item is the worse case of
/// the two, because a stun on a stopped item is a curse wasted for a second and
/// a stun on a broken one is wasted for the fight.
#[test]
fn an_aimed_stun_passes_over_a_broken_item() {
    use gm2d_core::combat::{Combatant, StunAim};
    let ch = wearing_it(true);
    let mut c = Combatant::player(ch.player_stats(), &ch.combat_items());
    assert!(c.items.len() > 1, "one item cannot show a choice being made");
    // Break the best of them, which is what an aimed stun would otherwise take.
    let best = c
        .items
        .iter()
        .enumerate()
        .max_by_key(|(_, it)| it.rating)
        .map(|(i, _)| i)
        .unwrap();
    c.items[best].broken = true;
    let (idx, _) = gm2d_core::combat::land_stun_for_test(&mut c, StunAim::Strongest, 0)
        .expect("something takes it");
    assert_ne!(idx, best, "the stun landed on an item that was already finished");
}

// ------------------------------------------------------------------ the file

/// **The Swing is a class's, and it says both halves of what it does.**
///
/// M10.1 shipped it on the van's table with no class attached, so the mechanic
/// could get a player's opinion before anything was built on it —
/// `PLAN-M10.md` said moving it into the tree afterwards would cost nothing,
/// and M10.2 moved it. It is priceless now, like the Yodregar Index and for the
/// same reason: a thing you can buy is not a thing a class is *about*.
#[test]
fn the_swing_is_a_class_s_and_says_what_it_costs() {
    let e = fragile();
    assert!(e.price.is_none(), "it is awarded, so nothing prices it");
    let line = e.effect.line();
    assert!(line.contains("200"), "the spec does not name the power: {line:?}");
    assert!(
        line.contains("break") || line.contains("once"),
        "the spec names the power and not the cost, which is the half that sells it: {line:?}"
    );
    assert!(!e.effect.detail().is_empty());

    let sold: Vec<String> = data::all_maps(Difficulty::Easy)
        .iter()
        .flat_map(|w| w.places.clone())
        .flat_map(|p| p.sells)
        .collect();
    assert!(!sold.contains(&e.id), "{} is on a table, and it is a class's", e.id);

    let tree = data::skills();
    let by = tree
        .trees
        .iter()
        .find(|t| {
            t.nodes.iter().any(|n| {
                n.effects.iter().any(|f| {
                    matches!(f, gm2d_core::skills::Effect::GivesEnch { ench } if *ench == e.id)
                })
            })
        })
        .unwrap_or_else(|| panic!("nothing awards {}", e.id));
    assert!(by.class.is_some(), "the Swing is awarded by the base tree, so it is nobody's");
}

/// Bolting it on and taking it off leaves the board where it was.
#[test]
fn the_break_is_for_the_fight_and_not_for_good() {
    let ch = wearing_it(true);
    let before: Vec<String> = ch.combat_items().iter().map(|p| p.name.clone()).collect();
    let _ = fight(&ch);
    let after: Vec<String> = ch.combat_items().iter().map(|p| p.name.clone()).collect();
    assert_eq!(before, after, "a fight took a component away");
    // Two fights in a row, and the second is the same fight.
    let a = fight(&ch);
    let b = fight(&ch);
    assert_eq!(a.entries.len(), b.entries.len(), "the item did not come back whole");
    let _ = Ench { on: ch.owned[0], id: fragile().id, active: true };
}
