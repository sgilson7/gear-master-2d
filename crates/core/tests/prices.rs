use gm2d_core::piece::{CATALOG, PieceKind, SlotKind};
use gm2d_core::rating::piece_rating;
#[test]
#[ignore]
fn show() {
    for n in ["Manaflay","The Split Wisdom","Tithe Collector","Wrathbreaker","Witherroot"] {
        let d = CATALOG.iter().find(|c| c.name == n).unwrap();
        println!("{:<20} {:>4}  {:?}/{:?}", n, piece_rating(d), d.slot, d.kind);
    }
    let best = CATALOG.iter()
        .filter(|c| c.slot == SlotKind::Weapon && c.kind == PieceKind::Accessory
                    && !gm2d_core::piece::is_boss_only(c.name))
        .max_by_key(|c| piece_rating(c)).unwrap();
    println!("best ordinary weapon accessory: {} at {}", best.name, piece_rating(best));
}




