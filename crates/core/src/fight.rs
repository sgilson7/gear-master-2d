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
    // goes in here rather than being folded into `player_stats`.
    //
    // **Every class since M13, not the first.** A character holds up to three,
    // and a fight that read only the level-five fork would honour two thirds
    // of what a player paid for — which is the `Showstopper`-reaches-nothing
    // failure with an extra step. A character
    // with no class passes an empty slice, which is exactly what
    // `simulate_at` does — so an unclassed fight is the same fight it was
    // before M5, and the golden fixture says so.
    let worn: Vec<crate::class::ClassDef> = game.character.class_defs();
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

/// The mark that says this visit's golem has had its fight.
///
/// **In `answered` rather than in a counter**, and cleared when a survey opens
/// rather than when one closes: what it records is *this entry*, and an entry
/// begins at the gate. A counter would have been a second answer to a question
/// one boolean answers.
pub const GOLEM_SPENT: &str = "the-golem-has-been";

/// What the map you are standing on is being read through.
///
/// `SurveyMod::none()` everywhere but a surveyed map, so every caller can add
/// its fields without asking whether there is a survey on.
fn lens(game: &Game) -> crate::survey::SurveyMod {
    match &game.world.active_survey {
        Some((map, kind)) if *map == game.world.map_id() => crate::survey::mods_for(
            map,
            kind,
            game.character.assembled_items(),
        ),
        _ => crate::survey::SurveyMod::none(),
    }
}

/// The boss standing on a tile, if one is, and what beating it leaves.
///
/// Reads the map the player is on. A `World` is cheap to build here — this
/// runs once per settled fight, not per frame.
fn boss_at(game: &Game, at: [u8; 2]) -> Option<(String, Vec<String>, Vec<String>)> {
    // **As the game has left it**, not as the file has it: a boss can stand
    // under a lake that has drained, and the map that answers questions about
    // the file does not know about that.
    let w = crate::data::map_now(&game.world.map_id(), Difficulty::Easy, &game.world);
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
    // **Two ways an encounter is settled without a fight**, and they are one
    // mechanism because they are one thing: something happened that meant the
    // fight did not. The Rat King's Mandate is a creature that gives up; the
    // survey golem is a creature that met the golem instead of you.
    let mandate = crate::rule::routs(&game.character.rules(), &e.enemy);
    let golem = lens(game).golem && !game.world.answered.iter().any(|a| a == GOLEM_SPENT);
    if !mandate && !golem {
        return None;
    }
    let e = game.encounter.take()?;
    if golem && !mandate {
        // **One a visit**, which is the fallback `PLAN-M11.md` §8 row 6 named
        // in advance so that taking it would be a decision rather than a
        // retreat: the golem *handles one*. Written into `answered` and taken
        // out again when the survey opens, so it is a fact about this entry
        // rather than a counter.
        game.world.answered.push(GOLEM_SPENT.to_string());
    }
    let difficulty = Difficulty::Easy;
    // **The plain bounty, and no speed bonus.** `Showstopper` pays for winning
    // a fight quickly and there was no fight to be quick about — the same
    // reasoning that makes a rout cost no tiredness. A routed rat paying half
    // again for taking no time at all would be the one arrangement in the game
    // where a class is paid for something not happening.
    let gold = reward::bounty_for(Outcome::Victory, spec.bounty);
    let rating = crate::rating::creature_rating(spec, difficulty);
    let xp = reward::xp_for(Outcome::Victory, crate::progression::xp_for_rating(rating));

    let mut receipt = vec![if mandate {
        format!("The {} will not come near you. Nothing was fought.", game.theme_name(spec.name))
    } else {
        format!(
            "The golem gets to the {} first. You watch. Nothing was fought, by you.",
            game.theme_name(spec.name)
        )
    }];
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

/// A fight that happened and that nobody watched.
///
/// **The third way an encounter is settled without a screen**, and the one that
/// is not [`rout`]'s. A rout and a golem are one mechanism because they are one
/// thing — *something happened that meant the fight did not* — and this is the
/// opposite: **the fight did happen.** So it pays the speed bonus, rolls the
/// drops, ticks the order book and costs the four percent, all of which a rout
/// deliberately does none of.
///
/// Which is why it is [`run`] followed by [`settle`] and not a line of new
/// settlement code. *A second answer to what a win pays* is the mistake this
/// project has paid for six times, and the whole of what is new here is that
/// nothing opens.
///
/// Returns `None` — the ordinary case, and not an error — when there is no
/// encounter, when this creature is not marked, or when a boss is standing on
/// the tile.
///
/// **The boss refusal is the tile's and not the name's**, which is a divergence
/// from `PLAN-M15.md` §1.5 and is measured rather than argued: eight of the
/// nine creatures that stand on a boss tile also stand in a region pool, so a
/// refusal at the name would take seven ordinary field encounters off the menu
/// on behalf of a room the player has not reached. This is exactly where
/// [`rout`] puts the identical rule, and for the identical reason: the thing
/// standing at the end of a corridor is the corridor's.
pub fn instant(game: &mut Game, difficulty: Difficulty) -> Option<Settlement> {
    let e = game.encounter.as_ref()?;
    let spec = spec(e)?;
    if !game.world.instant.iter().any(|c| c == spec.name) {
        return None;
    }
    if boss_at(game, e.at).is_some() {
        return None;
    }
    let log = run(game, difficulty)?;
    let name = game.theme_name(spec.name);
    // **The receipt is the whole interface.** There is no screen, no replay and
    // no card, so the strip is the only place this fight is ever going to be
    // reported — and it has to carry a defeat as plainly as a win, because a
    // player who marked something and then out-levelled their own board finds
    // out here or not at all. TONE 6: the reversal lands flat, in the shortest
    // sentence available.
    let head = match log.outcome {
        Outcome::Victory => {
            format!("The {name} again. It went the way it goes, and you did not stop to watch.")
        }
        _ => format!("The {name} again. It did not go the way it goes, and you were not watching."),
    };
    let mut s = settle(game, &log, difficulty)?;
    s.receipt.insert(0, head);
    Some(s)
}

/// How many wins over one creature buy the right to stop watching.
///
/// The human's number, and it is a threshold rather than a curve: *"after
/// defeating a specific enemy 5 times, you can set them to instant battle"*.
pub const INSTANT_AFTER: u32 = 5;

/// What marking a creature costs you, in the engine's words.
///
/// **A spec, not prose** — TONE 13a, unthemed and with the numbers in it,
/// because somebody deciding whether to flip a switch is weighing what it does
/// rather than reading about it. It lives here rather than in the markup for
/// the reason `Node::line` is derived rather than written into a blurb: a
/// sentence beside the behaviour it describes cannot go stale, and the two
/// figures in it are read off the constants that decide them.
///
/// The last line is the one nobody would guess. Eight of the nine creatures
/// that stand on a boss tile also stand in a region pool, so a marked creature
/// settles in the field and is fought at the end of the corridor — which is
/// correct, and is surprising, and *a rule the player cannot see is a rule they
/// report as a bug.*
pub fn what_a_mark_costs() -> Vec<String> {
    vec![
        "It is not drawn. There is no replay, no log to read afterwards, and no way to slow it down."
            .to_string(),
        "You cannot walk away from one. A fight you would have fled is a fight you have had."
            .to_string(),
        format!(
            "It costs the same {}% as any other fight, and losing one takes everything you are carrying.",
            crate::fatigue::PER_FIGHT
        ),
        "A creature standing on a boss's tile is fought either way. The switch is about meeting one out there."
            .to_string(),
    ]
}

/// The counter a creature's **meetings** are kept under.
///
/// Sibling of [`beat_key`] and deliberately a second counter rather than a
/// reading of the first: **meeting is not beating.** A creature that killed you
/// four times is one you have met four times and beaten none, and the bestiary
/// is about what you have *seen*.
pub fn met_key(creature: &str) -> String {
    format!("met:{creature}")
}

/// The counter a creature's wins are kept under.
///
/// One function so the writer and every reader spell it the same way. A
/// `format!` at each end is two copies of a key, and a key that is two copies
/// is a counter that silently splits in half the day one of them is retyped.
pub fn beat_key(creature: &str) -> String {
    format!("beat:{creature}")
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
    // **How many times this creature has gone down, and it is counted here
    // because here is the one place a win is paid.** `settle` and `rout` both
    // arrive through this function, which is why it exists — and a rout counts
    // on purpose: it is a win, and somebody who has routed a rat five times
    // has met the rat five times.
    //
    // A counter rather than a list, and read rather than mirrored: which
    // creatures are *eligible* to be marked is `count(beat:…) >= FIVE`, worked
    // out fresh. **And it is read by something** — `Game::beaten` feeds the
    // menu, which is what keeps it from being `Outcome::Xp`'s counter, written
    // for four blocks into a total nothing ever consulted.
    game.world.bump(&beat_key(creature));
    for name in crate::quest::on_victory(game, creature) {
        receipt.push(format!("Took a {}.", game.theme_piece(&name)));
    }
    // **What an atlas is for**, and the only place the number lands. The draw
    // is taken either way; the survey moves the threshold, so the stream is a
    // function of the fights you had rather than of what you walked in with.
    let generous = lens(game).drops_per_mille;
    for name in
        crate::drops::roll_with(&crate::data::drops(), &mut game.rng, creature, generous)
    {
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
/// How many cells `Rule::Spread` works after a fight this long.
///
/// A turn is `combat::SPIN_EVERY_MS`, which is the clock every spinning item
/// in the game runs on — read rather than restated, so a rule that retuned the
/// spin retunes this. Zero without the rule, and zero for a fight shorter than
/// one turn.
fn spread_turns(game: &Game, log: &CombatLog) -> u32 {
    let every: u32 = game
        .character
        .rules()
        .iter()
        .filter_map(|r| match r {
            crate::rule::Rule::Spread { every_turns } => Some(*every_turns),
            _ => None,
        })
        .min()
        .unwrap_or(0);
    if every == 0 {
        return 0;
    }
    let turns = log.duration_ms / crate::combat::SPIN_EVERY_MS.max(1);
    turns / every
}

pub fn settle(game: &mut Game, log: &CombatLog, difficulty: Difficulty) -> Option<Settlement> {
    let e = game.encounter.take()?;
    let spec = spec(&e)?;

    // **What the class adds, if it adds anything.** A settlement rule is read
    // where a settlement happens; `combat` ignores these on purpose and would
    // have gone on ignoring them for ever.
    let worn: Vec<crate::class::ClassDef> = game.character.class_defs();
    let plain = reward::bounty_for(log.outcome, spec.bounty);
    // **What the board and the corpse looked like at the bell.** Two of the
    // ten experts settle here and neither can be answered from the outcome and
    // the clock alone; see `reward::AtTheBell`, whose `Default` is a packed
    // board and an uncursed corpse.
    let at = reward::AtTheBell {
        empty_frames: game.character.empty_frames(),
        curses_standing: log.curse_bill.standing,
        curse_kinds: log.curse_bill.kinds.len() as u32,
        curses_expired: log.curse_bill.expired,
        streak: game.character.fast_wins,
        // **Read off the log rather than inferred from the outcome**, which is
        // the rule this file has kept since the first receipt: an unmaking is
        // a `Victory` like any other and only the log says how it happened.
        unmade: log
            .entries
            .iter()
            .any(|e| matches!(e.event, crate::combat::Event::Unmade { .. })),
    };
    let gold = reward::bounty_with_class(log.outcome, spec.bounty, &worn, log.duration_ms, at);
    let rating = crate::rating::creature_rating(spec, difficulty);
    // **And what an atlas pays.** On the experience only: the purse is the
    // class's argument and a survey has no business in it.
    let read = lens(game);
    let xp = crate::survey::shift(
        reward::xp_for(log.outcome, crate::progression::xp_for_rating(rating)),
        read.xp_pct,
    );

    let mut receipt = Vec::new();
    let mut sent_home = None;

    // **Every battle, won or lost.** Fatigue is what a fight costs whatever
    // happens in it: walking away from a hard one still leaves you the weaker
    // for having stood in it, and a rule that only tired the winner would make
    // losing the cheaper option.
    let before = game.character.fatigue;
    game.character.tire(crate::fatigue::PER_FIGHT);
    let tired = game.character.fatigue - before;

    // **An order ticks exactly when the character gets tired.** M12.2, and the
    // placement is the whole argument: `PLAN-M12.md` §6 entry 3 warns about "a
    // fight that resolves without ticking, which would make some fights count
    // and some not, invisibly", and the answer is not to go and find every
    // place a fight can end. It is to put the clock beside the one line that
    // already means *a fight happened, won or lost* — the line directly above
    // — so the two cannot drift apart.
    //
    // **A rout deliberately does not reach here**, and must not. `fight::rout`
    // settles an encounter with no fight in it and costs no tiredness, because
    // nothing was fought; a clock that counted routs would let the Rat King's
    // Mandate and the survey golem pace an order out on creatures that decline
    // to fight, which is the walking-in-circles loophole the frame chose
    // fights over steps to avoid, arriving through a different door.
    for c in &mut game.world.commissions {
        c.fights_left = c.fights_left.saturating_sub(1);
    }

    // **The underlay is worked, once the fight is over.** `Rule::Spread` is
    // the only thing in the game that changes an enchantment layer, and it
    // happens here rather than in the tick because a fight that wrote to the
    // character is a fight a mid-fight save could not carry a creature name
    // and a tile for. How many turns it had is `duration_ms` over the spin's
    // own second, so nothing about the count is invented here either.
    //
    // Won or lost, like the tiredness two lines above and for the same reason:
    // the frame turned for as long as the fight lasted, whatever it ended in.
    // **What crosses out of this fight and into the next.** Two things do, and
    // they are the only two in the block: how many fast wins are behind you,
    // and which permanent curses followed you. Both are written here, beside
    // the tiredness, because this is the one line in the game that means *a
    // fight happened, won or lost* — which is the argument M12.2's order clock
    // already made and the reason it sits in the same place.
    //
    // **The rule is the character's.** It was written here and touches nothing
    // but them, and a test asking *what does this fight leave behind* must not
    // have to build a `Game` around a board to find out — `told` is the one
    // knob in the block that crosses a fight boundary, so the only measurement
    // that can see it is one that crosses the same boundary.
    game.character.carry_out_of(log);

    let spread = spread_turns(game, log);
    if spread > 0 {
        let worked = game.character.spread_underlay(spread);
        if worked > 0 {
            receipt.push(format!(
                "{worked} cell{} of bare frame worked over",
                if worked == 1 { "" } else { "s" }
            ));
        }
    }

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
            if let Some((id, drops, prose)) = boss_at(game, e.at) {
                if !game.world.answered.iter().any(|a| *a == id) {
                    game.world.answered.push(id);
                    for name in drops {
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

/// The names of places this level made visible, in map order.
///
/// Reads every shipped map rather than the one underfoot: a level opens what it
/// opens, and being told about it while standing somewhere else is better than
/// not being told.
fn opened_by(was: u32, now: u32) -> Vec<String> {
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

/// Turn what the character is carrying into levels.
///
/// **The only place a level happens.** A fight used to do this the instant the
/// experience was won, which made the walk home a formality; now the walk home
/// is the game. Everything a level-up prints — the level, the point, what the
/// level opened — prints here, because here is where it occurs.
///
/// It no longer prints a row. M12.3 retired the rotation that grew one grid a
/// level; `grew` is still on the receipt because an errand or a node can still
/// pay one, and that is reported where it happens.
///
/// Safe to call with nothing carried: it says so and changes nothing.
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
        // **Both sources, since M12.3.** The tree's rows and the errands'.
        // A board can still grow here — a questline finished on the road pays
        // a row and this is the next time anything resizes — but a *level*
        // grows nothing, so this line has stopped being the level's receipt
        // and become the receipt of whatever was earned.
        let granted = {
            let g: &Game = game;
            g.granted_rows()
        };
        for (slot, rows) in game.character.resize_boards(granted) {
            let name = format!("{slot:?}").to_lowercase();
            receipt.push(format!("+{rows} row on the {name} frame"));
            grew.push((name, rows));
        }
        // **And where the next one comes from.** The level banner used to
        // promise a row and now it points at the place one is bought, which
        // is `PLAN-M12.md` §3 M12.3's "the screens" in one sentence: a system
        // nobody is told about is a bug report.
        if grew.is_empty() {
            receipt.push("A row is bought at the tree, or finished off an errand.".into());
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
    for id in opened_by(was, game.character.level()) {
        receipt.push(format!("Somebody is at {id} who was not there before."));
    }
    Banking { spent, levels, grew, receipt }
}
