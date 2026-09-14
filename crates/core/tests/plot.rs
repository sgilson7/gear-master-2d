//! The seed drawer, and eight seeds.
//!
//! M21.0. Nothing a player can see yet: a bag, a table of what grows into
//! what, and a drop that fills it.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::plot::{DRAWER_CAP, SEED_PER_MILLE, STAGES};

mod common;

const D: Difficulty = Difficulty::Easy;

/// A game whose board beats a Cave Rat over and over, which a starting kit does
/// not — `build_full_loadout` is the known-good fixture, the same one
/// `drops.rs` fights with and for the same reason.
fn a_fighter(seed: u64) -> Game {
    let mut g = Game::new(seed, "td");
    g.character = common::bench();
    common::build_full_loadout(&mut g.character);
    g
}

/// **Every art family drops a seed**, keyed the way its ingredient is.
///
/// So a creature added to the ladder cannot arrive without one: it already
/// fails a test without an art family, and this is the second half of that —
/// the same argument `every_creature_leaves_something_for_the_larder` makes,
/// over the same eight buckets.
#[test]
fn every_family_has_a_seed() {
    let plot = data::plot();
    let fams = data::art_families();
    let mut bare = Vec::new();
    for m in gm2d_core::combat::LADDER {
        match fams.get(m.name) {
            Some(f) if plot.from_family(f).is_some() => {}
            _ => bare.push(m.name),
        }
    }
    assert!(bare.is_empty(), "creatures that drop no seed: {bare:?}");
    // And the other direction: the eight seeds and the eight pairing
    // ingredients are one set of eight buckets, which is what makes a harvest
    // land somewhere the retort can use.
    let brews = data::brews();
    let pairing: Vec<&str> =
        brews.ingredients.iter().filter(|i| !i.ink_only).map(|i| i.id.as_str()).collect();
    assert_eq!(plot.seeds.len(), pairing.len(), "the seeds and the ingredients are not one set");
    for s in &plot.seeds {
        assert!(pairing.contains(&s.crop.as_str()), "{} harvests {:?}", s.id, s.crop);
    }
}

/// **A crop only ever gets bigger**, and every stage contains the one before.
///
/// A crop that moved cells as it grew could be planted legally and then not
/// fit, which is the stunting this game does not have: `Game::plant` refuses on
/// the *harvest* shape precisely so that a planted crop is a crop that will
/// come up.
#[test]
fn every_stage_shape_grows() {
    for s in &data::plot().seeds {
        assert_eq!(s.stages.len(), STAGES as usize, "{}", s.id);
        for w in s.stages.windows(2) {
            assert!(w[1].len() >= w[0].len(), "{} shrinks", s.id);
            for c in &w[0] {
                assert!(w[1].contains(c), "{} drops the cell {c:?}", s.id);
            }
        }
        // The first is one cell, which is what *you plant a seed* means, and
        // the last is a polyomino worth finding room for.
        assert_eq!(s.stages[0].len(), 1, "{} does not start as a seed", s.id);
        let n = s.harvest_shape().len();
        assert!((3..=5).contains(&n), "{} harvests at {n} cells", s.id);
    }
    // And no two families come up the same shape, or the bed is eight of one
    // puzzle rather than eight puzzles.
    let mut shapes: Vec<Vec<(i8, i8)>> =
        data::plot().seeds.iter().map(|s| s.harvest_shape().to_vec()).collect();
    for v in &mut shapes {
        v.sort_unstable();
    }
    let before = shapes.len();
    shapes.sort();
    shapes.dedup();
    assert_eq!(shapes.len(), before, "two seeds come up the same shape");
}

/// A win drops a seed at its rate, and it is the same seed the ingredient is.
///
/// **Measured over many wins rather than asserted on one**, because the roll is
/// a roll: one fight tells you nothing about a per-mille, and a check that
/// fought once and asserted *a seed* would be a check that fails one run in
/// three for the right reason.
#[test]
fn a_win_drops_a_seed_at_its_rate() {
    let mut held = 0u32;
    let mut wins = 0u32;
    let mut per_run: Vec<u32> = Vec::new();
    for seed in 0..12u64 {
        let mut g = a_fighter(0x5EED_0000_5EED_0000 + seed);
        for _ in 0..30 {
            g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
            let log = fight::run(&g, D).expect("a fight");
            fight::settle(&mut g, &log, D).expect("it settles");
            g.character.fatigue = 0;
            wins += 1;
        }
        let n: u32 = g.character.seed_drawer.values().sum();
        per_run.push(n);
        held += n;
    }
    // The rate, generously bracketed: this is a measurement, not a promise
    // about any one run.
    let want = wins * SEED_PER_MILLE / 1_000;
    let (low, high) = (want * 5 / 10, want * 16 / 10);
    assert!(
        (low..=high).contains(&held),
        "{held} seeds over {wins} wins; {SEED_PER_MILLE} per mille wants about {want} \
         (per run {per_run:?})"
    );
    // And it is the rat's own seed, not somebody else's.
    let mut g = a_fighter(0x5EED_0000_5EED_00FF);
    for _ in 0..60 {
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).expect("a fight");
        fight::settle(&mut g, &log, D).expect("it settles");
        g.character.fatigue = 0;
    }
    let want = data::plot()
        .from_family(data::art_families().get("Cave Rat").expect("the rat has a family"))
        .expect("the rat drops a seed")
        .id
        .clone();
    for k in g.character.seed_drawer.keys() {
        assert_eq!(*k, want, "a Cave Rat left somebody else's seed");
    }
}

/// The drawer stops filling at the cap, and the roll still happens.
///
/// **Rolled whether or not there is room**, and refused after: skipping the
/// draw would make the stream a function of what the player is carrying rather
/// than of the fights they had, which is the rule `drops::roll_with` has
/// followed since M9.1 and the reason a seeded walk replays at all.
#[test]
fn the_drawer_stops_at_the_cap() {
    let mut g = a_fighter(0x5EED_0000_5EED_0042);
    let seed = data::plot()
        .from_family(data::art_families().get("Cave Rat").expect("a family"))
        .expect("a seed")
        .id
        .clone();
    for _ in 0..DRAWER_CAP {
        g.character.pocket_seed(&seed);
    }
    for _ in 0..120 {
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).expect("a fight");
        fight::settle(&mut g, &log, D).expect("it settles");
        g.character.fatigue = 0;
    }
    assert_eq!(g.character.seeds_held(&seed), DRAWER_CAP, "the drawer went past its cap");
}

/// What goes in comes out, and a refusal spends nothing.
#[test]
fn the_drawer_gives_one_back() {
    let mut g = a_fighter(0x5EED_0000_5EED_0009);
    assert!(g.character.spend_seed("cairn-dust-seed").is_err(), "an empty drawer handed one over");
    g.character.pocket_seed("cairn-dust-seed");
    g.character.pocket_seed("cairn-dust-seed");
    assert_eq!(g.character.seeds_held("cairn-dust-seed"), 2);
    g.character.spend_seed("cairn-dust-seed").expect("one comes out");
    assert_eq!(g.character.seeds_held("cairn-dust-seed"), 1);
    g.character.spend_seed("cairn-dust-seed").expect("and the last one");
    assert_eq!(g.character.seeds_held("cairn-dust-seed"), 0);
    assert!(!g.character.seed_drawer.contains_key("cairn-dust-seed"), "an empty entry was kept");
}

/// **A drawer survives being written down, and a save without one opens.**
#[test]
fn a_save_without_a_drawer_opens() {
    let mut g = a_fighter(0x5EED_0000_5EED_0007);
    g.character.pocket_seed("cairn-dust-seed");
    g.character.pocket_seed("cairn-dust-seed");
    let text = gm2d_core::save::save(&g);
    let back = gm2d_core::save::load(&text).expect("a save this build wrote loads");
    assert_eq!(back.character.seeds_held("cairn-dust-seed"), 2, "the drawer did not survive");

    // And a file written before the Plot existed — which is every file there
    // is — opens with an empty one.
    let mut v: serde_json::Value = serde_json::from_str(&text).expect("it parses");
    let state = v.get_mut("state").map(|s| s.clone()).unwrap_or(v.clone());
    let mut state = state;
    state["character"].as_object_mut().expect("a character").remove("seed_drawer");
    if v.get("state").is_some() {
        v["state"] = state;
    } else {
        v = state;
    }
    let back = gm2d_core::save::load(&serde_json::to_string(&v).unwrap())
        .expect("a save from before the drawer still opens");
    assert!(back.character.seed_drawer.is_empty(), "it came back with something in it");
}

/// **A seed is not a component**, which is the whole reason the fingerprint did
/// not move and a player mid-run keeps their run.
#[test]
fn a_seed_is_not_a_component() {
    for s in &data::plot().seeds {
        assert!(
            !gm2d_core::piece::CATALOG.iter().any(|d| d.name == s.name),
            "{} is in the catalogue, which moves the save fingerprint",
            s.id
        );
    }
}

// ------------------------------------------------------------------- the bed

const TOWNS: [&str; 3] = ["the-end-of-all-gears", "kettleworks", "the-third-town"];

/// **Every seed can be planted in every bed**, which is the measurement that
/// put a `turn` on a crop.
///
/// Of the twenty-four pairs, **three fit nowhere at all** without one:
/// `toad-ichor` and `tallow-drip` in Kettleworks' gapped row and `reef-salt` in
/// the pit's round bed. A seed that is dead in a town for a reason the player
/// cannot act on is worse than a bed that is too easy, and the retort has
/// turned its ingredients since M20.
#[test]
fn every_seed_fits_every_bed() {
    let g = Game::new(0x5EED_0000_B3D0_0001, "td");
    let plot = data::plot();
    for town in TOWNS {
        let mask = g.bed_mask(town, D);
        assert!(!mask.is_empty(), "{town} has no bed");
        assert!((11..=14).contains(&mask.len()), "{town}'s bed is {} cells", mask.len());
        for s in &plot.seeds {
            let ok = (0..4).any(|turn| {
                (-2..8).any(|ax| {
                    (-2..8).any(|ay| {
                        let want = gm2d_core::plot::harvest_cells(plot, &s.id, (ax, ay), turn);
                        !want.is_empty() && want.iter().all(|c| mask.contains(c))
                    })
                })
            });
            assert!(ok, "{} fits nowhere in {town}'s bed", s.id);
        }
    }
}

/// A harvest shape that runs off the bed is refused, **by cell**.
#[test]
fn a_harvest_shape_that_does_not_fit_is_refused_by_cell() {
    let mut g = Game::new(0x5EED_0000_B3D0_0002, "td");
    let seed = "reef-salt-seed";
    g.character.pocket_seed(seed);
    // Far off the corner of the bed, so every cell it wants is missing.
    let why = g.plant("kettleworks", seed, (9, 9), 0, D).unwrap_err();
    assert!(why.contains('('), "a refusal that names no cell: {why}");
    assert!(why.contains("stops short"), "{why}");
    // **And it spent nothing.**
    assert_eq!(g.character.seeds_held(seed), 1, "a refusal took the seed");
    assert!(g.character.beds.is_empty(), "a refusal planted it anyway");
}

/// Two crops cannot stand on one cell, and the refusal says which.
#[test]
fn two_crops_do_not_share_a_cell() {
    let mut g = Game::new(0x5EED_0000_B3D0_0003, "td");
    g.character.pocket_seed("cairn-dust-seed");
    g.character.pocket_seed("cairn-dust-seed");
    g.plant("the-third-town", "cairn-dust-seed", (0, 1), 0, D).expect("the first one goes in");
    let why = g.plant("the-third-town", "cairn-dust-seed", (0, 1), 0, D).unwrap_err();
    assert!(why.contains("already coming up"), "{why}");
    assert_eq!(g.character.seeds_held("cairn-dust-seed"), 1, "a refusal took the seed");
}

/// **A crop grows one stage per win, anywhere** — and two beds grow at once.
#[test]
fn a_crop_grows_one_stage_per_win_anywhere() {
    let mut g = a_fighter(0x5EED_0000_B3D0_0004);
    g.character.pocket_seed("cairn-dust-seed");
    g.character.pocket_seed("cairn-dust-seed");
    g.plant("kettleworks", "cairn-dust-seed", (0, 0), 0, D).expect("one at the works");
    g.plant("the-third-town", "cairn-dust-seed", (0, 1), 0, D).expect("one down there");
    let stage = |g: &Game, t: &str| g.character.beds[t][0].stage;
    assert_eq!((stage(&g, "kettleworks"), stage(&g, "the-third-town")), (0, 0));

    // One win, on neither map.
    g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let log = fight::run(&g, D).expect("a fight");
    fight::settle(&mut g, &log, D).expect("it settles");
    assert_eq!(
        (stage(&g, "kettleworks"), stage(&g, "the-third-town")),
        (1, 1),
        "the bell did not reach both beds"
    );

    // And it stops when it is ready rather than running away.
    for _ in 0..6 {
        g.character.fatigue = 0;
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).expect("a fight");
        fight::settle(&mut g, &log, D).expect("it settles");
    }
    assert_eq!(stage(&g, "kettleworks"), STAGES - 1, "a crop grew past ready");
    assert!(g.character.beds["kettleworks"][0].ready());
}

/// A harvest fills the larder, and an unripe one is refused with the count.
#[test]
fn a_harvest_fills_the_larder() {
    let mut g = a_fighter(0x5EED_0000_B3D0_0005);
    g.character.pocket_seed("bone-meal-seed");
    g.plant("the-third-town", "bone-meal-seed", (0, 0), 0, D).expect("it goes in");

    let why = g.harvest("the-third-town", (0, 0)).unwrap_err();
    assert!(why.contains("not up yet") && why.contains("wins"), "{why}");

    for _ in 0..(STAGES - 1) {
        g.character.fatigue = 0;
        g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
        let log = fight::run(&g, D).expect("a fight");
        fight::settle(&mut g, &log, D).expect("it settles");
    }
    let before = g.character.in_larder("bone-meal");
    let (name, n) = g.harvest("the-third-town", (0, 0)).expect("it comes up");
    assert_eq!(n, gm2d_core::plot::HARVEST_YIELD);
    assert_eq!(g.character.in_larder("bone-meal"), before + n, "{name} did not reach the larder");
    assert!(g.character.beds.get("the-third-town").is_none(), "the bed kept a pulled crop");
}
