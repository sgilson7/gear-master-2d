//! M22.7 — what stands in the cup.
//!
//! The plank with eleven names burned into it and a twelfth cut fresh. The
//! first thing on the first map has been a plank burning through names since
//! M6; this is what is at the end of the list.

use gm2d_core::combat::{self, Difficulty, Event, Outcome, Side};
use gm2d_core::data;

mod common;

const D: Difficulty = Difficulty::Easy;
const IT: &str = "The Twelfth Name";

/// What a board does to a creature, and what it takes back.
fn bout(ch: &gm2d_core::character::Character, name: &str) -> (Outcome, u32, f32) {
    let spec = combat::creature(name).unwrap_or_else(|| panic!("{name} is not on the ladder"));
    let log = combat::simulate_holding(
        ch.player_stats(),
        &ch.combat_items(),
        spec,
        D,
        &[],
        0,
        ch.start_with(),
    );
    let theirs: i32 = log
        .entries
        .iter()
        .filter_map(|e| match &e.event {
            Event::Hit { by, damage, .. } if *by == Side::Enemy => Some(*damage),
            _ => None,
        })
        .sum();
    (log.outcome, log.duration_ms, theirs as f32 / (log.duration_ms.max(1) as f32 / 1000.0))
}

/// **It is the longest fight in the game and it is a fight you win.**
///
/// **Bracketed against `common::geared_from`, and `PLAN-M22.md` §M22.7 says
/// `common::from_save(common::THE_RUN)`.** M22.0 measured the run: 974 health
/// and 9 strength over 11 items at level 45, losing to The Unwritten in 7.7
/// seconds and to all ten of the Wextreen deep — so *a win between twenty and
/// twenty-eight seconds* against it is a bracket only something rated below
/// this map's own pool could meet. `every_region_has_a_fight_you_can_win_and_
/// every_boss_can_be_beaten` has required `geared_from` of every boss on a tile
/// since M11.7, and it is what the Tenth Surveyor was actually bracketed
/// against.
///
/// **And that window is two bands too shallow anyway.** Every fight
/// `geared_from` wins at this depth is decided inside the sudden-death ramp —
/// The Unwritten 39.0s, the Ninth Surveyor 44.0s — so the honest bracket is *a
/// victory that is not the buzzer's*, and longer than The Unwritten.
///
/// **Strength is the dial and health is not.** Swept: 13,000 to 15,500 health
/// changed the outcome and the clock **not once**, and strength 96 → 100 turned
/// a win at forty seconds into a loss at thirty-nine. It sits one notch under
/// that cliff, because what is behind it is the end of the writing rather than
/// a counter.
#[test]
fn the_twelfth_name_is_the_longest_fight_in_the_game_and_it_is_won() {
    let ch = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    let (out, ms, dps) = bout(&ch, IT);
    assert_eq!(out, Outcome::Victory, "the best board the game hands out cannot beat it");
    assert_ne!(
        ms, combat::SUDDEN_DEATH_MS,
        "it was killed on the buzzer's own second, which is not a win a board earned"
    );
    assert!(
        ms > combat::SUDDEN_DEATH_MS,
        "it dies at {ms}ms, which is inside the shallow band the plan asked for"
    );
    let (was, was_ms, was_dps) = bout(&ch, "The Unwritten");
    assert_eq!(was, Outcome::Victory);
    assert!(
        ms >= was_ms,
        "The Unwritten takes {was_ms}ms and this takes {ms}ms, so it is not the longest"
    );
    assert!(
        dps > was_dps,
        "it deals {dps:.0}/s against The Unwritten's {was_dps:.0}/s"
    );
    println!("The Twelfth Name: {out:?} at {ms}ms, {dps:.0}/s (The Unwritten: {was_ms}ms, {was_dps:.0}/s)");
}

/// **And the run loses to it, which is the floor rather than the target.**
///
/// The human's own board at level 45 is beaten by everything at this depth and
/// this is no exception. Asserted so that nothing is ever quietly tuned down to
/// something a weak level-45 caster board walks over — the run is the **floor**,
/// and `geared_from` is the ceiling.
#[test]
fn the_run_loses_to_the_twelfth_name() {
    let run = common::from_save(common::THE_RUN);
    let (out, ms, _) = bout(&run, IT);
    assert_eq!(out, Outcome::Defeat, "the run beats it at {ms}ms, so the bracket has slipped");
    // And it loses to the map's own boss too, which is the measurement this
    // bracket rests on rather than an opinion about it.
    assert_eq!(bout(&run, "The Unwritten").0, Outcome::Defeat);
}

/// **It invents no component, so the catalogue and the fingerprint do not
/// move.**
///
/// The Tailgate's rule and The Unwritten's: it wears **two slots of an existing
/// creature's board**, which is the one shape in this set a player survives —
/// *what a creature deals is mostly how many items its board makes*, which this
/// file has now found five times. There is a player mid-run at level 45 and a
/// save that stops opening is the worst thing this block could do to them.
#[test]
fn the_twelfth_name_wears_no_new_component() {
    let it = combat::creature(IT).expect("on the ladder");
    let was = combat::creature("The Unwritten").expect("on the ladder");
    let mine: Vec<&str> = it.gear.iter().map(|(n, ..)| *n).collect();
    let theirs: Vec<&str> = was.gear.iter().map(|(n, ..)| *n).collect();
    assert_eq!(mine, theirs, "it wears a different board from The Unwritten's two slots");
    for (name, ..) in it.gear {
        assert!(
            gm2d_core::piece::CATALOG.iter().any(|d| d.name == *name),
            "{name} is not in the catalogue"
        );
    }
    assert_eq!(gm2d_core::piece::CATALOG.len(), 568, "the catalogue moved");
}

/// **It stands nowhere but the cup.**
///
/// Divergence 15.1's finding, applied before it is a bug: **eight of the nine
/// creatures standing on a boss tile also stand in some region's pool**, so a
/// boss that is also dealt in a field is a boss beaten on behalf of a room
/// nobody has walked into. The cup has one way onto it and it is a pocket, so
/// what stands there has to be found rather than met.
#[test]
fn the_twelfth_name_stands_nowhere_but_the_cup() {
    let mut plates: Vec<String> = Vec::new();
    let mut pools: Vec<String> = Vec::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in &w.places {
            if p.creature.as_deref() == Some(IT) {
                plates.push(format!("{id}/{}", p.id));
            }
        }
        for r in &w.regions {
            if r.enemies.iter().any(|m| m.name == IT) {
                pools.push(format!("{id}/{}", r.id));
            }
        }
    }
    assert_eq!(plates, vec!["the-cup/the-twelfth-name".to_string()], "it stands at {plates:?}");
    assert!(pools.is_empty(), "it is also dealt in {pools:?}");
}

/// **The table's pool can be kennelled, and the thing on the plank cannot.**
///
/// Nothing was built for this: `kennel::OFFER_AT` is five prior wins and
/// `no_boss_is_kennelled` is older than this block. What is asserted is that
/// the content reaches it — a pool nobody can take a companion out of is a
/// system this map is outside of, and nothing else would have said so.
#[test]
fn the_tables_pool_can_be_kennelled_and_the_plank_cannot() {
    let w = data::map("the-lower-table", D);
    let pool: Vec<&str> = w.regions[0].enemies.iter().map(|m| m.name).collect();
    assert_eq!(pool.len(), 4, "the table's pool is {pool:?}");
    let mut g = gm2d_core::game::Game::default();
    for name in &pool {
        for _ in 0..gm2d_core::kennel::OFFER_AT {
            g.world.bump(&gm2d_core::fight::beat_key(name));
        }
        assert!(
            g.kennel_offer(name).is_ok(),
            "{name} is never offered, so the table is outside the kennel: {:?}",
            g.kennel_offer(name).err()
        );
    }
    for _ in 0..gm2d_core::kennel::OFFER_AT + 3 {
        g.world.bump(&gm2d_core::fight::beat_key(IT));
    }
    let why = g.kennel_offer(IT).expect_err("the thing on the plank can be taken home");
    assert!(why.contains("not coming with you"), "{why}");
}

/// **Seventy-eight creatures and seventy-eight figures.**
///
/// One colourway of `unwritten`, which is the one figure in this game whose
/// subject is an absence — a signpost with a planed face and nothing cut into
/// it. This is the same post burnt, with the twelfth cut fresh in it, so the
/// silhouette is shared and the palette is not. **`make art`'s own rule is
/// *draw it, then look at it***, and both were rasterised side by side: planed
/// grey against burnt amber, told apart at a glance.
#[test]
fn every_creature_has_a_figure_at_seventy_eight() {
    let art: serde_json::Value =
        serde_json::from_str(include_str!("../../../data/art.json")).expect("art.json");
    let drawn = art["creatures"].as_object().expect("creatures");
    let mut missing: Vec<&str> = Vec::new();
    let mut n = 0;
    for (id, _) in data::MAPS {
        for r in data::map(id, D).regions {
            for m in r.enemies {
                n += 1;
                if !drawn.contains_key(m.name) {
                    missing.push(m.name);
                }
            }
        }
    }
    assert!(missing.is_empty(), "creatures with no figure: {missing:?}");
    assert!(drawn.contains_key(IT), "the thing on the plank has no figure");
    assert_eq!(drawn.len(), 78, "the game draws {} creatures", drawn.len());
    let _ = n;
}

// ------------------------------------ M22.8, three errands you do with a cue

/// **The far corner is answered by a landing.**
///
/// *A landing on a table is an arrival* is M19's rule and it has been true
/// since `spoke_on_arrival` became one function — so a `Word` errand pointing
/// at a tile on a shot map needs no new machinery at all, which is what
/// `every_rung_is_a_thing_the_table_already_does` is for. What this asserts is
/// that the tile it points at is one a ball can come to rest on, because a word
/// you cannot get to is an errand nobody can finish.
#[test]
fn the_far_corner_is_answered_by_a_landing() {
    use gm2d_core::quest::Goal;
    let quests = data::quests();
    let q = quests.get("the-far-corner").expect("the errand");
    let Goal::Word { place } = &q.goal else { panic!("{:?} is not a word", q.goal) };
    let w = data::map("the-lower-table", D);
    let at = w
        .places
        .iter()
        .find(|p| p.id == *place)
        .unwrap_or_else(|| panic!("{place} is on no tile of the lower table"))
        .at;
    assert_eq!(w.traversal, gm2d_core::world::Traversal::Shot, "it points at a map you walk");
    // **A ball can stop there**, which on a table is the whole of *can you get
    // to it*. The flood in `shot.rs` proves it over every place; this is the
    // one the errand names.
    let a = gm2d_core::world::Allowances::default();
    assert!(
        gm2d_core::shot::aim_at(&w, &Default::default(), w.start, (at[0], at[1]), false, &a)
            .is_some()
            || (0..gm2d_core::shot::STEPS as u16).any(|angle| {
                (1..=10u8).any(|power| {
                    gm2d_core::shot::shoot(
                        &w,
                        w.start,
                        gm2d_core::shot::Shot::new(angle, power),
                        &a,
                    )
                    .rest == (at[0], at[1])
                })
            }),
        "no shot from the tee ever comes to rest on {place}"
    );
}

/// **What the table pays comes back up over the counter.**
///
/// A `Bring` is answered by a component in the bag, so the thing it asks for
/// has to be something the table actually hands over — which is an event's
/// `Give` and nothing else, because nothing down there drops anything.
#[test]
fn the_tables_gear_comes_back_up_over_the_counter() {
    use gm2d_core::quest::Goal;
    let quests = data::quests();
    let q = quests.get("what-the-table-pays").expect("the errand");
    let Goal::Bring { item, count } = &q.goal else { panic!("{:?} is not a bring", q.goal) };
    assert_eq!(*count, 1);
    // Somewhere on the table gives it.
    let w = data::map("the-lower-table", D);
    let events = data::events();
    // **Walked rather than asked**, because `Outcome` is a tree: an `All` holds
    // the others, and a `gives()` on the enum would be a second answer to
    // *what did this pay* beside the one `Game::pay` already gives.
    fn given(o: &gm2d_core::tile_event::Outcome, out: &mut Vec<String>) {
        use gm2d_core::tile_event::Outcome;
        match o {
            Outcome::All(many) => many.iter().for_each(|x| given(x, out)),
            Outcome::Give(name) => out.push(name.clone()),
            _ => {}
        }
    }
    let mut gives: Vec<String> = Vec::new();
    for p in w.places.iter().filter(|p| p.kind == gm2d_core::world::PlaceKind::Event) {
        if let Some(e) = events.get(&p.id) {
            for c in &e.choices {
                given(&c.outcome, &mut gives);
            }
        }
    }
    assert!(
        gives.iter().any(|g| g == item),
        "nothing on the lower table gives {item}; it gives {gives:?}"
    );
    // And it is handed in at the desk, which is one country up.
    assert_eq!(gm2d_core::quest::QuestsData::turn_in_of(q), "the-clerks-desk");
}

/// **The stop-line is behind the twelfth name, and the errand is the only
/// thing in the game that points at it.**
///
/// The Unwritten had no errand for four blocks — *nothing in the game points at
/// it* was M22.0's own recon line — and `cut-the-post` fixed that one. This is
/// the same move one map down, and it is a `Clear` for divergence 15.1's
/// reason: eight of the nine creatures standing on a boss tile also stand in
/// some region's pool, and a boss **tile** is the one thing there is exactly one
/// of.
#[test]
fn the_stop_line_is_behind_the_twelfth_name() {
    use gm2d_core::quest::Goal;
    let quests = data::quests();
    let q = quests.get("the-twelfth-name").expect("the errand");
    let Goal::Clear { place } = &q.goal else { panic!("{:?} is not a clearing", q.goal) };
    let cup = data::map("the-cup", D);
    let plate = cup
        .places
        .iter()
        .find(|p| p.id == *place)
        .unwrap_or_else(|| panic!("{place} is on no tile of the cup"));
    assert_eq!(plate.creature.as_deref(), Some(IT));
    // The door wants the same mark, so the errand and the last screen open
    // together — which is what makes the errand a pointer rather than a second
    // lock.
    let door = cup
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Door)
        .expect("the stop-line");
    assert_eq!(door.needs_all, vec![place.clone()], "the door wants {:?}", door.needs_all);
    // And the chain reaches it: desk, corner, gear, plank.
    assert_eq!(q.requires, vec!["what-the-table-pays".to_string()]);
    let mid = quests.get("what-the-table-pays").expect("the middle rung");
    assert_eq!(mid.requires, vec!["the-far-corner".to_string()]);
    assert_eq!(
        quests.get("the-far-corner").expect("the first rung").requires,
        vec!["cut-the-post".to_string()]
    );
}

/// **Every rung is a thing the table already does.**
///
/// No rung may want a mechanic that is not on the map: a landing is an arrival,
/// an event's `Give` is a component, a boss's tile id is a mark. Nothing here
/// needed building, which is the point of asserting it — a chain that wanted a
/// sixth goal kind would be a chain written for a different game.
#[test]
fn every_rung_is_a_thing_the_table_already_does() {
    use gm2d_core::quest::Goal;
    let quests = data::quests();
    for id in ["the-far-corner", "what-the-table-pays", "the-twelfth-name"] {
        let q = quests.get(id).expect(id);
        assert_eq!(q.giver, "the-clerks-desk", "{id} is given somewhere else");
        // **Not `granted`**, which the plan asks for and which means something
        // else: `Quest::granted` is *handed over by a choice*, and
        // `Outcome::Errand` is the only door into it. Nothing on the table
        // hands these over — they are counter errands behind a `requires`,
        // which is how the three rungs above them work too.
        assert!(!q.granted, "{id} is marked granted and nothing hands it over");
        assert!(q.gold > 0, "{id} pays nothing");
        match &q.goal {
            Goal::Word { .. } | Goal::Bring { .. } | Goal::Clear { .. } => {}
            other => panic!("{id} wants {other:?}, which the table does not do"),
        }
    }
}
