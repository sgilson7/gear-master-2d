//! The authoring bench: dress a creature, or read the one somebody else dressed.
//!
//! Ported from the original's pair, and both of its guarantees come with it.
//!
//! **Monsters wear the catalogue.** A creature is a `Loadout` of real
//! components in real slots, assembled by the rules the player plays by — never
//! a stat block. That is the reason the ladder can be re-rated by moving gear
//! rather than by editing numbers, and it is why `dress` searches the catalogue
//! rather than solving for stats.
//!
//! **Assembles or fails.** A spec whose gear does not come together is a
//! creature that fights in loose components, which reads as a difficulty bug
//! and is a typo. `tests/enemies.rs` is that guarantee and this prints the same
//! verdict before anything is written down.
//!
//! The original's other half was a GUI — *dress creatures by hand, which is the
//! game, editing somebody else's board*. GM2D has no GUI crate; the campaign
//! took macroquad with it. `read` is the honest port: it prints an existing
//! creature's board as a grid, its items and its rating, which is what the
//! hand-dressing mode was for reading.
//!
//!     make dress RATING=1200            # find a loadout that rates near 1200
//!     make dress RATING=1200 SLOTS=3    # and use only three grids
//!     make read NAME="Cog Priest"       # print a creature's board
//!
//! Nothing here is shipped. `crates/wasm` does not depend on this crate.

use std::collections::BTreeSet;
use std::env;

use gm2d_core::combat::{Difficulty, GearPlacement, MonsterSpec, LADDER};
use gm2d_core::loadout::Loadout;
use gm2d_core::piece::{PieceRegistry, SlotKind, CATALOG};
use gm2d_core::rating;

const D: Difficulty = Difficulty::Easy;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("read") => read(&args.collect::<Vec<_>>().join(" ")),
        Some("dress") | None => {
            let target: i32 =
                env::var("RATING").ok().and_then(|v| v.parse().ok()).unwrap_or(1000);
            let slots: usize =
                env::var("SLOTS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
            dress(target, slots.clamp(1, 5));
        }
        Some(other) => {
            eprintln!("no such command: {other}\n\n{}", USAGE);
            std::process::exit(2);
        }
    }
}

const USAGE: &str = "\
  dress            search the catalogue for a loadout near RATING (env), using
                   SLOTS grids (env, 1..5, default 5)
  read <name>      print an existing creature's board, items and rating";

// ------------------------------------------------------------------ reading

/// Print a creature's board the way somebody looking at it would read it.
fn read(name: &str) {
    let Some(spec) = LADDER.iter().find(|m| m.name.eq_ignore_ascii_case(name.trim())) else {
        eprintln!("nothing on the ladder is called {name:?}");
        let mut near: Vec<&str> = LADDER
            .iter()
            .map(|m| m.name)
            .filter(|n| n.to_lowercase().contains(&name.trim().to_lowercase()))
            .collect();
        near.truncate(8);
        if !near.is_empty() {
            eprintln!("did you mean: {}", near.join(", "));
        }
        std::process::exit(1);
    };
    println!("{}  —  rates {}", spec.name, rating::creature_rating(spec, D));
    println!(
        "  body: {} health, {} strength, {} regen, resists {}/{} phys/magic, \
         {} mind, {} curse",
        spec.health,
        spec.strength,
        spec.regen,
        spec.physical_resist,
        spec.magic_resist,
        spec.mind_resist,
        spec.curse_resist
    );
    if !spec.attacks.is_empty() {
        for a in spec.attacks {
            println!("  innate: {} every {}ms", a.name, a.cooldown_ms);
        }
    }
    let (reg, lo) = spec.loadout_at(D);
    for kind in SlotKind::ALL {
        let report = lo.report(&reg, kind);
        if report.items.is_empty() {
            continue;
        }
        println!("  {}:", kind.name());
        for item in &report.items {
            let names: Vec<&str> =
                item.pieces.iter().map(|&p| reg.def(p).name).collect();
            println!(
                "    [{}] {} — {}",
                if item.assembled { "assembled".to_string() } else { item.status.clone() },
                item.name.full,
                names.join(" + ")
            );
        }
    }
    // The same verdict `tests/enemies.rs` gives, before anything is written.
    let loose = lo
        .reports(&reg)
        .iter()
        .flat_map(|r| r.items.iter())
        .filter(|i| !i.assembled)
        .count();
    println!(
        "  {}",
        if loose == 0 {
            "every grid assembles.".to_string()
        } else {
            format!("**{loose} items do not assemble** — this is a typo, not a difficulty.")
        }
    );
}

// ------------------------------------------------------------------ dressing

/// Search the catalogue for a loadout that rates near `target`.
///
/// **Greedy and deterministic**, and both matter. Greedy because the search
/// space is five grids of forty-eight cells against five hundred and fifty
/// components and nobody needs the optimum — they need a starting point they
/// can then move by hand. Deterministic because two people asking for the same
/// rating should get the same creature to argue about.
fn dress(target: i32, slots: usize) {
    let want: Vec<SlotKind> = SlotKind::ALL.iter().copied().take(slots).collect();

    // The gear first, packed as well as a greedy walk manages; then the body,
    // which is the other half of a rating and the dial that closes the gap.
    let mut gear: Vec<GearPlacement> = Vec::new();
    for &kind in &want {
        gear.extend(fill(kind, &gear));
    }
    let health = body_for(&gear, target);
    let got = rating::creature_rating(dressed(health, &gear), D);
    println!("// rates {got} against a target of {target}");
    println!("MonsterSpec {{");
    println!("    name: \"CHANGE ME\",");
    println!("    health: {health},");
    println!("    strength: 12,");
    println!("    regen: 2,");
    println!("    mind_resist: 6,");
    println!("    curse_resist: 6,");
    println!("    physical_resist: 8,");
    println!("    magic_resist: 8,");
    println!("    attacks: &[],");
    println!("    gear: &[");
    for (name, kind, x, y, rot) in &gear {
        println!("        (\"{name}\", SlotKind::{kind:?}, {x}, {y}, {rot}),");
    }
    println!("    ],");
    println!("    gear_offset: 0,");
    println!("    bounty: {},", (got / 12).max(6));
    println!("    sprite: MonsterSprite::Sentinel,");
    println!("    rank: Rank::Ordinary,");
    println!("    drops: &[],");
    println!("    items: &[],");
    println!("}}");
    println!();
    println!("// **Now move it by hand.** This is a starting point and not an");
    println!("// answer: the search is greedy, it knows nothing about what the");
    println!("// creature is supposed to *be*, and a creature is a character");
    println!("// before it is a number. `make read NAME=...` prints the result.");
}

/// A rough body that lands a dressed creature nearer its target.
///
/// **Measured rather than derived.** The rating's own weights are private and
/// should stay that way — a bench that imported them would be a second copy of
/// the formula, and the formula is the one thing here that must have one home.
/// So this asks: two bodies, two ratings, and the line between them.
fn body_for(gear: &[GearPlacement], target: i32) -> i32 {
    let rate = |health: i32| {
        rating::creature_rating(dressed(health, gear), D)
    };
    let (lo_h, hi_h) = (200, 9600);
    let (lo_r, hi_r) = (rate(lo_h), rate(hi_h));
    if hi_r <= lo_r {
        return 1200;
    }
    let per = (hi_r - lo_r) as f32 / (hi_h - lo_h) as f32;
    let out = lo_h as f32 + (target - lo_r) as f32 / per;
    ((out.clamp(60.0, 60_000.0) / 10.0).round() * 10.0) as i32
}

/// One dressed creature, for the bench to weigh.
fn dressed(health: i32, gear: &[GearPlacement]) -> &'static MonsterSpec {
    leak(MonsterSpec {
        name: "The Dressed",
        health,
        strength: 12,
        regen: 2,
        mind_resist: 6,
        curse_resist: 6,
        physical_resist: 8,
        magic_resist: 8,
        attacks: &[],
        gear: leak_gear(gear),
        gear_offset: 0,
        bounty: 20,
        sprite: gm2d_core::combat::MonsterSprite::Sentinel,
        rank: gm2d_core::combat::Rank::Ordinary,
        drops: &[],
        items: &[],
    })
}

/// Seat as much of one grid as improves it, biggest first.
fn fill(kind: SlotKind, already: &[GearPlacement]) -> Vec<GearPlacement> {
    let taken: BTreeSet<&str> = already.iter().map(|(n, ..)| *n).collect();
    let mut reg = PieceRegistry::new();
    let mut lo = Loadout::new();
    let grow_by = 8 - lo.slot(kind).rows();
    lo.slot_mut(kind).grow(grow_by);

    let mut candidates: Vec<&'static gm2d_core::piece::PieceDef> = CATALOG
        .iter()
        .filter(|d| d.slot == kind)
        .filter(|d| !gm2d_core::piece::is_boss_only(d.name))
        .filter(|d| !gm2d_core::piece::is_event_only(d.name))
        .filter(|d| !d.kind.is_enchantment())
        .filter(|d| !taken.contains(d.name))
        .collect();
    // Dearest first, and ties by name: deterministic, and price is the
    // catalogue's own opinion of worth.
    candidates.sort_by(|a, b| b.price.cmp(&a.price).then(a.name.cmp(b.name)));

    // **Seed on a core, unconditionally.** An item rates nothing until it
    // assembles, so a walk that only keeps what improves the rating keeps
    // nothing at all: the first component is always a step from zero to zero.
    // The same shape Auto-pack has, and the same answer — seed, then grow.
    let mut out = Vec::new();
    let mut score = 0;
    let core = candidates
        .iter()
        .position(|d| d.kind.is_core())
        .map(|i| candidates.remove(i));
    if let Some(def) = core {
        if let Some(index) = CATALOG.iter().position(|c| c.name == def.name) {
            let id = reg.alloc(index);
            if lo.can_place(&reg, id, kind, 0, 0).is_ok() {
                lo.slot_mut(kind).place(&reg, id, 0, 0);
                out.push((def.name, kind, 0, 0, 0));
            }
        }
    }
    for def in candidates {
        let Some(index) = CATALOG.iter().position(|c| c.name == def.name) else { continue };
        let id = reg.alloc(index);
        let mut placed = None;
        'find: for y in 0..8u8 {
            for x in 0..6u8 {
                if lo.can_place(&reg, id, kind, x, y).is_ok() {
                    lo.slot_mut(kind).place(&reg, id, x, y);
                    placed = Some((x, y));
                    break 'find;
                }
            }
        }
        let Some((x, y)) = placed else { continue };
        let now: i32 = lo.report(&reg, kind).items.iter().map(|i| i.rating).sum();
        if now > score {
            score = now;
            out.push((def.name, kind, x, y, 0));
        } else {
            // **Taken straight back out.** A component that does not improve
            // the grid is a component in a cell somebody else could have used —
            // the same rule Auto-pack follows and for the same reason.
            lo.slot_mut(kind).remove(id);
        }
    }
    out
}

fn leak(spec: MonsterSpec) -> &'static MonsterSpec {
    Box::leak(Box::new(spec))
}

fn leak_gear(gear: &[GearPlacement]) -> &'static [GearPlacement] {
    Box::leak(gear.to_vec().into_boxed_slice())
}
