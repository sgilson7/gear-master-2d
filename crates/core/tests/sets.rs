//! Three sets, and the two rules that are the point of them.
//!
//! Each set is exactly one grid's recipe, so what a creature drops assembles
//! into a finished item with nothing bought to complete it. What makes it a
//! *set* rather than three components is `AssemblyBonus::names` and
//! `AssemblyBonus::grants`, and both are read through `loadout::set_of`, which
//! is the one answer in the game to "is this the Mandate".
//!
//! Two of these were promised to M9.0 and land here, because both needed a
//! component in `CATALOG` that grants a rule and there was not one until now:
//! `a_rule_from_an_item_reaches_the_fight` and
//! `an_unassembled_set_grants_nothing`.

mod common;

use common::bench;
use gm2d_core::combat::{Difficulty, Event, Side};
use gm2d_core::curse::CurseKind;
use gm2d_core::data;
use gm2d_core::fight;
use gm2d_core::game::Game;
use gm2d_core::loadout::{set_of, set_pieces, whole_set};
use gm2d_core::piece::{SlotKind, MANDATE, SETS, TOAD_FRAME, WEAVE};
use gm2d_core::rng::Rng;
use gm2d_core::rule::Rule;
use gm2d_core::world::{self, Allowances, Dir, WorldState};

const D: Difficulty = Difficulty::Easy;

/// Where each set sits so that its components touch and the item assembles.
const MANDATE_ON_THE_BOARD: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Ratskin Material", SlotKind::Gloves, 0, 0, 0),
    ("Ratskin Mold", SlotKind::Gloves, 2, 0, 0),
    ("Rat Signet", SlotKind::Gloves, 4, 0, 0),
];
const TOAD_ON_THE_BOARD: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Toad Frame", SlotKind::Chest, 0, 0, 0),
    ("Toad Hide", SlotKind::Chest, 3, 0, 0),
];
const WEAVE_ON_THE_BOARD: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Bone Crown", SlotKind::Helmet, 0, 0, 0),
    ("Bone Scale", SlotKind::Helmet, 3, 0, 0),
    ("Bone Fletch", SlotKind::Helmet, 1, 1, 0),
];

/// Seat an arrangement **without locking as it goes**.
///
/// `common::seat` locks each item as it assembles, which is the right thing
/// almost everywhere and exactly wrong here: the pair that assembles first
/// locks, and the third component of the set then arrives as a separate item.
/// A set is one group or it is not the set.
fn seat(ch: &mut gm2d_core::character::Character, rows: &[(&str, SlotKind, u8, u8, u8)]) {
    for &(name, slot, x, y, rot) in rows {
        let id = ch.find_by_name(name).unwrap_or_else(|| panic!("nobody owns a {name}"));
        ch.registry.set_rotation(id, rot);
        ch.equip(id, slot, x, y)
            .unwrap_or_else(|e| panic!("failed to seat {name} at {slot:?} ({x}, {y}): {e}"));
    }
}

fn wearing(rows: &[(&str, SlotKind, u8, u8, u8)]) -> Game {
    let mut g = Game::new(0x5E75, "td");
    g.character = bench();
    seat(&mut g.character, rows);
    g
}

// ------------------------------------------------------------------ the file

/// Every set is whole, in one grid, and assembles out of what one creature
/// drops.
#[test]
fn every_set_is_one_creatures_and_one_grids() {
    let drops = data::drops();
    assert_eq!(SETS.len(), 9, "three from M9 and six from M11.9");
    // **One stack of floors counts as one owner**, and that is M11.9's
    // widening. The Curd Mantle's three pieces are certainties off the Drambus
    // Stack's fifth, third and first floors, so *climbing the tower* is the
    // grind — which is the same shape as one creature's drop table and not a
    // shopping list. A floor is one sitting, so a set off a floor's *pool*
    // would be unfarmable; a set off its bosses is the tower paying at the
    // start, the middle and the top, which is what `PLAN-M11.md` §8 row 8 asks
    // for by a different route.
    let stacks: Vec<Vec<String>> = data::MAPS
        .iter()
        .flat_map(|(id, _)| data::map(id, D).places.clone())
        .filter(|p| !p.floors.is_empty())
        .map(|p| p.floors.iter().map(|f| f.map.clone()).collect())
        .collect();
    let stack_of = |piece: &str| -> Option<usize> {
        stacks.iter().position(|maps| {
            maps.iter().any(|m| {
                data::map(m, D).places.iter().any(|p| p.drops.iter().any(|d| d == piece))
            })
        })
    };

    for &set in SETS {
        let pieces = set_pieces(set);
        assert!(pieces.len() >= 2, "{set} is one component, which is not a set");
        // One creature owns the whole of it, or a set is a shopping list.
        let mut owners: Vec<&str> = pieces
            .iter()
            .flat_map(|p| drops.drops.iter().filter(move |d| d.piece == **p))
            .map(|d| d.creature.as_str())
            .collect();
        owners.sort_unstable();
        owners.dedup();
        if owners.is_empty() {
            let mut from: Vec<Option<usize>> = pieces.iter().map(|p| stack_of(p)).collect();
            from.dedup();
            assert_eq!(
                from.len(),
                1,
                "{set} comes off no one creature and no one stack of floors"
            );
            assert!(from[0].is_some(), "{set} is dropped by nothing at all");
            continue;
        }
        assert_eq!(owners.len(), 1, "{set} is dropped by {owners:?}, which is a shopping list");
        // And one grid, because an item lives in one.
        let slots: Vec<SlotKind> = pieces
            .iter()
            .map(|p| gm2d_core::piece::CATALOG.iter().find(|d| d.name == *p).unwrap().slot)
            .collect();
        assert!(slots.windows(2).all(|w| w[0] == w[1]), "{set} spans {slots:?}");
    }
    // Nothing names a set that is not in the list, which is what makes the
    // list worth having.
    for d in gm2d_core::piece::CATALOG {
        let Some(n) = d.assembly_bonus.and_then(|b| b.names) else { continue };
        assert!(SETS.contains(&n), "{} names {n:?}, which is not a shipped set", d.name);
    }
}

/// **Agreement and completeness, and neither is enough on its own.**
#[test]
fn a_set_is_the_set_or_it_is_gear() {
    let whole: Vec<(&str, Option<&str>)> = set_pieces(MANDATE)
        .into_iter()
        .map(|p| (p, Some(MANDATE)))
        .collect();
    assert_eq!(whole_set(whole.clone()), Some(MANDATE));

    // Two thirds of it. Legal gloves — the ring is optional in the recipe —
    // and not the Mandate.
    let mut partial = whole.clone();
    partial.pop();
    assert_eq!(whole_set(partial), None, "a partial set called itself whole");

    // The whole of it plus somebody else's ring.
    let mut plus = whole.clone();
    plus.push(("Signet of Vigour", None));
    assert_eq!(whole_set(plus), None, "a stranger in the item and it was still the Mandate");

    // Two sets at once is neither.
    let mixed = vec![("Ratskin Mold", Some(MANDATE)), ("Toad Hide", Some(TOAD_FRAME))];
    assert_eq!(whole_set(mixed), None);
    assert_eq!(whole_set(Vec::<(&str, Option<&str>)>::new()), None);
}

// ------------------------------------------------------------------ the name

/// A set has the name somebody wrote, and only once it is whole.
#[test]
fn a_whole_set_takes_its_own_name() {
    let g = wearing(MANDATE_ON_THE_BOARD);
    let report = g.character.report(SlotKind::Gloves);
    let item = report
        .items
        .iter()
        .find(|i| i.assembled)
        .expect("the Mandate did not assemble, so nothing here is being tested");
    assert_eq!(item.name.full, MANDATE);
    assert_eq!(item.name.short, MANDATE, "there is no short form of a name that is one thing");
    assert_eq!(set_of(&g.character.registry, &item.pieces), Some(MANDATE));

    // Take the signet back off and it is a glove again, with a generated name.
    let mut g = g;
    let signet = g.character.find_by_name("Rat Signet").unwrap();
    g.character.unequip(signet).unwrap();
    let report = g.character.report(SlotKind::Gloves);
    let item = report.items.iter().find(|i| i.assembled).expect("still a legal glove");
    assert_ne!(item.name.full, MANDATE, "two thirds of it answered to the whole name");
}

// ----------------------------------------------------------------- the rules

/// **A rule from an item reaches the fight**, and does not without the item.
///
/// The Weave's is a rule the engine already reads end to end — the Patent's
/// nodes have granted it since M8.3 — so the third set costs no new combat code
/// and proves the door at the same time.
#[test]
fn a_rule_from_an_item_reaches_the_fight() {
    let g = wearing(WEAVE_ON_THE_BOARD);
    let want = Rule::CurseOnActivate { slot: "helmet".into(), curse: "misfire".into() };
    assert!(g.character.rules().contains(&want), "{:?}", g.character.rules());
    assert!(g.character.start_with().rules.contains(&want), "it did not reach `Held`");

    let mut g = g;
    g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let log = fight::run(&g, D).unwrap();
    let cursed = log.entries.iter().any(|e| {
        matches!(&e.event, Event::Cursed { on: Side::Enemy, kind: CurseKind::Misfire, .. })
    });
    assert!(cursed, "the helmet activated all fight and nothing was ever cursed");

    // And nothing at all with the set in the bag rather than on the board.
    let mut bare = Game::new(0x5E75, "td");
    bare.character = bench();
    assert!(bare.character.rules().is_empty(), "{:?}", bare.character.rules());
}

/// **An unassembled set grants nothing**, which is the whole of "recombined in
/// your inventory".
#[test]
fn an_unassembled_set_grants_nothing() {
    let mut g = Game::new(0x5E75, "td");
    g.character = bench();
    // Seated, but far enough apart that they are three components rather than
    // one item. The recipe is satisfied and the group is not connected.
    seat(
        &mut g.character,
        &[
            ("Ratskin Material", SlotKind::Gloves, 0, 0, 0),
            ("Ratskin Mold", SlotKind::Gloves, 3, 3, 0),
            ("Rat Signet", SlotKind::Gloves, 0, 7, 0),
        ],
    );
    assert_eq!(
        g.character.report(SlotKind::Gloves).assembled_count(),
        0,
        "they assembled, so this test is checking nothing"
    );
    assert!(g.character.rules().is_empty(), "{:?}", g.character.rules());
}

/// And a set two thirds of the way there grants nothing either.
#[test]
fn a_partial_set_grants_nothing() {
    let mut g = Game::new(0x5E75, "td");
    g.character = bench();
    seat(&mut g.character, &MANDATE_ON_THE_BOARD[..2]);
    assert_eq!(
        g.character.report(SlotKind::Gloves).assembled_count(),
        1,
        "the pair has to be a legal glove or this proves nothing"
    );
    assert!(g.character.rules().is_empty(), "two thirds of the Mandate routed a rat");
}

// ------------------------------------------------------------------ the rout

/// The Mandate routs a rat and nothing else.
#[test]
fn the_mandate_routs_a_rat_and_nothing_else() {
    let mut g = wearing(MANDATE_ON_THE_BOARD);
    assert!(g.character.rules().iter().any(|r| matches!(r, Rule::Rout { .. })));

    g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let before = g.character.gold;
    let r = fight::rout(&mut g).expect("the Mandate did not rout a rat");
    assert_eq!(r.creature, "Cave Rat");
    assert!(r.gold > 0 && r.xp > 0, "being the Rat King was worth nothing");
    assert_eq!(g.character.gold, before + r.gold);
    assert!(g.encounter.is_none(), "a routed creature is still standing there");
    assert!(r.receipt.iter().any(|l| l.contains("A. Rat")), "{:?}", r.receipt);

    // A toad is still a fight.
    g.encounter = Some(fight::Encounter { enemy: "Bog Toad".into(), at: [1, 18] });
    assert!(fight::rout(&mut g).is_none(), "one set was a pass for the whole region");
    assert!(g.encounter.is_some(), "the toad was taken off the tile anyway");
}

/// **A rout costs no tiredness**, because nothing was fought — and a player
/// will check, so the receipt says so.
#[test]
fn a_rout_costs_nothing_and_says_so() {
    let mut g = wearing(MANDATE_ON_THE_BOARD);
    g.character.tire(20);
    g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let r = fight::rout(&mut g).unwrap();
    assert_eq!(g.character.fatigue, 20, "a rout tired somebody out");
    assert!(r.receipt.iter().any(|l| l.contains("0%")), "{:?}", r.receipt);
}

/// **A rout is not a farm**, which is the family `a_lose_win_cycle_is_not_a_gold_farm`
/// already guards and this is its sibling.
///
/// The divergence in `reward.rs` is about a *loss* paying; this is a win that
/// costs nothing at all. It pays, deliberately — six Fnorp is not a farm and
/// being the Rat King should be worth something — and what makes that safe is
/// that it is the *weakest* creature in the game, so a routed rat is worth
/// strictly less than the fight the player could have had instead.
#[test]
fn a_rout_pays_a_rat_and_a_rat_is_the_cheapest_thing_there_is() {
    let ladder = gm2d_core::combat::LADDER;
    let rat = gm2d_core::combat::creature("Cave Rat").unwrap();
    let dearer = ladder.iter().filter(|m| m.bounty > rat.bounty).count();
    assert_eq!(
        dearer,
        ladder.len() - 1,
        "something in the ladder pays no more than a Cave Rat, so routing one is \
         no longer the cheapest possible income"
    );
    // And a routed rat pays exactly what a fought one does, and no more.
    let mut a = wearing(MANDATE_ON_THE_BOARD);
    a.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: [1, 18] });
    let routed = fight::rout(&mut a).unwrap();
    assert_eq!(routed.gold, rat.bounty, "a rout paid more than the creature is worth");
}

/// A boss is never routed. The same rule that looks a boss drop up by the tile.
#[test]
fn a_boss_is_never_routed() {
    let cave = data::map("the-great-gear-cave", D);
    let boss = cave
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Boss)
        .expect("the cave has a boss");
    let who = boss.creature.clone().expect("standing there");

    let mut g = wearing(MANDATE_ON_THE_BOARD);
    g.world.map = cave.id.clone();
    g.world.at = boss.at;
    // Rigged: the set routs whatever is standing on the tile, which is exactly
    // the case the guard exists for.
    g.encounter = Some(fight::Encounter { enemy: who, at: boss.at });
    // The boss is not a Cave Rat, so first prove the guard is not being reached
    // by accident — then rout something the set really does name, on the same
    // tile, and watch it be refused all the same.
    assert!(fight::rout(&mut g).is_none());
    g.encounter = Some(fight::Encounter { enemy: "Cave Rat".into(), at: boss.at });
    assert!(
        fight::rout(&mut g).is_none(),
        "a set walked past the thing at the bottom of the cave, and the key with it"
    );
}

// ------------------------------------------------------------------ the wade

/// **The toad set opens the lake, all of it, since M11.4.**
///
/// It opened the rim and not the middle for two blocks, and the measurement
/// behind that is still in `tests/rules.rs` — fourteen of twenty-eight, row
/// nine crossable end to end. What changed is that there is a grating in the
/// middle of the lake now with two hundred and six steps under it, and the set
/// somebody ground three Bog Toads for is how you reach it before the Drambus
/// Stack comes down and empties the whole thing. `PLAN-M11.md` §8 row 1: widen
/// the rule rather than stand a second, deeper one beside it.
#[test]
fn the_toad_set_opens_the_whole_lake() {
    let g = wearing(TOAD_ON_THE_BOARD);
    assert_eq!(
        g.character.report(SlotKind::Chest).assembled_count(),
        1,
        "the toad set did not assemble"
    );
    assert!(g.character.allowances().wade, "{:?}", g.character.rules());

    let w = data::world(D);
    let allowed = g.character.allowances();
    let opened: Vec<[u8; 2]> = (0..w.height)
        .flat_map(|y| (0..w.width).map(move |x| (x, y)))
        .filter(|&(x, y)| !w.passable(x, y) && w.walkable(x, y, &allowed))
        .map(|(x, y)| [x, y])
        .collect();
    assert_eq!(opened.len(), 28, "the lake is twenty-eight tiles: {opened:?}");
    // Row nine end to end, which is what made it crossable at all.
    for x in 7..=10 {
        assert!(opened.contains(&[x, 9]), "({x}, 9) did not open");
    }
    // And the fourteen that used to be shut — four, six, four.
    let was_shut: Vec<(u8, u8)> = (7..=10)
        .map(|x| (x, 10))
        .chain((6..=11).map(|x| (x, 11)))
        .chain((7..=10).map(|x| (x, 12)))
        .collect();
    assert_eq!(was_shut.len(), 14);
    for (x, y) in was_shut {
        assert_eq!(w.terrain_name(x, y), "water", "({x}, {y}) is not even lake");
        assert!(w.walkable(x, y, &allowed), "({x}, {y}) is the middle and it is still shut");
    }
    // And what it opens is still water and nothing else: an allowance adds and
    // never converts.
    assert!(
        !opened.iter().any(|c| w.terrain_name(c[0], c[1]) != "water"),
        "something that is not water opened"
    );
    // The whole reason: the grating is in the middle of it.
    let grating = w
        .places
        .iter()
        .find(|p| p.id == "the-way-under-the-lake")
        .expect("the way under the lake");
    assert!(opened.contains(&grating.at), "the set does not reach the way down");
}

/// Take the chest apart and the lake is a wall again.
#[test]
fn a_set_that_is_not_assembled_grants_nothing() {
    let mut g = wearing(TOAD_ON_THE_BOARD);
    let w = data::world(D);
    let mut rng = Rng::new(4);
    g.world = WorldState { at: [7, 8], ..WorldState::at_start(&w) };

    let s = world::step(&w, &mut g.world, &mut rng, D, Dir::South, &g.character.allowances());
    assert!(s.moved, "the toad set did not open the lake");
    assert_eq!(g.world.at, [7, 9]);

    // Off the board and into the bag. Still owned, and no longer worn.
    let hide = g.character.find_by_name("Toad Hide").unwrap();
    g.character.unequip(hide).unwrap();
    assert!(g.character.holds("Toad Hide"), "it should still be in the bag");
    assert!(!g.character.allowances().wade, "a set in the bag still walked on water");

    // And the repair that follows puts them back on ground rather than
    // leaving them standing in a lake they can no longer be in.
    let was = w.repair(&mut g.world, &g.character.allowances());
    assert_eq!(was, Some([7, 9]));
    assert!(w.passable(g.world.at[0], g.world.at[1]));
}

/// Wading only ever adds, so nothing the map already promised has moved.
#[test]
fn wading_does_not_move_a_place_or_a_region() {
    let allowed = Allowances { wade: true, ..Allowances::default() };
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        // **One place stands on ground only a set reaches, and it is the
        // point of that set.** Everything else has to be ordinary ground: a
        // town or a card behind a rule is content three players in four never
        // find, and this is the check that stops the next one being an
        // accident rather than a decision.
        for p in &w.places {
            if p.id == "the-way-under-the-lake" {
                assert_eq!(w.terrain_name(p.at[0], p.at[1]), "water");
                continue;
            }
            assert!(
                w.passable(p.at[0], p.at[1]),
                "{}: {} stands on ground that only a waded set could reach",
                id,
                p.id
            );
        }
        for y in 0..w.height {
            for x in 0..w.width {
                if w.passable(x, y) {
                    assert!(w.walkable(x, y, &allowed), "{id} ({x}, {y}) stopped being ground");
                }
            }
        }
    }
}

// ---------------------------------------------------------- and the old saves

/// **The catalogue fingerprint moved, and a save from before it is refused.**
///
/// Stated rather than discovered. The refusal names both catalogues, which is
/// the sentence a player gets instead of a game that loads and is subtly wrong.
#[test]
fn a_save_from_before_this_block_is_refused_by_name() {
    let g = Game::new(1, "td");
    let text = gm2d_core::save::save(&g);
    let now = gm2d_core::piece::CATALOG.len();
    // **568 since M11.9**, which added six sets of three. It was 550 from
    // M11.5, which added the three instruments' six parts; 544 from M9.1; and
    // 536 before that. This line is where a move is said out loud — the deploy
    // note says it too, in the sentence a seam always gets. The block has two
    // seams and no more: M11.5 and M11.9.
    assert_eq!(now, 568, "the catalogue moved again and this line owns saying so");
    // A file written against the M9 catalogue: 544 components, and whatever
    // fingerprint that had. Either half is enough to refuse it.
    let older = text
        .replace(&format!("\"pieces\": {now}"), "\"pieces\": 550")
        .replace(
            &format!("\"fingerprint\": \"{}\"", gm2d_core::save::catalog_fingerprint()),
            "\"fingerprint\": \"0000000000000000\"",
        );
    assert_ne!(older, text, "the save no longer states its catalogue size");
    let why = gm2d_core::save::load(&older).expect_err("a pre-M11.9 save loaded");
    assert!(why.contains("550"), "{why}");
    assert!(why.contains(&gm2d_core::piece::CATALOG.len().to_string()), "{why}");
}

/// The set names are keys, so a typo in one is a set that can never be whole.
#[test]
fn every_set_name_is_matched_by_something() {
    for &set in SETS {
        assert!(!set_pieces(set).is_empty(), "nothing in the catalogue names {set:?}");
    }
    assert!(set_pieces("The Weave of Nothing At All").is_empty());
    assert_eq!(set_pieces(WEAVE).len(), 3);
}
