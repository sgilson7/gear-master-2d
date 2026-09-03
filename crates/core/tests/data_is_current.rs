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
    // **Every file, not four of them.** This was a list of four written out by
    // hand while `data.rs` embedded eleven, so a stale `shops.json`,
    // `quests.json`, `skills.json`, `supplies.json`, `enchs.json`,
    // a map file or `drops.json` would have shipped without a word. The list
    // lives in `data::FILES` now, beside the things it names — and a name in
    // it may have a directory in it, because the maps live in `data/maps/`.
    assert!(data::FILES.len() >= 13, "content was added to data.rs and not to FILES");
    for (name, embedded) in data::FILES.iter().copied() {
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
