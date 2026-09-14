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

/// What a pair of crops touching edge-on at harvest pays.
///
/// **One of four, and never two at once.** A pair that both doubled and paid an
/// ench seed would be a pair nobody could price against the other twenty-seven,
/// and the table is only worth authoring if its entries are comparable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Yield {
    /// Twice the ingredients, from both crops.
    Double(bool),
    /// The ink-style potency the pair's harvest carries, in percentage points.
    Potency(i32),
    /// One more of this ingredient, beside what the two paid.
    Second(String),
    /// **The Plot's second door.** An ench, straight into the rack, and one
    /// nothing else in the game sells.
    EnchSeed(String),
}

/// Two families that pay something when they come up touching.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Companion {
    pub a: String,
    pub b: String,
    pub blurb: String,
    pub gives: Yield,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotData {
    pub format: String,
    pub version: u32,
    pub seeds: Vec<SeedDef>,
    #[serde(default)]
    pub companions: Vec<Companion>,
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
        // **Every pair, and no pair twice.** A table of twenty-eight written by
        // hand is a table that can be twenty-seven, which is what
        // `every_pair_is_authored` is for — this is the load-time half, so a
        // data edit that duplicates one is refused rather than silently
        // shadowing the other.
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for c in &d.companions {
            if c.a == c.b {
                return Err(format!("{} is paired with itself", c.a));
            }
            for id in [&c.a, &c.b] {
                if !d.seeds.iter().any(|s| s.id == *id) {
                    return Err(format!("{id:?} is no seed"));
                }
            }
            let key = if c.a < c.b { (c.a.as_str(), c.b.as_str()) } else { (c.b.as_str(), c.a.as_str()) };
            if seen.contains(&key) {
                return Err(format!("{} and {} are paired twice", key.0, key.1));
            }
            seen.push(key);
            if c.blurb.is_empty() {
                return Err(format!("{} and {} say nothing", c.a, c.b));
            }
            match &c.gives {
                Yield::Potency(n) if *n <= 0 => {
                    return Err(format!("{} and {} pay nothing", c.a, c.b))
                }
                Yield::Second(id) if !brews.ingredients.iter().any(|i| i.id == *id) => {
                    return Err(format!("{} and {} pay {id:?}, which is no ingredient", c.a, c.b))
                }
                _ => {}
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&SeedDef> {
        self.seeds.iter().find(|s| s.id == id)
    }

    /// The seed a creature of this family drops, the way `from_family` answers
    /// for an ingredient — and it is the same question over the same buckets.
    /// What this pair pays, either way round.
    ///
    /// **Order-insensitive**, because which of the two you planted first is a
    /// fact about your afternoon and not about the pair — the same argument
    /// `expert::for_pair` makes, and for the same reason the lint can count.
    pub fn pair(&self, a: &str, b: &str) -> Option<&Companion> {
        self.companions
            .iter()
            .find(|c| (c.a == a && c.b == b) || (c.a == b && c.b == a))
    }

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
///
/// **Four, and the first draft was two — which made the Grower's whole promise
/// round to nothing.** `yield_pct` is 120 untuned, and `2 * 120 / 100` is two:
/// the specialization paid exactly what everybody else got, and the test that
/// found it read *a grower pulled 2 and everybody else pulls 2*. That is the
/// `SPELL_MANA_COST` failure, which this project already has written down —
/// *a percentage off three rounds to nothing* — with a crop in it instead of a
/// cast.
///
/// Four gives the ladder room: 120% is five, and a finished tree's 170% is
/// seven. The sum is rounded **the payer's way** in `Game::harvest`, which is
/// what `ExpertPower::cast_price` does and for the same reason.
pub const HARVEST_YIELD: u32 = 4;

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

/// Which cells count as touching, at a given reach.
///
/// **One is edge-on and two is edge-on and the corners.** The Grower's
/// *Companion, Thrice* reads the adjacency table wider rather than adding a
/// second one — the same move `Rule::Spread` made when a corner turned out to
/// be the tightest spread this board allows.
pub fn neighbours(reach: u32) -> Vec<(i8, i8)> {
    let mut v = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
    if reach >= 2 {
        v.extend([(1, 1), (1, -1), (-1, 1), (-1, -1)]);
    }
    v
}

/// A bed with `extra` more cells in it.
///
/// **Placed by the mask rather than by the player**, in reading order inside
/// the bounding box the mask already implies — so *The Long Row* gives the
/// same three cells to everybody in the same town, and a node cannot become a
/// second bed editor. Growing only, which is `resize_boards`'s rule one system
/// along: a bed that got smaller would tip out whatever was standing in it.
pub fn widened(mask: &[(i8, i8)], extra: u32) -> Vec<(i8, i8)> {
    let mut out = mask.to_vec();
    if extra == 0 || mask.is_empty() {
        return out;
    }
    let w = mask.iter().map(|c| c.0).max().unwrap_or(0);
    let h = mask.iter().map(|c| c.1).max().unwrap_or(0);
    let mut added = 0;
    // One row past the box as well, so a full rectangle can still be widened.
    for y in 0..=(h + 1) {
        for x in 0..=(w + 1) {
            if added >= extra {
                return out;
            }
            if !out.contains(&(x, y)) {
                out.push((x, y));
                added += 1;
            }
        }
    }
    out
}

