//! Save files the walker can start a walk from.
//!
//! **Not fixtures and not plants.** `drive.py` plants a save and asserts about
//! the state it planted; `make play` starts a new game and plays it, and that
//! is what makes its transcript worth reading — it found an Auto-pack seating
//! the starting kit for the whole game and a class fork opening underneath the
//! town, and both were green in the suite.
//!
//! What it cannot do is get everywhere. M14 put eight floors behind the Drambus
//! Stack, and a walk that plateaus at level eleven on the tower's fourth floor
//! never sees one of them — **which is a fact about the walker and not about the
//! floors**, and `PLAN.md` §6d row 3 has been saying so since M11.9.
//!
//! So this writes a **start line**: a save standing where the block's own
//! content begins, built out of `common::geared_from`, which is this
//! repository's answer to *the board a player actually has* and is what every
//! reachability question has been asked of since M11.7. The walk from there is
//! the same walk.
//!
//! Ignored, because it writes files:
//!
//!     cargo test -p gm2d-core --test start_lines -- --ignored --nocapture

use gm2d_core::combat::Difficulty;
use gm2d_core::game::Game;
use gm2d_core::save::SaveFile;

const D: Difficulty = Difficulty::Easy;

mod common;

fn write(name: &str, g: &Game) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/saves");
    std::fs::create_dir_all(dir).expect("somewhere to put it");
    let path = format!("{dir}/{name}.json");
    std::fs::write(&path, SaveFile::of(g).to_json()).expect("write it");
    println!("wrote {path}");
}

/// **Standing at the lip of the Wextreen Sump**, with the tide out, an
/// instrument on the frame and the board a player actually has.
///
/// Everything it needs and nothing it does not: the Reach finished, so the tide
/// is out; the tower down and the lake beaten, because that is what a player
/// standing here has done and because the Silt Stair's own start line is the
/// same character one door along.
#[test]
#[ignore = "writes testing/saves/*.json"]
fn write_the_start_lines() {
    let mut g = Game::default();
    g.character = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    g.character.gold = 20_000;
    // A level the road actually reaches: the Low Water brackets 17-20 and the
    // Sump 19-23, so this is somebody who has earned the right to be here.
    g.character.gain_xp(20_000);

    // What a player standing here has done.
    for m in [
        "the-drambus-stack-5-boss",
        "the-drambus-stack-4-boss",
        "the-drambus-stack-3-boss",
        "the-drambus-stack-2-boss",
        "the-drambus-stack-1-boss",
        "the-bottom-of-the-lake",
        "the-bottom-of-the-cave",
        "the-door-in-the-wall",
        // The Reach, read and signed and the tenth cairn built — which is what
        // takes the tide out and is therefore what puts anybody on this tile.
        "the-trig-stone",
        "the-nine-surveys",
        "the-wextreen-reach",
    ] {
        g.world.answered.push(m.to_string());
    }
    for f in ["read-the-reach", "signed-the-trig", "built-the-tenth", "read-the-ninth"] {
        g.world.flags.push(f.to_string());
    }
    g.world.last_town = "kettleworks".into();

    // **An instrument, on its own frame.** A compass is a shard, a lens and a
    // magnet; the lip opens for any of the three and the compass is the one
    // whose reading the first floor is about.
    common::seat(
        &mut g.character,
        &[
            ("Map Shard", gm2d_core::piece::SlotKind::Instrument, 0, 0, 0),
            ("Glass Lens", gm2d_core::piece::SlotKind::Instrument, 2, 0, 0),
            ("Magnet", gm2d_core::piece::SlotKind::Instrument, 3, 0, 0),
        ],
    );
    assert_eq!(
        g.survey_kind().as_deref(),
        Some("compass"),
        "the start line does not carry an instrument, so the lip refuses it"
    );

    // At the lip, one tile below it.
    g.world.map = "the-treyway".into();
    g.world.at = [7, 22];
    let w = gm2d_core::data::map_now("the-treyway", D, &g.world);
    assert!(w.passable(7, 22), "the start line stands in the sea");
    assert!(w.passable(8, 15), "the tide has not gone out on this start line");
    write("at-the-lip", &g);

    // And the other one: at the door under the lake, which is the Silt Stair's
    // front step. Two hundred and six steps down and `no_homeward` from here.
    g.world.map = "under-the-lake".into();
    g.world.at = [6, 7];
    let u = gm2d_core::data::map_now("under-the-lake", D, &g.world);
    assert!(u.passable(6, 7), "the start line stands in the lake");
    write("under-the-lake", &g);
}

/// **The start lines still open, and they still stand where they say.**
///
/// Not ignored, and this is the whole reason the files are checked in rather
/// than built on demand: a save is the one artefact in this repository that
/// goes stale *silently* — the catalogue moves, the fingerprint moves, and the
/// file is refused by name with nobody looking. `check_the_frozen_save_is_
/// playable` is the same guard for the save a player sent in.
///
/// Negative-tested by moving the lip one tile: the save came back standing
/// somewhere that is not the door it is named for.
#[test]
fn the_start_lines_open_and_stand_where_they_say() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/saves");
    for (name, map, at) in [
        ("at-the-lip", "the-treyway", [7u8, 22u8]),
        ("under-the-lake", "under-the-lake", [6, 7]),
    ] {
        let text = std::fs::read_to_string(format!("{dir}/{name}.json")).unwrap_or_else(|e| {
            panic!("{name}.json: {e}\nRegenerate with: cargo test -p gm2d-core \
                    --test start_lines -- --ignored")
        });
        let g = gm2d_core::save::load(&text)
            .unwrap_or_else(|e| panic!("{name}.json will not open: {e}"));
        assert_eq!(g.world.map_id(), map, "{name} opens on the wrong map");
        assert_eq!(g.world.at, at, "{name} opens somewhere else");
        // **It stands where it says**, which is the half a fingerprint check
        // cannot give you: `World::repair` moves a save that lands in scenery,
        // silently and correctly, so a start line that has drifted into the sea
        // opens at a town and walks a different game.
        let w = gm2d_core::data::map_now(map, D, &g.world);
        assert!(
            w.passable(at[0], at[1]),
            "{name} stands on {:?}, which is not ground",
            w.terrain_name(at[0], at[1])
        );
        // And it carries what the walk needs: an instrument, or the lip refuses
        // it and the walk from that start line is a walk in a field.
        assert!(
            g.survey_kind().is_some(),
            "{name} carries no instrument, so the lip turns it away"
        );
    }
}
