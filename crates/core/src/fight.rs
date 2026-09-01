//! An encounter, from the tile it started on to the receipt it leaves.
//!
//! # Why a mid-fight save is cheap
//!
//! `PLAN.md` §6 worried about saving mid-fight and answered it by storing the
//! pre-fight state and the seed. The engine makes that unnecessary and the
//! reason is worth writing down, because it is one of the properties the fork
//! exists to keep: **there is no RNG in combat.** A fight is a pure function of
//! the player's stats, their assembled items, the creature's spec and the
//! difficulty. Nothing is rolled once the bell goes.
//!
//! So an [`Encounter`] carries no log and no seed. It names the creature and
//! the tile, and that is enough: the board is in the save already, and running
//! [`run`] on a loaded game produces the same fight character for character.
//! The saved thing is the *situation*, not the replay.
//!
//! # The order settlement happens in
//!
//! Pay, then move. A loss pays nothing (`reward.rs` has the argument) and sends
//! the player back to the last town they stood in; a win pays the bounty and
//! the creature's rating as experience, and leaves them where they are. Nothing
//! here reaches into the map — the caller does the walking, because whoever
//! owns the world owns where the player is.

use serde::{Deserialize, Serialize};

use crate::combat::{self, CombatLog, Difficulty, MonsterSpec, Outcome};
use crate::game::Game;
use crate::reward;

/// A fight that has started and not yet been settled.
///
/// Saved with the game. It holds no log because a log is derivable, and no
/// seed because combat does not draw.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    /// Canonical creature name, as `combat::creature` knows it.
    pub enemy: String,
    /// Where it happened, so a loss knows what it is walking away from.
    pub at: [u8; 2],
}

/// What a settled fight did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settlement {
    pub outcome: Outcome,
    pub gold: i32,
    pub xp: i32,
    /// Levels crossed. More than one is possible off a single fight, and a
    /// receipt that only named the last would swallow a point and a row.
    pub levels: Vec<u32>,
    /// Which grids grew, and by how much. The plan asks the level-up to say
    /// *which* board grew, so it is recorded rather than left to be inferred.
    pub grew: Vec<(String, u8)>,
    /// Set when a loss sent the player home, naming the town.
    pub sent_home: Option<String>,
    /// One line each, in the order they happened, for the result card.
    pub receipt: Vec<String>,
}

/// The creature an encounter names.
pub fn spec(e: &Encounter) -> Option<&'static MonsterSpec> {
    combat::creature(&e.enemy)
}

/// Run the fight this game is standing in.
///
/// Returns `None` when there is no encounter, rather than inventing one. The
/// log is not stored anywhere: it is produced on demand and produced the same
/// way every time, which is what makes the replay button honest.
pub fn run(game: &Game, difficulty: Difficulty) -> Option<CombatLog> {
    let e = game.encounter.as_ref()?;
    let spec = spec(e)?;
    Some(combat::simulate_at(
        game.character.player_stats(),
        &game.character.combat_items(),
        spec,
        difficulty,
    ))
}

/// Bank the result and clear the encounter.
///
/// Idempotent in the sense that matters: with no encounter it does nothing and
/// says so, so a page that settles twice does not pay twice.
pub fn settle(game: &mut Game, log: &CombatLog, difficulty: Difficulty) -> Option<Settlement> {
    let e = game.encounter.take()?;
    let spec = spec(&e)?;

    let gold = reward::bounty_for(log.outcome, spec.bounty);
    let rating = crate::rating::creature_rating(spec, difficulty);
    let xp = reward::xp_for(log.outcome, crate::progression::xp_for_rating(rating));

    let mut receipt = Vec::new();
    let mut sent_home = None;
    let mut levels = Vec::new();
    let mut grew = Vec::new();

    match log.outcome {
        Outcome::Victory => {
            game.character.gold += gold;
            receipt.push(format!("+{gold} Fnorp"));
            if xp > 0 {
                levels = game.character.gain_xp(xp);
                receipt.push(format!("+{xp} experience"));
            }
            for level in &levels {
                receipt.push(format!("Level {level}. One point to spend."));
            }
            if !levels.is_empty() {
                let granted = crate::data::skills().granted_rows(&game.character.skills_taken);
                for (slot, rows) in game.character.resize_boards(granted) {
                    let name = format!("{slot:?}").to_lowercase();
                    receipt.push(format!(
                        "+{rows} row on the {name} frame",
                    ));
                    grew.push((name, rows));
                }
            }
            game.world.bump("wins");
        }
        Outcome::Defeat | Outcome::Stalemate => {
            game.world.bump("losses");
            // No bounty. The whole argument is in `reward.rs`; the short of it
            // is that upstream's reasoning held because a ladder is a corridor
            // and this is not one.
            receipt.push("No bounty. Nothing was beaten.".into());
            if xp < 0 {
                game.character.gain_xp(xp);
                receipt.push(format!("−{} experience", -xp));
            }
            sent_home = Some(game.world.last_town.clone()).filter(|t| !t.is_empty());
            receipt.push(match &sent_home {
                Some(_) => "You wake up walking, and you have been walking a while.".into(),
                None => "You wake up where you fell.".into(),
            });
        }
    }

    Some(Settlement { outcome: log.outcome, gold, xp, levels, grew, sent_home, receipt })
}
