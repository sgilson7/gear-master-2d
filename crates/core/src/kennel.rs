//! The Kennel: the creature you beat enough times comes with you.
//!
//! **Brewing's shape again, and the third system built to it.** A bag that
//! opens in town, a thing with a footprint that is not a component, a bench
//! that is not a rectangle, a `C(8,2)` table a lint proves complete, one door
//! into the character and a clock.
//!
//! **Nothing here is a component either.** A kennelled creature has no
//! `PieceKind` and no entry in `CATALOG`; what it has is a family, a
//! silhouette, and somewhere to stand. The save fingerprint does not move.
//!
//! **And a boss is never offered.** `no_boss_is_kennelled` reads `rank`, and it
//! is in the suite before the offer is written — a boss on your side is the
//! balance risk in the whole idea, and the cap below is the other half of the
//! same answer.

use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "gm2d-kennel";
pub const VERSION: u32 = 1;

/// How many prior wins against a creature before it is offered.
///
/// **Five, which is the tally `fight::INSTANT_AFTER` already counts** — so the
/// offer arrives on the sixth win, and the counter that decides it is one this
/// game has kept since M15 rather than a second one meaning nearly the same
/// thing. *Derived, never banked*, twice over: the count is `beat:<canonical>`
/// and the eligibility is worked out fresh from it.
pub const OFFER_AT: u32 = 5;

/// How many fights between an out creature's meals. One is every fight.
pub const FEED_EVERY: u32 = 1;

/// Wins together before it gets better, and by how much.
pub const TALLY_STEP: u32 = 10;
pub const TALLY_PCT: i32 = 5;

/// One creature in the kennel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kennelled {
    /// Its canonical name, which is what `combat::creature` is keyed by.
    pub spec: String,
    /// The art family it is drawn from, and whose silhouette it stands on.
    ///
    /// **Twenty of these, and `PLAN-M21.md` §M21.4 says eight.** There are
    /// twenty art families and a silhouette is looked up by one, so eight
    /// would leave twelve families with nothing to stand on. The eight is the
    /// *pair table's* number and it is right there — see `eats`.
    pub family: String,
    /// The ingredient it eats, which is also what it pairs on.
    ///
    /// **The eight buckets, which is where the `C(8,2)` is.** A creature's
    /// family decides its silhouette and its diet decides its company, and the
    /// diet is one of the same eight the seeds and the retort already use — so
    /// the pair table is twenty-eight and provable rather than
    /// `C(20,2)` = 190 and unauthorable.
    pub eats: String,
    /// How many fights it has been out for and won.
    #[serde(default)]
    pub wins_together: u32,
    /// Whether it is out. **Only one is out at a time** unless a Handler says
    /// otherwise, and only an out creature eats.
    #[serde(default)]
    pub out: bool,
    /// Fights it has been out for since it last ate.
    ///
    /// **Its own clock, and it had to be.** The first version read
    /// `world.count("encounters")`, on the principle that a second counter
    /// meaning nearly the same thing is how two answers get made — and that
    /// counter is the *shim's*, bumped when a step rolls an encounter and
    /// never by `fight::settle`. So it was zero in every test and in every
    /// fight reached any other way, `0 % 2 == 0`, and the creature ate every
    /// time however the Handler was tuned. Found by a test that asserted it
    /// stayed out and watched it go in.
    ///
    /// *Fights since it last ate* is not a thing anything else in this game
    /// counts, so counting it here is not a duplicate.
    #[serde(default)]
    pub since_fed: u32,
    /// Where it stands in the run, and which way round.
    #[serde(default)]
    pub at: (i8, i8),
    #[serde(default)]
    pub turn: u8,
}

impl Kennelled {
    /// What its wins together have bought, in percentage points.
    ///
    /// **Every `TALLY_STEP`, not per win**, so the number on the card moves in
    /// steps a player can see rather than by a fraction nobody can check.
    pub fn tally_pct(&self) -> i32 {
        self.tally_pct_at(TALLY_PCT)
    }

    /// The same, at a Handler's own rate.
    pub fn tally_pct_at(&self, per_step: i32) -> i32 {
        (self.wins_together / TALLY_STEP) as i32 * per_step
    }
}

/// One family's silhouette: what it takes up in the run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Silhouette {
    pub family: String,
    pub name: String,
    pub blurb: String,
    /// Its footprint, cut from the family figure's bounding cells.
    pub cells: Vec<(i8, i8)>,
}

/// Two families that pay something when they stand touching in the run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub a: String,
    pub b: String,
    pub blurb: String,
    /// **Expressed in the rule vocabulary that already exists**, which is the
    /// constraint that keeps this table from inventing twenty-eight mechanics:
    /// a `Rule` the engine already reads, or a flat percentage on what the two
    /// contribute.
    pub gives: PairGift,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PairGift {
    /// Both of them contribute this much more.
    Together(i32),
    /// A rule, granted while both are out.
    Rule(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KennelData {
    pub format: String,
    pub version: u32,
    pub silhouettes: Vec<Silhouette>,
    #[serde(default)]
    pub pairs: Vec<Pair>,
}

impl KennelData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: KennelData =
            serde_json::from_str(text).map_err(|e| format!("kennel.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "this kennel is version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        for s in &d.silhouettes {
            if s.name.is_empty() || s.blurb.is_empty() {
                return Err(format!("{}: a silhouette that says nothing", s.family));
            }
            if s.cells.is_empty() {
                return Err(format!("{}: a silhouette that stands on nothing", s.family));
            }
        }
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for p in &d.pairs {
            if p.a == p.b {
                return Err(format!("{} is paired with itself", p.a));
            }
            // **Pairs are over diets, not families**, which is where the
            // twenty-eight is: eight buckets, `C(8,2)`, and a lint can count
            // it.
            let brews = crate::brew::BrewsData::parse(crate::data::BREWS_JSON)
                .map_err(|e| format!("the shipped brews are broken: {e}"))?;
            for f in [&p.a, &p.b] {
                if !brews.ingredients.iter().any(|i| i.id == *f && !i.ink_only) {
                    return Err(format!("{f:?} is no diet"));
                }
            }
            let key =
                if p.a < p.b { (p.a.as_str(), p.b.as_str()) } else { (p.b.as_str(), p.a.as_str()) };
            if seen.contains(&key) {
                return Err(format!("{} and {} are paired twice", key.0, key.1));
            }
            seen.push(key);
            if let PairGift::Rule(r) = &p.gives {
                if !matches!(r.as_str(), "beacon" | "spin_extra" | "productivity") {
                    return Err(format!("{} and {} grant {r:?}, which is no rule", p.a, p.b));
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, family: &str) -> Option<&Silhouette> {
        self.silhouettes.iter().find(|s| s.family == family)
    }

    /// What this pair pays, either way round.
    pub fn pair(&self, a: &str, b: &str) -> Option<&Pair> {
        self.pairs.iter().find(|p| (p.a == a && p.b == b) || (p.a == b && p.b == a))
    }
}

/// The cells one kennelled creature stands on, in run coordinates.
pub fn cells_of(data: &KennelData, k: &Kennelled) -> Vec<(i8, i8)> {
    let Some(s) = data.get(&k.family) else { return Vec::new() };
    crate::shape::Shape::new(&s.cells)
        .rotated(k.turn)
        .cells()
        .iter()
        .map(|&(x, y)| (x + k.at.0, y + k.at.1))
        .collect()
}

/// What a creature out contributes, as a share of what it is worth.
///
/// **Capped at the *region's* bracket and never at its own**, which is
/// `PLAN-M21.md` §M21.5's own decision and the answer to the balance risk in
/// the whole idea: a creature kennelled in the deep and walked back to the pit
/// would otherwise be a boss on your side. What it is worth where you are
/// standing is what it gives.
///
/// Returned as a percentage so the caller scales profiles rather than this
/// module knowing what a profile is.
pub fn share(rating: i32, danger: i32, tally_pct: i32) -> i32 {
    if rating <= 0 {
        return 0;
    }
    // At or under the bracket it gives everything; over it, the bracket's
    // worth. **Never more than a hundred before the tally**, so the tally is
    // the only thing that takes it past its own strength.
    let capped = if danger <= 0 { rating } else { rating.min(danger) };
    let base = (capped * 100) / rating;
    base + tally_pct
}

