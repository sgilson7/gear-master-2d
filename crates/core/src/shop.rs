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
//! **Three tiers, and the barrel is what makes them possible.** Before there
//! was a floor under the shop, the shelf was the only place to buy anything at
//! all, so its prices had to be affordable or a player had no gear. With junk
//! available for pocket change, the shelf can go back to being what this file
//! says it is — the curated designed curve — and cost what a considered
//! purchase should:
//!
//! | tier | price |
//! |---|---|
//! | the barrel | the catalogue's, unmarked |
//! | the shelf | [`SHELF_PCT`] |
//! | a commission | [`COMMISSION_PCT`] |
//!
//! **This used to say there was no mark-up, and that §C.3 — *the screen shows
//! the price actually charged* — was therefore trivially true.** It is not
//! trivially true any more. What keeps it true is that there is exactly one
//! function per tier, the screen and `buy` both read it, and a test proves
//! they agree. M12.1 found this hole from the other end: adding fifty Fnorp to
//! every barrel offer passed seven checks, because not one of them read the
//! price actually charged.

use serde::{Deserialize, Serialize};

use crate::piece::{PieceDef, CATALOG};

/// What you start a run with.
///
/// You own a handle and a blade and nothing else, so this is the first upgrade
/// rather than pocket change. It buys two or three cheap components at the
/// starter town, which — with what the pit pays — is the opening.
pub const STARTING_GOLD: i32 = 28;

/// What a shelf charges, as a percentage of the catalogue's price.
///
/// **Set against the opening rather than by taste.** A starting purse is
/// [`STARTING_GOLD`] and the pit's cheapest shelf line is 3 Fnorp. At ×5 that
/// is 15, so a new character's first shop is *one considered piece and a
/// weapon out of the barrel* — 15 + 11 = 26 of 28 — and **not** two shelf
/// pieces, which is 30. That is the decision the tier exists to create, and
/// `the_opening_is_one_good_piece_or_a_frame_of_junk` is what holds it.
pub const SHELF_PCT: i32 = 500;

/// What ordering a piece costs, as a percentage of the catalogue's price.
///
/// **Twice the shelf.** `PLAN-M12.md` asks for a commission "priced above
/// shelf — ordering certainty costs more than finding luck"; this is that
/// sentence as a number. Used by M12.2.
pub const COMMISSION_PCT: i32 = 1000;

/// The catalogue's price scaled by a tier, in whole Fnorp.
///
/// Integer throughout, like every other number in this game that two machines
/// have to agree about, and **never below one**: a mark-up that rounded a
/// cheap thing down to free would make the dearest tier the cheapest.
pub fn at_pct(base: i32, pct: i32) -> i32 {
    (base * pct / 100).max(1)
}

/// What the shelf charges for this component.
///
/// **The one answer**, read by the screen and by `buy` alike. Two would be two
/// prices, and §C.3 is the rule that the one shown is the one taken.
pub fn shelf_price(def: &PieceDef) -> i32 {
    at_pct(def.price, SHELF_PCT)
}

/// What ordering this component costs.
pub fn commission_price(def: &PieceDef) -> i32 {
    at_pct(def.price, COMMISSION_PCT)
}

pub const FORMAT: &str = "gm2d-shops";
pub const VERSION: u32 = 1;

/// One line of a town's order book.
///
/// **Authored, so the power curve stays authored.** A commission is the
/// deterministic answer to "I want the thing, not the lottery", and the way it
/// stays an answer rather than a vending machine is that somebody wrote the
/// list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommissionDef {
    /// Canonical catalogue name.
    pub piece: String,
    /// How many fights it takes to arrive.
    ///
    /// **Per piece, in data**, because a rivet and a war frame should not take
    /// the same fortnight. `PLAN-M12.md` §3.
    pub fights: u16,
}

/// One town's shelf, in the order it is displayed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TownShelf {
    /// The place id from `data/tiles.json`.
    pub id: String,
    /// What this town will make to order.
    ///
    /// **Gated by being this town, and by nothing else.** The frame asks for
    /// commissions "gated the way the designed curve already gates — by level
    /// and by region reached", and that is what a town *is*: Kettleworks is
    /// behind a door and a crossing, and High Wick is on no map at all. A
    /// second gate written into this list would be a second copy of the
    /// world's shape, kept in a shop file.
    #[serde(default)]
    pub commissions: Vec<CommissionDef>,
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
        for t in &d.towns {
            for c in &t.commissions {
                let Some(i) = def_named(&c.piece) else {
                    return Err(format!(
                        "{} takes orders for {:?}, which is not in the catalogue",
                        t.id, c.piece
                    ));
                };
                if c.fights == 0 {
                    return Err(format!(
                        "{}'s order for {:?} arrives after no fights at all, which is a shelf",
                        t.id, c.piece
                    ));
                }
                if CATALOG[i].kind == crate::piece::PieceKind::Quest {
                    return Err(format!("{} takes orders for {:?}, which is carried and not worn",
                                       t.id, c.piece));
                }
                if crate::piece::EVENT_ONLY.contains(&CATALOG[i].name) {
                    return Err(format!(
                        "{} takes orders for {:?}, which is off a creature — a set you could \
                         order is not a set anybody earned",
                        t.id, c.piece
                    ));
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
    /// What is actually charged — [`shelf_price`], not the catalogue's.
    ///
    /// §C.3 lives here: this is the figure the screen prints *and* the figure
    /// `buy` takes, because there is one of it.
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
    /// The catalogue's, unmarked — **the barrel is the one tier that is not
    /// scaled**. These are cheap because they are cheap components, not
    /// because the barrel discounts them, which is why the floor stays put
    /// when the shelf's mark-up moves.
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

/// One line of a town's order book, priced.
pub struct CommissionOffer {
    /// Position in the town's order list. What `order` takes.
    pub index: usize,
    pub def: &'static PieceDef,
    /// [`commission_price`] — the dearest tier there is.
    pub price: i32,
    pub fights: u16,
}

/// What this town will make to order.
pub fn commissions(shops: &ShopsData, town: &str) -> Vec<CommissionOffer> {
    let Some(t) = shops.town(town) else { return Vec::new() };
    t.commissions
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let def = &CATALOG[def_named(&c.piece)?];
            Some(CommissionOffer { index: i, def, price: commission_price(def), fights: c.fights })
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
                price: shelf_price(def),
                sold: sold.iter().any(|(w, n)| w == town && *n as usize == i),
            })
        })
        .collect()
}
