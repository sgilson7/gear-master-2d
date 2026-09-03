//! Board pressure, as a number.
//!
//! **M12's thesis is that cells outnumber pieces**, so a board reads as
//! inventory space rather than as a puzzle and nothing you pick up ever costs
//! you something you already had. The block opens by measuring that and closes
//! by checking the measure moved, which means the measure has to exist before
//! anything tries to move it. This is it.
//!
//! Two numbers, and they are a pair rather than a list:
//!
//! - **fill** — how much of the board is under something. High fill on its own
//!   is a full board, which could just as easily be a board that has never
//!   grown.
//! - **bench** — owned components that fit nowhere at all. A nonempty bench on
//!   its own is a bag of things in the wrong slot.
//!
//! Tension is both at once: no room, *and* something worth making room for.
//! Neither number says that by itself, which is why nothing here reduces them
//! to one score — a single figure would hide exactly the case the block is
//! trying to produce.
//!
//! **It is core's and not the walker's.** `testing/playthrough.py` prints
//! these; it does not work them out. A number a design stakes itself on —
//! `PLAN-M12.md` §3 M12.3 stakes one on bench depth — computed by the thing
//! measuring it is the page recomputing a total, one level up.

use crate::character::Character;
use crate::piece::SlotKind;
use crate::slot::SLOT_W;

/// One grid's share of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotFill {
    pub slot: SlotKind,
    /// Cells with something in them.
    pub used: u32,
    /// Cells there are. `rows * SLOT_W`, and it grows.
    pub total: u32,
}

impl SlotFill {
    pub fn pct(&self) -> u32 {
        pct(self.used, self.total)
    }
}

/// What the boards look like right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pressure {
    pub slots: Vec<SlotFill>,
    pub used: u32,
    pub total: u32,
    /// Owned components that fit nowhere, at any turn, in any grid.
    pub bench: u32,
}

impl Pressure {
    pub fn pct(&self) -> u32 {
        pct(self.used, self.total)
    }
}

/// Integer percent, and a board with no cells is empty rather than a panic.
///
/// Rounded down, so "80%" means *at least* eighty and the target curve reads
/// the way it is written. Integer for the reason every roll in this game is
/// integer per-mille: two machines must agree on whether a gate was met.
fn pct(used: u32, total: u32) -> u32 {
    if total == 0 {
        0
    } else {
        used * 100 / total
    }
}

/// Measure a character's boards.
///
/// **Cells, not components.** A four-cell blade fills four cells and a ring
/// fills one, and fill is about the room left rather than about how many
/// things are on the board — which is the whole difference between a packed
/// board and a tidy one.
///
/// **An enchantment lies under the grid** and is not counted: it does not take
/// a cell away from anything, gear sits on top of it, and counting it would
/// report a board as full that has every cell still free.
pub fn of(c: &Character) -> Pressure {
    let mut slots = Vec::with_capacity(SlotKind::ALL.len());
    let mut used = 0;
    let mut total = 0;
    for &kind in SlotKind::ALL.iter() {
        let slot = c.loadout.slot(kind);
        let mut u = 0;
        for y in 0..slot.rows() {
            for x in 0..SLOT_W {
                if slot.get(x, y).is_some() {
                    u += 1;
                }
            }
        }
        let t = slot.rows() as u32 * SLOT_W as u32;
        slots.push(SlotFill { slot: kind, used: u, total: t });
        used += u;
        total += t;
    }
    // **A quest item is carried, never worn, and so it is never waiting for a
    // cell.** `can_equip` refuses `PieceKind::Quest` outright, so a tally of
    // toad eyes, a key that has been spent and one that has not would all sit
    // in this number for ever — and it would go *up* every time an errand was
    // taken, which is the one metric this block stakes a design decision on
    // moving for the right reason.
    let bench = c
        .inventory()
        .into_iter()
        .filter(|&id| c.registry.def(id).kind != crate::piece::PieceKind::Quest)
        .filter(|&id| !c.fits_anywhere(id))
        .count() as u32;
    Pressure { slots, used, total, bench }
}

/// The curve this block is trying to produce, and it is numbers because
/// "feels tense" is not a gate.
///
/// `PLAN-M12.md` §3 M12.0 writes them down and the human may tune them at
/// sign-off. They are **targets and not assertions**: today's game does not
/// meet them — that is the entire premise of the block — so nothing here fails
/// a build. What they are for is the close-out, where a claim of improvement
/// has to be a diff against a number somebody wrote down first.
pub mod target {
    /// `(level, overall fill percent)`, each a floor from that level on.
    pub const FILL: &[(u32, u32)] = &[(3, 70), (6, 80)];
    /// `(level, bench depth)` — something worth making room for.
    pub const BENCH: &[(u32, u32)] = &[(5, 2)];

    /// What the curve wants at this level, or nothing if it has not started.
    pub fn fill_at(level: u32) -> Option<u32> {
        FILL.iter().filter(|(l, _)| level >= *l).map(|(_, p)| *p).max()
    }

    pub fn bench_at(level: u32) -> Option<u32> {
        BENCH.iter().filter(|(l, _)| level >= *l).map(|(_, n)| *n).max()
    }
}
