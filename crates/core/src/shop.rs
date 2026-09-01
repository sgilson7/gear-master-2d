//! What a town sells.
//!
//! **A shelf is content, not state.** Which components a town deals is
//! `data/shops.json` and it never changes; what a save carries is the short
//! list of things already bought there. That is the discipline the map has
//! followed since M2 — "content is not state" — and it is the reason a shelf
//! can be retuned without touching anybody's file.
//!
//! It replaces a randomised, rerollable shop. That shop was upstream's, and it
//! was right for a ladder: one shelf for the whole run, turned over for a coin
//! whenever it dealt you nothing. GM2D is a map with towns on it, and a town
//! that sells something different every time you walk in is not a town — it is
//! the same slot machine in three costumes. Fixed stock makes a town a place:
//! High Wick is where the plating is, and if you want plating you go there.
//!
//! Prices are the catalogue's. There is no discount and no mark-up, which is
//! what makes §C.3 — *the screen shows the price actually charged* — trivially
//! true here rather than something to keep checking.

use serde::{Deserialize, Serialize};

use crate::piece::{PieceDef, CATALOG};

/// What you start a run with.
///
/// You own a handle and a blade and nothing else, so this is the first upgrade
/// rather than pocket change. It buys two or three cheap components at the
/// starter town, which — with what the pit pays — is the opening.
pub const STARTING_GOLD: i32 = 28;

pub const FORMAT: &str = "gm2d-shops";
pub const VERSION: u32 = 1;

/// One town's shelf, in the order it is displayed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TownShelf {
    /// The place id from `data/tiles.json`.
    pub id: String,
    /// Canonical catalogue names. Order is the display order and, because a
    /// save records purchases *by index*, it is also part of the file's
    /// contract: inserting into the middle of a list moves what somebody
    /// already bought. Append.
    pub stock: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopsData {
    pub format: String,
    pub version: u32,
    pub towns: Vec<TownShelf>,
}

impl ShopsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: ShopsData =
            serde_json::from_str(text).map_err(|e| format!("shops.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these shelves are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        for t in &d.towns {
            for name in &t.stock {
                if def_named(name).is_none() {
                    return Err(format!("{} stocks {name:?}, which is not in the catalogue", t.id));
                }
            }
        }
        Ok(d)
    }

    pub fn town(&self, id: &str) -> Option<&TownShelf> {
        self.towns.iter().find(|t| t.id == id)
    }
}

/// The catalogue index of a piece by canonical name.
pub fn def_named(name: &str) -> Option<usize> {
    CATALOG.iter().position(|d| d.name == name)
}

/// One line of a town's shelf as the screen needs it.
pub struct Offer {
    /// Position in the town's stock list. What a save records.
    pub index: usize,
    pub def: &'static PieceDef,
    /// What is actually charged. The catalogue's, and the only price there is.
    pub price: i32,
    pub sold: bool,
}

/// A town's whole shelf, sold entries included.
///
/// Sold entries are kept and marked rather than dropped, because the index is
/// the identity: dropping them would renumber everything after and a save
/// recording "bought number three" would come back pointing at a different
/// component.
pub fn shelf(shops: &ShopsData, town: &str, sold: &[(String, u16)]) -> Vec<Offer> {
    let Some(t) = shops.town(town) else { return Vec::new() };
    t.stock
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            let def = &CATALOG[def_named(name)?];
            Some(Offer {
                index: i,
                def,
                price: def.price,
                sold: sold.iter().any(|(w, n)| w == town && *n as usize == i),
            })
        })
        .collect()
}
