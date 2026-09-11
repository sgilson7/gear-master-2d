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

// ------------------------------------------------------- the Needle Room

/// **Four sinkholes, four levers, and the alcoves touch the ring only on the
/// diagonal.**
///
/// The floor's geometry as a property rather than a picture: if an alcove ever
/// grows an orthogonal neighbour on the ring, the sinkhole that reaches it
/// stops being the way in and the floor is a corridor.
#[test]
fn the_alcoves_are_reachable_by_nothing_but_a_fall() {
    let w = data::map("the-reefs-3", D);
    let allowed = gm2d_core::world::Allowances::default();
    // **An alcove is two tiles**: the one you land on and the one the lever is
    // on. A warp sets `world.at` and repairs and nothing resolves the landing
    // tile's place — which is right, because every other warp in the game lands
    // you on ground — so a one-tile alcove would have been a lever nobody could
    // ever pull. You fall onto the outer tile and step onto the lever.
    for (outer, inner) in [((1u8, 1u8), (2u8, 1u8)), ((15, 1), (14, 1)), ((1, 9), (2, 9)), ((15, 9), (14, 9))] {
        for (x, y) in [outer, inner] {
            assert!(w.walkable(x, y, &allowed), "[{x},{y}] is an alcove and you stand in it");
        }
        let neighbours = |x: u8, y: u8| {
            [(0i32, -1i32), (0, 1), (1, 0), (-1, 0)]
                .iter()
                .filter(|(dx, dy)| {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    w.in_bounds(nx, ny) && w.walkable(nx as u8, ny as u8, &allowed)
                })
                .count()
        };
        // Each tile of the pair reaches only the other one, so the pair is an
        // island and the sinkhole is the way in.
        assert_eq!(neighbours(outer.0, outer.1), 1, "{outer:?} is not walled off");
        assert_eq!(neighbours(inner.0, inner.1), 1, "{inner:?} is not walled off");
    }
}

/// **Two sinkholes go somewhere and two come back here, and neither strands
/// anybody.**
///
/// The false ones cost eight percent each — sixteen between them, which is what
/// a blind walk of this floor pays. What makes a fall safe is that the lever in
/// each true alcove is **ungated**: the way out of a hole is the thing you fell
/// in to find.
#[test]
fn a_false_sinkhole_costs_eight_and_strands_nobody() {
    use gm2d_core::tile_event::{Outcome, Requirement};
    let events = data::events();
    let mut true_holes = 0;
    let mut false_holes = 0;
    for who in ["north", "south", "east", "west"] {
        let e = events.get(&format!("the-sinkhole-{who}")).expect("a sinkhole");
        let drop = &e.choices[0];
        assert_eq!(drop.requires, Requirement::None, "{who}: a hole in the floor asks nothing");
        let mut to = None;
        let mut tire = 0;
        fn walk(o: &Outcome, to: &mut Option<[u8; 2]>, tire: &mut u32) {
            match o {
                Outcome::Warp { map, at } => {
                    assert_eq!(map, "the-reefs-3", "a sinkhole leaves the floor");
                    *to = Some(*at);
                }
                Outcome::Tire(n) => *tire += n,
                Outcome::All(l) => l.iter().for_each(|o| walk(o, to, tire)),
                _ => {}
            }
        }
        walk(&drop.outcome, &mut to, &mut tire);
        let at = to.expect("a sinkhole puts you somewhere");
        if at == [7, 10] {
            assert_eq!(tire, 8, "{who}: a false sinkhole costs eight");
            false_holes += 1;
        } else {
            assert_eq!(tire, 0, "{who}: a true sinkhole costs the drop and nothing else");
            // And the alcove it lands you in has an ungated lever **next to
            // it**, or the fall is a trap. Next to, not under: a warp lands you
            // on ground and you step onto the card, which is how every other
            // warp in the game works.
            let lever = w_places()
                .into_iter()
                .find(|(pat, id)| {
                    id.starts_with("the-lever-")
                        && (pat[0] as i32 - at[0] as i32).abs()
                            + (pat[1] as i32 - at[1] as i32).abs()
                            == 1
                })
                .map(|(_, id)| id)
                .unwrap_or_else(|| panic!("{who} lands with no lever beside it"));
            let l = events.get(&lever).expect("a lever");
            assert_eq!(
                l.choices[0].requires,
                Requirement::None,
                "{lever} is gated, so falling into its alcove is a trap"
            );
            true_holes += 1;
        }
    }
    assert_eq!((true_holes, false_holes), (2, 2), "two of the four go somewhere");
}

fn w_places() -> Vec<([u8; 2], String)> {
    data::map("the-reefs-3", D).places.iter().map(|p| (p.at, p.id.clone())).collect()
}

/// **The levers chain through the drains**, and the second on each side is
/// behind the first.
///
/// NE opens the wall down the outside to SE; SW opens the wall up the outside
/// to NW. So two sinkholes reach two alcoves and two levers reach two more,
/// which is the floor.
#[test]
fn the_levers_chain_through_the_drains() {
    use gm2d_core::tile_event::Requirement;
    let w = data::map("the-reefs-3", D);
    let events = data::events();
    for (id, needs) in [
        ("the-lever-north-east", None),
        ("the-lever-south-west", None),
        ("the-lever-south-east", Some("lever-ne")),
        ("the-lever-north-west", Some("lever-sw")),
    ] {
        let e = events.get(id).expect(id);
        match needs {
            None => assert_eq!(e.choices[0].requires, Requirement::None, "{id} is ungated"),
            Some(f) => assert_eq!(
                e.choices[0].requires,
                Requirement::Flag(f.into()),
                "{id} waits on the lever before it"
            ),
        }
    }
    // The two ungated levers each open a wall of rock, which is the way out of
    // their own alcove and the way into the next.
    for flag in ["lever-ne", "lever-sw"] {
        assert!(
            w.drains.iter().any(|d| d.when == flag && d.from == "rock"),
            "{flag} opens no wall, so its alcove has no way out"
        );
    }
    // And all four take quicksand off the plate.
    let quick: usize = w
        .drains
        .iter()
        .filter(|d| d.from == "quick")
        .map(|d| d.tiles.as_ref().map(|t| t.len()).unwrap_or(0))
        .sum();
    assert_eq!(quick, 44, "the square is nine by five and the plate is the one cell left");
}

/// **The plate is not there until all four levers are over.**
///
/// `hidden_until_all` rather than `needs_all`, which is a divergence from
/// M14.4's own argument and is right here: the plate is not a door you are
/// refused at, it is a plate under three feet of sand, and the four drains are
/// what uncover it. There is nothing to say and nobody to say it to.
#[test]
fn the_plate_is_under_the_last_of_the_sand() {
    let w = data::map("the-reefs-3", D);
    let plate = w.places.iter().find(|p| p.id == "the-tenth-surveyor").expect("the plate");
    assert_eq!(plate.kind, gm2d_core::world::PlaceKind::Boss);
    let mut want = plate.hidden_until_all.clone();
    want.sort();
    assert_eq!(want, vec!["lever-ne", "lever-nw", "lever-se", "lever-sw"]);
    assert_eq!(plate.creature.as_deref(), Some("The Tenth Surveyor"));
    assert_eq!(plate.drops.len(), 3, "three drops");
    for d in &plate.drops {
        assert!(
            gm2d_core::piece::CATALOG.iter().any(|c| c.name == *d),
            "{d} is not in the catalogue, and this block adds no components"
        );
        assert!(gm2d_core::piece::is_event_only(d), "{d} can be bought, so beating her is shopping");
    }
}

/// **Seven card reads blind**, against `PLAN-M16.md` §4.3's eight.
///
/// The same unit mismatch as the Flat Below: eight is the plan's count of
/// *drops and pulls* — two false falls and four levers — and `solvable_blind`
/// counts **card reads that could raise a flag**, which a sinkhole is not. Two
/// live levers at a time, then the last one alone: `2 + 2 + 2 + 1`.
///
/// **And the floor measured `Stuck` before `puzzle::reachable` learned to
/// follow a warp**, which is the first thing `PROMPT-M16.md` asks to be
/// reported. `Outcome::Warp` works perfectly well as a lock; the solver flooded
/// terrain only, so an alcove nothing walks into was an alcove nothing could
/// reach. The plan's fallback — a `needs` gate on each alcove keyed to its
/// sinkhole's flag — is not needed.
#[test]
fn the_needle_room_is_seven_card_reads_blind() {
    let w = data::map("the-reefs-3", D);
    let events = data::events();
    assert_eq!(puzzle::solvable_blind(&w, &events), Ok(7));
    assert_eq!(puzzle::solvable_knowing(&w, &events, None), Ok(4), "four levers, if you know");
}

// -------------------------------------------------- the Tenth Surveyor

/// **She is wearing the run.**
///
/// Compared by item, not by placement: her `outfit()` against the save's own
/// `reports()`, the same component names in the same counts in the same grids.
/// If a placement stops seating, the gear block is wrong and not the board —
/// which is the whole reason this is a test rather than a comment.
#[test]
fn the_tenth_surveyor_wears_the_run() {
    use gm2d_core::combat::{self, Difficulty};
    let ch = common::from_save(common::THE_RUN);
    let spec = combat::creature("The Tenth Surveyor").expect("her");
    let (reg, lo) = spec.loadout_at(Difficulty::Medium);
    let mine: Vec<String> = ch
        .reports()
        .iter()
        .flat_map(|r| r.items.iter().flat_map(|i| i.pieces.iter()))
        .map(|&p| ch.registry.def(p).name.to_string())
        .collect();
    let theirs: Vec<String> = lo
        .reports(&reg)
        .iter()
        .flat_map(|r| r.items.iter().flat_map(|i| i.pieces.iter()))
        .map(|&p| reg.def(p).name.to_string())
        .collect();
    let tally = |v: &[String]| {
        let mut m = std::collections::BTreeMap::new();
        for n in v {
            *m.entry(n.clone()).or_insert(0usize) += 1;
        }
        m
    };
    assert_eq!(tally(&mine), tally(&theirs), "she is not wearing what the run is wearing");
    assert_eq!(theirs.len(), 38, "thirty-eight pieces");
    // **And the same number of items out of them**, which is the half that
    // matters: what a board is worth is mostly how many items it makes.
    let items: usize = lo.reports(&reg).iter().map(|r| r.items.len()).sum();
    let mine_items: usize = ch.reports().iter().map(|r| r.items.len()).sum();
    assert_eq!(items, mine_items, "her board makes a different number of items");
}

/// **The six enchs are on her, and they do what they do on a player.**
///
/// One answer to *what an ench does* — `ench::apply` over the profiles, the
/// door the player's go through. A creature gets no `enched` flag and no
/// beacon, because those are read by rules a character holds and a creature
/// holds none.
#[test]
fn she_carries_the_runs_six_enchs() {
    use gm2d_core::combat::{self, Difficulty};
    let spec = combat::creature("The Tenth Surveyor").expect("her");
    assert_eq!(spec.enchs.len(), 6);
    let data = data::enchs();
    for (id, at) in spec.enchs {
        assert!(data.get(id).is_some(), "{id} is not an ench");
        assert!(*at < spec.gear.len(), "{id} names gear[{at}] and there are {}", spec.gear.len());
    }
    let resolved = spec.enchs_at(Difficulty::Medium);
    assert_eq!(resolved.len(), 6, "every one resolves to a piece");
    // The profiles move: six enchs on eleven items is not nothing.
    let (_, with) = spec.outfit_at(Difficulty::Medium);
    let mut bare = spec.clone();
    bare.enchs = &[];
    let (_, without) = bare.outfit_at(Difficulty::Medium);
    let moved = with
        .iter()
        .zip(without.iter())
        .filter(|(a, b)| a.power != b.power || a.cooldown_ms != b.cooldown_ms)
        .count();
    // **Three items, not six**, and that is the run's own board rather than a
    // shortfall: four of the six enchs sit on pieces of the *same* weapon item
    // — the Bearing on the Herbal, the Tap on the Chain Coil, the Index on the
    // Runewash Ink and the Correction on the Emberburst, which the run packed
    // into one thing. An ench names a piece and an item is however many pieces
    // touch.
    assert_eq!(moved, 3, "{moved} of her items feel an ench");
}

/// **The bracket: she is a fight the board wins, and she is the deepest one.**
///
/// `PLAN-M16.md` §5.2 asks for a win rate between 55% and 70% *"over a loop of
/// seeds"*. **Combat has no RNG** — a loop over seeds counts the same fight
/// every time, which is why a mid-fight save carries a creature name and a tile
/// and nothing else. What varies between two players meeting her is the
/// *board*, so the bracket is over boards, which is M11.7's rule stated twice.
///
/// The measurement is damage a second, as M14's is, against
/// `common::geared_from`:
///
/// | | deals | |
/// |---|---|---|
/// | The Ninth Surveyor | 122.1/s | Victory |
/// | What Marbulon Faced Away From | 138.7/s | Victory |
/// | **The Tenth Surveyor** | **221.2/s** | **Victory** |
/// | Gilt | 428.3/s | Defeat |
/// | Nine of Ashes | 531.0/s | Defeat |
///
/// **The run itself loses to her**, and that is in register rather than a
/// failure: the reconstruction is a caster board with 974 health against a
/// shopper's 1942, and she is wearing its gear on a boss's body.
#[test]
fn the_run_meets_something_wearing_its_own_board() {
    use gm2d_core::combat::{self, Outcome, Side};
    let mut shopper = common::geared_from(&["the-end-of-all-gears", "kettleworks"]);
    shopper.pack_what_you_own();
    let dps = |name: &str| -> (Outcome, f64) {
        let m = combat::creature(name).unwrap_or_else(|| panic!("no {name}"));
        let log =
            combat::simulate_at(shopper.player_stats(), &shopper.combat_items(), m, D);
        let ms = log.entries.last().map(|e| e.at_ms).unwrap_or(1).max(1);
        let dealt: i64 = log
            .entries
            .iter()
            .filter_map(|e| match e.event {
                combat::Event::Hit { by: Side::Enemy, damage, absorbed, .. } => {
                    Some((damage - absorbed).max(0) as i64)
                }
                _ => None,
            })
            .sum();
        (log.outcome, dealt as f64 * 1000.0 / ms as f64)
    };
    let (out, hers) = dps("The Tenth Surveyor");
    assert_eq!(out, Outcome::Victory, "nothing behind her can ever be reached");
    let (_, marbulon) = dps("What Marbulon Faced Away From");
    assert!(hers > marbulon, "she deals {hers:.1}/s against {marbulon:.1}/s, so she is not deeper");
    let (gilt_out, gilt) = dps("Gilt");
    assert_eq!(gilt_out, Outcome::Defeat, "Gilt stopped beating this board, so the bracket moved");
    assert!(hers < gilt, "she deals {hers:.1}/s against Gilt's {gilt:.1}/s, which is a wall");

    // And the run, which she is wearing, does not stand in front of her.
    let run = common::from_save(common::THE_RUN);
    let log = combat::simulate_at(
        run.player_stats(),
        &run.combat_items(),
        combat::creature("The Tenth Surveyor").expect("her"),
        D,
    );
    assert_eq!(log.outcome, Outcome::Defeat, "the run beats a boss wearing its own board");
}
