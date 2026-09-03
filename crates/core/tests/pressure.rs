//! The measure, before anything tries to move it.
//!
//! M12.0. These are about `pressure::of` being right, not about the game
//! meeting the curve — it does not, which is why there is a block.

mod common;

use gm2d_core::character::Character;
use gm2d_core::piece::SlotKind;
use gm2d_core::pressure;
use gm2d_core::slot::SLOT_W;

#[test]
fn an_empty_board_is_empty_and_a_seated_one_is_not() {
    let mut ch = Character::starting();
    let empty = pressure::of(&ch);
    assert_eq!(empty.used, 0, "nothing is seated");
    assert_eq!(empty.pct(), 0);
    // Every grid is counted, not just the ones with something on them.
    assert_eq!(empty.slots.len(), SlotKind::ALL.len());
    let cells: u32 = SlotKind::ALL
        .iter()
        .map(|&k| ch.loadout.slot(k).rows() as u32 * SLOT_W as u32)
        .sum();
    assert_eq!(empty.total, cells, "the whole canvas, not one grid's");

    ch.apply_preset();
    let packed = pressure::of(&ch);
    assert!(packed.used > 0, "the preset seats something");
    assert!(packed.pct() > empty.pct(), "a packed board reads fuller than an empty one");
}

#[test]
fn fill_counts_cells_and_not_components() {
    // A blade is four cells and a handle is one. Two components, five cells —
    // and the whole difference between a packed board and a tidy one is that
    // this number is the first and not the second.
    let mut ch = common::bench();
    ch.grow_boards(20);
    let before = pressure::of(&ch);
    common::seat(&mut ch, &[("Iron Blade", SlotKind::Weapon, 0, 0, 1)]);
    let after = pressure::of(&ch);
    let blade = gm2d_core::piece::CATALOG
        .iter()
        .find(|d| d.name == "Iron Blade")
        .expect("the starting blade");
    assert_eq!(
        after.used - before.used,
        blade.cells.len() as u32,
        "one component moved the count by its own footprint"
    );
    assert!(blade.cells.len() > 1, "and the footprint is not one cell, or this proves nothing");
}

#[test]
fn the_weapon_grid_is_reported_apart_from_the_others() {
    let mut ch = common::bench();
    ch.grow_boards(20);
    common::seat(&mut ch, &[("Iron Blade", SlotKind::Weapon, 0, 0, 1)]);
    let p = pressure::of(&ch);
    let weapon = p.slots.iter().find(|s| s.slot == SlotKind::Weapon).expect("a weapon row");
    let chest = p.slots.iter().find(|s| s.slot == SlotKind::Chest).expect("a chest row");
    assert!(weapon.used > 0, "the blade is in the weapon grid");
    assert_eq!(chest.used, 0, "and nowhere else");
}

#[test]
fn a_piece_that_only_fits_turned_is_not_on_the_bench() {
    // **The whole reason `fits_anywhere` tries four turns.** The Iron Blade is
    // one cell wide and four tall; a starting weapon frame is three rows. Read
    // at the rotation it happens to be wearing it fits nowhere, and a player
    // seats it on their first afternoon by turning it — which is why the
    // starting kit ships it turned and why the M4 soft-lock happened when it
    // did not.
    let ch = Character::starting();
    let mut fresh = Character::starting();
    // Take the seated kit off so the grids are empty and only geometry decides.
    for id in fresh.owned.clone() {
        let _ = fresh.unequip(id);
    }
    let blade = common::piece(&fresh, "Iron Blade");
    fresh.registry.set_rotation(blade, 0); // upright: 1 wide, 4 tall
    let upright = fresh.registry.shape(blade);
    assert!(
        upright.height() as usize > fresh.loadout.slot(SlotKind::Weapon).rows() as usize,
        "upright it is taller than the frame, or this test is about nothing"
    );
    assert!(fresh.fits_anywhere(blade), "turned, it fits — so it is not benched");
    let _ = ch;
}

/// Seat everything that will go into one grid, and stop when nothing more will.
///
/// **The bench is about the board as it is, not about whether everything could
/// fit at once.** On an empty board almost nothing is benched, because almost
/// everything has somewhere to go — which is the whole complaint the block
/// starts from. A component is on the bench when the room ran out.
fn cram(ch: &mut Character, kind: SlotKind) {
    loop {
        let mut seated = false;
        for id in ch.inventory() {
            if ch.registry.def(id).slot != kind {
                continue;
            }
            'place: for y in 0..ch.loadout.slot(kind).rows() {
                for x in 0..SLOT_W {
                    if ch.equip(id, kind, x, y).is_ok() {
                        seated = true;
                        break 'place;
                    }
                }
            }
        }
        if !seated {
            return;
        }
    }
}

#[test]
fn a_piece_with_no_room_is_on_the_bench() {
    let mut ch = common::bench();
    // An empty board benches nothing, because everything has somewhere to go.
    assert_eq!(pressure::of(&ch).bench, 0, "an empty board is all room");

    cram(&mut ch, SlotKind::Chest);
    let full = pressure::of(&ch);
    let chest = full.slots.iter().find(|s| s.slot == SlotKind::Chest).expect("a chest row");
    assert!(chest.pct() > 50, "the grid is packed, not sparse: {}%", chest.pct());
    assert!(
        full.bench > 0,
        "one of every component and a full chest grid leaves something with nowhere to go"
    );

    // And every one of them is a chest component: nothing else lost a home.
    for id in ch.inventory() {
        if !ch.fits_anywhere(id) && ch.registry.def(id).kind != gm2d_core::piece::PieceKind::Quest {
            assert_eq!(
                ch.registry.def(id).slot,
                SlotKind::Chest,
                "{} is benched and it is not a chest piece",
                ch.registry.def(id).name
            );
        }
    }
}

#[test]
fn a_quest_item_is_carried_and_never_counted_as_waiting_for_a_cell() {
    // A tally of toad eyes and a spent key are not components looking for
    // room. `can_equip` refuses `PieceKind::Quest` outright, so without this
    // they would sit in the bench number for ever and make the one metric the
    // block stakes a decision on go up for the wrong reason.
    let mut ch = Character::starting();
    ch.grow_boards(20);
    let token = gm2d_core::piece::CATALOG
        .iter()
        .find(|d| d.kind == gm2d_core::piece::PieceKind::Quest)
        .expect("the catalogue has quest items");
    let before = pressure::of(&ch).bench;
    ch.give(token.name);
    let after = pressure::of(&ch).bench;
    assert_eq!(before, after, "{} is carried, not benched", token.name);
    // And it is genuinely unseatable, or the exclusion is doing nothing.
    let id = common::piece(&ch, token.name);
    assert!(
        ch.can_equip(id, token.slot, 0, 0).is_err(),
        "a quest item cannot be worn, which is why it must not be counted"
    );
}

#[test]
fn asking_about_a_turn_is_not_a_way_to_seat_a_piece_in_one() {
    // **The hazard `can_equip_shape` introduces, tested where it lives.**
    // Asking "would this fit if it were turned" has to be a question. If
    // `equip` ever answered it with the shape it was asked about rather than
    // the shape the piece is wearing, a player would seat a blade sideways by
    // hovering it, and the board would hold a footprint the registry does not
    // agree with.
    let mut ch = Character::starting();
    for id in ch.owned.clone() {
        let _ = ch.unequip(id);
    }
    let blade = common::piece(&ch, "Iron Blade");
    ch.registry.set_rotation(blade, 0); // 1 wide, 4 tall, on a 3-row frame
    let turned = ch.registry.shape(blade).rotated(1); // 4 wide, 1 tall

    assert!(
        ch.can_equip_shape(blade, SlotKind::Weapon, &turned, 0, 0).is_ok(),
        "turned, it would fit at the corner"
    );
    assert!(
        ch.can_equip(blade, SlotKind::Weapon, 0, 0).is_err(),
        "as it is, it does not — and `can_equip` is the one `equip` asks"
    );
    assert!(
        ch.equip(blade, SlotKind::Weapon, 0, 0).is_err(),
        "so seating it is refused, whatever some other shape would have done"
    );
    assert!(ch.loadout.slot(SlotKind::Weapon).pieces().is_empty(), "and nothing was seated");
}

#[test]
fn the_target_curve_is_written_down_and_reads_as_a_floor() {
    // Not a gate — today's game does not meet it, and that is the premise of
    // the block. What it has to be is a number somebody wrote down first, so
    // the close-out can be a diff.
    assert_eq!(pressure::target::fill_at(1), None, "the curve has not started at level one");
    assert_eq!(pressure::target::fill_at(3), Some(70));
    assert_eq!(pressure::target::fill_at(5), Some(70), "a floor holds until the next one");
    assert_eq!(pressure::target::fill_at(6), Some(80));
    assert_eq!(pressure::target::fill_at(20), Some(80));
    assert_eq!(pressure::target::bench_at(4), None);
    assert_eq!(pressure::target::bench_at(5), Some(2));
}
