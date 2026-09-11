//! Scratch probe for the block's recon. Not shipped; not a test.
use gm2d_core::combat::{Difficulty, LADDER};
use gm2d_core::data;
use gm2d_core::world::PlaceKind;

fn main() {
    let mut bosses: Vec<String> = Vec::new();
    for (id, _) in data::MAPS {
        let w = data::map(id, Difficulty::Easy);
        for p in &w.places {
            if p.kind == PlaceKind::Boss {
                if let Some(c) = &p.creature {
                    bosses.push(c.clone());
                }
            }
        }
    }
    println!("{} boss tiles", bosses.len());
    println!("{:<34} {:>5} {:>5} {:>6} {}", "creature", "curse", "mind", "rating", "on a boss tile");
    for spec in LADDER {
        let (stats, _) = spec.outfit_at(Difficulty::Medium);
        let on = bosses.iter().any(|b| b == spec.name);
        if stats.curse_resist >= 100 || stats.mind_resist >= 100 || on {
            println!(
                "{:<34} {:>5} {:>5} {:>6} {}",
                spec.name,
                stats.curse_resist,
                stats.mind_resist,
                gm2d_core::rating::creature_rating(spec, Difficulty::Medium),
                if on { "BOSS" } else { "" }
            );
        }
    }
}
