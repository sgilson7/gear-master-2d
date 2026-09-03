//! The surveyable map, and the three lenses it is read through.
//!
//! M11.6. The map is **static and authored** — the same tiles every survey, so
//! an errand can name the trig stone and a person can learn where the drove way
//! is. What varies is the instrument, and everything it varies is a number.
//!
//! The architectural claim `PLAN-M11.md` asks to be enforced by a test is that
//! `SurveyMod` application is a pure function of `(map, kind, character)`, with
//! no survey state in any map file. Both halves are checked here, because the
//! payoff is that a second surveyable map is a data drop rather than a
//! milestone.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::survey::{self, SurveyMod};
use gm2d_core::world::{PlaceKind, World, WorldState};

mod common;

const D: Difficulty = Difficulty::Easy;
const REACH: &str = "the-reach";

fn reach() -> World {
    data::map(REACH, D)
}

fn surveying(kind: &str) -> WorldState {
    let mut st = WorldState::at_start(&reach());
    st.map = REACH.into();
    st.active_survey = Some((REACH.into(), kind.into()));
    st
}

/// It is a map, it is static, and nothing in its file mentions a survey.
///
/// **The architecture note, as a test.** A map that carried its own survey
/// tuning would be a map that had to be edited to add a second one; the lens is
/// entirely in `survey::mods_for`, and a data file that mentioned it would be
/// the first crack in that.
#[test]
fn nothing_in_a_map_file_knows_about_a_survey() {
    for (name, text) in data::FILES {
        if !name.starts_with("maps/") {
            continue;
        }
        for word in ["survey_mod", "encounter_pct", "drops_per_mille", "xp_pct", "golem"] {
            assert!(
                !text.contains(word),
                "{name} mentions {word:?}, so a survey has leaked into the map"
            );
        }
    }
    let w = reach();
    assert_eq!((w.width, w.height), (20, 20));
    assert_eq!(w.regions.len(), 2);
    assert!(w.survey.is_none(), "a map read off disk is read through nothing");
}

/// **Pure.** Same inputs, same answer, and nothing else moves it.
#[test]
fn the_lens_is_a_function_of_the_map_the_instrument_and_the_board() {
    for kind in gm2d_core::rule::INSTRUMENTS {
        for items in [0usize, 3, 5] {
            let a = survey::mods_for(REACH, kind, items);
            let b = survey::mods_for(REACH, kind, items);
            assert_eq!(a, b, "{kind} at {items} items answered twice");
        }
    }
    // An instrument nobody wrote reads the map as written, rather than
    // panicking or inventing something.
    assert!(survey::mods_for(REACH, "sextant", 4).is_none());
}

/// **Each of the three changes the same map, measurably and differently.**
///
/// The acceptance criterion, read off the map rather than off the constants:
/// the same tile, three lenses, three different numbers.
#[test]
fn each_instrument_reads_the_reach_differently() {
    let plain = reach();
    // Somewhere the ground actually rolls, or every number below is zero and
    // the whole check compares nothing with nothing.
    let tile = (0..plain.height)
        .flat_map(|y| (0..plain.width).map(move |x| (x, y)))
        .find(|&(x, y)| plain.encounter_per_mille(x, y) > 50)
        .expect("the reach has no ground that stops you");
    let base = plain.encounter_per_mille(tile.0, tile.1);

    let read = |kind: &str, items: usize| {
        data::map_read_through(REACH, D, &surveying(kind), items)
    };

    let compass = read("compass", 0);
    assert!(
        compass.encounter_per_mille(tile.0, tile.1) < base,
        "a compass did not quieten the ground: {} against {base}",
        compass.encounter_per_mille(tile.0, tile.1)
    );
    // *Augmented by gear*: a packed board reads better than an empty one.
    let packed = read("compass", 5);
    assert!(
        packed.encounter_per_mille(tile.0, tile.1)
            < compass.encounter_per_mille(tile.0, tile.1),
        "five items on the board bought nothing"
    );

    let atlas = read("atlas", 5);
    assert!(
        atlas.encounter_per_mille(tile.0, tile.1) > base,
        "an atlas is a promise and the reach did not hear it"
    );
    assert!(atlas.survey.drops_per_mille > 0 && atlas.survey.xp_pct > 0);

    let golem = read("golem", 5);
    assert!(golem.survey.golem);
    assert_eq!(
        golem.encounter_per_mille(tile.0, tile.1),
        base,
        "the golem reads the ground as it is; what it changes is who fights"
    );
}

/// **And only on the map it is pointed at.**
///
/// An instrument on the board while you are standing in the pit is an
/// instrument doing nothing, which is what makes it a survey rather than a
/// stat.
#[test]
fn a_survey_reads_one_map_and_no_other() {
    let st = surveying("atlas");
    let elsewhere = data::map_read_through(&gm2d_core::world::overworld(), D, &st, 5);
    assert!(elsewhere.survey.is_none(), "an atlas changed West Bambulon");
    let here = data::map_read_through(REACH, D, &st, 5);
    assert!(!here.survey.is_none());
}

/// The compass has a floor, because a rate that reaches zero is a map you
/// cannot fight on.
#[test]
fn a_compass_can_only_take_so_much_off() {
    let quiet = survey::mods_for(REACH, "compass", 500);
    assert_eq!(quiet.encounter_pct, survey::COMPASS_FLOOR_PCT);
    assert!(quiet.encounter_pct > -100, "a compass switched the game off");
}

// ------------------------------------------------------------ the way in

/// **The edge refuses without an instrument, and says so.**
///
/// `needs_survey` and not `needs`: an instrument is not a thing in the bag, it
/// is an assembled item on the board. The shim answers it for the reason it
/// answers a key — a map does not know about bags and does not know about rules
/// either.
#[test]
fn the_edge_of_the_reach_wants_an_instrument() {
    let treyway = data::map("the-treyway", D);
    let edge = treyway
        .places
        .iter()
        .find(|p| p.needs_survey)
        .expect("nothing on the Treyway wants an instrument");
    assert_eq!(edge.kind, PlaceKind::Gate);
    assert_eq!(edge.to.as_deref(), Some(REACH));
    assert!(edge.needs.is_none(), "the edge wants a component as well, which is two locks");
    assert!(!edge.shut.is_empty(), "it refuses in silence");
    // TONE 12: the refusal names what is missing.
    let said = edge.shut.to_lowercase();
    assert!(
        said.contains("read") || said.contains("nothing to"),
        "the refusal does not name the instrument: {said:?}"
    );

    // And the post you read about it is beside it, not on it — an errand that
    // says "go and read the nine clipboards" must not want an instrument.
    let post = treyway
        .places
        .iter()
        .find(|p| p.id == "the-wextreen-reach")
        .expect("the post with the clipboards");
    assert_eq!(post.kind, PlaceKind::Event);
    assert_ne!(post.at, edge.at);
}

/// **The reach has something to read and a way back off it.**
#[test]
fn the_reach_is_a_place_and_not_a_corridor() {
    let w = reach();
    let events = data::events();
    let readable = w.places.iter().filter(|p| p.kind == PlaceKind::Event).count();
    assert!(readable >= 5, "only {readable} things to read on a twenty-by-twenty map");
    for p in w.places.iter().filter(|p| p.kind == PlaceKind::Event) {
        assert!(events.get(&p.id).is_some(), "{} is placed and written nowhere", p.id);
    }
    let out = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Gate)
        .expect("no way back off the reach");
    assert_eq!(out.to.as_deref(), Some("the-treyway"));
    // It is not a sitting: you walk off it, and a save on it reopens on it.
    assert!(w.outside.is_none());
}

/// **The errands on it name places on it**, which is what static buys.
#[test]
fn the_reach_carries_a_questline_that_names_its_own_ground() {
    let quests = data::quests();
    let on_it: Vec<[u8; 2]> = reach().places.iter().map(|p| p.at).collect();
    let named = quests
        .quests
        .iter()
        .filter_map(|q| q.goal.place())
        .filter(|p| reach().places.iter().any(|x| x.id == *p))
        .count();
    assert!(named >= 1, "no errand anywhere names a place on the reach");
    assert!(!on_it.is_empty());

    // And the line that gets you there is three deep, ending on the reach.
    let mut line = vec![quests.get("the-count-at-the-pond").expect("the field's line")];
    loop {
        let last = line.last().unwrap().id.clone();
        match quests.quests.iter().find(|q| q.requires.iter().any(|r| *r == last)) {
            Some(next) => line.push(next),
            None => break,
        }
    }
    assert!(line.len() >= 5, "the line to the reach is {} errands", line.len());
    let ends_there = line
        .last()
        .and_then(|q| q.goal.creature())
        .map(|c| reach().regions.iter().any(|r| r.enemies.iter().any(|m| m.name == c)))
        .unwrap_or(false)
        || line
            .iter()
            .filter_map(|q| q.goal.place())
            .any(|p| reach().places.iter().any(|x| x.id == p));
    assert!(ends_there, "the line never actually reaches the reach");
}

// ------------------------------------------------------------- the golem

/// **The golem takes one fight an entry, and pays what a win pays.**
///
/// The fallback `PLAN-M11.md` §8 row 6 named in advance so that taking it would
/// be a decision rather than a retreat. What made it the right call is not the
/// replay's layout: it is rule 5. A third board is a third set of numbers the
/// page must not invent, and the honest version is a third combatant in
/// `combat.rs`, which is new combat code in a block that has added none.
#[test]
fn the_golem_handles_one_and_then_stops() {
    let mut g = Game::new(5, "td");
    g.world = surveying("golem");
    g.world.at = [1, 18];
    let met = "Vermin Sovereign";

    g.encounter = Some(gm2d_core::fight::Encounter { enemy: met.into(), at: g.world.at });
    let first = gm2d_core::fight::rout(&mut g).expect("the golem did not take the first one");
    assert!(first.gold > 0, "it was settled and paid nothing");
    assert!(
        first.receipt.iter().any(|l| l.to_lowercase().contains("golem")),
        "the receipt does not say who fought it: {:?}",
        first.receipt
    );
    assert!(
        g.world.answered.iter().any(|a| a == gm2d_core::fight::GOLEM_SPENT),
        "nothing recorded that the golem has been"
    );

    // And the second one is yours.
    g.encounter = Some(gm2d_core::fight::Encounter { enemy: met.into(), at: g.world.at });
    assert!(
        gm2d_core::fight::rout(&mut g).is_none(),
        "the golem fought twice in one visit"
    );
    assert!(g.encounter.is_some(), "the second encounter went missing");
}

/// A compass does not fight anything, and neither does an atlas.
#[test]
fn only_the_golem_takes_a_fight() {
    for kind in ["compass", "atlas"] {
        let mut g = Game::new(6, "td");
        g.world = surveying(kind);
        g.world.at = [1, 18];
        g.encounter = Some(gm2d_core::fight::Encounter {
            enemy: "Vermin Sovereign".into(),
            at: g.world.at,
        });
        assert!(gm2d_core::fight::rout(&mut g).is_none(), "{kind} fought something");
    }
}

/// The mod with nothing on is the map as written, so every caller can add its
/// fields without asking whether there is a survey on.
#[test]
fn no_survey_is_the_map_as_written() {
    let none = SurveyMod::none();
    assert_eq!(none.encounter_pct, 0);
    assert_eq!(none.drops_per_mille, 0);
    assert_eq!(none.xp_pct, 0);
    assert!(!none.golem);
    assert_eq!(survey::shift(180, 0), 180);
    assert_eq!(survey::shift(180, -20), 144);
    assert_eq!(survey::shift(0, 40), 0);
}

// ------------------------------------------------- what the block asks of you

/// **Every region has a fight you can win in it, and every boss can be beaten.**
///
/// The M4 soft-lock's shape, one block out. M11.7 measured the tower against the
/// best board the game hands out — both shelves bought out, every errand reward,
/// every set piece, auto-packed — and found the second floor's boss beat it.
/// That is a wall: the tower cannot be dropped, so the lake cannot be drained,
/// so the ending is unreachable, and 597 tests were green through it.
///
/// **The *most drawn* creature, not every creature.** `draw_enemy` weights a
/// pool so its hardest member is its rarest, so a region's teeth are supposed to
/// be a fight you sometimes lose — what must not happen is that the fight you
/// meet three times in five is one you cannot win. And every boss, because a
/// boss is not drawn: it stands there, and there is no going round it.
///
/// The yardstick is generous on purpose. It assumes you bought the whole of
/// both shelves and finished everything, which no real player will have done —
/// generous about *reachable* content is the safe direction for a check that
/// says "this is possible at all".
#[test]
fn every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten() {
    use gm2d_core::combat::{self, Outcome};

    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let beats = |name: &str| -> bool {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D).outcome
            == Outcome::Victory
    };

    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for r in &w.regions {
            // The most drawn is the *lowest rated*: the weight is
            // `(max + 1 − rating)`, so the easiest member is the commonest.
            let common = r
                .enemies
                .iter()
                .min_by_key(|m| gm2d_core::rating::creature_rating(m, D))
                .expect("a region with an empty pool");
            assert!(
                beats(common.name),
                "{id}/{}: the creature you meet most often is {} and the best board the \
                 game hands out cannot beat it",
                r.id,
                common.name
            );
        }
        for p in w.places.iter().filter_map(|p| p.creature.as_deref().map(|c| (p, c))) {
            let (place, who) = p;
            assert!(
                beats(who),
                "{id}/{}: {who} is standing on a tile and cannot be beaten, so nothing \
                 behind it can ever be reached",
                place.id
            );
        }
    }
}
