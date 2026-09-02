//! The three faults `PLANNING-BRIEF.md` §C names, each held down by a test.
//!
//! Two are code and are here. The third (§C.3, the shop quoting the catalogue
//! price rather than the charged one) was a fault in a CLI GM2D does not ship;
//! it survives as a UI rule in `CLAUDE.md` and will be a test in M3, when
//! there is a shop screen to assert against.

mod common;

use gm2d_core::character::Character;
use gm2d_core::combat::{simulate, Event, MonsterSpec, MonsterSprite, Outcome, Rank, Side};
use gm2d_core::piece::SlotKind;
use gm2d_core::reward::bounty_for;

// ------------------------------------------------------------------- §C.1

/// A loss pays nothing.
///
/// Upstream paid the bounty either way, on purpose and for a good reason that
/// only holds on a ladder. `reward.rs` carries the argument; this holds the
/// answer.
#[test]
fn loss_pays_no_bounty() {
    assert_eq!(bounty_for(Outcome::Victory, 17), 17, "a win pays the bounty");
    assert_eq!(bounty_for(Outcome::Defeat, 17), 0, "a loss pays nothing");
    assert_eq!(bounty_for(Outcome::Stalemate, 17), 0, "nothing was beaten");
}

/// The exploit the fix closes, stated as arithmetic.
///
/// Upstream's Grinder mode knocked you back one rung on a loss and paid the
/// bounty anyway, so lose-then-win netted a bounty for no progress — measured
/// at +17 a cycle, forever. In an open world there is no rung to knock back,
/// which makes the same cycle free rather than merely cheap.
#[test]
fn a_lose_win_cycle_is_not_a_gold_farm() {
    let bounty = 17;
    let cycle = bounty_for(Outcome::Defeat, bounty) + bounty_for(Outcome::Victory, bounty);
    assert_eq!(cycle, bounty, "a lose/win cycle pays exactly one win, not two");
}

// ------------------------------------------------------------------- §C.2

const DUMMY: MonsterSpec = MonsterSpec {
    name: "Dummy",
    health: 1_000_000,
    strength: 0,
    regen: 0,
    mind_resist: 0,
    physical_resist: 0,
    magic_resist: 0,
    curse_resist: 0,
    attacks: &[],
    gear: &[],
    gear_offset: 0,
    bounty: 0,
    sprite: MonsterSprite::Rat,
    rank: Rank::Ordinary,
    drops: &[],
    items: &[],
};

/// What the card says a weapon hits for is what the log says it hit for.
///
/// `ItemProfile.stats` is stored pre-multiplied by the item's power, and
/// upstream's `hit_for` multiplied by power a second time — so a weapon that
/// landed 30 in the log advertised 46 on the card, and the number a player
/// used to compare two builds was wrong by the size of their own power stat.
///
/// The dummy has no resistances, no armour and no attacks, so the first `Hit`
/// in the log is the swing itself with nothing taken off it. Nothing else in
/// the pipeline is allowed to sit between the two numbers.
#[test]
fn hit_for_matches_the_log() {
    let mut ch = Character::with_all_pieces();
    // **The fixture's board.** This is about one weapon's card agreeing with
    // that weapon's blow. A packed board banks rage, and rage sharpens the
    // physical half at the moment of the swing — which the card cannot know
    // and is not claiming to. Comparing the two on a board that banks nothing
    // is the only way this reads `hit_for` and nothing else.
    common::build_full_loadout(&mut ch);

    let items = ch.combat_items();
    let stats = ch.player_stats();
    assert!(
        items.iter().any(|i| i.slot == SlotKind::Weapon),
        "the packed board seats no weapon at all"
    );

    let log = simulate(stats, &items, &DUMMY);
    // **Whichever item actually swung**, not whichever is first in the list. A
    // packed board carries several weapons now, and picking the first while
    // reading the first blow compares one item's card against another item's
    // hit — which is a different bug wearing this one's clothes. The player has
    // no innate attacks, so `Activate`'s index is an index into `items`.
    let mut swung = None;
    let mut first = None;
    for e in &log.entries {
        match e.event {
            Event::Activate { side: Side::Player, index, .. } => swung = Some(index),
            Event::Hit { by: Side::Player, damage, absorbed, .. } => {
                first = Some((damage, absorbed));
                break;
            }
            _ => {}
        }
    }
    let first = first.expect("the packed board never landed a blow on a dummy");
    let weapon = &items[swung.expect("something activated before it hit")];
    let card = weapon.hit_for(stats.strength);
    assert!(card > 0, "the item that swung advertises no damage at all");

    assert_eq!(
        first.1, 0,
        "the dummy absorbed something, so this is not a clean reading"
    );
    assert_eq!(
        card, first.0,
        "the card says {} and the fight says {} - power is being applied twice somewhere",
        card, first.0
    );
}

/// The fix is arithmetic, not a fudge: doubling the wearer's strength moves
/// the advertised number by the wearer's share and nothing else.
///
/// Guards against the obvious wrong repair — dropping the power multiplier
/// altogether, which would make the card agree with the log for a power-100
/// item and disagree for every other one.
#[test]
fn hit_for_scales_the_wearer_and_not_the_item() {
    let mut ch = Character::with_all_pieces();
    ch.apply_preset();
    let items = ch.combat_items();
    let weapon = items
        .iter()
        .find(|i| i.slot == SlotKind::Weapon)
        .expect("the preset seats a weapon");

    let flat = weapon.hit_for(0);
    let with_str = weapon.hit_for(100);
    assert_eq!(
        with_str - flat,
        100 * weapon.power / 100,
        "a hundred strength should land as a hundred times the item's own power"
    );
}

// ------------------------------------------------------- M1's foundations

/// A seeded stream, saved and restored, keeps going rather than starting over.
///
/// M1's hard gate in miniature. Written now because `Rng::state` was added now
/// and an accessor with no test is an accessor somebody removes.
#[test]
fn an_rng_resumes_where_it_was_saved_not_where_it_started() {
    use gm2d_core::rng::Rng;
    let mut a = Rng::new(0x5EED_1234_ABCD_0001);
    let burned: Vec<u64> = (0..7).map(|_| a.next_u64()).collect();

    let mut b = Rng::from_state(a.state());
    let mut c = Rng::new(0x5EED_1234_ABCD_0001);

    let next_a: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
    let next_b: Vec<u64> = (0..5).map(|_| b.next_u64()).collect();
    let from_seed: Vec<u64> = (0..5).map(|_| c.next_u64()).collect();

    assert_eq!(next_a, next_b, "a restored state does not resume the stream");
    assert_ne!(
        next_b, from_seed,
        "restoring from the seed would replay draws the player has already seen"
    );
    assert_eq!(burned.len(), 7);
}

/// A board survives JSON: same pieces, same anchors, same rotations, same
/// locks, same name seed — and therefore the same items, with the same names.
///
/// The lock and the seed are asserted because the golden fixture found both
/// the expensive way. Without the locks a dense board reloads with different
/// items; without the seed every item is renamed and nothing else looks wrong.
#[test]
fn a_board_round_trips_through_json() {
    let mut before = Character::with_all_pieces();
    before.loadout.name_seed = 0x5EED_1234_ABCD_0001;
    before.apply_preset();
    before.toggle_lock_item(before.loadout.slot(SlotKind::Weapon).pieces()[0]);

    let json = serde_json::to_string(&before).expect("a character serialises");
    let mut after: Character = serde_json::from_str(&json).expect("and comes back");
    // The naming corpus is theme data reached by pointer, so a save carries
    // the theme id and the loader re-points it. M1 does that from the id; here
    // there is no save envelope yet, so it is done by hand.
    after.loadout.naming = before.loadout.naming;

    assert_eq!(after.loadout.name_seed, before.loadout.name_seed, "the name seed");
    assert_eq!(after.loadout.locks, before.loadout.locks, "the locks");
    assert_eq!(after.slot_rows(), before.slot_rows(), "the board sizes");
    assert_eq!(after.owned, before.owned, "what is owned");

    let want: Vec<(String, i32)> =
        before.combat_items().iter().map(|i| (i.name.clone(), i.rating)).collect();
    let got: Vec<(String, i32)> =
        after.combat_items().iter().map(|i| (i.name.clone(), i.rating)).collect();
    assert!(!want.is_empty(), "the preset assembled nothing, so this proves nothing");
    assert_eq!(want, got, "the same board came back as different items");

    assert_eq!(
        before.player_stats(),
        after.player_stats(),
        "the character sheet moved across a round trip"
    );
}
