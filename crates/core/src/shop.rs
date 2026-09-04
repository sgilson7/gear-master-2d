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
    /// **The barrel under the counter, and it is the same barrel everywhere.**
    ///
    /// One list, not a list per town, because the barrel is furniture rather
    /// than a place's character: a shelf says *High Wick is where the plating
    /// is*, and a barrel says nothing at all. A regional barrel would be a
    /// second designed curve to keep tuned, which is an M13 idea wearing this
    /// block's clothes — `PLAN-M12.md` §8 row 2.
    ///
    /// **It never runs out**, so nothing here is an index into anything and
    /// buying from it writes nothing to the save. That is the whole reason
    /// this block has no seam: `bought` is `(town, index)` because a shelf
    /// entry is spent, and a barrel entry is not.
    #[serde(default)]
    pub barrel: Vec<String>,
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
        for name in &d.barrel {
            let Some(i) = def_named(name) else {
                return Err(format!("the barrel holds {name:?}, which is not in the catalogue"));
            };
            // **A barrel entry must not also be on a shelf.** The barrel never
            // runs out and undercuts nothing — it is the *same* price — so a
            // component in both places is a shelf entry a player would never
            // reach for, and a shelf line nobody takes is a line of the
            // designed curve that has quietly stopped existing.
            if let Some(t) = d.towns.iter().find(|t| t.stock.iter().any(|s| s == name)) {
                return Err(format!(
                    "the barrel holds {name:?} and so does {}'s shelf; a shelf entry the                      barrel also carries is a shelf entry nobody takes",
                    t.id
                ));
            }
            // A quest item is carried, never worn, so a barrel full of them
            // would sell cells nobody can use.
            if CATALOG[i].kind == crate::piece::PieceKind::Quest {
                return Err(format!("the barrel holds {name:?}, which is carried and not worn"));
            }
            // **Nothing off a creature.** `EVENT_ONLY` is eighty-three names —
            // every set piece, every instrument part, every errand tally — and
            // a set you could buy out of a barrel is not a set anybody earned.
            // This is here because the recon that chose the stock read that
            // list with a regex and got thirteen of the eighty-three, which is
            // the same "second copy of a list" this project keeps paying for.
            if crate::piece::EVENT_ONLY.contains(&CATALOG[i].name) {
                return Err(format!(
                    "the barrel holds {name:?}, which is off a creature or out of an event"
                ));
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

/// One line of the barrel.
///
/// **There is no `sold` field and the absence is the statement.** An `Offer`
/// whose `sold` was always false would be a lie waiting for somebody to wire
/// `bought` into it; a type with nowhere to put the answer cannot be asked the
/// question. The barrel never empties, which is what makes it the shop's floor
/// rather than more of its ceiling.
pub struct BarrelOffer {
    /// Position in the barrel list. Display order, and what `buy_barrel`
    /// takes — but **not an identity a save records**, because nothing about
    /// the barrel is saved.
    pub index: usize,
    pub def: &'static PieceDef,
    /// The catalogue's, like everything else. There is no barrel discount:
    /// these are cheap because they are *cheap components*, not because the
    /// barrel marks them down, and §C.3 stays trivially true.
    pub price: i32,
}

/// What is in the barrel, in order.
pub fn barrel(shops: &ShopsData) -> Vec<BarrelOffer> {
    shops
        .barrel
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            let def = &CATALOG[def_named(name)?];
            Some(BarrelOffer { index: i, def, price: def.price })
        })
        .collect()
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
