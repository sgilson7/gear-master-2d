//! What a player who walks into the Kettleworks can actually fight.
//!
//! Reported from play: *"the monsters in area 2 the kettleworks are like
//! insanely difficult to defeat by the time you get there ... the only one I
//! can reliably kill is the thing in the fortieth kettle due to timing it
//! out."* Both halves were true, and the second is the tell — a win at the
//! sudden-death clock is not a win a board earned.
//!
//! The yardstick below is the one the human named: **level ten, any class, one
//! assembled item to a grid.** It is generous about *gear* and mean about
//! *space*: it owns both shelves and everything the errands pay, and it packs
//! that onto the three-row frames a level ten has, one item a slot. A player
//! who has done less than this is behind it, which is the safe direction for a
//! check that says a region is enterable at all.

mod common;

use gm2d_core::character::Character;
use gm2d_core::combat::{self, Difficulty, Outcome};
use gm2d_core::piece::SlotKind;
use gm2d_core::rating::creature_rating;

const D: Difficulty = Difficulty::Easy;

/// The clock, not a board, ending the fight. `simulate` gives up around here
/// and grinds both sides down, so a "victory" this late is the sudden-death
/// rule finishing something the player could not.
const ON_THE_BUZZER_MS: u32 = 40_000;

/// Level ten, one assembled item to a grid, optionally classed.
///
/// **One item a grid, exactly**, because that is the yardstick: Auto-pack will
/// seat a second weapon if the room is there, and a check that let it would be
/// measuring a board with more in it than the one being described.
fn a_level_ten(class: Option<&str>) -> Character {
    let mut ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    // `geared_from` grows the frames to the ceiling. Put them back to what a
    // level ten stands up in: the base three, which is what every frame is
    // since a row stopped arriving with a level.
    for k in SlotKind::ALL {
        *ch.loadout.slot_mut(k) = gm2d_core::slot::Slot::with_rows(k, 3);
    }
    while ch.level() < 10 {
        ch.carry(50);
        ch.bank();
    }
    if let Some(c) = class {
        ch.choose_class(c).expect("a class the game offers");
    }
    ch.apply_preset();
    keep_one_item_a_grid(&mut ch);
    ch
}

/// Take everything off but the best item in each grid.
fn keep_one_item_a_grid(ch: &mut Character) {
    for k in SlotKind::ALL {
        // **Bounded, because a grid that will not give a piece up is a loop
        // that never ends.** The first version spun on exactly that and hung
        // the run rather than failing it; a check that cannot finish tells you
        // nothing at all.
        for _ in 0..8 {
            let report = ch.report(k);
            let mut made: Vec<_> = report
                .items
                .iter()
                .filter(|i| i.assembled)
                .map(|i| (i.rating, i.pieces.clone()))
                .collect();
            if made.len() <= 1 {
                break;
            }
            made.sort_by_key(|(r, _)| *r);
            // The weakest goes. Auto-pack locks each item as it lands, so
            // most of these come off as a set; a loose one comes off on its
            // own, and both roads are tried because which it is depends on how
            // the board was built rather than on anything this check knows.
            let mut moved = false;
            for p in made[0].1.clone() {
                moved |= ch.unequip_locked(p).is_ok() || ch.unequip(p).is_ok();
            }
            assert!(moved, "{k:?} holds {} items and will not give one up", made.len());
        }
    }
}

fn fight(ch: &Character, who: &str) -> combat::CombatLog {
    let m = combat::creature(who).unwrap_or_else(|| panic!("no creature named {who}"));
    combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D)
}

/// Everything the Kettleworks field draws, easiest first.
fn the_field() -> Vec<&'static str> {
    let w = gm2d_core::data::map("kettleworks-field", D);
    let r = w.regions.iter().find(|r| r.id == "kettleworks-field").expect("the field");
    let mut pool: Vec<_> = r.enemies.iter().map(|m| (creature_rating(m, D), m.name)).collect();
    pool.sort();
    pool.into_iter().map(|(_, n)| n).collect()
}

#[test]
fn the_yardstick_is_one_item_to_a_grid() {
    let ch = a_level_ten(None);
    let items = ch.combat_items();
    assert_eq!(items.len(), 5, "one item a grid is five items, not {}", items.len());
    for k in SlotKind::ALL {
        assert_eq!(
            items.iter().filter(|i| i.slot == k).count(),
            1,
            "{k:?} does not hold exactly one assembled item"
        );
    }
    assert_eq!(ch.level(), 10, "the yardstick is a level ten");
    assert_eq!(ch.slot_rows(), [3; 5], "on the frames a level ten stands up in");
}

/// **The acceptance the human named.** Galapagos Jim is The Curator, and he is
/// the commonest draw in the field — `draw_enemy` weights a pool so its
/// easiest member is its most frequent, so this is the fight the region is
/// mostly made of.
#[test]
fn galapagos_jim_is_beatable_by_a_level_ten_of_any_class() {
    for class in [None].into_iter().chain(gm2d_core::class::OFFERED.iter().copied().map(Some)) {
        let ch = a_level_ten(class);
        let log = fight(&ch, "The Curator");
        assert_eq!(
            log.outcome,
            Outcome::Victory,
            "{class:?} at level ten with one item a grid loses to The Curator"
        );
        assert!(
            log.duration_ms < ON_THE_BUZZER_MS,
            "{class:?} only beat The Curator on the sudden-death clock ({}ms), which is \
             the thing that was reported rather than the thing that was asked for",
            log.duration_ms
        );
    }
}

/// The region has to be enterable, and by damage rather than by the clock.
///
/// **The commonest draw, not every draw.** `draw_enemy` makes a pool's hardest
/// member its rarest, so what must not happen is that the fight you meet three
/// times in five is one you cannot win. The Hoop Hound at the top of this pool
/// is a fight this board loses, and that is the pool having a top.
#[test]
fn the_fight_the_field_mostly_deals_is_one_this_board_wins() {
    let ch = a_level_ten(None);
    let pool = the_field();
    let commonest = pool.first().expect("a pool with something in it");
    let log = fight(&ch, commonest);
    assert_eq!(log.outcome, Outcome::Victory, "{commonest} is the commonest draw and is a loss");
    assert!(
        log.duration_ms < ON_THE_BUZZER_MS,
        "{commonest} is the commonest draw and is only survived, not beaten: {}ms",
        log.duration_ms
    );
    // And not only the first: a region whose second-commonest fight is a wall
    // is a region you cannot grind in.
    let second = pool.get(1).expect("more than one creature in the field");
    assert_eq!(fight(&ch, second).outcome, Outcome::Victory, "{second} is a loss");
}

/// **Lord Drabley Henpeck is off the first map**, and kept for a boss fight.
#[test]
fn the_hollow_king_is_on_no_pool_on_the_first_map() {
    let w = gm2d_core::data::map("west-bambulon", D);
    for r in &w.regions {
        assert!(
            !r.enemies.iter().any(|m| m.name == "The Hollow King"),
            "{}: Lord Drabley Henpeck is drawn on the first map again",
            r.id
        );
        assert!(!r.enemies.is_empty(), "{}: a region with an empty pool", r.id);
        assert!(
            r.enemies.len() >= 2,
            "{}: a pool of one is a region that deals the same fight for ever",
            r.id
        );
    }
    // Kept, rather than deleted: he is a boss somebody has still to place.
    assert!(combat::creature("The Hollow King").is_some(), "the creature itself is gone");
}

/// **The Rice Criers moved to the Kettleworks.**
#[test]
fn the_rice_criers_are_drawn_in_the_kettleworks_and_not_at_home() {
    assert!(the_field().contains(&"Grave Chorus"), "The Rice Criers are not in the field pool");
    let w = gm2d_core::data::map("west-bambulon", D);
    for r in &w.regions {
        assert!(
            !r.enemies.iter().any(|m| m.name == "Grave Chorus"),
            "{}: The Rice Criers are still drawn on the first map",
            r.id
        );
    }
}

/// The pool came down, and by how much is written here rather than felt.
///
/// **The baseline is pinned rather than derived.** Backing the `gear_offset`
/// out of a creature gives what it rated before the *gear* moved, not before
/// the change — four of these had their body trimmed as well, because stepping
/// gear runs out of family: the Hoop Hound's footprints bottom out at 8% and
/// the Kettle Wight's at 9%, so the dial alone could not reach the band that
/// was asked for. These are the numbers the region had before any of it, taken
/// off `creature_rating` at `Difficulty::Easy`.
const WAS: &[(&str, i32)] = &[
    ("The Curator", 453),
    ("Grave Chorus", 471),
    ("Kettle Wight", 521),
    ("Pale Twin", 580),
    ("Hoop Hound", 643),
    ("Ruin Hound", 715),
];

#[test]
fn the_field_is_a_tenth_to_a_fifth_easier_than_it_was() {
    let w = gm2d_core::data::map("kettleworks-field", D);
    let field = w.regions.iter().find(|r| r.id == "kettleworks-field").expect("the field");
    // **Every creature the field actually deals**, which is not every creature
    // in it. `draw_enemy` weights a pool so its hardest member is its rarest,
    // and the Ruin Hound at the ceiling of this one is dealt 0% of the time —
    // easing it buys a player nothing here, and it is the ceiling of the Kolok
    // Downs as well, where lowering it promoted the Slag Warden to
    // hardest-and-therefore-rarest and took an instrument part from 25 wins to
    // 647. It is left where it is on purpose, and this is the sentence that
    // says so.
    let rated: Vec<i32> = field.enemies.iter().map(|m| creature_rating(m, D)).collect();
    let max = *rated.iter().max().expect("a pool with something in it");
    let total: i32 = rated.iter().map(|v| (max + 1 - v).max(1)).sum();
    let mut checked = 0;
    for (m, now) in field.enemies.iter().zip(&rated) {
        let share = (max + 1 - now).max(1) * 100 / total;
        let was = WAS.iter().find(|(n, _)| *n == m.name).map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{} joined the field and WAS does not know it", m.name));
        if share < 5 {
            assert_eq!(*now, was, "{} is never dealt and was moved anyway", m.name);
            continue;
        }
        let cut = (was - now) * 100 / was;
        assert!(
            (10..=20).contains(&cut),
            "{} is dealt {share}% of the time and came down {cut}% ({was} -> {now}); \
             the ask was ten to twenty",
            m.name
        );
        checked += 1;
    }
    assert!(checked >= 4, "only {checked} of the pool is drawn often enough to check");
}
