//! What a fight costs you beyond the fight.
//!
//! Health has reset at every bell since M0, which is why the town had nothing
//! to restore and why there was never a reason to turn round. Fatigue is the
//! thing an expedition actually spends, so these are the tests that keep it
//! from being either a formality or a wall.

use gm2d_core::character::Character;
use gm2d_core::combat::{simulate_at, Difficulty, MonsterSpec, Outcome};
use gm2d_core::data;
use gm2d_core::fatigue::{self, worn, CAP, PER_FIGHT};
use gm2d_core::fight;
use gm2d_core::game::Game;

const D: Difficulty = Difficulty::Easy;

fn creature(name: &str) -> &'static MonsterSpec {
    gm2d_core::combat::LADDER.iter().find(|s| s.name == name).expect("a creature")
}

fn ready() -> Game {
    let mut g = Game::new(5, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.character.apply_preset();
    g
}

#[test]
fn a_fresh_character_is_not_tired() {
    let c = Character::starting();
    assert_eq!(c.fatigue, 0);
    assert_eq!(c.player_stats().health, c.rested_stats().health);
}

/// **Every battle, won or lost.**
///
/// A rule that only tired the winner would make losing the cheaper option,
/// which is the opposite of what this is for.
#[test]
fn every_fight_tires_you_whichever_way_it_goes() {
    for (who, want) in [("Cave Rat", Outcome::Victory), ("Francis", Outcome::Defeat)] {
        let mut g = ready();
        g.encounter = Some(fight::Encounter { enemy: who.into(), at: g.world.at });
        let log = fight::run(&g, D).unwrap();
        assert_eq!(log.outcome, want, "{who} went the other way");
        let before = g.character.fatigue;
        let s = fight::settle(&mut g, &log, D).unwrap();
        assert_eq!(g.character.fatigue, before + PER_FIGHT, "{who} cost no tiredness");
        assert!(
            s.receipt.iter().any(|l| l.contains("tired")),
            "the receipt never mentions it: {:?}",
            s.receipt
        );
    }
}

/// It wears the maximum down, and the fight is fought at the worn number.
#[test]
fn tiredness_comes_off_the_maximum_and_the_fight_feels_it() {
    let mut c = Character::starting();
    c.apply_preset();
    let rested = c.rested_stats().health;
    c.tire(25);
    assert_eq!(c.player_stats().health, worn(rested, 25));
    assert!(c.player_stats().health < rested, "tiredness did not take anything");
    // And it is the worn number the fight runs on.
    let tired_log = simulate_at(c.player_stats(), &c.combat_items(), creature("Bog Toad"), D);
    let mut fresh = Character::starting();
    fresh.apply_preset();
    let fresh_log = simulate_at(fresh.player_stats(), &fresh.combat_items(), creature("Bog Toad"), D);
    assert!(
        tired_log.player.max_health < fresh_log.player.max_health,
        "a tired character walked into the fight at full height"
    );
}

/// It stops somewhere. A maximum that can reach zero is a character who cannot
/// fight and cannot mend, which is a game over with no screen for it.
#[test]
fn tiredness_has_a_floor_and_never_reaches_it() {
    let mut c = Character::starting();
    for _ in 0..200 {
        c.tire(PER_FIGHT);
    }
    assert_eq!(c.fatigue, CAP);
    assert!(c.player_stats().health > 0, "worn all the way to nothing");
    assert!(
        c.player_stats().health >= c.rested_stats().health * (100 - CAP) / 100,
        "worn past the cap"
    );
}

/// **The budget.** Four percent is enough that a fifth fight is a decision and
/// not so much that the second is a coin flip.
///
/// Walked against the pit rather than asserted about the constant, so moving
/// the creatures moves the answer and this notices.
#[test]
fn a_full_expedition_is_a_budget_and_not_a_wall() {
    let mut c = Character::starting();
    c.apply_preset();
    let rat = creature("Cave Rat");
    let mut won = 0;
    for fight_no in 0..12 {
        let log = simulate_at(c.player_stats(), &c.combat_items(), rat, D);
        if log.outcome == Outcome::Victory {
            won += 1;
        }
        c.tire(PER_FIGHT);
        // The second fight has to still be winnable, or one bad tile ends the
        // trip and the mechanic is a wall.
        if fight_no == 1 {
            assert_eq!(log.outcome, Outcome::Victory, "the second fight of a trip is unwinnable");
        }
    }
    assert!(won >= 6, "only {won} of twelve pit fights survivable — the wear is too steep");
    assert!(
        c.fatigue >= 30,
        "twelve fights left the character {}% tired, which is not a budget",
        c.fatigue
    );
}

/// A restorative takes tiredness off, is spent doing it, and reports what it
/// actually used rather than what the tin says.
#[test]
fn a_restorative_is_spent_and_says_what_it_took() {
    let supplies = data::supplies();
    let small = supplies.supplies.iter().min_by_key(|s| s.restores).expect("a small one");
    let big = supplies.supplies.iter().max_by_key(|s| s.restores).expect("a big one");

    let mut c = Character::starting();
    assert!(c.use_supply(&small.id).is_err(), "drank one without having one");
    c.give_supply(&small.id, 2);
    assert!(c.use_supply(&small.id).is_err(), "drank one while rested");

    c.tire(CAP);
    let took = c.use_supply(&small.id).expect("a tired character may drink");
    assert_eq!(took, small.restores);
    assert_eq!(c.fatigue, CAP - small.restores);
    assert_eq!(c.supply_count(&small.id), 1, "the tin was not spent");

    // A big one against a small tiredness reports what it used, not what it
    // claims — a player who wastes forty points should be told.
    c.fatigue = 5;
    c.give_supply(&big.id, 1);
    assert_eq!(c.use_supply(&big.id).unwrap(), 5);
    assert_eq!(c.fatigue, 0);
    assert_eq!(c.supply_count(&big.id), 0);
}

/// Everything on the shelf can be paid for by the fights that made you need it.
#[test]
fn a_restorative_costs_about_what_a_fight_pays() {
    let supplies = data::supplies();
    for s in &supplies.supplies {
        let fights = (s.restores + fatigue::PER_FIGHT - 1) / fatigue::PER_FIGHT;
        // A tin should not cost more Fnorp than the fights it undoes are worth
        // several times over; the pit pays about six a win.
        assert!(
            s.price <= fights * 6,
            "{} undoes {fights} fights and costs {} Fnorp",
            s.id,
            s.price
        );
        assert!(s.price > 0, "{} is free", s.id);
    }
}
