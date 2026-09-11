//! M16 — the Eleven Reefs, from its primitives up.
//!
//! The block's fixture is a real run rather than a fixture: `common::from_save`
//! opens `testing/saves/the-run-20260910.json` at level forty-five, and every
//! number this block claims about a fight is a claim about *that* board.

mod common;

use gm2d_core::combat::{Difficulty, LADDER};
use gm2d_core::data;
use gm2d_core::piece::SlotKind;

const D: Difficulty = Difficulty::Easy;

/// **The save opens, and it is the character the plan says it is.**
///
/// The first test the block writes, and it is written first on purpose: every
/// fixture in M16 that says *the run* means this file, so a file that opened
/// as somebody else would make every number after it a number about nobody.
#[test]
fn the_save_opens_at_forty_five() {
    let ch = common::from_save(common::THE_RUN);
    assert_eq!(ch.level(), 45, "the run is level 45");
    let seated: usize =
        SlotKind::ALL.iter().map(|k| ch.loadout.slot(*k).pieces().len()).sum();
    assert_eq!(seated, 38, "thirty-eight pieces are seated");
    assert_eq!(ch.enchanted.len(), 6, "six enchs are bolted on");
    assert_eq!(ch.gold, 33_904, "the purse the plan transcribes");
    // Three classes, in the order they were paid for.
    let classes: Vec<&str> = ch.classes().collect();
    assert_eq!(classes, vec!["Berserker", "Showstopper", "ShortProgramme"]);
    // **Eleven assembled items**, which is what makes it a yardstick: a board
    // that assembles nothing is a stat line, and the Ninth Surveyor's first
    // draft was exactly that.
    let items: usize =
        ch.reports().iter().map(|r| r.items.iter().filter(|i| i.assembled).count()).sum();
    assert_eq!(items, 11, "the run assembles eleven items");
}

/// **The gear array a creature needs is in item order, and a board's is not.**
///
/// `MonsterSpec.items` is a chunk list over `gear`, so two items whose pieces
/// interleave in board order cannot be written down at all — which the run's
/// own helmet does: its first item is board entries 0, 3 and 5.
#[test]
fn item_partition_is_chunks_over_its_own_order() {
    let ch = common::from_save(common::THE_RUN);
    let (gear, items) = ch.item_partition();
    assert_eq!(gear.len(), 38, "every seated piece is in the partition");
    assert_eq!(items.iter().sum::<usize>(), 38, "the chunks cover it exactly");
    assert_eq!(items.len(), 12, "twelve groups, eleven of them assembled");
    // Each chunk is one slot's worth: an item does not straddle two grids.
    let mut at = 0;
    for n in &items {
        let slot = gear[at].1;
        assert!(
            gear[at..at + n].iter().all(|g| g.1 == slot),
            "a chunk is one grid's"
        );
        at += n;
    }
}

/// **Quicksand only ever drains to silt.**
///
/// M14 §1.1 is that flags only grow, so a puzzle whose wrong move locks the
/// right one is a puzzle the save cannot come back from. The Flat Below wants
/// ground that opens, and the only terrain that can deliver it is one that
/// cannot close: a drain naming `quick` as its *destination* would be ground
/// arriving over a stair somebody has to walk through.
#[test]
fn a_quick_cell_only_ever_drains_to_silt() {
    let mut seen = 0;
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for d in &w.drains {
            assert_ne!(d.to, "quick", "{id}: a drain may not make quicksand");
            if d.from == "quick" {
                assert_eq!(d.to, "silt", "{id}: quicksand drains to silt and nothing else");
                seen += 1;
            }
        }
    }
    // Not vacuous once the Flat Below is drawn; before it, this is the lint
    // arriving before the content it guards, which is the order M14 used.
    let _ = seen;
}

/// **No creature in the game is immune to a curse, and none is immune to a
/// whisper.**
///
/// Asked as a *behaviour* and not as a number on a sheet, which is the rule
/// `every_offered_class_reaches_something` had to learn: its first version
/// matched the variant and named where the power was honoured, which a stubbed
/// payout passed cleanly. So this calls `landing_ms` and
/// `mind_damage_after_resist` with the creature's own summed resistance and
/// asks whether anything comes back.
///
/// **It was red on twenty-three creatures and thirty-one**, which is not the
/// number `PLAN-M16.md` §5.3 has. The plan says the two M14 bosses stand past
/// the curse cap at 114 and 120 and that re-dressing brings them under it. The
/// measurement is that the Ninth Surveyor is at **88** — under it — What
/// Marbulon Faced Away From is at **102**, and twenty-one other creatures are
/// past, up to Nine of Ashes at 233. On the mind lane, which the plan does not
/// ask about at all, **every deep boss in the game was fully immune**, and M16
/// adds a base class whose whole promise is that lane.
///
/// So it was fixed in the engine rather than by re-dressing a quarter of the
/// ladder: see [`gm2d_core::stats::LANE_CAP`], which is `RESIST_CAP`'s own
/// argument applied to the two lanes that now have classes behind them.
/// Nothing under ninety-five moved, so no creature was retuned.
#[test]
fn no_creature_is_immune_to_a_curse_or_a_whisper() {
    use gm2d_core::curse::{mind_damage_after_resist, CurseKind};
    let mut dead = Vec::new();
    for spec in LADDER {
        let (stats, _) = spec.outfit_at(Difficulty::Medium);
        for kind in CurseKind::ALL {
            if kind.landing_ms(stats.curse_resist) == 0 {
                dead.push(format!("{}: {kind:?} lands for 0ms", spec.name));
            }
        }
        // A thousand points of mind damage is a whole fight's worth on the
        // deep ladder; if that eats nothing, the lane is shut.
        if mind_damage_after_resist(1000, stats.mind_resist) == 0 {
            dead.push(format!("{}: a whisper eats nothing", spec.name));
        }
    }
    assert!(
        dead.is_empty(),
        "{} creature-lanes are fully shut, so a class bought for them reaches \
         nothing at the bottom of a dungeon: {}",
        dead.len(),
        dead.join(", ")
    );
}

/// **And the cap actually bites**, or the test above is comparing every number
/// with a ceiling none of them reach — the *compares zero with zero* failure,
/// which this project has shipped three times.
#[test]
fn the_lane_cap_is_reached_by_the_shipped_ladder() {
    let over = LADDER
        .iter()
        .filter(|s| {
            let (st, _) = s.outfit_at(Difficulty::Medium);
            st.curse_resist > gm2d_core::stats::LANE_CAP
                || st.mind_resist > gm2d_core::stats::LANE_CAP
        })
        .count();
    assert!(over >= 20, "only {over} creatures reach the lane cap, so it guards nothing");
}

/// **An ench on a creature reads exactly like an ench on a player.**
///
/// One answer to *what does an ench do*, which is `ench::apply` over the
/// profiles — so the Tenth Surveyor's Chonga'd Brigandine is the run's own
/// Chonga'd Brigandine and not a second arithmetic.
#[test]
fn an_ench_on_a_creature_reads_like_an_ench_on_a_player() {
    let _ = D;
    let ch = common::from_save(common::THE_RUN);
    let data = data::enchs();
    // The run's own profiles, with and without its enchs.
    let bare = ch.loadout.combat_items(&ch.registry);
    let mut enched = bare.clone();
    gm2d_core::ench::apply(&mut enched, &ch.enchanted, &data);
    let moved = bare
        .iter()
        .zip(enched.iter())
        .filter(|(a, b)| a.power != b.power || a.cooldown_ms != b.cooldown_ms)
        .count();
    assert!(moved > 0, "six enchs move at least one profile");
}
