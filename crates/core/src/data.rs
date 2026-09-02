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
pub const DUNGEON_JSON: &str = include_str!("../../../data/dungeon.json");
pub const EVENTS_JSON: &str = include_str!("../../../data/events.json");
pub const THEME_TD_JSON: &str = include_str!("../../../data/theme.td.json");
pub const SKILLS_JSON: &str = include_str!("../../../data/skills.json");
pub const SHOPS_JSON: &str = include_str!("../../../data/shops.json");
pub const QUESTS_JSON: &str = include_str!("../../../data/quests.json");
pub const SUPPLIES_JSON: &str = include_str!("../../../data/supplies.json");

/// The shipped map, loaded and checked.
///
/// Panics if the shipped data is broken, and that is correct: a build whose own
/// map does not load is a build that cannot start, and the tests in
/// `tests/world.rs` are what stop one being made.
pub fn world(difficulty: crate::combat::Difficulty) -> crate::world::World {
    map(&crate::world::overworld(), difficulty)
}

/// Every map this build ships, by id.
pub const MAPS: &[(&str, &str)] = &[
    ("west-bambulon", TILES_JSON),
    ("the-great-gear-cave", DUNGEON_JSON),
];

/// One map by id, falling back to the overworld.
///
/// **Falls back rather than panics.** A save can name a map this build does
/// not have — a file from a later version, or one whose dungeon was renamed —
/// and putting the player on the overworld is a recoverable answer where a
/// panic is not. `World::repair` then finds them somewhere to stand.
pub fn map(id: &str, difficulty: crate::combat::Difficulty) -> crate::world::World {
    let text = MAPS
        .iter()
        .find(|(k, _)| *k == id)
        .or_else(|| MAPS.first())
        .map(|(_, t)| *t)
        .expect("at least one map ships");
    crate::world::World::load(TERRAIN_JSON, text, difficulty).expect("a shipped map is broken")
}

pub fn events() -> crate::tile_event::EventsData {
    crate::tile_event::EventsData::parse(EVENTS_JSON).expect("the shipped events are broken")
}

/// The shipped skill tree.
///
/// Parsed on every call rather than cached in a `OnceLock`: it is read when a
/// screen opens or a node is bought, never in a loop, and a cache would be a
/// second place for it to be stale.
pub fn skills() -> crate::skills::SkillsData {
    crate::skills::SkillsData::parse(SKILLS_JSON).expect("the shipped skill tree is broken")
}

/// What the towns sell. Parsed on every call, like the tree and for the same
/// reason: it is read when a screen opens, never in a loop, and a cache would
/// be a second place for it to be stale.
pub fn shops() -> crate::shop::ShopsData {
    crate::shop::ShopsData::parse(SHOPS_JSON).expect("the shipped shelves are broken")
}

/// The errands the towns hand out.
pub fn quests() -> crate::quest::QuestsData {
    crate::quest::QuestsData::parse(QUESTS_JSON).expect("the shipped errands are broken")
}

/// What a town sells to take the tiredness off.
pub fn supplies() -> crate::fatigue::SuppliesData {
    crate::fatigue::SuppliesData::parse(SUPPLIES_JSON).expect("the shipped supplies are broken")
}
