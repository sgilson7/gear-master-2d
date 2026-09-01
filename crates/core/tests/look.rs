//! The board is legible without colour, and these are what say so.
//!
//! Seven tests, ported near-verbatim from `sgilson7/gear-master`'s `crates/gui`.
//! They are what stops the three-channel system decaying back into decoration:
//! every one of them fails loudly the moment somebody picks a colour by eye.
//!
//! The design they hold: **slot → a motif and a hue, role → brightness.** Any
//! two of the three channels can be lost and the board still answers "which
//! grid is this" and "which part of the recipe is this".

use gm2d_core::look::{
    self, hex, kind_luminance, luminance, motif, motif_ink, slot_color, unplaced_color, Motif,
    INK_SEPARATION, ROLE_SEPARATION, ROLE_STEPS,
};
use gm2d_core::piece::{PieceKind, SlotKind, CATALOG};

/// Every slot has its own mark.
#[test]
fn every_slot_has_its_own_motif() {
    let motifs: Vec<Motif> = SlotKind::ALL.iter().map(|&s| motif(s)).collect();
    for (i, a) in motifs.iter().enumerate() {
        for b in &motifs[i + 1..] {
            assert_ne!(a, b, "two slots share the motif {a:?}");
        }
    }
    assert_eq!(motifs.len(), 5);
}

/// The shared diamond is not one of the five.
#[test]
fn the_shared_mark_is_not_one_of_the_slot_marks() {
    for slot in SlotKind::ALL {
        assert_ne!(motif(slot), Motif::Shared);
    }
}

/// **The three roles stay apart in greyscale, in every hue.**
///
/// The load-bearing test. The role is only legible without colour if the steps
/// survive being flattened to brightness — and they only survive in *every*
/// hue because `slot_color` bisects for a brightness target rather than picking
/// a lightness. Pick three lightnesses by eye and this fails in yellow first.
#[test]
fn the_three_roles_stay_apart_in_greyscale() {
    for slot in SlotKind::ALL {
        let lums: Vec<f32> =
            ROLE_STEPS.iter().map(|&k| luminance(slot_color(slot, kind_luminance(k)))).collect();
        for w in lums.windows(2) {
            assert!(
                w[1] - w[0] > ROLE_SEPARATION,
                "{slot:?}: roles only {:.3} apart in brightness ({lums:?})",
                w[1] - w[0]
            );
        }
    }
}

/// And the grey ramp a shared piece wears keeps the same separation.
#[test]
fn the_shared_grey_keeps_the_roles_apart_in_greyscale() {
    let lums: Vec<f32> = ROLE_STEPS.iter().map(|&k| luminance(unplaced_color(k))).collect();
    for w in lums.windows(2) {
        assert!(
            w[1] - w[0] > ROLE_SEPARATION,
            "shared greys only {:.3} apart ({lums:?})",
            w[1] - w[0]
        );
    }
}

/// A shared component is colourless until it is placed.
#[test]
fn a_shared_piece_is_colourless_until_it_is_placed() {
    let shared: Vec<_> = CATALOG.iter().filter(|d| d.shared()).collect();
    assert!(!shared.is_empty(), "no shared components to check");

    for def in &shared {
        let loose = look::look(def, None);
        assert_eq!(loose.motif, Motif::Shared, "{} loose should wear the shared mark", def.name);
        let [r, g, b] = loose.fill;
        assert!(
            (r - g).abs() < 0.001 && (g - b).abs() < 0.001,
            "{} loose should be grey, got {:?}",
            def.name,
            loose.fill
        );

        for slot in def.slots() {
            let placed = look::look(def, Some(slot));
            assert_eq!(placed.motif, motif(slot), "{} in {slot:?}", def.name);
            assert_eq!(
                placed.fill,
                slot_color(slot, kind_luminance(def.kind)),
                "{}",
                def.name
            );
        }
    }
}

/// Only ambiguity gets greyed out. Anything else would be losing information
/// for no reason.
#[test]
fn a_piece_that_goes_one_place_looks_the_same_loose_or_placed() {
    for def in CATALOG.iter().filter(|d| !d.shared()) {
        assert_eq!(
            look::look(def, None),
            look::look(def, Some(def.slot)),
            "{}",
            def.name
        );
    }
}

/// The motif's ink shows up on every tile it can land on.
#[test]
fn the_motif_ink_contrasts_with_every_tile_it_lands_on() {
    for slot in SlotKind::ALL {
        for &kind in &ROLE_STEPS {
            let fill = slot_color(slot, kind_luminance(kind));
            let (ink, alpha) = motif_ink(fill);
            // The ink is drawn over the fill, so what reaches the eye is the
            // two composited by the ink's own alpha.
            let mixed = [
                fill[0] + (ink[0] - fill[0]) * alpha,
                fill[1] + (ink[1] - fill[1]) * alpha,
                fill[2] + (ink[2] - fill[2]) * alpha,
            ];
            let gap = (luminance(mixed) - luminance(fill)).abs();
            assert!(
                gap > INK_SEPARATION,
                "{slot:?}/{kind:?}: motif only {gap:.3} from its tile"
            );
        }
    }
}

// ------------------------------------------------------------- sanity

/// Every kind in the catalogue has a role, and the enchantment layer is
/// lighter than anything that stands on it.
#[test]
fn every_kind_has_a_place_on_the_scale() {
    let mut seen = std::collections::BTreeSet::new();
    for d in CATALOG {
        seen.insert(format!("{:?}", d.kind));
        let l = kind_luminance(d.kind);
        assert!((0.0..=1.0).contains(&l), "{:?} rates {l}", d.kind);
    }
    assert!(seen.len() >= 10, "only {} kinds in the catalogue?", seen.len());

    let ground = kind_luminance(PieceKind::Enchantment);
    for &k in &ROLE_STEPS {
        assert!(ground > kind_luminance(k), "ground is darker than {k:?}, so gear sinks into it");
    }
}

/// The bisection actually hits the target it was given.
#[test]
fn the_bisection_lands_on_its_brightness_target() {
    for slot in SlotKind::ALL {
        for target in [0.22, 0.45, 0.72, 0.85] {
            let got = luminance(slot_color(slot, target));
            assert!(
                (got - target).abs() < 0.01,
                "{slot:?} at {target}: landed on {got:.3}"
            );
        }
    }
}

/// Hex output is well formed, because it crosses the boundary as a string.
#[test]
fn hexes_are_well_formed() {
    for slot in SlotKind::ALL {
        let h = hex(slot_color(slot, 0.45));
        assert_eq!(h.len(), 7, "{h}");
        assert!(h.starts_with('#'));
        assert!(h[1..].chars().all(|c| c.is_ascii_hexdigit()), "{h}");
    }
}
