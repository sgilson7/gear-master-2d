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
    /// Experience carried away from this fight, and what is on you now.
    ///
    /// **Not levels.** A fight cannot level you any more: it pays into your
    /// pocket and a town is the only place that spends it. `levels` and `grew`
    /// used to be here and moved to [`Banking`], which is where they happen.
    pub carried: i32,
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
    // The class is a rule the fight has to know about, not a stat bundle, so it
    // goes in here rather than being folded into `player_stats`. A character
    // with no class passes an empty slice, which is exactly what
    // `simulate_at` does — so an unclassed fight is the same fight it was
    // before M5, and the golden fixture says so.
    let worn: Vec<crate::class::ClassDef> =
        game.character.class_def().into_iter().cloned().collect();
    Some(combat::simulate_holding(
        game.character.player_stats(),
        &game.character.combat_items(),
        spec,
        difficulty,
        &worn,
        0,
        game.character.start_with(),
    ))
}

/// The boss standing on a tile, if one is, and what beating it leaves.
///
/// Reads the map the player is on. A `World` is cheap to build here — this
/// runs once per settled fight, not per frame.
fn boss_at(game: &Game, at: [u8; 2]) -> Option<(String, Option<String>)> {
    let w = crate::data::map(&game.world.map_id(), Difficulty::Easy);
    let p = w.place_at(at[0], at[1])?;
    if p.kind != crate::world::PlaceKind::Boss {
        return None;
    }
    Some((p.id.clone(), p.drops.clone()))
}

/// A creature that gave up, and what that paid.
///
/// **Not a [`Settlement`].** A settlement has an `Outcome`, which is the answer
/// to "how did the fight go", and there was no fight. Two types rather than an
/// outcome variant, so nothing downstream can ask a rout what its log said.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rout {
    /// Canonical, like everything the engine matches on.
    pub creature: String,
    pub gold: i32,
    pub xp: i32,
    /// What is on you now, the same number a win reports.
    pub carried: i32,
    pub receipt: Vec<String>,
}

/// Meet something that will not fight you.
///
/// `Rule::Rout` is the one rule in the game that resolves an **encounter**
/// rather than a fight or a step, and this is why it is here rather than in
/// `combat`: a fight decided before its first tick is a fight the replay has to
/// draw, and there is nothing to draw. So the encounter is settled where it
/// stands, pays what a win pays, and says why.
///
/// Returns `None` when there is no encounter or nothing routs this creature,
/// which is the ordinary case and is not an error.
///
/// **A boss is never routed.** The same rule that looks a boss drop up by the
/// tile rather than by the creature: the thing standing at the end of a
/// corridor is the corridor's, and a set that walked past it would walk past
/// the key as well.
pub fn rout(game: &mut Game) -> Option<Rout> {
    let e = game.encounter.as_ref()?;
    let spec = spec(e)?;
    if boss_at(game, e.at).is_some() {
        return None;
    }
    if !crate::rule::routs(&game.character.rules(), &e.enemy) {
        return None;
    }
    let e = game.encounter.take()?;
    let difficulty = Difficulty::Easy;
    let gold = reward::bounty_for(Outcome::Victory, spec.bounty);
    let rating = crate::rating::creature_rating(spec, difficulty);
    let xp = reward::xp_for(Outcome::Victory, crate::progression::xp_for_rating(rating));

    let mut receipt =
        vec![format!("The {} will not come near you. Nothing was fought.", game.theme_name(spec.name))];
    game.character.gold += gold;
    receipt.push(format!("+{gold} Fnorp"));
    if xp > 0 {
        game.character.carry(xp);
        receipt.push(format!("+{xp} experience, carried"));
        receipt.push(format!(
            "{} on you. It is worth nothing until you bank it.",
            game.character.carried
        ));
    }
    game.world.bump("wins");
    game.world.bump("routs");
    // An errand that counts this creature counts it. A set that broke a town's
    // errand would be a reward that took something away.
    for name in crate::quest::on_victory(game, spec.name) {
        receipt.push(format!("Took a {name}."));
    }
    // **And no tiredness.** A fight takes 4% of you whatever happens in it;
    // this was not one, and a player will check.
    receipt.push(format!(
        "0% more tired. There was no fight to be tired from, and {}% of you is still missing.",
        game.character.fatigue
    ));
    let _ = e;
    Some(Rout {
        creature: spec.name.to_string(),
        gold,
        xp,
        carried: game.character.carried,
        receipt,
    })
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

    // **Every battle, won or lost.** Fatigue is what a fight costs whatever
    // happens in it: walking away from a hard one still leaves you the weaker
    // for having stood in it, and a rule that only tired the winner would make
    // losing the cheaper option.
    let before = game.character.fatigue;
    game.character.tire(crate::fatigue::PER_FIGHT);
    let tired = game.character.fatigue - before;

    match log.outcome {
        Outcome::Victory => {
            game.character.gold += gold;
            receipt.push(format!("+{gold} Fnorp"));
            // **Carried, not spent.** A win pays experience into your pocket
            // and nothing else: no level, no point, no row. A town is the only
            // place that turns it into any of those, and a defeat before you
            // reach one takes the lot.
            if xp > 0 {
                game.character.carry(xp);
                receipt.push(format!("+{xp} experience, carried"));
                receipt.push(format!(
                    "{} on you. It is worth nothing until you bank it.",
                    game.character.carried
                ));
            }
            game.world.bump("wins");
            // What the corpse leaves for an errand that asked for it. Gated on
            // the errand rather than on the creature: a bag filling with toad
            // eyes before anybody wanted one is litter.
            for name in crate::quest::on_victory(game, spec.name) {
                receipt.push(format!("Took a {name}."));
            }
            // **What a boss leaves behind.**
            //
            // Looked up by the tile the fight happened on rather than by the
            // creature's name: the same creature stands in a region's pool as
            // an ordinary encounter, and beating one in a field must not hand
            // over the key to anywhere. The place is what makes it a boss.
            if let Some((id, drop)) = boss_at(game, e.at) {
                if !game.world.answered.iter().any(|a| *a == id) {
                    game.world.answered.push(id);
                    if let Some(name) = drop {
                        game.character.give(&name);
                        receipt.push(format!("It was carrying {name}."));
                    }
                }
            }
        }
        Outcome::Defeat | Outcome::Stalemate => {
            game.world.bump("losses");
            // No bounty. The whole argument is in `reward.rs`; the short of it
            // is that upstream's reasoning held because a ladder is a corridor
            // and this is not one.
            receipt.push("No bounty. Nothing was beaten.".into());
            // Everything unbanked, gone. Not a share and not a penalty on the
            // total: what you had spent is what you are, and what you were
            // carrying is what you were going to be.
            let lost = game.character.drop_carried();
            if lost > 0 {
                receipt.push(format!("The {lost} experience you were carrying is gone."));
            }
            sent_home = Some(game.world.last_town.clone()).filter(|t| !t.is_empty());
            receipt.push(match &sent_home {
                Some(_) => "You wake up walking, and you have been walking a while.".into(),
                None => "You wake up where you fell.".into(),
            });
        }
    }

    if tired > 0 {
        receipt.push(format!(
            "{tired}% more tired. {}% of you is missing until you take something for it.",
            game.character.fatigue
        ));
    } else if game.character.fatigue >= crate::fatigue::CAP {
        receipt.push(format!(
            "You cannot get any more tired than this. {}% of you is missing.",
            game.character.fatigue
        ));
    }

    Some(Settlement {
        outcome: log.outcome,
        gold,
        xp,
        carried: game.character.carried,
        sent_home,
        receipt,
    })
}

/// What a town does with what you were carrying.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banking {
    /// What went in. Zero means there was nothing to bank.
    pub spent: i32,
    /// Every level crossed, in order.
    pub levels: Vec<u32>,
    /// Which grids grew, and by how much.
    pub grew: Vec<(String, u8)>,
    pub receipt: Vec<String>,
}

/// Turn what the character is carrying into levels.
///
/// **The only place a level happens.** A fight used to do this the instant the
/// experience was won, which made the walk home a formality; now the walk home
/// is the game. Everything a level-up used to print — the level, the point,
/// the row — prints here instead, because here is where it occurs.
///
/// Safe to call with nothing carried: it says so and changes nothing.
pub fn bank(game: &mut Game) -> Banking {
    let spent = game.character.carried;
    if spent <= 0 {
        return Banking {
            spent: 0,
            levels: Vec::new(),
            grew: Vec::new(),
            receipt: vec!["You are carrying nothing to spend.".into()],
        };
    }
    let levels = game.character.bank();
    let mut receipt = vec![format!("{spent} experience spent.")];
    for level in &levels {
        receipt.push(format!("Level {level}. One point to spend."));
    }
    let mut grew = Vec::new();
    if !levels.is_empty() {
        let granted = crate::data::skills().granted_rows(&game.character.skills_taken);
        for (slot, rows) in game.character.resize_boards(granted) {
            let name = format!("{slot:?}").to_lowercase();
            receipt.push(format!("+{rows} row on the {name} frame"));
            grew.push((name, rows));
        }
    }
    if levels.is_empty() {
        let (into, need) = crate::progression::progress(game.character.xp);
        receipt.push(format!("{into} of {need} towards the next level."));
    }
    Banking { spent, levels, grew, receipt }
}
