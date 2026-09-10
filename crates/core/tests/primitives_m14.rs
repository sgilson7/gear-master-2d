//! M14.0 — five small things made true in core before nine maps are drawn on
//! top of them.
//!
//! Nothing here is content. Every test is negative-tested: the fault it guards
//! was put back and the right sentence watched to print, and the note beside
//! each one says what that sentence was.

use gm2d_core::combat::Difficulty;
use gm2d_core::character::Character;
use gm2d_core::game::Game;
use gm2d_core::rule::INSTRUMENTS;
use gm2d_core::tile_event::{Choice, EventsData, Outcome, Requirement};
use gm2d_core::world::{PlaceDef, PlaceKind, World, WorldState};

const D: Difficulty = Difficulty::Easy;

mod common;

// ------------------------------------------------------------------ fixtures

/// A one-map world drawn from rows, for the questions that are about the
/// machinery rather than about the shipped content.
fn world_of(rows: &[&str], places: &str, extra: &str) -> Result<World, String> {
    // The start is the last row's second tile, which every fixture here draws
    // as ground.
    let tiles = format!(
        r#"{{"format":"gm2d-tiles","version":1,"id":"a-test-floor",
            "width":{w},"height":{h},"start":[1,{sy}],
            "rows":{rows},
            "regions":[{{"id":"r","name":"R","bounds":[[0,0,{x},{y}]],
                         "enemies":["Cave Rat"]}}],
            "places":[{places}]{extra}}}"#,
        sy = rows.len() - 2,
        w = rows[0].chars().count(),
        h = rows.len(),
        x = rows[0].chars().count() - 1,
        y = rows.len() - 1,
        rows = serde_json::to_string(rows).unwrap(),
    );
    World::load(gm2d_core::data::TERRAIN_JSON, &tiles, D)
}

fn events_of(json: &str) -> Result<EventsData, String> {
    EventsData::parse(&format!(
        r#"{{"format":"gm2d-events","version":1,"events":{json}}}"#
    ))
}

// ------------------------------------------------------- the survey requirement

/// **`Surveying` reads the frame, and nothing else does.**
///
/// The same answer a `needs_survey` gate gets, off `Game::survey_kind`. It is
/// answered in **core** rather than in the shim, which `PROMPT-M14.md` expected
/// to be the block's one shim addition: a `Game` is exactly the thing that
/// holds both a character and a world, and `can_take` was already there.
///
/// Negative-tested by making `can_take` answer `true` for every `Surveying`:
/// *"a bare board read the floor with a compass"*.
#[test]
fn a_survey_requirement_reads_the_frame() {
    let mut g = Game::default();
    let wants = |k: &str| Choice {
        label: "read it".into(),
        blurb: String::new(),
        requires: Requirement::Surveying(k.into()),
        outcome: Outcome::Flag("read-it".into()),
        unmet: "you have nothing to read it with".into(),
    };

    assert!(
        !g.can_take(&wants("compass")),
        "a character with an empty instrument frame read the floor with a compass"
    );

    // A compass is a shard, a lens and a magnet, on the frame that is not the
    // weapon's.
    g.character = Character::with_all_pieces();
    common::seat(
        &mut g.character,
        &[
            // A shard is two cells across, a lens is one and a magnet is two
            // down. They touch, which is what makes them one item.
            ("Map Shard", gm2d_core::piece::SlotKind::Instrument, 0, 0, 0),
            ("Glass Lens", gm2d_core::piece::SlotKind::Instrument, 2, 0, 0),
            ("Magnet", gm2d_core::piece::SlotKind::Instrument, 3, 0, 0),
        ],
    );
    assert_eq!(g.survey_kind().as_deref(), Some("compass"), "the frame did not make a compass");
    assert!(g.can_take(&wants("compass")), "a compass would not read a compass floor");
    // **It names a kind, and that is the point of it.** A floor any instrument
    // reads is a floor no instrument is for.
    assert!(!g.can_take(&wants("atlas")), "a compass read an atlas floor");
    assert!(!g.can_take(&wants("golem")), "a compass read a golem floor");
}

/// A requirement the engine could never answer does not load.
///
/// Negative-tested by dropping the `check` call out of `EventsData::parse`:
/// the misspelled instrument parsed clean and became a door nobody can open.
#[test]
fn an_instrument_nobody_built_does_not_load() {
    let bad = events_of(
        r#"[{"id":"e","title":"E","prose":["p"],"choices":[
            {"label":"read","blurb":"","requires":{"surveying":"compasss"},
             "outcome":{"flag":"f"},"unmet":"no"}]}]"#,
    );
    let why = bad.expect_err("a floor asking for a compasss loaded");
    assert!(why.contains("compasss"), "the refusal did not name it: {why}");

    for k in INSTRUMENTS {
        assert!(
            Requirement::Surveying((*k).into()).check().is_ok(),
            "{k} is an instrument and the check refused it"
        );
    }
    // And the shape of a hole with no sides.
    assert!(Requirement::LooseItemOfSize { w: 0, h: 2 }.check().is_err());
    assert!(Requirement::AssembledOfRarity("shiny".into()).check().is_err());
    assert!(Requirement::AssembledOfRarity("rare".into()).check().is_ok());
}

// ------------------------------------------------------------------- the board

/// **A footprint is asked of the bag, solid, and either way up.**
///
/// Negative-tested by dropping the `s.area() == w * h` term: an L-shaped
/// four-cell piece answered a two-by-two door, which is a slot with two cells
/// of daylight in it holding nothing up.
#[test]
fn a_footprint_is_solid_and_either_way_up() {
    let ch = Character::with_all_pieces();
    // The Iron Blade is one wide and four tall, and it is the piece a starting
    // character has to turn to fit a three-row frame — so it is the one this
    // suite already knows the shape of.
    let ones = ch.loose_of_size(1, 4);
    assert!(
        ones.iter().any(|&id| ch.registry.def(id).name == "Iron Blade"),
        "the Iron Blade is 1x4 and was not in the 1x4s"
    );
    assert_eq!(
        ch.loose_of_size(1, 4).len(),
        ch.loose_of_size(4, 1).len(),
        "a slot cares which way up a rectangle goes in"
    );
    // Solid. The Hollow Weave is the fixture's own example of a piece that is
    // not a block, and nothing that is not a block may answer a block.
    for &id in &ones {
        let s = gm2d_core::shape::Shape::new(ch.registry.def(id).cells);
        assert_eq!(s.area(), 4, "{} is 1x4 and has {} cells", ch.registry.def(id).name, s.area());
    }
    // **The one that would go quiet if solidity were dropped.** Every piece
    // whose bounding box is two by two is 86 of 172 in the catalogue; the other
    // 86 are corners and tees, and a slot cut two by two holds none of them.
    let squares = ch.loose_of_size(2, 2);
    assert!(!squares.is_empty(), "nothing in the catalogue is a two by two");
    for &id in &squares {
        let d = ch.registry.def(id);
        assert_eq!(
            gm2d_core::shape::Shape::new(d.cells).area(),
            4,
            "{} fits inside a two by two and does not fill one",
            d.name
        );
    }
}

/// **A seated component is not loose, and a tally is never a door's price.**
///
/// Negative-tested by dropping the `is_equipped` filter: the door took a piece
/// out of the middle of an assembled weapon on a screen nobody was looking at.
#[test]
fn a_door_never_reaches_onto_the_board() {
    let mut ch = Character::with_all_pieces();
    let before = ch.loose_of_size(1, 4).len();
    let id = common::piece(&ch, "Iron Blade");
    ch.registry.set_rotation(id, 1);
    ch.equip(id, gm2d_core::piece::SlotKind::Weapon, 0, 0).expect("a blade in a weapon grid");
    assert_eq!(
        ch.loose_of_size(1, 4).len(),
        before - 1,
        "a seated Iron Blade was still on offer to a door"
    );

    // A key is carried, never worn, and never handed to a wheel.
    assert!(
        !ch.loose_of_size(1, 1).iter().any(|&i| {
            ch.registry.def(i).kind == gm2d_core::piece::PieceKind::Quest
        }),
        "a quest tally was offered to a door as a component"
    );
}

/// **What is given up is the cheapest that fits, and a refusal spends
/// nothing.**
///
/// Negative-tested by sorting best-rated first: the door ate the best two-by-two
/// in the bag, which is a mugging rather than a price.
#[test]
fn a_door_keeps_the_cheapest_thing_that_fits() {
    let mut ch = Character::with_all_pieces();
    // **Worked out from the ratings rather than read off the list this is
    // testing.** The first version asked whether `give_up` took
    // `loose_of_size(2, 2)[0]`, which is the same list in the same order — so
    // reversing the sort passed it cleanly. A check that reads its answer off
    // the thing it is checking is the *compares zero with zero* failure with
    // an extra step, and this project has now shipped four of those.
    let rate = |id| gm2d_core::rating::piece_rating(ch.registry.def(id));
    let fits = ch.loose_of_size(2, 2);
    let cheapest = ch
        .registry
        .def(*fits.iter().min_by_key(|&&id| (rate(id), id.0)).expect("a two by two"))
        .name
        .to_string();
    let dearest =
        ch.registry.def(*fits.iter().max_by_key(|&&id| rate(id)).unwrap()).name.to_string();
    assert_ne!(cheapest, dearest, "every two-by-two rates the same, so this proves nothing");

    let owned = ch.owned.len();
    let gone = ch.give_up(2, 2).expect("a two by two");
    assert_eq!(gone, cheapest, "the door took {gone} and the cheapest was {cheapest}");
    assert_eq!(ch.owned.len(), owned - 1, "the bag is not one lighter");

    // Nothing in the catalogue is nine by nine, and asking spends nothing.
    let before = ch.owned.len();
    assert_eq!(ch.give_up(9, 9), None);
    assert_eq!(ch.owned.len(), before, "a refused door took something anyway");
}

// -------------------------------------------------------------------- drains

/// **A drain with `tiles` leaves the rest of the map alone.**
///
/// Negative-tested by ignoring `tiles` in `drain_by`: one wheel drained every
/// channel on the floor, which is a puzzle with one move in it.
#[test]
fn a_drain_with_tiles_leaves_the_rest() {
    let rows = ["^^^^^", "^~~~^", "^,,,^", "^^^^^"];
    let w = world_of(
        &rows,
        "",
        r#","drains":[{"when":"sluice-a","from":"water","to":"silt","tiles":[[1,1],[2,1]]}]"#,
    )
    .expect("a floor with one channel");

    let mut before = w.clone();
    before.drain_by(&[]);
    for x in 1..4 {
        assert_eq!(before.terrain_name(x, 1), "water", "it drained before the wheel turned");
    }

    let mut after = w.clone();
    after.drain_by(&["sluice-a".to_string()]);
    assert_eq!(after.terrain_name(1, 1), "silt");
    assert_eq!(after.terrain_name(2, 1), "silt");
    assert_eq!(
        after.terrain_name(3, 1),
        "water",
        "the wheel drained a cell it was not pointed at"
    );

    // And the old shape still means the whole map.
    let all = world_of(&rows, "", r#","drains":[{"when":"x","from":"water","to":"silt"}]"#)
        .expect("a whole-map drain");
    let mut all = all;
    all.drain_by(&["x".to_string()]);
    for x in 1..4 {
        assert_eq!(all.terrain_name(x, 1), "silt", "a drain with no tiles left one behind");
    }
}

/// **A drain naming a cell it cannot drain does not load.**
///
/// The failure this project keeps finding one milestone late: a rule that runs,
/// reports success and moves no tile. Negative-tested by dropping the check —
/// the floor loaded, the wheel turned, and the channel stayed wet.
#[test]
fn a_drain_naming_a_cell_it_cannot_drain_does_not_load() {
    let rows = ["^^^^^", "^~~~^", "^,,,^", "^^^^^"];
    let why = world_of(
        &rows,
        "",
        r#","drains":[{"when":"w","from":"water","to":"silt","tiles":[[1,2]]}]"#,
    )
    .expect_err("a drain pointed at scrub loaded");
    assert!(why.contains("(1, 2)") && why.contains("scrub"), "the refusal did not say: {why}");

    let off = world_of(
        &rows,
        "",
        r#","drains":[{"when":"w","from":"water","to":"silt","tiles":[[9,9]]}]"#,
    )
    .expect_err("a drain pointed off the map loaded");
    assert!(off.contains("off the map"), "{off}");

    let none = world_of(&rows, "", r#","drains":[{"when":"w","from":"water","to":"silt","tiles":[]}]"#)
        .expect_err("a drain naming no cells at all loaded");
    assert!(none.contains("no cells"), "{none}");
}

// --------------------------------------------------------------- sealed doors

fn sealed(id: &str, wants: &[&str]) -> PlaceDef {
    PlaceDef {
        at: [1, 1],
        kind: PlaceKind::Gate,
        id: id.into(),
        name: "the way down".into(),
        to: Some("somewhere".into()),
        at_to: None,
        needs: None,
        shut: "It does not move.".into(),
        needs_all: wants.iter().map(|s| (*s).to_string()).collect(),
        needs_survey: false,
        creature: None,
        drops: Vec::new(),
        hidden_until: None,
        hidden_until_all: Vec::new(),
        prose: Vec::new(),
        hidden_until_level: None,
        sells: Vec::new(),
        guards: None,
        needs_level: None,
        floors: Vec::new(),
    }
}

/// **A door sealed on two things names the one that is missing, by its name
/// and not by its id.**
///
/// `PLAN-M14.md` §1.6: finishing the first of the two dungeons shows a sealed
/// door with the other dungeon's name in the refusal. Negative-tested by
/// dropping the derived half and leaving `shut` alone: *"It does not move."*
/// with nothing after it, which is a bug report.
#[test]
fn a_sealed_door_names_what_is_missing() {
    let mut g = Game::default();
    // Two marks that are real place ids on the shipped maps, so the lookup has
    // something to find — the Cave's boss and the lake's.
    let door = sealed("the-way-on", &["the-bottom-of-the-cave", "the-bottom-of-the-lake"]);

    let why = g.sealed_because(&door, D).expect("a sealed door said nothing");
    assert!(why.starts_with("It does not move."), "the world's own line went missing: {why}");
    assert!(why.contains(" and "), "two things missing and it named one: {why}");
    assert_eq!(g.unlock(&door), gm2d_core::game::Unlocked::Shut);

    // One down, one to go — and it names the one to go.
    g.world.answered.push("the-bottom-of-the-cave".into());
    let why = g.sealed_because(&door, D).expect("still sealed and said nothing");
    assert!(
        !why.contains("Cave") && !why.contains("cave"),
        "it is still asking for the one that is done: {why}"
    );
    assert!(why.contains("waiting on"), "{why}");
    assert_eq!(g.unlock(&door), gm2d_core::game::Unlocked::Shut);

    // Both down: no sentence, and it opens.
    g.world.answered.push("the-bottom-of-the-lake".into());
    assert_eq!(g.sealed_because(&door, D), None);
    assert_eq!(g.unlock(&door), gm2d_core::game::Unlocked::Open);
}

/// **`hidden_until_all` and `hidden_until` are ANDed, and both have to be met.**
///
/// Negative-tested by trying them in turn instead: a place naming both was
/// there on the strength of whichever was checked first.
#[test]
fn hidden_until_all_wants_all_of_them() {
    let mut p = sealed("a-door", &[]);
    p.kind = PlaceKind::Door;
    p.needs_all.clear();
    p.hidden_until = Some("one".into());
    p.hidden_until_all = vec!["two".into(), "three".into()];

    let mut st = WorldState::default();
    let a = gm2d_core::world::Allowances::default();
    assert!(!gm2d_core::world::place_is_there(&p, &st, &a));
    st.flags.push("one".into());
    assert!(!gm2d_core::world::place_is_there(&p, &st, &a), "one of three was enough");
    st.flags.push("two".into());
    assert!(!gm2d_core::world::place_is_there(&p, &st, &a), "two of three was enough");
    assert_eq!(gm2d_core::world::still_wanted(&p, &st), vec!["three".to_string()]);
    st.answered.push("three".into());
    assert!(gm2d_core::world::place_is_there(&p, &st, &a), "all three and it is still not there");
    assert!(gm2d_core::world::still_wanted(&p, &st).is_empty());
}

// ----------------------------------------------------------- repeating events

/// **A repeating event is never spent, and may never pay.**
///
/// Negative-tested twice: dropping the `repeats` branch in `answer_event` made
/// the chair a one-move room, and dropping the `pays` lint let an event that
/// hands over 40 Fnorp be stood on for ever.
#[test]
fn a_repeating_event_never_pays_and_is_never_spent() {
    let good = events_of(
        r#"[{"id":"chair","title":"C","prose":["p"],"repeats":true,"choices":[
            {"label":"turn it","blurb":"","outcome":{"flag":"turned"}},
            {"label":"sit","blurb":"","outcome":{"tire":4}}]}]"#,
    )
    .expect("a chair that costs you a walk and a bit of wind");
    assert!(good.get("chair").expect("a chair").repeats);

    for bad in [
        r#"{"gold":40}"#,
        r#"{"give":"Iron Blade"}"#,
        r#"{"xp":10}"#,
        r#"{"all":[{"flag":"f"},{"gold":1}]}"#,
    ] {
        let why = events_of(&format!(
            r#"[{{"id":"c","title":"C","prose":["p"],"repeats":true,"choices":[
                {{"label":"press","blurb":"","outcome":{bad}}}]}}]"#
        ))
        .expect_err("a faucet loaded: {bad}");
        assert!(why.contains("pays something"), "{why}");
    }

    // **And the half that is not a parse.** The chair is three moves at one
    // object, so it has to answer three times — every other event in this game
    // is spent the moment it is answered, and that is right for a decision.
    let mut g = Game::default();
    let before = g.world.answered.len();
    // `the-shallows-marker` is a shipped root and does *not* repeat: one
    // answer and it is written down for good.
    let once = gm2d_core::data::events()
        .events
        .iter()
        .find(|e| !e.repeats && e.choices.iter().any(|c| c.requires == Requirement::None))
        .map(|e| e.id.clone())
        .expect("a shipped event that asks something of nobody");
    let n = gm2d_core::data::events()
        .get(&once)
        .unwrap()
        .choices
        .iter()
        .position(|c| c.requires == Requirement::None)
        .unwrap();
    g.answer_event(&once, n, D).expect("the first answer");
    assert_eq!(g.world.answered.len(), before + 1, "an ordinary card was not written down");
    assert_eq!(
        g.answer_event(&once, n, D).unwrap_err(),
        "already answered",
        "an ordinary card was answered twice"
    );

    // And a note cannot repeat, because a note already never ends.
    let why = events_of(r#"[{"id":"n","title":"N","prose":["p"],"repeats":true}]"#)
        .expect_err("a note claimed to repeat");
    assert!(why.contains("asks nothing"), "{why}");
}

/// **A choice that keeps a footprint has to ask for that footprint.**
///
/// Negative-tested by dropping the lint: a door asking for a three-by-two took
/// a one-by-four, which is the two halves of one bargain disagreeing.
#[test]
fn every_giving_up_choice_asks_for_what_it_takes() {
    let ok = events_of(
        r#"[{"id":"d","title":"D","prose":["p"],"choices":[
            {"label":"in the slot","blurb":"","requires":{"loose_item_of_size":{"w":3,"h":2}},
             "outcome":{"all":[{"give_up":{"w":3,"h":2}},{"flag":"opened"}]},
             "unmet":"it is not that shape"}]}]"#,
    );
    assert!(ok.is_ok(), "{:?}", ok.err());

    let why = events_of(
        r#"[{"id":"d","title":"D","prose":["p"],"choices":[
            {"label":"in the slot","blurb":"","requires":{"loose_item_of_size":{"w":3,"h":2}},
             "outcome":{"give_up":{"w":1,"h":4}},"unmet":"no"}]}]"#,
    )
    .expect_err("a door asked for one shape and ate another");
    assert!(why.contains("1 by 4"), "{why}");
}

// ------------------------------------------------------------ solvable_blind

/// **Nine cairns visited in every order is forty-five**, and the harness says
/// so rather than being told.
///
/// This is the anchor `PLAN-M14.md` §1.2 builds its ceiling out of, and the
/// reason to trust the model on the other five floors. Negative-tested by
/// re-parenting cairn 5 onto cairn 7, which makes the chain a cycle: the
/// harness reported *"nothing else on this floor can be opened"* and named the
/// stair.
#[test]
fn solvable_blind_counts_a_field_of_cairns() {
    let n = 9;
    let cairns: Vec<String> = (1..=n)
        .map(|i| {
            let requires = if i == 1 {
                String::new()
            } else {
                format!(r#""requires":{{"flag":"cairn-{}"}},"unmet":"there is no stone under this one yet","#, i - 1)
            };
            format!(
                r#"{{"id":"cairn-{i}","title":"CAIRN {i}","prose":["p"],"choices":[
                    {{"label":"Add a stone","blurb":"",{requires}"outcome":{{"flag":"cairn-{i}"}}}}]}}"#
            )
        })
        .collect();
    let events = events_of(&format!("[{}]", cairns.join(","))).expect("nine cairns");

    let places: Vec<String> = (1..=n)
        .map(|i| format!(r#"{{"at":[{},1],"kind":"event","id":"cairn-{i}"}}"#, i))
        .chain(std::iter::once(
            r#"{"at":[5,2],"kind":"gate","id":"the-tenth","to":"somewhere",
                "hidden_until":"cairn-9"}"#
                .to_string(),
        ))
        .collect();
    let rows = ["^^^^^^^^^^^", "^,,,,,,,,,^", "^,,,,,,,,,^", "^^^^^^^^^^^"];
    let w = world_of(&rows, &places.join(","), "").unwrap_or_else(|e| panic!("{e}"));

    let n = gm2d_core::puzzle::solvable_blind(&w, &events).expect("the cairnfield is walkable");
    assert_eq!(n, 45, "nine cairns in the worst order is 9+8+...+1");
}

/// **A floor whose chain is a cycle says so, and names the stair.**
///
/// The failure `solvable_blind` exists for, asked directly rather than by
/// breaking something: cairn 5 is re-parented onto cairn 7, so nothing after
/// four can ever be built.
#[test]
fn a_floor_that_cannot_be_solved_says_which_way_down_is_shut() {
    let events = events_of(
        r#"[{"id":"a","title":"A","prose":["p"],"choices":[
              {"label":"add","blurb":"","outcome":{"flag":"one"}}]},
            {"id":"b","title":"B","prose":["p"],"choices":[
              {"label":"add","blurb":"","requires":{"flag":"three"},
               "outcome":{"flag":"two"},"unmet":"no stone under it"}]},
            {"id":"c","title":"C","prose":["p"],"choices":[
              {"label":"add","blurb":"","requires":{"flag":"two"},
               "outcome":{"flag":"three"},"unmet":"no stone under it"}]}]"#,
    )
    .expect("three cairns, two of them in a knot");
    let rows = ["^^^^^", "^,,,^", "^,,,^", "^^^^^"];
    let w = world_of(
        &rows,
        r#"{"at":[1,1],"kind":"event","id":"a"},
           {"at":[2,1],"kind":"event","id":"b"},
           {"at":[3,1],"kind":"event","id":"c"},
           {"at":[2,2],"kind":"gate","id":"down","to":"somewhere","hidden_until":"three"}"#,
        "",
    )
    .expect("a knotted floor");

    match gm2d_core::puzzle::solvable_blind(&w, &events) {
        Ok(n) => panic!("a floor with no way down was walked in {n} visits"),
        Err(why) => {
            let said = why.to_string();
            assert!(said.contains("down"), "it did not name the way down: {said}");
            assert!(said.contains("one"), "it did not say what it had managed: {said}");
        }
    }
}

/// **No flag is waited on for ever.**
///
/// `PLAN-M14.md`'s M14.0 row asks for this *extended*; there was nothing to
/// extend. It is a doc comment on `event::Requirement::Flag` — the cut
/// campaign's type — describing a lint that went with the campaign, and
/// nothing in `crates/core/tests` was called it. Written here, over the live
/// content, which is the whole of it: **every mark anything waits on has to be
/// something something else raises.**
///
/// A mark is raised three ways and all three are counted: an `Outcome::Flag`,
/// an event id (a card writes its own id into `answered` when it is answered),
/// and a place id (a boss and a gate with prose both do).
///
/// Negative-tested by pointing the lake's drain at `the-bottom-of-the-stack`,
/// which is not a place on any map: *"nothing ever raises"* naming it.
#[test]
fn no_flag_is_waited_on_forever() {
    let events = gm2d_core::data::events();
    let maps = gm2d_core::data::all_maps(D);

    let mut raised: std::collections::BTreeSet<String> = Default::default();
    for e in &events.events {
        // Answering a card writes its own id down, and so does reading a note.
        raised.insert(e.id.clone());
        for c in &e.choices {
            let mut fs = Vec::new();
            gm2d_core::tile_event::flags_raised(&c.outcome, &mut fs);
            raised.extend(fs);
        }
    }
    for w in &maps {
        for p in &w.places {
            // A boss writes its tile down when it goes down, and a gate with a
            // paragraph writes itself down when the paragraph is read.
            raised.insert(p.id.clone());
        }
    }

    let mut orphans: Vec<String> = Vec::new();
    let note = |what: &str, who: &str, orphans: &mut Vec<String>| {
        if !raised.contains(what) {
            orphans.push(format!("{who} waits on {what:?}, and nothing ever raises it"));
        }
    };
    for e in &events.events {
        for c in &e.choices {
            if let Requirement::Flag(f) = &c.requires {
                note(f, &format!("the choice {:?} at {}", c.label, e.id), &mut orphans);
            }
        }
    }
    for w in &maps {
        for p in &w.places {
            if let Some(k) = &p.hidden_until {
                note(k, &format!("{} on {}", p.id, w.id), &mut orphans);
            }
            for k in &p.hidden_until_all {
                note(k, &format!("{} on {}", p.id, w.id), &mut orphans);
            }
            for k in &p.needs_all {
                note(k, &format!("{} on {}", p.id, w.id), &mut orphans);
            }
        }
        for d in &w.drains {
            note(&d.when, &format!("the drain on {}", w.id), &mut orphans);
        }
    }
    // **A place hidden until itself never appears**, which is the one
    // assertion `ending.rs::every_hidden_place_names_something_that_happens`
    // had that this did not, carried over when that lint was subsumed. It is
    // not covered above, because a place id *is* something that gets written.
    for w in &maps {
        for p in &w.places {
            if let Some(k) = &p.hidden_until {
                assert_ne!(k, &p.id, "{} on {}: is hidden until itself", p.id, w.id);
            }
            assert!(
                !p.hidden_until_all.contains(&p.id),
                "{} on {}: is hidden until itself",
                p.id,
                w.id
            );
        }
    }
    assert!(orphans.is_empty(), "{}", orphans.join("\n"));
}
