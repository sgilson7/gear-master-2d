//! Levels, and what a level does.
//!
//! Two rules, both of them pure functions of the level, and both of them stated
//! here rather than anywhere they could drift.
//!
//! # The curve
//!
//! **Quadratic to fifty, exponential after it**, which is the human's ask in
//! their own words: *"make the experience curve for levelling up less steep, so
//! you level up a bit faster at higher levels. i want the experience needed to
//! be quadratic up till level 50, then exponential after level 50."*
//!
//! ```text
//! xp_to_next(L) = A·L² + B·L + C                    for L <= 50
//!               = (A·50² + B·50 + C) · 1.35^(L−50)  for L >= 50
//! ```
//!
//! The two arms **agree at fifty** rather than meeting near it, which is not
//! negotiable: fifty is the one level a player will be watching for, and a
//! curve with a cliff in it there is a curve that has been described wrongly.
//! [`curve`] is the one place the sum is done and [`XP_TO_NEXT`] is it
//! evaluated once, so the table and the formula cannot drift —
//! `the_table_matches_the_formula` regenerates it over **both** arms.
//!
//! It was `round(20 · 1.35^(L−1))`, exponential from level one, which is why
//! level twenty cost seventeen thousand and levels past twenty were a wall
//! rather than a curve. See [`CURVE_A`] for what the three coefficients are
//! fitted to and why the base past fifty is not a new number.
//!
//! Read from the table at runtime and never computed, so nothing does float
//! arithmetic on a level-up — a level is a thing a player watches happen, and a
//! level that lands one experience point differently on one machine is a save
//! that disagrees with itself.
//!
//! # The rows
//!
//! **A level does not hand one out.** Every grid starts at [`STARTING_ROWS`]
//! and stays there until something is earned for it — a skill point spent on a
//! row node, or an errand finished that pays one. M12.3 retired the rotation
//! that used to grow one grid a level, and [`board_rows`] is what replaced
//! [`rows_for`]: a pure function of what has been earned rather than of the
//! level, so it is still checkable rather than trusted.
//!
//! Why it changed is measured rather than argued: fill went *down* as a
//! character levelled — 43% at five, 37% at eight — because rows arrived on a
//! clock and components did not. See `crate::pressure`.



/// Rows every grid starts with.
///
/// Three, not the engine's eight. A Sprocketman climbs out of the pit with
/// almost nothing and grows into their frames; the creatures they fight keep
/// full boards, which is why the early game is spent outgunned in board area.
/// That is the intended shape of a grindy game rather than an oversight, and
/// `enemies.json` is the evidence they are not sharing this number.
pub const STARTING_ROWS: u8 = 3;

/// The tallest a grid is ever built to, however the rows were earned.
///
/// The engine's own `SLOT_H`. Beyond it the catalogue has nothing that needs
/// the room, and a board taller than the pieces is a board with a dead half.
pub const MAX_ROWS: u8 = crate::slot::SLOT_H;

// `ROTATION` — the fixed weapon/chest/helmet/gloves/greaves order a level used
// to grow — is deleted rather than kept, because M12.3 retired the thing it
// described. A `pub const` nothing calls is a comment nothing checks, and this
// file has one of those in `character.rs::STARTER` already.

/// The highest level the table covers.
///
/// **Sixty, because a curve that changes shape at fifty needs somewhere to go
/// past it** or the change of shape is decoration. It was 32, which is a table
/// that stops nine levels before the joint.
///
/// Ten levels of exponential is the recommendation `PLAN-M15.md` §5 decision 1
/// makes — *"60 is a suggestion and not an answer"* — taken because the work
/// could not start without a number, and flagged as the human's. What decides
/// whether it can go further is [`CURVE_BASE`]: the first level whose cost does
/// not fit an `i32` is **95**, so there are thirty-five levels of headroom and
/// `the_curve_never_overflows` is what holds it.
pub const MAX_LEVEL: usize = 60;

/// Where the quadratic stops and the exponential starts.
///
/// The human's number. Both arms are evaluated here and they agree, which is
/// what makes it a joint rather than a step.
pub const JOINT: u32 = 50;

/// The quadratic's square term.
///
/// **Solved, not chosen.** Three constraints fix all three coefficients, and
/// the first two are contracts this game already had:
///
/// | constraint | value | why |
/// |---|---|---|
/// | `xp_to_next(1)` | 20 | the first level costs what it always has |
/// | `xp_to_reach(5)` | 132 | the one measured contract in the game |
/// | `xp_to_reach(20)` | 4,300 | the ask |
///
/// **Pinning `reach(5)` at 132 exactly is the useful half.**
/// `level_five_lands_where_the_plan_says` walks the shipped map and demands
/// level five in 25–35 fights, and [`XP_DIVISOR`] is set by that test rather
/// than by taste. Fitting the curve to hold the one number that test measures
/// means the divisor does not move and the test passes untouched — and a
/// divisor that *had* to move would mean the fit was wrong rather than that the
/// test was.
///
/// **4,300 is measured and it is not the plan's 6,000.** The human's anchor is
/// *"about 150 fights, and less than it is now by about half — take whichever
/// is lower"*, and `PLAN-M15.md` §2.5 recommends 6,000 on the reasoning that
/// 150 fights at forty experience a win is plausible. It is not: replaying the
/// shipped walk's own payouts, the mean over its first 150 wins is **28.7**, so
/// 6,000 puts level twenty at 184 fights and 5,500 at 175 — both outside the
/// band. 4,300 lands it at **150 exactly**, and clears the half-bound of 8,526
/// with room to spare, because the half was never the binding one. The plan
/// says what to do in this case in as many words: *if the half still leaves 150
/// out of reach, go under it.*
pub const CURVE_A: f64 = 1.4257309942;
/// The quadratic's linear term. See [`CURVE_A`].
pub const CURVE_B: f64 = 2.4884990253;
/// The quadratic's constant term. See [`CURVE_A`].
pub const CURVE_C: f64 = 16.0857699805;

/// What each level past [`JOINT`] multiplies the one before it by.
///
/// **Not a new number: it is the curve this game already had.** Every level
/// from one to thirty-two used to cost 1.35 times the one before it, and what
/// M15.3 does is replace that with a quadratic *up to fifty* and let the old
/// growth rate take over again past it. So the wall past the joint is the wall
/// the game has always had, moved to where the ask puts it.
///
/// It is steep on purpose and by a measured amount: at level sixty the
/// exponential asks **14 times** what the quadratic would have asked, which is
/// what makes the change of shape a change rather than a decoration.
/// `the_curve_past_fifty_is_steeper_than_the_quadratic_would_have_been` is that
/// sentence as a check, because an exponential that a quadratic keeps up with
/// is an exponential nobody would notice.
pub const CURVE_BASE: f64 = 1.35;

/// What leaving `level` costs, as the formula rather than as the table.
///
/// **The one place the sum is done.** [`XP_TO_NEXT`] is this evaluated once at
/// authoring time and `the_table_matches_the_formula` regenerates it over both
/// arms, so the table cannot drift from the curve — which is the same
/// arrangement `Node::line` has with the effect it describes.
///
/// Both arms are defined at [`JOINT`] and both give the same answer there. That
/// is the joint, and `the_curve_has_no_step_at_fifty` asks each arm for it
/// separately rather than trusting the `<=`.
pub fn curve(level: u32) -> f64 {
    let l = level.max(1) as f64;
    let j = JOINT as f64;
    let quadratic = |x: f64| CURVE_A * x * x + CURVE_B * x + CURVE_C;
    if level <= JOINT {
        quadratic(l)
    } else {
        quadratic(j) * CURVE_BASE.powi((level - JOINT) as i32)
    }
}

/// Experience from one level to the next, indexed by level − 1.
///
/// `XP_TO_NEXT[0]` is what level 1 costs to leave. Generated by the test
/// `the_table_matches_the_formula` off [`curve`], so the numbers here and the
/// formula above it cannot drift apart without something going red.
pub const XP_TO_NEXT: [i32; MAX_LEVEL] = [
    20, 27, 36, 49, 64, 82, 103, 127,
    154, 184, 216, 251, 289, 330, 374, 421,
    470, 523, 578, 636, 697, 761, 828, 897,
    969, 1045, 1123, 1204, 1287, 1374, 1463, 1556,
    1651, 1749, 1850, 1953, 2060, 2169, 2282, 2397,
    2515, 2636, 2759, 2886, 3015, 3147, 3282, 3420,
    3561, 3705, 5002, 6752, 9115, 12306, 16613, 22427,
    30276, 40873, 55179, 74492,
];

/// What it costs to leave `level`.
pub fn xp_to_next(level: u32) -> i32 {
    let i = (level.max(1) as usize - 1).min(MAX_LEVEL - 1);
    XP_TO_NEXT[i]
}

/// Total experience banked to reach `level` from level 1.
pub fn xp_to_reach(level: u32) -> i32 {
    (1..level.max(1)).map(xp_to_next).sum()
}

/// The level a total amount of banked experience buys.
pub fn level_for(total_xp: i32) -> u32 {
    let mut level = 1;
    let mut spent = 0;
    while (level as usize) < MAX_LEVEL {
        let need = xp_to_next(level);
        if total_xp - spent < need {
            break;
        }
        spent += need;
        level += 1;
    }
    level
}

/// How far through the current level a total stands: `(into, needed)`.
pub fn progress(total_xp: i32) -> (i32, i32) {
    let level = level_for(total_xp);
    (total_xp - xp_to_reach(level), xp_to_next(level))
}

/// **A level grows no board. M12.3 retired that, and it was an MVP pillar.**
///
/// `PLAN.md` M4 made board size a pure function of level and the rotation in
/// `ROTATION` decided which grid grew when. That was right while pieces were
/// scarce by accident: a row was the only thing that ever changed about a
/// board, so handing one out on a schedule was the whole of progression.
///
/// M12.0 measured what it costs once pieces are not scarce. **Fill goes
/// *down* as you level** — 43% at five, 37% at eight — because rows arrive on
/// a clock and components do not, so levelling dilutes you. A scheduled row is
/// dilution on a timer.
///
/// So a row is a thing you **earn** now: a skill point spent on it, or a quest
/// line finished. Every level poses the game's central question with the
/// player's own hands — power on the board you have, or a bigger board — and
/// pressure regulates itself instead of leaking away every fifth level.
///
/// **Nothing is banked to make this work**, which is a divergence from
/// `PLAN-M12.md` and the reason is that the save already carries it:
/// `BoardSave::rows` is written and restored verbatim and `resize_boards` only
/// ever grows, so an old file keeps every row it had without a ledger and
/// without a migration. What a row came *from* is derived — from
/// `skills_taken` and from `quests_done` — the same way a node's effect and a
/// tower's fallen floors are.
pub fn base_rows() -> u8 {
    STARTING_ROWS
}

/// Rows this grid has: the base, plus everything earned for it.
pub fn board_rows(granted: u8) -> u8 {
    (STARTING_ROWS + granted).min(MAX_ROWS)
}

/// How much experience beating this creature is worth.
///
/// The creature's rating on the shared scale, divided by [`XP_DIVISOR`]. Not a
/// hand-authored number per creature: a creature is worth what it is worth, and
/// `rating::creature_rating` is the one thing that knows.
///
/// The floor of one matters more than it looks. Early creatures rate well under
/// the divisor, so without it a rat would be worth nothing and the first hour
/// of the game would pay in zeroes.
pub fn xp_for_rating(rating: i32) -> i32 {
    (rating / XP_DIVISOR).max(1)
}

/// Turns a creature's rating into experience.
///
/// **Set by `tests/progression.rs::level_five_lands_where_the_plan_says`, not
/// by taste.** The plan committed to level 5 arriving in 25–35 fights; that
/// test walks the shipped map and fails outside the band, so this number is a
/// measurement of the map rather than an opinion about it. Changing the map's
/// regions can move it, which is the point — the band is the contract and this
/// is what is tuned to hold it.
pub const XP_DIVISOR: i32 = 5;
