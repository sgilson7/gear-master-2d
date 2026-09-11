//! Shared fixtures for the integration tests.
//!
//! Upstream's version of this file was built around `Run`, and around three
//! things GM2D does not have: share codes, a dungeon fixture, and a run mode.
//! What is left is the two jobs the surviving suite actually asks for —
//! walking a piece's actions, and seating a board — and the second is now done
//! against [`Character`] instead of a campaign.
#![allow(dead_code)] // each test binary uses a different subset

use gm2d_core::character::Character;
use gm2d_core::piece::{Action, PieceDef, PieceId, SlotKind, Trigger, CATALOG};

/// Run `f` over every action a trigger can reach.
///
/// This was a copy of `piece::walk_actions` and is now a call to it. The
/// engine's own doc had already noticed: *"The test suite has carried a copy
/// of this for a while; `rating.rs` needs the same answer, and two of them
/// would drift."* They drifted the moment a trigger variant was added — the
/// engine's walker knew about `OnEnemyActivate` and this one did not, and the
/// only reason that was caught is that the match was exhaustive.
///
/// One walker. A lint over the catalogue that misses a payload is a lint that
/// reports a clean catalogue.
pub fn actions_of(t: &Trigger, f: &mut impl FnMut(&Action)) {
    gm2d_core::piece::walk_actions(t, f)
}

/// Does any action this piece can reach satisfy `want`?
pub fn does(def: &PieceDef, want: fn(&Action) -> bool) -> bool {
    let mut hit = false;
    for t in def.triggers {
        actions_of(t, &mut |a| hit |= want(a));
    }
    hit
}

/// Does this piece carry a trigger satisfying `want`?
pub fn has(def: &PieceDef, want: fn(&Trigger) -> bool) -> bool {
    def.triggers.iter().any(want)
}

// ------------------------------------------------------------ seating a board

/// A character owning one of every component, for tests that need to arrange
/// arbitrary pieces without shopping for them.
pub fn bench() -> Character {
    Character::with_all_pieces()
}

/// Look an owned component up by name.
pub fn piece(ch: &Character, name: &str) -> PieceId {
    ch.find_by_name(name)
        .unwrap_or_else(|| panic!("no piece named {name}"))
}

/// Look up the first *unworn* component with this name.
///
/// Distinct from [`piece`] because `with_all_pieces` owns exactly one of each,
/// and a test that seats the same name twice wants to know that rather than
/// silently move the piece it already placed.
pub fn spare(ch: &Character, name: &str) -> PieceId {
    ch.owned
        .iter()
        .copied()
        .find(|&id| ch.registry.def(id).name == name && !ch.is_equipped(id))
        .unwrap_or_else(|| panic!("no unworn piece named {name}"))
}

/// Equip by name, failing loudly with the reason if the placement is illegal.
pub fn equip(ch: &mut Character, name: &str, slot: SlotKind, ax: u8, ay: u8) {
    let id = piece(ch, name);
    ch.equip(id, slot, ax, ay)
        .unwrap_or_else(|e| panic!("failed to equip {name} at ({ax}, {ay}): {e}"));
}

/// Seat a whole board from `(name, slot, x, y, rotation)` rows.
///
/// **Locks as each item completes**, which is what a player does while
/// building and what upstream's `share.rs` learned to do the expensive way: a
/// densely packed board derived in one pass at the end asks which pieces are
/// connected, and the answer on a full grid is "most of them" — nineteen
/// weapon pieces came back as one item. Anything that wants a board *without*
/// locks should seat it by hand.
pub fn seat(ch: &mut Character, rows: &[(&str, SlotKind, u8, u8, u8)]) {
    for &(name, slot, x, y, rot) in rows {
        let id = spare(ch, name);
        ch.registry.set_rotation(id, rot);
        ch.equip(id, slot, x, y)
            .unwrap_or_else(|e| panic!("failed to seat {name} at {slot:?} ({x}, {y}): {e}"));
        gm2d_core::loadout::lock_assembled_in(&mut ch.loadout, &ch.registry, slot);
    }
}

/// The preset board: a complete, legal loadout that assembles all five slots
/// and lights every assembly bonus.
///
/// Lifted verbatim from upstream's `Run::apply_preset`, which was the
/// auto-build button and the test fixture at once so the two could not drift.
/// GM2D has no auto-build button yet; when it grows one it should call this
/// rather than grow a second arrangement.
///
/// Deliberately shows off the mechanics rather than maxing the numbers: chest,
/// gloves and greaves each carry two separate finished items, the weapon's
/// Runed Edge doubles the Ruby Inlay next to it, and the Hollow Weave sits out
/// in open space where its empty-cell bonus counts.
pub const PRESET: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
    ("Crest of Vigor", SlotKind::Helmet, 3, 0, 0),
    ("Iron Plating", SlotKind::Helmet, 0, 2, 0),
    ("Visor of Focus", SlotKind::Helmet, 0, 4, 0),
    ("Padded Base", SlotKind::Chest, 0, 0, 0),
    ("Keystone Base", SlotKind::Chest, 0, 0, 0),
    ("Hollow Weave", SlotKind::Chest, 5, 2, 1),
    ("Chain Layer", SlotKind::Chest, 0, 3, 0),
    ("Woven Underlayer", SlotKind::Chest, 0, 4, 0),
    ("Hide Base", SlotKind::Chest, 3, 6, 0),
    ("Leather Material", SlotKind::Gloves, 0, 0, 0),
    ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
    ("Steel Material", SlotKind::Gloves, 0, 4, 0),
    ("Gauntlet Mold", SlotKind::Gloves, 2, 4, 0),
    ("Runed Material", SlotKind::Greaves, 0, 0, 0),
    ("Greave Mold", SlotKind::Greaves, 2, 0, 0),
    ("Boiled Leather", SlotKind::Greaves, 0, 4, 0),
    ("Runner's Mold", SlotKind::Greaves, 3, 4, 0),
    ("Balanced Grip", SlotKind::Weapon, 0, 0, 0),
    ("Runed Edge", SlotKind::Weapon, 1, 0, 0),
    ("Ruby Inlay", SlotKind::Weapon, 2, 0, 0),
    ("Balance Weight", SlotKind::Weapon, 2, 2, 0),
];

/// A character wearing [`PRESET`], seated without locks — the arrangement the
/// golden fixture was captured from.
pub fn preset_board() -> Character {
    let mut ch = Character::with_all_pieces();
    for &(name, slot, x, y, rot) in PRESET {
        let Some(id) = ch.owned.iter().copied().find(|&i| {
            ch.registry.def(i).name == name && !ch.is_equipped(i)
        }) else {
            continue;
        };
        ch.registry.set_rotation(id, rot);
        if ch.can_equip(id, slot, x, y).is_ok() {
            let _ = ch.equip(id, slot, x, y);
        }
    }
    ch
}

/// Every catalogue index, for lints that walk the whole catalogue.
pub fn all_def_indices() -> Vec<usize> {
    (0..CATALOG.len()).collect()
}

/// A known-good full board, seated by name.
///
/// **Its own list, not `apply_preset`.** It was that until M8.8, when Auto-pack
/// stopped being a fixed arrangement and became a packer — and the assembly
/// tests that use this are about *recipes*, not about the button. Sharing one
/// list meant a change to how the button packs broke four tests about what an
/// assembly bonus does, which is two subjects tangled in one helper.
///
/// This is the arrangement `apply_preset` used to seat, kept here because it
/// is a board somebody checked: every grid assembles, and chest, gloves and
/// greaves each carry two separate items.
pub fn build_full_loadout(ch: &mut Character) {
    const PRESET: &[(&str, SlotKind, u8, u8, u8)] = &[
        ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
        ("Crest of Vigor", SlotKind::Helmet, 3, 0, 0),
        ("Iron Plating", SlotKind::Helmet, 0, 2, 0),
        ("Visor of Focus", SlotKind::Helmet, 0, 4, 0),
        ("Padded Base", SlotKind::Chest, 0, 0, 0),
        ("Keystone Base", SlotKind::Chest, 0, 0, 0),
        ("Hollow Weave", SlotKind::Chest, 5, 2, 1),
        ("Chain Layer", SlotKind::Chest, 0, 3, 0),
        ("Woven Underlayer", SlotKind::Chest, 0, 4, 0),
        ("Hide Base", SlotKind::Chest, 3, 6, 0),
        ("Leather Material", SlotKind::Gloves, 0, 0, 0),
        ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
        ("Steel Material", SlotKind::Gloves, 0, 4, 0),
        ("Gauntlet Mold", SlotKind::Gloves, 2, 4, 0),
        ("Runed Material", SlotKind::Greaves, 0, 0, 0),
        ("Greave Mold", SlotKind::Greaves, 2, 0, 0),
        ("Boiled Leather", SlotKind::Greaves, 0, 4, 0),
        ("Runner's Mold", SlotKind::Greaves, 3, 4, 0),
        ("Balanced Grip", SlotKind::Weapon, 0, 0, 0),
        ("Runed Edge", SlotKind::Weapon, 1, 0, 0),
        ("Ruby Inlay", SlotKind::Weapon, 2, 0, 0),
        ("Balance Weight", SlotKind::Weapon, 2, 2, 0),
    ];
    for k in SlotKind::ALL {
        ch.loadout.slot_mut(k).clear();
    }
    for &(name, kind, ax, ay, rot) in PRESET {
        let Some(id) = ch.find_by_name(name) else { continue };
        ch.registry.set_rotation(id, rot);
        ch.loadout.remove_anywhere(id);
        if ch.can_equip(id, kind, ax, ay).is_ok() {
            let _ = ch.equip(id, kind, ax, ay);
        }
    }
}

/// A character carrying what the game actually hands out by a given point.
///
/// **The fixture the tower needed and nothing had.** There were two boards to
/// test against and neither is a player: `Character::starting()` owns two
/// components, and `bench()` owns all five hundred and forty-four and packs to
/// seven thousand health, which beats every creature in the ladder including
/// Francis. A question like *is this floor beatable* has no answer against
/// either of them.
///
/// This is the third: every shelf named, every errand reward in the game, every
/// set piece any creature drops, on a board grown to its ceiling, arranged by
/// the button a player is given. It is generous — it assumes you bought the
/// whole of both shelves and finished everything — but it is generous about
/// *reachable* content, which is the direction that makes a difficulty check
/// mean something.
///
/// Measured when it was written: 55 components, 1,039 health, 300% weapon
/// power, and it wins about half its fights between rating 450 and 1,550. The
/// half it loses is what the packing screen is for.
pub fn geared_from(towns: &[&str]) -> Character {
    use gm2d_core::data;
    let mut ch = Character::starting();
    // The boards cap at level six; twenty is comfortably past it.
    ch.grow_boards(20);
    let shops = data::shops();
    for town in towns {
        for name in &shops.town(town).expect("a shelf that exists").stock {
            ch.give(name);
        }
    }
    // **At most one chain reward per root, because the chains are exclusive.**
    // An event is answered once, so taking one half of The Gate That Was Yours
    // closes its other two chains for good. Handing this fixture all twenty-one
    // chain rewards would build a character no playthrough can produce — and
    // it did: it made floor five of the Stack trivially winnable and
    // `the_floors_cost_more_than_the_things_at_the_end_of_them` said so.
    //
    // A `granted` errand is one a *choice* hands over, and its `giver` is the
    // event it comes out of, so one per giver is exactly one branch per root.
    let quests = data::quests();
    let mut roots_taken: Vec<&str> = Vec::new();
    for q in &quests.quests {
        if q.granted {
            if roots_taken.contains(&q.giver.as_str()) {
                continue;
            }
            roots_taken.push(&q.giver);
        }
        for r in &q.reward {
            ch.give(r);
        }
    }
    for d in &data::drops().drops {
        ch.give(&d.piece);
    }
    ch.apply_preset();
    ch
}

/// **Finished items in a row, each touching the next**, in the gloves, and
/// nothing else on the board.
///
/// The repository had no such fixture and needed one the moment `Rule::Beacon`
/// landed: `build_full_loadout` makes eight items and **no two of them share an
/// edge**, because that preset spaces them out on purpose — so anything
/// measured against it about *neighbours* is measured against a board with
/// none. The only board in the suite with adjacency was Auto-pack's over the
/// whole catalogue, which is nineteen items and a second and a half of packing
/// every time a test asks for one.
///
/// A row rather than a cluster, because the two questions a lending rule has
/// are *does it reach my neighbour* and *does it reach my neighbour's
/// neighbour* — and three in a line is the smallest board that can ask the
/// second.
///
/// **Seated through `seat`, which locks each item as it completes.** Two items
/// that touch and are not locked are one item: `loadout` groups by adjacency
/// and a lock is the only thing that says otherwise. That is upstream's
/// expensive lesson — nineteen weapon pieces came back as one — and it is the
/// whole reason this cannot be written as six `equip` calls.
///
/// Returns nothing: the items are `ch.combat_items()` in seating order, which
/// is the order they were laid, so item `i` touches `i - 1` and `i + 1`.
pub fn items_in_a_row(ch: &mut Character, n: usize) {
    // A glove is a Material and a Mold that touch. Four two-by-two materials
    // and four two-wide molds, none of which is anything else's — `with_all_
    // pieces` owns one of each name, so a row of four needs four of each.
    const MATERIALS: &[&str] =
        &["Leather Material", "Scaled Material", "Waxed Material", "Hide Material"];
    const MOLDS: &[&str] = &["Padded Mold", "Braced Mold", "Vicegrip Mold", "Gripping Mold"];
    assert!(n >= 2 && n <= MATERIALS.len(), "a row of {n} is not one this fixture can lay");

    for k in SlotKind::ALL {
        ch.loadout.slot_mut(k).clear();
    }
    ch.loadout.slot_mut(SlotKind::Gloves).grow(2 * n as u8);
    let rows: Vec<(&str, SlotKind, u8, u8, u8)> = (0..n)
        .flat_map(|i| {
            let y = (i * 2) as u8;
            [
                (MATERIALS[i], SlotKind::Gloves, 0, y, 0),
                (MOLDS[i], SlotKind::Gloves, 2, y, 0),
            ]
        })
        .collect();
    seat(ch, &rows);

    let made = ch.combat_items();
    assert_eq!(made.len(), n, "the fixture made {} items, not {n}", made.len());
    for i in 0..n {
        for j in [i.wrapping_sub(1), i + 1] {
            if j >= n {
                continue;
            }
            assert!(
                made[i].adjacent_items.contains(&j),
                "item {i} does not touch {j}: {:?}",
                made[i].adjacent_items,
            );
        }
    }
}

/// Two finished items that touch, which is the smallest board with a neighbour
/// on it. See [`items_in_a_row`].
pub fn two_items_that_touch(ch: &mut Character) {
    items_in_a_row(ch, 2);
}


/// Towns that are on a map and deliberately have nothing in them.
///
/// **The mirror of `avail.rs`'s `STAGED`, and the same rule from the other
/// side.** That one is a shelf with no ground under it; this is ground with no
/// shelf on it, and both are fine *only* because somebody wrote the name down.
/// A third empty town fails wherever this is read.
///
/// One entry, and it is `PLAN-M14.md` §1.5: the third town ships with a name, a
/// start tile and a region, and a shop with no shelves and no errands. **It is
/// not a placeholder drawn as a town, it is a town drawn honestly** — what goes
/// in it is the next plan's, and a shelf invented to keep a lint quiet would be
/// content nobody asked for standing exactly where the content that *was* asked
/// for has to go.
///
/// **Here rather than in one of the two test files that read it**, because two
/// copies of a list of exceptions is two places for it to go stale — which is
/// the sixth time this project has paid for a hand-written list and the first
/// time it was caught before it was written twice.
pub const UNWRITTEN: &[&str] = &["the-third-town"];

/// **The run**, off disk, as a `Character`.
///
/// `PLAN-M16.md` §1.1: this block is bracketed against a real save at level
/// forty-five, not against a walker's board. `HANDOFF-M14.md` §2.7 is why — *a
/// level-22 board is not a board this game produces*, and the two M14 bosses
/// were rated against one. [`geared_from`] stays, and is still the right
/// yardstick for the early game; this is the deep one.
///
/// **It refuses loudly rather than patching.** A save whose catalogue
/// fingerprint has moved is a save this build cannot open, and the one thing a
/// fixture must never do is quietly open a different character from the one it
/// names — so the panic carries `save::parse`'s own sentence, which names both
/// catalogues.
///
/// Path is relative to the crate root, which is where `cargo test` runs.
pub fn from_save(path: &str) -> Character {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(path);
    let text = std::fs::read_to_string(&here)
        .unwrap_or_else(|e| panic!("{}: {e}", here.display()));
    let game = gm2d_core::save::load(&text)
        .unwrap_or_else(|e| panic!("{}: {e}", here.display()));
    game.character
}

/// The one save this block is bracketed against.
pub const THE_RUN: &str = "testing/saves/the-run-20260910.json";

/// Everything a creature resists a curse with: its own number **plus its
/// gear's**.
///
/// The sum is the point. `Combatant` adds the two before anything asks, so a
/// creature written at 90 whose helmet carries another 30 resists at 120 — and
/// `CurseKind::landing_ms` clamps at [`gm2d_core::stats::MIND_CAP`], which is
/// 100, so every curse in the game lands on it for **zero**. Nobody had summed
/// them.
pub fn curse_resist_of(spec: &gm2d_core::combat::MonsterSpec) -> i32 {
    let (stats, _) = spec.outfit_at(gm2d_core::combat::Difficulty::Medium);
    stats.curse_resist
}

/// A character carrying one assembled instrument on the frame, and nothing else.
///
/// **The frame is six by three and outside `SlotKind::ALL`**, so this is not a
/// board: nothing that asks what gear is worth counts any of it. What it is for
/// is the handful of checks that need `Character::instrument` to answer, which
/// is the one door every `Requirement::Surveying` goes through.
///
/// The parts come off the recipe table rather than a list of names — *ask the
/// recipe table, never a list of kinds* — so a recipe that grows a part grows
/// this too.
pub fn with_instrument(kind: &str) -> Character {
    use gm2d_core::piece::PieceKind;
    let want: &[(PieceKind, usize)] = match kind {
        "compass" => &[(PieceKind::Shard, 1), (PieceKind::Lens, 1), (PieceKind::Magnet, 1)],
        "atlas" => &[
            (PieceKind::Shard, 2),
            (PieceKind::Lens, 1),
            (PieceKind::Orb, 1),
            (PieceKind::Alignment, 1),
        ],
        "golem" => &[(PieceKind::Shard, 3), (PieceKind::Earth, 2)],
        other => panic!("{other} is not an instrument"),
    };
    let mut ch = Character::starting();
    ch.clear_all();
    let mut at = 0u8;
    for &(k, n) in want {
        let def = CATALOG
            .iter()
            .find(|d| d.kind == k && d.slot == SlotKind::Instrument)
            .unwrap_or_else(|| panic!("the catalogue has no {k:?} for the frame"));
        for _ in 0..n {
            let id = ch.give(def.name).expect("a catalogue name");
            let mut seated = false;
            'cells: for y in 0..3u8 {
                for x in 0..6u8 {
                    if ch.equip(id, SlotKind::Instrument, x, y).is_ok() {
                        seated = true;
                        break 'cells;
                    }
                }
            }
            assert!(seated, "{} will not sit on the frame", def.name);
            at += 1;
        }
    }
    let _ = at;
    assert_eq!(
        ch.instrument(),
        Some(kind),
        "the frame did not come out as a {kind}"
    );
    ch
}
