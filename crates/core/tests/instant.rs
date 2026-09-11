//! A fight you have already had — the tally, and what may be marked.
//!
//! M15.0. Nothing here is a screen: this file is about whether the engine can
//! *count* the thing the menu is going to list, which is the order the block
//! runs in for a reason — a screen listing something the engine cannot count is
//! a screen built on a guess.

use gm2d_core::combat::Difficulty;
use gm2d_core::fight::{self, Encounter, INSTANT_AFTER};
use gm2d_core::game::Game;
use gm2d_core::piece::SlotKind;

mod common;

const D: Difficulty = Difficulty::Easy;

/// A game standing on the pit's own tile with an encounter rolled.
fn meeting(enemy: &str) -> Game {
    let mut g = Game::new(9, "td");
    g.world = gm2d_core::world::WorldState::at_start(&gm2d_core::data::world(D));
    g.encounter = Some(Encounter { enemy: enemy.into(), at: g.world.at });
    g
}

// ------------------------------------------------------------- the counting

/// A win counts, and it counts under the creature that lost it.
///
/// **Through `settle`, not through a hand-written bump**, because the counter's
/// whole legitimacy is that `pay_a_win` is the one place a win is paid. A test
/// that bumped the counter itself would be testing `WorldState::bump`.
#[test]
fn a_win_is_counted_by_creature() {
    let mut g = meeting("Cave Rat");
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    let log = fight::run(&g, D).expect("there is an encounter to run");
    assert_eq!(
        log.outcome,
        gm2d_core::combat::Outcome::Victory,
        "the board this test is built on cannot beat a Cave Rat, so it proves nothing"
    );
    fight::settle(&mut g, &log, D).expect("a fight that happened settles");
    assert_eq!(g.beaten("Cave Rat"), 1);
    assert_eq!(g.beaten("Bog Toad"), 0, "the win went into somebody else's column");
}

/// A rout counts, because it is a win.
///
/// A player who has routed a rat five times has met the rat five times, and
/// the whole of the feature is *a fight you have already had*. Both paths go
/// through `pay_a_win`, which is why this is one line rather than a second
/// mechanism.
#[test]
fn a_rout_counts_too() {
    let mut g = meeting("Cave Rat");
    g.character = common::bench();
    for (name, x, y) in
        [("Ratskin Material", 0u8, 0u8), ("Ratskin Mold", 2, 0), ("Rat Signet", 4, 0)]
    {
        let id = common::spare(&g.character, name);
        g.character.registry.set_rotation(id, 0);
        g.character.equip(id, SlotKind::Gloves, x, y).expect("the Mandate seats");
    }
    gm2d_core::loadout::lock_assembled_in(
        &mut g.character.loadout,
        &g.character.registry,
        SlotKind::Gloves,
    );
    assert!(
        gm2d_core::rule::routs(&g.character.rules(), "Cave Rat"),
        "the Mandate is not assembled, so nothing routs and this proves nothing"
    );
    fight::rout(&mut g).expect("the Mandate routs a Cave Rat");
    assert_eq!(g.beaten("Cave Rat"), 1, "a rout is a win and pays what a win pays");
}

// --------------------------------------------------------------- the marking

/// Nothing may be marked that has not been beaten five times, and the refusal
/// says how far off you are.
///
/// **The count is in the sentence**, because a button that greys with no reason
/// is a button reported as a bug — this project has written that down six
/// times.
#[test]
fn nothing_under_five_can_be_marked() {
    let mut g = meeting("Cave Rat");
    for had in 0..INSTANT_AFTER {
        let why = g.mark_instant("Cave Rat").expect_err("marked with too few wins");
        assert!(
            why.contains(&had.to_string()) && why.contains(&INSTANT_AFTER.to_string()),
            "the refusal does not count: {why}"
        );
        assert!(!g.is_instant("Cave Rat"), "a refused mark went on anyway");
        g.world.bump(&fight::beat_key("Cave Rat"));
    }
    g.mark_instant("Cave Rat").expect("five wins is the price and it has been paid");
    assert!(g.is_instant("Cave Rat"));
    g.unmark_instant("Cave Rat");
    assert!(!g.is_instant("Cave Rat"), "taking the mark off left it on");
}

/// The list is the count and nothing else, and it is worked out fresh.
#[test]
fn the_menu_lists_what_has_been_beaten_five_times() {
    let mut g = meeting("Cave Rat");
    assert!(g.instant_candidates().is_empty(), "a new game has beaten nothing");
    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER - 1);
    g.world.add(&fight::beat_key("Bog Toad"), INSTANT_AFTER + 3);
    let lines = g.instant_candidates();
    assert_eq!(lines.len(), 1, "four wins is not five: {lines:?}");
    assert_eq!(lines[0].canonical, "Bog Toad");
    assert_eq!(lines[0].beaten, INSTANT_AFTER + 3);
    assert!(!lines[0].marked);
    assert!(!lines[0].name.is_empty(), "the line has no themed name to print");

    // **A counter naming a creature this build has not got is dropped**, not
    // listed: there is nothing to fight and nothing to theme it with.
    g.world.add(&fight::beat_key("A Thing From Another Build"), 40);
    assert_eq!(g.instant_candidates().len(), 1, "a stranger reached the menu");
}

/// Marking is idempotent, because the switch is a switch.
#[test]
fn marking_twice_marks_once() {
    let mut g = meeting("Cave Rat");
    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
    g.mark_instant("Cave Rat").unwrap();
    g.mark_instant("Cave Rat").unwrap();
    assert_eq!(g.world.instant.len(), 1, "one creature, two marks");
}

// ------------------------------------------------------------------ the save

/// An empty mark list and a full one both survive a round trip, and neither
/// moves the fingerprint.
#[test]
fn the_mark_survives_a_round_trip_and_makes_no_seam() {
    let before = gm2d_core::save::catalog_fingerprint();

    let mut g = meeting("Cave Rat");
    g.encounter = None;
    let plain = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&plain).expect("and reads");
    assert!(back.world.instant.is_empty());

    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
    g.mark_instant("Cave Rat").unwrap();
    let marked = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&marked).expect("and reads");
    assert_eq!(back.world.instant, vec!["Cave Rat".to_string()]);
    assert_eq!(back.beaten("Cave Rat"), INSTANT_AFTER, "the counter did not come back");

    assert_eq!(
        before,
        gm2d_core::save::catalog_fingerprint(),
        "M15 moved the catalogue, and it has no business doing that"
    );
}

/// A file written before the field existed opens with nobody marked.
///
/// **This is the no-seam claim stated as a check** rather than as a sentence in
/// a commit: `#[serde(default)]` is only a promise until something plants a
/// file without the key.
#[test]
fn a_save_written_before_the_mark_opens_with_nobody_marked() {
    let mut g = meeting("Cave Rat");
    g.encounter = None;
    let json = gm2d_core::save::save(&g);
    let mut v: serde_json::Value = serde_json::from_str(&json).expect("is json");
    let w = v.get_mut("state").and_then(|s| s.get_mut("world")).expect("the save has a world");
    assert!(
        w.as_object_mut().expect("an object").remove("instant").is_some(),
        "there is no `instant` key to take out, so this check proves nothing"
    );
    let back = gm2d_core::save::load(&serde_json::to_string(&v).unwrap())
        .expect("a file with no `instant` key still opens");
    assert!(back.world.instant.is_empty());
}

// ------------------------------------------------- the battle nobody watches

/// **Everything it pays and everything it costs, against a fought fight.**
///
/// M15.1's whole claim is that an instant battle *is* `run` then `settle`, so
/// the honest check is to run the same encounter twice out of the same game and
/// compare the two settlements field for field — the purse, the experience, the
/// tiredness, the drops, the errand tally and the order clock.
///
/// **Not against constants.** `an_instant_battle_costs_what_a_fight_costs`
/// asked against `fatigue::PER_FIGHT` in a first draft, which is a second copy
/// of `PER_FIGHT` in a file that has no business holding one — and it would
/// have gone on passing if `settle` stopped tiring you at all, because it would
/// have been comparing the constant with itself.
#[test]
fn an_instant_battle_pays_what_a_fought_one_pays() {
    let base = {
        let mut g = meeting("Cave Rat");
        g.character = common::geared_from(&["the-end-of-all-gears"]);
        g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
        g
    };

    // The fight, watched.
    let mut watched = base.clone();
    let log = fight::run(&watched, D).expect("there is something to fight");
    assert_eq!(
        log.outcome,
        gm2d_core::combat::Outcome::Victory,
        "the fixture loses to a Cave Rat, so this compares two defeats"
    );
    let a = fight::settle(&mut watched, &log, D).expect("it settles");

    // The same fight, not watched.
    let mut skipped = base.clone();
    skipped.mark_instant("Cave Rat").expect("five wins is the price and it has been paid");
    let b = fight::instant(&mut skipped, D).expect("a marked creature settles where it stands");

    assert_eq!(b.outcome, a.outcome, "a fight that was not watched went differently");
    assert_eq!(b.gold, a.gold, "the purse");
    assert_eq!(b.xp, a.xp, "the experience");
    assert_eq!(b.carried, a.carried, "what is on you");
    assert_eq!(
        skipped.character.fatigue, watched.character.fatigue,
        "an instant battle did not tire you the way a fight does"
    );
    assert!(
        skipped.character.fatigue > base.character.fatigue,
        "neither fight tired anybody, so the comparison above is two zeroes"
    );
    assert_eq!(
        skipped.character.gold, watched.character.gold,
        "the bounty landed differently"
    );
    // **The drops, which are a roll off the same stream.** Two games from one
    // seed take the same draw, so the bag is the check that the roll happened
    // at all rather than being skipped along with the screen.
    assert_eq!(
        skipped.character.owned.len(),
        watched.character.owned.len(),
        "the drop roll went differently"
    );
    assert_eq!(skipped.rng.state(), watched.rng.state(), "the stream is in a different place");

    // **And the receipt says which fight this was**, because it is the only
    // thing the player is ever going to see of it.
    assert!(
        b.receipt.first().map(|l| l.contains("again")).unwrap_or(false),
        "the receipt does not open by saying this is one you have had: {:?}",
        b.receipt.first()
    );
    assert!(
        b.receipt.len() > a.receipt.len(),
        "the instant receipt says no more than the watched one, and it is the only screen"
    );
}

/// The order book ticks, because a fight happened.
///
/// **The mirror of `a_rout_is_not_a_fight_and_does_not_tick`.** A rout must not
/// pace an order out on creatures that decline to fight; an instant battle
/// must, because it fought them. Both are settled without a screen and the
/// difference between them is this line.
#[test]
fn an_instant_battle_is_a_fight_and_does_tick() {
    let shops = gm2d_core::data::shops();
    let o = gm2d_core::shop::commissions(&shops, "the-end-of-all-gears")
        .into_iter()
        .next()
        .expect("the pit takes orders");
    let mut g = meeting("Cave Rat");
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    g.character.gold = 50_000;
    g.world.last_town = "the-end-of-all-gears".into();
    g.order("the-end-of-all-gears", o.index).expect("the order is placed");
    let was = g.world.commissions[0].fights_left;
    assert!(was > 0, "the order arrived already, so nothing can tick");

    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER);
    g.mark_instant("Cave Rat").unwrap();
    fight::instant(&mut g, D).expect("it settles");
    assert_eq!(
        g.world.commissions[0].fights_left,
        was - 1,
        "an instant battle did not move the order along, and a fight happened"
    );
}

/// A defeat is still a defeat, and it still takes what you were carrying.
///
/// **This is what the feature is paid for with.** `PLAN-M15.md` §5 decision 4
/// asks whether an instant battle may be lost; the alternative is refusing to
/// settle a fight the simulation lost, which is a lie about a fight that
/// happened. The receipt has to carry it, because there is no screen to see it
/// coming on.
#[test]
fn an_instant_defeat_still_costs_you() {
    let mut g = meeting("Rust Colossus");
    // A bare bench against the deepest thing on the first map.
    g.character = common::bench();
    g.character.carry(400);
    g.world.last_town = "the-end-of-all-gears".into();
    g.world.add(&fight::beat_key("Rust Colossus"), INSTANT_AFTER);
    g.mark_instant("Rust Colossus").unwrap();

    let s = fight::instant(&mut g, D).expect("it settles");
    assert_eq!(s.outcome, gm2d_core::combat::Outcome::Defeat, "an empty board won");
    assert_eq!(g.character.carried, 0, "a lost instant battle kept the pocket");
    assert_eq!(s.sent_home.as_deref(), Some("the-end-of-all-gears"), "it did not send you home");
    let said = s.receipt.join(" ");
    assert!(
        said.contains("did not go the way"),
        "the receipt does not say the fight was lost: {said:?}"
    );
    assert!(said.contains("walking"), "nothing on the strip says you were carried off: {said:?}");
}

/// Nothing is instant that has not been marked.
#[test]
fn an_unmarked_creature_still_opens_the_screen() {
    let mut g = meeting("Cave Rat");
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    g.world.add(&fight::beat_key("Cave Rat"), INSTANT_AFTER + 20);
    assert!(
        fight::instant(&mut g, D).is_none(),
        "twenty wins settled a fight nobody asked to skip"
    );
    assert!(g.encounter.is_some(), "the encounter was taken by a refusal");
}

/// **A boss is never instant, and the refusal is the tile's.**
///
/// Eight of the nine creatures that stand on a boss tile also stand in a region
/// pool, so this is asked twice of one creature: on the tile it is the end of a
/// dungeon and the whole of it is the fight; in a field it is an encounter like
/// any other. `fight::rout` puts the identical rule in the identical place.
#[test]
fn a_boss_is_never_instant() {
    let boss = gm2d_core::data::map("the-great-gear-cave", D)
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Boss)
        .cloned()
        .expect("the cave has a boss");
    let creature = boss.creature.clone().expect("the boss is somebody");

    let mut g = Game::new(9, "td");
    g.character = common::geared_from(&["the-end-of-all-gears"]);
    g.world.map = "the-great-gear-cave".into();
    g.world.at = boss.at;
    g.world.add(&fight::beat_key(&creature), INSTANT_AFTER);
    g.mark_instant(&creature).expect("a boss creature may be marked; it stands in pools too");

    g.encounter = Some(Encounter { enemy: creature.clone(), at: boss.at });
    assert!(
        fight::instant(&mut g, D).is_none(),
        "the thing at the end of the corridor settled without a screen"
    );
    assert!(g.encounter.is_some(), "and the refusal took the encounter with it");

    // **The same creature, in a field.** This is the half a name-level refusal
    // would have got wrong, and it is seven creatures wide.
    g.world.map = "west-bambulon".into();
    g.world.at = [4, 4];
    g.encounter = Some(Encounter { enemy: creature.clone(), at: [4, 4] });
    assert!(
        fight::instant(&mut g, D).is_some(),
        "{creature} stands in a region pool and could never be skipped anywhere"
    );
}

/// **A pocket is a boss tile, and six of them on one floor are still six tiles.**
///
/// `SECOND-ORDER-M16.md` row 12: the Reefs' six sinkhole pockets are
/// `PlaceKind::Boss` with no drops, which makes each a **certainty** rather
/// than a 260‰ roll — and that took the game's boss-tile count from nine to
/// fifteen under M15.1's rule that *the boss refusal is the tile's and not the
/// name's*. The row asks whether marking something met in a pocket reads
/// wrong.
///
/// It does not, and the two halves are separate:
///
/// 1. **A pocket refuses, exactly as the plate at the bottom does.** It is a
///    boss tile, so `instant` will not settle it — you fight what is in a hole
///    you fell into.
/// 2. **And it cannot be farmed for the mark.** Three of the six creatures
///    stand in two pockets each and three in one, so the whole floor is worth
///    at most **two** of the five wins a mark costs — and each pocket is spent
///    the first time, because a boss writes its own tile id into `answered`.
///    A player who marks the Iron Abbot marked it by meeting it in the field,
///    which is what the mark is supposed to mean.
#[test]
fn a_pocket_is_a_boss_tile_and_cannot_be_farmed_for_a_mark() {
    let floor = gm2d_core::data::map("the-reefs-1", D);
    let pockets: Vec<_> = floor
        .places
        .iter()
        .filter(|p| p.kind == gm2d_core::world::PlaceKind::Boss)
        .cloned()
        .collect();
    assert_eq!(pockets.len(), 6, "the Flat Below has {} pockets", pockets.len());

    // **At most two of the five.** Counted rather than asserted per creature,
    // because the floor gaining a seventh pocket is exactly the change that
    // should land here.
    let mut most = std::collections::BTreeMap::new();
    for p in &pockets {
        let who = p.creature.clone().expect("a pocket has somebody in it");
        *most.entry(who).or_insert(0u32) += 1;
    }
    let worst = *most.values().max().expect("six pockets hold somebody");
    assert!(
        worst < INSTANT_AFTER,
        "one creature is in {worst} pockets of a floor and a mark costs {INSTANT_AFTER} - \
         the whole floor is a shortcut to marking it"
    );

    // And each of them refuses, the way the plate at the bottom does.
    for p in &pockets {
        let creature = p.creature.clone().expect("a pocket has somebody in it");
        let mut g = Game::new(9, "td");
        g.character = common::geared_from(&["the-end-of-all-gears"]);
        g.world.map = "the-reefs-1".into();
        g.world.at = p.at;
        g.world.add(&fight::beat_key(&creature), INSTANT_AFTER);
        g.mark_instant(&creature).expect("every pocket creature stands in pools too");
        g.encounter = Some(Encounter { enemy: creature.clone(), at: p.at });
        assert!(
            fight::instant(&mut g, D).is_none(),
            "{}: the thing in the hole settled without a screen",
            p.id
        );
    }
}
