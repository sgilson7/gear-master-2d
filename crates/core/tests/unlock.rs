//! What a shut thing wants, all the way down.
//!
//! Reported from play, standing at the bar of shingle: *"the event outside of
//! the diamond to get to the southern area should say very specifically the
//! mechanical necessities to unlock the area ... anything that is locked
//! should have such an explanation."*

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::game::Game;
use gm2d_core::world::{Allowances, Dir, PlaceKind};

const D: Difficulty = Difficulty::Easy;

/// The bar of shingle names every hop between here and the tide going out.
///
/// Four of them, on two maps: the reach is read, the plate is signed, the tenth
/// cairn is cut — and none of that is reachable at all without an instrument
/// assembled on its own frame, which is the hop the one-level version left out
/// and the one a player is most likely to be stuck behind.
#[test]
fn the_tide_crossing_says_every_hop() {
    let mut g = Game::new(9, "td");
    g.world.go_to("the-treyway");
    g.world.at = [8, 14];
    let w = data::map("the-treyway", D);
    let allowed = Allowances::of(&g.character.rules());
    let s = gm2d_core::world::step(&w, &mut g.world, &mut g.rng, D, Dir::South, &allowed);
    let said = s.blocked.clone().expect("the bar refuses");
    println!("{said}");
    assert_eq!(s.refused_by.as_deref(), Some("the-tide-crossing"), "it is the place's refusal");
    for want in [
        "THE WEXTREEN REACH",
        "THE TRIG STONE",
        "THE NINE SURVEYS",
        "survey instrument",
        "the edge of the Wextreen Reach",
    ] {
        assert!(said.contains(want), "the refusal does not mention {want:?}:\n  {said}");
    }
    // In the order they have to be done in, which is the whole of what makes it
    // a plan rather than a list. **THE WEXTREEN REACH stands on the Treyway and
    // the trig stone is inside the Reach**, so the card comes before the frame
    // and the frame before the plate — which is a fact about where two tiles
    // are and not one anybody would guess.
    let at = |n: &str| said.find(n).unwrap_or(usize::MAX);
    assert!(at("THE WEXTREEN REACH") < at("survey instrument"), "the card is this side of the edge");
    assert!(at("survey instrument") < at("the edge of the Wextreen Reach"), "build it, then go in");
    assert!(at("the edge of the Wextreen Reach") < at("THE TRIG STONE"), "the plate is inside");
    assert!(at("THE TRIG STONE") < at("THE NINE SURVEYS"), "sign it before the tenth cairn");
}

/// A hop already made is not printed again.
#[test]
fn it_prints_the_part_that_is_left() {
    let done: Vec<String> = vec!["read-the-reach".into(), "signed-the-trig".into()];
    let steps = gm2d_core::unlock::steps_to("built-the-tenth", &done, D);
    let joined = steps.join(" | ");
    assert!(joined.contains("THE NINE SURVEYS"), "{joined}");
    assert!(!joined.contains("THE TRIG STONE"), "a hop already made is printed again:\n  {joined}");
}

/// **Every locked door says what would open it.**
///
/// Not *is there a sentence* — the map file's `shut` is a sentence and it is
/// the thing that was reported as not enough. This asks whether the engine's
/// half is there: a mark, a key or an instrument, named, with the steps under
/// it.
#[test]
fn every_locked_place_says_what_would_open_it() {
    let g = Game::new(3, "td");
    let mut bare = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            let locked = !p.needs_all.is_empty() || p.needs.is_some() || p.needs_survey;
            if p.kind != PlaceKind::Gate || !locked {
                continue;
            }
            match g.sealed_because(&p, D) {
                Some(said) if said.contains("To open it:") => {}
                Some(said) => bare.push(format!("{id}/{}: {said}", p.id)),
                None => bare.push(format!("{id}/{}: says nothing at all", p.id)),
            }
        }
    }
    assert!(bare.is_empty(), "locked doors with no plan under them:\n  {}", bare.join("\n  "));
}
