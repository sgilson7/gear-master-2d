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
///
/// **Twenty-eight until the fivefold pass, and it had to move with it.** Every
/// cost in the game was multiplied by five and what a fight pays was not, so
/// money is scarcer everywhere it is earned — which is the point. The one
/// place that could not absorb it is the first afternoon: at 28 a beginner
/// could afford three of thirteen barrel lines and no helmet at all, and
/// `the_first_shop_can_finish_a_helmet` said so, which is the M4 soft-lock
/// guard doing exactly what it is for. So the purse moved with the prices and
/// the opening is the opening it always was; everything after it is dearer.
pub const STARTING_GOLD: i32 = 140;

/// What a shelf charges, as a percentage of the catalogue's price.
///
/// **Set against the opening rather than by taste.** A starting purse is
/// [`STARTING_GOLD`] and the pit's cheapest shelf line is the catalogue's 15.
/// At ×5 that is 75, so a new character's first shop is *one considered piece
/// and a weapon out of the barrel* — 75 + 55 = 130 of 140 — and **not** two
/// shelf pieces, which is 150. That is the decision the tier exists to create,
/// and `the_opening_is_one_good_piece_or_a_frame_of_junk` holds it. Both
/// numbers moved fivefold in the same pass and their ratio did not, which is
/// why that test needed no edit.
pub const SHELF_PCT: i32 = 500;

/// The dearest thing the barrel will hold, and the band an order comes from.
///
/// **One place, because these moved together and will again.** Every cost in
/// the game was multiplied by five in one pass; these were written as bare
/// numbers against the old scale and three separate checks failed on them.
/// Named here so the next time the economy is redenominated it is four edits
/// in one file rather than a hunt.
pub const BARREL_CEILING: i32 = 60;
/// What an order's book is drawn from: above the barrel, below the absurd.
pub const LEDGER_FLOOR: i32 = 65;
pub const LEDGER_CEILING: i32 = 200;

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

// ------------------------------------------------------------------- papers
//
// M13. Three papers on one counter, two of them refused when you first see
// them, and the refusal names what is in the way.

/// What a shelf wants before it will sell.
///
/// **The van at [4, 6] already gates its whole stock on a level**, through
/// `PlaceDef::hidden_until_level`; this gates one *line* of it, and on a fact
/// about the trees rather than about walking. One field, two readers, and the
/// **line stays drawn** — a refused entry is priced, described and greyed,
/// because a shelf you cannot see is not a shelf, it is a secret.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StockGate {
    /// Not before this level.
    Level(u32),
    /// Not before this many of your class trees are finished.
    TreesFinished(u8),
}

impl StockGate {
    pub fn met(self, level: u32, finished: u8) -> bool {
        match self {
            StockGate::Level(n) => level >= n,
            StockGate::TreesFinished(n) => finished >= n,
        }
    }

    /// Why not, with the count in it.
    ///
    /// **TONE rule 12: he can count.** A line that greys out saying only "not
    /// yet" reads as broken, and this project has written that sentence down
    /// four times — a button that greys with no reason is a button a player
    /// reports as a bug.
    pub fn refusal(self, level: u32, finished: u8, of: u8) -> String {
        match self {
            StockGate::Level(n) => format!("level {n}, and you are level {level}"),
            StockGate::TreesFinished(n) if of == 0 => {
                format!("{n} finished class trees, and you have no class yet")
            }
            StockGate::TreesFinished(n) => format!(
                "{n} finished class {}, and you have finished {finished} of the {of} you are",
                if n == 1 { "tree" } else { "trees" }
            ),
        }
    }
}

/// The three things on Spike's counter that are not enchs.
///
/// **All three are visible from the first time you walk in.** A locked line on
/// a shelf you can read is a goal; an absent line is a secret, and this game
/// has one shut door whose answer the player may already be carrying — the
/// Reach's frame — and that door opens rather than refusing for the same
/// reason.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Paper {
    /// The Kaklon Patent's licence, which the van has sold since M10.
    Patent,
    /// A second class, reopening the level-five fork with the four you did not
    /// take.
    Second,
    /// The expert your pair reaches. Free, because you have already paid twice.
    Expert,
}

impl Paper {
    pub const ALL: [Paper; 3] = [Paper::Patent, Paper::Second, Paper::Expert];

    pub fn id(self) -> &'static str {
        match self {
            Paper::Patent => "licence",
            Paper::Second => "second-paper",
            Paper::Expert => "expert-paper",
        }
    }

    /// **5,000 for the second class, matching the Patent.** The two cost the
    /// same on purpose: one buys a licence and one buys a class, and Spike
    /// does not price by what a thing is worth to you. The expert paper is
    /// free because the twenty-four points that reached it were the price.
    pub fn price(self) -> i32 {
        match self {
            Paper::Patent | Paper::Second => crate::ench::LICENCE_PRICE,
            Paper::Expert => 0,
        }
    }

    /// What has to be true before he will sell it.
    ///
    /// The Patent has never had one — it is the way round the fork for anybody
    /// whose class did not come with it, and gating that on anything would be
    /// gating the exception.
    ///
    /// **And the Second Paper has none either, since M15.4.** It wanted one
    /// finished tree and the human took that off in their own words: *"the
    /// second class should no longer be gated behind finishing the first one,
    /// instead you get it whenever you can afford the 2nd paper at spike
    /// kaklons van."* So it is gated by its price and by the level that puts
    /// the van on the road, and by nothing else.
    ///
    /// **The expert paper is not in that ask and keeps its gate.** Two finished
    /// trees is what makes it free — the twenty-four points *are* the price, and
    /// a free paper anybody can walk up to is not a paper, it is a fourth class
    /// on the fork. That is the whole of why the two lines are different, and
    /// it is why `StockGate::TreesFinished` still has a user rather than
    /// becoming a variant nothing constructs.
    pub fn gate(self) -> Option<StockGate> {
        match self {
            Paper::Patent | Paper::Second => None,
            Paper::Expert => Some(StockGate::TreesFinished(2)),
        }
    }
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
            // **On the map**, for the reason `on_a_shelf_you_can_reach` gives:
            // a shelf with no ground under it is a counter nobody has stood at.
            let placed = crate::data::towns_on_the_map();
            if let Some(t) = d
                .towns
                .iter()
                .filter(|t| placed.iter().any(|p| *p == t.id))
                .find(|t| t.stock.iter().any(|s| s == name))
            {
                return Err(format!(
                    "the barrel holds {name:?} and so does {}'s shelf; a shelf entry the barrel \
                     also carries is a shelf entry nobody takes",
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

// ------------------------------------------------------------------ rerolls

/// What a reroll costs, in Fnorp: `n * n` for the nth one.
///
/// **1, 4, 9, 16.** The first is loose change and the fourth is a decision,
/// which is the shape a cost curve wants: turning the barrel over once because
/// you did not like it is free enough to be a shrug, and doing it eight times
/// costs sixty-four and is a thing you thought about.
///
/// `n` is how many have already been paid for, so the first reroll is `n = 1`.
pub fn reroll_price(done: u32) -> i32 {
    let n = done as i64 + 1;
    (n * n).min(i32::MAX as i64) as i32
}

/// How many levels of grinding wipe the counters.
///
/// **Every tenth level, everywhere.** Without a reset the curve is a wall: by
/// level fifteen a reroll costs what a boss pays and the feature is one nobody
/// touches again. Ten levels is long enough that it is not a tap and short
/// enough that it comes back.
pub const REROLL_RESET_LEVELS: u32 = 10;

/// Which band of ten levels a character is in. The counters reset when it moves.
pub fn reroll_band(level: u32) -> u32 {
    level / REROLL_RESET_LEVELS
}

/// The two things that can be turned over, and they count separately.
///
/// **Per type, which is the whole of the ask**: rerolling the barrel eight
/// times must not make the ledger's first reroll cost eighty-one. They are
/// different appetites — one is *give me different junk* and the other is
/// *give me a different thing to want* — and a shared counter would price the
/// second by how impatient you were about the first.
pub const REROLL_BARREL: &str = "barrel";
pub const REROLL_LEDGER: &str = "ledger";

/// Everything the barrel is allowed to hold.
///
/// **The rules are the rules whoever rolled it.** A rerolled barrel has to pass
/// what the authored one passes — nothing off a creature, nothing a town has on
/// its shelf, nothing carried rather than worn, small and cheap — or a reroll
/// is a way to shake a set piece out of the catalogue.
pub fn barrel_pool() -> Vec<&'static PieceDef> {
    let on_a_shelf = on_a_shelf_you_can_reach();
    let paid = what_an_errand_pays();
    CATALOG
        .iter()
        .filter(|d| {
            d.price > 0
                && d.price <= BARREL_CEILING
                && d.cells.len() <= 4
                && d.kind != crate::piece::PieceKind::Quest
                && d.quest.is_none()
                && !crate::piece::EVENT_ONLY.contains(&d.name)
                && !on_a_shelf.iter().any(|n| n == d.name)
                && !paid.iter().any(|n| n == d.name)
        })
        .collect()
}

/// Everything an errand pays.
///
/// **A reward you could have bought makes the errand a slow way to shop**,
/// which this project wrote down when the errands were built and enforced for
/// the *shelf* only. `EVENT_ONLY` holds every set piece and every chain reward
/// off every counter; **eighteen ordinary errand rewards were never on that
/// list**, so the barrel could sell you the Warding Sigil an errand was about
/// to hand over, and the order book could take a commission for eleven more.
/// Found by nearly authoring it — the first draft of the barrel's spell line
/// *was* the Warding Sigil.
///
/// The cheap tiers are rolled, so this cannot be a hand-kept list: it is what
/// `quests.json` says it pays, read fresh.
fn what_an_errand_pays() -> Vec<String> {
    crate::data::quests().quests.iter().flat_map(|q| q.reward.clone()).collect()
}

/// Everything a town **on the map** has on its shelf.
///
/// **The exclusion is about undercutting, so it is about shelves a player can
/// walk up to.** `shops.json` carries a shelf for High Wick, which is on no
/// map — staged content, which this project allows and names in `avail.rs` —
/// and asking every shelf meant the barrel and the order book were both
/// stepping around a counter nobody has ever stood at. High Wick is the
/// *arcane* shelf: a book, an ink and three of the five spells the barrel could
/// otherwise afford.
///
/// So a shelf with no ground under it excludes nothing, and the day High Wick
/// is placed its stock leaves the cheap tiers on its own — which is this rule
/// working rather than a change to it.
fn on_a_shelf_you_can_reach() -> Vec<String> {
    let shops = crate::data::shops();
    let placed = crate::data::towns_on_the_map();
    shops
        .towns
        .iter()
        .filter(|t| placed.iter().any(|p| *p == t.id))
        .flat_map(|t| t.stock.iter().cloned())
        .collect()
}

/// One of each kind the recipes need, and as many of each as they ask for.
///
/// **Derived from `piece::recipes`, and it used to be a list.** The list said
/// Handle, Damaging, Frame, Plating, Base, Layer, Material, Mold, Material,
/// Mold, Ring, Accessory, Crest — thirteen kinds, and its comment said *one of
/// each kind the five recipes need*. There are **seven** recipes across the
/// five worn grids, because a weapon can be a blade, a book or a crystal ball,
/// and the list covered the blade. So the barrel could never hold a book, a
/// spell or an orb, a rerolled one never could either, and **a third of the
/// weapon grid's design space — a hundred and six components — was on no
/// counter in the game.**
///
/// A list of kinds written by hand is the failure this project has now found
/// five times: `package-web.sh`'s modules, the `EVENT_ONLY` regex twice, the
/// gate's own ceiling, Auto-pack's twenty-two names. **This one went stale the
/// moment the weapon grew a second way of being built**, which was before the
/// fork.
///
/// The counts are the recipe's own: a book wants one spell and a ball wants
/// **two**, so a barrel holding one spell is a barrel that cannot finish a
/// ball. Deduplicated by the largest ask, per kind, per grid.
pub fn barrel_wants() -> Vec<(crate::piece::PieceKind, crate::piece::SlotKind)> {
    let mut want: Vec<(crate::piece::PieceKind, crate::piece::SlotKind, usize)> = Vec::new();
    for &slot in crate::piece::SlotKind::ALL.iter() {
        for recipe in crate::piece::recipes(slot) {
            for &(kind, least, _) in recipe.iter() {
                if least == 0 {
                    continue;
                }
                match want.iter_mut().find(|(k, s, _)| *k == kind && *s == slot) {
                    // **The largest ask, not the sum.** One recipe wanting a
                    // spell and another wanting two is a grid that needs two,
                    // not three.
                    Some((_, _, n)) => *n = (*n).max(least),
                    None => want.push((kind, slot, least)),
                }
            }
        }
    }
    want.into_iter().flat_map(|(k, s, n)| std::iter::repeat_n((k, s), n)).collect()
}

/// The kinds a recipe will take and does not require.
///
/// **The barrel's extras, and they are rolled rather than listed** — an ink, an
/// alignment, a ring, a crest, an accessory. None of them finishes anything on
/// its own, which is what makes them the right thing to fill the last places
/// with: a barrel that is only cores is a barrel with no decisions in it.
pub fn barrel_extras() -> Vec<(crate::piece::PieceKind, crate::piece::SlotKind)> {
    let required = barrel_wants();
    let mut out: Vec<(crate::piece::PieceKind, crate::piece::SlotKind)> = Vec::new();
    for &slot in crate::piece::SlotKind::ALL.iter() {
        for recipe in crate::piece::recipes(slot) {
            for &(kind, least, _) in recipe.iter() {
                if least == 0 && !required.contains(&(kind, slot)) && !out.contains(&(kind, slot)) {
                    out.push((kind, slot));
                }
            }
        }
    }
    out
}

/// How many places the barrel has past its cores.
///
/// Two, which is what `the_barrel_is_stocked_and_small` leaves once every
/// recipe's cores are in: fourteen and sixteen. The barrel is the bargain bin,
/// and a bargain bin the size of a shop is a shop.
pub const BARREL_EXTRAS: usize = 2;

/// Roll a barrel: every recipe's cores, and a couple of the things a recipe
/// will take.
///
/// **Deterministic off the run's own stream**, like every other roll in this
/// game, so a seeded walk still replays and two machines agree about what the
/// barrel held.
pub fn roll_barrel(rng: &mut crate::rng::Rng) -> Vec<String> {
    let pool = barrel_pool();
    let mut out: Vec<String> = Vec::new();
    let take = |kind: crate::piece::PieceKind,
                slot: crate::piece::SlotKind,
                out: &mut Vec<String>,
                rng: &mut crate::rng::Rng| {
        let mut fits: Vec<&&PieceDef> = pool
            .iter()
            .filter(|d| d.kind == kind && d.fits(slot))
            .filter(|d| !out.iter().any(|n| n == d.name))
            .collect();
        if fits.is_empty() {
            return;
        }
        fits.sort_by_key(|d| d.name);
        let i = rng.below(fits.len());
        out.push(fits[i].name.to_string());
    };
    for (kind, slot) in barrel_wants() {
        take(kind, slot, &mut out, rng);
    }
    // And the extras, rolled from the kinds a recipe will take. **Which ones
    // is the roll's**, so turning the barrel over changes the fillers as well
    // as the cores — which is most of what a reroll is for.
    let mut extras = barrel_extras();
    for _ in 0..BARREL_EXTRAS {
        if extras.is_empty() {
            break;
        }
        let i = rng.below(extras.len());
        let (kind, slot) = extras.remove(i);
        take(kind, slot, &mut out, rng);
    }
    out
}

/// Everything the order book is allowed to hold.
///
/// Dearer and larger than the barrel's pool — an order is the thing you *chose*
/// and the tier above the shelf — and still nothing off a creature and nothing
/// a town already stocks.
pub fn ledger_pool() -> Vec<&'static PieceDef> {
    let shops = crate::data::shops();
    let on_a_shelf = on_a_shelf_you_can_reach();
    let paid = what_an_errand_pays();
    let in_barrel: Vec<&str> = shops.barrel.iter().map(|s| s.as_str()).collect();
    CATALOG
        .iter()
        .filter(|d| {
            (LEDGER_FLOOR..=LEDGER_CEILING).contains(&d.price)
                && d.cells.len() <= 6
                && d.kind != crate::piece::PieceKind::Quest
                && d.quest.is_none()
                && !crate::piece::EVENT_ONLY.contains(&d.name)
                && !on_a_shelf.iter().any(|n| n == d.name)
                && !paid.iter().any(|n| n == d.name)
                && !in_barrel.contains(&d.name)
        })
        .collect()
}

/// Roll a town's order book, keeping anything already on order.
///
/// **The one you are waiting for is not rerolled.** You paid for it and its
/// clock is running; turning the book over must not turn over the thing that
/// is already being made, or a reroll would be a way to lose an order you had
/// bought. Everything else on the counter changes.
pub fn roll_commissions(
    rng: &mut crate::rng::Rng,
    keep: Option<&str>,
    how_many: usize,
) -> Vec<CommissionDef> {
    let pool = ledger_pool();
    let mut out: Vec<CommissionDef> = Vec::new();
    if let Some(name) = keep {
        if let Some(i) = def_named(name) {
            // Its fights are re-derived from its price the same way any other
            // line's are, so the kept one is not a different kind of entry.
            out.push(CommissionDef { piece: CATALOG[i].name.to_string(), fights: fights_for(&CATALOG[i]) });
        }
    }
    while out.len() < how_many {
        let mut fits: Vec<&&PieceDef> =
            pool.iter().filter(|d| !out.iter().any(|c| c.piece == d.name)).collect();
        if fits.is_empty() {
            break;
        }
        fits.sort_by_key(|d| d.name);
        let i = rng.below(fits.len());
        out.push(CommissionDef { piece: fits[i].name.to_string(), fights: fights_for(fits[i]) });
    }
    out
}

/// How long a rolled order takes: dearer things take longer.
///
/// Authored lines say so in the file; a rolled one has nobody to say it, so it
/// is derived from the one number the piece already carries. Three fights at
/// the cheap end and ten at the dear.
fn fights_for(def: &PieceDef) -> u16 {
    (3 + (def.price as u16).saturating_sub(LEDGER_FLOOR as u16) / 20).clamp(3, 10)
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

/// The same, over a list of names rather than the shipped barrel.
pub fn barrel_of(names: &[String]) -> Vec<BarrelOffer> {
    names
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            let def = &CATALOG[def_named(name)?];
            Some(BarrelOffer { index: i, def, price: def.price })
        })
        .collect()
}

/// The same, over a rolled book rather than a town's authored one.
pub fn commissions_of(rolled: &[CommissionDef]) -> Vec<CommissionOffer> {
    rolled
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
