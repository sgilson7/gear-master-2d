//! Who the player is, and what is on their frames.
//!
//! # Why this is not `Run`
//!
//! Upstream's `Run` was two things welded together: a character with five
//! grids, and a campaign — a 49-rung ladder with a road, towns, a county,
//! dungeons and quests hung off it. Ninety fields, and more than half of them
//! were the second thing. GM2D wants the first and writes its own second, so
//! the weld is cut here rather than carried.
//!
//! What came across is everything that answers "what is on the board and is it
//! legal": the registry, the five grids, what you own, equip and unequip,
//! rotate, lock, undo, and the two derived readings combat runs on —
//! [`Character::player_stats`] and [`Character::combat_items`]. What did not is
//! every field whose meaning was "where on the ladder".
//!
//! Three things upstream tangled into these that are gone with it, and each
//! removal is a rule GM2D no longer has:
//!
//! - **`Phase`.** `Run` refused board edits outside `Phase::Loadout`, which is
//!   how it stopped you re-packing mid-fight. GM2D fights from a snapshot
//!   taken when the encounter opens, so the board is not live during a fight
//!   and has nothing to refuse. Whoever owns the encounter owns that rule.
//! - **Relics.** `player_stats` and `combat_items` both folded in a relic
//!   payout that read the whole run. Relics are campaign.
//! - **Classes.** A `Standing` class added its bonus to the character sheet
//!   here. GM2D's classes are chosen at level 5 and their trees are data
//!   (M5), so the fold happens where the tree is read, not in the base stats.
//!
//! Levels, XP and skill points are M4's and are not here yet. What is here is
//! the thing they will hang off.

use crate::loadout::{Loadout, LockedItem, SlotReport};
use crate::piece::{PieceId, PieceRegistry, SlotKind, CATALOG};
use crate::slot::{PlaceError, SLOT_W};
use crate::stats::Stats;

/// What a Sprocketman climbs out of the pit with, and where it sits.
///
/// Every component here is at most three cells tall, because the frames start
/// at three rows. A whole weapon, most of a helmet, and a pair of molds — no
/// chest, because a chest wants a base and a layer and there is no room for
/// both. The chest is the first thing a player buys into.
const STARTER: &[(&str, SlotKind, u8, u8, u8)] = &[
    ("Oak Handle", SlotKind::Weapon, 0, 0, 0),
    // **Turned, and it has to be.** An Iron Blade is one cell wide and four
    // tall, and a starting weapon frame is three rows: upright it does not fit
    // anywhere on the board, the weapon assembles nothing, and a character who
    // cannot win cannot earn. That is the M4 soft-lock exactly, and the reason
    // the fifth field of these rows exists.
    ("Iron Blade", SlotKind::Weapon, 1, 0, 1),
    ("Ruby Inlay", SlotKind::Weapon, 1, 1, 0),
    ("Balance Weight", SlotKind::Weapon, 2, 1, 0),
    ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
    ("Iron Plating", SlotKind::Helmet, 3, 0, 0),
    ("Visor of Focus", SlotKind::Helmet, 0, 2, 0),
    ("Leather Material", SlotKind::Gloves, 0, 0, 0),
    ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
    ("Runed Material", SlotKind::Greaves, 0, 0, 0),
    ("Greave Mold", SlotKind::Greaves, 2, 0, 0),
];

/// What a new character owns. **Two pieces, and they make one weapon.**
///
/// Everything else is bought or earned. The kit used to be eleven components —
/// most of a helmet, a pair of molds and a whole weapon — which meant the shop
/// was decoration for the first hour and the first quest was a formality. A
/// character now walks out of the pit with a blade on a stick and a reason to
/// go into town.
///
/// It still has to *win* in the pit, which is not a matter of taste:
/// `a_starting_character_can_win_in_the_pit` fights the region's whole roster
/// with exactly this and refuses a kit that beats none of it.
const STARTING_KIT: &[&str] = &["Oak Handle", "Iron Blade"];

/// How many board changes can be taken back.
pub const UNDO_DEPTH: usize = 40;

/// Why a board edit was refused.
///
/// Upstream's `RuleError` also carried `LoadoutLocked`, `NotEnoughGold`,
/// `NothingThere` and `TrayFull`. The first belonged to `Phase`, which is
/// gone; the other three belong to the shop and are raised there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    Place(PlaceError),
    NotEquipped,
    /// Tried to wear a quest item. They are carried and never worn.
    NotWearable,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleError::Place(e) => write!(f, "{e}"),
            RuleError::NotEquipped => write!(f, "that piece isn't equipped"),
            RuleError::NotWearable => {
                write!(f, "that is a quest item - it is carried, not worn")
            }
        }
    }
}

impl std::error::Error for RuleError {}

impl From<PlaceError> for RuleError {
    fn from(e: PlaceError) -> Self {
        RuleError::Place(e)
    }
}

/// One step of board history.
#[derive(Clone, Debug)]
pub(crate) struct BoardSnapshot {
    loadout: Loadout,
    registry: PieceRegistry,
    /// What you owned. Buying and selling are board changes too, and upstream's
    /// undo once restored the grids without them: sell a piece, undo, and the
    /// piece came back to the board while the money stayed in your pocket.
    owned: Vec<PieceId>,
    gold: i32,
    /// What the change was, so the interface can say what it undid.
    label: String,
}

/// The player: five grids, what is on them, and what is in the bag.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Character {
    pub registry: PieceRegistry,
    /// Every component this character has, worn or not.
    pub owned: Vec<PieceId>,
    pub loadout: Loadout,
    pub gold: i32,
    /// Maximum health earned outside the boards — the one stat a reward can
    /// add to the character rather than to a grid.
    pub grown_health: i32,
    /// Experience **spent on levels**, ever. **Not per level**: the level is
    /// derived from this, so the two cannot disagree, and a save carrying both
    /// would have a pair of numbers that could contradict each other.
    pub xp: i32,
    /// How worn out you are, in percent of maximum health.
    ///
    /// **The only thing a fight spends for good.** Health resets at the bell,
    /// so this is what makes a fourth fight in a row a different fight from
    /// the first. A town takes all of it off on arrival and a restorative
    /// takes some of it off wherever you are standing — the second is the
    /// decision on the road, the first is what makes the walk home worth
    /// taking. See `Game::arrive_in_town`.
    pub fatigue: i32,
    /// Restoratives carried, by id and count.
    pub supplies: Vec<(String, u32)>,
    /// Experience won and not yet spent.
    ///
    /// **At risk.** A town turns this into levels; a defeat takes all of it.
    /// It is a second number and it is not a second answer to the same
    /// question: `xp` is what you have become and this is what you are
    /// carrying, and the whole tension of the road is the gap between them.
    pub carried: i32,
    /// Points earned and not yet spent. One a level.
    pub skill_points: u32,
    /// Node ids taken, in the order they were taken.
    pub skills_taken: Vec<String>,
    /// Enchs owned and not bolted to anything.
    ///
    /// Ids into `data/enchs.json`, and a list rather than a set because two of
    /// the same ench are two things you can bolt to two components.
    #[serde(default)]
    pub enchs_owned: Vec<String>,
    /// Enchs bolted to a component, and whether each is switched on.
    ///
    /// The attachment names a `PieceId`, not a cell, so it survives a repack:
    /// pick the component up, turn it, move it to another grid, and the ench
    /// is still on it. See `ench.rs`.
    #[serde(default)]
    pub enchanted: Vec<crate::ench::Ench>,
    /// The class, by **canonical** name — `"Berserker"`, not `"Gorillathon"`.
    /// The theme renames it on the way to the screen, like every other name.
    ///
    /// `None` until level 5, and permanent after. See [`Character::choose_class`].
    pub class: Option<String>,
    /// **Not serialised.** Undo is a session's history of its own edits, not
    /// part of the character: a save that restored forty snapshots would be a
    /// save that let you undo your way back into a previous session's board.
    #[serde(skip)]
    pub(crate) undo_stack: Vec<BoardSnapshot>,
}

impl Default for Character {
    fn default() -> Self {
        Self::new()
    }
}

impl Character {
    /// A character with nothing on and nothing in the bag.
    ///
    /// `name_seed` is left at zero, which is a deliberate default and not an
    /// oversight: it is the seed every item name is hashed against, so a
    /// character that is going to be saved must have it set from the save's
    /// own seed. A board rebuilt without it keeps every stat and renames every
    /// item.
    pub fn new() -> Self {
        Character {
            registry: PieceRegistry::new(),
            owned: Vec::new(),
            loadout: Loadout::new(),
            gold: 0,
            grown_health: 0,
            xp: 0,
            carried: 0,
            fatigue: 0,
            enchs_owned: Vec::new(),
            enchanted: Vec::new(),
            supplies: Vec::new(),
            skill_points: 0,
            skills_taken: Vec::new(),
            class: None,
            undo_stack: Vec::new(),
        }
    }

    /// The same, with the name seed set. Prefer this everywhere a real
    /// character is made.
    pub fn seeded(name_seed: u64) -> Self {
        let mut c = Self::new();
        c.loadout.name_seed = name_seed;
        c
    }

    /// One of every wearable component in the catalogue, owned and unworn.
    ///
    /// A test fixture, and the reason the packing and assembly suites can ask
    /// questions about arbitrary arrangements without shopping for the parts
    /// first.
    pub fn with_all_pieces() -> Self {
        let mut c = Self::new();
        c.owned = (0..CATALOG.len())
            .filter(|&d| CATALOG[d].kind != crate::piece::PieceKind::Quest)
            .map(|d| c.registry.alloc(d))
            .collect();
        c
    }

    /// What a Sprocketman climbs out of the pit with.
    ///
    /// Scrap, and not much of it: two grips, an edge, a plate, a base and a
    /// mold. Enough to assemble one weapon badly and cover one other slot,
    /// which is the position the frame story puts you in and the position a
    /// shop is only interesting from. `with_all_pieces` owns the whole
    /// catalogue and is a test fixture, not a starting point.
    pub fn starting() -> Self {
        let mut c = Self::new();
        // Six by three, not the engine's six by eight. `Loadout::new` keeps the
        // full height because that is what a *creature* wears — `enemies.json`
        // seats gear as low as row 6 — and shrinking it globally would put
        // every monster in a frame it does not fit. The player grows into
        // theirs; that asymmetry is the early game.
        for k in SlotKind::ALL {
            *c.loadout.slot_mut(k) =
                crate::slot::Slot::with_rows(k, crate::progression::STARTING_ROWS);
        }
        c.gold = crate::shop::STARTING_GOLD;
        // **The preset's own components, and only the ones that fit in three
        // rows.** The kit and the auto-pack button have to agree or the button
        // seats nothing: the first version of this handed out a different set
        // of scrap, `apply_preset` found almost none of it, and a starting
        // character walked out of the pit wearing one glove and no weapon. It
        // lost every fight, and a loss pays nothing, so there was no way out of
        // the first region at all.
        //
        // A blade and something to hold it by, and nothing else at all.
        for name in STARTING_KIT {
            c.give(name);
        }
        c.forget_undo();
        c
    }

    /// [`Character::starting`], with the name seed set.
    pub fn starting_seeded(name_seed: u64) -> Self {
        let mut c = Self::starting();
        c.loadout.name_seed = name_seed;
        c
    }

    // ------------------------------------------------------------ the bag

    /// Everything owned that is not currently on a grid.
    pub fn inventory(&self) -> Vec<PieceId> {
        self.owned
            .iter()
            .copied()
            .filter(|id| self.loadout.slot_holding(*id).is_none())
            .collect()
    }

    pub fn is_equipped(&self, id: PieceId) -> bool {
        self.loadout.slot_holding(id).is_some()
    }

    /// First owned component with this catalogue name.
    pub fn find_by_name(&self, name: &str) -> Option<PieceId> {
        self.owned
            .iter()
            .copied()
            .find(|&id| self.registry.def(id).name == name)
    }

    /// Does this character own a component with this name?
    pub fn holds(&self, name: &str) -> bool {
        self.find_by_name(name).is_some()
    }

    /// Take ownership of one catalogue entry by name, returning its new id.
    pub fn give(&mut self, name: &str) -> Option<PieceId> {
        let def = CATALOG.iter().position(|d| d.name == name)?;
        self.remember(format!("taking {name}"));
        let id = self.registry.alloc(def);
        self.owned.push(id);
        Some(id)
    }

    // ------------------------------------------------------------ the board

    /// Can `id` be dropped into `kind` with its anchor at `(ax, ay)`?
    ///
    /// Pure query. The board UI calls this every frame while dragging so it
    /// can tint the preview, and must never work the answer out for itself —
    /// a fit preview that computes its own legality is a second rulebook.
    pub fn can_equip(
        &self,
        id: PieceId,
        kind: SlotKind,
        ax: u8,
        ay: u8,
    ) -> Result<(), RuleError> {
        // A quest item is carried, never worn. One place has to know this;
        // upstream let it be enforced by such items not being worth seating,
        // which is a rule nothing states and everything has to remember.
        if self.registry.def(id).kind == crate::piece::PieceKind::Quest {
            return Err(RuleError::NotWearable);
        }
        // A piece being moved within its own slot must not collide with
        // itself; `Slot::can_place` already allows that.
        Ok(self.loadout.can_place(&self.registry, id, kind, ax, ay)?)
    }

    /// Place `id` into `kind` at `(ax, ay)`, taking it out of wherever it was.
    pub fn equip(&mut self, id: PieceId, kind: SlotKind, ax: u8, ay: u8) -> Result<(), RuleError> {
        self.can_equip(id, kind, ax, ay)?;
        let moving = self.is_equipped(id);
        self.remember(format!(
            "{} {}",
            if moving { "moving" } else { "placing" },
            self.registry.def(id).name
        ));
        self.loadout.remove_anywhere(id);
        self.loadout.slot_mut(kind).place(&self.registry, id, ax, ay);
        Ok(())
    }

    /// Take `id` off and return it to the bag.
    pub fn unequip(&mut self, id: PieceId) -> Result<(), RuleError> {
        if !self.is_equipped(id) {
            return Err(RuleError::NotEquipped);
        }
        self.remember(format!("removing {}", self.registry.def(id).name));
        self.loadout.remove_anywhere(id);
        Ok(())
    }

    /// Rotate `id` a quarter turn clockwise.
    ///
    /// A piece already on a grid only turns if it still fits afterwards —
    /// otherwise the rotation is undone, so a rejected rotation leaves the
    /// world untouched and leaves no history either.
    pub fn rotate(&mut self, id: PieceId) -> Result<(), RuleError> {
        let before = self.registry.rotation(id);
        self.remember(format!("turning {}", self.registry.def(id).name));
        self.registry.rotate_cw(id);

        if let Some(kind) = self.loadout.slot_holding(id) {
            let anchor = self
                .loadout
                .slot(kind)
                .anchor_of(id)
                .expect("a held piece has an anchor");
            // Re-place from scratch: clear the old footprint, then test.
            self.loadout.remove_anywhere(id);
            match self.loadout.can_place(&self.registry, id, kind, anchor.0, anchor.1) {
                Ok(()) => {
                    self.loadout.slot_mut(kind).place(&self.registry, id, anchor.0, anchor.1);
                }
                Err(e) => {
                    self.registry.set_rotation(id, before);
                    self.loadout.slot_mut(kind).place(&self.registry, id, anchor.0, anchor.1);
                    self.undo_stack.pop();
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    /// Strip every grid and reset every rotation.
    pub fn clear_all(&mut self) {
        self.remember("clearing every slot");
        for kind in SlotKind::ALL {
            self.loadout.slot_mut(kind).clear();
        }
        let owned = self.owned.clone();
        for id in owned {
            self.registry.set_rotation(id, 0);
        }
    }

    /// Rows every grid has beyond the eight it started with.
    ///
    /// Read off the boards rather than tracked, so it cannot disagree with
    /// them — upstream kept a counter beside them and the two could drift.
    pub fn extra_rows(&self) -> u8 {
        SlotKind::ALL
            .iter()
            .map(|&k| self.loadout.slot(k).rows().saturating_sub(crate::slot::SLOT_H))
            .min()
            .unwrap_or(0)
    }

    // ------------------------------------------------------------ locks

    /// Lock the assembled item `piece` belongs to, or release it if it is
    /// already locked. Returns whether it is locked afterwards.
    ///
    /// A lock is state, not geometry: two pieces that touch are one item
    /// unless a lock says otherwise. Nothing derives these, and anything that
    /// restores a board — the save file included — has to restore them too.
    pub fn toggle_lock_item(&mut self, piece: PieceId) -> bool {
        if let Some(at) = self.loadout.locks.iter().position(|l| l.pieces.contains(&piece)) {
            self.remember("releasing an item");
            self.loadout.locks.remove(at);
            return false;
        }
        let Some(kind) = self.loadout.slot_holding(piece) else { return false };
        let Some(item) = self
            .report(kind)
            .items
            .into_iter()
            .find(|i| i.assembled && i.pieces.contains(&piece))
        else {
            return false;
        };
        self.remember("locking an item");
        let offsets = self.shape_of(kind, &item.pieces);
        self.loadout.locks.push(LockedItem { pieces: item.pieces, offsets });
        true
    }

    /// Where each of `pieces` sits relative to the group's top-left corner.
    fn shape_of(&self, kind: SlotKind, pieces: &[PieceId]) -> Vec<(u8, u8)> {
        let slot = self.loadout.slot(kind);
        let anchors: Vec<(u8, u8)> =
            pieces.iter().map(|&p| slot.anchor_of(p).unwrap_or((0, 0))).collect();
        let minx = anchors.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let miny = anchors.iter().map(|(_, y)| *y).min().unwrap_or(0);
        anchors.iter().map(|&(x, y)| (x - minx, y - miny)).collect()
    }

    pub fn locked_set(&self, piece: PieceId) -> Option<&[PieceId]> {
        self.loadout
            .locks
            .iter()
            .find(|l| l.pieces.contains(&piece))
            .map(|l| l.pieces.as_slice())
    }

    /// The pieces of a locked item and where each sits relative to the item's
    /// own top-left, so it can be carried and put back down as one shape.
    pub fn locked_shape(&self, piece: PieceId) -> Option<Vec<(PieceId, u8, u8)>> {
        let l = self.loadout.locks.iter().find(|l| l.pieces.contains(&piece))?;
        Some(
            l.pieces
                .iter()
                .zip(l.offsets.iter())
                .map(|(&p, &(dx, dy))| (p, dx, dy))
                .collect(),
        )
    }

    /// Is this piece part of a locked item?
    pub fn is_locked_item(&self, piece: PieceId) -> bool {
        self.locked_set(piece).is_some()
    }

    pub fn equip_locked_at(
        &mut self,
        piece: PieceId,
        kind: SlotKind,
        ax: u8,
        ay: u8,
    ) -> Result<(), RuleError> {
        let Some(shape) = self.locked_shape(piece) else {
            return Err(RuleError::NotEquipped);
        };
        // Every piece has to fit before any of them is placed, or a rejected
        // drop would leave the item scattered across the grid.
        for &(p, dx, dy) in &shape {
            let (x, y) = (ax as u32 + dx as u32, ay as u32 + dy as u32);
            // The slot's own height, not the tallest board's. Same shape of
            // fault as the one `branching-events.md` records: "anything
            // comparing against the constant is asking the wrong question",
            // and the constant grew into a per-board number that has now grown
            // into a per-slot one.
            if x >= SLOT_W as u32 || y >= self.loadout.slot(kind).rows() as u32 {
                return Err(RuleError::Place(PlaceError::OutOfBounds));
            }
            self.loadout.can_place(&self.registry, p, kind, x as u8, y as u8)?;
        }
        self.remember("placing a locked item");
        for &(p, dx, dy) in &shape {
            self.loadout.slot_mut(kind).place(&self.registry, p, ax + dx, ay + dy);
        }
        Ok(())
    }

    pub fn unequip_locked(&mut self, piece: PieceId) -> Result<(), RuleError> {
        let Some(set) = self.locked_set(piece).map(|s| s.to_vec()) else {
            return Err(RuleError::NotEquipped);
        };
        self.remember("removing a locked item");
        for p in set {
            self.loadout.remove_anywhere(p);
        }
        Ok(())
    }

    pub fn rotate_locked(&mut self, piece: PieceId) -> Result<(), RuleError> {
        let Some(set) = self.locked_set(piece).map(|s| s.to_vec()) else {
            return Err(RuleError::NotEquipped);
        };
        let Some(kind) = self.loadout.slot_holding(piece) else {
            return Err(RuleError::NotEquipped);
        };

        let slot = self.loadout.slot(kind);
        let cells: Vec<(PieceId, Vec<(u8, u8)>)> =
            set.iter().map(|&p| (p, slot.cells_of(p))).collect();
        let minx = cells.iter().flat_map(|(_, c)| c.iter().map(|(x, _)| *x)).min().unwrap_or(0);
        let miny = cells.iter().flat_map(|(_, c)| c.iter().map(|(_, y)| *y)).min().unwrap_or(0);
        let maxy = cells.iter().flat_map(|(_, c)| c.iter().map(|(_, y)| *y)).max().unwrap_or(0);
        let height = maxy - miny + 1;

        // Where each piece's own footprint lands once the item has turned.
        let mut want: Vec<(PieceId, u8, u8)> = Vec::new();
        for (p, cs) in &cells {
            let turned: Vec<(u8, u8)> = cs
                .iter()
                .map(|&(x, y)| (minx + (height - 1 - (y - miny)), miny + (x - minx)))
                .collect();
            let ax = turned.iter().map(|(x, _)| *x).min().unwrap_or(0);
            let ay = turned.iter().map(|(_, y)| *y).min().unwrap_or(0);
            want.push((*p, ax, ay));
        }

        self.remember("turning a locked item");
        let before: Vec<(PieceId, u8, u8, u8)> = cells
            .iter()
            .map(|(p, _)| {
                let a = self.loadout.slot(kind).anchor_of(*p).unwrap_or((0, 0));
                (*p, a.0, a.1, self.registry.rotation(*p))
            })
            .collect();

        for &(p, ..) in &before {
            self.loadout.slot_mut(kind).remove(p);
            self.registry.rotate_cw(p);
        }
        let mut ok = true;
        for &(p, ax, ay) in &want {
            if self.loadout.can_place(&self.registry, p, kind, ax, ay).is_ok() {
                self.loadout.slot_mut(kind).place(&self.registry, p, ax, ay);
            } else {
                ok = false;
                break;
            }
        }
        if !ok {
            for &(p, ax, ay, rot) in &before {
                self.loadout.slot_mut(kind).remove(p);
                self.registry.set_rotation(p, rot);
                self.loadout.slot_mut(kind).place(&self.registry, p, ax, ay);
            }
            self.undo_stack.pop();
            return Err(RuleError::Place(PlaceError::OutOfBounds));
        }
        // The item has a new shape now, and the stored one is what puts it back
        // down if it is lifted into the inventory.
        let offsets = self.shape_of(kind, &set);
        if let Some(l) = self.loadout.locks.iter_mut().find(|l| l.pieces.contains(&piece)) {
            l.offsets = offsets;
        }
        Ok(())
    }

    pub fn inventory_groups(&self) -> Vec<Vec<PieceId>> {
        let loose = self.inventory();
        let mut out: Vec<Vec<PieceId>> = Vec::new();
        let mut taken: Vec<PieceId> = Vec::new();
        for &id in &loose {
            if taken.contains(&id) {
                continue;
            }
            match self.locked_set(id) {
                Some(set) if set.iter().all(|p| loose.contains(p)) => {
                    taken.extend(set.iter().copied());
                    out.push(set.to_vec());
                }
                _ => out.push(vec![id]),
            }
        }
        out
    }

    // ------------------------------------------------------------ undo

    fn remember(&mut self, what: impl Into<String>) {
        self.undo_stack.push(BoardSnapshot {
            loadout: self.loadout.clone(),
            registry: self.registry.clone(),
            owned: self.owned.clone(),
            gold: self.gold,
            label: what.into(),
        });
        if self.undo_stack.len() > UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// Step the board back one change, returning what was undone.
    pub fn undo(&mut self) -> Option<String> {
        let snap = self.undo_stack.pop()?;
        self.loadout = snap.loadout;
        self.registry = snap.registry;
        self.owned = snap.owned;
        self.gold = snap.gold;
        Some(snap.label)
    }

    /// What the next undo would take back, if anything.
    pub fn undoable(&self) -> Option<&str> {
        self.undo_stack.last().map(|s| s.label.as_str())
    }

    /// Drop the history, for when the board stops being the one the history
    /// describes.
    pub fn forget_undo(&mut self) {
        self.undo_stack.clear();
    }

    // ------------------------------------------------------------ levels

    /// The level this character's **spent** experience buys.
    ///
    /// Derived, never stored. A level and a total that could disagree is a
    /// save with two answers to the same question — and note the number it is
    /// derived from is `xp`, which is what has been *banked*. What you are
    /// carrying does not count until you have taken it somewhere safe.
    pub fn level(&self) -> u32 {
        crate::progression::level_for(self.xp)
    }

    /// Win experience. It goes in your pocket, not into a level.
    ///
    /// **This is the whole of the souls rule.** Experience is carried out of a
    /// fight and is worth nothing until a town turns it into levels; a defeat
    /// takes all of it. Nothing on the road calls [`gain_xp`](Self::gain_xp) —
    /// the road calls this, and only [`bank`](Self::bank) crosses a level.
    pub fn carry(&mut self, by: i32) {
        self.carried = (self.carried + by.max(0)).max(0);
    }

    /// Turn everything carried into levels. Only a town calls this.
    ///
    /// Returns the levels crossed, same as `gain_xp`, so the receipt a town
    /// prints is the receipt a fight used to.
    pub fn bank(&mut self) -> Vec<u32> {
        let held = std::mem::take(&mut self.carried);
        self.gain_xp(held)
    }

    /// Lose what has not been banked. A defeat, and nothing else.
    ///
    /// Returns what was lost, because a receipt that does not name the number
    /// is a receipt nobody believes.
    pub fn drop_carried(&mut self) -> i32 {
        std::mem::take(&mut self.carried)
    }

    /// Spend experience on levels, returning every level crossed.
    ///
    /// Returns the levels rather than just the new one, because a single
    /// banking can cross two and a screen that only said "you reached 7" would
    /// have swallowed a skill point and a row.
    pub fn gain_xp(&mut self, by: i32) -> Vec<u32> {
        if by <= 0 {
            self.xp = (self.xp + by).max(0);
            return Vec::new();
        }
        let before = self.level();
        self.xp += by;
        let after = self.level();
        let crossed: Vec<u32> = (before + 1..=after).collect();
        self.skill_points += crossed.len() as u32;
        crossed
    }

    /// Rebuild every grid to the height this level and these skills imply.
    ///
    /// **Grows only.** Called after a level-up and after a skill is taken, and
    /// it must never shrink a board: a board that got shorter would drop
    /// whatever was seated in the rows it lost, silently, and the player would
    /// find out in a fight.
    pub fn resize_boards(&mut self, granted: [u8; 5]) -> Vec<(SlotKind, u8)> {
        let level = self.level();
        let mut grew = Vec::new();
        for k in SlotKind::ALL {
            let want = crate::progression::board_rows(k, level, granted[k.index()]);
            let have = self.loadout.slot(k).rows();
            if want > have {
                self.loadout.grow_one(k, want - have);
                grew.push((k, want - have));
            }
        }
        grew
    }

    /// Spend a point on a node.
    ///
    /// Every refusal comes from `SkillsData::can_take`, which is the one place
    /// the three rules live — bought twice, without its prerequisite, without a
    /// point. Taking a node re-applies its effects immediately: rows appear,
    /// stats appear, and the assembly percentage moves, so a player sees what
    /// they bought rather than what they will have after the next fight.
    pub fn take_skill(
        &mut self,
        tree: &crate::skills::SkillsData,
        id: &str,
    ) -> Result<(), crate::skills::Refusal> {
        let cost = tree
            .can_take(id, &self.skills_taken, self.skill_points, self.class.as_deref())?
            .cost;
        self.skill_points -= cost;
        self.skills_taken.push(id.to_string());
        self.apply_skills(tree);
        Ok(())
    }

    /// Re-derive everything the taken nodes imply.
    ///
    /// Idempotent, and called after loading as well as after buying: the save
    /// carries which nodes were taken, not what they did, so what they did is
    /// worked out from the tree every time. A save that stored the consequences
    /// would go stale the first time a node was retuned.
    pub fn apply_skills(&mut self, tree: &crate::skills::SkillsData) {
        self.loadout.assembly_pct = tree.assembly_pct(&self.skills_taken);
        let granted = tree.granted_rows(&self.skills_taken);
        self.resize_boards(granted);
    }

    /// The level a class may be chosen at.
    pub const CLASS_AT: u32 = 5;

    /// Is the character owed a class choice?
    ///
    /// True at level 5 and every level after, until one is taken. A save made
    /// before level 5 loads and is still asked; a save made at level 9 with no
    /// class is still asked, because the question was never answered rather
    /// than declined.
    pub fn owed_a_class(&self) -> bool {
        self.class.is_none() && self.level() >= Self::CLASS_AT
    }

    /// Take a class. **Permanent within a save.**
    ///
    /// There is no path that clears one — no `clear_class`, no `None` write
    /// anywhere but construction — and `a_class_is_permanent` is what says so.
    /// The plan makes this a decision rather than a loadout, and a decision you
    /// can take back is not one.
    pub fn choose_class(&mut self, canonical: &str) -> Result<&'static crate::class::ClassDef, String> {
        if let Some(have) = &self.class {
            return Err(format!("you are already a {have}, and that does not come off"));
        }
        if self.level() < Self::CLASS_AT {
            return Err(format!(
                "a class is chosen at level {}, and you are level {}",
                Self::CLASS_AT,
                self.level()
            ));
        }
        let def = crate::class::CLASSES
            .iter()
            .find(|c| c.name == canonical)
            .ok_or_else(|| format!("there is no such class as {canonical}"))?;
        self.class = Some(def.name.to_string());
        Ok(def)
    }

    /// The chosen class, resolved.
    pub fn class_def(&self) -> Option<&'static crate::class::ClassDef> {
        let name = self.class.as_ref()?;
        crate::class::CLASSES.iter().find(|c| c.name == *name)
    }

    // ------------------------------------------------------------ growth

    /// Give every grid another row.
    pub fn grow_boards(&mut self, by: u8) {
        self.loadout.grow(by);
    }

    /// Give one grid another row. GM2D's level-up rotation calls this; upstream
    /// only had it behind a granted-row counter.
    pub fn grow_slot(&mut self, kind: SlotKind, by: u8) {
        self.loadout.grow_one(kind, by);
    }

    /// How many rows each grid has, indexed by `SlotKind::index`.
    ///
    /// Read off the boards rather than tracked, so it cannot disagree with
    /// them.
    pub fn slot_rows(&self) -> [u8; 5] {
        let mut out = [0u8; 5];
        for k in SlotKind::ALL {
            out[k.index()] = self.loadout.slot(k).rows();
        }
        out
    }

    /// A complete, legal loadout that assembles all five slots and lights
    /// every assembly bonus.
    ///
    /// The auto-build button and the test fixture at once, which is how
    /// upstream kept them from drifting: a demo arrangement nobody exercises
    /// stops being a demo of anything. Deliberately shows off the mechanics
    /// rather than maxing the numbers — chest, gloves and greaves each carry
    /// two separate finished items, the weapon's Runed Edge doubles the Ruby
    /// Inlay beside it, and the Hollow Weave sits in open space where its
    /// empty-cell bonus counts.
    ///
    /// **Seats only what is owned.** An earlier version handed out any missing
    /// component, which made the auto-build button a way to get the whole
    /// preset for nothing and made the shop pointless. A character who owns
    /// scrap gets their scrap arranged well; a character who owns everything —
    /// `with_all_pieces`, which is what the tests use — gets the whole preset,
    /// so nothing about the fixtures changed.
    pub fn apply_preset(&mut self) {
        // Six by eight or six by three: two layouts, because an arrangement is
        // only an arrangement of a particular board.
        //
        // This is the bug M4 shipped and then found. `PRESET` is an eight-row
        // arrangement, and `Balanced Grip` is one cell wide and **four tall** —
        // on a three-row frame it does not fit, so a starting character seated
        // an edge, an inlay and a weight with no handle under them, assembled
        // nothing, and lost every fight. A loss pays nothing, so there was no
        // way out of the first region at all: the game was unwinnable from its
        // own first tile. `a_starting_character_can_win_in_the_pit` is the test
        // that would have caught it, and now does.
        self.pack_what_you_own();
    }

    /// Seat everything you own that will go on, biggest first.
    ///
    /// # The bug this replaces, which the M8.8 playthrough found
    ///
    /// Auto-pack used to seat a **fixed list of twenty-two component names**,
    /// skipping anything not owned. Five of the eleven things the only town on
    /// the map sells are not on that list — `Tin Frame`, `Tin Plating`,
    /// `Sackcloth Base`, `Rag Layer`, `Plain Sole` — so a player who bought the
    /// whole shelf and pressed the button they were given watched most of it
    /// stay in the bag. With every component the map can hand out, the board
    /// came to **two assembled items of five frames**, and two items lose to
    /// the Cave's boss, so the key never dropped and the door never appeared.
    /// The suite was green and the demo could not be finished.
    ///
    /// The old list was not wrong when it was written; it was written against a
    /// starting kit of eleven components and outlived it by three milestones.
    /// A list of names is a second copy of what the shops sell, and the two
    /// drifted the moment the shelves became content.
    ///
    /// # How it packs
    ///
    /// Greedy and deterministic: biggest first, into the first legal cell,
    /// trying each of the four turns. It is not an optimiser and is not meant
    /// to be — the whole game is the arrangement, and a button that packed
    /// perfectly would be a button that played for you. What it has to do is
    /// leave nothing in the bag that would obviously have gone on.
    ///
    /// Order is fixed by area and then by id, so the same bag always produces
    /// the same board. A seeded walk that repacked differently on two machines
    /// would be a seeded walk that fought different fights.
    pub fn pack_what_you_own(&mut self) {
        for k in SlotKind::ALL {
            self.loadout.slot_mut(k).clear();
        }
        // **The weapon first.** It is the only thing that swings, and a
        // component that fits several grids is worth more in it than anywhere
        // else. Then the chest, which wants the most room, and the three
        // smaller grids after.
        const ORDER: [SlotKind; 5] = [
            SlotKind::Weapon,
            SlotKind::Chest,
            SlotKind::Helmet,
            SlotKind::Gloves,
            SlotKind::Greaves,
        ];
        for kind in ORDER {
            self.pack_one(kind);
        }
    }

    /// How good a grid is: how many items came together, then what they rate.
    ///
    /// The count first and the rating second, because one assembled item beats
    /// any amount of loose metal — an item that did not come together does
    /// nothing at all, whatever its parts are worth.
    fn worth_of(r: &SlotReport) -> (usize, i32) {
        let made = r.items.iter().filter(|i| i.assembled).count();
        let worth = r.items.iter().filter(|i| i.assembled).map(|i| i.rating).sum();
        (made, worth)
    }

    /// Pack one grid: seed on a core, grow what improves, repeat.
    ///
    /// # The bug this exists because of
    ///
    /// The first version filled the grid with everything that fitted, and
    /// **packing more is not packing better**: two components that touch are
    /// one item, so a seven-row weapon frame packed solid is one enormous
    /// group that assembles nothing at all. A character carrying every reward
    /// on the map lost to a Cave Rat, because the button had given them a
    /// weapon frame full of books and no weapon.
    ///
    /// # The rule
    ///
    /// A **core** is seeded where it touches nothing, because a recipe is
    /// built round one. Everything after it has to *strictly improve* the grid
    /// — one more item, or the same items worth more — and is taken straight
    /// back out otherwise. Then another core is seeded somewhere clear and the
    /// pass runs again, up to four items a grid.
    ///
    /// It is not an optimiser and must not become one: **the whole game is the
    /// arrangement**, and a button that packed perfectly would be a button that
    /// played for you. What it has to do is leave nothing obvious in the bag
    /// and never hand back a board worse than the one you could have made by
    /// putting the first two things down.
    fn pack_one(&mut self, kind: SlotKind) {
        // A core that made nothing is taken back out. **Seeding is
        // provisional**: a lone core on a grid is a component doing nothing in
        // a cell somebody else could have used, and the first version left one
        // behind on every grid with an odd part in the bag.
        let mut skip: Vec<PieceId> = Vec::new();
        for _ in 0..8 {
            let before = Self::worth_of(&self.report(kind));
            let keep_slot = self.loadout.slot(kind).clone();
            let keep_rots: Vec<(PieceId, u8)> = self
                .loose_for(kind)
                .into_iter()
                .map(|id| (id, self.registry.rotation(id)))
                .collect();

            let Some(seed) = self.seed_a_core(kind, &skip) else { break };
            self.grow(kind);
            if Self::worth_of(&self.report(kind)).0 > before.0 {
                continue;
            }
            // That core led nowhere. Put the grid back the way it was and try
            // the next one down.
            for (id, r) in keep_rots {
                self.registry.set_rotation(id, r);
            }
            *self.loadout.slot_mut(kind) = keep_slot;
            skip.push(seed);
        }
    }

    /// Everything owned, loose, and allowed on this grid — best rated first.
    fn loose_for(&self, kind: SlotKind) -> Vec<PieceId> {
        let mut out: Vec<PieceId> = self
            .owned
            .iter()
            .copied()
            .filter(|&id| !self.is_equipped(id))
            .filter(|&id| {
                let d = self.registry.def(id);
                // A quest item is carried, never worn — `equip` refuses one,
                // and skipping it here saves a bag full of keys a great many
                // failed placements a repack.
                d.kind != crate::piece::PieceKind::Quest && d.slots().contains(&kind)
            })
            .collect();
        // Ties broken by id, so the same bag always packs the same way. A
        // seeded walk that repacked differently on two machines would be a
        // seeded walk that fought different fights.
        out.sort_by_key(|&id| {
            (std::cmp::Reverse(crate::rating::piece_rating(self.registry.def(id))), id.0)
        });
        out
    }

    /// Put the best unused core down somewhere it touches nothing.
    ///
    /// Touching nothing is what makes it the start of a new item rather than
    /// another part of the one already there.
    fn seed_a_core(&mut self, kind: SlotKind, skip: &[PieceId]) -> Option<PieceId> {
        for id in self.loose_for(kind) {
            if !self.registry.def(id).kind.is_core() || skip.contains(&id) {
                continue;
            }
            let was = self.registry.rotation(id);
            for turn in 0..4u8 {
                self.registry.set_rotation(id, (was + turn) % 4);
                for y in 0..self.loadout.slot(kind).rows() {
                    for x in 0..SLOT_W {
                        if self.loadout.can_place(&self.registry, id, kind, x, y).is_err() {
                            continue;
                        }
                        if !self.stands_clear(kind, id, x, y) {
                            continue;
                        }
                        self.loadout.slot_mut(kind).place(&self.registry, id, x, y);
                        return Some(id);
                    }
                }
            }
            self.registry.set_rotation(id, was);
        }
        None
    }

    /// Would this piece, seated here, touch nothing already on the grid?
    fn stands_clear(&self, kind: SlotKind, id: PieceId, ax: u8, ay: u8) -> bool {
        let slot = self.loadout.slot(kind);
        self.registry.shape(id).cells().iter().all(|&(dx, dy)| {
            let (cx, cy) = (ax as i32 + dx as i32, ay as i32 + dy as i32);
            [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().all(|&(ox, oy)| {
                let (nx, ny) = (cx + ox, cy + oy);
                !slot.in_bounds(nx, ny) || slot.get(nx as u8, ny as u8).is_none()
            })
        })
    }

    /// Add anything that makes the grid strictly better, and nothing else.
    ///
    /// Only anchors where the piece would *touch* something already down are
    /// tried: a component standing on its own changes no item, so it cannot
    /// improve one, and trying every empty cell of a six-by-eight frame for
    /// every piece is how a button becomes a pause.
    fn grow(&mut self, kind: SlotKind) -> bool {
        let mut any = false;
        for _ in 0..2 {
            let mut moved = false;
            for id in self.loose_for(kind) {
                let before = Self::worth_of(&self.report(kind));
                let was = self.registry.rotation(id);
                let mut placed = false;
                'turns: for turn in 0..4u8 {
                    self.registry.set_rotation(id, (was + turn) % 4);
                    for y in 0..self.loadout.slot(kind).rows() {
                        for x in 0..SLOT_W {
                            if self.loadout.can_place(&self.registry, id, kind, x, y).is_err() {
                                continue;
                            }
                            if self.stands_clear(kind, id, x, y) {
                                continue;
                            }
                            self.loadout.slot_mut(kind).place(&self.registry, id, x, y);
                            if Self::worth_of(&self.report(kind)) > before {
                                placed = true;
                                break 'turns;
                            }
                            self.loadout.slot_mut(kind).remove(id);
                        }
                    }
                }
                if placed {
                    moved = true;
                    any = true;
                } else {
                    self.registry.set_rotation(id, was);
                }
            }
            if !moved {
                break;
            }
        }
        any
    }

    /// Clear every grid and seat a layout, skipping anything not owned or not
    /// fitting. Shared by both arrangements so they cannot diverge in how they
    /// are applied.
    fn seat(&mut self, layout: &[(&str, SlotKind, u8, u8, u8)]) {
        for k in SlotKind::ALL {
            self.loadout.slot_mut(k).clear();
        }
        for &(name, kind, ax, ay, rot) in layout {
            let Some(id) = self.find_by_name(name) else { continue };
            self.registry.set_rotation(id, rot);
            self.loadout.remove_anywhere(id);
            if self.loadout.can_place(&self.registry, id, kind, ax, ay).is_ok() {
                self.loadout.slot_mut(kind).place(&self.registry, id, ax, ay);
            }
        }
    }

    // ------------------------------------------------------------ readings

    pub fn reports(&self) -> Vec<SlotReport> {
        self.loadout.reports(&self.registry)
    }

    pub fn report(&self, kind: SlotKind) -> SlotReport {
        self.loadout.report(&self.registry, kind)
    }

    /// Every grid's contribution, plus health earned off the boards, plus
    /// whatever the skill tree adds.
    ///
    /// The tree is read here rather than folded in when a node is bought,
    /// because a bought node's *effect* is not state — the node is. Reading it
    /// every time means retuning a node retunes every save that took it.
    pub fn player_stats(&self) -> Stats {
        let mut base = self.rested_stats();
        // **Last, and on the total.** Fatigue is a share of the maximum you
        // would otherwise have, so it has to be applied after everything that
        // adds to it — a board, the grown health and the tree. Taking it off
        // the base and then adding gear back would make a helmet cure
        // tiredness.
        base.health = crate::fatigue::worn(base.health, self.fatigue);
        base
    }

    /// The same, as if you had just got up.
    ///
    /// What the character *is*, before the road is taken off it. The sheet
    /// shows both, because "160, and 24 of it is missing" is two facts and a
    /// player needs the pair to decide whether to turn round.
    pub fn rested_stats(&self) -> Stats {
        let mut base = self.loadout.total_stats(&self.registry);
        base.health += self.grown_health;
        if !self.skills_taken.is_empty() {
            base += crate::data::skills().stats_from(&self.skills_taken);
        }
        base
    }

    /// One fight's wear.
    pub fn tire(&mut self, by: i32) {
        self.fatigue = (self.fatigue + by).clamp(0, crate::fatigue::CAP);
    }

    /// How many of a restorative are in the pack.
    pub fn supply_count(&self, id: &str) -> u32 {
        self.supplies.iter().find(|(s, _)| s == id).map(|(_, n)| *n).unwrap_or(0)
    }

    pub fn give_supply(&mut self, id: &str, n: u32) {
        match self.supplies.iter_mut().find(|(s, _)| s == id) {
            Some((_, have)) => *have += n,
            None => self.supplies.push((id.to_string(), n)),
        }
    }

    /// Drink one. Returns how much tiredness actually came off, or why not.
    ///
    /// Reports what was *used* rather than what the tin claims, because
    /// drinking a sixty against twelve points of tiredness is a thing a player
    /// should be told they did.
    pub fn use_supply(&mut self, id: &str) -> Result<i32, String> {
        let supplies = crate::data::supplies();
        let Some(def) = supplies.get(id) else { return Err("there is no such thing".into()) };
        if self.supply_count(id) == 0 {
            return Err(format!("You have no {}.", def.name));
        }
        if self.fatigue == 0 {
            return Err("You are not tired.".into());
        }
        let took = def.restores.min(self.fatigue);
        self.fatigue -= took;
        for (s, n) in self.supplies.iter_mut() {
            if s == id {
                *n -= 1;
            }
        }
        self.supplies.retain(|(_, n)| *n > 0);
        Ok(took)
    }

    /// The board's shape, as the class ranker reads it.
    ///
    /// Upstream hung class *acquisition* off this — fountains, axis
    /// thresholds, a ranking. GM2D chooses a class at level 5 from a tree in
    /// data (M5), so what survives here is the reading itself, which is a
    /// property of the board and belongs with the board.
    pub fn fingerprint(&self) -> crate::class::Fingerprint {
        let filled: usize = SlotKind::ALL
            .iter()
            .map(|&k| {
                let slot = self.loadout.slot(k);
                slot.pieces().iter().map(|&p| slot.cells_of(p).len()).sum::<usize>()
            })
            .sum();
        crate::class::Fingerprint::of(&self.registry, &self.combat_items(), filled)
    }

    /// Activation profiles for every assembled item — what combat runs on.
    /// Armour and mana the skill tree says you begin a fight already holding.
    ///
    /// Read every time rather than banked when the node is bought, for the same
    /// reason `player_stats` reads the tree: a bought node's *effect* is not
    /// state, the node is.
    pub fn start_with(&self) -> crate::combat::Held {
        let mut held = if self.skills_taken.is_empty() {
            crate::combat::Held::default()
        } else {
            crate::data::skills().start_with(&self.skills_taken)
        };
        // **And every rule an assembled item grants.** Not folded into the
        // tree's answer, because the tree does not know about the board; added
        // here, because `Held` is what a fight is handed and this is the one
        // place that has both.
        held.rules.extend(self.item_rules());
        held
    }

    /// Every rule this character has, from wherever it came.
    ///
    /// The tree's plus every rule granted by an item that is **currently
    /// assembled**. One list, read fresh every time it is asked, for the same
    /// reason a node's effect is read fresh rather than banked when the point
    /// is spent: a bought node's effect is not state, and neither is a seated
    /// item's.
    ///
    /// It is on `Character` and not on `Loadout` for the reason enchs are: a
    /// loadout that knew about granted rules would be a loadout that knew about
    /// a skill tree.
    pub fn rules(&self) -> Vec<crate::rule::Rule> {
        let mut out = if self.skills_taken.is_empty() {
            Vec::new()
        } else {
            crate::data::skills().rules_from(&self.skills_taken)
        };
        out.extend(self.item_rules());
        out
    }

    /// The board's half of [`rules`](Self::rules).
    ///
    /// **Assembled items only**, and that is not a check written anywhere: a
    /// `SlotReport` says which groups came together and this walks those. "An
    /// unassembled set grants nothing" is the shape of the loop rather than a
    /// condition inside it.
    fn item_rules(&self) -> Vec<crate::rule::Rule> {
        let mut out = Vec::new();
        for report in self.loadout.reports(&self.registry) {
            for item in report.items.iter().filter(|i| i.assembled) {
                for &p in &item.pieces {
                    let Some(b) = self.registry.def(p).assembly_bonus else { continue };
                    out.extend(b.grants.iter().cloned());
                }
            }
        }
        out
    }

    /// What a step is allowed to do that it would otherwise refuse.
    ///
    /// The caller's half of *a map does not know about bags*: `world::step`
    /// takes this, and this is filled from what the character has.
    pub fn allowances(&self) -> crate::world::Allowances {
        crate::world::Allowances::of(&self.rules())
    }

    /// Can this character read the map's numbers?
    ///
    /// Core's answer, so the shim can decide nothing: a screen that worked out
    /// for itself whether to print the danger would be a second copy of the
    /// rule, and it would go on printing it after the node was retuned.
    pub fn scouting(&self) -> bool {
        self.rules().iter().any(|r| matches!(r, crate::rule::Rule::Scout))
    }

    pub fn combat_items(&self) -> Vec<crate::loadout::ItemProfile> {
        let mut out = self.loadout.combat_items(&self.registry);
        // **Enchs land here, not in the loadout.** A profile is the board's
        // answer to "what did these cells make"; an ench is the character's,
        // and a loadout that knew about them would be a loadout that knew
        // about a licence.
        if !self.enchanted.is_empty() {
            crate::ench::apply(&mut out, &self.enchanted, &crate::data::enchs());
        }
        out
    }

    // ---------------------------------------------------------------- enchs

    /// Is this character licensed to bolt anything to anything?
    ///
    /// The Kaklon Patent's, and nobody else's. **The class is the gate**, not a
    /// node inside it: enching is what the class *is*, and a class whose
    /// identity waited on a point spent would be a class you could take and not
    /// notice you had taken.
    pub fn licensed(&self) -> bool {
        self.class.as_deref() == Some(crate::ench::LICENSED_CLASS)
    }

    /// What is bolted to this component, if anything.
    pub fn ench_on(&self, piece: PieceId) -> Option<&crate::ench::Ench> {
        self.enchanted.iter().find(|e| e.on == piece)
    }

    pub fn give_ench(&mut self, id: &str) {
        self.enchs_owned.push(id.to_string());
    }

    /// How many of an ench are loose in the rack.
    pub fn enchs_loose(&self, id: &str) -> usize {
        self.enchs_owned.iter().filter(|e| *e == id).count()
    }

    /// Bolt an ench to a component. Returns why not.
    pub fn attach_ench(
        &mut self,
        id: &str,
        piece: PieceId,
    ) -> Result<(), crate::ench::Refusal> {
        use crate::ench::Refusal;
        if !self.licensed() {
            return Err(Refusal::NoLicence);
        }
        let data = crate::data::enchs();
        if data.get(id).is_none() {
            return Err(Refusal::NoSuchEnch);
        }
        if self.enchs_loose(id) == 0 {
            return Err(Refusal::NotYours);
        }
        if !self.owned.contains(&piece) {
            return Err(Refusal::NoSuchPiece);
        }
        // **One ench a component.** Two is a bigger space and a bigger screen,
        // and neither has earned its place. Enforced here rather than assumed
        // by whatever is drawing the rack.
        if let Some(there) = self.ench_on(piece) {
            let what = data.get(&there.id).map(|d| d.name.clone()).unwrap_or_else(|| there.id.clone());
            return Err(Refusal::AlreadyEnched(what));
        }
        let at = self
            .enchs_owned
            .iter()
            .position(|e| e == id)
            .ok_or(Refusal::NotYours)?;
        self.enchs_owned.remove(at);
        self.enchanted.push(crate::ench::Ench {
            on: piece,
            id: id.to_string(),
            active: true,
        });
        Ok(())
    }

    /// Take one off. It goes back in the rack. Returns which, or nothing.
    pub fn detach_ench(&mut self, piece: PieceId) -> Option<String> {
        let at = self.enchanted.iter().position(|e| e.on == piece)?;
        let e = self.enchanted.remove(at);
        self.enchs_owned.push(e.id.clone());
        Some(e.id)
    }

    /// Switch one on or off where it is. Returns the new state.
    pub fn toggle_ench(&mut self, piece: PieceId) -> Option<bool> {
        let e = self.enchanted.iter_mut().find(|e| e.on == piece)?;
        e.active = !e.active;
        Some(e.active)
    }

    /// Drop any attachment to a component that is no longer owned.
    ///
    /// Handing a component over a counter, or selling one, takes the ench with
    /// it back to the rack rather than leaving an attachment pointing at
    /// something the character has not got.
    pub fn tidy_enchs(&mut self) {
        let gone: Vec<PieceId> = self
            .enchanted
            .iter()
            .map(|e| e.on)
            .filter(|p| !self.owned.contains(p))
            .collect();
        for p in gone {
            self.detach_ench(p);
        }
    }
}
