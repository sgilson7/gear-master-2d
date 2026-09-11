//! M14.3 — the Silt Stair, behind the door under the lake.
//!
//! The motif is *carrying*: the stair was cut for people carrying something,
//! and the first floor will not take you down without something in the groove.

use gm2d_core::combat::{self, Difficulty, Outcome};
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::puzzle;
use gm2d_core::tile_event::Requirement;
use gm2d_core::world::{Allowances, PlaceKind, WorldState};

const D: Difficulty = Difficulty::Easy;
const FLOORS: &[&str] = &[
    "the-silt-stair-1",
    "the-silt-stair-2",
    "the-silt-stair-3",
    "the-silt-stair-4",
];

mod common;

/// **Every floor is walkable blind, and here is what it costs.**
///
/// | floor | blind | the plan said |
/// |---|---|---|
/// | the Landing | **1** | 2 |
/// | the Chair Room | **3** | 27 |
/// | the Drowned Gallery | **3** | 3 |
///
/// The Gallery agreed, and it has since stopped being this kind of puzzle: the
/// chains still flood and drain it in the same three reads, and what opens the
/// stair now is the stones. See `the_gallery_is_two_puzzles_now`.
#[test]
fn each_stair_floor_is_solvable_blind_at_its_number() {
    let events = data::events();
    // **The Gallery is not on this list any more**, and that is the change
    // rather than a gap: its way on is three stones pushed onto three marks,
    // not a flag off a card, so `solvable_blind` correctly reports that
    // nothing on the floor opens the stair. What is still true of it —
    // both halves — is `the_gallery_is_two_puzzles_now`.
    for (id, n) in [("the-silt-stair-1", 1usize), ("the-silt-stair-2", 3)] {
        let w = data::map(id, D);
        let got = puzzle::solvable_blind(&w, &events)
            .unwrap_or_else(|why| panic!("{id} cannot be solved blind: {why}"));
        assert_eq!(got, n, "{id} takes {got} card reads blind and the number is {n}");
    }
}

/// **The chair is three moves, and `PLAN-M14.md` §5.2 asks for nine.**
///
/// The sequence is Marbulon's and it is already written, in the prose on her
/// own door on the first map: *"She turns the chair round to face the door,
/// counts to four under her breath, and turns it back. She does this twice more
/// while you are standing there."* Face, four, back — and she does it three
/// times because she is nervous.
///
/// **Nine rungs is not expressible in monotone flags over three
/// always-offered labels, and §1.1 is what says so.** A choice carries one
/// requirement and one outcome, so "Turn it to face the door" has to raise one
/// flag; to be the first, fourth and seventh move it would have to raise three
/// different ones. The alternatives are both worse: nine choices puts the
/// answer on the card as a list of labels, and a counter with a modulus in it
/// is a flag that goes down, which the save cannot come back from.
///
/// So the player does it once and the chair opens. The three-times is hers.
///
/// Negative-tested by ungating "Count to four": the room became one move and
/// the sequence stopped being one.
#[test]
fn the_chair_is_three_moves_and_not_nine() {
    let events = data::events();
    let chair = events.get("the-chair").expect("no chair");
    assert!(chair.repeats, "the chair is spent after one move, so it is a card and not a lock");
    assert_eq!(chair.choices.len(), 3, "the chair does not offer three things");
    // **All three, always.** A room that greys out the wrong answers is a room
    // that tells you the answer.
    assert!(
        chair.choices.iter().all(|c| !c.label.is_empty()),
        "a nameless choice"
    );

    let mut g = Game::default();
    let want = ["chair-facing", "chair-counted", "chair-done"];
    // Out of order, the room says no and nothing moves.
    assert!(g.answer_event("the-chair", 1, D).is_err(), "counted in front of a chair facing away");
    assert!(g.answer_event("the-chair", 2, D).is_err(), "turned back a chair nobody counted at");
    assert!(g.world.flags.is_empty(), "a refused move left a mark");

    for (i, flag) in want.iter().enumerate() {
        g.answer_event("the-chair", i, D)
            .unwrap_or_else(|why| panic!("move {i} of the sequence: {why}"));
        assert!(g.world.flags.iter().any(|f| f == flag), "move {i} did not raise {flag}");
    }
    // **Three answers at one object**, which is what `repeats` is for: an
    // ordinary card refuses the second.
    assert!(
        !g.world.answered.iter().any(|a| a == "the-chair"),
        "the chair was written down, so it is spent"
    );

    // The door it is facing is behind the third and nothing else.
    let room = data::map("the-silt-stair-2", D);
    let door = room
        .places
        .iter()
        .find(|p| p.id == "the-silt-stair-2-stair")
        .expect("no door in the north wall");
    assert_eq!(door.hidden_until.as_deref(), Some("chair-done"));

    // And she is the one who taught it to you, on the first map.
    let hers = events.get("marbulons-door").expect("Marbulon");
    let said = hers.prose.join(" ").to_lowercase();
    for word in ["face the door", "counts to four", "turns it back"] {
        assert!(said.contains(word), "the sequence is not in her prose: {word:?}");
    }
}

/// **Chain B will not move in a dry room, and says which room.**
///
/// Negative-tested by dropping B's requirement: the gallery came up without
/// ever going under, which is a stair that was never hidden.
#[test]
fn chain_b_will_not_move_dry() {
    let events = data::events();
    let b = events.get("the-chain-marked-b").expect("no second chain");
    let pull = &b.choices[0];
    assert_eq!(pull.requires, Requirement::Flag("gallery-flooded".into()));
    assert!(
        pull.unmet.to_lowercase().contains("dry"),
        "the refusal does not say the room is dry: {:?}",
        pull.unmet
    );

    let mut g = Game::default();
    assert!(g.answer_event("the-chain-marked-b", 0, D).is_err(), "B moved in a dry room");
    g.answer_event("the-chain-marked-a", 0, D).expect("A pulls");
    g.answer_event("the-chain-marked-b", 0, D).expect("B pulls once the room is wet");
    assert!(g.world.flags.iter().any(|f| f == "gallery-drained"));
}

/// **The gallery drains to silt and not to lakebed**, and it goes under water
/// on the way.
///
/// Eleven inches of it were banked against the door under the lake, and the
/// door is the point. Negative-tested by draining it to `lakebed`: the terrain
/// came back as the thing a lake leaves rather than the thing that was banked.
#[test]
fn the_gallery_drains_to_silt_and_not_lakebed() {
    let flooded = |flags: &[&str]| {
        let mut st = WorldState::default();
        st.map = "the-silt-stair-3".into();
        st.flags = flags.iter().map(|s| s.to_string()).collect();
        data::map_now("the-silt-stair-3", D, &st)
    };
    let dry = flooded(&[]);
    assert_eq!(dry.terrain_name(5, 5), "road", "the gallery does not start as a floor");

    let wet = flooded(&["gallery-flooded"]);
    assert_eq!(wet.terrain_name(5, 5), "water", "chain A did not flood it");
    assert!(!wet.passable(5, 5), "a flooded gallery is still a floor");

    let done = flooded(&["gallery-flooded", "gallery-drained"]);
    assert_eq!(done.terrain_name(5, 5), "silt", "the gallery came back as something else");
    assert!(done.passable(5, 5));

    // And the ring outside it never moved.
    for w in [&dry, &wet, &done] {
        // **The ring outside the gallery is road**, like every other dungeon
        // floor in the game — see the map's own note. What matters here is that
        // it does not move when the gallery does.
        assert_eq!(w.terrain_name(1, 5), "road", "the walk round the gallery went with it");
        assert_eq!(w.terrain_name(5, 9), "road", "the tile the stair stands on drained");
    }
}

/// **The wading shortcut is worth drawing, and here is the number.**
///
/// `PLAN-M14.md` §9 decision 4 leaves it open — *"if it saves nothing it is
/// cut"* — and asks the recon to answer. It saves **eight tiles**: the chains
/// are in opposite walls, so a flooded gallery is seventeen tiles round and
/// nine across, and the nine are only across for somebody wearing the frame
/// three Bog Toads pay for.
///
/// **Flooding the room makes the walk worse**, which is the design rather than
/// an accident: chain A costs you the crossing you had, and the Toad's Own
/// Frame is what gives it back.
#[test]
fn a_wader_reaches_chain_b_on_water() {
    let mut st = WorldState::default();
    st.map = "the-silt-stair-3".into();
    st.flags = vec!["gallery-flooded".into()];
    let w = data::map_now("the-silt-stair-3", D, &st);

    // **Read off the map rather than written down here.** The first version
    // hardcoded [1, 5] and [10, 5], so moving a chain moved the puzzle and not
    // the test — which the negative pass caught by putting B beside A and
    // watching this still pass.
    let at = |id: &str| {
        w.places.iter().find(|p| p.id == id).unwrap_or_else(|| panic!("no {id}")).at
    };
    let steps = |wade: bool| -> Option<usize> {
        let a = Allowances { wade, level: 99 };
        let from = at("the-chain-marked-a");
        let to = at("the-chain-marked-b");
        let mut seen: std::collections::BTreeMap<[u8; 2], usize> = Default::default();
        let mut q = std::collections::VecDeque::new();
        seen.insert(from, 0);
        q.push_back(from);
        while let Some([x, y]) = q.pop_front() {
            let d = seen[&[x, y]];
            if [x, y] == to {
                return Some(d);
            }
            for (dx, dy) in [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let (nx, ny) = (nx as u8, ny as u8);
                if w.walkable(nx, ny, &a) && !seen.contains_key(&[nx, ny]) {
                    seen.insert([nx, ny], d + 1);
                    q.push_back([nx, ny]);
                }
            }
        }
        None
    };

    let round = steps(false).expect("chain B is unreachable on foot with the room flooded");
    let across = steps(true).expect("chain B is unreachable even to a wader");
    assert_eq!(across, 9, "the crossing is not nine tiles");
    assert_eq!(round, 17, "the walk round is not seventeen tiles");
    assert!(round > across, "wading saves nothing, so the shortcut is a sentence and not a design");
}

/// **The stair takes a one-by-four, or the lens that was carried down once
/// already.**
///
/// Nineteen catalogue pieces are one by four and fifteen of them are things a
/// player can get. The Cracked Lens is Sootmother's drop, so somebody who came
/// through the lake the intended way is already holding the key — and the
/// manifest on the wall says so, in a hand that stops after *a lens, cracked*.
#[test]
fn the_lens_carries_you_down() {
    let events = data::events();
    let stair = events.get("the-cut-stair").expect("no cut stair");
    assert_eq!(stair.choices.len(), 2, "the stair takes one thing only");

    // The groove keeps a component and does not keep the lens.
    let lays = &stair.choices[0];
    assert_eq!(lays.requires, Requirement::LooseItemOfSize { w: 1, h: 4 });
    assert!(lays.outcome.describe().iter().any(|l| l.contains("stays in it")));

    let carries = &stair.choices[1];
    assert_eq!(carries.requires, Requirement::Holding("The Cracked Lens".into()));
    assert!(
        !carries.outcome.describe().iter().any(|l| l.contains("stays in it")),
        "carrying the lens down costs you the lens"
    );

    // And it is a thing the game hands out, off the boss two hundred and six
    // steps above this one.
    let lake = data::map("under-the-lake", D);
    let boss = lake.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("the lake's boss");
    assert!(
        boss.drops.iter().any(|d| d == "The Cracked Lens"),
        "nothing in the game hands over The Cracked Lens"
    );

    // A player holding it goes down without spending anything.
    let mut g = Game::default();
    g.character.give("The Cracked Lens");
    let owned = g.character.owned.len();
    g.answer_event("the-cut-stair", 1, D).expect("the lens goes in the groove");
    assert_eq!(g.character.owned.len(), owned, "the groove kept the lens");
    assert!(g.world.flags.iter().any(|f| f == "the-stair-takes-it"));

    // The manifest is a list and not a lock: it says the lens went down and
    // asks nothing.
    let m = events.get("the-manifest").expect("no manifest");
    assert!(m.is_examinable(), "the manifest asks something, so it is a lock");
    assert!(
        m.prose.join(" ").to_lowercase().contains("lens, cracked"),
        "the manifest does not say what was carried down"
    );
}

/// **What Marbulon Faced Away From is a fight the board wins, and it is the
/// harder of the two bottoms.**
///
/// One map's worth of walking deeper than the Ninth Surveyor and dressed
/// against the same bracket: what decides a fight for this board is damage a
/// second, and the ceiling is whatever Gilt deals, because Gilt beats it.
#[test]
fn what_marbulon_faced_away_from_is_the_deeper_of_the_two() {
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

    let (out, hers) = dps("What Marbulon Faced Away From");
    assert_eq!(out, Outcome::Victory, "nothing behind her can ever be reached");
    let (_, ninth) = dps("The Ninth Surveyor");
    assert!(
        hers > ninth,
        "she deals {hers:.1} a second against the Ninth Surveyor's {ninth:.1}, and she is \
         one map deeper"
    );
    let (_, gilt) = dps("Gilt");
    assert!(hers < gilt, "she hits harder than the thing this board loses to");

    // Her third drop is the golem's, the same as the Surveyor's, and between
    // them the two bottoms make one buildable off certainties.
    let floor = data::map("the-silt-stair-4", D);
    let boss = floor.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("a boss");
    assert_eq!(boss.creature.as_deref(), Some("What Marbulon Faced Away From"));
    assert!(boss.drops.iter().any(|d| d == "Living Earth"));
    for d in &boss.drops {
        assert!(
            gm2d_core::piece::CATALOG.iter().any(|c| c.name == d),
            "{d:?} is not in the catalogue"
        );
    }
    let sump = data::map("the-sump-4", D);
    let other = sump.places.iter().find(|p| p.kind == PlaceKind::Boss).expect("a boss");
    let earths = boss.drops.iter().filter(|d| *d == "Living Earth").count()
        + other.drops.iter().filter(|d| *d == "Living Earth").count();
    assert_eq!(earths, 2, "the golem wants two Living Earth and the two bottoms drop {earths}");
}

/// **The Stair is not one sitting, and the gear cannot post you out of it.**
///
/// `no_homeward` for the lake's own reason one map further down: it is the one
/// place where the walk *is* the content, and a set that posted you out of it
/// would delete the thing the two hundred and six steps cost.
#[test]
fn nothing_rides_home_out_of_the_stair() {
    for id in FLOORS {
        let w = data::map(id, D);
        assert!(w.no_homeward, "{id}: the Drover's Stride posts you out of a stair under a lake");
        assert!(w.outside.is_none(), "{id} is one sitting");
        let up = w
            .places
            .iter()
            .find(|p| p.id.ends_with("-up"))
            .unwrap_or_else(|| panic!("{id} has no way back up"));
        assert_eq!(up.kind, PlaceKind::Gate);
    }
    // And the way in is the door under the lake, which is a way on now.
    let lake = data::map("under-the-lake", D);
    let door = lake
        .places
        .iter()
        .find(|p| p.id == "the-door-under-the-lake")
        .expect("the door");
    assert_eq!(door.to.as_deref(), Some("the-silt-stair-1"));
    assert!(lake.no_homeward, "the lake stopped being the lake");
}


/// **The Gallery is two puzzles now, and the stair is behind the second.**
///
/// The chains are unchanged — A floods it, B drains what A made, and B will not
/// move in a dry room — and draining it no longer opens the way down. What does
/// is three stones on three marks, on the silt the water left.
///
/// **Pushing is the one shape this engine's rule forbids**, which is the whole
/// reason the floor is built the way it is: every other puzzle here is monotone
/// because flags only grow, and *a stone pushed into a corner is exactly the
/// move that makes the way on unreachable*. So the stones are a fact about this
/// visit rather than about the run — walk up the stair and back down and the
/// room is as it was — and that is what `the_stones_come_back_when_you_do`
/// holds.
#[test]
fn the_gallery_is_two_puzzles_now() {
    let allowed = gm2d_core::world::Allowances::default();
    let stair = |st: &gm2d_core::world::WorldState| {
        let w = data::map_now("the-silt-stair-3", D, st);
        w.places
            .iter()
            .find(|p| p.id == "the-silt-stair-3-stair")
            .map(|p| gm2d_core::world::place_is_there(p, st, &allowed))
            .unwrap_or(false)
    };

    // Draining it is no longer enough.
    let mut st = gm2d_core::world::WorldState::default();
    st.map = "the-silt-stair-3".into();
    assert!(!stair(&st), "the stair is there in a dry gallery");
    st.flags.push("gallery-flooded".into());
    st.flags.push("gallery-drained".into());
    assert!(!stair(&st), "draining the gallery still opens the stair on its own");

    // The stones do it, and they are pushable.
    let w = data::map_now("the-silt-stair-3", D, &st);
    assert_eq!(
        puzzle::stones_solvable(&w),
        Some(25),
        "the Gallery's stones no longer go into place in twenty-five steps"
    );
    st.flags.push(w.blocks.when_set.clone());
    assert!(stair(&st), "every mark has a stone on it and the stair is still not there");
}

/// **The stones come back when you do**, which is what makes a pushing puzzle
/// safe in a game whose every other puzzle is monotone.
#[test]
fn the_stones_come_back_when_you_do() {
    let mut st = gm2d_core::world::WorldState::default();
    st.map = "the-silt-stair-3".into();
    st.flags.push("gallery-drained".into());
    let w = data::map_now("the-silt-stair-3", D, &st);

    let first = gm2d_core::world::stones_now(&w, &st);
    assert_eq!(first, w.blocks.at, "the stones did not start where the map file puts them");

    // Jam one somewhere useless, the way a player can.
    st.blocks = Some(gm2d_core::world::Blocks {
        map: "the-silt-stair-3".into(),
        at: vec![[2, 2], [2, 3], [3, 2]],
    });
    assert_eq!(gm2d_core::world::stones_now(&w, &st).len(), 3, "a jam lost a stone");

    // Walk out — any other floor — and back in. **Through `go_to`**, which is
    // the one door onto a map and is where leaving is noticed: setting `map`
    // by hand is what six of the seven callers used to do and is exactly the
    // bug this test found.
    st.go_to("the-silt-stair-2");
    st.go_to("the-silt-stair-3");
    assert_eq!(
        gm2d_core::world::stones_now(&w, &st),
        w.blocks.at,
        "coming back down left the stones where they were jammed, so the floor is a dead end"
    );
}
