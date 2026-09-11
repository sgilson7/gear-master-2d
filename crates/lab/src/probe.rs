//! Scratch probe for the block's recon. Not shipped; not a test.
use gm2d_core::save;

fn main() {
    let txt = std::fs::read_to_string(std::env::args().nth(1).expect("a save")).expect("read");
    let g = save::load(&txt).expect("open");
    let ch = &g.character;
    let (gear, items) = ch.item_partition();
    println!("        gear: &[");
    for (name, slot, x, y, rot) in &gear {
        println!("        (\"{name}\", SlotKind::{slot:?}, {x}, {y}, {rot}),");
    }
    println!("        ],");
    println!("        items: &{items:?},");
    // and where each ench sits in that order
    println!("        enchs: &[");
    for e in &ch.enchanted {
        let name = ch.registry.def(e.on).name;
        // find its index in the partition by matching name+slot+cell
        let mut at = None;
        for (i, (n, slot, x, y, _)) in gear.iter().enumerate() {
            if *n == name
                && ch.loadout.slot_holding(e.on) == Some(*slot)
                && ch.loadout.slot(*slot).anchor_of(e.on) == Some((*x, *y))
            {
                at = Some(i);
                break;
            }
        }
        println!("        (\"{}\", {}),  // {name}", e.id, at.expect("the ench is on a seated piece"));
    }
    println!("        ],");
}
