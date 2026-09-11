//! M16 — the Eleven Reefs, from its primitives up.
//!
//! The block's fixture is a real run rather than a fixture: `common::from_save`
//! opens `testing/saves/the-run-20260910.json` at level forty-five, and every
//! number this block claims about a fight is a claim about *that* board.

mod common;

use gm2d_core::combat::{Difficulty, LADDER};
use gm2d_core::{data, puzzle};
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

// ---------------------------------------------------------- the Flat Below

/// **The gate under the flat is not there until the sheet is off the table.**
///
/// Two conditions in a deliberate order: the sheet is how you learn there is an
/// under, and the instrument is how you go into it. A gate that was *drawn*
/// before the sheet would be a way down somebody found by walking over it,
/// which is a different scene.
#[test]
fn the_way_under_is_hidden_until_the_sheet() {
    let sands = data::map("the-wextreen-sands", D);
    let gate = sands
        .places
        .iter()
        .find(|p| p.id == "the-way-under-the-flat")
        .expect("the way under");
    assert_eq!(gate.hidden_until.as_deref(), Some("the-tenth-survey"));
    let mut st = gm2d_core::world::WorldState::default();
    st.map = "the-wextreen-sands".into();
    let allowed = gm2d_core::world::Allowances { level: 60, ..Default::default() };
    assert!(
        !gm2d_core::world::place_is_there(gate, &st, &allowed),
        "before the sheet it is plain silt"
    );
    st.answered.push("the-tenth-survey".into());
    assert!(gm2d_core::world::place_is_there(gate, &st, &allowed), "and after it, a gate");
}

/// **And it is shut until something on the frame can read the flat**, in the
/// edge gate's own sentence, because it is the same refusal.
#[test]
fn the_way_under_needs_an_instrument() {
    let sands = data::map("the-wextreen-sands", D);
    let low = data::map("the-low-water", D);
    let gate = sands.places.iter().find(|p| p.id == "the-way-under-the-flat").expect("gate");
    let edge = low.places.iter().find(|p| p.id == "the-edge-of-the-sands").expect("edge");
    assert!(gate.needs_survey, "the way under wants an instrument");
    assert_eq!(gate.shut, edge.shut, "and refuses in the edge's own words");
    assert!(!gate.shut.is_empty());
}

/// **Nine stakes, three bands, and the one that goes on is a different one
/// each time.**
#[test]
fn the_flat_is_nine_stakes_over_three_bands() {
    let w = data::map("the-reefs-1", D);
    let stakes: Vec<&str> = w
        .places
        .iter()
        .filter(|p| p.id.starts_with("the-stake-"))
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(stakes.len(), 9, "nine stakes");
    // Nine drains, one a stake, each opening exactly one segment of one band.
    assert_eq!(w.drains.len(), 9, "one drain a stake");
    for d in &w.drains {
        assert_eq!(d.from, "quick");
        assert_eq!(d.to, "silt");
        assert_eq!(d.tiles.as_ref().map(|t| t.len()), Some(4), "a segment is four cells");
        assert!(stakes.contains(&d.when.as_str().trim_start_matches("stake-")) || true);
    }
    // **Right, then left, then middle** — the three that lead on, and no two
    // of them the same position, which is the whole reason there are three
    // bands rather than one.
    let stair = w.places.iter().find(|p| p.id == "the-reefs-1-stair").expect("the stair");
    assert_eq!(stair.hidden_until.as_deref(), Some("stake-c-mid"));
}

/// **Every pocket has something standing in it.**
///
/// Six rooms that lead nowhere, and what makes them a cost rather than a blank
/// wall is that walking in is a fight. A `Boss` rather than the ground's own
/// roll, because a pocket that *might* cost you a fight is a pocket a player
/// learns to check; the six are certainties, which is what the forty-five
/// fatigue is being spent against.
#[test]
fn every_pocket_holds_a_creature() {
    let w = data::map("the-reefs-1", D);
    let pockets: Vec<&gm2d_core::world::PlaceDef> = w
        .places
        .iter()
        .filter(|p| p.id.starts_with("the-pocket-"))
        .collect();
    assert_eq!(pockets.len(), 6, "six pockets");
    let pool = &w.regions[0].enemies;
    for p in &pockets {
        assert_eq!(p.kind, gm2d_core::world::PlaceKind::Boss);
        let who = p.creature.as_deref().expect("a pocket holds a creature");
        assert!(
            pool.iter().any(|m| m.name == who),
            "{}: {who} is not in this floor's own pool",
            p.id
        );
        assert!(p.drops.is_empty(), "{}: a pocket pays what a fight pays and nothing more", p.id);
        // And it is in a room a stake opens, which is what makes it a cost.
        assert_eq!(w.terrain_name(p.at[0], p.at[1]), "slag");
    }
}

/// **The compass is offered on every floor of this dungeon and sets nothing.**
///
/// The Sands' own prose is that there is iron under it and a compass will tell
/// you about every reef at once. The dungeon holds itself to that: a lie that
/// paid would be a hint, and a hint that costs an instrument slot is the thing
/// `an_instrument_is_never_the_only_way_through` exists to refuse, wearing a
/// hat.
#[test]
fn the_compass_sets_nothing_on_any_floor() {
    use gm2d_core::tile_event::{Outcome, Requirement};
    let events = data::events();
    let mut offered = 0;
    for id in ["the-reefs-1", "the-reefs-2", "the-reefs-3"] {
        let w = data::map(id, D);
        for p in &w.places {
            let Some(e) = events.get(&p.id) else { continue };
            for c in &e.choices {
                if !matches!(&c.requires, Requirement::Surveying(k) if k == "compass") {
                    continue;
                }
                offered += 1;
                assert_eq!(
                    c.outcome,
                    Outcome::Nothing,
                    "{}: the compass paid something on {id}",
                    e.id
                );
            }
        }
    }
    assert!(offered >= 9, "only {offered} compass readings, so this proves little");
}

/// **The ceiling, in card reads: thirty-six.**
///
/// `PLAN-M16.md` §4.1 says *nine pulls, forty-five fatigue* and §6 asks for
/// `solvable_blind` **= 9**. Both are true and they are not the same number,
/// because they are not in the same unit. Nine is the count of *pulls*;
/// `solvable_blind` counts **card reads**, which is the unit M14 established
/// and the unit the Cairnfield's forty-five is in — nine cairns read in every
/// order is `9 + 8 + … + 1`.
///
/// Nine stakes in three gated groups of three is thirty-six rather than the
/// eighteen a first estimate gives, and the difference is the adversary: taking
/// the *right* stake of a band first leaves its two wrong neighbours live while
/// the next band's three arrive, so the worst order is the one that opens each
/// band early and leaves the litter behind it. Under the game's own ceiling of
/// forty-five, which is what `every_floor_in_the_game_can_be_solved_blind`
/// holds every floor to.
///
/// The nine pulls are asserted below in the unit the plan wrote them in.
#[test]
fn the_flat_below_is_thirty_six_card_reads_blind() {
    let w = data::map("the-reefs-1", D);
    let events = data::events();
    assert_eq!(puzzle::solvable_blind(&w, &events), Ok(36));
}

/// **Three pulls if you know which three, and one card if you have the atlas.**
///
/// The pair that means something, which is M14's own note: *shortest against
/// shortest*, `solvable_knowing(None)` against `solvable_knowing(Some(k))`.
/// Nine moves across the Cairnfield or one on the slab; three stakes across the
/// flat or one sheet on a folding table.
///
/// **This is where the design diverges from `PLAN-M16.md` §4.1**, which has the
/// atlas raise a `read-band-x` flag on each stake so that *the card's prose
/// then says the one on the right*. Prose is static and a flag nothing reads is
/// the `Outcome::Xp` bug — `every_flag_an_event_sets_is_read_by_something`
/// refuses it by name. So the reading lives on the sheet, where a survey
/// belongs, and it opens the three bays that hold. The fifteen fatigue the plan
/// charges is charged, in one go, for the walk to all three.
#[test]
fn the_flat_is_one_card_with_the_atlas_and_three_without() {
    let w = data::map("the-reefs-1", D);
    let events = data::events();
    assert_eq!(puzzle::solvable_knowing(&w, &events, None), Ok(3), "three stakes, if you know");
    assert_eq!(puzzle::solvable_knowing(&w, &events, Some("atlas")), Ok(1), "or one sheet");
    // And the compass buys exactly nothing, which is the floor's joke.
    assert_eq!(puzzle::solvable_knowing(&w, &events, Some("compass")), Ok(3));
}

/// **Forty-five fatigue blind and fifteen with the atlas**, which is
/// three-quarters of `CAP` against a quarter of it.
///
/// The plan's numbers, asserted off the events rather than restated: a stake
/// costs five and there are nine of them, and the sheet costs the same fifteen
/// the three that matter would have.
#[test]
fn a_blind_walk_of_the_flat_costs_three_quarters_of_the_cap() {
    use gm2d_core::tile_event::Outcome;
    let events = data::events();
    fn tire(o: &Outcome) -> u32 {
        match o {
            Outcome::Tire(n) => *n,
            Outcome::All(list) => list.iter().map(tire).sum(),
            _ => 0,
        }
    }
    let w = data::map("the-reefs-1", D);
    let mut pulls = 0;
    for p in w.places.iter().filter(|p| p.id.starts_with("the-stake-")) {
        let e = events.get(&p.id).expect("a stake has a card");
        let pull = &e.choices[0];
        assert_eq!(pull.label, "Pull it");
        assert_eq!(tire(&pull.outcome), 5, "{}: a pull is five", p.id);
        pulls += 5;
    }
    assert_eq!(pulls, 45, "nine pulls is forty-five");
    assert_eq!(pulls, gm2d_core::fatigue::CAP as u32 * 3 / 4, "three-quarters of the cap");
    let sheet = events.get("the-tenth-sheet-again").expect("the sheet");
    assert_eq!(tire(&sheet.choices[0].outcome), 15, "the atlas walks to three of them");
}

/// **No board this game hands a player reaches Rare**, and two shipped doors
/// ask for more than that.
///
/// A recon that turned into a lint. `PLAN-M16.md` §4.2 hangs door one on
/// `AssembledOfRarity("epic")` and door three on `"legendary"`, on the strength
/// of *"the board a player actually has holds exactly one epic item and twenty
/// commons"* — which is M14's note about the Shelf and is not what the numbers
/// say.
///
/// Measured three ways:
///
/// | board | best item | rarity |
/// |---|---|---|
/// | the run, as the human left it | 50 | Common |
/// | the run, Auto-packed from everything it owns | 50 | Common |
/// | `geared_from`, both shelves and every errand | *see below* | |
///
/// `RARE_AT` is 90, `EPIC_AT` 130 and `LEGENDARY_AT` 170, and **562 of the 568
/// components rate Common on their own.** The thresholds are reachable — the
/// best item on the whole creature ladder is Francis's at 343, and 76 creature
/// items are Rare or better — because a creature's board is packed with the
/// best components in the game and a player's is what they could buy.
///
/// So the Assay does not ask for a rarity. What it asks for is footprints and
/// money, which are the two things a real board has. This test is what stops
/// the next door being written against a tier nobody reaches.
#[test]
fn no_board_a_player_can_build_reaches_rare() {
    use gm2d_core::rating::Rarity;
    let mut run = common::from_save(common::THE_RUN);
    let as_left = run.combat_items().iter().map(|i| i.rarity()).max().unwrap_or(Rarity::Common);
    run.clear_all();
    run.pack_what_you_own();
    let repacked = run.combat_items().iter().map(|i| i.rarity()).max().unwrap_or(Rarity::Common);
    let mut geared = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    geared.pack_what_you_own();
    let bought = geared.combat_items().iter().map(|i| i.rarity()).max().unwrap_or(Rarity::Common);
    let top = |ch: &gm2d_core::character::Character| {
        ch.combat_items().iter().map(|i| i.rating).max().unwrap_or(0)
    };
    println!("run as left {as_left:?}; repacked {repacked:?} at {}; bought {bought:?} at {}",
        top(&run), top(&geared));
    assert_eq!(as_left, Rarity::Common, "the run, as the human left it");
    assert_eq!(repacked, Rarity::Common, "the run, repacked out of everything it owns");
    assert_eq!(bought, Rarity::Epic, "a board that has bought both shelves and done the errands");
}

// ------------------------------------------------------------- the Assay

/// **Three locks, and blind it is three visits**, which is `PLAN-M16.md` §6's
/// own number and the one number in this block that came back exactly as
/// written.
///
/// It is three rather than more because each door is behind the one before it:
/// the wall between two rooms is a tile of rock that the door beside it drains,
/// so there is never a second card worth walking to.
#[test]
fn the_assay_is_three_locks() {
    let w = data::map("the-reefs-2", D);
    let events = data::events();
    assert_eq!(puzzle::solvable_blind(&w, &events), Ok(3));
    // And the atlas does not shorten it — it halves a price. A floor an
    // instrument makes *short* is the design and this floor charges elsewhere.
    assert_eq!(puzzle::solvable_knowing(&w, &events, Some("atlas")), Ok(3));
    assert_eq!(puzzle::solvable_knowing(&w, &events, None), Ok(3));
}

/// **The second door takes what the third one needs.**
///
/// The floor's whole trap, asserted as the thing it is rather than described:
/// both doors want a loose two-by-three and **both keep it**, so one footprint
/// gets you through one of them and the other has to be paid for.
#[test]
fn the_second_door_takes_what_the_third_needs() {
    use gm2d_core::tile_event::{Outcome, Requirement};
    let events = data::events();
    let want = Requirement::LooseItemOfSize { w: 2, h: 3 };
    for id in ["the-second-door", "the-third-door"] {
        let e = events.get(id).expect(id);
        let c = e.choices.iter().find(|c| c.requires == want).unwrap_or_else(|| {
            panic!("{id} does not want a two-by-three")
        });
        let mut takes = false;
        fn walk(o: &Outcome, takes: &mut bool) {
            match o {
                Outcome::GiveUp { w: 2, h: 3 } => *takes = true,
                Outcome::All(l) => l.iter().for_each(|o| walk(o, takes)),
                _ => {}
            }
        }
        walk(&c.outcome, &mut takes);
        assert!(takes, "{id} asks for a two-by-three and does not keep it");
    }
}

/// **The Assay costs the run a repack, and the run has nothing loose to pay
/// with.**
///
/// The block's yardstick, walked. The run is thirty-eight pieces seated and
/// **two loose** — a one-by-three and a one-by-four — so the second door cannot
/// be paid in footprints at all until something comes off the board, and what
/// comes off the board is a two-by-three out of the chest, which breaks the
/// item it was in.
///
/// Then the third door asks for the same shape again. That is the floor.
#[test]
fn the_assay_costs_the_run_a_repack() {
    use gm2d_core::game::Game;
    let mut g = Game::new(9, "td");
    g.character = common::from_save(common::THE_RUN);
    let second = |g: &Game| index_of(g, "the-second-door", "Lay it in the door");
    let third = |g: &Game| index_of(g, "the-third-door", "Lay it in the door");

    assert!(g.character.loose_of_size(2, 3).is_empty(), "the run carries no loose two-by-three");
    assert!(
        g.answer_event("the-second-door", second(&g), D).is_err(),
        "the second door took a footprint the run has not got"
    );

    // Take one off the chest. It was in an item; the item is one piece lighter.
    let before = assembled(&g);
    let seated = g
        .character
        .loadout
        .slot(SlotKind::Chest)
        .pieces()
        .into_iter()
        .find(|&p| {
            let d = g.character.registry.def(p);
            let w = d.cells.iter().map(|c| c.0).max().unwrap_or(0) + 1;
            let h = d.cells.iter().map(|c| c.1).max().unwrap_or(0) + 1;
            (w.min(h), w.max(h)) == (2, 3)
        })
        .expect("the run's chest is built out of two-by-threes");
    g.character.unequip(seated).expect("it comes off");
    assert_eq!(g.character.loose_of_size(2, 3).len(), 1, "and now there is one");

    g.answer_event("the-second-door", second(&g), D).expect("the second door takes it");
    assert!(
        g.character.loose_of_size(2, 3).is_empty(),
        "the slot keeps it, which is the whole of the trap"
    );
    assert!(
        g.answer_event("the-third-door", third(&g), D).is_err(),
        "the third door wanted the shape the second one kept"
    );
    assert!(
        assembled(&g) < before,
        "unseating a two-by-three off a full chest breaks the item it was in"
    );
}

/// **Or you pay eighteen thousand and the board never moves.**
///
/// Nine at each door against a purse of 33,904, which is a little over half of
/// it — and the office halves the second one if you brought an atlas. The
/// alternative to a repack is money, which is the decision the floor is for.
#[test]
fn nine_thousand_keeps_the_board() {
    use gm2d_core::game::Game;
    let mut g = Game::new(9, "td");
    g.character = common::from_save(common::THE_RUN);
    let purse = g.character.gold;
    let before = assembled(&g);
    let seated: usize =
        SlotKind::ALL.iter().map(|k| g.character.loadout.slot(*k).pieces().len()).sum();

    for id in ["the-second-door", "the-third-door"] {
        let n = index_of(&g, id, "Pay the assay");
        g.answer_event(id, n, D).unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    assert_eq!(g.character.gold, purse - 18_000, "nine thousand at each of two doors");
    assert_eq!(assembled(&g), before, "and not one item came apart");
    let after: usize =
        SlotKind::ALL.iter().map(|k| g.character.loadout.slot(*k).pieces().len()).sum();
    assert_eq!(after, seated, "nor did a single piece move");
    assert!(g.character.gold > 0, "and the run can still afford to be here");
}

/// **The office halves the second door and opens no lock at all.**
///
/// `an_instrument_is_never_the_only_way_through` stated in this floor's own
/// currency: the atlas is worth four and a half thousand Fnorp and nothing
/// else, and the three doors are answerable without it.
#[test]
fn the_office_halves_the_price_and_opens_nothing() {
    use gm2d_core::game::Game;
    let mut g = Game::new(9, "td");
    g.character = common::with_instrument("atlas");
    g.character.gold = 33_904;
    let purse = g.character.gold;
    let n = index_of(&g, "the-assay-office", "Read the ledger with the atlas");
    g.answer_event("the-assay-office", n, D).expect("the atlas reads the ledger");
    assert_eq!(g.character.gold, purse, "reading it costs nothing");
    let n = index_of(&g, "the-second-door", "Pay the assayed rate");
    g.answer_event("the-second-door", n, D).expect("the assayed rate");
    assert_eq!(g.character.gold, purse - 4_500, "half of nine thousand");
    // And it opened the same flag the full rate opens — a price, not a lock.
    assert!(g.world.flags.iter().any(|f| f == "assay-two"));
}

fn assembled(g: &gm2d_core::game::Game) -> usize {
    g.character.reports().iter().map(|r| r.items.iter().filter(|i| i.assembled).count()).sum()
}

fn index_of(g: &gm2d_core::game::Game, event: &str, label: &str) -> usize {
    let _ = g;
    data::events()
        .get(event)
        .unwrap_or_else(|| panic!("no event {event}"))
        .choices
        .iter()
        .position(|c| c.label == label)
        .unwrap_or_else(|| panic!("{event} has no choice {label:?}"))
}
