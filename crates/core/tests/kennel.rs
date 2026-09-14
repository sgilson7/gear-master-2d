//! The kennel, the offer, the feed and the yard.
//!
//! M21.4. Everything here is a bag, a clock and a shape — the fight is M21.5's.

use gm2d_core::combat::{self, Difficulty, Rank};
use gm2d_core::data;
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::kennel::{OFFER_AT, TALLY_PCT, TALLY_STEP};

mod common;

const D: Difficulty = Difficulty::Easy;

fn a_fighter(seed: u64) -> Game {
    let mut g = Game::new(seed, "td");
    g.character = common::bench();
    common::build_full_loadout(&mut g.character);
    g
}

fn win(g: &mut Game, who: &str) {
    g.character.fatigue = 0;
    g.encounter = Some(fight::Encounter { enemy: who.into(), at: [1, 18] });
    let log = fight::run(g, D).expect("a fight");
    fight::settle(g, &log, D).expect("it settles");
}

/// **A boss is never kennelled**, which is the balance risk in the whole idea
/// stated as a rule — and it is in the suite before the offer is written.
#[test]
fn no_boss_is_kennelled() {
    let mut g = a_fighter(0x5EED_0000_6E00_0001);
    let bosses: Vec<&str> =
        combat::LADDER.iter().filter(|m| m.rank == Rank::Boss).map(|m| m.name).collect();
    assert!(!bosses.is_empty(), "no boss in the ladder, so this checks nothing");
    for b in &bosses {
        // However well you know it.
        for _ in 0..(OFFER_AT + 3) {
            g.world.bump(&fight::beat_key(b));
        }
        let why = g.kennel_offer(b).unwrap_err();
        assert!(why.contains("not coming with you"), "{b}: {why}");
        assert!(g.take_along(b).is_err(), "{b} came along anyway");
    }
}

/// **The sixth win offers, and once per family.**
#[test]
fn the_sixth_win_offers_once_per_family() {
    let mut g = a_fighter(0x5EED_0000_6E00_0002);
    let rat = "Cave Rat";
    // Five wins is not enough, and the refusal counts down.
    for n in 0..OFFER_AT {
        let why = g.kennel_offer(rat).unwrap_err();
        assert!(why.contains("does not know you"), "{why}");
        assert!(why.contains(&(OFFER_AT - n).to_string()), "the refusal does not count: {why}");
        win(&mut g, rat);
    }
    g.kennel_offer(rat).expect("the sixth win offers");
    g.take_along(rat).expect("and it comes");
    assert_eq!(g.character.kennel.len(), 1);

    // Not twice.
    assert!(g.take_along(rat).unwrap_err().contains("already"));

    // And not something else of the same family, however well you know it.
    let family = data::art_families().get(rat).cloned().expect("the rat has a family");
    let kin = combat::LADDER
        .iter()
        .find(|m| {
            m.name != rat
                && m.rank != Rank::Boss
                && data::art_families().get(m.name) == Some(&family)
        })
        .map(|m| m.name);
    if let Some(kin) = kin {
        for _ in 0..(OFFER_AT + 1) {
            g.world.bump(&fight::beat_key(kin));
        }
        let why = g.kennel_offer(kin).unwrap_err();
        assert!(why.contains("enough like it"), "{kin}: {why}");
    }
}

/// **A creature out eats at the bell, and an empty larder keeps it in.**
#[test]
fn a_creature_out_eats_at_the_bell() {
    let mut g = a_fighter(0x5EED_0000_6E00_0003);
    let rat = "Cave Rat";
    for _ in 0..OFFER_AT {
        win(&mut g, rat);
    }
    g.take_along(rat).expect("it comes");
    let eats = g.character.kennel[0].eats.clone();
    // Somewhere to stand.
    let mask = g.run_mask("the-end-of-all-gears", D);
    let at = *mask.first().expect("the pit has a run");
    g.put_out("the-end-of-all-gears", rat, at, 0, D).expect("it goes out");

    // Fed.
    let before = g.character.in_larder(&eats);
    assert!(before > 0, "the fighting should have filled the larder");
    win(&mut g, rat);
    assert_eq!(g.character.in_larder(&eats), before + 1 - 1, "it did not eat exactly one");
    assert!(g.character.kennel[0].out, "a fed creature was put away");

    // **And an empty larder keeps it in — but only if you fight something
    // else.** A win pays a family-matched ingredient *before* the feed runs,
    // so a creature fighting its own kin feeds itself out of the corpse and
    // the larder never empties. Notebook row 13. So this fights out of the
    // rat's bucket.
    let brews = data::brews();
    let other = combat::LADDER
        .iter()
        .find(|m| {
            m.rank != Rank::Boss
                && data::art_families()
                    .get(m.name)
                    .and_then(|f| brews.from_family(f))
                    .is_some_and(|i| i.id != eats)
                && ["Bone Archer", "Rust Golem", "Frost Wisp"].contains(&m.name)
        })
        .map(|m| m.name)
        .expect("something eats out of another bucket");
    g.character.larder.remove(&eats);
    g.character.fatigue = 0;
    g.encounter = Some(fight::Encounter { enemy: other.into(), at: [1, 18] });
    let log = fight::run(&g, D).expect("a fight");
    let s = fight::settle(&mut g, &log, D).expect("it settles");
    assert!(
        !g.character.kennel[0].out,
        "an empty larder did not put it away: {:?}",
        s.receipt
    );
    assert!(
        s.receipt.iter().any(|l| l.contains("stayed in")),
        "it went in and nothing said so: {:?}",
        s.receipt
    );
}

/// A silhouette that will not fit the run is refused **by cell**.
#[test]
fn a_silhouette_that_does_not_fit_the_run_is_refused_by_cell() {
    let mut g = a_fighter(0x5EED_0000_6E00_0004);
    let rat = "Cave Rat";
    for _ in 0..OFFER_AT {
        win(&mut g, rat);
    }
    g.take_along(rat).expect("it comes");
    let why = g.put_out("the-end-of-all-gears", rat, (9, 9), 0, D).unwrap_err();
    assert!(why.contains('(') && why.contains("stops short"), "{why}");
    assert!(!g.character.kennel[0].out, "a refusal put it out anyway");
}

/// **Every family has a silhouette and it fits every run**, which is the
/// measurement the plan asked for before anything was authored.
#[test]
fn every_family_stands_somewhere() {
    let g = Game::new(0x5EED_0000_6E00_0005, "td");
    let data = data::kennel();
    let fams = data::art_families();
    for m in combat::LADDER {
        let Some(f) = fams.get(m.name) else { continue };
        assert!(data.get(f).is_some(), "{}'s family {f} has no silhouette", m.name);
    }
    for town in ["the-end-of-all-gears", "kettleworks", "the-third-town"] {
        let mask = g.run_mask(town, D);
        assert!(!mask.is_empty(), "{town} has no run");
        for s in &data.silhouettes {
            let fits = (0..4).any(|turn| {
                (-1..8).any(|ax| {
                    (-1..8).any(|ay| {
                        let k = gm2d_core::kennel::Kennelled {
                            spec: String::new(),
                            family: s.family.clone(),
                            eats: String::new(),
                            wins_together: 0,
                            out: false,
                            at: (ax, ay),
                            turn,
                        };
                        let cs = gm2d_core::kennel::cells_of(data, &k);
                        !cs.is_empty() && cs.iter().all(|c| mask.contains(c))
                    })
                })
            });
            assert!(fits, "{} stands nowhere in {town}'s run", s.family);
        }
    }
}

/// **Every pair of diets is authored**, which is `C(8,2)` over the eight
/// buckets rather than `C(20,2)` over the families.
#[test]
fn every_pair_of_diets_is_authored() {
    let data = data::kennel();
    let brews = data::brews();
    let diets: Vec<&str> =
        brews.ingredients.iter().filter(|i| !i.ink_only).map(|i| i.id.as_str()).collect();
    let want = diets.len() * (diets.len() - 1) / 2;
    assert_eq!(data.pairs.len(), want, "{} diets want {want} pairs", diets.len());
    for i in 0..diets.len() {
        for j in (i + 1)..diets.len() {
            let (a, b) = (diets[i], diets[j]);
            let got = data.pair(a, b).unwrap_or_else(|| panic!("{a} and {b} pay nothing"));
            assert_eq!(data.pair(b, a).map(|p| &p.blurb), Some(&got.blurb), "{a}/{b} is ordered");
        }
    }
}

/// Ten wins together pay five percent, in steps a player can see.
#[test]
fn ten_wins_together_pay_five() {
    let mut k = gm2d_core::kennel::Kennelled {
        spec: "Cave Rat".into(),
        family: "a-rat".into(),
        eats: "toad-ichor".into(),
        wins_together: 0,
        out: true,
        at: (0, 0),
        turn: 0,
    };
    assert_eq!(k.tally_pct(), 0);
    k.wins_together = TALLY_STEP - 1;
    assert_eq!(k.tally_pct(), 0, "it paid before the step");
    k.wins_together = TALLY_STEP;
    assert_eq!(k.tally_pct(), TALLY_PCT);
    k.wins_together = TALLY_STEP * 3;
    assert_eq!(k.tally_pct(), TALLY_PCT * 3);
}

// ------------------------------------------------------- and into the fight

/// **A creature out fights, and is capped at the region rather than its own
/// strength.**
///
/// That cap is the answer to the balance risk in the whole idea: something
/// kennelled in the deep and walked back to the pit would otherwise be a boss
/// on your side. `Region::danger` is the mean of `creature_rating` over the
/// pool, measured at load — *danger is measured, never typed*.
#[test]
fn a_creature_out_fights_and_is_capped_at_the_region() {
    let mut g = a_fighter(0x5EED_0000_6E00_0010);
    let rat = "Cave Rat";
    for _ in 0..OFFER_AT {
        win(&mut g, rat);
    }
    g.take_along(rat).expect("it comes");
    let mask = g.run_mask("the-end-of-all-gears", D);
    let at = *mask.first().expect("a run");

    let alone = g.character.combat_items().len();
    g.put_out("the-end-of-all-gears", rat, at, 0, D).expect("it goes out");
    let danger = g.here_danger(D);
    let with = g.character.combat_items().len()
        + g.character.companion_items(danger, D).len();
    assert!(with > alone, "a creature out contributed nothing");

    // **Capped.** Something far over the bracket gives the bracket's worth,
    // not its own — asked of `share` directly, which is where the sum is done.
    let modest = gm2d_core::kennel::share(100, 1_000, 0);
    let huge = gm2d_core::kennel::share(5_000, 1_000, 0);
    assert_eq!(modest, 100, "something under the bracket gave {modest}%");
    assert!(huge < 100, "something five times the bracket gave {huge}%");
    assert!(huge > 0, "and it gave nothing at all");
}

/// **Ten wins together pay five percent**, and it is the only thing that takes
/// a companion past its own strength.
#[test]
fn the_tally_is_the_only_thing_past_its_own_strength() {
    let plain = gm2d_core::kennel::share(500, 500, 0);
    let tallied = gm2d_core::kennel::share(500, 500, TALLY_PCT);
    assert_eq!(plain, 100);
    assert_eq!(tallied, 100 + TALLY_PCT);
}

/// **A lost fight puts it back and it forgets.**
#[test]
fn a_lost_fight_resets_it() {
    let mut g = a_fighter(0x5EED_0000_6E00_0011);
    let rat = "Cave Rat";
    for _ in 0..OFFER_AT {
        win(&mut g, rat);
    }
    g.take_along(rat).expect("it comes");
    let mask = g.run_mask("the-end-of-all-gears", D);
    g.put_out("the-end-of-all-gears", rat, mask[0], 0, D).expect("out");
    g.character.kennel[0].wins_together = 25;

    // A board that loses: strip it and meet something that does not lose.
    g.character = common::bench();
    g.character.kennel = vec![gm2d_core::kennel::Kennelled {
        spec: rat.into(),
        family: "a-rat".into(),
        eats: "toad-ichor".into(),
        wins_together: 25,
        out: true,
        at: (0, 0),
        turn: 0,
    }];
    g.encounter = Some(fight::Encounter { enemy: "Nine of Ashes".into(), at: [1, 18] });
    let log = fight::run(&g, D).expect("a fight");
    assert_ne!(log.outcome, combat::Outcome::Victory, "the stripped board won");
    let s = fight::settle(&mut g, &log, D).expect("it settles");
    assert!(!g.character.kennel[0].out, "a lost fight left it out");
    assert_eq!(g.character.kennel[0].wins_together, 0, "it remembered");
    assert!(
        s.receipt.iter().any(|l| l.contains("went back in the kennel")),
        "nothing said so: {:?}",
        s.receipt
    );
}

/// **A win together counts**, and only for what is out.
#[test]
fn a_win_together_counts_for_what_is_out() {
    let mut g = a_fighter(0x5EED_0000_6E00_0012);
    let rat = "Cave Rat";
    for _ in 0..OFFER_AT {
        win(&mut g, rat);
    }
    g.take_along(rat).expect("it comes");
    // In: the tally does not move.
    win(&mut g, rat);
    assert_eq!(g.character.kennel[0].wins_together, 0, "something in got credit");
    // Out: it does.
    let mask = g.run_mask("the-end-of-all-gears", D);
    g.put_out("the-end-of-all-gears", rat, mask[0], 0, D).expect("out");
    win(&mut g, rat);
    assert_eq!(g.character.kennel[0].wins_together, 1);
}
