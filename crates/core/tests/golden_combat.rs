//! Combat replays what it replayed upstream.
//!
//! The fixture was taken from `sgilson7/gear-master @ e93a391` before GM2D
//! deleted the campaign, and every board in it is stored as placements rather
//! than as engine state. So this test rebuilds each board from the fixture's
//! own text using only the modules GM2D keeps — `piece`, `slot`, `loadout`,
//! `combat`, `stats` — and asserts the transcript comes out character for
//! character.
//!
//! Two things that would otherwise be invisible are pinned here:
//!
//! 1. **Combat is unchanged by the fork.** M0 deletes twelve modules out from
//!    under this code. Nothing in `simulate` is supposed to notice.
//! 2. **A board is reconstructible from placements alone.** M1's save file
//!    stores exactly this — piece name, slot, anchor, rotation — so if a board
//!    could not be rebuilt from it, the save format would be lossy and would
//!    not find out until a player lost a run to it.
//!
//! Rebaseline with `REBASELINE_GOLDEN_COMBAT=1` against the capture harness,
//! and say in the commit what started fighting differently.

use gm2d_core::combat::{self, Difficulty, MonsterSpec};
use gm2d_core::loadout::{LockedItem, Loadout};
use gm2d_core::piece::{PieceRegistry, CATALOG};

mod golden;
use golden::{parse, transcript, Placement, FIXTURE};

#[test]
fn every_golden_fight_replays_character_for_character() {
    let text = std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|e| panic!("cannot read {FIXTURE}: {e}\n\
             Capture it with REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core"));
    let scenarios = parse(&text);
    assert!(
        !scenarios.is_empty(),
        "{FIXTURE} parsed to nothing - the fixture is empty or its format moved"
    );

    for s in &scenarios {
        let (reg, lo) = rebuild(&s.board, &s.locks, s.name_seed);
        let profiles = lo.combat_items(&reg);
        let stats = lo.total_stats(&reg);
        let log = combat::simulate_at(stats, &profiles, spec(&s.monster), difficulty(&s.difficulty));
        let got = transcript(&log);

        if got != s.log {
            let first = s
                .log
                .lines()
                .zip(got.lines())
                .find(|(a, b)| a != b)
                .map(|(a, b)| format!("was: {a}\nnow: {b}"))
                .unwrap_or_else(|| {
                    format!(
                        "the transcript changed length: {} lines, now {}",
                        s.log.lines().count(),
                        got.lines().count()
                    )
                });
            panic!(
                "scenario {} fights differently than it did upstream:\n{first}\n(fixture: {FIXTURE})",
                s.name
            );
        }
    }
}

/// The fixture is only worth what its inputs are worth. A board of no pieces
/// against a monster of no gear would replay perfectly and prove nothing.
#[test]
fn the_fixture_covers_real_boards_and_real_monsters() {
    let text = std::fs::read_to_string(FIXTURE).expect("fixture");
    let scenarios = parse(&text);

    let seated: usize = scenarios.iter().map(|s| s.board.len()).sum();
    assert!(seated >= 20, "the fixture seats only {seated} pieces across all scenarios");

    let geared = scenarios
        .iter()
        .filter(|s| !spec(&s.monster).gear.is_empty())
        .count();
    assert!(geared >= 3, "only {geared} scenarios fight a monster that wears gear");

    for s in &scenarios {
        assert!(!s.log.is_empty(), "scenario {} recorded no log", s.name);
    }
}

/// Seat a board from placements, then apply exactly the locks recorded.
///
/// Not `lock_assembled_in`. Locking as each item completes is what a *player*
/// does while building, and re-running it here would be this test guessing at
/// state instead of restoring it — the first version did that, invented locks
/// the original board never had, and produced a board with more items than it
/// started with. The fixture says which items are locked; this applies that
/// and nothing else.
///
/// The offset arithmetic mirrors `lock_assembled_in`'s, because a `LockedItem`
/// carries where its pieces sit relative to each other and a lock rebuilt
/// without them is a lock that cannot be moved.
fn rebuild(
    board: &[Placement],
    locks: &[Vec<usize>],
    name_seed: u64,
) -> (PieceRegistry, Loadout) {
    let mut reg = PieceRegistry::new();
    let mut lo = Loadout::new();
    lo.name_seed = name_seed;
    let mut ids = Vec::with_capacity(board.len());
    for (name, slot, x, y, rot) in board {
        let def = CATALOG
            .iter()
            .position(|d| d.name == name)
            .unwrap_or_else(|| panic!("the fixture names a piece the catalogue has not got: {name}"));
        let id = reg.alloc(def);
        reg.set_rotation(id, *rot);
        assert!(
            lo.can_place(&reg, id, *slot, *x, *y).is_ok(),
            "the fixture seats {name} at {slot:?} ({x}, {y}) and the board refuses it"
        );
        lo.slot_mut(*slot).place(&reg, id, *x, *y);
        ids.push(id);
    }
    for set in locks {
        let pieces: Vec<_> = set.iter().map(|&i| ids[i]).collect();
        let slot = board[set[0]].1;
        let g = lo.slot(slot);
        let anchors: Vec<(u8, u8)> =
            pieces.iter().map(|&p| g.anchor_of(p).unwrap_or((0, 0))).collect();
        let minx = anchors.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let miny = anchors.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let offsets = anchors.iter().map(|&(x, y)| (x - minx, y - miny)).collect();
        lo.locks.push(LockedItem { pieces, offsets });
    }
    (reg, lo)
}

fn spec(name: &str) -> &'static MonsterSpec {
    combat::LADDER
        .iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("the fixture names a monster that is not on the ladder: {name}"))
}

fn difficulty(s: &str) -> Difficulty {
    match s {
        "Easy" => Difficulty::Easy,
        "Medium" => Difficulty::Medium,
        "Hard" => Difficulty::Hard,
        "Insane" => Difficulty::Insane,
        other => panic!("unknown difficulty in the fixture: {other}"),
    }
}
