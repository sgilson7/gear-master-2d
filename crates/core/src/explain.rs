//! What a single component does, in the engine's own words.
//!
//! **The engine owns the sentence.** `tests/tooltips.rs` states the principle
//! for doors and receipts — if the interface writes the sentence, then the
//! interface writes it again for the next screen, the theme layer has nothing
//! to swap, and the copies drift. A component is the last thing in the game
//! that could not explain itself: a card said what an *item* does, and a piece
//! on the board said its name and nothing else.
//!
//! Unthemed and numeric, the same register as a skill node's line and for the
//! same reason (`TONE.md` 13a). Somebody deciding between two blades on a shelf
//! is comparing numbers, and a number wearing a joke has to be translated
//! first. The *name* above these lines is the theme's; these are not.
//!
//! Both matches are exhaustive on purpose. A new `Action` or `Trigger` is a
//! compile error here until somebody says what it does, which is the only way
//! this stays true as the catalogue grows.

use crate::piece::PieceDef;

/// `+3` / `−3`, so a cost and a gain never read alike.
fn signed(n: i32) -> String {
    if n < 0 {
        format!("−{}", -n)
    } else {
        format!("+{n}")
    }
}

fn secs(ms: u32) -> String {
    format!("{:.2}s", ms as f32 / 1000.0)
}

// `Action::describe` and `Trigger::describe` already existed in `piece.rs`,
// exhaustive and in this register. The first draft of this file wrote them
// again — which is the exact failure the principle above warns about, arrived
// at from the other direction. They are used here, not duplicated.

/// Everything one component contributes, as lines.
///
/// The same split the item card uses, because it is the same question: what
/// does it do standing still, and what does it do when its item comes round.
/// A piece's own `base` is folded into its item's total, so the classification
/// has to match or the two screens disagree about the same number.
pub fn piece_lines(def: &PieceDef) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    let s = &def.base;
    for (v, label, unit) in [
        (s.health, "max health", ""),
        (s.strength, "strength", ""),
        (s.regen, "regen a second", ""),
        (s.power, "weapon power", "%"),
        (s.mind_resist, "mind resist", "%"),
        (s.curse_resist, "curse resist", "%"),
        (s.physical_resist, "physical resist", "%"),
        (s.magic_resist, "magic resist", "%"),
        (s.physical_pierce, "physical piercing", "%"),
        (s.magic_pierce, "magic piercing", "%"),
        (s.physical_harden, "physical hardening", "%"),
        (s.magic_harden, "magic hardening", "%"),
        (s.reflect, "reflected", "%"),
    ] {
        if v != 0 {
            out.push(("standing", format!("{}{unit} {label}", signed(v))));
        }
    }
    for (v, label) in [
        (s.physical_damage, "physical damage"),
        (s.magic_damage, "magic damage"),
        (s.mind, "mind damage"),
        (s.armor, "armor"),
        (s.mana, "mana"),
        (s.rage, "rage"),
        (s.faith, "faith"),
        (s.nature, "nature"),
    ] {
        if v != 0 {
            out.push(("activation", format!("{} {label}", signed(v))));
        }
    }
    if def.power_bonus != 0 {
        out.push(("standing", format!("{}% power, to this item only", def.power_bonus)));
    }
    if def.speed_bonus != 0 {
        out.push(("standing", format!("{}% to this item's speed", signed(def.speed_bonus))));
    }
    if def.cooldown_ms > 0 {
        out.push(("standing", format!("sets the item's cadence at {}", secs(def.cooldown_ms))));
    }
    if let Some(b) = def.assembly_bonus {
        out.push(("assembled", format!("{} — {}", b.label, b.stats.summary())));
    }
    if let Some(e) = def.effect {
        out.push(("effect", e.label.to_string()));
    }
    for t in def.triggers {
        out.push(("trigger", t.describe()));
    }
    out
}
