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
            // **Which grids, by name.** `SLOTS=3` took the first three of
            // `SlotKind::ALL`, which meant every creature dressed at every
            // rating came out wearing the same helmet — the search is
            // deterministic and a deterministic search asked the same question
            // gives the same answer. Naming the grids is the difference between
            // a bench and a stamp.
            let only: Vec<SlotKind> = match env::var("ONLY").ok().filter(|v| !v.is_empty()) {
                Some(v) => v.split(',').filter_map(|n| slot_named(n.trim())).collect(),
                None => {
                    let n: usize =
                        env::var("SLOTS").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
                    SlotKind::ALL.iter().copied().take(n.clamp(1, 5)).collect()
                }
            };
            // And **where in the catalogue to start looking**. Candidates are
            // sorted dearest first, so skipping the first few dresses a
            // creature out of a different part of the shelf — which is how two
            // creatures at one rating stop being the same creature.
            let skip: usize = env::var("SKIP").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            // **And how many pieces a grid may hold.** Left uncapped, the
            // greedy fill packs all forty-eight cells, and a creature wearing
            // twenty-two components is a creature whose panel nobody can read
            // and whose fight takes four times as long to simulate. The
            // shipped ladder hand-dresses two to twelve; six a grid is that
            // shape.
            let per: usize = env::var("PER").ok().and_then(|v| v.parse().ok()).unwrap_or(6);
            dress(target, &only, skip, per.max(1));
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
fn slot_named(n: &str) -> Option<SlotKind> {
    Some(match n.to_lowercase().as_str() {
        "weapon" => SlotKind::Weapon,
        "helmet" => SlotKind::Helmet,
        "chest" => SlotKind::Chest,
        "gloves" => SlotKind::Gloves,
        "greaves" => SlotKind::Greaves,
        _ => return None,
    })
}

fn dress(target: i32, want: &[SlotKind], skip: usize, per: usize) {

    // The gear first, packed as well as a greedy walk manages; then the body,
    // which is the other half of a rating and the dial that closes the gap.
    let mut gear: Vec<GearPlacement> = Vec::new();
    for &kind in want {
        gear.extend(fill(kind, &gear, skip, per));
    }
    let health = body_for(&gear, target);
    let got = rating::creature_rating(dressed(health, &gear, target), D);
    println!("// rates {got} against a target of {target}");
    println!("MonsterSpec {{");
    println!("    name: \"CHANGE ME\",");
    let b = body_of(target);
    println!("    health: {health},");
    println!("    strength: {},", b.strength);
    println!("    regen: {},", b.regen);
    println!("    mind_resist: {},", b.mind);
    println!("    curse_resist: {},", b.mind);
    println!("    physical_resist: {},", b.resist);
    println!("    magic_resist: {},", b.resist * 9 / 10);
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
    let rate = |health: i32| rating::creature_rating(dressed(health, gear, target), D);
    let (lo_h, hi_h) = (200, 9600);
    let (lo_r, hi_r) = (rate(lo_h), rate(hi_h));
    if hi_r <= lo_r {
        return 1200;
    }
    let per = (hi_r - lo_r) as f32 / (hi_h - lo_h) as f32;
    let out = lo_h as f32 + (target - lo_r) as f32 / per;
    // **A ceiling, and it is the bench's most useful line.** Health is the
    // cheapest dial and the interpolation will happily spend ten thousand of it
    // to hit a number — which produces a creature that rates a thousand and is
    // a punching bag, because rating is not difficulty and a body with no
    // weapons behind it just takes longer to knock over. The shipped ladder
    // runs about two-and-a-bit health to a point of rating; past that the
    // answer is *more gear*, not more meat, and the bench says so.
    let ceiling = (target as f32 * HEALTH_TO_RATING).max(200.0);
    if out > ceiling {
        eprintln!(
            "// the gear only rates {}: {} health would be needed and the ceiling is {}.\n\
             // Give it more grids (ONLY=) or more pieces (PER=) rather than more meat.",
            rate(60),
            out as i32,
            ceiling as i32
        );
    }
    ((out.min(ceiling).clamp(60.0, 60_000.0) / 10.0).round() * 10.0) as i32
}

/// How much health a point of rating is worth on the shipped ladder.
///
/// Measured rather than chosen: Cog Priest is 2,100 health at 999, the Iron
/// Warden 900 at 212, Francis 9,000 at 2,958. Two and a bit, everywhere.
const HEALTH_TO_RATING: f32 = 2.3;

/// The body a creature of this weight has, apart from its health.
///
/// **Measured off the ladder rather than chosen.** A rung-forty creature is not
/// simply fatter than a rung-ten one: the High Cork Priest rates 999 with 58
/// strength and 45/40 resists, the Iron Warden 212 with 20 and 18/15, Francis
/// 2,958 with 150 and 70/70. Roughly a seventeenth, a twenty-second and a
/// twentieth of the rating, everywhere.
///
/// It matters because the first draft held these flat at twelve and eight, so
/// every point of rating past the gear had to come out of health — which is how
/// the bench produced a creature that rated a thousand and lost to an Oak Handle
/// and an Iron Blade. Rating is not difficulty; a body with no weapons behind it
/// just takes longer to knock over.
struct Body {
    strength: i32,
    regen: i32,
    resist: i32,
    mind: i32,
}

fn body_of(target: i32) -> Body {
    Body {
        strength: (target / 17).clamp(4, 200),
        regen: (target / 250).clamp(0, 20),
        resist: (target / 22).clamp(2, 75),
        mind: (target / 20).clamp(2, 75),
    }
}

/// One dressed creature, for the bench to weigh.
fn dressed(health: i32, gear: &[GearPlacement], target: i32) -> &'static MonsterSpec {
    let b = body_of(target);
    leak(MonsterSpec {
        name: "The Dressed",
        health,
        strength: b.strength,
        regen: b.regen,
        mind_resist: b.mind,
        curse_resist: b.mind,
        physical_resist: b.resist,
        magic_resist: b.resist * 9 / 10,
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
fn fill(kind: SlotKind, already: &[GearPlacement], skip: usize, per: usize) -> Vec<GearPlacement> {
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
    if skip < candidates.len() {
        candidates.drain(..skip);
    }

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
        if out.len() >= per {
            break;
        }
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
