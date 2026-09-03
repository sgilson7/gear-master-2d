//! How a component reads, before anything draws it.
//!
//! # Why this is in core
//!
//! `crates/core` is graphics-free and stays that way: nothing here imports a
//! rendering library, touches a canvas, or knows what a pixel is. What is here
//! is a *classification* — which grid a component belongs to, which part of a
//! recipe it is — expressed as numbers a renderer can use. That is core's
//! business, and putting it here has one concrete payoff: **the accessibility
//! contract is enforced by `cargo test`** rather than by somebody looking at a
//! screenshot.
//!
//! Lifted from `sgilson7/gear-master`'s `crates/gui`, whose own comment states
//! the whole design:
//!
//! > A piece's tile has to answer two questions — which slot does this belong
//! > to, and which part of the recipe is it — and it has to answer both without
//! > relying on colour, because some players see none of it.
//!
//! Three channels, and any two of them can be lost:
//!
//! | channel | carries | survives |
//! |---|---|---|
//! | [`motif`] — a mark stamped on every cell | the slot | no colour at all |
//! | [`kind_luminance`] — brightness | the role | no colour at all |
//! | [`slot_hue`] — an Okabe-Ito hue | the slot again | for those who see it |
//!
//! # Brightness, not lightness
//!
//! The role steps are stated as *perceived brightness* and reached by
//! bisection, not as HSL lightness. The same lightness lands at wildly
//! different brightness depending on the hue — yellow in particular flattens
//! its top two steps into one — so picking three lightnesses by eye produces a
//! scale that reads in three hues and collapses in the other two.
//!
//! `tests/look.rs` holds every claim above, including the one number that makes
//! them true: **0.08 luminance between consecutive role steps, in every hue.**

use crate::piece::{PieceDef, PieceKind, SlotKind};

// ------------------------------------------------------------------ the hues

/// Hue per slot, from the Okabe-Ito colour-blind-safe palette: vermillion, sky
/// blue, bluish green, reddish purple, yellow.
///
/// Okabe-Ito rather than an even spread around the wheel, because evenly spaced
/// hues collapse into pairs under red-green colour blindness.
pub fn slot_hue(slot: SlotKind) -> f32 {
    match slot {
        SlotKind::Weapon => 0.073,
        SlotKind::Helmet => 0.552,
        SlotKind::Chest => 0.443,
        SlotKind::Gloves => 0.912,
        SlotKind::Greaves => 0.156,
    }
}

/// Saturation per slot.
///
/// Okabe-Ito's colours are not equally saturated, and evening them out is what
/// pushes the two blues together and the two warms together.
pub fn slot_sat(slot: SlotKind) -> f32 {
    match slot {
        SlotKind::Weapon => 0.80,
        SlotKind::Helmet => 0.68,
        SlotKind::Chest => 0.72,
        SlotKind::Gloves => 0.44,
        SlotKind::Greaves => 0.74,
    }
}

// ------------------------------------------------------------------ the roles

/// Brightness per role, so the piece a recipe is built around reads darkest.
///
/// This is the channel that carries the role once colour is gone.
pub fn kind_luminance(kind: PieceKind) -> f32 {
    match kind {
        // Cores darkest. A book or an orb anchors a spell exactly as a handle
        // anchors a weapon, so it reads at the same brightness.
        PieceKind::Handle
        | PieceKind::Frame
        | PieceKind::Base
        | PieceKind::Material
        | PieceKind::Book
        | PieceKind::Orb
        // And a map shard anchors an instrument. Same role, same brightness:
        // the channel carries *what a piece is for in its recipe*, and a core
        // is a core whichever of the four things the weapon grid is building.
        | PieceKind::Shard => 0.22,
        // The body of the recipe.
        PieceKind::Damaging
        | PieceKind::Plating
        | PieceKind::Layer
        | PieceKind::Mold
        | PieceKind::Ink
        // The instruments' supporters. Every one of them is required rather
        // than optional — a compass with no magnet is not a worse compass, it
        // is not a compass — so they read as the body of the recipe.
        | PieceKind::Lens
        | PieceKind::Magnet
        | PieceKind::Earth => 0.45,
        // What you add once the item works.
        PieceKind::Accessory
        | PieceKind::Crest
        | PieceKind::Spell
        | PieceKind::Ring
        | PieceKind::Alignment => 0.72,
        // Drawn beneath the grid, so it wants to read as ground rather than as
        // gear: lighter than anything standing on it.
        PieceKind::Enchantment => 0.85,
        // Never reaches a grid at all. It has a brightness only because the bag
        // draws every component the same way.
        PieceKind::Quest => 0.60,
    }
}

/// The three role steps a player is meant to tell apart on a board.
///
/// `Enchantment` and `Quest` are not in it: one is ground and the other never
/// reaches a grid, so neither competes with the scale.
pub const ROLE_STEPS: [PieceKind; 3] =
    [PieceKind::Handle, PieceKind::Damaging, PieceKind::Accessory];

/// The least brightness two consecutive role steps may differ by.
///
/// **The one number that makes "cores are darker" true rather than
/// aspirational.** Pick colours by eye and this is the first thing that breaks.
pub const ROLE_SEPARATION: f32 = 0.08;

// ------------------------------------------------------------------ the marks

/// The shape stamped on every cell of a component.
///
/// This is the channel that says which grid a tile belongs to when colour says
/// nothing at all, so the six have to stay distinct from each other.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Motif {
    /// A blade's edge.
    Diagonal,
    /// A helm's dome.
    Dome,
    /// The bands of a cuirass.
    Bands,
    /// A gauntlet's weave.
    Weave,
    /// The straps of a greave.
    Straps,
    /// Not any one grid's mark: the component fits more than one and is in
    /// none of them yet.
    Shared,
}

impl Motif {
    /// A stable name for the renderer to switch on.
    pub fn name(self) -> &'static str {
        match self {
            Motif::Diagonal => "diagonal",
            Motif::Dome => "dome",
            Motif::Bands => "bands",
            Motif::Weave => "weave",
            Motif::Straps => "straps",
            Motif::Shared => "shared",
        }
    }
}

pub fn motif(slot: SlotKind) -> Motif {
    match slot {
        SlotKind::Weapon => Motif::Diagonal,
        SlotKind::Helmet => Motif::Dome,
        SlotKind::Chest => Motif::Bands,
        SlotKind::Gloves => Motif::Weave,
        SlotKind::Greaves => Motif::Straps,
    }
}

// ------------------------------------------------------------------ colour

/// Straight red, green and blue, each 0..1.
pub type Rgb = [f32; 3];

/// Perceived brightness, 0..1.
///
/// Used to pick an ink that will show up on a tile whatever colour it is, and —
/// more importantly — to check in the tests that the roles stay apart once
/// colour is taken away.
pub fn luminance(c: Rgb) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// HSL to RGB, hue in turns.
fn hsl(h: f32, s: f32, l: f32) -> Rgb {
    if s <= 0.0 {
        return [l, l, l];
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let f = |mut t: f32| {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [f(h + 1.0 / 3.0), f(h), f(h - 1.0 / 3.0)]
}

/// A slot's hue at a given brightness.
///
/// Luminance rises monotonically with HSL lightness, so a short bisection lands
/// on the lightness that hits the target whatever the hue happens to be worth.
/// Sixteen halvings is well past the precision of an eight-bit channel.
pub fn slot_color(slot: SlotKind, target: f32) -> Rgb {
    let (hue, sat) = (slot_hue(slot), slot_sat(slot));
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut out = hsl(hue, sat, 0.5);
    for _ in 0..16 {
        let mid = 0.5 * (lo + hi);
        out = hsl(hue, sat, mid);
        if luminance(out) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    out
}

/// The grey a shared component wears before it is placed, at the brightness its
/// role calls for.
///
/// Grey has no hue to carry a slot, which is the point — a steel material is
/// not a glove or a greave until it is in one. Role brightness still reads, so
/// the three-step scale survives.
///
/// `luminance`'s weights sum to 1, so a neutral grey's luminance is its own
/// channel value and no bisection is needed.
pub fn unplaced_color(kind: PieceKind) -> Rgb {
    let l = kind_luminance(kind);
    [l, l, l]
}

/// Ink for a motif on a given tile: black on light, white on dark.
///
/// Returned as `(rgb, alpha)`. One branch, and a test proves the composited
/// result always lands at least [`INK_SEPARATION`] away from the tile it sits
/// on, for every slot and role.
pub fn motif_ink(fill: Rgb) -> (Rgb, f32) {
    if luminance(fill) > 0.46 {
        ([0.0, 0.0, 0.0], 0.42)
    } else {
        ([1.0, 1.0, 1.0], 0.40)
    }
}

/// The least the ink may differ, in brightness, from the tile under it.
pub const INK_SEPARATION: f32 = 0.06;

/// How a component reads: its fill, and the mark it wears.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Look {
    pub fill: Rgb,
    pub motif: Motif,
}

/// The mark and fill a component wears — its grid's if it is in one, the shared
/// mark and no colour at all if it fits several and is in none.
///
/// A component that only ever goes one place is drawn as that place even when
/// it is loose: there is no ambiguity to represent, and greying it would lose
/// information for nothing.
pub fn look(def: &PieceDef, worn_in: Option<SlotKind>) -> Look {
    match worn_in {
        Some(slot) => Look {
            fill: slot_color(slot, kind_luminance(def.kind)),
            motif: motif(slot),
        },
        None if !def.shared() => Look {
            fill: slot_color(def.slot, kind_luminance(def.kind)),
            motif: motif(def.slot),
        },
        None => Look { fill: unplaced_color(def.kind), motif: Motif::Shared },
    }
}

/// `#rrggbb`, for a renderer that wants a string.
pub fn hex(c: Rgb) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
}

// ------------------------------------------------------------------ the board

/// What a board's ink does, in one place, so a renderer states none of it.
///
/// The rules the original arrived at, with the reasoning that produced them.
pub mod board {
    /// The empty grid, alternating on `(x + y) % 2`. Low contrast on purpose:
    /// it is a ruler, not a subject.
    pub const CELL_A: &str = "#242432";
    pub const CELL_B: &str = "#2b2b3a";

    /// The dark edge traced round a component — and **only** where the
    /// component actually ends, so a four-cell blade reads as one blade and the
    /// lines you do see inside an item are the seams between its parts.
    pub const PIECE_EDGE: &str = "rgba(0,0,0,.75)";

    /// An assembled item: pulsing white, thick.
    ///
    /// **Not gold against red.** Upstream's own note, and the reason:
    ///
    /// > A gold-versus-red pair is the one distinction red-green colour
    /// > blindness is worst at, and the gold read as the greaves besides.
    ///
    /// Brightness *and* stroke weight, so it survives monochromacy and a bad
    /// display both.
    pub const ASSEMBLED: &str = "#ffffff";
    pub const ASSEMBLED_WIDTH: f32 = 3.5;
    /// Alpha swings between these at [`PULSE_HZ`].
    pub const ASSEMBLED_ALPHA: (f32, f32) = (0.72, 1.00);
    pub const PULSE_HZ: f32 = 0.477; // 3 rad/s

    /// An item that has not come together: near-black, thin. It recedes rather
    /// than shouting — the status line is what says what is missing.
    pub const UNASSEMBLED: &str = "#18161e";
    pub const UNASSEMBLED_WIDTH: f32 = 2.0;

    /// A locked item: solid gold, so that "I decided this" reads differently
    /// from "this happens to be assembled".
    pub const LOCKED: &str = "#f0c85a";
    pub const LOCKED_WIDTH: f32 = 3.0;

    /// The drag footprint: the cells a drop would actually claim.
    pub const LEGAL: &str = "#5ac882";
    pub const ILLEGAL: &str = "#e65f5f";
    pub const FOOTPRINT_ALPHA: f32 = 0.38;

    /// Markers for a cell carrying a positional effect or a trigger.
    pub const EFFECT: &str = "#69cdeb";
    pub const TRIGGER: &str = "#e182e1";
}
