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

/// What the classes add to a bounty, and what they add nothing to.
///
/// # Why this is here and not in `combat`
///
/// `Showstopper` — *a fight won under ten seconds pays fifty percent more* —
/// existed, was tuned, was themed, and was **honoured nowhere**. `combat.rs`
/// ignores it on purpose and correctly: it is a settlement rule and not a
/// combat one, and the fight has nothing to do with it. But `fight::settle`
/// never read the class either, so a player who took it would have paid an
/// irreversible choice at level five for nothing at all — which is the failure
/// eight skill nodes already cost this project two milestones.
///
/// So it lands where "what a fight pays" is argued, which is this file. §C.1 is
/// the precedent: the bounty's rules live here even when the thing that moves
/// them does not.
///
/// **Exhaustive**, so a class added to the game is a class somebody has decided
/// does not pay, rather than one that quietly does not.
/// `every_offered_class_reaches_something` is the other half of that guard.
pub fn bounty_with_class(
    outcome: Outcome,
    bounty: i32,
    classes: &[crate::class::ClassDef],
    duration_ms: u32,
) -> i32 {
    use crate::class::ClassPower;
    let base = bounty_for(outcome, bounty);
    if base == 0 {
        return 0;
    }
    let mut pct = 0;
    for c in classes {
        match c.power {
            // The one that pays. Quick is measured off the log's own duration,
            // so it is the fight that happened rather than an estimate of it.
            ClassPower::Showstopper { pct: more, under_ms } => {
                if duration_ms < under_ms {
                    pct += more;
                }
            }
            // Everything else is the fight's or the map's, and says so here so
            // that adding a class is a decision about the purse rather than a
            // silence. `Prospector` would belong here if anything in GM2D dealt
            // a named creature's gear; nothing does.
            ClassPower::Standing(_)
            | ClassPower::SlowTime(_)
            | ClassPower::Leeching(_)
            | ClassPower::Overflowing(_)
            | ClassPower::Echo(_)
            | ClassPower::Bastion(_)
            | ClassPower::Contagion(_)
            | ClassPower::Longhaul { .. }
            | ClassPower::Trundle { .. }
            | ClassPower::Recycler { .. }
            | ClassPower::Piety { .. }
            | ClassPower::Tired { .. }
            | ClassPower::Ticket { .. }
            | ClassPower::Guilt
            | ClassPower::Reprisal(_)
            | ClassPower::Riposte(_)
            | ClassPower::Momentum(_)
            | ClassPower::Resonance(_)
            | ClassPower::Transmute(_)
            | ClassPower::Adaptable(_)
            | ClassPower::Untimely(_)
            | ClassPower::Cascade(_)
            | ClassPower::Consecrate(_)
            | ClassPower::Bloodscent(_)
            | ClassPower::Confluence(_)
            | ClassPower::Splintered(_)
            | ClassPower::Unionized { .. }
            | ClassPower::Prospector(_)
            | ClassPower::FirstBlood
            | ClassPower::WrongSense(_)
            | ClassPower::Avenged(_) => {}
        }
    }
    base + base * pct / 100
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
