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
/// What a settlement rule needs to know beyond the outcome and the clock.
///
/// **The board and the corpse, carried in.** `bounty_with_class` had two
/// arguments and could answer everything `Showstopper` asked; the two purse
/// experts ask about things it has never been told — how much of the board you
/// left empty, and what was still standing on the creature when it went down.
///
/// A struct rather than four more parameters, because the next settlement rule
/// will want a fifth and a five-argument call at three sites is a call nobody
/// can read. `Default` is *a packed board, an uncursed corpse and no streak*,
/// which is the honest answer for every caller that has no board — and it
/// makes every existing test compile unchanged while measuring the same game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtTheBell {
    /// Worn frames with nothing seated in them, 0 to 5.
    pub empty_frames: u32,
    /// Curses still standing on what you beat when it went down.
    pub curses_standing: u32,
    /// How many *different* kinds those were. There are four kinds.
    pub curse_kinds: u32,
    /// Curses you landed on it that had expired before the bell.
    pub curses_expired: u32,
    /// Whether the thing that went down was **unmade** rather than merely
    /// killed.
    ///
    /// **Curtain Line's, and it is the one fact about a fight the purse could
    /// not already read.** A Short Programme pays for a fast win however it
    /// ended; this pays for how it ended, so the two are different questions
    /// and the field is what makes them so.
    pub unmade: bool,
    /// Fast wins in a row **before** this one.
    ///
    /// Carried on the character, because it outlives the fight — see
    /// `Character::fast_wins`. The only other thing in this block that reaches
    /// past the bell is Standing Fact's `told`.
    pub streak: u32,
}

pub fn bounty_with_class(
    outcome: Outcome,
    bounty: i32,
    classes: &[crate::class::ClassDef],
    duration_ms: u32,
    at: AtTheBell,
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
            // Neither of M16's pays a purse. The Stoker is a fight rule and
            // the Whisperer changes where a fight *ends* — and an unmaking is a
            // kill, so it pays what a kill pays through the ordinary path
            // rather than through a second one here.
            ClassPower::Stoker { .. } | ClassPower::Whisperer { .. } => {}
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
            // **Two of the ten experts argue here and eight do not.** Short
            // Programme is Showstopper's window widened by what you left
            // empty, and Eleventh Season bills the fallen by the curse; both
            // are settlement rules and neither is a fight rule, which is the
            // same division `Showstopper` itself makes. The other eight are
            // the fight's or the board's and say so, so that adding an expert
            // is a decision about the purse rather than a silence.
            ClassPower::Expert(e) => pct += expert_pct(e, duration_ms, at),
        }
    }
    base + base * pct / 100
}

/// What one expert adds to a purse, in percentage points.
///
/// **Exhaustive over all ten**, for the reason the match above is: an expert
/// added to the game is one somebody decided does not pay, rather than one
/// that quietly does not.
fn expert_pct(e: crate::expert::ExpertPower, duration_ms: u32, at: AtTheBell) -> i32 {
    use crate::expert::ExpertPower::*;
    match e {
        // **Showstopper's window, widened by what you left empty.** The base
        // is a floor rather than the whole answer, which is the class in one
        // line: a packed build gets the floor and a bare one gets the floor
        // plus a second a frame. The streak adds to the *window*, not to the
        // purse — a class that paid more for being on a run would be paying
        // twice for one fast fight.
        ShortProgramme { per_slot, floor_ms, pct, streak } => {
            let window = floor_ms
                + per_slot * 1000 * at.empty_frames as i32
                + streak * 1000 * (at.streak as i32).min(crate::expert::ExpertPower::STREAK_CAP);
            if (duration_ms as i32) < window {
                pct
            } else {
                0
            }
        }
        // **Billed by the curse.** What was standing when it went down, capped;
        // plus tenths of a curse for each one that had already expired, which
        // is what rescues the long fight this class otherwise pays least for.
        // Four *different* kinds doubles the column — there are exactly four
        // kinds, so it is the only thing in the game that asks for all of them.
        EleventhSeason { pct, count_cap, distinct, posthumous } => {
            let counted = at.curses_standing.min(count_cap.max(0) as u32) as i32;
            // Tenths, so half a curse is `5` and the arithmetic stays integer
            // — the same reason every roll in this game is per-mille.
            let ghosts = at.curses_expired as i32 * posthumous / 10;
            let mut column = (counted + ghosts) * pct;
            if distinct > 0 && at.curse_kinds >= crate::curse::CurseKind::ALL.len() as u32 {
                column *= 2;
            }
            column
        }
        // **Curtain Line, which is the Showstopper's other half.** A creature
        // *unmade* inside the window pays again on top of whatever the bill
        // already was — and `at.unmade` is the one thing that makes it a
        // different question from Short Programme's, which pays for a fast
        // win however it ended.
        CurtainLine { mult, under_ms, .. } => {
            if at.unmade && (duration_ms as i32) < under_ms {
                mult
            } else {
                0
            }
        }
        // **Flash Powder is the Showstopper's half of its pairing**, so its
        // window and its cut are the purse's the way Short Programme's are —
        // and it is a different question from that one: this pays for a fast
        // win on a board that *spent everything at the bell*, which is the
        // trade, and Short Programme pays for a fast win on a bare board.
        FlashPowder { until_ms, pct, .. } => {
            if (duration_ms as i32) < until_ms {
                pct
            } else {
                0
            }
        }
        // The fight's, read off `Combatant::expert` at the tick.
        BareFurnace { .. }
        | FiredFunnel { .. }
        | ColdStoke { .. }
        | PonkeyBoiler { .. }
        | LoudDoubt { .. }
        | RequisitionedSilence { .. }
        | ToldOnce { .. }
        | LicensedRumour { .. }
        | AshAndWhisper { .. }
        | LoudCalculation { .. }
        | StandingFact { .. }
        | OverwoundArm { .. }
        | CurseRequisition { .. }
        | PatentedFunnel { .. }
        | OpeningNumber { .. }
        | CursedLicence { .. } => 0,
        // The board's: how many enchs fit on a component and what an enched
        // component lends its neighbours are already in the profiles.
        FullBill { .. } => 0,
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
