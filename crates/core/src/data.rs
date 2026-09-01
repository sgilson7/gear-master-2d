//! The shipped data files, compiled in.
//!
//! There is no filesystem in a browser, so the content has to travel with the
//! binary. `include_str!` rather than a build script: the files are small, they
//! are checked in, and a build step that generates them would be a second place
//! for them to be wrong.
//!
//! The tests read the same files off disk instead, and
//! `tests/data_is_current.rs` asserts the two agree — so a data edit that never
//! reaches a rebuild is caught rather than shipped.

pub const TERRAIN_JSON: &str = include_str!("../../../data/terrain.json");
pub const TILES_JSON: &str = include_str!("../../../data/tiles.json");
pub const EVENTS_JSON: &str = include_str!("../../../data/events.json");
pub const THEME_TD_JSON: &str = include_str!("../../../data/theme.td.json");

/// The shipped map, loaded and checked.
///
/// Panics if the shipped data is broken, and that is correct: a build whose own
/// map does not load is a build that cannot start, and the tests in
/// `tests/world.rs` are what stop one being made.
pub fn world(difficulty: crate::combat::Difficulty) -> crate::world::World {
    crate::world::World::load(TERRAIN_JSON, TILES_JSON, difficulty)
        .expect("the shipped map is broken")
}

pub fn events() -> crate::tile_event::EventsData {
    crate::tile_event::EventsData::parse(EVENTS_JSON).expect("the shipped events are broken")
}
