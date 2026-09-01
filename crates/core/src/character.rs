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
use crate::slot::PlaceError;
use crate::stats::Stats;

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
struct BoardSnapshot {
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
#[derive(Clone, Debug)]
pub struct Character {
    pub registry: PieceRegistry,
    /// Every component this character has, worn or not.
    pub owned: Vec<PieceId>,
    pub loadout: Loadout,
    pub gold: i32,
    /// Maximum health earned outside the boards — the one stat a reward can
    /// add to the character rather than to a grid.
    pub grown_health: i32,
    undo_stack: Vec<BoardSnapshot>,
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

    // ------------------------------------------------------------ readings

    pub fn reports(&self) -> Vec<SlotReport> {
        self.loadout.reports(&self.registry)
    }

    pub fn report(&self, kind: SlotKind) -> SlotReport {
        self.loadout.report(&self.registry, kind)
    }

    /// Every grid's contribution, plus health earned off the boards.
    pub fn player_stats(&self) -> Stats {
        let mut base = self.loadout.total_stats(&self.registry);
        base.health += self.grown_health;
        base
    }

    /// Activation profiles for every assembled item — what combat runs on.
    pub fn combat_items(&self) -> Vec<crate::loadout::ItemProfile> {
        self.loadout.combat_items(&self.registry)
    }
}
