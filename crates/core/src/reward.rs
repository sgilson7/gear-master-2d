//! What a fight pays.
//!
//! One function, and a long reason for it.
//!
//! # The bounty is not paid on a loss
//!
//! Upstream paid it either way, and said so on purpose (`run.rs`, `settle`):
//!
//! > The bounty is paid whatever happened. Losing is meant to be a setback,
//! > not a dead end: a run with no income cannot buy its way past whatever
//! > just beat it, and would have nothing to do but replay a fight it already
//! > knows it loses.
//!
//! That argument is correct, and it is correct **because a ladder is a
//! corridor**. On a ladder the only fight available is the one in front of
//! you, so a player with no income has no move; paying them anyway is what
//! keeps the run alive.
//!
//! GM2D is not a corridor. A player who loses can walk to a lower-danger
//! region, fight something they can beat, and come back — the world supplies
//! the escape hatch the ladder could not. Which removes the justification and
//! leaves only the consequence: a lose/win cycle that pays every time is an
//! unbounded, risk-free gold farm, measured upstream at +17 gold a cycle with
//! Grinder's one-rung knockback. In an open world with no rung to knock back,
//! it is the whole game.
//!
//! So: paid on a win, and not otherwise. What losing costs beyond the missed
//! reward is [`LOSS_XP_PCT`], which ships at zero — the walk back to town is
//! the penalty, and whether that is enough is a question for the first build
//! anybody plays in sequence.
//!
//! This is a divergence from upstream's stated intent rather than a bug found
//! in it, and `CLAUDE.md` says so.

use crate::combat::Outcome;

/// Share of banked XP a loss takes back, as a percentage.
///
/// Zero, deliberately. The knob exists so the answer can change after gate 4
/// without the question having to be re-litigated; it is not a placeholder for
/// a number nobody has picked.
pub const LOSS_XP_PCT: i32 = 0;

/// Gold for a finished fight.
///
/// `bounty` is the creature's own, from its spec. A stalemate pays nothing for
/// the same reason a defeat does: nothing was beaten.
pub fn bounty_for(outcome: Outcome, bounty: i32) -> i32 {
    match outcome {
        Outcome::Victory => bounty.max(0),
        Outcome::Defeat | Outcome::Stalemate => 0,
    }
}

/// XP for a finished fight, before the level curve is consulted.
///
/// A win pays the creature's rating; a loss pays [`LOSS_XP_PCT`] of it back,
/// which at zero means a loss neither gives nor takes. Kept beside the bounty
/// so the two answers to "what did that fight do for me" are read off one
/// page.
pub fn xp_for(outcome: Outcome, rating: i32) -> i32 {
    match outcome {
        Outcome::Victory => rating.max(0),
        Outcome::Defeat | Outcome::Stalemate => -(rating.max(0) * LOSS_XP_PCT / 100),
    }
}
