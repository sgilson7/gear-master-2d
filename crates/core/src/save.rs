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

/// An ench bolted to a component, as an index into `registry`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnchSave {
    pub id: String,
    pub on: u32,
    #[serde(default = "yes")]
    pub active: bool,
}

fn yes() -> bool {
    true
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
    /// What is in the bank, as registry indices, exactly like `owned`.
    ///
    /// Defaults empty, so every save written before there was a bank opens
    /// with an empty one — which is what those characters had.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub banked: Vec<u32>,
    pub boards: Vec<(String, BoardSave)>,
    pub locks: Vec<LockSave>,
    /// Seeds the item-name hash. Without it every stat survives a round trip
    /// and every item is renamed.
    pub name_seed: u64,
    /// **Not written any more, and read only to be thrown away.**
    ///
    /// It was banked, and it never had to be: the loader has always re-derived
    /// it off `skills_taken` and the class the moment the file is open, so what
    /// the file said was overwritten a hundred lines later. A number that is
    /// stored and ignored is a number somebody will one day believe — and
    /// `the_three_fields_round_trip` caught exactly that shape in M13.0, where
    /// a class taken without the re-derivation came back from a round trip
    /// carrying a figure it did not go in with.
    ///
    /// **Derived, never banked**, which is the rule this project holds
    /// everywhere else: the level off experience, a node's effect off the node,
    /// the tower's fallen floors off `answered`. Kept in the struct with a
    /// default so a file written before this still opens, and skipped on the
    /// way out so nothing new carries it.
    #[serde(default, skip_serializing)]
    pub assembly_pct: i32,
    /// Experience banked, ever. The level is derived from it and is **not**
    /// stored: two numbers that could disagree is two answers to one question.
    #[serde(default)]
    pub xp: i32,
    /// Experience won and not yet spent at a town.
    ///
    /// `default` so a file written before the souls rule opens: it arrives
    /// carrying nothing, which is exactly what it was doing — every point it
    /// had won was already spent the moment it was won.
    #[serde(default)]
    pub carried: i32,
    /// How worn out, in percent. `default` so a file written before fatigue
    /// existed opens rested, which is what it was.
    #[serde(default)]
    pub fatigue: i32,
    /// Restoratives carried, by id and count.
    #[serde(default)]
    pub supplies: Vec<(String, u32)>,
    #[serde(default)]
    pub skill_points: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills_taken: Vec<String>,
    /// Enchs in the rack, by id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enchs_owned: Vec<String>,
    /// A licence bought off the van, for a character whose class does not carry
    /// one. Defaults false, so every older file opens as what it was.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bought_licence: bool,
    /// Enchs bolted to a component: which one, which component, switched on.
    ///
    /// The component is a **registry index**, exactly as `owned` and every
    /// board placement are — the registry is written whole and in order, so an
    /// index into it survives the trip. Writing a catalogue name instead would
    /// lose which of two identical components the ench was on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enchanted: Vec<EnchSave>,
    /// The class, by canonical name. Absent until level 5 and permanent after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// The second class, off Spike's paper. `default`, so a file written before
    /// M13 opens with no second class, no expert and no paper — which is what
    /// those characters had.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_class: Option<String>,
    /// The expert class, taken free once two trees are finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expert: Option<String>,
    /// A second paper bought and not yet answered.
    ///
    /// **Spent on the choice rather than on the purchase**, so it sits in the
    /// file until the fork is answered — which is what lets a player sleep on
    /// it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub second_paper: bool,
    /// Short Programme's streak: how many fast wins are behind you.
    ///
    /// One of the two things in this block that carry a fact about a *fight*
    /// rather than about a character, and neither is derivable — see
    /// `Character::carry_out_of`.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub fast_wins: u32,
    /// Standing Fact's curses: the ones that followed you out of the last
    /// fight and land before the next one acts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub told_curses: Vec<String>,
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveState {
    /// The stream's position, not the seed it started from.
    pub rng_state: u64,
    pub theme: String,
    pub character: CharacterSave,
    /// Where you are standing and what you have answered. The map itself is
    /// never here: it is `data/tiles.json`, it does not change, and a save that
    /// carried a copy would be a save that could disagree with the game.
    #[serde(default)]
    pub world: crate::world::WorldState,
    /// The fight in progress, if the save was taken mid-encounter.
    ///
    /// The creature and the tile, and nothing else. Combat does not draw, so
    /// the fight this reopens is the fight that was interrupted, character for
    /// character — no seed and no partial log needed. `PLAN.md` §6 proposed
    /// storing both; the engine made them unnecessary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter: Option<crate::fight::Encounter>,
}
// **The shelf is not in the save.** It was, by catalogue name, while a shop
// was a randomised stock the run owned. A town's stock is `data/shops.json`
// now and never changes, so it is content; what survives here is
// `WorldState::bought`, which is state. A file written before this still
// loads — serde ignores the key it no longer knows — and arrives with the
// shelves it always had, because they are the same shelves for everybody.


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
        let Game { rng, theme, character, world, encounter } = game;
        let Character {
            registry,
            owned,
            banked,
            loadout,
            gold,
            grown_health,
            xp,
            carried,
            fatigue,
            supplies,
            skill_points,
            skills_taken,
            class,
            second_class,
            expert,
            second_paper,
            fast_wins,
            told_curses,
            enchs_owned,
            enchanted,
            bought_licence,
            undo_stack: _,
        } = character;
        // `assembly_pct` is destructured and dropped: it is derived on load and
        // is no longer written. See the field's own note.
        let Loadout { slots, locks, name_seed, naming: _, assembly_pct: _ } = loadout;

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
                // **`worn()`, which is the gear layer.** `pieces()` walks
                // both, and `enchanted` below writes the underlay again — so
                // every enchantment went into the file twice, at the same
                // anchor, and only `Slot::place` routing by kind kept that from
                // being a bug. A file that says a thing twice is a file whose
                // two copies can one day disagree.
                let placed = slot
                    .worn()
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
                world: world.clone(),
                encounter: encounter.clone(),
                character: CharacterSave {
                    gold: *gold,
                    grown_health: *grown_health,
                    carried: *carried,
                    fatigue: *fatigue,
                    supplies: supplies.clone(),
                    registry: instances,
                    owned: owned.iter().map(|p| p.0).collect(),
                    banked: banked.iter().map(|p| p.0).collect(),
                    boards,
                    locks: locks
                        .iter()
                        .map(|l| LockSave {
                            pieces: l.pieces.iter().map(|p| p.0).collect(),
                            offsets: l.offsets.iter().map(|&(x, y)| [x, y]).collect(),
                        })
                        .collect(),
                    name_seed: *name_seed,
                    // **Zero on the way out, and skipped by serde.** It is
                    // derived on the way in; writing it would be writing a
                    // number nothing reads.
                    assembly_pct: 0,
                    xp: *xp,
                    skill_points: *skill_points,
                    skills_taken: skills_taken.clone(),
                    class: class.clone(),
                    second_class: second_class.clone(),
                    expert: expert.clone(),
                    second_paper: *second_paper,
                    fast_wins: *fast_wins,
                    told_curses: told_curses.clone(),
                    enchs_owned: enchs_owned.clone(),
                    bought_licence: *bought_licence,
                    enchanted: enchanted
                        .iter()
                        .map(|e| EnchSave { id: e.id.clone(), on: e.on.0, active: e.active })
                        .collect(),
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
        let SaveState { rng_state, theme, character, world, encounter } = state;
        let CharacterSave {
            bought_licence,
            gold,
            grown_health,
            registry: instances,
            owned,
            banked,
            boards,
            locks,
            name_seed,
            // Read to be dropped: the loader derives it below. See the field.
            assembly_pct: _,
            xp,
            carried,
            fatigue,
            supplies,
            skill_points,
            skills_taken,
            class,
            second_class,
            expert,
            second_paper,
            fast_wins,
            told_curses,
            enchs_owned,
            enchanted,
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
        // **Nothing sets `assembly_pct` here.** `refresh_assembly_pct` at the
        // end of this function is the one thing that fills it, rather than the
        // second of two — which is how a stale figure used to get as far as
        // being compared.

        // **A save from before there was an instrument frame gets one.**
        // `Loadout::new` builds it at the engine's full height, which is what a
        // creature wears; a player's is three rows. The boards loop below
        // overwrites whatever it finds a board for, so this only has to answer
        // for the file that names five.
        if !boards.iter().any(|(n, _)| n == "instrument") {
            *loadout.slot_mut(SlotKind::Instrument) =
                Slot::with_rows(SlotKind::Instrument, crate::progression::STARTING_ROWS);
        }

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
        // Checked against the registry the same way, because a bank naming a
        // component the file does not carry is the same corruption as a bag
        // doing it.
        character.banked = banked
            .iter()
            .map(|&p| check(p, "the bank"))
            .collect::<Result<_, _>>()?;
        character.loadout = loadout;
        character.gold = gold;
        character.grown_health = grown_health;
        character.xp = xp;
        character.carried = carried;
        character.fatigue = fatigue;
        character.supplies = supplies;
        character.skill_points = skill_points;
        character.skills_taken = skills_taken;
        character.class = class;
        // **All three, and the paper.** A character holds up to three classes
        // since M13; a file written before it defaults to one, which is what
        // those characters had.
        character.second_class = second_class;
        character.expert = expert;
        character.second_paper = second_paper;
        // The two things a fight leaves behind. Neither is derivable — see
        // `Character::carry_out_of`.
        character.fast_wins = fast_wins;
        character.told_curses = told_curses;
        character.enchs_owned = enchs_owned;
        character.bought_licence = bought_licence;
        // Checked like every other index into the registry. An ench bolted to
        // component 400 of 12 is a damaged file, and saying so beats panicking
        // three screens later.
        character.enchanted = enchanted
            .into_iter()
            .map(|e| {
                check(e.on, "an ench").map(|on| crate::ench::Ench {
                    on,
                    id: e.id,
                    active: e.active,
                })
            })
            .collect::<Result<_, _>>()?;

        let mut game = Game { rng: Rng::from_state(rng_state), theme, character, world, encounter };
        // The one pointer the file could not carry, put back from the id it
        // carried instead.
        game.character.loadout.naming = crate::theme::by_id(&game.theme).naming;
        // **`enchs_owned` changed meaning in M10** — it was what is loose and
        // it is what you have — so a file written before it holds only the
        // unbolted ones. Left alone, an ench that was bolted on would go on
        // working and would vanish the moment it was taken off. A field whose
        // meaning moved is a field that will arrive wrong, and the loader is
        // where that is caught, exactly as `World::repair` is.
        game.character.repair_enchs();
        // **And anything sitting in a grid it does not belong in**, which since
        // M13 means the instrument parts a file written before there was an
        // instrument frame left among the blades. See `repair_boards`.
        game.character.repair_boards();
        // **What the nodes and the classes imply, derived.** The save carries
        // which nodes were taken and which classes were chosen, not what they
        // did — and since M13.9 it does not carry `assembly_pct` at all, so
        // this is the only thing that ever sets it on the way in rather than
        // the second of two. The frames are deliberately left alone: they are
        // in the file and resizing on load is a different decision.
        game.character.refresh_assembly_pct();
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
        SlotKind::Instrument => "instrument",
    }
}

fn slot_kind(name: &str) -> Option<SlotKind> {
    Some(match name {
        "weapon" => SlotKind::Weapon,
        "helmet" => SlotKind::Helmet,
        "chest" => SlotKind::Chest,
        "gloves" => SlotKind::Gloves,
        "greaves" => SlotKind::Greaves,
        "instrument" => SlotKind::Instrument,
        _ => return None,
    })
}
