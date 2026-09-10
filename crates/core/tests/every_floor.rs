//! M14.6 — the notebook executed: the questions that were asked of six floors,
//! asked of every floor there is.
//!
//! **Six floors were checked by hand in two files and the seventh was checked
//! by nobody.** `sump.rs` names three and `silt_stair.rs` names three, at their
//! numbers, which is right and is what the block is measured by — and a floor
//! added tomorrow is a floor with a puzzle nobody has walked. A list of six
//! written by hand is a list that can be five, which is the seventh time this
//! project has said that sentence.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::piece::{PieceKind, CATALOG};
use gm2d_core::puzzle;
use gm2d_core::world::PlaceKind;

const D: Difficulty = Difficulty::Easy;

/// **Every floor in the game with something hidden on it can be solved by
/// somebody who does not know the answer.**
///
/// Not *at its number* — that is the two per-dungeon files' job, and a number is
/// a design decision about one floor. This is the property: no map in the game
/// hides a way on behind a chain that cannot be walked from its own arrival
/// tile with a plausible bag.
///
/// It is not vacuous — eight maps have a hidden place on them — and it counts
/// them so that it says so if that ever stops being true.
///
/// Negative-tested by hanging the Cairnfield's stair on `cairn-9` **and**
/// `read-the-ninth`, a flag raised two hundred paces away on the shore: the
/// harness named the tenth cairn and said what it had managed.
#[test]
fn every_floor_in_the_game_can_be_solved_blind() {
    let events = data::events();
    let mut walked = 0;
    let mut worst = 0;
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        let hidden = w
            .places
            .iter()
            .filter(|p| p.hidden_until.is_some() || !p.hidden_until_all.is_empty())
            .count();
        if hidden == 0 {
            continue;
        }
        // A place hidden behind a *boss* is not a puzzle — the door in the
        // western wall waits for the Cave's own tile id, which nothing on the
        // map raises and nothing should. What this is about is a floor whose
        // way on is behind something the floor itself hands you.
        let by_flag = w.places.iter().any(|p| {
            p.hidden_until
                .iter()
                .chain(p.hidden_until_all.iter())
                .any(|k| {
                    w.places.iter().filter_map(|q| events.get(&q.id)).any(|e| {
                        e.choices.iter().any(|c| {
                            let mut fs = Vec::new();
                            gm2d_core::tile_event::flags_raised(&c.outcome, &mut fs);
                            fs.iter().any(|f| f == k)
                        })
                    })
                })
        });
        if !by_flag {
            continue;
        }
        walked += 1;
        let n = puzzle::solvable_blind(&w, &events)
            .unwrap_or_else(|why| panic!("{id} cannot be solved blind: {why}"));
        worst = worst.max(n);
    }
    assert_eq!(walked, 6, "{walked} floors in the game have a puzzle on them, not six");
    // **The ceiling `PLAN-M14.md` §1.2 builds the whole design on**, asserted
    // over every floor rather than over the one it was worked out for: nine
    // cairns visited in every order is 45, and no floor may cost more.
    assert!(worst <= 45, "the worst floor in the game takes {worst} card reads blind");
}

/// **There is one screen in the game that is not a loop, and it is on the last
/// map.**
///
/// Notebook row 15, answered by the content rather than by deleting a variant.
/// M14.3 turned the door under the lake into a gate onto the Silt Stair, which
/// left `PlaceKind::Door` without a user for one milestone; M14.4 put the
/// ending on the Undercountry, one tile south of a town with nothing in it.
///
/// A second `Door` would be a second place the game claims to stop, which is
/// the thing this asks about. Negative-tested by leaving the lake's paragraph
/// on the lake: two screens said the writing stopped.
#[test]
fn there_is_one_door_in_the_game_and_it_is_the_last_thing() {
    let mut doors = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places.iter().filter(|p| p.kind == PlaceKind::Door) {
            doors.push((*id, p.id.clone(), p.prose.join(" ")));
        }
    }
    assert_eq!(
        doors.len(),
        1,
        "the game says the writing stops in {} places: {:?}",
        doors.len(),
        doors.iter().map(|(m, i, _)| (m, i)).collect::<Vec<_>>()
    );
    let (map, _, said) = &doors[0];
    assert_eq!(*map, "the-undercountry", "the last screen is on {map}");
    assert!(
        said.to_lowercase().contains("nobody has decided"),
        "the last screen does not say what it is"
    );
    // And nothing else in the game says it either.
    for (id, _) in data::MAPS {
        for p in &data::map(id, D).places {
            if p.kind == PlaceKind::Door {
                continue;
            }
            assert!(
                !p.prose.join(" ").to_lowercase().contains("nobody has decided"),
                "{id}/{}: a second thing says the writing stops here",
                p.id
            );
        }
    }
}

/// **The golem is buildable off certainties, and the count is the answer to
/// §9 decision 5.**
///
/// Notebook row 6, asked end to end rather than asserted per boss. The plan
/// claims *"a player who did both dungeons should be able to build the golem"*
/// and calls it a claim about the catalogue the recon has to count.
///
/// Counted: the golem is **three `Map Shard` and two `Living Earth`**. Map
/// Shards are certainties — five off the Drambus Stack's floors and one off
/// Sootmother — and before M14 `Living Earth` had exactly **one** source in the
/// game, Bone Cantor at 300 per mille, a roll you have to win twice. The two
/// new bottoms drop one each, so it is a walk rather than a wait.
///
/// **A certainty is a boss's `drops`**, which is the field that is not rolled.
/// Negative-tested by taking `Living Earth` off one bottom: the count came back
/// one of two and named the recipe.
#[test]
fn the_two_bottoms_make_a_golem_out_of_certainties() {
    let mut certain: std::collections::BTreeMap<&str, usize> = Default::default();
    for (id, _) in data::MAPS {
        for p in &data::map(id, D).places {
            for d in &p.drops {
                let name = CATALOG
                    .iter()
                    .find(|c| c.name == d)
                    .unwrap_or_else(|| panic!("{id}/{}: {d:?} is not a component", p.id))
                    .name;
                *certain.entry(name).or_insert(0) += 1;
            }
        }
    }

    // The golem's own recipe, read off the table rather than written here —
    // *ask the recipe table, never a list of kinds*, which is the sixth thing
    // this project paid for and the reason `roll_barrel` was rewritten.
    let golem = gm2d_core::piece::recipes(gm2d_core::piece::SlotKind::Instrument)
        .iter()
        .find(|way| way.iter().any(|(k, ..)| *k == PieceKind::Earth))
        .expect("no way of building an instrument wants the ground itself");

    for &(kind, min, _) in golem.iter() {
        let have: usize = certain
            .iter()
            .filter(|(n, _)| CATALOG.iter().any(|c| c.name == **n && c.kind == kind))
            .map(|(_, n)| *n)
            .sum();
        assert!(
            have >= min as usize,
            "the golem wants {min} of {kind:?} and the game hands out {have} for certain"
        );
    }

    // And it is the two new bottoms that made it true: before them there was
    // one Living Earth in the game and it was a roll.
    assert_eq!(
        certain.get("Living Earth").copied().unwrap_or(0),
        2,
        "the two bottoms do not drop two Living Earth between them"
    );
}
