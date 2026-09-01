//! The wasm boundary. A shim, and nothing else.
//!
//! Every rule lives in `gm2d-core`. This crate moves strings and numbers
//! across the boundary and must never grow a decision of its own — the moment
//! a rule is decided here it is decided somewhere `cargo test` cannot reach in
//! a few seconds, and then there are two rulebooks.
//!
//! The save surface is exactly two functions, as `PLAN.md` 1.3 requires:
//! [`save_json`] and [`load_json`]. Everything else here is a getter the page
//! draws with, or a setter the page needs to prove the round trip.

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use gm2d_core::combat::Difficulty;
use gm2d_core::game::Game;
use gm2d_core::save;
use gm2d_core::world::{self, Dir, World, WorldState};

const DIFFICULTY: Difficulty = Difficulty::Easy;

// One game, because a page is one session. `RefCell` rather than a lock:
// wasm32-unknown-unknown is single-threaded, and a mutex here would be
// ceremony around a borrow that cannot be contended.
thread_local! {
    static GAME: RefCell<Game> = RefCell::new(Game::default());
    /// The map. Loaded once, never mutated, never saved — it is content, and
    /// `WorldState` in the game is the only part of it that is state.
    static WORLD: World = gm2d_core::data::world(DIFFICULTY);
}

fn map<T>(f: impl FnOnce(&World) -> T) -> T {
    WORLD.with(f)
}

fn with<T>(f: impl FnOnce(&Game) -> T) -> T {
    GAME.with(|g| f(&g.borrow()))
}

fn with_mut<T>(f: impl FnOnce(&mut Game) -> T) -> T {
    GAME.with(|g| f(&mut g.borrow_mut()))
}

// ---------------------------------------------------------------- the save

/// The whole game state as a save file.
#[wasm_bindgen]
pub fn save_json() -> String {
    with(save::save)
}

/// Replace the game with the one this text describes.
///
/// The error is the sentence core produced, unchanged. The page shows it to
/// the player as-is, because core is where the reason is known and a second
/// wording here would be a second, worse explanation.
#[wasm_bindgen]
pub fn load_json(text: &str) -> Result<(), JsValue> {
    match save::load(text) {
        Ok(g) => {
            with_mut(|slot| *slot = g);
            Ok(())
        }
        Err(why) => Err(JsValue::from_str(&why)),
    }
}

/// Start over from a seed.
#[wasm_bindgen]
pub fn new_game(seed: f64) -> () {
    with_mut(|g| {
        *g = Game::new(seed as u64, "td");
        // A new game starts where the map says, not at (0, 0) — which on this
        // map is rock, and on any map is an assumption.
        g.world = map(WorldState::at_start);
    });
}

// ---------------------------------------------------------------- readings

#[wasm_bindgen]
pub fn gold() -> i32 {
    with(|g| g.character.gold)
}

/// Move the purse. The number a visitor changes before downloading.
#[wasm_bindgen]
pub fn add_gold(n: i32) {
    with_mut(|g| g.character.gold = (g.character.gold + n).max(0));
}

/// Draw from the run's random stream, returning what came out.
///
/// The stream every encounter will be rolled against in M2. Exposed now
/// because "the save restores the RNG" is the half of the gate that a gold
/// counter cannot demonstrate: a save that stored the seed rather than the
/// position would restore the purse perfectly and then hand the player the
/// same next draw they had already seen.
#[wasm_bindgen]
pub fn draw() -> u32 {
    with_mut(|g| g.rng.below(1000) as u32)
}

/// Where the stream is standing, as hex. Shown so the page can display the
/// thing being preserved rather than only its consequences.
#[wasm_bindgen]
pub fn rng_state() -> String {
    with(|g| format!("{:016x}", g.rng.state()))
}

/// How many draws have been taken. Kept on the page rather than in core: it is
/// a fact about this demonstration, not about the game.
#[wasm_bindgen]
pub fn theme_id() -> String {
    with(|g| g.theme.clone())
}

/// The assembled board, one item a line: `name\trating\trarity`.
#[wasm_bindgen]
pub fn items() -> String {
    with(|g| {
        g.character
            .combat_items()
            .iter()
            .map(|i| format!("{}\t{}\t{}", i.name, i.rating, i.rarity().name()))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Arrange what the player owns into the engine's own preset positions.
///
/// The auto-build button. It seats components the character *has* and skips the
/// rest, so it is a convenience rather than a supply of free gear — which it
/// was, briefly, and which made the shop pointless.
#[wasm_bindgen]
pub fn apply_preset() {
    with_mut(|g| {
        g.character.loadout.naming = gm2d_core::theme::by_id(&g.theme).naming;
        g.character.apply_preset();
    });
}

// ---------------------------------------------------------------- the rest

#[wasm_bindgen]
pub fn piece_count() -> usize {
    gm2d_core::piece::CATALOG.len()
}

#[wasm_bindgen]
pub fn monster_count() -> usize {
    gm2d_core::combat::LADDER.len()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The save format this build reads and writes, for the page's footer. A
/// player comparing two builds should be able to see this without opening a
/// file.
#[wasm_bindgen]
pub fn save_version() -> u32 {
    save::VERSION
}

// ---------------------------------------------------------------- the world

/// The map as the canvas needs it: the terrain grid, the places on it, and the
/// regions with their measured danger.
///
/// Sent once at startup. The page redraws from this and from [`position`]; it
/// never asks core to draw anything, and core never learns what a pixel is.
#[wasm_bindgen]
pub fn world_json() -> String {
    map(|w| {
        let mut rows = Vec::new();
        for y in 0..w.height {
            let mut row = Vec::new();
            for x in 0..w.width {
                row.push(w.terrain_name(x, y).to_string());
            }
            rows.push(row);
        }
        let places: Vec<_> = w
            .places
            .iter()
            .map(|p| {
                serde_json::json!({
                    "at": p.at,
                    "kind": format!("{:?}", p.kind).to_lowercase(),
                    "id": p.id,
                    "name": p.name,
                })
            })
            .collect();
        let regions: Vec<_> = w
            .regions
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "danger": r.danger,
                    "enemies": r.enemies.iter().map(|m| m.name).collect::<Vec<_>>(),
                })
            })
            .collect();
        // The per-tile encounter chance, computed here.
        //
        // The page could work these out from the terrain table and the region
        // danger, and an earlier draft did — which put the encounter formula in
        // two places, in two languages, with only one of them tested. The
        // overlay draws numbers it was given.
        let mut chances = Vec::new();
        for y in 0..w.height {
            let mut row = Vec::new();
            for x in 0..w.width {
                row.push(w.encounter_per_mille(x, y));
            }
            chances.push(row);
        }
        serde_json::json!({
            "width": w.width, "height": w.height, "rows": rows,
            "chances": chances, "places": places, "regions": regions,
        })
        .to_string()
    })
}

/// Where the player is standing, and what is under them.
#[wasm_bindgen]
pub fn position() -> String {
    with(|g| {
        map(|w| {
            let [x, y] = g.world.at;
            serde_json::json!({
                "x": x, "y": y,
                "terrain": w.terrain_name(x, y),
                "region": w.region_at(x, y).map(|r| r.name.clone()),
                "danger": w.region_at(x, y).map(|r| r.danger),
                "chance": w.encounter_per_mille(x, y),
                "town": g.world.last_town,
                "walked": g.world.count("tiles-walked"),
                "fights": g.world.count("encounters"),
            })
            .to_string()
        })
    })
}

/// Take one step. `dir` is one of `n`, `s`, `e`, `w`.
///
/// Returns what happened, as JSON. The page renders it; it does not decide
/// anything about it — whether a tile is walkable, whether a fight starts and
/// which creature it is are all answered here.
#[wasm_bindgen]
pub fn try_step(dir: &str) -> String {
    let d = match dir {
        "n" => Dir::North,
        "s" => Dir::South,
        "e" => Dir::East,
        "w" => Dir::West,
        _ => return serde_json::json!({ "moved": false, "blocked": "no such direction" }).to_string(),
    };
    with_mut(|g| {
        map(|w| {
            let s = world::step(w, &mut g.world, &mut g.rng, DIFFICULTY, d);
            // An encounter becomes state the moment it is rolled. Holding it
            // only in the page would mean a player who saved while a creature
            // was on screen came back with no creature and a free step.
            if let Some(m) = s.encounter {
                g.encounter = Some(gm2d_core::fight::Encounter {
                    enemy: m.name.to_string(),
                    at: g.world.at,
                });
            }
            serde_json::json!({
                "moved": s.moved,
                "blocked": s.blocked,
                "event": s.event,
                "town": s.town,
                "encounter": s.encounter.map(|m| serde_json::json!({
                    "name": g.theme_name(m.name),
                    "canonical": m.name,
                    "rating": gm2d_core::rating::creature_rating(m, DIFFICULTY),
                    "note": gm2d_core::theme::by_id(&g.theme).note(m.name),
                })),
            })
            .to_string()
        })
    })
}

/// The event standing on the current tile, with each choice already judged
/// against what the player has.
///
/// The `takeable` flag and the `unmet` line are worked out here for the same
/// reason legality is: a page that decides for itself whether a choice is
/// available is a page with a second copy of the rules in it.
#[wasm_bindgen]
pub fn event_json(id: &str) -> String {
    use gm2d_core::tile_event::Requirement;
    with(|g| {
        let events = gm2d_core::data::events();
        let Some(e) = events.get(id) else {
            return serde_json::json!({ "error": format!("no event called {id}") }).to_string();
        };
        let choices: Vec<_> = e
            .choices
            .iter()
            .map(|c| {
                let ok = match &c.requires {
                    Requirement::None => true,
                    Requirement::Gold(n) => g.character.gold >= *n,
                    Requirement::Flag(f) => g.world.flags.iter().any(|x| x == f),
                    Requirement::Holding(name) => g.character.holds(name),
                };
                serde_json::json!({
                    "label": c.label, "blurb": c.blurb,
                    "takeable": ok,
                    "unmet": if ok { String::new() } else { c.unmet.clone() },
                })
            })
            .collect();
        serde_json::json!({ "id": e.id, "title": e.title, "prose": e.prose, "choices": choices })
            .to_string()
    })
}

/// Take choice `n` of the event on this tile. Returns the receipt.
#[wasm_bindgen]
pub fn answer(id: &str, n: usize) -> String {
    use gm2d_core::tile_event::{Outcome, Requirement};

    fn apply(g: &mut Game, o: &Outcome, receipt: &mut Vec<String>) {
        match o {
            Outcome::All(list) => list.iter().for_each(|i| apply(g, i, receipt)),
            Outcome::Gold(n) => {
                g.character.gold = (g.character.gold + n).max(0);
                receipt.push(if *n >= 0 {
                    format!("+{n} Fnorp")
                } else {
                    format!("{n} Fnorp")
                });
            }
            Outcome::Flag(f) => {
                if !g.world.flags.iter().any(|x| x == f) {
                    g.world.flags.push(f.clone());
                }
            }
            Outcome::Give(name) => match g.character.give(name) {
                Some(_) => receipt.push(format!("Gained: {name}")),
                None => receipt.push(format!("{name} is not in the catalogue")),
            },
            Outcome::Xp(n) => {
                // Banked, not spent. M4 is what turns this into a level; until
                // then the number is kept honestly rather than discarded, so
                // M4 inherits real figures instead of starting from zero.
                g.world.add("xp", (*n).max(0) as u32);
                receipt.push(format!("+{n} toward the next level"));
            }
            Outcome::Nothing => receipt.push("Nothing you could point to".into()),
        }
    }

    with_mut(|g| {
        let events = gm2d_core::data::events();
        let Some(e) = events.get(id) else {
            return serde_json::json!({ "error": "no such event" }).to_string();
        };
        if g.world.answered.iter().any(|a| a == id) {
            return serde_json::json!({ "error": "already answered" }).to_string();
        }
        let Some(c) = e.choices.get(n) else {
            return serde_json::json!({ "error": "no such choice" }).to_string();
        };
        let ok = match &c.requires {
            Requirement::None => true,
            Requirement::Gold(n) => g.character.gold >= *n,
            Requirement::Flag(f) => g.world.flags.iter().any(|x| x == f),
            Requirement::Holding(name) => g.character.holds(name),
        };
        if !ok {
            return serde_json::json!({ "error": c.unmet }).to_string();
        }
        let mut receipt = Vec::new();
        apply(g, &c.outcome, &mut receipt);
        g.world.answered.push(id.to_string());
        serde_json::json!({ "receipt": receipt }).to_string()
    })
}

/// Put the player back at the start, as a loss will in M3.
#[wasm_bindgen]
pub fn to_last_town() {
    with_mut(|g| {
        map(|w| {
            let home = w
                .places
                .iter()
                .find(|p| p.id == g.world.last_town)
                .or_else(|| w.place_at(w.start.0, w.start.1));
            if let Some(p) = home {
                g.world.at = p.at;
            }
        })
    });
}

// ---------------------------------------------------------------- the board

fn slot_of(name: &str) -> Option<gm2d_core::piece::SlotKind> {
    use gm2d_core::piece::SlotKind::*;
    Some(match name {
        "weapon" => Weapon,
        "helmet" => Helmet,
        "chest" => Chest,
        "gloves" => Gloves,
        "greaves" => Greaves,
        _ => return None,
    })
}

fn slot_name(s: gm2d_core::piece::SlotKind) -> String {
    format!("{s:?}").to_lowercase()
}

/// The five grids, everything on them, and what it all assembles into.
///
/// One call rather than a dozen getters, because the board is drawn as a whole
/// and a page that fetched it piecemeal could draw half of one arrangement and
/// half of the next.
///
/// **Every judgement here is core's.** Which pieces form an item, whether an
/// item assembled, what it is called, what it is worth, and what it is missing
/// if it did not — the page draws these and does not compute any of them.
#[wasm_bindgen]
pub fn board_json() -> String {
    use gm2d_core::piece::SlotKind;
    with(|g| {
        let ch = &g.character;
        // Every component name a player reads goes through the theme, the same
        // as every other one. The engine still says "Oak Handle" everywhere,
        // because everything it decides depends on that name meaning one thing.
        let theme = gm2d_core::theme::by_id(&g.theme);
        let slots: Vec<_> = SlotKind::ALL
            .iter()
            .map(|&k| {
                let slot = ch.loadout.slot(k);
                let report = ch.report(k);
                let placed: Vec<_> = slot
                    .pieces()
                    .into_iter()
                    .filter_map(|p| {
                        let (x, y) = slot.anchor_of(p)?;
                        Some(serde_json::json!({
                            "id": p.0, "x": x, "y": y,
                            "name": theme.piece(ch.registry.def(p).name),
                            "cells": slot.cells_of(p),
                            "locked": ch.is_locked_item(p),
                        }))
                    })
                    .collect();
                let items: Vec<_> = report
                    .items
                    .iter()
                    .map(|i| {
                        serde_json::json!({
                            "pieces": i.pieces.iter().map(|p| p.0).collect::<Vec<_>>(),
                            "cells": i.pieces.iter()
                                .flat_map(|&p| slot.cells_of(p))
                                .collect::<Vec<_>>(),
                            "assembled": i.assembled,
                            "status": i.status,
                            "name": i.name.full,
                            "short": i.name.short,
                            "rating": i.rating,
                            "notes": i.notes,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "slot": slot_name(k),
                    "rows": slot.rows(),
                    "cols": gm2d_core::slot::SLOT_W,
                    "placed": placed,
                    "items": items,
                })
            })
            .collect();

        let bag: Vec<_> = ch
            .inventory()
            .into_iter()
            .map(|p| {
                let d = ch.registry.def(p);
                serde_json::json!({
                    "id": p.0,
                    "name": theme.piece(d.name),
                    "slot": slot_name(d.slot),
                    "kind": format!("{:?}", d.kind),
                    "cells": ch.registry.shape(p).cells(),
                    "rotation": ch.registry.rotation(p),
                    "price": d.price,
                })
            })
            .collect();

        let stats = ch.player_stats();
        serde_json::json!({
            "slots": slots,
            "bag": bag,
            "undoable": ch.undoable(),
            "stats": {
                "health": stats.health, "strength": stats.strength,
                "armor": stats.armor, "mana": stats.mana, "regen": stats.regen,
            },
        })
        .to_string()
    })
}

/// Where this piece may be seated in this slot, as `[x, y]` pairs.
///
/// The fit preview *is* this list rendered. The page must never work out for
/// itself whether a cell is legal — `Slot::legal_anchors` is the rulebook, and
/// a preview that computed its own answer would be a second one.
#[wasm_bindgen]
pub fn legal_anchors(piece: u32, slot: &str) -> String {
    use gm2d_core::piece::PieceId;
    let Some(kind) = slot_of(slot) else { return "[]".into() };
    with(|g| {
        let id = PieceId(piece);
        let mut out = Vec::new();
        for y in 0..g.character.loadout.slot(kind).rows() {
            for x in 0..gm2d_core::slot::SLOT_W {
                if g.character.can_equip(id, kind, x, y).is_ok() {
                    out.push([x, y]);
                }
            }
        }
        serde_json::to_string(&out).unwrap_or_else(|_| "[]".into())
    })
}

/// Seat a piece. Returns an empty string on success, or the reason it was
/// refused — the sentence core wrote, shown unchanged.
#[wasm_bindgen]
pub fn place(piece: u32, slot: &str, x: u8, y: u8) -> String {
    use gm2d_core::piece::PieceId;
    let Some(kind) = slot_of(slot) else { return "no such slot".into() };
    with_mut(|g| match g.character.equip(PieceId(piece), kind, x, y) {
        Ok(()) => String::new(),
        Err(e) => e.to_string(),
    })
}

/// Take a piece off the board and back into the bag.
#[wasm_bindgen]
pub fn pick_up(piece: u32) -> String {
    use gm2d_core::piece::PieceId;
    with_mut(|g| match g.character.unequip(PieceId(piece)) {
        Ok(()) => String::new(),
        Err(e) => e.to_string(),
    })
}

/// Turn a piece a quarter turn clockwise. A seated piece only turns if it still
/// fits, and a refused turn leaves the board and the history untouched.
#[wasm_bindgen]
pub fn rotate(piece: u32) -> String {
    use gm2d_core::piece::PieceId;
    with_mut(|g| match g.character.rotate(PieceId(piece)) {
        Ok(()) => String::new(),
        Err(e) => e.to_string(),
    })
}

/// Lock or release the assembled item this piece belongs to.
#[wasm_bindgen]
pub fn toggle_lock(piece: u32) -> bool {
    use gm2d_core::piece::PieceId;
    with_mut(|g| g.character.toggle_lock_item(PieceId(piece)))
}

/// Take back the last board change, returning what was undone.
#[wasm_bindgen]
pub fn undo() -> String {
    with_mut(|g| g.character.undo().unwrap_or_default())
}

/// Clear every grid.
#[wasm_bindgen]
pub fn clear_board() {
    with_mut(|g| g.character.clear_all());
}

// ---------------------------------------------------------------- the fight

/// The creature waiting, or `null`.
#[wasm_bindgen]
pub fn encounter_json() -> String {
    with(|g| {
        let Some(e) = g.encounter.as_ref() else { return "null".into() };
        let Some(spec) = gm2d_core::fight::spec(e) else { return "null".into() };
        let (reg, lo) = spec.loadout_at(DIFFICULTY);
        serde_json::json!({
            "name": g.theme_name(spec.name),
            "canonical": spec.name,
            "note": gm2d_core::theme::by_id(&g.theme).note(spec.name),
            "rank": format!("{:?}", spec.rank).to_lowercase(),
            "health": spec.health,
            "bounty": spec.bounty,
            "rating": gm2d_core::rating::creature_rating(spec, DIFFICULTY),
            "items": lo.combat_items(&reg).iter()
                .map(|i| serde_json::json!({ "name": i.name, "rating": i.rating }))
                .collect::<Vec<_>>(),
        })
        .to_string()
    })
}

/// Run the fight and hand back the log.
///
/// Nothing is banked here. The page plays the replay first and calls
/// [`settle_fight`] when it is over, so a player who closes the tab mid-replay
/// has not been paid for a fight they did not watch — and, more to the point,
/// so the encounter is still in the save if they come back.
#[wasm_bindgen]
pub fn fight_json() -> String {
    use gm2d_core::combat::{Event, Side};
    with(|g| {
        let Some(log) = gm2d_core::fight::run(g, DIFFICULTY) else {
            return serde_json::json!({ "error": "there is nothing to fight" }).to_string();
        };
        // The player's items in the order combat indexed them, so the replay's
        // cooldown bars line up with `Activate { index }`.
        let items: Vec<_> = g
            .character
            .combat_items()
            .iter()
            .map(|i| serde_json::json!({
                "name": i.name,
                "cooldown_ms": i.cooldown_ms,
                "hit_for": i.hit_for(g.character.player_stats().strength),
                "slot": slot_name(i.slot),
            }))
            .collect();
        let entries: Vec<_> = log
            .entries
            .iter()
            .map(|e| {
                let (kind, side, item, index, amount) = match &e.event {
                    Event::Activate { side, item, index } =>
                        ("activate", *side, item.clone(), *index as i64, 0),
                    Event::Hit { by, damage, .. } => ("hit", *by, String::new(), -1, *damage as i64),
                    Event::MindHit { by, amount, .. } =>
                        ("mind", *by, String::new(), -1, *amount as i64),
                    Event::Burn { side, damage, .. } =>
                        ("burn", *side, String::new(), -1, *damage as i64),
                    Event::Regen { side, amount, .. } =>
                        ("regen", *side, String::new(), -1, *amount as i64),
                    Event::GainArmor { side, amount, .. } =>
                        ("armor", *side, String::new(), -1, *amount as i64),
                    Event::Cast { side, .. } => ("cast", *side, String::new(), -1, 0),
                    Event::Misfired { side, item } =>
                        ("misfire", *side, item.clone(), -1, 0),
                    Event::SuddenDeath { .. } => ("sudden", Side::Player, String::new(), -1, 0),
                    _ => ("other", Side::Player, String::new(), -1, 0),
                };
                serde_json::json!({
                    "at": e.at_ms, "kind": kind,
                    "side": if side == Side::Player { "player" } else { "enemy" },
                    "item": item, "index": index, "amount": amount,
                })
            })
            .collect();
        serde_json::json!({
            "outcome": format!("{:?}", log.outcome).to_lowercase(),
            "duration_ms": log.duration_ms,
            "player": { "name": "you", "max_health": log.player.max_health },
            "enemy": log.enemies.first().map(|c| serde_json::json!({
                "name": g.theme_name(gm2d_core::combat::creature(&c.name).map(|s| s.name).unwrap_or("")),
                "max_health": c.max_health,
            })),
            "items": items,
            "entries": entries,
        })
        .to_string()
    })
}

/// Bank the fight just watched and clear it.
#[wasm_bindgen]
pub fn settle_fight() -> String {
    with_mut(|g| {
        let Some(log) = gm2d_core::fight::run(g, DIFFICULTY) else {
            return serde_json::json!({ "error": "there is nothing to settle" }).to_string();
        };
        let Some(s) = gm2d_core::fight::settle(g, &log, DIFFICULTY) else {
            return serde_json::json!({ "error": "nothing to settle" }).to_string();
        };
        // A loss walks you home. The world owns where the player is, so the
        // move happens here rather than inside `settle`.
        if s.sent_home.is_some() {
            map(|w| {
                if let Some(p) = w.places.iter().find(|p| p.id == g.world.last_town) {
                    g.world.at = p.at;
                }
            });
        }
        serde_json::json!({
            "outcome": format!("{:?}", s.outcome).to_lowercase(),
            "gold": s.gold,
            "xp": s.xp,
            "sent_home": s.sent_home,
            "receipt": s.receipt,
        })
        .to_string()
    })
}

/// Walk away without fighting. The creature is forgotten and the tile is not.
#[wasm_bindgen]
pub fn flee() {
    with_mut(|g| g.encounter = None);
}

// ---------------------------------------------------------------- the shop

/// What the towns are selling.
///
/// Prices come from `rating::shop_price`, which is derived from what a
/// component is actually worth — deliberately steeper than linear, because
/// slots are scarce and the strong parts are what a build is short of.
#[wasm_bindgen]
pub fn shop_json() -> String {
    with(|g| {
        let theme = gm2d_core::theme::by_id(&g.theme);
        let shelf: Vec<_> = (0..g.shop.stock.len())
            .filter_map(|i| {
                let def = g.shop.def(i)?;
                let price = g.shop.price(i)?;
                Some(serde_json::json!({
                    "slot": i,
                    "name": theme.piece(def.name),
                    "canonical": def.name,
                    "for": slot_name(def.slot),
                    "kind": format!("{:?}", def.kind),
                    "price": price,
                    "rating": gm2d_core::rating::piece_rating(def),
                    "afford": g.character.gold >= price,
                    "locked": g.shop.is_locked(i),
                }))
            })
            .collect();
        serde_json::json!({
            "gold": g.character.gold,
            "reroll": gm2d_core::shop::REROLL_COST,
            "shelf": shelf,
        })
        .to_string()
    })
}

/// Buy shelf `slot`. Returns an empty string, or why not.
#[wasm_bindgen]
pub fn buy(slot: usize) -> String {
    with_mut(|g| {
        let Some(price) = g.shop.price(slot) else { return "nothing for sale there".into() };
        if g.character.gold < price {
            return format!("{price} Fnorp, and you have {}.", g.character.gold);
        }
        let Some(def) = g.shop.take(slot) else { return "nothing for sale there".into() };
        g.character.gold -= price;
        let id = g.character.registry.alloc(def);
        g.character.owned.push(id);
        String::new()
    })
}

/// Turn the shelf over.
#[wasm_bindgen]
pub fn reroll() -> String {
    with_mut(|g| {
        let cost = gm2d_core::shop::REROLL_COST;
        if g.character.gold < cost {
            return format!("{cost} Fnorp to turn the shelf, and you have {}.", g.character.gold);
        }
        g.character.gold -= cost;
        let need = g.character.combat_items().is_empty();
        let mut rng = g.rng.clone();
        g.shop.restock(&mut rng, need);
        g.rng = rng;
        String::new()
    })
}

/// Pin a shelf so a restock leaves it alone.
#[wasm_bindgen]
pub fn pin(slot: usize) -> bool {
    with_mut(|g| g.shop.toggle_lock(slot))
}

/// Open the board outside a fight, so a player can pack in town.
#[wasm_bindgen]
pub fn packing() -> bool {
    with(|g| g.encounter.is_none())
}

// ---------------------------------------------------------------- levels

/// Where the character stands: level, experience, points, and the boards the
/// level implies.
#[wasm_bindgen]
pub fn character_json() -> String {
    use gm2d_core::piece::SlotKind;
    with(|g| {
        let c = &g.character;
        let level = c.level();
        let (into, needed) = gm2d_core::progression::progress(c.xp);
        let rows: Vec<_> = SlotKind::ALL
            .iter()
            .map(|&k| {
                serde_json::json!({ "slot": slot_name(k), "rows": c.loadout.slot(k).rows() })
            })
            .collect();
        let stats = c.player_stats();
        serde_json::json!({
            "level": level,
            "xp": c.xp,
            "into": into,
            "needed": needed,
            "points": c.skill_points,
            "taken": c.skills_taken,
            "gold": c.gold,
            "rows": rows,
            "next_grows": gm2d_core::progression::grows_at(level + 1).map(slot_name),
            "stats": {
                "health": stats.health, "strength": stats.strength,
                "armor": stats.armor, "mana": stats.mana, "regen": stats.regen,
                "mind_resist": stats.mind_resist, "curse_resist": stats.curse_resist,
            },
        })
        .to_string()
    })
}

/// The base tree, with every node already judged against what the player has.
///
/// `takeable` and `why` are worked out here for the same reason legality is on
/// the board: a screen that greys a button out for its own reasons is a fourth
/// rule nobody tested.
#[wasm_bindgen]
pub fn skills_json() -> String {
    with(|g| {
        let tree = gm2d_core::data::skills();
        let Some(base) = tree.base() else { return "null".into() };
        let nodes: Vec<_> = base
            .nodes
            .iter()
            .map(|n| {
                let taken = g.character.skills_taken.iter().any(|t| *t == n.id);
                let verdict = tree.can_take(
                    &n.id,
                    &g.character.skills_taken,
                    g.character.skill_points,
                    g.character.class.as_deref(),
                );
                serde_json::json!({
                    "id": n.id,
                    "name": n.name,
                    "blurb": n.blurb,
                    "cost": n.cost,
                    "requires": n.requires,
                    "taken": taken,
                    "takeable": verdict.is_ok(),
                    "why": verdict.err().map(|e| e.to_string()).unwrap_or_default(),
                })
            })
            .collect();
        serde_json::json!({
            "name": base.name,
            "points": g.character.skill_points,
            "nodes": nodes,
        })
        .to_string()
    })
}

/// Spend a point. Returns an empty string, or the sentence core refused with.
#[wasm_bindgen]
pub fn take_skill(id: &str) -> String {
    with_mut(|g| {
        let tree = gm2d_core::data::skills();
        match g.character.take_skill(&tree, id) {
            Ok(()) => String::new(),
            Err(e) => e.to_string(),
        }
    })
}

// ---------------------------------------------------------------- the fork

/// The three classes on offer, or `null` when none is owed.
///
/// Named by the theme, promising in the engine's own words: `ClassPower` can
/// describe itself, so the mechanical line beside each is the rule rather than
/// a sentence about the rule.
#[wasm_bindgen]
pub fn class_offer_json() -> String {
    const THREE: [&str; 3] = ["Berserker", "Hexweaver", "Bloodletter"];
    with(|g| {
        if !g.character.owed_a_class() {
            return "null".into();
        }
        let theme = gm2d_core::theme::by_id(&g.theme);
        let tree = gm2d_core::data::skills();
        let offer: Vec<_> = THREE
            .iter()
            .filter_map(|canonical| {
                let def = gm2d_core::class::CLASSES.iter().find(|c| c.name == *canonical)?;
                let t = tree.tree_for_class(canonical);
                Some(serde_json::json!({
                    "canonical": def.name,
                    "name": theme.class(def.name),
                    // `retell` swaps the engine's words for the theme's, whole
                    // word at a time — so a promise about curses arrives
                    // talking about the Roast and Nut Freeze rather than
                    // searing and frost. TONE.md rule 13.
                    "blurb": theme.retell(def.blurb),
                    "promise": theme.retell(&def.power.describe()),
                    "nodes": t.map(|t| t.nodes.len()).unwrap_or(0),
                    "first": t.and_then(|t| t.nodes.first()).map(|n| n.name.clone()),
                }))
            })
            .collect();
        serde_json::json!({ "level": g.character.level(), "classes": offer }).to_string()
    })
}

/// Take the fork. Permanent.
#[wasm_bindgen]
pub fn choose_class(canonical: &str) -> String {
    with_mut(|g| match g.character.choose_class(canonical) {
        Ok(_) => String::new(),
        Err(why) => why,
    })
}

/// The class in play, themed, or an empty string.
#[wasm_bindgen]
pub fn class_name() -> String {
    // Through `class_def` rather than the stored string: the theme's lookup
    // wants a canonical name that outlives the call, and `CLASSES` is where
    // those live. The save stores a `String`; this is the resolver.
    with(|g| match g.character.class_def() {
        Some(def) => gm2d_core::theme::by_id(&g.theme).class(def.name).to_string(),
        None => String::new(),
    })
}

/// Every tree the character may spend in: the base one, plus their own.
#[wasm_bindgen]
pub fn all_trees_json() -> String {
    with(|g| {
        let tree = gm2d_core::data::skills();
        let mine = g.character.class.as_deref();
        let trees: Vec<_> = tree
            .trees
            .iter()
            .filter(|t| t.class.is_none() || t.class.as_deref() == mine)
            .map(|t| {
                let nodes: Vec<_> = t
                    .nodes
                    .iter()
                    .map(|n| {
                        let taken = g.character.skills_taken.iter().any(|x| *x == n.id);
                        let v = tree.can_take(
                            &n.id,
                            &g.character.skills_taken,
                            g.character.skill_points,
                            mine,
                        );
                        serde_json::json!({
                            "id": n.id, "name": n.name, "blurb": n.blurb, "cost": n.cost,
                            "taken": taken,
                            "takeable": v.is_ok(),
                            "why": v.err().map(|e| e.to_string()).unwrap_or_default(),
                        })
                    })
                    .collect();
                serde_json::json!({ "id": t.id, "name": t.name, "class": t.class, "nodes": nodes })
            })
            .collect();
        serde_json::json!({ "points": g.character.skill_points, "trees": trees }).to_string()
    })
}
