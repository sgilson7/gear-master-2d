//! Scratch probe for the block's recon. Not shipped; not a test.
use gm2d_core::combat::Difficulty;
use gm2d_core::{data, puzzle};

fn main() {
    let events = data::events();
    for id in std::env::args().skip(1) {
        let w = data::map(&id, Difficulty::Easy);
        println!(
            "{id}: blind {:?}  shortest-none {:?}  atlas {:?}  compass {:?}",
            puzzle::solvable_blind(&w, &events),
            puzzle::solvable_knowing(&w, &events, None),
            puzzle::solvable_knowing(&w, &events, Some("atlas")),
            puzzle::solvable_knowing(&w, &events, Some("compass")),
        );
    }
}
