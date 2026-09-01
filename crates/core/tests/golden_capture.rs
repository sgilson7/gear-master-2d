//! Capture the golden combat fixture from the pristine fork.
//!
//! **This file is temporary and is deleted once M0's campaign deletion runs.**
//! It is the only thing in the suite that reads `Run`, and it exists for one
//! reason: the fixture has to be taken from upstream's combat *before* the
//! fork diverges. A golden file captured after the fact proves that the code
//! agrees with itself, which is not a fact anybody needs.
//!
//! What it writes is read back by `golden_combat.rs`, which rebuilds every
//! board from the fixture's own text using only the modules GM2D keeps. That
//! is the handover: after this file goes, the fixture still verifies.
//!
//!     REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core --test golden_capture

use gm2d_core::combat::{self, Difficulty, MonsterSpec};
use gm2d_core::piece::SlotKind;
use gm2d_core::run::Run;

mod golden;
use golden::{render, FIXTURE};

/// The spread. Seven rungs chosen because each one is the cheapest test of a
/// different mechanic: an innate attack with no gear at all, heavy plate,
/// curse resistance high enough to make a curse build fail, a health pool
/// nothing gets through quickly, and the three bosses whose kits are the most
/// crowded in the game.
const RUNGS: &[&str] = &[
    "Cave Rat",
    "Rust Golem",
    "Warded Idol",
    "Rust Colossus",
    "Vermin Sovereign",
    "The Tallow Saint",
    "Francis",
];

fn spec(name: &str) -> &'static MonsterSpec {
    combat::LADDER
        .iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("no monster named {name} on the ladder"))
}

#[test]
fn capture_the_golden_fixture() {
    if std::env::var("REBASELINE_GOLDEN_COMBAT").as_deref() != Ok("1") {
        return;
    }

    let mut out = String::new();
    out.push_str(
        "# gm2d golden combat fixture, version 1\n\
         #\n\
         # Captured from sgilson7/gear-master @ e93a391, before GM2D deleted the\n\
         # campaign. Every board below is written as placements so it can be rebuilt\n\
         # without `Run`; `golden_combat.rs` does exactly that and asserts the\n\
         # transcript still comes out character for character.\n\
         #\n\
         # A diff here means combat changed. That is either the bug or the commit\n\
         # message.\n",
    );

    // The engine's own preset, against the spread. `apply_preset` is what the
    // auto-build button produces, so this is a board a player could actually
    // have, not one assembled to make a test pass.
    for name in RUNGS {
        let mut run = Run::seeded(0x5EED_1234_ABCD_0001);
        run.apply_preset();

        let (placements, locks) = board_of(&run);
        let profiles = run.loadout.combat_items(&run.registry);
        let stats = run.loadout.total_stats(&run.registry);
        let log = combat::simulate_at(stats, &profiles, spec(name), Difficulty::Easy);

        out.push_str(&render(
            &format!("preset-vs-{}", slug(name)),
            name,
            "Easy",
            run.loadout.name_seed,
            &placements,
            &locks,
            &log,
        ));
    }

    // Bare hands. The degenerate case is worth pinning because it is the one
    // every later refactor forgets: no items, no stats, one innate attack on
    // the other side.
    let run = Run::seeded(0x5EED_1234_ABCD_0001);
    let profiles = run.loadout.combat_items(&run.registry);
    let stats = run.loadout.total_stats(&run.registry);
    let log = combat::simulate_at(stats, &profiles, spec("Cave Rat"), Difficulty::Easy);
    out.push_str(&render("bare-hands-vs-cave-rat",
        "Cave Rat",
        "Easy",
        run.loadout.name_seed,
        &[],
        &[],
        &log));

    std::fs::create_dir_all(std::path::Path::new(FIXTURE).parent().unwrap()).unwrap();
    std::fs::write(FIXTURE, &out).unwrap();
    eprintln!("wrote {FIXTURE} ({} bytes)", out.len());
}

/// Read a seated board back out as placements, plus the locks over them.
///
/// **Both halves, because a board is not just its geometry.** `Loadout::locks`
/// is state the player created by building in a particular order, and two
/// pieces that touch belong to one item unless a lock says they do not. The
/// first attempt at this fixture recorded placements only and re-derived the
/// locks on the way back in; the rebuilt board came out with more items than
/// it went in with, and the fight diverged on the second entry. Locks are
/// written down.
///
/// Order is `Slot::pieces`' own cell-walk order, unsorted, so the list a save
/// file would hold is the list this returns.
fn board_of(run: &Run) -> (Vec<(String, SlotKind, u8, u8, u8)>, Vec<Vec<usize>>) {
    let mut board = Vec::new();
    let mut index = std::collections::HashMap::new();
    for kind in SlotKind::ALL {
        let slot = run.loadout.slot(kind);
        for id in slot.pieces() {
            let Some((x, y)) = slot.anchor_of(id) else { continue };
            index.insert(id, board.len());
            board.push((
                run.registry.def(id).name.to_string(),
                kind,
                x,
                y,
                run.registry.rotation(id),
            ));
        }
    }
    let locks = run
        .loadout
        .locks
        .iter()
        .map(|l| l.pieces.iter().filter_map(|p| index.get(p).copied()).collect())
        .collect();
    (board, locks)
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
