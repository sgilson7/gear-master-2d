//! What it costs to not fight, and what it costs to guess.
//!
//! Four asks, and they are one design: **fatigue is the only thing a fight
//! spends for good, so anything that dodges a fight has to spend it too.**
//!
//! > you should not be able to run away from enemies anymore for free; if you
//! > run away, you lose 20% tiredness, which can go beyond the 60% normal
//! > threshold as well
//!
//! > make the cairn room more dangerous ... by making the walk away action make
//! > you lose 10% tiredness, and it can go past the max of 60% to 99%
//!
//! **Two caps, and the difference between them is the whole of it.** `CAP` is
//! sixty and is what a *fight* leaves on you, because a fight is a budget.
//! `HARD_CAP` is ninety-nine and is where the things you do *instead of*
//! fighting stop.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::fatigue::{self, SupplyDoes, CAP, HARD_CAP, RUNNING_AWAY};
use gm2d_core::fight::Encounter;
use gm2d_core::game::Game;

const D: Difficulty = Difficulty::Easy;

fn met() -> Game {
    let mut g = Game::new(9, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.encounter = Some(Encounter { enemy: "Cave Rat".into(), at: g.world.at });
    g
}

// ------------------------------------------------------------ running away

/// **It is not free, and it takes the encounter with it.**
#[test]
fn running_away_costs_twenty() {
    let mut g = met();
    let cost = g.flee().expect("there is something to run from");
    assert_eq!(cost, Some(RUNNING_AWAY));
    assert_eq!(g.character.fatigue, RUNNING_AWAY);
    assert!(g.encounter.is_none(), "you ran and the creature is still there");
    assert_eq!(g.flee(), Err("there is nothing to run from".into()));
}

/// **And it goes past what a fight can do.**
///
/// This is the half that needed two caps. A character worn to sixty by fighting
/// is at the ceiling of what fighting does; running away from the next one
/// still costs, and the check is that it *moves*.
#[test]
fn running_away_goes_past_the_fight_cap() {
    let mut g = met();
    g.character.fatigue = CAP;
    g.flee().unwrap();
    assert!(g.character.fatigue > CAP, "running away stopped where a fight stops");
    assert_eq!(g.character.fatigue, CAP + RUNNING_AWAY);

    // And it stops at the hard cap rather than running away with itself.
    for _ in 0..10 {
        g.encounter = Some(Encounter { enemy: "Cave Rat".into(), at: g.world.at });
        g.flee().unwrap();
    }
    assert_eq!(g.character.fatigue, HARD_CAP);
    assert!(HARD_CAP < 100, "a hundred percent tired is a division nobody wrote down");
}

/// **A fight still stops at sixty**, or the two caps are one cap.
#[test]
fn a_fight_still_stops_where_it_always_did() {
    let mut g = met();
    for _ in 0..40 {
        g.character.tire(fatigue::PER_FIGHT);
    }
    assert_eq!(g.character.fatigue, CAP, "ordinary wear went past the fight's own ceiling");
}

/// **Ninety-nine percent tired is not a dead end**, which is what makes the
/// hard cap a penalty rather than a soft-lock.
///
/// Walking costs nothing, so a character pinned at the ceiling can always reach
/// a town — and a town takes all of it off, from ninety-nine as readily as from
/// sixty.
#[test]
fn the_hard_cap_is_never_a_dead_end() {
    let mut g = met();
    g.character.fatigue = HARD_CAP;
    // Alive, by more than a rounding rule.
    assert!(g.character.player_stats().health >= 1);
    // Walking is free: stepping does not tire you.
    let w = data::world(D);
    let allowed = g.character.allowances();
    let before = g.character.fatigue;
    gm2d_core::world::step(
        &w,
        &mut g.world,
        &mut g.rng,
        D,
        gm2d_core::world::Dir::East,
        &allowed,
    );
    assert_eq!(g.character.fatigue, before, "walking cost tiredness, so there is no way out");
    // And a town takes the lot.
    let town = w.places.iter().find(|p| p.kind == gm2d_core::world::PlaceKind::Town).unwrap();
    let mended = g.arrive_in_town(&town.id);
    assert_eq!(mended, HARD_CAP);
    assert_eq!(g.character.fatigue, 0, "a town could not mend a character at the ceiling");
}

/// **The tiredness reaches the fight**, or the cap above sixty is decoration.
#[test]
fn being_past_the_cap_is_felt() {
    let mut g = met();
    g.character.fatigue = CAP;
    let at_sixty = g.character.player_stats().health;
    g.character.fatigue = HARD_CAP;
    let at_the_ceiling = g.character.player_stats().health;
    assert!(
        at_the_ceiling < at_sixty,
        "sixty and ninety-nine wear the same, so `worn` is still clamping at the fight's cap"
    );
}

// -------------------------------------------------------------- the charms

/// **A Quiet Word pays instead of you, and it is spent doing it.**
#[test]
fn a_quiet_word_pays_the_fare() {
    let charm = data::supplies()
        .supplies
        .iter()
        .find(|s| s.does == SupplyDoes::Flight)
        .expect("nothing in the game is a flight charm")
        .id
        .clone();
    let mut g = met();
    g.character.give_supply(&charm, 2);
    assert_eq!(g.flee().unwrap(), None, "the charm did not pay");
    assert_eq!(g.character.fatigue, 0, "it charged you as well");
    assert_eq!(g.character.supply_count(&charm), 1, "the charm was not spent");

    // The second one pays for the second flight, and the third costs you.
    g.encounter = Some(Encounter { enemy: "Cave Rat".into(), at: g.world.at });
    assert_eq!(g.flee().unwrap(), None);
    g.encounter = Some(Encounter { enemy: "Cave Rat".into(), at: g.world.at });
    assert_eq!(g.flee().unwrap(), Some(RUNNING_AWAY), "a third flight was free");
}

/// It is not a thing you drink, and saying so beats a button that does nothing.
#[test]
fn a_charm_is_not_a_tin() {
    let supplies = data::supplies();
    let mut c = gm2d_core::character::Character::starting();
    c.tire(CAP);
    for s in supplies.supplies.iter().filter(|s| s.does != SupplyDoes::Restore) {
        c.give_supply(&s.id, 1);
        let why = c.use_supply(&s.id).expect_err("a charm was drunk");
        assert!(why.contains(&s.name), "the refusal does not name it: {why}");
        assert_eq!(c.supply_count(&s.id), 1, "a refused drink spent it");
    }
    assert_eq!(c.fatigue, CAP, "a refused drink took tiredness off");
}

/// **The Short Way Back puts you in your last town, and spends itself.**
#[test]
fn the_short_way_back_takes_you_home() {
    let charm = data::supplies()
        .supplies
        .iter()
        .find(|s| s.does == SupplyDoes::Home)
        .expect("nothing in the game is a way home")
        .id
        .clone();
    let mut g = Game::new(3, "td");
    g.world = gm2d_core::world::WorldState::at_start(&data::world(D));
    g.world.last_town = "the-end-of-all-gears".into();
    g.world.map = "the-great-gear-cave".into();
    g.world.at = [1, 2];
    g.character.tire_hard(HARD_CAP);

    // Without one, it refuses and changes nothing.
    let why = g.warp_home(D).expect_err("warped with nothing in the pack");
    assert!(why.contains("pack"), "{why}");
    assert_eq!(g.world.map, "the-great-gear-cave", "a refusal moved somebody");

    g.character.give_supply(&charm, 1);
    let h = g.warp_home(D).expect("the charm takes you home");
    assert_eq!(h.town, "the-end-of-all-gears");
    assert_eq!(g.world.map_id(), "west-bambulon", "it did not cross the map");
    assert_eq!(g.character.fatigue, 0, "arriving did not mend you");
    assert_eq!(h.mended, HARD_CAP);
    assert_eq!(g.character.supply_count(&charm), 0, "the charm was not spent");
}

/// **And not from under the lake**, which is the one map where the walk is the
/// content — the same refusal the Drover's Stride gives, for the same reason.
#[test]
fn the_short_way_back_is_refused_under_the_lake() {
    let charm = data::supplies()
        .supplies
        .iter()
        .find(|s| s.does == SupplyDoes::Home)
        .unwrap()
        .id
        .clone();
    let mut g = Game::new(3, "td");
    g.world.last_town = "the-end-of-all-gears".into();
    g.world.map = "under-the-lake".into();
    g.world.at = [1, 1];
    g.character.give_supply(&charm, 1);
    let why = g.warp_home(D).expect_err("warped out from under the lake");
    assert!(why.contains("upwards"), "{why}");
    assert_eq!(g.character.supply_count(&charm), 1, "a refusal spent the charm");
}

// ------------------------------------------------------------- the cairns

/// **A cairn you cannot use costs you the walk.**
#[test]
fn walking_off_a_cairn_costs_ten() {
    let mut g = Game::new(7, "td");
    g.world.map = "the-sump-3".into();
    // The first cairn in the chain wants nothing, so every *other* one refuses.
    let events = data::events();
    let shut = events
        .events
        .iter()
        .find(|e| {
            e.id.starts_with("the-cairn-at-")
                && e.choices.iter().all(|c| !matches!(c.requires, gm2d_core::tile_event::Requirement::None))
        })
        .expect("every cairn is open, so none of them can refuse");
    let paid = g.read_event(&shut.id);
    assert_eq!(paid, fatigue::WALKING_OFF);
    assert_eq!(g.character.fatigue, fatigue::WALKING_OFF);
}

/// **And the one you can use costs nothing**, so the toll is a price on being
/// wrong rather than a price on the floor.
#[test]
fn the_cairn_you_can_use_is_free() {
    let mut g = Game::new(7, "td");
    g.world.map = "the-sump-3".into();
    let events = data::events();
    let open = events
        .events
        .iter()
        .find(|e| {
            e.id.starts_with("the-cairn-at-")
                && e.choices.iter().any(|c| matches!(c.requires, gm2d_core::tile_event::Requirement::None))
        })
        .expect("no cairn starts the chain");
    assert_eq!(g.read_event(&open.id), 0);
    assert_eq!(g.character.fatigue, 0);
}

/// **It goes past sixty too**, which is what *"can go past the max of 60% to
/// 99%"* asks for — and the Cairnfield is where it will actually happen,
/// because guessing it blind is thirty-six refusals.
#[test]
fn the_cairns_go_past_the_fight_cap() {
    let mut g = Game::new(7, "td");
    g.world.map = "the-sump-3".into();
    let events = data::events();
    let shut: Vec<String> = events
        .events
        .iter()
        .filter(|e| {
            e.id.starts_with("the-cairn-at-")
                && e.choices.iter().all(|c| !matches!(c.requires, gm2d_core::tile_event::Requirement::None))
        })
        .map(|e| e.id.clone())
        .collect();
    for _ in 0..20 {
        for id in &shut {
            g.read_event(id);
        }
    }
    assert_eq!(g.character.fatigue, HARD_CAP, "the cairns stop at the fight's own cap");
}

/// A card that has been answered is free to read again.
///
/// **Check the second visit.** Re-reading a heap you already built is not a
/// walk, and a toll that charged for it would charge for the walk *out*.
#[test]
fn a_cairn_you_finished_is_free_to_read() {
    let mut g = Game::new(7, "td");
    g.world.map = "the-sump-3".into();
    let events = data::events();
    let open = events
        .events
        .iter()
        .find(|e| e.id.starts_with("the-cairn-at-") && e.choices.iter().any(|c| matches!(c.requires, gm2d_core::tile_event::Requirement::None)))
        .unwrap();
    g.answer_event(&open.id, 0, D).expect("the first stone goes on");
    let was = g.character.fatigue;
    assert_eq!(g.read_event(&open.id), 0);
    assert_eq!(g.character.fatigue, was);
}

/// **Nothing else in the game charges a toll**, so this is the Cairnfield's
/// rule rather than a tax on reading.
#[test]
fn only_the_cairns_charge_the_walk() {
    let events = data::events();
    let tolled: Vec<&str> =
        events.events.iter().filter(|e| e.toll > 0).map(|e| e.id.as_str()).collect();
    assert!(!tolled.is_empty(), "nothing charges a toll, so the field is not dangerous");
    for id in &tolled {
        assert!(id.starts_with("the-cairn-at-"), "{id} charges a toll and is not a cairn");
    }
    assert_eq!(tolled.len(), 9, "there are nine heaps and {} charge", tolled.len());
}
