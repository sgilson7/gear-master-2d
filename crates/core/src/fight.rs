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
fn boss_at(game: &Game, at: [u8; 2]) -> Option<(String, Option<String>, Vec<String>)> {
    let w = crate::data::map(&game.world.map_id(), Difficulty::Easy);
    let p = w.place_at(at[0], at[1])?;
    if p.kind != crate::world::PlaceKind::Boss {
        return None;
    }
    Some((p.id.clone(), p.drops.clone(), p.prose.clone()))
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
    // **The plain bounty, and no speed bonus.** `Showstopper` pays for winning
    // a fight quickly and there was no fight to be quick about — the same
    // reasoning that makes a rout cost no tiredness. A routed rat paying half
    // again for taking no time at all would be the one arrangement in the game
    // where a class is paid for something not happening.
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
    // An errand that counts this creature counts it, and it still leaves what
    // it leaves. A set that broke a town's errand would be a reward that took
    // something away, and one that stopped the drops would be a set that shut
    // the door behind itself.
    pay_a_win(game, spec.name, &mut receipt);
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

/// What a beaten creature leaves, whether it was fought or routed.
///
/// One function rather than two lists, because a rout pays what a win pays and
/// two copies of "what a win pays" is two answers to one question. Both halves
/// are gated, and on different things:
///
/// - **An errand's tally is gated on the errand**, not on the creature. A bag
///   filling with toad eyes before anybody wanted one is litter.
/// - **A drop is gated on nothing but the roll**, and refused afterwards if the
///   piece is already in the bag. A set is three specific pieces and not three
///   of a kind, and the refusal is *after* the roll on purpose: skipping the
///   draw would make the stream a function of what the player is carrying
///   rather than of the fights they had.
fn pay_a_win(game: &mut Game, creature: &'static str, receipt: &mut Vec<String>) {
    for name in crate::quest::on_victory(game, creature) {
        receipt.push(format!("Took a {}.", game.theme_piece(&name)));
    }
    for name in crate::drops::roll(&crate::data::drops(), &mut game.rng, creature) {
        if game.character.holds(&name) {
            continue;
        }
        // Its own voice. The errand's line is "Took a ...", which is somebody
        // collecting what they were sent for, and the boss's is "It was
        // carrying ...", which is a thing that was always going to be there.
        // This one is luck and should read like it.
        receipt.push(format!("It had a {} on it. They do not usually.", game.theme_piece(&name)));
        game.character.give(&name);
    }
}

/// Bank the result and clear the encounter.
///
/// Idempotent in the sense that matters: with no encounter it does nothing and
/// says so, so a page that settles twice does not pay twice.
pub fn settle(game: &mut Game, log: &CombatLog, difficulty: Difficulty) -> Option<Settlement> {
    let e = game.encounter.take()?;
    let spec = spec(&e)?;

    // **What the class adds, if it adds anything.** A settlement rule is read
    // where a settlement happens; `combat` ignores these on purpose and would
    // have gone on ignoring them for ever.
    let worn: Vec<crate::class::ClassDef> =
        game.character.class_def().into_iter().cloned().collect();
    let plain = reward::bounty_for(log.outcome, spec.bounty);
    let gold = reward::bounty_with_class(log.outcome, spec.bounty, &worn, log.duration_ms);
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
            // Two facts and two lines: what it was worth, and what being quick
            // about it was worth. One number would have hidden the whole of
            // what the class does — *a derived number needs somewhere it is
            // shown*, and this is the only place this one appears.
            receipt.push(format!("+{plain} Fnorp"));
            if gold > plain {
                receipt.push(format!(
                    "+{} more for the speed of it. {:.1}s.",
                    gold - plain,
                    log.duration_ms as f32 / 1000.0
                ));
            }
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
            pay_a_win(game, spec.name, &mut receipt);
            // **What a boss leaves behind.**
            //
            // Looked up by the tile the fight happened on rather than by the
            // creature's name: the same creature stands in a region's pool as
            // an ordinary encounter, and beating one in a field must not hand
            // over the key to anywhere. The place is what makes it a boss.
            if let Some((id, drop, prose)) = boss_at(game, e.at) {
                if !game.world.answered.iter().any(|a| *a == id) {
                    game.world.answered.push(id);
                    if let Some(name) = drop {
                        receipt.push(format!("It was carrying {}.", game.theme_piece(&name)));
                        game.character.give(&name);
                    }
                    // **What the place says when it happens.** A boss on a
                    // tower floor is the floor coming down, and the paragraph
                    // about that is content — it is on the place, in the map
                    // file, and it counts the floors that are left because the
                    // order they come down in is fixed and written.
                    receipt.extend(prose);
                }
            }
            // **A floor is one sitting, and beating it ends the sitting.**
            //
            // The kick is a position write and not a death: nothing is lost,
            // the walk out is not part of the budget, and the next time you go
            // in it is a different map. Here rather than in the shim because
            // "clearing a floor puts you outside" is a rule, and a rule decided
            // in the shim is a rule the fast suite cannot reach.
            crate::world::leave_the_sitting(&mut game.world, difficulty);
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
/// The names of places this level made visible, in map order.
///
/// Reads every shipped map rather than the one underfoot: a level opens what it
/// opens, and being told about it while standing somewhere else is better than
/// not being told.
fn opened_by(game: &Game, was: u32, now: u32) -> Vec<String> {
    if now <= was {
        return Vec::new();
    }
    let mut out = Vec::new();
    for w in crate::data::all_maps(Difficulty::Easy) {
        for p in &w.places {
            let Some(need) = p.hidden_until_level else { continue };
            if need > was && need <= now {
                out.push(if p.name.is_empty() { p.id.clone() } else { p.name.clone() });
            }
        }
    }
    out
}

pub fn bank(game: &mut Game) -> Banking {
    let was = game.character.level();
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
    // **What the level opened.** A place can be hidden until one since M10.0,
    // and the map redraws — but a redraw is not a sentence, and a man who
    // appears on a road thirty tiles away is a man nobody finds. The
    // playthrough reached level twelve and never met him.
    //
    // Core's, because *which* places a level opens is a rule; the page prints
    // what it is told.
    for id in opened_by(game, was, game.character.level()) {
        receipt.push(format!("Somebody is at {id} who was not there before."));
    }
    Banking { spent, levels, grew, receipt }
}
