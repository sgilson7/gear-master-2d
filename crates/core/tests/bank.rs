//! The bank: one vault, every town, and what "banked" costs you.
//!
//! Asked for as *"make it so you can place your items from your bag into a
//! bank accessible from any town, its the same bank for all towns with
//! infinite size"*. The size is the easy half. The half worth testing is that
//! a deposited component is genuinely **out of play** — it does not pack, it
//! does not bench, and it is not a thing you are holding when somebody at a
//! counter asks. A bank that left everything counting as yours would be a
//! bigger bag with a screen in front of it.

mod common;

use gm2d_core::character::Character;
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;
use gm2d_core::{pressure, quest, save};

/// A loose component goes in, and leaves the bag.
#[test]
fn a_loose_component_banks() {
    let mut ch = Character::with_all_pieces();
    let id = *ch.owned.iter().find(|&&i| !ch.is_equipped(i)).unwrap();
    let had = ch.owned.len();

    ch.deposit(id).expect("a loose component banks");
    assert!(!ch.owned.contains(&id), "a banked component is still in the bag");
    assert!(ch.banked.contains(&id), "a banked component is not in the bank");
    assert_eq!(ch.owned.len(), had - 1, "banking did not take it out of the bag");
}

/// And comes back out into the bag, not onto a board.
///
/// Where a component goes is the packing screen's question; this is a counter.
#[test]
fn what_is_banked_comes_back() {
    let mut ch = Character::with_all_pieces();
    let id = *ch.owned.iter().find(|&&i| !ch.is_equipped(i)).unwrap();
    ch.deposit(id).unwrap();
    ch.withdraw(id).expect("what went in comes out");

    assert!(ch.owned.contains(&id), "it did not come back to the bag");
    assert!(ch.banked.is_empty(), "it is in two places");
    assert!(!ch.is_equipped(id), "it came back onto a board");
}

/// A seated component is refused **by name**, and nothing moves.
///
/// Banking happens in a town, where the board is not on the screen. Lifting a
/// piece off a grid here would break an item somewhere the player cannot watch
/// it happen — so it is refused, and the refusal says which piece and what to
/// do. *A refusal spends nothing* is the rule the shop's reroll already obeys.
#[test]
fn a_seated_component_is_refused_and_stays_put() {
    let mut ch = common::preset_board();
    let id = ch.loadout.slot(SlotKind::Weapon).pieces()[0];
    let name = ch.registry.def(id).name;
    let before = (ch.owned.clone(), ch.banked.clone());

    let err = ch.deposit(id).expect_err("a seated component must not bank");
    assert!(err.contains(name), "the refusal does not name the piece: {err}");
    assert_eq!((ch.owned.clone(), ch.banked.clone()), before, "a refusal moved something");
    assert!(ch.is_equipped(id), "a refused deposit took it off the board anyway");
}

/// Something that is not yours is refused too, and says so.
#[test]
fn the_bank_refuses_what_you_do_not_have() {
    let mut ch = Character::with_all_pieces();
    let id = *ch.owned.iter().find(|&&i| !ch.is_equipped(i)).unwrap();
    ch.deposit(id).unwrap();

    // Already in the bank, so not in the bag: banking it again is nonsense.
    assert!(ch.deposit(id).is_err(), "a banked component banked twice");
    // And withdrawing something that was never deposited.
    let other = *ch.owned.iter().find(|&&i| !ch.is_equipped(i)).unwrap();
    assert!(ch.withdraw(other).is_err(), "the bank handed over what it never had");
}

/// **Banked is not carried**, which is the whole point of the feature.
///
/// Three consumers, each asked directly rather than by reading the field:
/// Auto-pack cannot seat it, the bench does not count it, and a counter does
/// not see it. All three read `owned`, so all three are right for free — and
/// this is here to say so out loud, because the next person to add a fourth
/// consumer will read one of them.
#[test]
fn what_is_banked_is_out_of_play() {
    let mut g = Game::new(7, "td");
    g.character = Character::with_all_pieces();

    let id = *g.character.owned.iter().find(|&&i| !g.character.is_equipped(i)).unwrap();
    let name = g.character.registry.def(id).name;
    let held_before = quest::holding(&g, name);
    assert!(held_before > 0, "the fixture is not holding the piece under test");

    g.character.deposit(id).unwrap();

    assert_eq!(
        quest::holding(&g, name),
        held_before - 1,
        "a counter still counts a banked component as held",
    );

    // The bench is owned components that fit nowhere. A banked one is not
    // owned, so it cannot be on it — asked through `pressure::of`, which is
    // where the milestone's own number comes from.
    //
    // **Not a comparison that can come out equal.** Everything benched is
    // banked and the bench has to reach zero; an assertion that banking one
    // piece does not *raise* the bench would pass on a bank that did nothing,
    // which is the shape of check this project has shipped vacuous three
    // times.
    // Packed first, because on an empty board everything fits and a bench of
    // zero would make the assertion below true for the wrong reason.
    g.character.apply_preset();
    let benched: Vec<_> = g
        .character
        .owned
        .iter()
        .copied()
        .filter(|&i| !g.character.fits_anywhere(i))
        .collect();
    assert!(
        !benched.is_empty(),
        "the fixture benches nothing, so this proves nothing",
    );
    assert!(pressure::of(&g.character).bench > 0, "a bench of things that fit nowhere is zero");
    for id in benched {
        g.character.deposit(id).unwrap();
    }
    assert_eq!(
        pressure::of(&g.character).bench,
        0,
        "the bench still counts components that are in the bank",
    );

    // And Auto-pack: whatever it seats, none of it is in the bank.
    if g.character.owned.contains(&id) {
        g.character.deposit(id).unwrap();
    }
    g.character.apply_preset();
    assert!(
        !g.character.is_equipped(id),
        "Auto-pack seated a component out of the bank",
    );
}

/// A bank survives being written down, with its enchs still on.
///
/// An ench names a `PieceId` and not a cell, so it rides into the vault and
/// back out. The registry keeps a deposited piece, which is what makes the id
/// mean the same component on the far side.
#[test]
fn a_bank_round_trips() {
    let mut g = Game::new(0x8A_9B_1234, "td");
    g.character = Character::with_all_pieces();
    g.character.loadout.name_seed = 0x8A_9B_1234;
    g.character.loadout.naming = gm2d_core::theme::by_id("td").naming;

    let ids: Vec<_> = g
        .character
        .owned
        .iter()
        .copied()
        .filter(|&i| !g.character.is_equipped(i))
        .take(3)
        .collect();
    for id in &ids {
        g.character.deposit(*id).unwrap();
    }
    let names: Vec<_> = ids.iter().map(|&i| g.character.registry.def(i).name).collect();

    let back = save::load(&save::save(&g)).expect("a save this build wrote loads");
    assert_eq!(back.character.banked, ids, "the bank did not survive the round trip");
    let got: Vec<_> = back
        .character
        .banked
        .iter()
        .map(|&i| back.character.registry.def(i).name)
        .collect();
    assert_eq!(got, names, "the bank came back holding different components");
}

/// A save written before there was a bank opens with an empty one.
///
/// `banked` is `#[serde(default)]` and skipped when empty, so no fingerprint
/// moved and no file is refused — which is what every character who has one
/// of those files had: no bank.
#[test]
fn a_save_from_before_the_bank_opens_without_one() {
    let g = Game::new(3, "td");
    let text = save::save(&g);
    assert!(!text.contains("\"banked\""), "an empty bank is being written into every save");
    let back = save::load(&text).expect("it loads");
    assert!(back.character.banked.is_empty(), "a bank appeared out of nowhere");
}
