//! Scratch probe for the block's recon. Not shipped; not a test.
use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::shot::{self, Shot, POWER_UNIT, RESTITUTION, STEPS};
use gm2d_core::world::Allowances;

fn main() {
    let w = data::map("the-treyway", Difficulty::Easy);
    let a = Allowances::default();
    println!("power_unit {POWER_UNIT}, restitution {RESTITUTION}");
    let from = (8u8, 13u8);
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
