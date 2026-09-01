//! The golden combat fixture's format, written once and read once.
//!
//! Both halves live here so the writer and the reader cannot drift: the
//! capture harness (`golden_capture.rs`, temporary) calls [`render`], and the
//! verifier (`golden_combat.rs`, permanent) calls [`parse`]. When the capture
//! harness is deleted with the rest of the campaign, [`render`] goes unused
//! and the fixture is still checkable, which is the whole point of writing the
//! boards down as text rather than as a `Run`.

#![allow(dead_code)] // render() is unused once the capture harness is deleted

use gm2d_core::combat::CombatLog;
use gm2d_core::piece::SlotKind;

pub const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/combat.txt");

/// One board seating, as it survives in text: `(piece, slot, x, y, rotation)`.
pub type Placement = (String, SlotKind, u8, u8, u8);

pub struct Scenario {
    pub name: String,
    pub monster: String,
    pub difficulty: String,
    pub board: Vec<Placement>,
    /// `Loadout::name_seed`. Item names are hashed from the arrangement *and*
    /// this number, so a board rebuilt without it keeps every stat and renames
    /// every item — "Resonant Sliver" comes back as "Resonant Thorn". Another
    /// field M1's save has to carry, and one nothing but a golden transcript
    /// would have caught.
    pub name_seed: u64,
    /// Locked items, each as indices into `board`.
    ///
    /// **Locks are state, not geometry.** `Loadout::locks` cannot be derived
    /// from a seated board: two pieces that touch are one item unless a lock
    /// says otherwise, and which locks exist depends on the order the player
    /// built in. A fixture that re-derived them would be testing its own
    /// guess. M1's save file has to carry this field for the same reason.
    pub locks: Vec<Vec<usize>>,
    /// The transcript, one entry a line, exactly as [`render`] wrote it.
    pub log: String,
}

/// One scenario as fixture text.
pub fn render(
    name: &str,
    monster: &str,
    difficulty: &str,
    name_seed: u64,
    board: &[Placement],
    locks: &[Vec<usize>],
    log: &CombatLog,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n== scenario: {name}\n"));
    s.push_str(&format!("-- monster: {monster}\n"));
    s.push_str(&format!("-- difficulty: {difficulty}\n"));
    s.push_str(&format!("-- name_seed: {name_seed}\n"));
    s.push_str("-- board\n");
    for (piece, slot, x, y, rot) in board {
        s.push_str(&format!("{piece}|{slot:?}|{x}|{y}|{rot}\n"));
    }
    s.push_str("-- locks\n");
    for l in locks {
        let idx: Vec<String> = l.iter().map(|i| i.to_string()).collect();
        s.push_str(&format!("{}\n", idx.join(",")));
    }
    s.push_str("-- log\n");
    s.push_str(&transcript(log));
    s
}

/// The part of a fight that has to stay byte-identical.
///
/// Derived `Debug` on every entry rather than a prose rendering, because a
/// prose rendering is a second implementation of the log and would need its
/// own tests. `Debug` changes only when the data changes, which is the
/// question being asked.
pub fn transcript(log: &CombatLog) -> String {
    let mut s = String::new();
    for e in &log.entries {
        s.push_str(&format!("{:>6} {} {:?}\n", e.at_ms, e.who, e.event));
    }
    s.push_str(&format!("outcome {:?} in {}ms\n", log.outcome, log.duration_ms));
    s
}

pub fn parse(text: &str) -> Vec<Scenario> {
    let mut out: Vec<Scenario> = Vec::new();
    let mut section = "";
    for line in text.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("== scenario: ") {
            out.push(Scenario {
                name: name.trim().to_string(),
                monster: String::new(),
                difficulty: String::new(),
                board: Vec::new(),
                name_seed: 0,
                locks: Vec::new(),
                log: String::new(),
            });
            section = "";
            continue;
        }
        let Some(cur) = out.last_mut() else { continue };
        if let Some(m) = line.strip_prefix("-- monster: ") {
            cur.monster = m.trim().to_string();
        } else if let Some(d) = line.strip_prefix("-- difficulty: ") {
            cur.difficulty = d.trim().to_string();
        } else if let Some(n) = line.strip_prefix("-- name_seed: ") {
            cur.name_seed = n.trim().parse().expect("name_seed");
        } else if line.starts_with("-- board") {
            section = "board";
        } else if line.starts_with("-- locks") {
            section = "locks";
        } else if line.starts_with("-- log") {
            section = "log";
        } else if section == "board" && !line.trim().is_empty() {
            cur.board.push(placement(line));
        } else if section == "locks" && !line.trim().is_empty() {
            cur.locks.push(
                line.split(',')
                    .map(|n| n.trim().parse().expect("lock index"))
                    .collect(),
            );
        } else if section == "log" && !line.trim().is_empty() {
            cur.log.push_str(line);
            cur.log.push('\n');
        }
    }
    out
}

fn placement(line: &str) -> Placement {
    let f: Vec<&str> = line.split('|').collect();
    assert_eq!(f.len(), 5, "malformed board line: {line}");
    let slot = SlotKind::ALL
        .iter()
        .copied()
        .find(|k| format!("{k:?}") == f[1])
        .unwrap_or_else(|| panic!("unknown slot {} in: {line}", f[1]));
    (
        f[0].to_string(),
        slot,
        f[2].parse().expect("x"),
        f[3].parse().expect("y"),
        f[4].parse().expect("rotation"),
    )
}
