//! The map, and what happens when you step on it.
//!
//! # Derived, never stored
//!
//! The discipline is borrowed wholesale from upstream's `county.rs`, whose own
//! header states it best: the map *"is derived, never stored"* — the run keeps
//! only where you are standing and what you have cleared. Upstream generated
//! its county from a seed; GM2D authors its map in `data/tiles.json` so the
//! difficulty gradient can be designed rather than rolled. The split is the
//! same either way, and it is what keeps a save small and robust: the grid is
//! content, and content is not state.
//!
//! So [`World`] is loaded once and never changes, and [`WorldState`] — which
//! is the part that goes in the save — holds a position, a set of answered
//! events, and nothing else that a map could tell you.
//!
//! # One stream, and it is integers
//!
//! Every roll here comes from `Game::rng`. There is no second stream, because
//! two streams is how a replay stops replaying.
//!
//! And every roll is **integer arithmetic in per-mille**. A seeded walk has to
//! produce the same encounters on every machine and in every browser, and float
//! rounding is the one thing that can break that silently — the fault would
//! surface as a save that replays correctly for the person who wrote it and
//! differently for the person they sent it to.
//!
//! # Danger is measured, not typed
//!
//! A region's danger is the mean of `rating::creature_rating` over its enemy
//! pool. No data file contains a difficulty number, and
//! `tests/world.rs::no_data_file_types_a_danger_number` fails the build if one
//! ever does. Tuning the map means moving creatures between pools, which is a
//! statement about what lives there; typing a number would be tuning the ruler.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::combat::{Difficulty, MonsterSpec};
use crate::rng::Rng;

/// Divisor turning a region's danger into a multiplier on the terrain's base
/// encounter rate.
///
/// At `danger == DANGER_REF` a tile is twice as likely to start a fight as the
/// same terrain in a region of harmless creatures. Set where it is because the
/// ladder's ratings run from 16 to about 3000, so this puts the early regions
/// near 1× and the late ones near 4×, which is the spread the terrain table
/// was written against.
pub const DANGER_REF: i32 = 800;

/// The most likely any single step is allowed to be, in per-mille.
///
/// A cap rather than a formula that happens not to exceed it: without one, a
/// late region on slow terrain would roll a fight on nearly every step and the
/// map would stop being a place and start being a corridor.
pub const MAX_ENCOUNTER_PER_MILLE: i32 = 450;

// ------------------------------------------------------------------ the data

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainDef {
    /// The character this terrain is drawn as in `tiles.json`'s rows.
    pub glyph: char,
    pub passable: bool,
    /// Steps' worth of time. Not spent yet — M4's grind will want it.
    #[serde(default = "one")]
    pub cost: u8,
    /// Base chance of an encounter on entering, in per-mille, before the
    /// region's danger multiplies it.
    pub encounter_per_mille: i32,
}

fn one() -> u8 {
    1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainData {
    pub format: String,
    pub version: u32,
    pub terrain: BTreeMap<String, TerrainDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionDef {
    pub id: String,
    pub name: String,
    /// Inclusive box, `[x0, y0, x1, y1]`. Boxes rather than tile lists because
    /// a region is a part of the map a person can point at, and a list of two
    /// hundred coordinates is not.
    pub bounds: Vec<[u8; 4]>,
    /// Canonical creature names. **No danger number**: see the module header.
    pub enemies: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaceKind {
    Town,
    Event,
    /// A way onto another map.
    ///
    /// The first one is the gate Henpeck put on the Great Gear Cave. A gate
    /// may want something in your hands before it opens, which is what makes a
    /// questline a door rather than a receipt.
    Gate,
    /// A creature that is standing here, rather than one the ground rolled.
    ///
    /// The only fights in the game that are not a draw against a region's
    /// pool. It is at the end of a dungeon because that is what a dungeon is:
    /// a corridor with something certain at the end of it.
    Boss,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceDef {
    pub at: [u8; 2],
    pub kind: PlaceKind,
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// `Gate`: the map it opens onto, and where you arrive on it.
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub at_to: Option<[u8; 2]>,
    /// `Gate`: the canonical component you must be carrying for it to open.
    #[serde(default)]
    pub needs: Option<String>,
    /// `Gate`: what to say when you are not.
    #[serde(default)]
    pub shut: String,
    /// `Boss`: the creature standing here, by canonical name.
    #[serde(default)]
    pub creature: Option<String>,
    /// `Boss`: the canonical component beating it leaves behind.
    #[serde(default)]
    pub drops: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilesData {
    pub format: String,
    pub version: u32,
    /// Which map this is. Defaulted so the overworld file, which was the only
    /// map when it was written, does not have to say so.
    #[serde(default = "overworld")]
    pub id: String,
    pub width: u8,
    pub height: u8,
    pub start: [u8; 2],
    /// One string a row, one glyph a tile. The map is authored as a picture
    /// because that is what it is.
    pub rows: Vec<String>,
    pub regions: Vec<RegionDef>,
    pub places: Vec<PlaceDef>,
}

// ------------------------------------------------------------------ resolved

/// A region with its danger worked out.
#[derive(Clone, Debug)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub enemies: Vec<&'static MonsterSpec>,
    /// Mean `creature_rating` over the pool. Measured at load.
    pub danger: i32,
}

/// The map, loaded and checked.
#[derive(Clone, Debug)]
pub struct World {
    /// Which map this is. The overworld is `"west-bambulon"`; a dungeon is its
    /// own, and `WorldState::map` says which one the player is standing on.
    pub id: String,
    pub width: u8,
    pub height: u8,
    pub start: (u8, u8),
    terrain: Vec<(String, TerrainDef)>,
    /// Index into `terrain`, one per tile, row-major.
    tiles: Vec<usize>,
    /// Index into `regions`, one per tile. Every tile belongs to exactly one.
    region_of: Vec<usize>,
    pub regions: Vec<Region>,
    pub places: Vec<PlaceDef>,
}

/// Why a map would not load. Every one is a sentence naming the tile.
pub type WorldError = String;

/// The map everything starts on.
pub fn overworld() -> String {
    "west-bambulon".to_string()
}

impl WorldState {
    /// Which map this state is on, with the empty default resolved.
    pub fn map_id(&self) -> String {
        if self.map.is_empty() {
            overworld()
        } else {
            self.map.clone()
        }
    }
}

impl World {
    /// Load a map from its two data files, checking everything a later stage
    /// would otherwise discover by crashing.
    pub fn load(
        terrain_json: &str,
        tiles_json: &str,
        difficulty: Difficulty,
    ) -> Result<Self, WorldError> {
        let td: TerrainData = serde_json::from_str(terrain_json)
            .map_err(|e| format!("terrain.json will not parse: {e}"))?;
        if td.format != "gm2d-terrain" {
            return Err(format!("expected a gm2d-terrain file, got {:?}", td.format));
        }
        let tl: TilesData = serde_json::from_str(tiles_json)
            .map_err(|e| format!("tiles.json will not parse: {e}"))?;
        if tl.format != "gm2d-tiles" {
            return Err(format!("expected a gm2d-tiles file, got {:?}", tl.format));
        }

        let terrain: Vec<(String, TerrainDef)> = td.terrain.into_iter().collect();
        let by_glyph: BTreeMap<char, usize> =
            terrain.iter().enumerate().map(|(i, (_, t))| (t.glyph, i)).collect();
        if by_glyph.len() != terrain.len() {
            return Err("two terrains share a glyph, so the map is ambiguous".into());
        }

        if tl.rows.len() != tl.height as usize {
            return Err(format!(
                "the map says it is {} rows tall and has {}",
                tl.height,
                tl.rows.len()
            ));
        }
        let mut tiles = Vec::with_capacity(tl.width as usize * tl.height as usize);
        for (y, row) in tl.rows.iter().enumerate() {
            let cells: Vec<char> = row.chars().collect();
            if cells.len() != tl.width as usize {
                return Err(format!(
                    "row {y} is {} tiles wide and the map says {}",
                    cells.len(),
                    tl.width
                ));
            }
            for (x, c) in cells.into_iter().enumerate() {
                let i = *by_glyph
                    .get(&c)
                    .ok_or_else(|| format!("no terrain is drawn {c:?} (row {y}, column {x})"))?;
                tiles.push(i);
            }
        }

        // Regions, with danger measured off the pool.
        let mut regions = Vec::new();
        for r in &tl.regions {
            let mut enemies = Vec::new();
            for name in &r.enemies {
                let spec = crate::combat::creature(name)
                    .ok_or_else(|| format!("region {:?} names no such creature: {name:?}", r.id))?;
                enemies.push(spec);
            }
            if enemies.is_empty() {
                return Err(format!("region {:?} has an empty enemy pool", r.id));
            }
            let danger = enemies
                .iter()
                .map(|m| crate::rating::creature_rating(m, difficulty))
                .sum::<i32>()
                / enemies.len() as i32;
            regions.push(Region {
                id: r.id.clone(),
                name: r.name.clone(),
                enemies,
                danger,
            });
        }

        // Every tile in exactly one region. Checked rather than defaulted: a
        // tile with no region is a tile whose encounters have no pool to draw
        // from, and defaulting it to the first region would hide the hole.
        let n = tiles.len();
        let mut region_of = vec![usize::MAX; n];
        for (ri, r) in tl.regions.iter().enumerate() {
            for &[x0, y0, x1, y1] in &r.bounds {
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if x >= tl.width || y >= tl.height {
                            return Err(format!(
                                "region {:?} covers ({x}, {y}), which is off the map",
                                r.id
                            ));
                        }
                        let i = y as usize * tl.width as usize + x as usize;
                        if region_of[i] != usize::MAX {
                            return Err(format!(
                                "({x}, {y}) is in two regions: {:?} and {:?}",
                                tl.regions[region_of[i]].id, r.id
                            ));
                        }
                        region_of[i] = ri;
                    }
                }
            }
        }

        let world = World {
            id: tl.id.clone(),
            width: tl.width,
            height: tl.height,
            start: (tl.start[0], tl.start[1]),
            terrain,
            tiles,
            region_of,
            regions,
            places: tl.places,
        };

        for y in 0..world.height {
            for x in 0..world.width {
                if world.passable(x, y) && world.region_index(x, y).is_none() {
                    return Err(format!("({x}, {y}) is walkable and in no region"));
                }
            }
        }
        for p in &world.places {
            let (x, y) = (p.at[0], p.at[1]);
            if !world.in_bounds(x as i32, y as i32) {
                return Err(format!("{:?} is placed at ({x}, {y}), off the map", p.id));
            }
            if !world.passable(x, y) {
                return Err(format!("{:?} is placed on impassable ground at ({x}, {y})", p.id));
            }
        }
        if !world.passable(world.start.0, world.start.1) {
            return Err("the starting tile is impassable".into());
        }
        Ok(world)
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    fn idx(&self, x: u8, y: u8) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn terrain_at(&self, x: u8, y: u8) -> &TerrainDef {
        &self.terrain[self.tiles[self.idx(x, y)]].1
    }

    pub fn terrain_name(&self, x: u8, y: u8) -> &str {
        &self.terrain[self.tiles[self.idx(x, y)]].0
    }

    pub fn passable(&self, x: u8, y: u8) -> bool {
        self.terrain_at(x, y).passable
    }

    fn region_index(&self, x: u8, y: u8) -> Option<usize> {
        let i = self.region_of[self.idx(x, y)];
        (i != usize::MAX).then_some(i)
    }

    pub fn region_at(&self, x: u8, y: u8) -> Option<&Region> {
        self.region_index(x, y).map(|i| &self.regions[i])
    }

    pub fn place_at(&self, x: u8, y: u8) -> Option<&PlaceDef> {
        self.places.iter().find(|p| p.at == [x, y])
    }

    /// The regions whose pool holds this creature.
    ///
    /// The answer to "where do I find a Whisperling", which is a question about
    /// the map's pools and therefore the map's to answer. A page working it out
    /// for itself would be a second copy of "what lives where", and the pools
    /// are the one thing on this map that gets retuned.
    pub fn regions_holding(&self, creature: &str) -> Vec<&Region> {
        self.regions
            .iter()
            .filter(|r| r.enemies.iter().any(|m| m.name == creature))
            .collect()
    }

    /// Every tile of a region, as coordinates. Empty for a region this map
    /// does not have.
    pub fn tiles_of(&self, region: &str) -> Vec<[u8; 2]> {
        let Some(i) = self.regions.iter().position(|r| r.id == region) else { return Vec::new() };
        let mut out = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.region_index(x, y) == Some(i) && self.passable(x, y) {
                    out.push([x, y]);
                }
            }
        }
        out
    }

    /// The chance of an encounter on entering this tile, in per-mille.
    ///
    /// `terrain.base × (1 + danger/DANGER_REF)`, capped, in integers
    /// throughout. Monotone in danger, which is the property the brief asks
    /// for: a region of harder creatures is a region you get stopped in more
    /// often, and there is no arrangement of the table where it is not.
    pub fn encounter_per_mille(&self, x: u8, y: u8) -> i32 {
        let base = self.terrain_at(x, y).encounter_per_mille;
        if base <= 0 {
            return 0;
        }
        let danger = self.region_at(x, y).map(|r| r.danger).unwrap_or(0);
        let scaled = base as i64 * (100 + (danger as i64 * 100 / DANGER_REF as i64)) / 100;
        scaled.clamp(0, MAX_ENCOUNTER_PER_MILLE as i64) as i32
    }

    /// Put a player back somewhere they can stand.
    ///
    /// A save should never place you where you cannot be, and this is what
    /// makes that true regardless of why it happened. Returns `Some` naming
    /// where they were moved from, or `None` if they were already fine.
    ///
    /// **This exists because they were not.** `WorldState::world` is
    /// `#[serde(default)]` so that saves written before M2 still open — and a
    /// default `WorldState` stands at `(0, 0)`, which on this map is rock in
    /// the top-left corner. A player carrying an autosave from an older build
    /// arrived wedged in it, unable to move in any direction, with no way out
    /// but a new game. Anything that loads a position runs this.
    ///
    /// The order is deliberate: the last town first, because that is where the
    /// player *was* in any sense that survives, and the map's start only if
    /// that fails too.
    pub fn repair(&self, state: &mut WorldState) -> Option<[u8; 2]> {
        let [x, y] = state.at;
        if self.in_bounds(x as i32, y as i32) && self.passable(x, y) {
            return None;
        }
        let was = state.at;
        let home = self
            .places
            .iter()
            .find(|p| p.id == state.last_town && self.passable(p.at[0], p.at[1]))
            .map(|p| p.at)
            .unwrap_or([self.start.0, self.start.1]);
        state.at = home;
        Some(was)
    }

    /// Draw an opponent from this tile's region.
    ///
    /// Weighted by `(max + 1 − rating)`, so the hardest creature in a pool is
    /// the rarest one in it and every region stays winnable while still being
    /// able to frighten you. The draw is one `below` call so a replay's stream
    /// advances by a fixed amount however the pool is shaped.
    pub fn draw_enemy(&self, x: u8, y: u8, difficulty: Difficulty, rng: &mut Rng) -> Option<&'static MonsterSpec> {
        let region = self.region_at(x, y)?;
        let rated: Vec<(i32, &'static MonsterSpec)> = region
            .enemies
            .iter()
            .map(|&m| (crate::rating::creature_rating(m, difficulty), m))
            .collect();
        let max = rated.iter().map(|(r, _)| *r).max().unwrap_or(0);
        let weights: Vec<i32> = rated.iter().map(|(r, _)| (max + 1 - r).max(1)).collect();
        let total: i32 = weights.iter().sum();
        let mut pick = rng.below(total.max(1) as usize) as i32;
        for (w, (_, m)) in weights.iter().zip(rated.iter()) {
            pick -= w;
            if pick < 0 {
                return Some(m);
            }
        }
        rated.last().map(|(_, m)| *m)
    }
}

// ------------------------------------------------------------------ state

/// The part of the world that goes in the save.
///
/// Position, what has been answered, and flags. **Not the map.**
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    /// Which map you are standing on.
    ///
    /// `default` is the empty string and means the overworld, so a file
    /// written before there was a second map opens where it left off.
    #[serde(default)]
    pub map: String,
    pub at: [u8; 2],
    /// The town to return to after a loss. Empty until one is reached.
    #[serde(default)]
    pub last_town: String,
    /// Event ids already answered. A tile-bound event fires once.
    #[serde(default)]
    pub answered: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub counters: Vec<(String, u32)>,
    /// What has been bought off a town's shelf: the place id and the index in
    /// that town's stock list.
    ///
    /// The shelf itself is `data/shops.json` and is not here, for the reason
    /// the map is not here. The *index* is the identity, which is why a sold
    /// entry is greyed rather than dropped — renumbering would move what
    /// somebody already bought.
    #[serde(default)]
    pub bought: Vec<(String, u16)>,
    /// Errands taken and not yet handed in.
    #[serde(default)]
    pub quests_taken: Vec<String>,
    /// Errands handed in. A town does not offer one twice.
    #[serde(default)]
    pub quests_done: Vec<String>,
    /// The errand the map is currently pointing at, if one is pinned.
    ///
    /// **State, because a pin that died with the screen would be a reference
    /// rather than a tool.** You pin an errand in the log, close the log, and
    /// walk — the whole value is in the walking, and it is the only part the
    /// log itself cannot do.
    #[serde(default)]
    pub pinned: Option<String>,
}

impl WorldState {
    pub fn at_start(world: &World) -> Self {
        WorldState {
            at: [world.start.0, world.start.1],
            last_town: world
                .place_at(world.start.0, world.start.1)
                .filter(|p| p.kind == PlaceKind::Town)
                .map(|p| p.id.clone())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn bump(&mut self, what: &str) {
        self.add(what, 1);
    }

    /// Add to a counter, creating it if it is not there.
    pub fn add(&mut self, what: &str, by: u32) {
        match self.counters.iter_mut().find(|(k, _)| k == what) {
            Some((_, n)) => *n += by,
            None => self.counters.push((what.to_string(), by)),
        }
    }

    pub fn count(&self, what: &str) -> u32 {
        self.counters.iter().find(|(k, _)| k == what).map(|(_, n)| *n).unwrap_or(0)
    }
}

/// Which way a step went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    North,
    South,
    East,
    West,
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::North => (0, -1),
            Dir::South => (0, 1),
            Dir::East => (1, 0),
            Dir::West => (-1, 0),
        }
    }
}

/// What one step produced.
///
/// `PartialEq` is hand-written: `MonsterSpec` does not implement it, and two
/// steps that met the same creature met the same creature — comparing the
/// specs by name is both what a test means by it and the only thing available.
#[derive(Clone, Debug)]
pub struct Step {
    pub moved: bool,
    /// Why the step was refused, if it was.
    pub blocked: Option<String>,
    /// An event standing on the tile that has not been answered yet.
    pub event: Option<String>,
    /// Whether that event's choices have already been answered.
    ///
    /// **The card always opens; only the choices are spent.** An event's prose
    /// and its doors are a one-time thing, which is right — but an errand board
    /// is a standing feature of a place, the same as a town's. Refusing to
    /// reopen the card made Marbulon's tile inert the moment you spoke to her,
    /// and her two errands are the questline that unlocks the cave.
    pub spent: bool,
    /// A town on the tile.
    pub town: Option<String>,
    /// A gate you are now standing on. Whether it opens is the caller's
    /// question, because it depends on what is in the bag.
    pub gate: Option<String>,
    /// A creature standing here rather than one the ground rolled.
    pub boss: Option<String>,
    /// A fight rolled on entering.
    pub encounter: Option<&'static MonsterSpec>,
}

impl PartialEq for Step {
    fn eq(&self, other: &Self) -> bool {
        self.moved == other.moved
            && self.blocked == other.blocked
            && self.event == other.event
            && self.town == other.town
            && self.encounter.map(|m| m.name) == other.encounter.map(|m| m.name)
    }
}

impl Eq for Step {}

impl Step {
    /// The name of whatever this step ran into, for a transcript.
    pub fn met(&self) -> Option<&'static str> {
        self.encounter.map(|m| m.name)
    }

    fn nowhere(why: &str) -> Self {
        Step {
            moved: false,
            blocked: Some(why.to_string()),
            event: None,
            spent: false,
            town: None,
            gate: None,
            boss: None,
            encounter: None,
        }
    }
}

/// Take one step, rolling for an encounter on arrival.
///
/// **Order matters and is fixed:** the move, then the place, then the roll. A
/// tile with an event on it does not also roll a fight, because arriving
/// somewhere and being ambushed on the way in is two things happening in one
/// keypress and the player can only answer one of them. Towns never roll —
/// that is what makes a town a place to stand.
///
/// The roll happens **once per entered tile**, so a walk's encounter sequence
/// is a function of the path and the stream, and nothing else. A blocked step
/// draws nothing at all: bumping into a wall must not advance the stream, or
/// two players walking the same route would see different fights depending on
/// how often they misjudged a cliff.
pub fn step(
    world: &World,
    state: &mut WorldState,
    rng: &mut Rng,
    difficulty: Difficulty,
    dir: Dir,
) -> Step {
    let (dx, dy) = dir.delta();
    let (nx, ny) = (state.at[0] as i32 + dx, state.at[1] as i32 + dy);
    if !world.in_bounds(nx, ny) {
        return Step::nowhere("the map ends here");
    }
    let (nx, ny) = (nx as u8, ny as u8);
    if !world.passable(nx, ny) {
        return Step::nowhere(match world.terrain_name(nx, ny) {
            "water" => "you would have to swim, and you are wearing a frame",
            "rock" => "rock, and no way up it",
            _ => "there is no way through",
        });
    }

    state.at = [nx, ny];
    state.bump("tiles-walked");

    let mut out = Step {
        moved: true,
        blocked: None,
        event: None,
        spent: false,
        town: None,
        gate: None,
        boss: None,
        encounter: None,
    };

    if let Some(p) = world.place_at(nx, ny) {
        match p.kind {
            PlaceKind::Town => {
                state.last_town = p.id.clone();
                out.town = Some(p.id.clone());
                return out;
            }
            PlaceKind::Event => {
                out.event = Some(p.id.clone());
                out.spent = state.answered.iter().any(|a| a == &p.id);
                return out;
            }
            PlaceKind::Gate => {
                // The step lands you on the gate; whether it opens is not the
                // world's business, because it depends on what is in the bag
                // and a `World` does not know about bags.
                out.gate = Some(p.id.clone());
                return out;
            }
            PlaceKind::Boss => {
                if !state.answered.iter().any(|a| a == &p.id) {
                    out.boss = Some(p.id.clone());
                    return out;
                }
            }
        }
    }

    let chance = world.encounter_per_mille(nx, ny);
    if chance > 0 && (rng.below(1000) as i32) < chance {
        out.encounter = world.draw_enemy(nx, ny, difficulty, rng);
        if out.encounter.is_some() {
            state.bump("encounters");
        }
    }
    out
}
