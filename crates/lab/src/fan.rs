//! **How wide the fan is: how many distinct tiles a shot can reach, per power.**
//!
//! `PLAN-M17.md` §10.3 asks whether the Treyway is too small to be an
//! interesting table, and the first instinct is to measure **mean distance**.
//! That number saturates by construction on a sixteen-by-sixteen map — the
//! largest `|dx| + |dy|` is about fifteen — so it flattens from power three
//! whatever the constants are, and it says nothing about whether a cue is
//! worth aiming.
//!
//! What means something for a cue is **how many different places a shot can
//! put you**, which is `SECOND-ORDER-M17.md` row 3, and it is also the shape
//! of the question `shot::aim_at` answers one target at a time. So: sweep
//! every angle at every power from one tile and count the distinct landings.
//!
//! **It is a bench, not a test.** The number it produces is a fact about the
//! map rather than a contract — a table gaining a bumper moves it — and a
//! number in a test is a number somebody has to keep true. What *is* a test is
//! `every_place_on_a_shot_map_is_reachable_by_shots`, which asks whether every
//! place can be landed on rather than how many tiles can.
//!
//! Its answer as this was written, from each map's own start:
//!
//! | table | reached | walkable | |
//! |---|---|---|---|
//! | the Treyway | 161 | 176 | **91%** |
//! | the Undercountry | 247 | 308 | **80%** |
//!
//! `SECOND-ORDER-M17.md` row 2 reports 164 of 176, measured on the bare
//! Treyway before M17.1 put nine obstacles on it. **The interesting half is
//! the comparison**: the bigger table is the one where a single shot does
//! *not* reach nearly everywhere, which is the shape §10.3 is asking about.
//! The call is the human's.
//!
//!     cargo run -p gm2d-lab --bin fan                  # the Treyway
//!     cargo run -p gm2d-lab --bin fan the-undercountry
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::shot::{self, Shot, POWER_UNIT, RESTITUTION, STEPS};
use gm2d_core::world::Allowances;

fn main() {
    // **Whichever table you name**, because there are two now and the second
    // one's number is the one nobody has looked at.
    let id = std::env::args().nth(1).unwrap_or_else(|| "the-treyway".into());
    let w = data::map(&id, Difficulty::Easy);
    let a = Allowances::default();
    println!("{id}, {}x{}", w.width, w.height);
    println!("power_unit {POWER_UNIT}, restitution {RESTITUTION}");
    // The map's own start, which is where a player's first shot is taken from.
    let from = (w.start.0, w.start.1);
    println!("  from the start, {from:?}");
    let mut all = std::collections::BTreeSet::new();
    for power in 1..=10u8 {
        let mut here = std::collections::BTreeSet::new();
        for angle in 0..STEPS as u16 {
            let f = shot::shoot(&w, from, Shot::new(angle, power), &a);
            here.insert(f.rest);
            all.insert(f.rest);
        }
        println!("  power {power:>2}: {:>3} distinct landings", here.len());
    }
    println!("  all seven hundred and twenty shots reach {} distinct tiles", all.len());
    let walkable: usize = (0..w.height)
        .flat_map(|y| (0..w.width).map(move |x| (x, y)))
        .filter(|(x, y)| w.walkable(*x, *y, &a))
        .count();
    println!("  the map has {walkable} walkable tiles");
}
