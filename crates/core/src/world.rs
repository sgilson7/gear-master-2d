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

/// How far off dry land a waded tile may be.
///
/// One, and the plan measured what one means before it was written: a water
/// tile that touches somewhere you could already stand. On the first map that
/// opens fourteen of the lake's twenty-eight tiles — the rim — and leaves the
/// middle fourteen shut, so the lake stops being a wall at its edge while
/// staying one through its middle. No new terrain and no repaint.
pub const WADE_DEPTH: u8 = 1;

/// What the character stepping is allowed to do that nobody else is.
///
/// **A map does not know about bags**, and [`step`] takes this rather than the
/// character so that it never has to. The caller fills it from
/// `Character::rules`; the same division a gate's key makes, and a door's, made
/// once and given a name.
///
/// Plain and small on purpose. Every field here is a *refusal a step would
/// otherwise make*, so a `Default` is the game everybody has always played and
/// no existing caller has to say it holds nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Allowances {
    /// Water within [`WADE_DEPTH`] of land is ground.
    pub wade: bool,
    /// How far up the map the walker may go, as a level.
    ///
    /// **Not from a rule**, which is why [`Allowances::of`] does not set it —
    /// it is a fact about the character that a crossing asks for, handed in
    /// with everything else the map is not allowed to go and read. `Default` is
    /// zero, which is nobody: a caller that has a character says so, and one
    /// that does not is refused by every crossing, which is the safe way round.
    pub level: u32,
}

impl Allowances {
    /// Read a rule list for everything a step cares about.
    ///
    /// **Exhaustive**, which is the point of it being here: a rule added to the
    /// game is a rule somebody has to decide is not about walking.
    /// Everything a rule list grants. **[`Allowances::level`] is not one of
    /// them** and is filled in by the caller — see the field.
    pub fn of(rules: &[crate::rule::Rule]) -> Self {
        use crate::rule::Rule;
        let mut out = Allowances::default();
        for r in rules {
            match r {
                Rule::Wade => out.wade = true,
                // Combat rules, a map-screen rule and an encounter rule. None
                // of them is about whether a tile can be stood on.
                Rule::CurseOnActivate { .. }
                | Rule::SpinExtra { .. }
                | Rule::SpinKeep { .. }
                | Rule::SpinEvery { .. }
                | Rule::Scout
                | Rule::Rout { .. } => {}
            }
        }
        out
    }
}

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
    /// A way out of everything that is written.
    ///
    /// **Not a gate.** A gate leads to another map and this leads to whatever
    /// is past the demo, which is nothing yet — so it gets its own kind and
    /// its own mark rather than borrowing the diamond and reading as a place
    /// you could come back from.
    Door,
    /// Somebody selling what no town does.
    ///
    /// **The only shop in the game that is not a town.** M10 took the ench
    /// bench off every town shelf, because a thing every town sells is not a
    /// thing you went and got — so what a skill tree does not award is sold
    /// here, by one person, on a tile that is not there until level ten.
    ///
    /// It is a `PlaceKind` and not an `Event` because an event's choices are
    /// spent as a set: `answer` refuses a second choice and writes the whole
    /// event id into `answered`, so a card could sell one thing once. A bench
    /// sells each line once, which is the shelf rule the towns already follow.
    Bench,
    /// A threshold you may cross only when something is true of you.
    ///
    /// A gate's sibling: a gate is a way onto another map and a crossing is a
    /// way further up this one. It exists because *nothing stopped a level-one
    /// character walking fifteen tiles north into a region of two-thousand
    /// rated creatures* — the gradient was a gradient and not a gate.
    ///
    /// **It guards a region, not its own tile**, which is a divergence from
    /// `PLAN-M9.md` and the map is the reason: rows four to fifteen are open
    /// ground twelve tiles wide, so a crossing that refused only the square it
    /// stands on would need a dozen of them across a row, which is the wall the
    /// plan rejected. So the place is a milestone you can walk up to and read,
    /// standing on the near side of what it guards, and `guards` names the
    /// region behind it.
    ///
    /// **It is the first thing in the game gated on what you *are* rather than
    /// on what you carry.** A key is in the bag and a level is not, which is
    /// why the number reaches `step` through [`Allowances`] with everything
    /// else the map is not allowed to know about the walker.
    Crossing,
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
    /// Not here at all until this id is in `answered` or in `flags`.
    ///
    /// **The first conditional place.** A rock in the wall that is a door once
    /// the Cave's boss is down — not drawn, not steppable, and absent from
    /// [`World::place_now`] until then.
    ///
    /// Spawning one at runtime was the other option and is rejected for the
    /// reason the map is not in the save: **places are content and content is
    /// not state.** A place that is always in the file and sometimes invisible
    /// keeps that true; a place that appears in a save does not.
    ///
    /// It reads `answered` rather than the bag, because a `World` does not know
    /// about bags — the same rule that puts a gate's key in the shim. What the
    /// door *wants* is still a component, and `needs` is where that lives.
    #[serde(default)]
    pub hidden_until: Option<String>,
    /// `Door`: what it says when it opens. The only prose a place carries, and
    /// it is here rather than in `events.json` because a door is not a card:
    /// there is nothing to choose and nothing to answer.
    #[serde(default)]
    pub prose: Vec<String>,
    /// Not here at all until the walker is this level.
    ///
    /// The sibling of [`PlaceDef::hidden_until`], and a separate field rather
    /// than a second meaning for it: that one names something in `answered` or
    /// `flags`, and a level is in neither. Writing `level-10` into `flags` when
    /// it was reached would have worked and is refused — the level is derived
    /// from experience and never stored, and a flag saying otherwise would be
    /// the second answer to a question that has had one since M4.
    ///
    /// **Distinct from `needs_level`**, which is a `Crossing`'s and is about
    /// getting *past* something. This is about whether the place is there.
    #[serde(default)]
    pub hidden_until_level: Option<u32>,
    /// `Bench`: the enchs this one keeps, by id, in the order they are shown.
    ///
    /// **Here rather than in `enchs.json`**, which is the division the
    /// components already make: the catalogue says what a thing is and what it
    /// costs, and the shelf says who sells it. `World::load` refuses a bench
    /// naming an ench the catalogue has not got, and one selling an ench that
    /// has no price — *a priceless ench is on nobody's bench*, which is the
    /// errands' half of the rule and has been since M8.
    #[serde(default)]
    pub sells: Vec<String>,
    /// `Crossing`: the id of the region on the far side of it.
    #[serde(default)]
    pub guards: Option<String>,
    /// `Crossing`: the level it asks for.
    ///
    /// **Not a danger number.** Danger is measured — the mean rating of what a
    /// region holds — and `no_data_file_types_a_danger_number` fails the build
    /// on one typed into a data file. This is the opposite kind of number: it
    /// is what the road asks of *you*, and there is nothing to derive it from
    /// because it is a pacing decision rather than a measurement.
    #[serde(default)]
    pub needs_level: Option<u32>,
}

/// Is this place there yet?
///
/// A free function rather than a method, so the check is one place and every
/// caller is obviously asking the same question.
pub fn place_is_there(p: &PlaceDef, state: &WorldState, allowed: &Allowances) -> bool {
    // Two gates and both must hold. One reads what has happened and the other
    // reads who is asking — see the two fields.
    if p.hidden_until_level.is_some_and(|n| allowed.level < n) {
        return false;
    }
    match &p.hidden_until {
        None => true,
        Some(k) => {
            state.answered.iter().any(|a| a == k) || state.flags.iter().any(|f| f == k)
        }
    }
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
        // A bench selling an ench the catalogue has not got is a shop nobody
        // can buy from, and nothing else in the game would say so. The same
        // guard `Rule::check` is, and it runs where the map is read.
        let enchs = crate::ench::EnchsData::parse(crate::data::ENCHS_JSON)
            .map_err(|e| format!("the shipped enchs are broken: {e}"))?;
        for p in &world.places {
            if p.kind != PlaceKind::Bench && !p.sells.is_empty() {
                return Err(format!("{}: only a bench sells anything", p.id));
            }
            if p.kind == PlaceKind::Bench && p.sells.is_empty() {
                return Err(format!("{}: a bench with nothing on it", p.id));
            }
            for id in &p.sells {
                match enchs.get(id) {
                    None => return Err(format!("{}: there is no ench called {id:?}", p.id)),
                    // **A priceless ench is on nobody's bench**, which is the
                    // errands' half of the rule and has been since M8: a reward
                    // you could have bought makes the errand a slow way to shop.
                    Some(e) if e.price.is_none() => {
                        return Err(format!("{}: {id:?} is not for sale anywhere", p.id))
                    }
                    Some(_) => {}
                }
            }
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

    /// Is this tile within [`WADE_DEPTH`] of somewhere you could stand?
    ///
    /// The map's half of `Rule::Wade`: a question about the ground, which the
    /// map can answer, and not about who is walking on it, which it cannot.
    /// Only impassable tiles are ever asked — [`passable`](Self::passable) has
    /// already said yes to everything else.
    pub fn shallow(&self, x: u8, y: u8) -> bool {
        let d = WADE_DEPTH as i32;
        for dy in -d..=d {
            for dx in -d..=d {
                // Orthogonal only. A diagonal touch is a corner, and a corner
                // is not somewhere you can put a foot down on the way in.
                if dx.abs() + dy.abs() > d || (dx == 0 && dy == 0) {
                    continue;
                }
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if self.in_bounds(nx, ny) && self.passable(nx as u8, ny as u8) {
                    return true;
                }
            }
        }
        false
    }

    /// Can this character stand here?
    ///
    /// [`passable`](Self::passable) is the ground's answer and this is the
    /// game's. Everything that decides where a player may be goes through it —
    /// a step, and nothing else yet, because `repair` deliberately does not:
    /// see the note on it.
    pub fn walkable(&self, x: u8, y: u8, allowed: &Allowances) -> bool {
        self.passable(x, y)
            || (allowed.wade && self.terrain_name(x, y) == "water" && self.shallow(x, y))
    }

    fn region_index(&self, x: u8, y: u8) -> Option<usize> {
        let i = self.region_of[self.idx(x, y)];
        (i != usize::MAX).then_some(i)
    }

    pub fn region_at(&self, x: u8, y: u8) -> Option<&Region> {
        self.region_index(x, y).map(|i| &self.regions[i])
    }

    /// The place on this tile, **whether or not it is there yet**.
    ///
    /// The raw map query. Everything a player can see or walk into goes
    /// through [`World::place_now`]; this is for the questions that are about
    /// the file rather than about a game — where the last town is, whether the
    /// data is well formed.
    pub fn place_at(&self, x: u8, y: u8) -> Option<&PlaceDef> {
        self.places.iter().find(|p| p.at == [x, y])
    }

    /// The place on this tile that is there right now.
    pub fn place_now(
        &self,
        state: &WorldState,
        x: u8,
        y: u8,
        allowed: &Allowances,
    ) -> Option<&PlaceDef> {
        self.place_at(x, y).filter(|p| place_is_there(p, state, allowed))
    }

    /// The crossing guarding the region this tile is in, if one does.
    ///
    /// A region rather than a tile — see [`PlaceKind::Crossing`]. Found by
    /// walking the places rather than by an index, because there are two of
    /// them on the biggest map this build ships and a cache would be a second
    /// place for the answer to be stale.
    pub fn crossing_into(
        &self,
        state: &WorldState,
        x: u8,
        y: u8,
        allowed: &Allowances,
    ) -> Option<&PlaceDef> {
        let region = self.region_at(x, y)?;
        self.places.iter().find(|p| {
            p.kind == PlaceKind::Crossing
                && p.guards.as_deref() == Some(region.id.as_str())
                && place_is_there(p, state, allowed)
        })
    }

    /// Why a crossing refuses this step, if one does.
    ///
    /// **Only on the way in.** A step that stays inside the guarded region is
    /// never refused, which is what stops a crossing being a way to strand
    /// somebody: a save planted on the far side of one can still walk out of
    /// it. A threshold is a threshold and not a cage.
    ///
    /// The sentence is two registers kept apart, the same split `TONE.md` 13a
    /// makes everywhere else: `shut` is the world's and is written in
    /// `tiles.json`, and the numbers are the engine's and are derived here. A
    /// `shut` line that quoted its own level would be a second copy of
    /// `needs_level` sitting two lines above it in the same file.
    pub fn crossing_refuses(
        &self,
        state: &WorldState,
        to: (u8, u8),
        allowed: &Allowances,
    ) -> Option<String> {
        let c = self.crossing_between(state, to, allowed)?;
        let need = c.needs_level?;
        Some(format!("{} It wants level {need}, and you are {}.", c.shut, allowed.level))
    }

    /// The crossing standing between the player and this tile, if one is shut.
    ///
    /// The question `crossing_refuses` asks about the next step and the quest
    /// log asks about somewhere thirty tiles away. One answer to it, because
    /// two would be a map that refused a step for one reason and explained it
    /// with another.
    pub fn crossing_between(
        &self,
        state: &WorldState,
        to: (u8, u8),
        allowed: &Allowances,
    ) -> Option<&PlaceDef> {
        let here = self.region_at(state.at[0], state.at[1]).map(|r| r.id.as_str());
        let there = self.region_at(to.0, to.1)?;
        if here == Some(there.id.as_str()) {
            return None;
        }
        let c = self.crossing_into(state, to.0, to.1, allowed)?;
        (allowed.level < c.needs_level?).then_some(c)
    }

    /// Every place on this map that is there right now.
    pub fn places_now(&self, state: &WorldState, allowed: &Allowances) -> Vec<&PlaceDef> {
        self.places.iter().filter(|p| place_is_there(p, state, allowed)).collect()
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
    ///
    /// **It takes the allowances too, and lands you only on ground.** A player
    /// standing on a waded tile is standing somewhere they are allowed to be,
    /// so the first argument is needed or the next keypress would walk them
    /// home out of the middle of a lake they were legally in. Where it *puts*
    /// somebody ignores them entirely, and has to: a rim tile is only a place
    /// to stand while the set is on the board, and a repair that used one
    /// would be a repair you could unpack your way out of.
    pub fn repair(&self, state: &mut WorldState, allowed: &Allowances) -> Option<[u8; 2]> {
        let [x, y] = state.at;
        if self.in_bounds(x as i32, y as i32) && self.walkable(x, y, allowed) {
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
    /// Enchs bought off a bench, by id.
    ///
    /// **A separate list from `bought`**, which is `(town, index)` into a
    /// shelf. An ench is bought by name and there is exactly one of each, so an
    /// index would be one list answering two questions — and the shelf's index
    /// rule exists because dropping a sold entry would renumber the rest.
    #[serde(default)]
    pub bought_enchs: Vec<String>,
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
    /// A door you are now standing on. Whether it opens is the caller's, for
    /// the reason a gate's is: it depends on what is in the bag.
    pub door: Option<String>,
    /// A gate you are now standing on. Whether it opens is the caller's
    /// question, because it depends on what is in the bag.
    pub gate: Option<String>,
    /// A creature standing here rather than one the ground rolled.
    pub boss: Option<String>,
    /// A bench you are now standing at.
    pub bench: Option<String>,
    /// The crossing that refused this step, if one did.
    ///
    /// `blocked` already carries the sentence; this says *which kind* of
    /// refusal it was, so the page can put it where a player will read it. A
    /// cliff is a bump and belongs in the one-line flash at the bottom of the
    /// map; a crossing is a fact about where the game goes next and deserves
    /// the message panel.
    pub crossing: Option<String>,
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
            door: None,
            boss: None,
            bench: None,
            crossing: None,
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
    allowed: &Allowances,
) -> Step {
    let (dx, dy) = dir.delta();
    let (nx, ny) = (state.at[0] as i32 + dx, state.at[1] as i32 + dy);
    if !world.in_bounds(nx, ny) {
        return Step::nowhere("the map ends here");
    }
    let (nx, ny) = (nx as u8, ny as u8);
    // **What the walker is allowed to do, not who they are.** `allowed` is a
    // handful of bools the caller filled in; a `World` that took a character
    // to answer this would be a map that knew about bags.
    if !world.walkable(nx, ny, allowed) {
        return Step::nowhere(match world.terrain_name(nx, ny) {
            // Still the frame's fault, and a toad's frame is the answer to it.
            "water" => "you would have to swim, and you are wearing a frame",
            "rock" => "rock, and no way up it",
            _ => "there is no way through",
        });
    }

    // **The crossing, before the step rather than after it.** Everything else a
    // place does happens once you are standing on it; this is the one that has
    // to happen before, because what it does is refuse. Nothing is drawn and no
    // roll is made — a refused step is a step that never happened, the same as
    // walking into a cliff.
    if let Some(why) = world.crossing_refuses(state, (nx, ny), allowed) {
        let mut out = Step::nowhere(&why);
        out.crossing = world.crossing_into(state, nx, ny, allowed).map(|c| c.id.clone());
        return out;
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
        door: None,
        boss: None,
        bench: None,
        crossing: None,
        encounter: None,
    };

    // **Not there is not there.** A hidden place is not walked onto, not
    // reported and not drawn, so a door in a wall is a wall until the thing
    // that opens it has happened.
    if let Some(p) = world.place_now(state, nx, ny, allowed) {
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
            PlaceKind::Door => {
                // Whether it opens depends on what is in the bag, and a map
                // does not know about bags — the same division a gate makes.
                out.door = Some(p.id.clone());
                return out;
            }
            PlaceKind::Boss => {
                if !state.answered.iter().any(|a| a == &p.id) {
                    out.boss = Some(p.id.clone());
                    return out;
                }
            }
            // Somebody standing there with things for sale. Whether you can
            // afford any of it is not the world's business — the same division
            // a gate's key makes.
            PlaceKind::Bench => {
                out.bench = Some(p.id.clone());
                return out;
            }
            // **You walk over it.** A crossing that stopped you on its own tile
            // would be a gate, and it is not one: it has already had its say,
            // above, before the step was taken. Falls through to the encounter
            // roll like any ordinary ground.
            PlaceKind::Crossing => {}
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
