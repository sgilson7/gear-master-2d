//! The Plot: things that grow while you fight.
//!
//! **Brewing's shape, one system along.** A bag that opens in town, a thing
//! with a footprint that is not a component, a bench that is not a rectangle,
//! a `C(8,2)` table a lint proves complete, one door into the character and a
//! clock. `brew.rs` is the template and this is deliberately built to match it,
//! because the third system in that shape is the one that proves the shape.
//!
//! **Nothing here is a component.** A seed has no `PieceKind`, no entry in
//! `CATALOG` and no home in any of the five grids — adding to the catalogue
//! moves the save fingerprint and there is a player mid-run. What a seed *has*
//! is somewhere to be (the drawer) and, once planted, a footprint that grows.
//!
//! **A seed is keyed exactly as an ingredient is**, by the art family of the
//! creature that dropped it, so a creature cannot arrive without one: it
//! already fails a test without a family, and `every_family_has_a_seed` is the
//! other half. The eight seeds and the eight pairing ingredients are the same
//! eight buckets — which is what makes the Plot *feed the larder* rather than
//! open a second economy beside it.

use crate::shape::Shape;
use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "gm2d-plot";
pub const VERSION: u32 = 1;

/// How many stages a crop grows through before it can be pulled.
///
/// **Three, and the pitch said seven.** Seven wins to a harvest is twenty
/// minutes at the run fixture's pace and a whole session at a new player's,
/// which makes the first thing the Plot ever does *nothing, for a session*.
/// Three is three wins — long enough that you plant and go away, short enough
/// that you come back. The Grower's `stages -1` takes it to two, which is why
/// that specialization exists.
pub const STAGES: u8 = 3;

/// Per-mille chance a win drops a seed, beside the ingredient it already pays.
///
/// **Per-mille like every other roll in this game**, for the reason all of them
/// are: a seeded walk has to produce the same drawer in every browser.
pub const SEED_PER_MILLE: u32 = 350;

/// How many of one seed the drawer holds before a win stops paying it.
///
/// **Not a cap on the drawer, a cap on the drop.** Six of one seed is two
/// harvests' worth of one family and more than any bed has room for at once;
/// past that a seventh is litter, and litter is what the errand tallies were
/// before they were gated on being asked for.
pub const DRAWER_CAP: u32 = 6;

/// One seed, and the crop it grows into.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedDef {
    pub id: String,
    pub name: String,
    pub blurb: String,
    /// The art families whose creatures drop it — **the same buckets
    /// `brew.rs` uses**, so the eight seeds and the eight pairing ingredients
    /// are one set of eight and a harvest lands somewhere the retort can use.
    pub from: Vec<String>,
    /// The ingredient a harvest pays into the larder.
    pub crop: String,
    /// What it takes up at each stage. Three of them, and each contains the
    /// one before it — `every_stage_shape_grows`.
    pub stages: Vec<Vec<(i8, i8)>>,
}

impl SeedDef {
    /// Its footprint at `stage`, as a `Shape`, so it turns the way everything
    /// else with a footprint in this game turns.
    pub fn shape_at_turned(&self, stage: u8, turn: u8) -> Shape {
        Shape::new(self.shape_at(stage)).rotated(turn)
    }

    /// Its footprint at `stage`, clamped to the last one it has.
    ///
    /// **Clamped rather than refused**, because a crop past its last stage is
    /// a crop that is ready and the caller asking is drawing it.
    pub fn shape_at(&self, stage: u8) -> &[(i8, i8)] {
        let i = (stage as usize).min(self.stages.len().saturating_sub(1));
        &self.stages[i]
    }

    /// The shape it ends at, which is what has to fit before it may be planted.
    pub fn harvest_shape(&self) -> &[(i8, i8)] {
        self.stages.last().map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotData {
    pub format: String,
    pub version: u32,
    pub seeds: Vec<SeedDef>,
}

impl PlotData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: PlotData =
            serde_json::from_str(text).map_err(|e| format!("plot.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these seeds are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        let brews = crate::brew::BrewsData::parse(crate::data::BREWS_JSON)
            .map_err(|e| format!("the shipped brews are broken: {e}"))?;
        for s in &d.seeds {
            if s.name.is_empty() || s.blurb.is_empty() {
                return Err(format!("{}: a seed that says nothing about itself", s.id));
            }
            if s.from.is_empty() {
                return Err(format!("{}: nothing drops it", s.id));
            }
            if s.stages.len() != STAGES as usize {
                return Err(format!(
                    "{}: {} stages, and a crop grows through {STAGES}",
                    s.id,
                    s.stages.len()
                ));
            }
            // **Every stage contains the one before it.** A crop that moved
            // cells as it grew would be a crop that can be planted legally and
            // then not fit, which is the stunting this game does not have — see
            // `Game::plant`, which refuses on the *harvest* shape for exactly
            // that reason.
            for w in s.stages.windows(2) {
                for c in &w[0] {
                    if !w[1].contains(c) {
                        return Err(format!("{}: a stage drops the cell {c:?}", s.id));
                    }
                }
            }
            // And what it grows into is something the retort can use.
            if !brews.ingredients.iter().any(|i| i.id == s.crop) {
                return Err(format!("{}: harvests {:?}, which is no ingredient", s.id, s.crop));
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&SeedDef> {
        self.seeds.iter().find(|s| s.id == id)
    }

    /// The seed a creature of this family drops, the way `from_family` answers
    /// for an ingredient — and it is the same question over the same buckets.
    pub fn from_family(&self, family: &str) -> Option<&SeedDef> {
        self.seeds.iter().find(|s| s.from.iter().any(|f| f == family))
    }
}

/// A seed in the ground: which one, how far along, and where it was planted.
///
/// **`at` is the anchor and the shape is the seed's**, which is the same
/// division a component makes between a `PieceId` and its cells: what it
/// covers is derived from what it is and how grown it is, so a crop cannot be
/// saved into a shape it does not have.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Crop {
    pub seed: String,
    pub at: (i8, i8),
    pub stage: u8,
    /// Which way round it was planted.
    ///
    /// **A divergence from `PLAN-M21.md` §M21.1**, which gives
    /// `Game::plant(town, seed, at)` and no turn — and it was measured rather
    /// than argued: of the twenty-four seed-and-bed pairs, **three fit nowhere
    /// at all** without one. A seed that is dead in a town for a reason the
    /// player cannot act on is worse than a bed that is too easy.
    ///
    /// It is also what the template already does. `brew::Seat` is
    /// `{ id, turn, at }` and the retort tries every rotation, because *a
    /// heuristic that refuses an arrangement somebody can see with their eyes*
    /// is the thing `brew::fits` exists not to be.
    #[serde(default)]
    pub turn: u8,
}

impl Crop {
    pub fn ready(&self) -> bool {
        self.stage + 1 >= STAGES
    }
}

/// The cells a crop covers right now, in bed coordinates.
pub fn cells_of(data: &PlotData, c: &Crop) -> Vec<(i8, i8)> {
    let Some(def) = data.get(&c.seed) else { return Vec::new() };
    at_of(&def.shape_at_turned(c.stage, c.turn), c.at)
}

/// The cells a crop will cover when it is ready, which is what must fit before
/// it may be planted at all.
pub fn harvest_cells(data: &PlotData, seed: &str, at: (i8, i8), turn: u8) -> Vec<(i8, i8)> {
    let Some(def) = data.get(seed) else { return Vec::new() };
    at_of(&def.shape_at_turned(STAGES - 1, turn), at)
}

fn at_of(shape: &Shape, at: (i8, i8)) -> Vec<(i8, i8)> {
    shape.cells().iter().map(|&(x, y)| (x + at.0, y + at.1)).collect()
}

/// What a harvest pays into the larder, per crop.
pub const HARVEST_YIELD: u32 = 2;

/// `3 cells` / `one cell`, for a refusal that says how big a thing is.
pub fn size_of(cells: &[(i8, i8)]) -> String {
    match cells.len() {
        1 => "one cell".into(),
        n => format!("{n} cells"),
    }
}

/// Cells as a player reads them, for a refusal that names them.
///
/// **TONE 12: a refusal names the thing in the way.** *It will not fit* is a
/// wall; *the bed stops short at (4, 1) and (5, 1)* is something to do about
/// it. Capped at four, because a list of nine coordinates is a wall with
/// numbers on it.
pub fn name_cells(cells: &[(i8, i8)]) -> String {
    let mut v: Vec<String> = cells.iter().take(4).map(|(x, y)| format!("({x}, {y})")).collect();
    if cells.len() > 4 {
        v.push(format!("and {} more", cells.len() - 4));
    }
    v.join(", ")
}

/// **One stage on every crop everywhere, per fight won.**
///
/// *Anywhere*, which is the whole of what makes the Plot the first thing in
/// this game that pays you for what you did between visits to town: the clock
/// is the bell, and the bell rings wherever you are.
pub fn tick(beds: &mut std::collections::BTreeMap<String, Vec<Crop>>, stages: u8) {
    for crops in beds.values_mut() {
        for c in crops.iter_mut() {
            if c.stage + 1 < stages {
                c.stage += 1;
            }
        }
    }
}
