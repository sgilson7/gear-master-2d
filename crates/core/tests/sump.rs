//! M14.2 — the Wextreen Sump: four floors, three puzzles and the Ninth
//! Surveyor.
//!
//! **The counts are asserted at their number**, never as `< 100`: a ceiling
//! nobody can hit is a ceiling nobody checked, which is the
//! `(2..5).contains(&taken)` failure M11.7 lost a whole block to.

use gm2d_core::combat::{self, Difficulty, Outcome};
use gm2d_core::data;
use gm2d_core::puzzle;
use gm2d_core::tile_event::Requirement;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const FLOORS: &[&str] = &["the-sump-1", "the-sump-2", "the-sump-3", "the-sump-4"];

mod common;

/// **Every floor is walkable by somebody who does not know the answer, and
/// here is what it costs them.**
///
/// | floor | blind | the plan said |
/// |---|---|---|
/// | the Lip | **8** | 10 |
/// | the Shelf | **1** | 11 |
/// | the Cairnfield | **45** | 45 |
///
/// The Cairnfield agrees exactly, which is the number `PLAN-M14.md` §1.2 builds
/// its whole ceiling out of and the reason to believe the other two.
///
/// The Lip is 8 rather than 10 because a card on the far side of a channel is
/// not a card yet — the model floods the floor as the flags have left it, which
/// the first version did not. **The Shelf is 1 and the plan's 11 is a fiction
/// about an engine this is not**: §4.2 says *"a player cycles their tray"*, and
/// there is no cycling — a footprint requirement is met or it is not, and the
/// card says which. What the Shelf actually costs is a component, and
/// `the_shelf_takes_a_3x2_and_keeps_it` is where that is measured.
///
/// Negative-tested by re-parenting the fourth cairn onto the seventh: the
/// harness reported the field unsolvable and named the tenth cairn.
#[test]
fn each_sump_floor_is_solvable_blind_at_its_number() {
    let events = data::events();
    let want = [("the-sump-1", 8usize), ("the-sump-2", 1), ("the-sump-3", 45)];
    for (id, n) in want {
        let w = data::map(id, D);
        let got = puzzle::solvable_blind(&w, &events)
            .unwrap_or_else(|why| panic!("{id} cannot be solved blind: {why}"));
        assert_eq!(got, n, "{id} takes {got} card reads blind and the number is {n}");
    }
    // And the bottom is a fight rather than a puzzle: nothing on it is hidden.
    let bottom = data::map("the-sump-4", D);
    assert!(
        bottom.places.iter().all(|p| p.hidden_until.is_none() && p.hidden_until_all.is_empty()),
        "the floor of the Sump has a puzzle on it as well as the thing at the bottom"
    );
}

/// **An instrument makes a floor short, and it is never the only way through.**
///
/// The two halves of `PLAN-M14.md` §1.2, and they are measured in different
/// currencies because the three floors charge in different currencies — which
/// is the finding, and it is why the plan's *1 / 1 / 9* does not survive
/// contact:
///
/// - **the Lip** charges **fatigue**. Blind you pace the bearing out at twelve
///   points; with the compass you read it off the plate.
/// - **the Shelf** charges a **component**. Blind you leave a three-by-two in
///   the slot; with the atlas you read the shorthand and set the catch.
/// - **the Cairnfield** charges **walking**, and it is the only one of the
///   three where the instrument moves the count: **45 card reads to 1.**
#[test]
fn each_sump_floor_is_shorter_with_its_instrument() {
    let events = data::events();

    // The Cairnfield: nine cairns by hand, or one slab. **Shortest against
    // shortest**, which is the only pair in the same unit — see the note where
    // `solvable_blind_with` is not.
    let field = data::map("the-sump-3", D);
    assert_eq!(
        puzzle::solvable_knowing(&field, &events, None),
        Ok(9),
        "the field is not nine cairns to somebody laying them by hand"
    );
    assert_eq!(
        puzzle::solvable_knowing(&field, &events, Some("golem")),
        Ok(1),
        "the golem read the field and it still took more than standing on the slab"
    );
    // And the wrong instrument is no instrument.
    assert_eq!(puzzle::solvable_knowing(&field, &events, Some("compass")), Ok(9));
    assert_eq!(puzzle::solvable_knowing(&field, &events, Some("atlas")), Ok(9));

    // The Lip: the compass takes the twelve points of fatigue off.
    let plate = events.get("the-bearing-plate").expect("the plate");
    let compass = plate
        .choices
        .iter()
        .find(|c| c.requires == Requirement::Surveying("compass".into()))
        .expect("the plate does not want a compass");
    let paced = plate
        .choices
        .iter()
        .find(|c| c.requires == Requirement::None)
        .expect("there is no way to pace it out");
    let tires = |o: &gm2d_core::tile_event::Outcome| {
        o.describe().iter().any(|l| l.contains("more tired"))
    };
    assert!(!tires(&compass.outcome), "reading it with a compass costs fatigue");
    assert!(tires(&paced.outcome), "pacing it out is free, so the compass buys nothing");

    // The Shelf: the atlas takes the component off.
    let door = events.get("the-weighed-door").expect("the door");
    let keeps = |c: &gm2d_core::tile_event::Choice| {
        c.outcome.describe().iter().any(|l| l.contains("stays in it"))
    };
    let by_atlas = door
        .choices
        .iter()
        .find(|c| c.requires == Requirement::Flag("read-the-shelf".into()))
        .expect("the door has no way through for somebody who read the lintel");
    assert!(!keeps(by_atlas), "reading the shorthand still costs you the piece");
    assert!(
        door.choices.iter().any(keeps),
        "nothing at the door costs a component, so there is nothing for the atlas to save"
    );
}

/// **No door in the game is opened only by an instrument.**
///
/// `PLAN-M14.md` §1.2's hard half. Stated over the *place* rather than over the
/// choice, because that is where it matters: the lintel wants an atlas and
/// nothing else, and the lintel is not a door — the door it talks about has
/// three other ways through. What may never happen is a **stair** whose flag
/// only a survey can raise.
///
/// The walk is the flag graph with every `Surveying` choice deleted: if the
/// stair's mark is still reachable, somebody without an instrument can get
/// there. Negative-tested by deleting the plate's *"Pace it out"*, which left
/// the compass as the only way onto floor two and named the first stair.
#[test]
fn an_instrument_is_never_the_only_way_through() {
    let events = data::events();
    // Every flag raisable without ever reading anything, taken to a fixed
    // point — a chain is as long as it is.
    let mut open: std::collections::BTreeSet<String> = Default::default();
    loop {
        let before = open.len();
        for e in &events.events {
            for c in &e.choices {
                let ok = match &c.requires {
                    Requirement::Surveying(_) => false,
                    Requirement::Flag(f) => open.contains(f),
                    _ => true,
                };
                if ok {
                    let mut fs = Vec::new();
                    gm2d_core::tile_event::flags_raised(&c.outcome, &mut fs);
                    open.extend(fs);
                }
            }
        }
        // Answering a card writes its own id down, and a boss its tile.
        open.extend(events.events.iter().map(|e| e.id.clone()));
        for (id, _) in data::MAPS {
            open.extend(data::map(id, D).places.iter().map(|p| p.id.clone()));
        }
        if open.len() == before {
            break;
        }
    }

    let mut shut = Vec::new();
    for (id, _) in data::MAPS {
        for p in &data::map(id, D).places {
            for k in p.hidden_until.iter().chain(p.hidden_until_all.iter()).chain(p.needs_all.iter())
            {
                if !open.contains(k) {
                    shut.push(format!("{id}/{}: only a survey ever raises {k:?}", p.id));
                }
            }
        }
    }
    assert!(shut.is_empty(), "{}", shut.join("\n"));
    // Not vacuous: there are survey-gated choices to be wrong about.
    assert!(
        events
            .events
            .iter()
            .flat_map(|e| e.choices.iter())
            .filter(|c| matches!(c.requires, Requirement::Surveying(_)))
            .count()
            >= 3,
        "nothing in the game wants an instrument, so this checks nothing"
    );
}

/// **One of the three wheels is not needed, and it is the one that costs you.**
///
/// Channel C is walked round at its dry east end, so the wheel that opens it
/// buys a crossing that is already there — and it is the only wheel in the game
/// that keeps what you feed it. The floor is a tax on not looking.
///
/// Measured as *what wheel C adds to the floor*, which is the honest way to ask
/// it: every event you can reach with C turned, you could reach without it.
/// Negative-tested by drawing [9, 8] and [10, 8] as water, which made the wheel
/// compulsory and this named it.
#[test]
fn the_lip_needs_one_wheel() {
    let w = data::map("the-sump-1", D);
    let reach = |flags: &[&str]| -> std::collections::BTreeSet<[u8; 2]> {
        let mut st = WorldState::default();
        st.map = "the-sump-1".into();
        st.flags = flags.iter().map(|s| s.to_string()).collect();
        let m = data::map_now("the-sump-1", D, &st);
        let a = Allowances::default();
        let mut seen: std::collections::BTreeSet<[u8; 2]> = Default::default();
        let mut q = vec![[m.start.0, m.start.1]];
        seen.insert([m.start.0, m.start.1]);
        while let Some([x, y]) = q.pop() {
            for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if !m.in_bounds(nx, ny) {
                    continue;
                }
                let (nx, ny) = (nx as u8, ny as u8);
                if m.walkable(nx, ny, &a) && seen.insert([nx, ny]) {
                    q.push([nx, ny]);
                }
            }
        }
        seen
    };
    let at = |id: &str| w.places.iter().find(|p| p.id == id).expect("a place").at;

    // Wheel A is on the far side of channel C and you get to it round the dry
    // end, without turning wheel C at all.
    let dry = reach(&[]);
    assert!(dry.contains(&at("the-wheel-that-is-marked-a")), "wheel A is unreachable");
    assert!(dry.contains(&at("the-wheel-that-is-marked-c")), "wheel C is unreachable");
    assert!(!dry.contains(&at("the-bearing-plate")), "the plate is not behind channel A");

    // And turning C adds nothing a place stands on.
    let with_c = reach(&["sluice-c"]);
    for p in &w.places {
        assert!(
            !with_c.contains(&p.at) || dry.contains(&p.at),
            "{} is only reachable once wheel C is turned, so wheel C is not the tax",
            p.id
        );
    }

    // Two wheels and the plate reach the stair, and the third is never wanted.
    let two = reach(&["sluice-a", "sluice-b"]);
    assert!(
        two.contains(&at("the-sump-1-stair")),
        "the way down is not reachable with the two wheels that matter"
    );
}

/// **The door keeps the three-by-two, and the bag is one lighter for it.**
///
/// Negative-tested by dropping the `give_up` out of the outcome: the door came
/// up and the piece was still in the tray, which is a price that is not one.
#[test]
fn the_shelf_takes_a_3x2_and_keeps_it() {
    let mut g = gm2d_core::game::Game::default();
    g.character = gm2d_core::character::Character::with_all_pieces();
    let before = g.character.owned.len();
    let had = g.character.loose_of_size(3, 2).len();
    assert!(had > 0, "the fixture has no three-by-two, so this proves nothing");

    let n = data::events()
        .get("the-weighed-door")
        .expect("the door")
        .choices
        .iter()
        .position(|c| c.requires == Requirement::LooseItemOfSize { w: 3, h: 2 })
        .expect("nothing at the door wants a three-by-two");
    let receipt = g.answer_event("the-weighed-door", n, D).expect("the slot took it");

    assert!(
        receipt.iter().any(|l| l.contains("stays in it")),
        "the door ate a component and said nothing: {receipt:?}"
    );
    assert_eq!(g.character.owned.len(), before - 1, "the bag is not one lighter");
    assert_eq!(g.character.loose_of_size(3, 2).len(), had - 1);
    assert!(g.world.flags.iter().any(|f| f == "the-shelf-is-open"), "the door did not come up");
}

/// **A cairn says there is nothing under it and never which cairn that is.**
///
/// `PROMPT-M14.md`'s standing constraint, and the one place in this block where
/// a refusal deliberately stops short of naming the thing in the way — because
/// naming it *is* the puzzle. TONE rule 12 is satisfied by naming a cairn
/// without naming which.
///
/// Negative-tested by writing the surveyor's number into each cairn's title:
/// the field became a numbered list and this said so.
#[test]
fn a_cairn_says_nothing_about_which_is_under_it() {
    let events = data::events();
    let field = data::map("the-sump-3", D);
    let cairns: Vec<_> = field
        .places
        .iter()
        .filter(|p| p.id.starts_with("the-cairn-at-"))
        .filter_map(|p| events.get(&p.id))
        .collect();
    assert_eq!(cairns.len(), 9, "the Cairnfield does not have nine cairns on it");

    // One title, nine heaps. A card that named itself would be the clipboard
    // for nothing.
    let titles: std::collections::BTreeSet<&str> = cairns.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles.len(), 1, "the cairns have {} different titles", titles.len());

    for e in &cairns {
        for c in &e.choices {
            let said = format!("{} {}", c.unmet, e.prose.join(" ")).to_lowercase();
            assert!(
                !said.contains("cairn-"),
                "{}: a card names a cairn by its id",
                e.id
            );
            for ord in [
                "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth",
                "ninth",
            ] {
                assert!(
                    !c.unmet.to_lowercase().contains(ord),
                    "{}: the refusal says {ord:?}, which is the answer",
                    e.id
                );
            }
        }
    }

    // And the heights are what tells them apart, so they had better differ.
    let mut heights: Vec<&str> = Vec::new();
    for e in &cairns {
        let first = e.prose[0].as_str();
        let h = first.split(" high").next().unwrap_or("");
        heights.push(h);
    }
    let uniq: std::collections::BTreeSet<&&str> = heights.iter().collect();
    assert_eq!(uniq.len(), 9, "two cairns are the same height, so the sheet cannot tell them apart");
}

/// **The Ninth Surveyor is a fight the board a player actually has can win, and
/// it is harder than the one at the bottom of the lake.**
///
/// Named for what it measures. `PLAN-M14.md` M14.2 calls this
/// `the_ninth_surveyor_is_beatable_by_the_walker_at_22`, and **a level-22 board
/// is not a board this game produces** — the shipped transcript ends at level
/// fourteen. `common::geared_from` is what M11.7 established as *the board a
/// player actually has*, and it is what every reachability question in this
/// repository has been asked of since.
///
/// **Harder is measured in damage per second, not in rating**, which is the
/// recon this creature was dressed against: the same board beats Francis at
/// 2958 and loses to Cairn Chorus at 1141, so a rating predicts nothing. The
/// first draft of her weapon grid was a hilt and two accessories, which
/// assemble nothing — she dealt 7.8 damage a second and dealt exactly the same
/// at strength 152 and at 320, because strength pays a swing and there was no
/// swing to pay.
#[test]
fn the_ninth_surveyor_is_a_fight_the_board_wins() {
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let dps = |name: &str| -> (Outcome, f64) {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        let log = combat::simulate_at(ch.player_stats(), &ch.combat_items(), m, D);
        let ms = log.entries.last().map(|e| e.at_ms).unwrap_or(1).max(1);
        let dealt: i64 = log
            .entries
            .iter()
            .filter_map(|e| match e.event {
                combat::Event::Hit { by: combat::Side::Enemy, damage, absorbed, .. } => {
                    Some((damage - absorbed).max(0) as i64)
                }
                _ => None,
            })
            .sum();
        (log.outcome, dealt as f64 * 1000.0 / ms as f64)
    };

    let (out, hers) = dps("The Ninth Surveyor");
    assert_eq!(out, Outcome::Victory, "nothing behind her can ever be reached");
    let (_, soot) = dps("Sootmother");
    assert!(
        hers > soot * 3.0,
        "she deals {hers:.1} a second against Sootmother's {soot:.1}, which is not a step down"
    );
    // And she is under the two that beat this board, so she is a fight and not
    // a wall. Gilt deals 428 and Nine of Ashes 531, and both of them win.
    let (gilt, gilt_dps) = dps("Gilt");
    assert_eq!(gilt, Outcome::Defeat, "Gilt stopped beating this board, so the bracket moved");
    assert!(hers < gilt_dps, "she hits harder than the thing this board loses to");

    // Her three drops are catalogue and one of them is the golem's.
    let floor = data::map("the-sump-4", D);
    let boss = floor.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("a boss");
    assert_eq!(boss.creature.as_deref(), Some("The Ninth Surveyor"));
    for d in &boss.drops {
        assert!(
            gm2d_core::piece::CATALOG.iter().any(|c| c.name == d),
            "{d:?} is not in the catalogue, so M14 added a component"
        );
    }
    assert!(
        boss.drops.iter().any(|d| d == "Living Earth"),
        "the golem wants two Living Earth and the only other source is a 300 per mille roll"
    );
}

/// **The Sump is not one sitting, and every floor has a way back up.**
///
/// The Drambus Stack's budget is fatigue, so it is one sitting and the tower
/// kicks you out. This one's budget is *what you brought*: a player who needs a
/// three-by-two from the van can go and get it, and what that costs is the
/// walk. So no floor names an `outside`, and a save taken on one reopens on it.
#[test]
fn the_sump_is_not_one_sitting() {
    for id in FLOORS {
        let w = data::map(id, D);
        assert!(w.outside.is_none(), "{id} is one sitting, so what you brought does not matter");
        let up = w
            .places
            .iter()
            .find(|p| p.id.ends_with("-up"))
            .unwrap_or_else(|| panic!("{id} has no way back up"));
        assert_eq!(up.kind, PlaceKind::Gate);
        let to = up.to.as_deref().expect("a stair to nowhere");
        assert!(data::MAPS.iter().any(|(m, _)| *m == to), "{id}: the stair up goes to {to:?}");
        // And a save taken on it reopens on it.
        let mut st = WorldState::default();
        st.map = (*id).to_string();
        st.at = [w.start.0, w.start.1];
        assert!(
            gm2d_core::world::leave_the_sitting(&mut st, D).is_none(),
            "{id}: a save taken here reopens somewhere else"
        );
    }
}
