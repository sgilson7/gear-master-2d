//! The save file. Version 1.
//!
//! # Why this is a hand-written mirror and not a derive
//!
//! Two reasons, and the second is the one that would have bitten.
//!
//! **Ids, not pointers.** Upstream's state was threaded with `&'static`
//! references into `const` tables. Serialising one is easy — write the id.
//! Deserialising needs to resolve an id *back* to a static, which no derive
//! can do. GM2D has one of these left (`Loadout::naming`, a pointer into a
//! theme's word tables) and it is handled the way the rest were: the save
//! carries the theme's id and [`SaveFile::into_game`] re-points the field.
//!
//! **Indices are not names.** `PieceRegistry` stores each component as an
//! index into `CATALOG`, which is stable only while catalogue order is. A save
//! that wrote those indices would survive exactly until a component was
//! inserted rather than appended, and would then hand the player a board of
//! the wrong pieces with no error anywhere. So the file stores canonical
//! *names*, plus a fingerprint of the catalogue it was written against, and a
//! mismatch is a sentence rather than a rat wearing a crown.
//!
//! # The forgotten-field problem
//!
//! The failure this file is most likely to have is not a bug in it. It is a
//! field added to [`Game`] in M4 and never added here — the round trip still
//! passes, every existing test still passes, and a level-5 character quietly
//! loads at level 1.
//!
//! So every conversion below **destructures exhaustively**. Adding a field to
//! `Game`, `Character` or `Loadout` makes this file stop compiling until
//! somebody has said what happens to it. `the_mirror_names_every_field` in
//! `tests/save.rs` explains the arrangement to whoever hits it.

use serde::{Deserialize, Serialize};

use crate::character::Character;
use crate::game::Game;
use crate::loadout::{Loadout, LockedItem};
use crate::piece::{PieceId, PieceRegistry, SlotKind, CATALOG};
use crate::rng::Rng;
use crate::slot::Slot;

pub const FORMAT: &str = "gm2d-save";
pub const VERSION: u32 = 1;

// ---------------------------------------------------------------- the file

/// The envelope. Everything a reader needs to decide whether it can read the
/// rest before it tries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveFile {
    pub format: String,
    pub version: u32,
    pub catalog: CatalogStamp,
    pub state: SaveState,
}

/// Which catalogue this save was written against.
///
/// The count is for the error message; the fingerprint is what is actually
/// compared. Both, because "374 pieces, b1946ac9" tells a person which build
/// to go and find and a bare hash does not.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogStamp {
    pub pieces: usize,
    pub fingerprint: String,
}

/// A component instance: which catalogue entry, turned how far.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceSave {
    pub def: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rot: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

/// A locked item, as indices into `registry`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockSave {
    pub pieces: Vec<u32>,
    pub offsets: Vec<[u8; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSave {
    pub rows: u8,
    /// `[piece, x, y]`, where `piece` indexes `registry`.
    pub placed: Vec<[u32; 3]>,
    /// The enchantment layer, which sits under the gear and is not gear.
    /// Separate because `Slot` keeps it separate, and merging the two here
    /// would be this file inventing a rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enchanted: Vec<[u32; 3]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSave {
    pub gold: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub grown_health: i32,
    /// Every component in play, in `PieceId` order. `owned` and every
    /// `placed` entry index into this, which is exactly how `PieceId` works,
    /// so rebuilding in order restores the ids as well as the pieces.
    pub registry: Vec<InstanceSave>,
    pub owned: Vec<u32>,
    pub boards: Vec<(String, BoardSave)>,
    pub locks: Vec<LockSave>,
    /// Seeds the item-name hash. Without it every stat survives a round trip
    /// and every item is renamed.
    pub name_seed: u64,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub assembly_pct: i32,
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveState {
    /// The stream's position, not the seed it started from.
    pub rng_state: u64,
    pub theme: String,
    pub character: CharacterSave,
}

// ---------------------------------------------------------------- writing

/// A fingerprint of the catalogue's canonical names, in order.
///
/// FNV-1a over the names with separators, so an insertion and a rename are
/// both visible. Deliberately not a cryptographic hash: this detects "a
/// different build wrote this", not tampering, and a dependency would be a
/// dependency in a crate that has almost none.
pub fn catalog_fingerprint() -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for d in CATALOG {
        for b in d.name.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

pub fn catalog_stamp() -> CatalogStamp {
    CatalogStamp { pieces: CATALOG.len(), fingerprint: catalog_fingerprint() }
}

impl SaveFile {
    /// Read a game out.
    ///
    /// Every `let ... = ...` below is an exhaustive destructure. That is the
    /// point of them: a new field on any of these types stops this function
    /// compiling.
    pub fn of(game: &Game) -> Self {
        let Game { rng, theme, character } = game;
        let Character { registry, owned, loadout, gold, grown_health, undo_stack: _ } = character;
        let Loadout { slots, locks, name_seed, naming: _, assembly_pct } = loadout;

        // `naming` is skipped on purpose: it is a pointer into a theme's word
        // tables and `theme` above is how it comes back. `undo_stack` is
        // skipped because undo is a session's history of its own edits, and a
        // save that restored it would let you undo into a previous session.

        let instances = (0..registry.count())
            .map(|i| {
                let id = PieceId(i as u32);
                InstanceSave {
                    def: registry.def(id).name.to_string(),
                    rot: registry.rotation(id),
                }
            })
            .collect();

        let boards = slots
            .iter()
            .map(|slot| {
                let placed = slot
                    .pieces()
                    .into_iter()
                    .filter_map(|p| slot.anchor_of(p).map(|(x, y)| [p.0, x as u32, y as u32]))
                    .collect();
                let enchanted = slot
                    .enchantments()
                    .into_iter()
                    .filter_map(|p| {
                        slot.enchant_cells(p).first().map(|&(x, y)| [p.0, x as u32, y as u32])
                    })
                    .collect();
                (
                    slot_name(slot.kind).to_string(),
                    BoardSave { rows: slot.rows(), placed, enchanted },
                )
            })
            .collect();

        SaveFile {
            format: FORMAT.to_string(),
            version: VERSION,
            catalog: catalog_stamp(),
            state: SaveState {
                rng_state: rng.state(),
                theme: theme.clone(),
                character: CharacterSave {
                    gold: *gold,
                    grown_health: *grown_health,
                    registry: instances,
                    owned: owned.iter().map(|p| p.0).collect(),
                    boards,
                    locks: locks
                        .iter()
                        .map(|l| LockSave {
                            pieces: l.pieces.iter().map(|p| p.0).collect(),
                            offsets: l.offsets.iter().map(|&(x, y)| [x, y]).collect(),
                        })
                        .collect(),
                    name_seed: *name_seed,
                    assembly_pct: *assembly_pct,
                },
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a save serialises")
    }
}

// ---------------------------------------------------------------- reading

/// Parse and check the envelope before trusting anything inside it.
///
/// Every failure is one sentence a person can act on. None of them is a panic:
/// a bad file is a thing a player will hand this program, not a bug.
pub fn parse(text: &str) -> Result<SaveFile, String> {
    // The envelope first, on its own.
    //
    // Reading the whole file and *then* checking `format` gets the order
    // backwards: a theme file, or a save from a future version whose shape has
    // moved, fails on whichever field serde happens to miss first, and the
    // player is told `missing field \`catalog\`` about a file whose real
    // problem is that it is not a save at all. Two passes, so the first
    // question asked is the first question worth answering.
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default)]
        format: String,
        #[serde(default)]
        version: u32,
    }
    let envelope: Envelope = serde_json::from_str(text)
        .map_err(|e| format!("this does not look like a Gear Master 2D save: {e}"))?;
    if envelope.format != FORMAT {
        return Err(format!(
            "this is a {:?} file and Gear Master 2D reads {FORMAT:?} files.",
            envelope.format
        ));
    }
    if envelope.version > VERSION {
        return Err(format!(
            "this save is version {} and this build reads up to version {VERSION}. \
             Update the game, or load it in the version that wrote it.",
            envelope.version
        ));
    }

    let file: SaveFile = serde_json::from_str(text)
        .map_err(|e| format!("this save is damaged and cannot be read: {e}"))?;

    if file.format != FORMAT {
        return Err(format!(
            "this is a {:?} file and Gear Master 2D reads {FORMAT:?} files.",
            file.format
        ));
    }
    if file.version > VERSION {
        return Err(format!(
            "this save is version {} and this build reads up to version {VERSION}. \
             Update the game, or load it in the version that wrote it.",
            file.version
        ));
    }
    let here = catalog_stamp();
    if file.catalog.fingerprint != here.fingerprint {
        return Err(format!(
            "this save was made with a different catalogue ({} pieces, {}) and this build has \
             {} pieces, {}. Load it in the version that wrote it, or start a new game.",
            file.catalog.pieces, file.catalog.fingerprint, here.pieces, here.fingerprint
        ));
    }
    Ok(file)
}

/// Parse a save and build the game it describes.
pub fn load(text: &str) -> Result<Game, String> {
    parse(text).and_then(|f| f.into_game())
}

/// Write a game out as JSON.
pub fn save(game: &Game) -> String {
    SaveFile::of(game).to_json()
}

impl SaveFile {
    /// Build the game this file describes.
    ///
    /// Assumes the envelope has already been checked by [`parse`]; everything
    /// that can still go wrong here is a file that passed the envelope and is
    /// internally inconsistent, which is a corrupt save rather than an old one.
    pub fn into_game(self) -> Result<Game, String> {
        let SaveFile { format: _, version, catalog: _, state } = self;
        let state = migrate(version, state)?;
        let SaveState { rng_state, theme, character } = state;
        let CharacterSave {
            gold,
            grown_health,
            registry: instances,
            owned,
            boards,
            locks,
            name_seed,
            assembly_pct,
        } = character;

        // The registry first, in order, so `PieceId(i)` means what it meant.
        let mut registry = PieceRegistry::new();
        for (i, inst) in instances.iter().enumerate() {
            let def = CATALOG.iter().position(|d| d.name == inst.def).ok_or_else(|| {
                format!(
                    "this save holds a component this build has not got: {:?}. \
                     Load it in the version that wrote it.",
                    inst.def
                )
            })?;
            let id = registry.alloc(def);
            debug_assert_eq!(id.0 as usize, i, "alloc is not sequential");
            registry.set_rotation(id, inst.rot);
        }
        let count = registry.count() as u32;
        let check = |p: u32, what: &str| -> Result<PieceId, String> {
            if p < count {
                Ok(PieceId(p))
            } else {
                Err(format!("this save is damaged: {what} names component {p} of {count}."))
            }
        };

        let mut loadout = Loadout::new();
        loadout.name_seed = name_seed;
        loadout.assembly_pct = assembly_pct;

        for (name, board) in &boards {
            let kind = slot_kind(name)
                .ok_or_else(|| format!("this save is damaged: it names a slot called {name:?}."))?;
            let slot = loadout.slot_mut(kind);
            *slot = Slot::with_rows(kind, board.rows);
            for &[p, x, y] in &board.placed {
                let id = check(p, "a board")?;
                slot.place(&registry, id, x as u8, y as u8);
            }
            for &[p, x, y] in &board.enchanted {
                let id = check(p, "an enchantment")?;
                slot.place(&registry, id, x as u8, y as u8);
            }
        }

        // Locks last, and applied rather than re-derived. **This is the field
        // that would be silently wrong.** Two pieces that touch are one item
        // unless a lock says otherwise, so a loader that ran the locking pass
        // itself would hand back a board with different items, different
        // stats, and a different fight — which is what the first golden
        // fixture rebuild did.
        for l in locks {
            let pieces = l.pieces.iter().map(|&p| check(p, "a lock")).collect::<Result<_, _>>()?;
            let offsets = l.offsets.iter().map(|&[x, y]| (x, y)).collect();
            loadout.locks.push(LockedItem { pieces, offsets });
        }

        let mut character = Character::new();
        character.registry = registry;
        character.owned = owned
            .iter()
            .map(|&p| check(p, "the inventory"))
            .collect::<Result<_, _>>()?;
        character.loadout = loadout;
        character.gold = gold;
        character.grown_health = grown_health;

        let mut game = Game { rng: Rng::from_state(rng_state), theme, character };
        // The one pointer the file could not carry, put back from the id it
        // carried instead.
        game.character.loadout.naming = crate::theme::by_id(&game.theme).naming;
        Ok(game)
    }
}

// ---------------------------------------------------------------- migration

/// Bring an older state forward.
///
/// One arm per version, and the arm for the current version is the identity.
/// Written now, with nothing to do, because the moment a v2 exists is the
/// worst moment to be designing the mechanism that reaches it — and because a
/// migration path with no test is a migration path that has never run.
fn migrate(version: u32, state: SaveState) -> Result<SaveState, String> {
    match version {
        1 => Ok(state),
        // v2's arm goes here, taking a v1 state and returning a v2 one. When
        // it lands, this function stops being a formality and
        // `tests/save.rs::a_v1_save_still_loads` becomes the test that matters.
        v => Err(format!(
            "this save is version {v}, which this build has no way to bring forward."
        )),
    }
}

// ---------------------------------------------------------------- slot names

/// Slot names in the file are words, not numbers.
///
/// A save is a thing a person opens in a text editor when something has gone
/// wrong, and `"weapon"` tells them where they are while `3` does not. The
/// pairing is exhaustive both ways so a new slot kind cannot be half-added.
fn slot_name(k: SlotKind) -> &'static str {
    match k {
        SlotKind::Weapon => "weapon",
        SlotKind::Helmet => "helmet",
        SlotKind::Chest => "chest",
        SlotKind::Gloves => "gloves",
        SlotKind::Greaves => "greaves",
    }
}

fn slot_kind(name: &str) -> Option<SlotKind> {
    Some(match name {
        "weapon" => SlotKind::Weapon,
        "helmet" => SlotKind::Helmet,
        "chest" => SlotKind::Chest,
        "gloves" => SlotKind::Gloves,
        "greaves" => SlotKind::Greaves,
        _ => return None,
    })
}
