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

// **One file per map, under `data/maps/<id>.tiles.json`.** They were
// `tiles.json` and `dungeon.json` at the top of `data/`, which was two names
// for one kind of thing and no room for a third — and the third was going to
// be called `overworld.json` beside a `tiles.json` that *is* an overworld.
// The name of the file is the id of the map now, so a map that is misfiled is
// a map that will not load rather than a map that loads as somebody else.
pub const TILES_JSON: &str = include_str!("../../../data/maps/west-bambulon.tiles.json");
pub const DUNGEON_JSON: &str = include_str!("../../../data/maps/the-great-gear-cave.tiles.json");
/// The DQ-scale map behind the door in the western wall.
pub const TREYWAY_JSON: &str = include_str!("../../../data/maps/the-treyway.tiles.json");
/// The dense map: Kettleworks, the field, and the Drambus Stack in the middle.
pub const FIELD_JSON: &str = include_str!("../../../data/maps/kettleworks-field.tiles.json");

/// The Drambus Stack, top down. Five maps and one sitting each.
///
/// Listed in the order they are entered, which is also the order they come
/// down in — floor five first, because you always enter the current top and the
/// top is what comes off.
pub const STACK_5_JSON: &str = include_str!("../../../data/maps/the-drambus-stack-5.tiles.json");
pub const STACK_4_JSON: &str = include_str!("../../../data/maps/the-drambus-stack-4.tiles.json");
pub const STACK_3_JSON: &str = include_str!("../../../data/maps/the-drambus-stack-3.tiles.json");
pub const STACK_2_JSON: &str = include_str!("../../../data/maps/the-drambus-stack-2.tiles.json");
pub const STACK_1_JSON: &str = include_str!("../../../data/maps/the-drambus-stack-1.tiles.json");
/// What is under the lake, and the same map twice — see its own note.
pub const UNDER_LAKE_JSON: &str = include_str!("../../../data/maps/under-the-lake.tiles.json");
/// The surveyable map. Static, authored, and read through whatever you carried.
pub const REACH_JSON: &str = include_str!("../../../data/maps/the-reach.tiles.json");
/// The Treyway's south, over a bar of shingle the tide leaves.
///
/// **Its own file, and it was not.** The shore was drawn into the Treyway at
/// sixteen by twenty-six on the *one country, one file* principle, which is the
/// right instinct and was the wrong call: `#map` had been a fixed square in CSS
/// since the first map, so a grid half again as tall as it is wide came out
/// squashed. Reported from play as *"the resolution for the overworld looks all
/// messed up"*.
pub const LOW_WATER_JSON: &str = include_str!("../../../data/maps/the-low-water.tiles.json");
/// The flat past the low water, and **the second surveyable map**.
///
/// `survey::mods_for` has taken a `map` argument since M11.6 and never read
/// it — its own doc says a second one is meant to be *a data drop plus an arm
/// here*, and this is the data drop. There is iron under the sand, so the
/// instrument that reads the Reach best reads this worst: a compass is loud
/// here and an atlas is the quiet one. That is what makes *which instrument you
/// built* a question about where you are going.
pub const SANDS_JSON: &str = include_str!("../../../data/maps/the-wextreen-sands.tiles.json");
/// The Wextreen Sump, top down. Four floors, a puzzle each, and **not** one
/// sitting: the Stack's budget is fatigue and this one's is what you brought,
/// so every floor has a stair back up and none of them names an `outside`.
pub const SUMP_1_JSON: &str = include_str!("../../../data/maps/the-sump-1.tiles.json");
pub const SUMP_2_JSON: &str = include_str!("../../../data/maps/the-sump-2.tiles.json");
pub const SUMP_3_JSON: &str = include_str!("../../../data/maps/the-sump-3.tiles.json");
pub const SUMP_4_JSON: &str = include_str!("../../../data/maps/the-sump-4.tiles.json");
/// The Silt Stair, behind the door under the lake. Four floors, cut for people
/// carrying something, and `no_homeward` on every one of them for the lake's
/// own reason one map further down.
pub const STAIR_1_JSON: &str = include_str!("../../../data/maps/the-silt-stair-1.tiles.json");
pub const STAIR_2_JSON: &str = include_str!("../../../data/maps/the-silt-stair-2.tiles.json");
pub const STAIR_3_JSON: &str = include_str!("../../../data/maps/the-silt-stair-3.tiles.json");
pub const STAIR_4_JSON: &str = include_str!("../../../data/maps/the-silt-stair-4.tiles.json");
/// The country under the country. One town on it and the town is empty.
pub const UNDERCOUNTRY_JSON: &str =
    include_str!("../../../data/maps/the-undercountry.tiles.json");
/// The Eleven Reefs: three floors under the Wextreen Sands.
pub const REEFS_1_JSON: &str = include_str!("../../../data/maps/the-reefs-1.tiles.json");
pub const REEFS_2_JSON: &str = include_str!("../../../data/maps/the-reefs-2.tiles.json");
pub const REEFS_3_JSON: &str = include_str!("../../../data/maps/the-reefs-3.tiles.json");
pub const CAIRNWORKS_1_JSON: &str = include_str!("../../../data/maps/the-cairnworks-1.tiles.json");
pub const CAIRNWORKS_2_JSON: &str = include_str!("../../../data/maps/the-cairnworks-2.tiles.json");
pub const CAIRNWORKS_3_JSON: &str = include_str!("../../../data/maps/the-cairnworks-3.tiles.json");
pub const CAIRNWORKS_4_JSON: &str = include_str!("../../../data/maps/the-cairnworks-4.tiles.json");
pub const EVENTS_JSON: &str = include_str!("../../../data/events.json");
pub const THEME_TD_JSON: &str = include_str!("../../../data/theme.td.json");
pub const SKILLS_JSON: &str = include_str!("../../../data/skills.json");
pub const SHOPS_JSON: &str = include_str!("../../../data/shops.json");
pub const QUESTS_JSON: &str = include_str!("../../../data/quests.json");
pub const SUPPLIES_JSON: &str = include_str!("../../../data/supplies.json");
pub const BREWS_JSON: &str = include_str!("../../../data/brews.json");
/// The art manifest, which is also the one place a creature's **family** is
/// written down.
///
/// **Read here rather than copied into a second list.** `make art` compiles a
/// figure per creature out of this and rewrites the creature half of
/// `data/art.json` from it, so it is already the single source for *which
/// silhouette is this creature cut from* — and M20 needs the same answer for
/// *which ingredient does it leave*. A list of seventy-five creature names in
/// `brews.json` would be the seventh hand-written list this project has paid
/// for.
pub const CREATURE_ART_JSON: &str = include_str!("../../../art/creatures.json");
pub const ENCHS_JSON: &str = include_str!("../../../data/enchs.json");
pub const DROPS_JSON: &str = include_str!("../../../data/drops.json");

/// Every content file this build compiled in, by name.
///
/// A list, because `tests/data_is_current.rs` walked four of the eleven and had
/// no way to know it. The drift it catches goes in exactly one direction — an
/// edit that never reached a rebuild — and a partial list catches it for a
/// quarter of the content. Adding a file here is the second half of adding one
/// above, and the test is what says so.
pub const FILES: &[(&str, &str)] = &[
    ("terrain.json", TERRAIN_JSON),
    ("maps/west-bambulon.tiles.json", TILES_JSON),
    ("maps/the-great-gear-cave.tiles.json", DUNGEON_JSON),
    ("maps/the-treyway.tiles.json", TREYWAY_JSON),
    ("maps/kettleworks-field.tiles.json", FIELD_JSON),
    ("maps/the-drambus-stack-5.tiles.json", STACK_5_JSON),
    ("maps/the-drambus-stack-4.tiles.json", STACK_4_JSON),
    ("maps/the-drambus-stack-3.tiles.json", STACK_3_JSON),
    ("maps/the-drambus-stack-2.tiles.json", STACK_2_JSON),
    ("maps/the-drambus-stack-1.tiles.json", STACK_1_JSON),
    ("maps/under-the-lake.tiles.json", UNDER_LAKE_JSON),
    ("maps/the-reach.tiles.json", REACH_JSON),
    ("maps/the-low-water.tiles.json", LOW_WATER_JSON),
    ("maps/the-wextreen-sands.tiles.json", SANDS_JSON),
    ("maps/the-sump-1.tiles.json", SUMP_1_JSON),
    ("maps/the-sump-2.tiles.json", SUMP_2_JSON),
    ("maps/the-sump-3.tiles.json", SUMP_3_JSON),
    ("maps/the-sump-4.tiles.json", SUMP_4_JSON),
    ("maps/the-silt-stair-1.tiles.json", STAIR_1_JSON),
    ("maps/the-silt-stair-2.tiles.json", STAIR_2_JSON),
    ("maps/the-silt-stair-3.tiles.json", STAIR_3_JSON),
    ("maps/the-silt-stair-4.tiles.json", STAIR_4_JSON),
    ("maps/the-undercountry.tiles.json", UNDERCOUNTRY_JSON),
    ("maps/the-reefs-1.tiles.json", REEFS_1_JSON),
    ("maps/the-reefs-2.tiles.json", REEFS_2_JSON),
    ("maps/the-reefs-3.tiles.json", REEFS_3_JSON),
    ("maps/the-cairnworks-1.tiles.json", CAIRNWORKS_1_JSON),
    ("maps/the-cairnworks-2.tiles.json", CAIRNWORKS_2_JSON),
    ("maps/the-cairnworks-3.tiles.json", CAIRNWORKS_3_JSON),
    ("maps/the-cairnworks-4.tiles.json", CAIRNWORKS_4_JSON),
    ("events.json", EVENTS_JSON),
    ("theme.td.json", THEME_TD_JSON),
    ("skills.json", SKILLS_JSON),
    ("shops.json", SHOPS_JSON),
    ("quests.json", QUESTS_JSON),
    ("supplies.json", SUPPLIES_JSON),
    ("brews.json", BREWS_JSON),
    ("enchs.json", ENCHS_JSON),
    ("drops.json", DROPS_JSON),
];

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
    ("the-treyway", TREYWAY_JSON),
    ("kettleworks-field", FIELD_JSON),
    ("the-drambus-stack-5", STACK_5_JSON),
    ("the-drambus-stack-4", STACK_4_JSON),
    ("the-drambus-stack-3", STACK_3_JSON),
    ("the-drambus-stack-2", STACK_2_JSON),
    ("the-drambus-stack-1", STACK_1_JSON),
    ("under-the-lake", UNDER_LAKE_JSON),
    ("the-reach", REACH_JSON),
    ("the-low-water", LOW_WATER_JSON),
    ("the-wextreen-sands", SANDS_JSON),
    ("the-sump-1", SUMP_1_JSON),
    ("the-sump-2", SUMP_2_JSON),
    ("the-sump-3", SUMP_3_JSON),
    ("the-sump-4", SUMP_4_JSON),
    ("the-silt-stair-1", STAIR_1_JSON),
    ("the-silt-stair-2", STAIR_2_JSON),
    ("the-silt-stair-3", STAIR_3_JSON),
    ("the-silt-stair-4", STAIR_4_JSON),
    ("the-undercountry", UNDERCOUNTRY_JSON),
    ("the-reefs-1", REEFS_1_JSON),
    ("the-reefs-2", REEFS_2_JSON),
    ("the-reefs-3", REEFS_3_JSON),
    ("the-cairnworks-1", CAIRNWORKS_1_JSON),
    ("the-cairnworks-2", CAIRNWORKS_2_JSON),
    ("the-cairnworks-3", CAIRNWORKS_3_JSON),
    ("the-cairnworks-4", CAIRNWORKS_4_JSON),
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

/// One map, as the game has left it.
///
/// The same map `map` returns with everything the state says has drained,
/// drained. **Every question a *game* asks goes through this**; `map` is for
/// questions about the file, and a lint that only ever saw the drained lake
/// could not see the undrained one.
pub fn map_now(
    id: &str,
    difficulty: crate::combat::Difficulty,
    state: &crate::world::WorldState,
) -> crate::world::World {
    map_read_through(id, difficulty, state, 0)
}

/// The same, through whatever instrument is on the board.
///
/// **The board's count is the caller's** — `world.rs` may not go and read a
/// character, which is the same division `Allowances` makes and for the same
/// reason. A caller with no character passes zero, which is a compass with no
/// gear behind it and is what `map_now` does.
pub fn map_read_through(
    id: &str,
    difficulty: crate::combat::Difficulty,
    state: &crate::world::WorldState,
    items_assembled: usize,
) -> crate::world::World {
    let mut w = map(id, difficulty);
    w.drain(state);
    if let Some((surveyed, kind)) = &state.active_survey {
        if surveyed == &w.id {
            w.survey = crate::survey::mods_for(&w.id, kind, items_assembled);
        }
    }
    w
}

/// Every map this build ships, loaded.
///
/// For the questions that are about the world rather than about the tile you
/// are standing on — where a creature lives, which towns are placed. A guide
/// that only looked at the map underfoot would tell a player in a dungeon that
/// their errand points nowhere.
pub fn all_maps(difficulty: crate::combat::Difficulty) -> Vec<crate::world::World> {
    MAPS.iter().map(|(id, _)| map(id, difficulty)).collect()
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

/// The ids of every town a player can actually walk into.
///
/// **`shops.json` holds shelves and `data/maps/*` holds ground, and the two do
/// not have to agree.** A shelf with no map under it is *staged* — content
/// waiting for a map, which this project allows and names — and `avail.rs`'s
/// `STAGED` list is where that is written down.
///
/// What it is for: the barrel and the order book both refuse to stock anything
/// a town already has on its shelf, so that the cheap tier cannot undercut the
/// authored one. **A shelf nobody can reach undercuts nothing**, and High Wick
/// is the arcane shelf — the only one in the game that sells a book, an ink and
/// three spells — sitting on a map that does not exist. It was taking three of
/// the five barrel-priced spells in the catalogue out of the barrel's reach on
/// behalf of a counter no player has ever stood at.
///
/// Parsed on every call, like everything else in this file and for the same
/// reason: a cache is a second place for it to be stale. The places are read
/// straight off the tiles rather than through `World::load`, because *which
/// towns exist* is a question about the file and building eleven worlds to
/// answer it would be building eleven worlds.
pub fn towns_on_the_map() -> Vec<String> {
    #[derive(serde::Deserialize)]
    struct JustPlaces {
        #[serde(default)]
        places: Vec<JustPlace>,
    }
    #[derive(serde::Deserialize)]
    struct JustPlace {
        id: String,
        kind: String,
    }
    let mut out: Vec<String> = Vec::new();
    for (_, text) in MAPS {
        let Ok(f) = serde_json::from_str::<JustPlaces>(text) else { continue };
        for p in f.places {
            if p.kind == "town" && !out.contains(&p.id) {
                out.push(p.id);
            }
        }
    }
    out
}

/// The errands the towns hand out.
pub fn quests() -> crate::quest::QuestsData {
    crate::quest::QuestsData::parse(QUESTS_JSON).expect("the shipped errands are broken")
}

/// What a town sells to take the tiredness off.
pub fn supplies() -> crate::fatigue::SuppliesData {
    crate::fatigue::SuppliesData::parse(SUPPLIES_JSON).expect("the shipped supplies are broken")
}

/// Which family drawing each creature is cut from.
///
/// Keys beginning `_` are the manifest's own notes and are not creatures.
pub fn art_families() -> std::collections::BTreeMap<String, String> {
    #[derive(serde::Deserialize)]
    struct Row {
        family: String,
    }
    let raw: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_str(CREATURE_ART_JSON).expect("the art manifest is broken");
    raw.into_iter()
        .filter(|(k, _)| !k.starts_with('_'))
        .filter_map(|(k, v)| serde_json::from_value::<Row>(v).ok().map(|r| (k, r.family)))
        .collect()
}

/// The larder and the bench.
pub fn brews() -> crate::brew::BrewsData {
    crate::brew::BrewsData::parse(BREWS_JSON).expect("the shipped brews are broken")
}

/// What a licensee can bolt to a component.
pub fn enchs() -> crate::ench::EnchsData {
    crate::ench::EnchsData::parse(ENCHS_JSON).expect("the shipped enchs are broken")
}

/// What a creature leaves behind, and how often.
pub fn drops() -> crate::drops::DropsData {
    crate::drops::DropsData::parse(DROPS_JSON).expect("the shipped drops are broken")
}
