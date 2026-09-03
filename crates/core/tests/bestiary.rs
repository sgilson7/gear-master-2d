//! M11.9's six sets, and the one that takes you home.
//!
//! The block's second and last save seam. Eighteen components, one grid's whole
//! recipe each, and the M9 conventions unchanged: three drops make one finished
//! item, the rule rides one component and pays off the whole set, and nothing is
//! on a shelf.
//!
//! **Five of the six rules are ones the engine already had**, tuned to a new
//! instance, which is the block's standing rule — nothing new is invented in
//! combat for a set. The sixth is `Rule::Homeward`, which is not a combat rule
//! at all.

use gm2d_core::character::Character;
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::loadout::set_pieces;
use gm2d_core::piece::{SlotKind, CATALOG, SETS};
use gm2d_core::rule::Rule;
use gm2d_core::world::WorldState;

mod common;

const D: Difficulty = Difficulty::Easy;

/// The six M11.9 adds, by their constants' order in `SETS`.
const NEW: &[&str] = &[
    "The Rime Coat",
    "The Sentinel's Tread",
    "The Idol's Ward",
    "The Chorus Robe",
    "The Curd Mantle",
    "The Drover's Stride",
];

/// **Nine sets, and every one of them grants something different.**
///
/// Distinct *instances* rather than distinct variants: `CurseOnActivate` on
/// gloves with Frost is not the Wallspider Weave's, and `Rout` on The Curator is
/// not the Rat King's. A block that invented six new combat mechanics for six
/// sets would be a block that invented six new combat mechanics.
#[test]
fn every_set_grants_something_and_no_two_grant_the_same_thing() {
    assert_eq!(SETS.len(), 9, "three from M9 and six from M11.9");
    let mut granted: Vec<(&str, Rule)> = Vec::new();
    for &set in SETS {
        let mut mine: Vec<Rule> = Vec::new();
        for name in set_pieces(set) {
            let def = CATALOG.iter().find(|d| d.name == name).expect("in the catalogue");
            mine.extend(def.assembly_bonus.into_iter().flat_map(|b| b.grants).cloned());
        }
        assert_eq!(
            mine.len(),
            1,
            "{set} grants {} rules; a set is one rule and it rides one component",
            mine.len()
        );
        let r = mine.remove(0);
        r.check().unwrap_or_else(|e| panic!("{set} grants a rule the engine has not got: {e}"));
        if let Some((other, _)) = granted.iter().find(|(_, g)| *g == r) {
            panic!("{set} and {other} both grant {r:?}");
        }
        granted.push((set, r));
    }
}

/// Every new set is on no shelf, off one thing, and says what it does.
#[test]
fn every_new_set_is_earned_and_explains_itself() {
    let shops = data::shops();
    for &set in NEW {
        let pieces = set_pieces(set);
        assert_eq!(pieces.len(), 3, "{set} is {} pieces", pieces.len());
        for name in &pieces {
            assert!(
                gm2d_core::piece::is_event_only(name),
                "{name} is not EVENT_ONLY, so the ladder could deal it"
            );
            for t in &shops.towns {
                assert!(!t.stock.iter().any(|s| s == name), "{} sells {name}", t.id);
            }
            // The card names the set and what it does — M9.4's finding, and it
            // is derived, so it cannot go stale.
            let def = CATALOG.iter().find(|d| d.name == *name).expect("in the catalogue");
            let lines = gm2d_core::explain::piece_lines(def);
            let said: Vec<&String> =
                lines.iter().filter(|(k, _)| *k == "set").map(|(_, v)| v).collect();
            assert!(!said.is_empty(), "{name} says nothing about being part of a set");
            assert!(
                said.iter().any(|l| l.contains(set)),
                "{name}'s card does not name {set}: {said:?}"
            );
        }
    }
}

/// **Every farmable set is off the creature its region draws most.**
///
/// `PLAN.md` §6b row 1, obeyed rather than paid off. `draw_enemy` makes a pool's
/// hardest member its rarest, so a set behind the hardest thing in a region is a
/// set nobody finishes — this block has found that three times now, and this is
/// the check that stops a fourth.
///
/// The Curd Mantle is exempt and is the reason the check names the others: its
/// pieces are certainties off three tower floors, so climbing *is* the grind.
#[test]
fn a_set_is_never_behind_the_rarest_fight_in_its_region() {
    let drops = data::drops();
    for &set in NEW {
        let Some(first) = set_pieces(set).first().cloned() else { continue };
        let Some(owner) = drops.drops.iter().find(|d| d.piece == first) else {
            continue; // the tower's, which nothing rolls for
        };
        let who = owner.creature.as_str();
        let mut best = 0;
        for (id, _) in data::MAPS {
            for r in data::map(id, D).regions {
                if !r.enemies.iter().any(|m| m.name == who) {
                    continue;
                }
                let rated: Vec<i32> = r
                    .enemies
                    .iter()
                    .map(|m| gm2d_core::rating::creature_rating(m, D))
                    .collect();
                let max = rated.iter().copied().max().unwrap_or(0);
                let weights: Vec<i32> = rated.iter().map(|v| (max + 1 - v).max(1)).collect();
                let total: i32 = weights.iter().sum();
                let mine = r
                    .enemies
                    .iter()
                    .zip(&weights)
                    .find(|(m, _)| m.name == who)
                    .map(|(_, w)| *w)
                    .unwrap_or(0);
                best = best.max(mine * 100 / total.max(1));
            }
        }
        assert!(
            best >= 20,
            "{set} comes off {who}, which is drawn {best}% of the time at best"
        );
    }
}

/// **The tower's set is assembled by climbing it**, one piece at three floors.
#[test]
fn the_curd_mantle_is_the_tower_paying_three_times() {
    let want = set_pieces("The Curd Mantle");
    assert_eq!(want.len(), 3);
    let mut floors: Vec<&str> = Vec::new();
    for n in [5, 4, 3, 2, 1] {
        let id: &'static str = match n {
            5 => "the-drambus-stack-5",
            4 => "the-drambus-stack-4",
            3 => "the-drambus-stack-3",
            2 => "the-drambus-stack-2",
            _ => "the-drambus-stack-1",
        };
        let w = data::map(id, D);
        if w.places.iter().any(|p| p.drops.iter().any(|d| want.contains(&d.as_str()))) {
            floors.push(id);
        }
    }
    assert_eq!(
        floors,
        ["the-drambus-stack-5", "the-drambus-stack-3", "the-drambus-stack-1"],
        "the Mantle should pay at the top, the middle and the bottom"
    );
    // And nothing rolls for it: a floor is one sitting, so a set off a floor's
    // pool would be unfarmable and a set off its boss is a certainty.
    for p in want {
        assert!(
            !data::drops().drops.iter().any(|d| d.piece == p),
            "{p} is in a drop table as well as on a tile"
        );
    }
}

// ------------------------------------------------------------- the way home

fn wearing_the_stride() -> Game {
    let mut g = Game::new(0x11_09, "td");
    g.character = common::bench();
    for k in SlotKind::ALL {
        g.character.loadout.slot_mut(k).clear();
    }
    g.character.grow_boards(20);
    // The three, touching, in the greaves grid: a material, a mold and a
    // plating is that recipe entire.
    for (name, x, y) in
        [("Drover's Material", 0u8, 0u8), ("Drover's Mold", 2, 0), ("Drover's Sole", 2, 2)]
    {
        let id = common::piece(&g.character, name);
        g.character
            .equip(id, SlotKind::Greaves, x, y)
            .unwrap_or_else(|e| panic!("{name} at ({x}, {y}): {e}"));
    }
    g.world = WorldState::at_start(&data::world(D));
    g.world.last_town = "the-end-of-all-gears".into();
    g
}

/// **Assembled and whole, it takes you home, and it drinks a tin doing it.**
#[test]
fn the_stride_takes_you_home_for_one_restorative() {
    let mut g = wearing_the_stride();
    assert!(
        g.character.rules().contains(&Rule::Homeward),
        "the whole set grants {:?}",
        g.character.rules()
    );
    // Somewhere else, tired, with one tin.
    g.world.map = "the-treyway".into();
    g.world.at = [7, 8];
    g.character.tire(30);
    g.character.give_supply("cork-tea", 1);

    let home = g.go_home(D).expect("the gear knows the way");
    assert_eq!(home.town, "the-end-of-all-gears");
    assert_eq!(g.world.map, gm2d_core::world::overworld(), "it left you on the wrong map");
    assert_eq!(g.character.supply_count("cork-tea"), 0, "the fare was not paid");
    assert!(home.mended > 0, "arriving in a town did not take the tiredness off");
    assert_eq!(g.character.fatigue, 0);
    assert!(!home.fare.is_empty(), "the receipt does not name what it drank");
    // And where you were is remembered, the same as any other crossing.
    assert_eq!(g.world.recall("the-treyway"), Some([7, 8]));
}

/// **Four refusals, and each names the thing that is in the way.**
#[test]
fn the_way_home_says_why_it_will_not() {
    // Not wearing it.
    let mut bare = Game::new(2, "td");
    bare.world = WorldState::at_start(&data::world(D));
    bare.world.last_town = "the-end-of-all-gears".into();
    bare.character.give_supply("cork-tea", 1);
    let why = bare.go_home(D).expect_err("a bare character walked home");
    assert!(why.contains("wearing"), "{why}");

    // Nowhere to go back to.
    let mut lost = wearing_the_stride();
    lost.world.last_town = String::new();
    lost.character.give_supply("cork-tea", 1);
    let why = lost.go_home(D).expect_err("home with no home");
    assert!(why.contains("town"), "{why}");

    // No fare.
    let mut skint = wearing_the_stride();
    assert_eq!(skint.character.supply_count("cork-tea"), 0);
    let why = skint.go_home(D).expect_err("home for free");
    assert!(why.to_lowercase().contains("restorative"), "{why}");

    // And not from under the lake.
    let mut deep = wearing_the_stride();
    deep.character.give_supply("cork-tea", 1);
    deep.world.map = "under-the-lake".into();
    deep.world.at = [1, 1];
    let why = deep.go_home(D).expect_err("posted itself out from under a lake");
    assert!(why.contains("steps"), "{why}");
    assert_eq!(deep.character.supply_count("cork-tea"), 1, "a refusal spent the fare");
}

/// **From inside the tower, though**, which `PLAN-M11.md` §8 row 9 says yes to:
/// it is five entries by design and the kick already moves you.
#[test]
fn the_way_home_works_from_inside_the_stack() {
    let mut g = wearing_the_stride();
    g.character.give_supply("cork-tea", 1);
    g.world.map = "the-drambus-stack-3".into();
    g.world.at = [1, 8];
    let home = g.go_home(D).expect("the tower is not under a lake");
    assert_eq!(home.town, "the-end-of-all-gears");
}

/// **The cheapest tin, and only one.**
///
/// A player spending a fare pays it out of small change, and choosing which tin
/// to burn is a decision nobody wants to make twice a session.
#[test]
fn it_pays_the_fare_out_of_small_change() {
    let mut g = wearing_the_stride();
    g.character.give_supply("cork-tea", 2);
    g.character.give_supply("long-shift-tin", 1);
    g.world.map = "the-treyway".into();
    g.world.at = [7, 8];
    g.go_home(D).expect("home");
    assert_eq!(g.character.supply_count("cork-tea"), 1, "it drank the wrong number");
    assert_eq!(g.character.supply_count("long-shift-tin"), 1, "it drank the dear one");
}

/// Taking the set apart takes the way home with it.
#[test]
fn an_unassembled_stride_knows_no_way_home() {
    let mut g = wearing_the_stride();
    g.character.give_supply("cork-tea", 1);
    let sole = common::piece(&g.character, "Drover's Sole");
    g.character.unequip(sole).expect("off the board");
    assert!(!g.character.rules().contains(&Rule::Homeward));
    assert!(g.go_home(D).is_err(), "two thirds of a set walked home");
}

/// The starting kit is not secretly wearing any of this.
#[test]
fn a_new_character_grants_none_of_it() {
    let mut ch = Character::starting();
    ch.apply_preset();
    assert!(ch.rules().is_empty(), "a starting character grants {:?}", ch.rules());
}
