//! The hard gate: a save that round-trips, and a bad file that explains itself.
//!
//! `PLANNING-BRIEF.md` §0.4 makes this the one requirement a build cannot ship
//! without, and M1 exists to prove it on the smallest state there is rather
//! than retrofit it onto a finished game. If anything in this file is red,
//! nothing else in the repo matters.

use gm2d_core::character::Character;
use gm2d_core::game::Game;
use gm2d_core::piece::{PieceId, SlotKind, CATALOG};
use gm2d_core::save::{self, SaveFile, FORMAT, VERSION};

/// A game with something in every field a save is supposed to carry.
///
/// Deliberately not a fresh one: a round-trip test on an empty board proves
/// that zero survives being written down.
fn a_played_game() -> Game {
    let mut g = Game::new(0x5EED_1234_ABCD_0001, "td");
    g.character = Character::with_all_pieces();
    g.character.loadout.name_seed = 0x5EED_1234_ABCD_0001;
    g.character.loadout.naming = gm2d_core::theme::by_id("td").naming;
    g.character.apply_preset();
    g.character.gold = 240;
    g.character.grown_health = 12;

    // A lock, because locks are state and re-deriving them is the mistake this
    // whole milestone is built to not make.
    let weapon = g.character.loadout.slot(SlotKind::Weapon).pieces()[0];
    g.character.toggle_lock_item(weapon);

    // Boards that are not all the same height, so a save that stored one row
    // count for all five would be caught.
    g.character.grow_slot(SlotKind::Weapon, 2);
    g.character.grow_slot(SlotKind::Chest, 1);

    // And a stream that has moved, so restoring the seed instead of the state
    // would be caught.
    for _ in 0..11 {
        g.rng.next_u64();
    }
    g
}

// ------------------------------------------------------------- the property

/// `load(save(g)) == g`, for a game with something in every field.
///
/// `Game`'s `PartialEq` is hand-written and says what "the same game" means:
/// the RNG's position, the theme, the boards, the locks, the name seed, what is
/// owned and what it is worth. It deliberately does not count the undo stack or
/// the naming pointer, because neither is the game.
#[test]
fn a_played_game_round_trips() {
    let before = a_played_game();
    let text = save::save(&before);
    let after = save::load(&text).expect("a save this build wrote should load");
    assert_eq!(before, after, "a game did not survive being written down");
}

/// The round trip is stable: saving what was loaded produces the same bytes.
///
/// Catches the class of fault where a field survives one hop and is normalised
/// on the second — a save that drifts every time it is opened is a save that
/// cannot be diffed, and diffing one is how the next bug in here gets found.
#[test]
fn saving_a_loaded_game_produces_the_same_file() {
    let text = save::save(&a_played_game());
    let again = save::save(&save::load(&text).unwrap());
    assert_eq!(text, again, "a save is not a fixed point");
}

/// The RNG resumes rather than restarts.
///
/// Called out separately from the round-trip property because it is the one
/// the brief names — "restoring the exact game state including the random
/// encounter RNG" — and because it is the one a `#[derive(PartialEq)]` on a
/// seed field would have passed while being wrong.
#[test]
fn the_random_stream_resumes_where_it_was_saved() {
    let mut before = a_played_game();
    let text = save::save(&before);
    let mut after = save::load(&text).unwrap();

    let want: Vec<u64> = (0..8).map(|_| before.rng.next_u64()).collect();
    let got: Vec<u64> = (0..8).map(|_| after.rng.next_u64()).collect();
    assert_eq!(want, got, "the next eight encounters would be different ones");

    let mut from_seed = Game::new(0x5EED_1234_ABCD_0001, "td");
    let restarted: Vec<u64> = (0..8).map(|_| from_seed.rng.next_u64()).collect();
    assert_ne!(
        got, restarted,
        "the stream restarted, so a save is handing the player their first encounter again"
    );
}

/// The board comes back as the same *items*, not merely the same pieces.
///
/// The distinction is the whole of what locks are for. Two pieces that touch
/// are one item unless a lock says otherwise, so a loader that re-derived the
/// locks would return a board holding every piece it started with, arranged
/// identically, that fights differently.
#[test]
fn the_same_pieces_come_back_as_the_same_items() {
    let before = a_played_game();
    let after = save::load(&save::save(&before)).unwrap();

    let want: Vec<(String, i32)> =
        before.character.combat_items().iter().map(|i| (i.name.clone(), i.rating)).collect();
    let got: Vec<(String, i32)> =
        after.character.combat_items().iter().map(|i| (i.name.clone(), i.rating)).collect();

    assert!(!want.is_empty(), "the fixture assembled nothing, so this proves nothing");
    assert_eq!(want, got, "the board came back as different items");
    assert_eq!(
        before.character.player_stats(),
        after.character.player_stats(),
        "the character sheet moved across a round trip"
    );
}

/// A fight fought after loading is the fight that would have been fought
/// before saving, entry for entry.
///
/// The end-to-end version of the property, and the one a player would actually
/// notice: everything above could pass while some field combat reads was
/// quietly reset.
#[test]
fn a_fight_after_loading_is_the_fight_before_saving() {
    use gm2d_core::combat::{simulate_at, Difficulty, LADDER};
    let before = a_played_game();
    let after = save::load(&save::save(&before)).unwrap();
    let spec = LADDER.iter().find(|m| m.name == "Rust Golem").unwrap();

    let a = simulate_at(before.character.player_stats(), &before.character.combat_items(), spec, Difficulty::Easy);
    let b = simulate_at(after.character.player_stats(), &after.character.combat_items(), spec, Difficulty::Easy);

    assert_eq!(a.outcome, b.outcome, "the fight ended differently");
    assert_eq!(a.duration_ms, b.duration_ms, "the fight took a different length of time");
    assert_eq!(
        format!("{:?}", a.entries),
        format!("{:?}", b.entries),
        "the fight went differently after a round trip"
    );
}

// ------------------------------------------------------------- bad files

/// Every refusal is a sentence, and none of them is a panic.
///
/// A bad file is a thing a player hands this program, not a bug in it. The
/// assertions check the message names the thing that is wrong, because
/// "failed to load" is not a message anybody can act on.
#[test]
fn a_file_this_build_cannot_read_says_why() {
    let good: SaveFile = save::parse(&save::save(&a_played_game())).unwrap();

    let mut wrong_format = good.clone();
    wrong_format.format = "gm2d-theme".into();
    let e = save::parse(&serde_json::to_string(&wrong_format).unwrap()).unwrap_err();
    assert!(e.contains("gm2d-theme") && e.contains(FORMAT), "should name both formats: {e}");

    let mut future = good.clone();
    future.version = VERSION + 1;
    let e = save::parse(&serde_json::to_string(&future).unwrap()).unwrap_err();
    assert!(
        e.contains(&(VERSION + 1).to_string()) && e.contains(&VERSION.to_string()),
        "should name both versions: {e}"
    );

    let mut other_catalog = good.clone();
    other_catalog.catalog.fingerprint = "0000000000000000".into();
    other_catalog.catalog.pieces = 374;
    let e = save::parse(&serde_json::to_string(&other_catalog).unwrap()).unwrap_err();
    assert!(
        e.contains("374") && e.contains(&CATALOG.len().to_string()),
        "should name both catalogues so a person can find the build: {e}"
    );

    for junk in ["", "{}", "null", "not json at all", "{\"format\":\"gm2d-save\"}"] {
        let e = save::parse(junk).unwrap_err();
        assert!(!e.is_empty(), "{junk:?} produced an empty message");
    }
}

/// A file that passes the envelope and is internally inconsistent is a corrupt
/// save, and is refused rather than half-loaded.
///
/// Half-loading is the dangerous outcome: a board missing one piece still
/// plays, and the player finds out three hours later.
#[test]
fn a_damaged_file_is_refused_rather_than_half_loaded() {
    let mut f = save::parse(&save::save(&a_played_game())).unwrap();
    f.state.character.owned.push(99_999);
    let e = f.into_game().unwrap_err();
    assert!(e.contains("damaged"), "should say the save is damaged: {e}");

    let mut f = save::parse(&save::save(&a_played_game())).unwrap();
    f.state.character.registry[0].def = "A Component From Another Game".into();
    let e = f.into_game().unwrap_err();
    assert!(
        e.contains("A Component From Another Game"),
        "should name the component it does not have: {e}"
    );
}

// ------------------------------------------------------------- the shape

/// The file on disk is the shape `PLAN.md` §4 promises.
///
/// Checked as text rather than through the types, because the schema is a
/// commitment to whoever writes the next reader — including a person opening
/// the file in an editor when something has gone wrong.
#[test]
fn the_file_has_the_documented_shape() {
    let text = save::save(&a_played_game());
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(v["format"], FORMAT);
    assert_eq!(v["version"], VERSION);
    assert!(v["catalog"]["fingerprint"].is_string());
    assert_eq!(v["catalog"]["pieces"], CATALOG.len());
    assert!(v["state"]["rng_state"].is_u64());
    assert_eq!(v["state"]["theme"], "td");

    let c = &v["state"]["character"];
    assert!(c["name_seed"].is_u64(), "the name seed is not in the file");
    assert!(c["locks"].as_array().is_some_and(|l| !l.is_empty()), "the locks are not in the file");
    assert!(c["registry"].as_array().is_some_and(|r| !r.is_empty()));

    // Components are named, never numbered. An index is stable only while
    // catalogue order is, and a save that outlives one insertion would hand
    // back a board of the wrong pieces with nothing to show for it.
    assert!(
        c["registry"][0]["def"].is_string(),
        "the registry stores catalogue indices, which is the bug this schema exists to avoid"
    );

    // Slots are words. A save is a thing a person opens in an editor when
    // something has gone wrong, and "weapon" tells them where they are.
    let boards = c["boards"].as_array().expect("boards");
    let names: Vec<&str> = boards.iter().map(|b| b[0].as_str().unwrap()).collect();
    assert!(names.contains(&"weapon") && names.contains(&"greaves"), "got {names:?}");
}

/// Board sizes are stored *and* checkable against the pieces on them.
///
/// Storing the row count makes the file readable; the assertion here is that
/// it is not the only record — a board whose stored height disagreed with the
/// pieces seated in it would be caught when they are placed.
#[test]
fn board_heights_survive_independently_of_each_other() {
    let before = a_played_game();
    let after = save::load(&save::save(&before)).unwrap();
    assert_eq!(before.character.slot_rows(), after.character.slot_rows());
    assert_ne!(
        before.character.slot_rows()[SlotKind::Weapon.index()],
        before.character.slot_rows()[SlotKind::Helmet.index()],
        "the fixture's boards are all the same height, so this proves nothing"
    );
}

// ------------------------------------------------------------- migration

/// A v1 save loads, through the migration path rather than around it.
///
/// The path has nothing to do yet. It is written and exercised now because the
/// moment a v2 exists is the worst possible moment to be designing the
/// mechanism that reaches it, and an untested migration is a migration that has
/// never run.
#[test]
fn a_v1_save_still_loads() {
    let text = save::save(&a_played_game());
    let f = save::parse(&text).unwrap();
    assert_eq!(f.version, 1);
    assert!(f.into_game().is_ok(), "the v1 arm of the migration does not work");
}

// ------------------------------------------------------------- the mirror

/// Adding a field to `Game` without adding it to the save is a **compile
/// error**, not a silent data loss.
///
/// This test asserts nothing at runtime and cannot. It is a note to whoever
/// arrives here after the compiler has stopped them, and the arrangement it
/// describes is the real guard:
///
/// `SaveFile::of` destructures `Game`, `Character` and `Loadout` exhaustively —
/// `let Game { rng, theme, character } = game;` — so a new field has no binding
/// and the function does not build. `into_game` destructures the save types the
/// same way, so a new *saved* field must also be read back.
///
/// Two fields are skipped on purpose and each says so at the point it is
/// skipped: `Loadout::naming`, a pointer into a theme's word tables that the
/// theme id restores, and `Character::undo_stack`, which is a session's history
/// of its own edits.
///
/// The failure this prevents is the one that would otherwise reach a player: a
/// field added in M4, forgotten here, and a level-5 character loading at level
/// one with every existing test still green.
#[test]
fn the_mirror_names_every_field() {
    let g = a_played_game();
    let f = SaveFile::of(&g);
    assert_eq!(f.state.theme, g.theme);
    assert_eq!(f.state.rng_state, g.rng.state());
    assert_eq!(f.state.character.gold, g.character.gold);
    assert_eq!(f.state.character.grown_health, g.character.grown_health);
    assert_eq!(f.state.character.name_seed, g.character.loadout.name_seed);
    assert_eq!(f.state.character.registry.len(), g.character.registry.count());
    assert_eq!(f.state.character.owned.len(), g.character.owned.len());
    assert_eq!(f.state.character.boards.len(), 5);
    assert_eq!(f.state.character.locks.len(), g.character.loadout.locks.len());
}

/// Rotation survives, which nothing else in this file would notice.
///
/// A rotated piece occupies different cells but the same count of them, so a
/// save that dropped rotations would come back with a board that is the right
/// size, holds the right pieces, and does not assemble.
#[test]
fn rotations_survive() {
    let mut g = Game::new(7, "td");
    g.character = Character::with_all_pieces();
    let id = PieceId(3);
    g.character.registry.set_rotation(id, 3);
    let after = save::load(&save::save(&g)).unwrap();
    assert_eq!(after.character.registry.rotation(id), 3, "a rotation was lost");
}

/// The theme id comes back, and with it the pointer the file could not carry.
#[test]
fn the_theme_pointer_is_restored_from_its_id() {
    let g = a_played_game();
    let after = save::load(&save::save(&g)).unwrap();
    assert_eq!(after.theme, "td");
    assert!(
        std::ptr::eq(after.character.loadout.naming, gm2d_core::theme::by_id("td").naming),
        "the loadout came back pointing at the wrong theme's words, so every item is renamed"
    );
}
