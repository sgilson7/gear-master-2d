//! What is compiled in is what is on disk.
//!
//! `data.rs` embeds the content with `include_str!` because a browser has no
//! filesystem, and every other test reads the same files off disk. The two
//! could drift in exactly one direction — an edit that never reached a rebuild
//! — and the symptom would be a test suite passing against content the shipped
//! build has never seen.

use gm2d_core::data;

fn on_disk(name: &str) -> String {
    std::fs::read_to_string(format!("{}/../../data/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn the_compiled_in_data_matches_the_files() {
    for (name, embedded) in [
        ("terrain.json", data::TERRAIN_JSON),
        ("tiles.json", data::TILES_JSON),
        ("events.json", data::EVENTS_JSON),
        ("theme.td.json", data::THEME_TD_JSON),
    ] {
        assert_eq!(
            on_disk(name),
            embedded,
            "{name} on disk differs from the copy compiled in. Rebuild."
        );
    }
}

/// The shipped map loads through the same path the browser uses.
#[test]
fn the_shipped_world_loads() {
    let w = data::world(gm2d_core::combat::Difficulty::Easy);
    assert_eq!((w.width, w.height), (20, 20));
    assert!(!w.regions.is_empty());
    assert!(!data::events().events.is_empty());
}
