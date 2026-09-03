//! Placement and assembly rules: recipes, the touching requirement, several
//! finished items per slot, and assembly bonuses firing only on success.

mod common;

use common::{build_full_loadout, equip, piece};
use gm2d_core::piece::SlotKind;
use gm2d_core::character::Character;
use gm2d_core::slot::PlaceError;
use gm2d_core::stats::Stats;

// ------------------------------------------------------------- placement

#[test]
fn a_piece_only_goes_in_its_own_slot() {
    let mut ch = Character::with_all_pieces();
    let blade = piece(&ch, "Iron Blade");

    let err = ch.equip(blade, SlotKind::Helmet, 0, 0).unwrap_err();
    assert_eq!(err.to_string(), PlaceError::WrongSlot.to_string());
    assert!(!ch.is_equipped(blade), "a rejected equip must not place it");
}

#[test]
fn a_shape_may_not_hang_off_the_edge() {
    let ch = Character::with_all_pieces();
    let base = piece(&ch, "Padded Base"); // 4 wide, 3 tall, in a 6x8 slot

    assert!(ch.can_equip(base, SlotKind::Chest, 2, 5).is_ok(), "fits at the far corner");
    assert_eq!(
        ch.can_equip(base, SlotKind::Chest, 3, 0).unwrap_err().to_string(),
        PlaceError::OutOfBounds.to_string(),
        "one column too far right"
    );
}

#[test]
fn pieces_may_not_overlap() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0); // occupies (0, 0..3)
    let blade = piece(&ch, "Iron Blade");

    assert_eq!(
        ch.can_equip(blade, SlotKind::Weapon, 0, 2).unwrap_err().to_string(),
        PlaceError::Occupied.to_string()
    );
    assert!(ch.can_equip(blade, SlotKind::Weapon, 1, 0).is_ok(), "the next column is free");
}

#[test]
fn equipping_removes_a_piece_from_the_inventory() {
    let mut ch = Character::with_all_pieces();
    let before = ch.inventory().len();
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0);

    assert_eq!(ch.inventory().len(), before - 1);
    assert_eq!(ch.loadout.slot_holding(piece(&ch, "Balanced Grip")), Some(SlotKind::Weapon));
}

#[test]
fn unequipping_returns_a_piece_to_the_inventory() {
    let mut ch = Character::with_all_pieces();
    let grip = piece(&ch, "Balanced Grip");
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0);

    ch.unequip(grip).expect("equipped, so it can come off");

    assert!(!ch.is_equipped(grip));
    assert!(ch.inventory().contains(&grip));
    assert_eq!(ch.inventory().len(), ch.owned.len());
}

#[test]
fn moving_a_piece_within_its_slot_does_not_collide_with_itself() {
    let mut ch = Character::with_all_pieces();
    let grip = piece(&ch, "Balanced Grip");
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0); // (0, 0..3)

    // Shift down one row — the new footprint overlaps the old one.
    ch.equip(grip, SlotKind::Weapon, 0, 1).expect("a piece never blocks itself");

    assert_eq!(ch.loadout.slot(SlotKind::Weapon).anchor_of(grip), Some((0, 1)));
    assert_eq!(ch.loadout.slot(SlotKind::Weapon).get(0, 0), None, "old cell released");
}

// -------------------------------------------------------------- recipes

#[test]
fn an_empty_slot_holds_no_items() {
    let ch = Character::with_all_pieces();
    for slot in SlotKind::ALL {
        let r = ch.report(slot);
        assert!(r.is_empty(), "{} should start empty", slot.name());
        assert_eq!(r.summary(), "empty");
        assert_eq!(r.stats, Stats::ZERO);
    }
}

#[test]
fn a_weapon_needs_a_damaging_piece_as_well_as_a_handle() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0);

    let r = ch.report(SlotKind::Weapon);
    assert_eq!(r.assembled_count(), 0);
    assert_eq!(r.items[0].status, "needs 1 more damaging");
}

#[test]
fn a_weapon_assembles_from_a_handle_and_a_blade() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);

    let r = ch.report(SlotKind::Weapon);
    assert_eq!(r.assembled_count(), 1, "{}", r.summary());
    assert_eq!(r.summary(), "1 item assembled");
}

#[test]
fn components_that_do_not_touch_are_judged_as_separate_items() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 0, 0); // column 0
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 3, 0); // column 3 — a gap between

    let r = ch.report(SlotKind::Weapon);
    assert_eq!(r.items.len(), 2, "two groups, not one weapon");
    assert_eq!(r.assembled_count(), 0);
    // Each half complains about what it is missing on its own.
    let statuses: Vec<&str> = r.items.iter().map(|i| i.status.as_str()).collect();
    assert!(statuses.contains(&"needs 1 more damaging"), "{:?}", statuses);
    assert!(statuses.contains(&"needs 1 more handle"), "{:?}", statuses);
}

#[test]
fn too_many_components_of_one_kind_in_a_single_item_is_rejected() {
    let mut ch = Character::with_all_pieces();
    // One base with four layers glued to it: one layer over the maximum.
    equip(&mut ch, "Padded Base", SlotKind::Chest, 0, 0); // (0..3, 0..2)
    equip(&mut ch, "Chain Layer", SlotKind::Chest, 0, 3);
    equip(&mut ch, "Plate Layer", SlotKind::Chest, 0, 4);
    equip(&mut ch, "Woven Underlayer", SlotKind::Chest, 0, 5);
    assert_eq!(ch.report(SlotKind::Chest).assembled_count(), 1, "three layers is the max");

    equip(&mut ch, "Hollow Weave", SlotKind::Chest, 0, 6);

    let r = ch.report(SlotKind::Chest);
    assert_eq!(r.items.len(), 1, "all five are touching, so it is one item");
    assert_eq!(r.items[0].status, "too many layer (max 3)");
    assert_eq!(r.assembled_count(), 0);
}

// -------------------------------------------------- several items a slot

#[test]
fn one_slot_can_hold_two_finished_items() {
    let mut ch = Character::with_all_pieces();
    // Two complete gloves, kept apart by empty rows 2 and 3.
    equip(&mut ch, "Leather Material", SlotKind::Gloves, 0, 0); // (0..1, 0..1)
    equip(&mut ch, "Gripping Mold", SlotKind::Gloves, 2, 0); // (2..3, 0), (2, 1)
    equip(&mut ch, "Steel Material", SlotKind::Gloves, 0, 4); // (0..1, 4..6)
    equip(&mut ch, "Gauntlet Mold", SlotKind::Gloves, 2, 4); // (2, 4..6), (3, 6)

    let r = ch.report(SlotKind::Gloves);
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.assembled_count(), 2, "{}", r.summary());
    assert_eq!(r.summary(), "2 items assembled");
    // Both items' stats count: 2 + 15x power, then 5 hp + 4 + 1 str + 2 bonus.
    assert_eq!((r.stats.health, r.stats.strength, r.stats.power), (5, 9, 15));
}

#[test]
fn two_items_may_sit_flush_against_each_other() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Leather Material", SlotKind::Gloves, 0, 0); // (0..1, 0..1)
    equip(&mut ch, "Gripping Mold", SlotKind::Gloves, 2, 0); // touches the leather
    // Butted straight up against the first glove, with no gap at all.
    equip(&mut ch, "Steel Material", SlotKind::Gloves, 0, 2); // (0..1, 2..4)

    let r = ch.report(SlotKind::Gloves);
    // Two materials means two cores, so two items — even though every piece
    // here is one connected lump.
    assert_eq!(r.items.len(), 2, "each core anchors its own item");
    assert_eq!(r.assembled_count(), 1, "leather + mold is a finished glove");
    assert_eq!(r.loose_count(), 1, "the steel material still wants a mold");
}

#[test]
fn a_loose_piece_joins_whichever_core_it_is_nearest() {
    let mut ch = Character::with_all_pieces();
    // Two handles in a row with a single blade hanging off the second one.
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0); // (0, 0..2)
    equip(&mut ch, "Balanced Grip", SlotKind::Weapon, 1, 0); // (1, 0..3)
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 2, 0); // (2, 0..3), touches the grip

    let r = ch.report(SlotKind::Weapon);
    assert_eq!(r.items.len(), 2, "two handles, two weapons");

    let grip = piece(&ch, "Balanced Grip");
    let blade = piece(&ch, "Iron Blade");
    let with_grip = r.items.iter().find(|i| i.pieces.contains(&grip)).unwrap();
    assert!(
        with_grip.pieces.contains(&blade),
        "the blade belongs to the handle it actually touches"
    );
    assert!(with_grip.assembled, "handle + blade is a weapon");

    let oak = piece(&ch, "Oak Handle");
    let lonely = r.items.iter().find(|i| i.pieces.contains(&oak)).unwrap();
    assert!(!lonely.assembled);
    assert_eq!(lonely.status, "needs 1 more damaging");
}

#[test]
fn a_blob_with_no_core_at_all_is_one_unfinished_item() {
    let mut ch = Character::with_all_pieces();
    // Two layers touching, and not a base between them.
    equip(&mut ch, "Chain Layer", SlotKind::Chest, 0, 0);
    equip(&mut ch, "Plate Layer", SlotKind::Chest, 0, 1);

    let r = ch.report(SlotKind::Chest);
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].status, "needs 1 more base");
}

#[test]
fn a_slot_can_hold_a_finished_item_and_loose_pieces_at_once() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Leather Material", SlotKind::Gloves, 0, 0);
    equip(&mut ch, "Gripping Mold", SlotKind::Gloves, 2, 0);
    equip(&mut ch, "Steel Material", SlotKind::Gloves, 0, 4); // no mold to pair with

    let r = ch.report(SlotKind::Gloves);
    assert_eq!(r.assembled_count(), 1);
    assert_eq!(r.loose_count(), 1);
    assert_eq!(r.summary(), "1 assembled, 1 loose");
    // The loose material still contributes its base stats.
    assert_eq!((r.stats.health, r.stats.strength, r.stats.power), (5, 6, 15));
}

#[test]
fn every_slot_assembles_on_the_preset_loadout() {
    let mut ch = Character::with_all_pieces();
    build_full_loadout(&mut ch);

    for slot in SlotKind::ALL {
        let r = ch.report(slot);
        assert!(
            r.assembled_count() >= 1,
            "{} failed to assemble: {}",
            slot.name(),
            r.summary()
        );
        // Enchantments excepted, because "loose" is what an enchantment
        // permanently is: it lies under the grid, no recipe names its kind,
        // and `groups` walks the gear layer - so it can never join an item and
        // the report has nowhere else to file it. The preset lays one under
        // the chest.
        let stranded = r
            .items
            .iter()
            .filter(|i| !i.assembled)
            .filter(|i| {
                !i.pieces.iter().all(|&p| ch.registry.def(p).kind.is_enchantment())
            })
            .count();
        assert_eq!(stranded, 0, "{} left loose pieces: {}", slot.name(), r.summary());
    }
    // Chest, gloves and greaves each carry two separate items.
    assert_eq!(ch.report(SlotKind::Chest).assembled_count(), 2);
    assert_eq!(ch.report(SlotKind::Gloves).assembled_count(), 2);
    assert_eq!(ch.report(SlotKind::Greaves).assembled_count(), 2);
}

// ---------------------------------------------------- assembly bonuses

#[test]
fn an_assembly_bonus_stays_dormant_until_the_item_assembles() {
    let mut ch = Character::with_all_pieces();
    // Runed Material alone: base +12 armour, and its +75 health bonus must NOT
    // fire.
    //
    // Read on two different stats on purpose. It used to carry base health and
    // a health bonus, so "base only" and "base plus bonus" were 25 and 100 -
    // one number doing two jobs, and a zero would have looked like either. The
    // sweep took the base health off it, because a Material floats between
    // gloves and greaves and health above fifteen is the body's; what it gives
    // now is armour, which belongs to nobody. Base in one currency and bonus in
    // another says which is which.
    equip(&mut ch, "Runed Material", SlotKind::Greaves, 0, 0);

    let r = ch.report(SlotKind::Greaves);
    assert_eq!(r.assembled_count(), 0);
    assert_eq!(r.stats.armor, 12, "only the base contribution");
    assert_eq!(r.stats.health, 0, "and the bonus is dormant");
    assert!(r.notes().is_empty());

    // Add the mold next to it and the greaves come together.
    equip(&mut ch, "Greave Mold", SlotKind::Greaves, 2, 0);

    let r = ch.report(SlotKind::Greaves);
    assert_eq!(r.assembled_count(), 1, "{}", r.summary());
    // The mold's +1 regen is gone: regeneration is the body's, and the greaves
    // sweep sent the feet's padding to the chest. What Greave Mold gives now is
    // cadence, which a stat report has no column for - so this reads zero and
    // means "the mold contributes nothing this report can see", which is the
    // truth. The two numbers that carry the test are the first two.
    assert_eq!((r.stats.armor, r.stats.health, r.stats.regen), (12, 75, 0), "base armour kept, bonus health added, and the mold's padding gone");
    // "Runed", not "Runed: +75 health". A label is a name now and the figure
    // comes from the stat block, which is the line above this one - twenty-nine
    // labels stated their own numbers in prose and eight stated nothing at all,
    // and both halves were the same fault.
    assert_eq!(r.notes(), vec!["Runed"]);
}

#[test]
fn breaking_the_assembly_switches_the_bonus_back_off() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Runed Material", SlotKind::Greaves, 0, 0);
    equip(&mut ch, "Greave Mold", SlotKind::Greaves, 2, 0);
    assert_eq!(ch.report(SlotKind::Greaves).stats.health, 75);

    // Slide the mold away so nothing touches any more.
    let mold = piece(&ch, "Greave Mold");
    ch.equip(mold, SlotKind::Greaves, 4, 4).expect("legal placement");

    let r = ch.report(SlotKind::Greaves);
    assert_eq!(r.assembled_count(), 0);
    assert_eq!(r.stats.health, 0, "the +75 bonus is withdrawn");
    assert_eq!(r.stats.armor, 12, "and the base is still there");
    assert!(r.notes().is_empty());
}

#[test]
fn each_slots_bonus_fires_exactly_once_on_the_preset() {
    let mut ch = Character::with_all_pieces();
    build_full_loadout(&mut ch);

    let notes: Vec<String> = ch.reports().iter().flat_map(|r| r.notes()).collect();
    // Names, not specifications: the figures are the stat block's and the card
    // prints them from there.
    for label in ["Focused", "Woven", "Gauntleted", "Runed", "Balanced"] {
        assert_eq!(
            notes.iter().filter(|n| n.as_str() == label).count(),
            1,
            "expected {:?} exactly once in {:?}",
            label,
            notes
        );
    }
}

// ------------------------------------------------------------- rotation

#[test]
fn rotating_an_equipped_piece_that_no_longer_fits_changes_nothing() {
    let mut ch = Character::with_all_pieces();
    let base = piece(&ch, "Padded Base"); // 4 wide x 3 tall
    equip(&mut ch, "Padded Base", SlotKind::Chest, 2, 0); // occupies x 2..5

    // Rotated it is 3 wide x 4 tall — still fine — so confirm the legal case
    // first, then wedge it where the turn cannot happen.
    ch.rotate(base).expect("3x4 fits at x=2");
    assert_eq!(ch.registry.rotation(base), 1);

    ch.equip(base, SlotKind::Chest, 3, 4).expect("3x4 fits at (3, 4)");
    let err = ch.rotate(base).unwrap_err();

    assert_eq!(err.to_string(), PlaceError::OutOfBounds.to_string());
    assert_eq!(ch.registry.rotation(base), 1, "rotation rolled back");
    assert_eq!(
        ch.loadout.slot(SlotKind::Chest).anchor_of(base),
        Some((3, 4)),
        "and the piece stayed put"
    );
}

#[test]
fn rotating_a_piece_in_the_inventory_always_works() {
    let mut ch = Character::with_all_pieces();
    let mold = piece(&ch, "Gauntlet Mold");
    let before = ch.registry.shape(mold);

    ch.rotate(mold).expect("nothing constrains an unequipped piece");

    assert_ne!(ch.registry.shape(mold), before);
}

// ------------------------------------------------------------------ art

// The GUI draws each finished item from `sigil_seed`, so the emblem is only
// meaningful if the seed behaves the same way the generated name does: stable
// for a given build, different for a different one.

#[test]
fn an_items_emblem_seed_is_stable_for_the_same_build() {
    let mut a = Character::with_all_pieces();
    equip(&mut a, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut a, "Iron Blade", SlotKind::Weapon, 1, 0);

    let mut b = Character::with_all_pieces();
    equip(&mut b, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut b, "Iron Blade", SlotKind::Weapon, 1, 0);

    let (pa, pb) = (a.combat_items(), b.combat_items());
    assert_eq!(pa.len(), 1);
    assert_eq!(pa[0].sigil_seed, pb[0].sigil_seed);
    assert_eq!(pa[0].name, pb[0].name, "and it agrees with the name");
}

#[test]
fn moving_a_piece_redraws_the_emblem() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    let blade = piece(&ch, "Iron Blade");
    ch.equip(blade, SlotKind::Weapon, 1, 0).unwrap();
    let before = ch.combat_items()[0].sigil_seed;

    ch.equip(blade, SlotKind::Weapon, 1, 1).unwrap();
    let after = ch.combat_items()[0].sigil_seed;

    assert_ne!(before, after, "a different placement is a different item");
}

#[test]
fn different_items_get_different_emblems() {
    let mut ch = Character::with_all_pieces();
    build_full_loadout(&mut ch);

    let mut seeds = std::collections::HashSet::new();
    for p in ch.combat_items() {
        assert!(seeds.insert(p.sigil_seed), "{} reused an emblem seed", p.name);
    }
    assert!(seeds.len() >= 5, "a full loadout should assemble several items");
}

// ----------------------------------------------------------------- undo

#[test]
fn undo_puts_a_piece_back_where_it_was() {
    let mut ch = Character::with_all_pieces();
    let handle = piece(&ch, "Oak Handle");
    ch.equip(handle, SlotKind::Weapon, 0, 0).unwrap();
    ch.equip(handle, SlotKind::Weapon, 3, 2).unwrap();

    assert_eq!(ch.loadout.slot(SlotKind::Weapon).anchor_of(handle), Some((3, 2)));
    assert!(ch.undo().is_some());
    assert_eq!(
        ch.loadout.slot(SlotKind::Weapon).anchor_of(handle),
        Some((0, 0)),
        "back to where it was before the move"
    );
    assert!(ch.undo().is_some());
    assert!(!ch.is_equipped(handle), "and back off the board entirely");
    assert!(ch.undo().is_none(), "nothing left to take back");
}

#[test]
fn undo_restores_a_rotation() {
    let mut ch = Character::with_all_pieces();
    let mold = piece(&ch, "Gauntlet Mold");
    let before = ch.registry.shape(mold);

    ch.rotate(mold).unwrap();
    assert_ne!(ch.registry.shape(mold), before);

    ch.undo();
    assert_eq!(ch.registry.shape(mold), before, "rotations live on the registry too");
}

#[test]
fn a_refused_rotation_leaves_nothing_to_undo() {
    let mut ch = Character::with_all_pieces();
    let base = piece(&ch, "Padded Base"); // 4 wide x 3 tall
    ch.equip(base, SlotKind::Chest, 2, 0).unwrap();
    ch.rotate(base).expect("3x4 still fits at x=2");
    ch.equip(base, SlotKind::Chest, 3, 4).expect("3x4 fits at (3, 4)");
    // Now wedged: turning back to 4x3 would hang off the right edge.
    let depth_before = ch.undoable().map(|s| s.to_string());

    assert!(ch.rotate(base).is_err());

    assert_eq!(
        ch.undoable().map(|s| s.to_string()),
        depth_before,
        "a rotation that could not happen must not push history"
    );
}

#[test]
fn undo_takes_back_a_clear_all() {
    let mut ch = Character::with_all_pieces();
    build_full_loadout(&mut ch);
    let before: Vec<usize> =
        SlotKind::ALL.iter().map(|&k| ch.loadout.slot(k).pieces().len()).collect();
    assert!(before.iter().sum::<usize>() > 0);

    ch.clear_all();
    assert!(SlotKind::ALL.iter().all(|&k| ch.loadout.slot(k).is_empty()));

    ch.undo();
    let after: Vec<usize> =
        SlotKind::ALL.iter().map(|&k| ch.loadout.slot(k).pieces().len()).collect();
    assert_eq!(after, before, "the whole board comes back");
}





// ---------------------------------------------------------------- spells

#[test]
fn a_book_an_ink_and_a_spell_make_a_weapon() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Pocket Grimoire", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Soot Ink", SlotKind::Weapon, 1, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 2, 0);

    let report = ch.report(SlotKind::Weapon);
    assert_eq!(report.assembled_count(), 1, "{}", report.summary());
}

#[test]
fn a_martial_weapon_still_assembles_alongside_the_new_recipes() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1);
}

/// A book binds **one** spell and an orb wants two, and that is the line
/// between them.
///
/// Its name was `a_book_will_not_take_a_second_spell_but_an_orb_wants_one`,
/// which was the old rule and is the new one - what changed underneath it is
/// everything *else* the book takes. §2.2 of
/// `design/assembly-bonuses-and-books.md` relaxed the ink from required to
/// optional, allowed two of them, and added an alignment; it also drew the
/// book at one or two spells, and **the owner amended that**: breadth is the
/// ball's whole identity, and a book that could take a second spell was a ball
/// with worse breadth rather than a different thing.
///
/// So the book is depth - one payload, up to two inks multiplying it - and the
/// orb is breadth - two or three payloads, no ink, one alignment across all of
/// them. They do not overlap anywhere.
#[test]
fn a_book_binds_one_spell_and_an_orb_wants_two() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Pocket Grimoire", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Soot Ink", SlotKind::Weapon, 1, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 2, 0);
    equip(&mut ch, "Rime Nova", SlotKind::Weapon, 2, 2);
    assert_eq!(
        ch.report(SlotKind::Weapon).assembled_count(),
        0,
        "a book bound two spells, which is the ball's breadth and the only thing that \
         separates the two"
    );

    // And a book with **no ink at all**, which is the half of the relaxation
    // that makes the book reachable rather than merely bigger: before this, a
    // book without an ink was a pile.
    let mut bare = Character::with_all_pieces();
    equip(&mut bare, "Pocket Grimoire", SlotKind::Weapon, 0, 0);
    equip(&mut bare, "Emberburst", SlotKind::Weapon, 1, 0);
    let report = bare.report(SlotKind::Weapon);
    assert_eq!(report.assembled_count(), 1, "a book and a spell is a weapon: {}", report.summary());

    // Two spells around an orb are exactly what it asks for - and no ink,
    // which an orb has not wanted since alignments took over that job.
    let mut orb = Character::with_all_pieces();
    equip(&mut orb, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut orb, "Emberburst", SlotKind::Weapon, 1, 3);
    equip(&mut orb, "Rime Nova", SlotKind::Weapon, 4, 0);
    let report = orb.report(SlotKind::Weapon);
    assert_eq!(report.assembled_count(), 1, "{}", report.summary());
}

#[test]
fn ink_scales_its_own_cast_and_nobody_elses() {
    use gm2d_core::stats::Stats;
    let mut ch = Character::with_all_pieces();
    // A spell with strong ink, and a plain martial weapon beside it.
    equip(&mut ch, "Leaden Tome", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Bloodletter's Ink", SlotKind::Weapon, 3, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 3, 1);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1);

    let items = ch.combat_items();
    let spell = items.iter().find(|i| i.power_bonus > 0).expect("the spell is here");
    assert!(spell.power_bonus >= 240, "ink and book both add to it: {}", spell.power_bonus);

    // The wearer's own power is untouched by ink.
    let base = Stats::base_character().power;
    assert_eq!(ch.player_stats().power, base, "ink never reaches the wearer");
}

#[test]
fn an_orb_casts_a_different_spell_each_time() {
    use gm2d_core::combat::{simulate, Event, Side, LADDER};
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 1, 3);
    equip(&mut ch, "Rime Nova", SlotKind::Weapon, 4, 0);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1);

    let profiles = ch.combat_items();
    let orb = profiles.iter().find(|p| p.casts.len() > 1).expect("an orb holds several");
    assert_eq!(orb.casts.len(), 2);

    // Over a long fight the log should name both spells. The player is given
    // enough health to survive one, since the point is the orb's rotation.
    let mut stats = ch.player_stats();
    stats.health = 100_000;
    let log = simulate(stats, &profiles, &LADDER[LADDER.len() - 1]);
    let mut named: Vec<String> = Vec::new();
    for entry in &log.entries {
        if let Event::Activate { side: Side::Player, item, .. } = &entry.event {
            if item.contains('(') && !named.contains(item) {
                named.push(item.clone());
            }
        }
    }
    assert!(named.len() >= 2, "the orb should cycle its spells, saw {:?}", named);
}

// ---------------------------------------------------------------- quests

/// Sit a Helm of Blades on rows a bladed weapon also occupies, and fight until
/// its tally is met.
fn blades_run() -> Character {
    let mut ch = Character::with_all_pieces();
    // A weapon built with an Iron Blade, on rows 0-2.
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    // The helm, on the same rows in another slot: that is what "aligned" is.
    equip(&mut ch, "Helm of Blades", SlotKind::Helmet, 0, 0);
    equip(&mut ch, "Warding Plate", SlotKind::Helmet, 0, 2);
    ch
}







#[test]
fn the_blade_of_helms_gives_armour_where_a_damaging_piece_gives_damage() {
    use gm2d_core::piece::CATALOG;
    let d = CATALOG.iter().find(|d| d.name == "Blade of Helms").expect("it exists");
    assert_eq!(d.base.physical_damage, 0, "it is a damaging piece that deals none");
    assert!(
        d.triggers.iter().any(|t| matches!(
            t,
            gm2d_core::piece::Trigger::OnActivate(
                gm2d_core::piece::Action::GainArmor(_)
            )
        )),
        "it should hand out armour instead"
    );
}

#[test]
fn every_quest_names_a_component_that_exists() {
    use gm2d_core::piece::CATALOG;
    let mut with_quests = 0;
    for d in CATALOG {
        if let Some(q) = d.quest {
            with_quests += 1;
            assert!(
                CATALOG.iter().any(|t| t.name == q.becomes),
                "{} finishes into {}, which is not a component",
                d.name,
                q.becomes
            );
            assert!(q.goal > 0, "{} has a quest with no goal", d.name);
            assert!(
                CATALOG.iter().find(|t| t.name == q.becomes).unwrap().quest.is_none(),
                "{} finishes into something that is itself a quest, which would chain",
                d.name
            );
        }
    }
    assert!(with_quests >= 5, "only {} components carry quests", with_quests);
}

// ------------------------------------------------- shared pools and rings

#[test]
fn a_material_goes_on_a_hand_or_a_foot_alike() {
    use gm2d_core::piece::CATALOG;
    let mat = CATALOG.iter().find(|d| d.name == "Steel Material").expect("it exists");
    assert!(mat.fits(SlotKind::Gloves));
    assert!(mat.fits(SlotKind::Greaves));
    assert!(!mat.fits(SlotKind::Helmet), "but not on a head");

    // And it really places in the grid it was not declared for.
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Steel Material", SlotKind::Greaves, 0, 0);
    equip(&mut ch, "Runner's Mold", SlotKind::Greaves, 2, 0);
    assert_eq!(ch.report(SlotKind::Greaves).assembled_count(), 1);
}

#[test]
fn plating_covers_a_head_or_a_shin() {
    use gm2d_core::piece::CATALOG;
    let plate = CATALOG.iter().find(|d| d.name == "Iron Plating").expect("it exists");
    assert!(plate.fits(SlotKind::Helmet) && plate.fits(SlotKind::Greaves));
    assert!(!plate.fits(SlotKind::Chest));
}

#[test]
fn a_glove_takes_two_rings_and_a_greave_takes_none() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Steel Material", SlotKind::Gloves, 0, 0);
    equip(&mut ch, "Gauntlet Mold", SlotKind::Gloves, 2, 0);
    equip(&mut ch, "Iron Band", SlotKind::Gloves, 0, 3);
    equip(&mut ch, "Bloodring", SlotKind::Gloves, 1, 3);
    let report = ch.report(SlotKind::Gloves);
    assert_eq!(report.assembled_count(), 1, "{}", report.summary());

    // A third ring is one too many.
    let mut over = Character::with_all_pieces();
    equip(&mut over, "Steel Material", SlotKind::Gloves, 0, 0);
    equip(&mut over, "Gauntlet Mold", SlotKind::Gloves, 2, 0);
    for (i, r) in ["Iron Band", "Bloodring", "Oathring"].iter().enumerate() {
        equip(&mut over, r, SlotKind::Gloves, i as u8, 3);
    }
    assert_eq!(over.report(SlotKind::Gloves).assembled_count(), 0, "three rings is too many");

    // Rings belong on hands only.
    let ring = gm2d_core::piece::CATALOG.iter().find(|d| d.name == "Iron Band").unwrap();
    assert!(!ring.fits(SlotKind::Greaves));
}

#[test]
fn a_greave_can_take_a_plate() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Runed Material", SlotKind::Greaves, 0, 0);
    equip(&mut ch, "Runner's Mold", SlotKind::Greaves, 2, 0);
    equip(&mut ch, "Warding Plate", SlotKind::Greaves, 0, 2);
    let report = ch.report(SlotKind::Greaves);
    assert_eq!(report.assembled_count(), 1, "{}", report.summary());
}

// ------------------------------------------- spells packed against weapons

#[test]
fn a_spell_and_a_weapon_can_sit_flush_without_confusing_each_other() {
    // The whole point of the spell recipes: books and orbs anchor items of
    // their own, so a spell can be packed hard against a martial weapon and
    // neither steals the other's parts. Three damaging pieces in one grid
    // would be illegal in one weapon - split across two items it is fine.
    let mut ch = Character::with_all_pieces();
    // A martial weapon on the left.
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    // A spell immediately beside it, touching.
    equip(&mut ch, "Pocket Grimoire", SlotKind::Weapon, 2, 0);
    equip(&mut ch, "Soot Ink", SlotKind::Weapon, 3, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 3, 1);

    let report = ch.report(SlotKind::Weapon);
    assert_eq!(
        report.assembled_count(),
        2,
        "a weapon and a spell, side by side: {}",
        report.summary()
    );
}

#[test]
fn two_spell_cores_can_be_neighbours() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Pocket Grimoire", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Soot Ink", SlotKind::Weapon, 1, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 0, 2);
    equip(&mut ch, "Apprentice's Primer", SlotKind::Weapon, 3, 0);
    equip(&mut ch, "Prismatic Ink", SlotKind::Weapon, 3, 2);
    equip(&mut ch, "Rime Nova", SlotKind::Weapon, 3, 3);

    let report = ch.report(SlotKind::Weapon);
    assert_eq!(report.assembled_count(), 2, "two books, two spells: {}", report.summary());
}

#[test]
fn packing_a_spell_beside_a_weapon_beats_leaving_the_room_empty() {
    // The claim the spell system rests on: access to it lets you fit more
    // into one grid than the martial recipe alone allows.
    let mut martial = Character::with_all_pieces();
    equip(&mut martial, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut martial, "Iron Blade", SlotKind::Weapon, 1, 0);
    let alone = martial.report(SlotKind::Weapon).stats;

    let mut both = Character::with_all_pieces();
    equip(&mut both, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut both, "Iron Blade", SlotKind::Weapon, 1, 0);
    equip(&mut both, "Pocket Grimoire", SlotKind::Weapon, 2, 0);
    equip(&mut both, "Soot Ink", SlotKind::Weapon, 3, 0);
    equip(&mut both, "Emberburst", SlotKind::Weapon, 3, 1);
    let packed = both.report(SlotKind::Weapon);

    assert_eq!(packed.assembled_count(), 2);
    assert!(
        packed.stats.magic_damage > alone.magic_damage,
        "the spell should be adding a payload the weapon alone had no room for: {:?} vs {:?}",
        packed.stats.magic_damage,
        alone.magic_damage
    );
}

// -------------------------------------------------------- locked items

/// A weapon and a spell packed flush, which is exactly the case where the
/// split can be argued with.
fn packed_pair() -> Character {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    equip(&mut ch, "Pocket Grimoire", SlotKind::Weapon, 2, 0);
    equip(&mut ch, "Soot Ink", SlotKind::Weapon, 3, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 3, 1);
    ch
}

#[test]
fn locking_an_item_stops_it_negotiating_with_its_neighbours() {
    let mut ch = packed_pair();
    let handle = piece(&ch, "Oak Handle");
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 2);

    assert!(ch.toggle_lock_item(handle), "an assembled item can be locked");
    let set = ch.locked_set(handle).expect("it is locked").to_vec();
    assert!(set.contains(&piece(&ch, "Iron Blade")), "the lock holds the whole item");
    assert_eq!(set.len(), 2);

    // Still two items, and the locked one is unchanged.
    let report = ch.report(SlotKind::Weapon);
    assert_eq!(report.assembled_count(), 2, "{}", report.summary());

    assert!(!ch.toggle_lock_item(handle), "and it can be released again");
    assert!(ch.locked_set(handle).is_none());
}

#[test]
fn a_locked_item_will_not_absorb_a_piece_dropped_beside_it() {
    let mut ch = packed_pair();
    let handle = piece(&ch, "Oak Handle");
    ch.toggle_lock_item(handle);
    let locked = ch.locked_set(handle).unwrap().to_vec();

    // Drop another damaging piece against the locked weapon.
    equip(&mut ch, "Serrated Edge", SlotKind::Weapon, 0, 4);

    assert_eq!(
        ch.locked_set(handle).unwrap(),
        locked.as_slice(),
        "the locked item is exactly what it was"
    );
    let report = ch.report(SlotKind::Weapon);
    assert!(
        report.items.iter().any(|i| i.pieces == locked && i.assembled),
        "and it is still assembled: {}",
        report.summary()
    );
}

#[test]
fn a_locked_item_turns_as_one_piece() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    let handle = piece(&ch, "Oak Handle");
    let blade = piece(&ch, "Iron Blade");
    ch.toggle_lock_item(handle);

    let before = (
        ch.loadout.slot(SlotKind::Weapon).cells_of(handle).len(),
        ch.loadout.slot(SlotKind::Weapon).cells_of(blade).len(),
    );
    ch.rotate_locked(handle).expect("there is room to turn");

    // Both pieces turned, both are still on the board, and the item is intact.
    let after = (
        ch.loadout.slot(SlotKind::Weapon).cells_of(handle).len(),
        ch.loadout.slot(SlotKind::Weapon).cells_of(blade).len(),
    );
    assert_eq!(before, after, "no piece lost a cell");
    assert_eq!(ch.registry.rotation(handle), 1);
    assert_eq!(ch.registry.rotation(blade), 1);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1, "still one item");
}

#[test]
fn a_locked_item_comes_off_the_board_as_one_thing() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    let handle = piece(&ch, "Oak Handle");
    ch.toggle_lock_item(handle);

    ch.unequip_locked(handle).expect("it can come off");
    assert!(!ch.is_equipped(handle));
    assert!(!ch.is_equipped(piece(&ch, "Iron Blade")), "and so did the rest of it");

    // And the inventory carries it as a single entry.
    let groups = ch.inventory_groups();
    let together = groups.iter().find(|g| g.contains(&handle)).expect("it is in there");
    assert_eq!(together.len(), 2, "carried as one thing, not two");
}

// ---------------------------------------------------------------- recipe text

/// The split the interface draws: what makes an item work, and what only makes
/// it better. A helmet is finished with a frame and one plating - the second
/// plating and the crest are improvements to gear that already counts.
#[test]
fn a_recipe_separates_what_is_required_from_what_is_extra() {
    use gm2d_core::piece::recipe_parts;

    let helm = &recipe_parts(SlotKind::Helmet)[0];
    assert_eq!(helm.required, vec!["1 frame", "1 plating"]);
    assert_eq!(helm.optional, vec!["1 more plating", "1 crest"]);

    // "gloves mold", not "mold": greaves want a mold too and the two do not
    // interchange, so the bare word would invite exactly the wrong guess.
    let gloves = &recipe_parts(SlotKind::Gloves)[0];
    assert_eq!(gloves.required, vec!["1 material", "1 gloves mold"]);
    assert_eq!(gloves.optional, vec!["2 rings"]);

    let greaves = &recipe_parts(SlotKind::Greaves)[0];
    assert_eq!(greaves.required, vec!["1 material", "1 greaves mold"]);
}

/// A role that two slots want but whose pieces do not carry between them is
/// named for its slot; one whose pieces really are shared keeps the bare name.
#[test]
fn only_roles_that_do_not_interchange_are_qualified_by_slot() {
    use gm2d_core::piece::PieceKind;

    assert!(PieceKind::Mold.is_slot_specific(), "gloves and greaves molds do not swap");
    assert_eq!(PieceKind::Mold.name_in(SlotKind::Gloves), "gloves mold");
    assert_eq!(PieceKind::Mold.name_in(SlotKind::Greaves), "greaves mold");

    // These do swap, so qualifying them would be the lie.
    for kind in [PieceKind::Material, PieceKind::Plating] {
        assert!(!kind.is_slot_specific(), "{:?} is shared between slots", kind);
        assert_eq!(kind.name_in(SlotKind::Greaves), kind.name());
    }

    // A role only one slot uses needs no qualifying either.
    assert!(!PieceKind::Ring.is_slot_specific());
    assert!(!PieceKind::Crest.is_slot_specific());
}

/// Every way of building a slot is described separately, and each is named for
/// the piece it is built around.
#[test]
fn a_slot_with_several_recipes_describes_each_one() {
    use gm2d_core::piece::recipe_parts;

    let ways = recipe_parts(SlotKind::Weapon);
    assert_eq!(ways.len(), 6, "weapon builds six ways");
    let titles: Vec<&str> = ways.iter().map(|w| w.title).collect();
    assert_eq!(
        titles,
        vec!["Martial weapon", "Book spell", "Crystal ball", "Compass", "Atlas", "Survey golem"],
        "the weapon grid builds three weapons and three instruments"
    );

    // **The three instruments, and every bound exact.** What separates a
    // compass from an atlas from a golem is the count of shards, so there is no
    // optional half to any of them: a compass with no magnet is not a worse
    // compass, it is not a compass.
    assert_eq!(ways[3].required, vec!["1 map shard", "1 lens", "1 magnet"]);
    assert_eq!(
        ways[4].required,
        vec!["2 map shards", "1 lens", "1 crystal ball", "1 alignment"]
    );
    assert_eq!(ways[5].required, vec!["3 map shards", "2 living earths"]);
    for w in &ways[3..] {
        assert!(w.optional.is_empty(), "{}: an instrument has an optional half", w.title);
    }

    // The book, as `design/assembly-bonuses-and-books.md` §2.2 asks for it: a
    // core and something to cast, and everything else a choice. It read
    // `["1 book", "1 ink", "1 spell"]` until the recipe caught up with the
    // document, which is the line M3's own spec named as the one to re-pin.
    //
    // **This is what the `?` beside the weapon grid shows**, and it shows it
    // because it is derived - `recipe_tip` reads `recipe_parts`, which reads
    // `recipes`. Nothing in the interface had to be told.
    assert_eq!(ways[1].required, vec!["1 book", "1 spell"]);
    assert_eq!(
        ways[1].optional,
        vec!["2 inks", "1 alignment", "1 accessory"],
        "the book's optional half is what a player reads off the ? beside the weapon - and \
         a second spell is not in it, because breadth is the ball's"
    );

    // And the orb is untouched beside it, which is what makes the two
    // identities separate rather than the book simply becoming an orb.
    assert_eq!(ways[2].required, vec!["1 crystal ball", "2 spells"]);
    assert_eq!(ways[2].optional, vec!["1 more spell", "1 alignment"]);
}

/// The required half is exactly what the assembly rule enforces. If a recipe's
/// minimums change, the text changes with them rather than going stale.
#[test]
fn the_required_half_matches_the_recipe_minimums() {
    use gm2d_core::piece::{recipe_parts, recipes};

    for slot in SlotKind::ALL {
        for (r, text) in recipes(slot).iter().zip(recipe_parts(slot)) {
            let wanted = r.iter().filter(|(_, min, _)| *min > 0).count();
            assert_eq!(
                text.required.len(),
                wanted,
                "{:?}: {} required entries for {} non-zero minimums",
                slot,
                text.required.len(),
                wanted
            );
        }
    }
}

/// Counts read as English on both sides of the split.
#[test]
fn recipe_counts_are_pluralised() {
    use gm2d_core::piece::recipe_parts;

    let all: Vec<String> = SlotKind::ALL
        .iter()
        .flat_map(|&s| recipe_parts(s))
        .flat_map(|p| p.required.into_iter().chain(p.optional))
        .collect();

    assert!(all.iter().any(|s| s == "2 rings"), "plural noun: {:?}", all);
    assert!(all.iter().any(|s| s == "2 more layers"), "plural after 'more': {:?}", all);
    // Mass nouns stay singular however many there are.
    assert!(all.iter().any(|s| s == "1 more damaging"), "{:?}", all);
    assert!(!all.iter().any(|s| s.contains("platings")), "no 'platings': {:?}", all);
    // "accessory" -> "accessories", not "accessorys".
    assert!(all.iter().any(|s| s == "2 accessories"), "{:?}", all);
    assert!(!all.iter().any(|s| s.contains("accessorys")), "{:?}", all);
}

/// The bug: a locked item's pieces could still be dragged out one at a time,
/// which is exactly what locking is supposed to prevent. The interface now
/// lifts the whole set, and the shape it needs to do that lives on the lock.
#[test]
fn a_locked_item_knows_its_own_shape() {
    let mut ch = packed_pair();
    let handle = piece(&ch, "Oak Handle");
    let blade = piece(&ch, "Iron Blade");
    ch.toggle_lock_item(handle);

    let shape = ch.locked_shape(handle).expect("a locked item carries its shape");
    assert_eq!(shape.len(), 2, "both pieces are in it");
    // Oak Handle at (0,0) and Iron Blade at (1,0): offsets relative to the
    // item's own corner, so they survive being put down somewhere else.
    let at = |id| shape.iter().find(|&&(p, ..)| p == id).map(|&(_, x, y)| (x, y));
    assert_eq!(at(handle), Some((0, 0)));
    assert_eq!(at(blade), Some((1, 0)));

    // Any piece of it answers with the same shape - you can grab it anywhere.
    assert_eq!(ch.locked_shape(blade), ch.locked_shape(handle));
}

/// The other bug: a whole assembled item could not be moved to the inventory
/// and back. It comes off as one thing and goes down as one thing.
#[test]
fn a_locked_item_travels_to_the_inventory_and_back_whole() {
    let mut ch = packed_pair();
    let handle = piece(&ch, "Oak Handle");
    let blade = piece(&ch, "Iron Blade");
    ch.toggle_lock_item(handle);

    ch.unequip_locked(handle).expect("it comes off as one piece");
    assert!(ch.loadout.slot(SlotKind::Weapon).anchor_of(handle).is_none());
    assert!(ch.loadout.slot(SlotKind::Weapon).anchor_of(blade).is_none());
    assert!(ch.is_locked_item(handle), "stowing it does not release it");

    // The inventory carries it as one entry, not two.
    let groups = ch.inventory_groups();
    let together = groups.iter().find(|g| g.contains(&handle)).expect("it is in the tray");
    assert_eq!(together.len(), 2, "carried as one thing");

    // And it goes back down somewhere else with its shape intact.
    ch.equip_locked_at(handle, SlotKind::Weapon, 2, 3).expect("it fits there");
    assert_eq!(ch.loadout.slot(SlotKind::Weapon).anchor_of(handle), Some((2, 3)));
    assert_eq!(ch.loadout.slot(SlotKind::Weapon).anchor_of(blade), Some((3, 3)));
    assert!(ch.report(SlotKind::Weapon).items.iter().any(|i| i.assembled));
}

/// A drop that will not fit must leave the board alone rather than strewing
/// half an item across it.
#[test]
fn a_locked_item_that_does_not_fit_is_refused_whole() {
    let mut ch = packed_pair();
    let handle = piece(&ch, "Oak Handle");
    let blade = piece(&ch, "Iron Blade");
    ch.toggle_lock_item(handle);
    ch.unequip_locked(handle).unwrap();

    // Hard against the right edge: the handle would fit, the blade would not.
    let w = gm2d_core::slot::SLOT_W - 1;
    assert!(ch.equip_locked_at(handle, SlotKind::Weapon, w, 0).is_err());
    assert!(
        ch.loadout.slot(SlotKind::Weapon).anchor_of(handle).is_none(),
        "a refused drop places nothing at all"
    );
    assert!(ch.loadout.slot(SlotKind::Weapon).anchor_of(blade).is_none());
}

/// Turning a locked item changes its shape, and the stored shape has to follow
/// or it would go back down in its old arrangement.
#[test]
fn turning_a_locked_item_updates_the_shape_it_travels_with() {
    // Not `packed_pair`: the spell there sits flush against the weapon, so
    // there is genuinely nowhere for it to turn into.
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Oak Handle", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Iron Blade", SlotKind::Weapon, 1, 0);
    let handle = piece(&ch, "Oak Handle");
    ch.toggle_lock_item(handle);

    let before = ch.locked_shape(handle).unwrap();
    ch.rotate_locked(handle).expect("there is room to turn");
    let after = ch.locked_shape(handle).unwrap();
    assert_ne!(before, after, "the stored shape turned with the item");

    // And it still goes back down as the shape it is now.
    ch.unequip_locked(handle).unwrap();
    ch.equip_locked_at(handle, SlotKind::Weapon, 0, 0).expect("it fits");
    let slot = ch.loadout.slot(SlotKind::Weapon);
    for &(p, dx, dy) in &after {
        assert_eq!(slot.anchor_of(p), Some((dx, dy)), "piece landed where the shape says");
    }
}



// ------------------------------------------------------- orbs and alignments

/// An orb takes no ink any more: two spells are the whole requirement, and an
/// alignment is the optional third part.
#[test]
fn an_orb_assembles_from_spells_alone_and_takes_an_alignment() {
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Emberburst", SlotKind::Weapon, 1, 3);
    equip(&mut ch, "Rime Nova", SlotKind::Weapon, 4, 0);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1, "two spells is enough");

    // And the alignment joins without breaking it.
    equip(&mut ch, "Azure Alignment", SlotKind::Weapon, 4, 2);
    let r = ch.report(SlotKind::Weapon);
    assert_eq!(r.assembled_count(), 1, "{}", r.summary());
}

/// An alignment is never cast itself - it colours every spell the ball holds.
#[test]
fn an_alignment_colours_every_spell_in_the_ball() {
    let mut plain = Character::with_all_pieces();
    equip(&mut plain, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut plain, "Emberburst", SlotKind::Weapon, 1, 3);
    equip(&mut plain, "Rime Nova", SlotKind::Weapon, 4, 0);
    let before = plain.combat_items();
    let before = before.iter().find(|i| i.casts.len() > 1).expect("an orb");
    assert_eq!(before.casts.len(), 2, "the alignment is not itself a cast");
    let mana_before: Vec<i32> = before.casts.iter().map(|c| c.stats.mana).collect();

    let mut aligned = Character::with_all_pieces();
    equip(&mut aligned, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut aligned, "Emberburst", SlotKind::Weapon, 1, 3);
    equip(&mut aligned, "Rime Nova", SlotKind::Weapon, 4, 0);
    equip(&mut aligned, "Azure Alignment", SlotKind::Weapon, 4, 2);
    let after = aligned.combat_items();
    let after = after.iter().find(|i| i.casts.len() > 1).expect("an orb");
    assert_eq!(after.casts.len(), 2, "still two casts, not three");

    // Every spell gained, and gained the same - which is the point of an
    // alignment. Not by exactly two: the alignment also carries power, and an
    // item's power multiplies its own numbers, so what reaches each cast is
    // the alignment's mana with the ball's multiplier already on it.
    let gained: Vec<i32> =
        after.casts.iter().enumerate().map(|(i, c)| c.stats.mana - mana_before[i]).collect();
    assert!(gained.iter().all(|g| *g > 0), "no spell gained anything: {:?}", gained);
    assert!(
        gained.windows(2).all(|w| w[0] == w[1]),
        "the alignment reached one spell and not the others: {:?}",
        gained
    );
}

/// A spell that answers its siblings pays out when one of them is cast, which
/// is what a ball can do and a book cannot.
#[test]
fn a_spell_answers_its_siblings_going_off() {
    use gm2d_core::piece::{Action, Trigger};
    let mut ch = Character::with_all_pieces();
    equip(&mut ch, "Scrying Orb", SlotKind::Weapon, 0, 0);
    equip(&mut ch, "Echo Sigil", SlotKind::Weapon, 1, 3);
    equip(&mut ch, "Rime Nova", SlotKind::Weapon, 4, 0);
    assert_eq!(ch.report(SlotKind::Weapon).assembled_count(), 1);

    let items = ch.combat_items();
    let orb = items.iter().find(|i| i.casts.len() > 1).expect("an orb");
    let echo = orb
        .casts
        .iter()
        .find(|c| c.name == "Echo Sigil")
        .expect("the sigil is one of its casts");
    assert!(
        echo.triggers.iter().any(|t| matches!(t, Trigger::OnOtherCast(Action::GainMana(_)))),
        "it carries the answering trigger: {:?}",
        echo.triggers
    );
}

/// Every spell that answers a sibling is worth having: a book holds one spell,
/// so the trigger would be dead weight there. This pins that they exist and
/// that the catalogue offers enough of them to build a ball around.
#[test]
fn there_are_spells_that_answer_other_spells() {
    use gm2d_core::piece::{PieceKind, Trigger, CATALOG};
    let answering = CATALOG
        .iter()
        .filter(|d| d.kind == PieceKind::Spell)
        .filter(|d| d.triggers.iter().any(|t| matches!(t, Trigger::OnOtherCast(_))))
        .count();
    assert!(answering >= 5, "only {} spells answer their siblings", answering);
}

// ------------------------------------------------- actions that make sense

/// No action in the catalogue is written in a shape that has no reading.
///
/// Two rules, and only one of them needs a test. `Accrue` on a **fused** pool
/// is representable and wrong: a fusion is deliberately fuel for nothing
/// (`piece::Resource`), so a proportional income on one would be a second
/// currency at better rates than the first. `combat::apply` refuses it at
/// runtime as well, because a rule only a lint enforces is a rule a
/// hand-built `ItemProfile` walks straight through.
///
/// The other rule is not here because it cannot be broken. The spec asks this
/// test to refuse `Derail { target: Yourself }` - there being no reading of
/// setting your own best item back that is not a stun you paid for - and
/// `Action::Derail` carries no target at all. Making it unrepresentable beats
/// linting it: a lint that can only ever pass is a type that should have said
/// so, which is `CLAUDE.md` §6 trap 22 read from the other end.
#[test]
fn every_action_is_well_formed() {
    use gm2d_core::piece::{walk_actions, Action, CATALOG};

    let mut bad: Vec<String> = Vec::new();
    for d in CATALOG {
        for t in d.triggers {
            walk_actions(t, &mut |a| {
                if let Action::Accrue { what, pct } = a {
                    if what.is_fused() {
                        bad.push(format!("{} accrues {}, which is a fusion", d.name, what.name()));
                    }
                    if *pct <= 0 {
                        bad.push(format!("{} accrues {}% of a pool", d.name, pct));
                    }
                }
                if let Action::Shunt { ms } = a {
                    if *ms == 0 {
                        bad.push(format!("{} shunts nothing", d.name));
                    }
                }
                if let Action::Derail { window_ms, back_ms } = a {
                    if *window_ms == 0 || *back_ms == 0 {
                        bad.push(format!("{} derails nothing", d.name));
                    }
                }
                if let Action::Ballast(n) = a {
                    if *n <= 0 {
                        bad.push(format!("{} ballasts {}", d.name, n));
                    }
                }
            });
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
