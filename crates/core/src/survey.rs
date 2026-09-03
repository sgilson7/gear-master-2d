//! What an instrument does to the map it is pointed at.
//!
//! # The map is static and the lens is not
//!
//! `PLAN-M11.md`'s ask is explicit and is the opposite of Path of Exile's: a
//! surveyable map is **authored, not rolled**, so quests can name places on it
//! and a person can learn where things are. What varies is the instrument you
//! were carrying when you walked in, and everything it varies is a *number* —
//! how often the ground stops you, how much a fight pays, what falls off it.
//!
//! # A pure function, and the test says so
//!
//! [`mods_for`] takes a map id, an instrument and one number off the character,
//! and returns a [`SurveyMod`]. It reads no state, writes none, and nothing
//! about a survey is stored in a map file — which is the whole of what makes a
//! second surveyable map a data drop rather than a milestone.
//!
//! # Why the compass reads the board
//!
//! *Augmented by gear*, from the ask. A packed board surveys better, which is
//! the game's own thesis restated: the arrangement is the input. It is the
//! count of assembled items rather than a rating, because a count is a thing a
//! player can see on the packing screen without opening anything.

use serde::{Deserialize, Serialize};

/// What one entry onto a surveyed map reads like.
///
/// Every field is a percentage or a per-mille, and `SurveyMod::none()` is the
/// map as written — so a caller that has no survey does not have to know that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyMod {
    /// Percent on every tile's encounter chance. Negative is quieter ground.
    pub encounter_pct: i32,
    /// Added to every drop roll's per-mille, on this map only.
    pub drops_per_mille: i32,
    /// Percent on the experience a win here pays.
    pub xp_pct: i32,
    /// A golem walked in with you and takes the first fight.
    pub golem: bool,
}

impl SurveyMod {
    /// The map as written.
    pub const fn none() -> Self {
        SurveyMod { encounter_pct: 0, drops_per_mille: 0, xp_pct: 0, golem: false }
    }

    pub fn is_none(&self) -> bool {
        *self == SurveyMod::none()
    }
}

/// What a compass takes off the encounter rate before the board is counted.
pub const COMPASS_QUIET_PCT: i32 = -20;
/// And what each assembled item takes off on top of it.
///
/// *Augmented by gear.* Five items — a full board — is another fifteen percent,
/// which is a third off the rate for somebody who packed properly and nothing
/// at all for somebody who did not.
pub const COMPASS_PER_ITEM_PCT: i32 = -3;
/// The most a compass may ever take off, however packed the board is.
///
/// A rate that can reach zero is a map you cannot fight on, which would make
/// the compass a way of switching the game off rather than a way of reading it.
pub const COMPASS_FLOOR_PCT: i32 = -45;

/// What an atlas adds to every drop roll on the map it is pointed at.
pub const ATLAS_DROPS_PER_MILLE: i32 = 120;
/// And to what a win pays.
pub const ATLAS_XP_PCT: i32 = 40;
/// And what it costs: the reach heard the promise.
pub const ATLAS_LOUD_PCT: i32 = 10;

/// The modifiers one instrument makes to one map.
///
/// **Pure**, and `tests/reach.rs` says so by calling it twice. `map` is taken
/// and currently unread, which is deliberate: a second surveyable map is meant
/// to be a data drop plus an arm here, and a signature that could not tell two
/// maps apart would have to change to become one that could.
pub fn mods_for(map: &str, kind: &str, items_assembled: usize) -> SurveyMod {
    let _ = map;
    match kind {
        // **The honest read.** Fewer things stop you, and how many fewer is
        // what is on your board.
        "compass" => SurveyMod {
            encounter_pct: (COMPASS_QUIET_PCT
                + COMPASS_PER_ITEM_PCT * items_assembled as i32)
                .max(COMPASS_FLOOR_PCT),
            ..SurveyMod::none()
        },
        // **The cosmic read.** More falls off it and it pays better, and it
        // stops you more often, because an atlas is a promise and the reach
        // heard it.
        "atlas" => SurveyMod {
            encounter_pct: ATLAS_LOUD_PCT,
            drops_per_mille: ATLAS_DROPS_PER_MILLE,
            xp_pct: ATLAS_XP_PCT,
            golem: false,
        },
        // **The accompanied read.** Something walked in with you.
        //
        // `PLAN-M11.md` §M11.6 wanted an ally row in the replay and named the
        // fallback in advance so that taking it would be a decision rather than
        // a retreat (§8 row 6): the golem *handles one*. It takes the first
        // encounter of each entry, which is settled where a rout is settled —
        // before there is a fight to draw — and pays what a win pays.
        //
        // The ally row is M12's. What made the fallback the right call is not
        // the layout: it is rule 5. A third board is a third set of numbers the
        // page must not invent, and the honest version of it is a third
        // combatant in `combat.rs`, which is new combat code in a block that
        // has added none.
        "golem" => SurveyMod { golem: true, ..SurveyMod::none() },
        _ => SurveyMod::none(),
    }
}

/// Apply a percentage to a per-mille, in integers, never below zero.
pub fn shift(per_mille: i32, pct: i32) -> i32 {
    let out = per_mille as i64 * (100 + pct as i64) / 100;
    out.clamp(0, i32::MAX as i64) as i32
}
