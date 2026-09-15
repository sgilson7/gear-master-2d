//! The Stall: a shelf of your own, and somebody who wants what is on it.
//!
//! **Brewing's shape a fourth time** — a bag that opens in town, a bench that
//! is not a rectangle, a `C(8,2)` table a lint proves complete, one door into
//! the character and a clock.
//!
//! **And the one place a `CATALOG` piece is placed by shape outside the five
//! grids.** It comes from the tray and it goes back to the tray: nothing is
//! created and nothing is destroyed but by a sale, which is what keeps this
//! from being a second way to make components.

use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "gm2d-stall";
pub const VERSION: u32 = 1;

/// How many buyers come by per fight won, anywhere.
pub const CUSTOMERS_PER_BELL: u32 = 1;

/// How far either side of the barrel's figure still counts as fair, in percent.
pub const FAIR_PCT: i32 = 20;

/// One buyer in how many buys at a high price.
pub const HIGH_ODDS: u32 = 3;

/// Per-mille that a pair of kin buyers in a row pays a bargain.
///
/// **Not a certainty**, because two kin in a row is already the uncommon thing:
/// eight buyers drawn at random put a given pair together about one time in
/// sixty-four, and a certainty on top of that would be a faucet somebody could
/// sit at.
pub const BARGAIN_PER_MILLE: u32 = 400;

/// One person who comes by wanting something.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buyer {
    pub id: String,
    pub name: String,
    pub blurb: String,
    /// The two grids they will buy out of.
    pub wants: Vec<String>,
    /// What an item has to rate before they are interested.
    pub floor: i32,
}

/// Two buyers who are kin, and what a bargain off them is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kin {
    pub a: String,
    pub b: String,
    pub blurb: String,
    /// What they leave. **An ench or a component**, and never anything a
    /// counter sells — see `a_bargain_is_never_on_the_barrel`.
    pub gives: Bargain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Bargain {
    /// **A unique ench, off a sale.** Asked for: *you should sometimes receive
    /// unique enchs from selling items in your store front.* Three of the six
    /// M21.6 wrote are the Stall's, and this is how they are got — the only
    /// way, which is what makes them the Stall's.
    Ench(String),
    /// A component nothing sells, onto the shelf, free.
    Piece(String),
}

/// One sale, kept so the ledger can be read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sale {
    pub buyer: String,
    pub item: String,
    pub paid: i32,
    /// What the barrel would have charged, so the ledger says whether you did
    /// well rather than only what you got.
    pub worth: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StallData {
    pub format: String,
    pub version: u32,
    pub buyers: Vec<Buyer>,
    #[serde(default)]
    pub kin: Vec<Kin>,
}

impl StallData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: StallData =
            serde_json::from_str(text).map_err(|e| format!("stall.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "this stall is version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        let enchs = crate::ench::EnchsData::parse(crate::data::ENCHS_JSON)
            .map_err(|e| format!("the shipped enchs are broken: {e}"))?;
        for b in &d.buyers {
            if b.name.is_empty() || b.blurb.is_empty() {
                return Err(format!("{}: a buyer who says nothing", b.id));
            }
            if b.wants.len() != 2 {
                return Err(format!("{}: wants {} grids, and a buyer wants two", b.id, b.wants.len()));
            }
            for w in &b.wants {
                if crate::skills::slot_of(w).is_none() {
                    return Err(format!("{}: {w:?} is no grid", b.id));
                }
            }
            if b.floor < 0 {
                return Err(format!("{}: a floor under nothing", b.id));
            }
        }
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for k in &d.kin {
            if k.a == k.b {
                return Err(format!("{} is kin to itself", k.a));
            }
            for id in [&k.a, &k.b] {
                if !d.buyers.iter().any(|b| b.id == *id) {
                    return Err(format!("{id:?} is no buyer"));
                }
            }
            let key = if k.a < k.b { (k.a.as_str(), k.b.as_str()) } else { (k.b.as_str(), k.a.as_str()) };
            if seen.contains(&key) {
                return Err(format!("{} and {} are kin twice", key.0, key.1));
            }
            seen.push(key);
            match &k.gives {
                Bargain::Ench(id) => {
                    let Some(e) = enchs.get(id) else {
                        return Err(format!("{} and {} pay {id:?}, which is no ench", k.a, k.b));
                    };
                    // **Priceless, which is what keeps it off every counter.**
                    // A bargain you could have bought makes the selling a slow
                    // way to shop — the errands' own rule since M8.
                    if e.price.is_some() {
                        return Err(format!("{id:?} is a bargain and also for sale"));
                    }
                }
                Bargain::Piece(name) => {
                    if !crate::piece::CATALOG.iter().any(|d| d.name == *name) {
                        return Err(format!("{} and {} pay {name:?}, which is no component", k.a, k.b));
                    }
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&Buyer> {
        self.buyers.iter().find(|b| b.id == id)
    }

    pub fn kin_of(&self, a: &str, b: &str) -> Option<&Kin> {
        self.kin.iter().find(|k| (k.a == a && k.b == b) || (k.a == b && k.b == a))
    }
}

/// One component on the shelf, at a price you set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnShelf {
    pub piece: crate::piece::PieceId,
    pub at: (i8, i8),
    #[serde(default)]
    pub turn: u8,
    pub price: i32,
}

/// The shelf's mask. **Fourteen cells in a five-by-four, and not a rectangle.**
///
/// A broad counter with a trestle at the end of it. The number is measured
/// rather than chosen, and the first draft was nine cells in a four-by-three —
/// which **refused the Iron Blade**, the weapon every character in this game
/// starts holding, because a blade is one cell by four and the counter was
/// three tall. *A counter that cannot hold the starting weapon is a counter
/// nobody opens.*
///
/// What the catalogue actually is, measured: bounding boxes from one by one to
/// six by six, 537 of the 568 components inside three by two, **19 of them four
/// by one**, four of them four by three, and two longer than five. So the mask
/// takes a four by three exactly — and then has two cells left, which is the
/// whole packing decision — admits a five in one row, and refuses the six.
pub const SHELF: &[(i8, i8)] = &[
    (0, 0), (1, 0), (2, 0), (3, 0),
    (0, 1), (1, 1), (2, 1), (3, 1),
    (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),
                                    (4, 3),
];

/// Whether a price is fair, high or low against what the thing is worth.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Low,
    Fair,
    High,
}

/// **The one answer**, so the card and the sale cannot disagree — which is
/// §C.3's rule (*the price shown is the price taken*) read from the other side
/// of the counter.
pub fn ask_of(price: i32, worth: i32) -> Ask {
    ask_of_with(price, worth, 0)
}

/// The same, with a Factor's patience added to the band.
///
/// **A different axis from the margin.** The margin pays you over on a sale you
/// were always going to make; patience makes a sale you were not — somebody
/// stretching past what a thing is worth. So the two nodes are not two names
/// for one number, which is the constraint every expert tree is written to.
pub fn ask_of_with(price: i32, worth: i32, patience_pct: i32) -> Ask {
    if worth <= 0 {
        return Ask::Fair;
    }
    let margin = worth * (FAIR_PCT + patience_pct.max(0)) / 100;
    if price > worth + margin {
        Ask::High
    } else if price < worth - margin {
        Ask::Low
    } else {
        Ask::Fair
    }
}
