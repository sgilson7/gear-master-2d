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
}

impl Crop {
    pub fn ready(&self) -> bool {
        self.stage + 1 >= STAGES
    }
}

/// The cells a crop covers right now, in bed coordinates.
pub fn cells_of(data: &PlotData, c: &Crop) -> Vec<(i8, i8)> {
    let Some(def) = data.get(&c.seed) else { return Vec::new() };
    def.shape_at(c.stage).iter().map(|(x, y)| (c.at.0 + x, c.at.1 + y)).collect()
}

/// The cells a crop will cover when it is ready, which is what must fit before
/// it may be planted at all.
pub fn harvest_cells(data: &PlotData, seed: &str, at: (i8, i8)) -> Vec<(i8, i8)> {
    let Some(def) = data.get(seed) else { return Vec::new() };
    def.harvest_shape().iter().map(|(x, y)| (at.0 + x, at.1 + y)).collect()
}
