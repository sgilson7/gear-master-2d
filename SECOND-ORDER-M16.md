# SECOND-ORDER-M16.md — the notebook

*Written while the work is done, not at the end of it. M13's convention, kept
because it paid for itself: seven of M13's rows were marked as candidates, all
seven were executed in M13.9, and three of them turned up something that was
actually wrong.*

**Marked `worklist` is a row the last milestone of the combined block executes.**

---

## Rows

| # | row | kind | status |
|---|---|---|---|
| 1 | **The save did not arrive.** `PROMPT-M16.md` says to place `testing/saves/the-run-20260910.json` before starting and to treat it as the human's own file. It was not in the tree. `PLAN-M16.md` §5.1 transcribes the board in full, so the fixture is **reconstructed** from the transcription by `crates/lab/src/mkrun.rs` and checked in. `the_save_opens_at_forty_five` and `the_tenth_surveyor_wears_the_run` are what make that honest: if the transcription and the reconstruction disagree, two tests say so. | divergence | done (M16.0) |
| 2 | **A board's placement order is not item order, so `MonsterSpec.items` cannot be written over §5.1's list.** The run's first helmet item is gear entries 0, 3 and 5 and its second is 1, 2 and 4 — interleaved. `Character::item_partition` therefore returns **the gear in item order and the chunk list**, not the chunk list alone. Reordering is free: a placement carries its own absolute cell. | divergence | done (M16.0) |
| 3 | **Three of the run's weapon pieces cannot be in an assembled item.** The bottom row of §5.1's weapon board is Accessory + Ink + Alignment with no core, so it assembles under no locking discipline. The search over all 4,096 lock masks tops out at 9 of 12 on that grid; the other four grids are 6/6, 6/6, 8/8 and 6/6. Eleven assembled items out of twelve groups. | finding | done (M16.0) |
| 4 | **`PLAN-M16.md` §5.3's curse-cap numbers do not reproduce, in either direction.** The plan says two creatures are past the cap, the Ninth Surveyor at 114 and What Marbulon Faced Away From at 120. Measured: the Ninth Surveyor is at **88** — under it — Marbulon is at **102**, and **twenty-three** creatures are past, up to Nine of Ashes at 233. | finding | done (M16.0) |
| 5 | **And nobody had asked the mind lane the same question.** Thirty-one of the sixty creatures are at or past a hundred mind resist, **every deep boss among them** — Marbulon 128, the Ninth Surveyor 126, The Rust Parliament 200, Gilt 190. M16.4 adds a base class whose whole promise is that lane. The fix is `stats::LANE_CAP` at 95, which is `RESIST_CAP`'s own argument applied to the two lanes that now have classes behind them — **not** the plan's re-dressing, because re-dressing a quarter of the ladder to move a constant is *ease a pool, not a creature* two levels up. Nothing under 95 moved. | divergence | done (M16.0) |
| 6 | **Three unit tests pinned "fully resisted, never lands"** and were updated rather than deleted, with the reason at each. A fourth thing to watch: `explain::defences_of` now prints three ceilings where it printed two. | finding | done (M16.0) |
| 7 | `crates/lab/src/probe.rs` is a scratch recon binary. It should either earn a name and a doc comment or be deleted before the block closes. | worklist | open |
| 8 | **§4.1's and §4.2's ASCII rows are not all the same width** — floor one's mix 14 and 15, floor two's mix 14 and 15, floor three's mix 14 and 15. Redrawn at a consistent width in each case: 16×15, 16×8, 15×12. The rooms and necks the annotations name are all preserved. | divergence | done (M16.1) |
| 9 | **`solvable_blind` on the Flat Below is 36, and §6 asks for 9.** Nine is the plan's own §4.1 count of *pulls*; `solvable_blind` counts **card reads**, which is M14's unit and the unit the Cairnfield's 45 is in. Nine stakes in three gated groups is 36, not the 18 a first estimate gives, because the adversary opens each band early and leaves its two wrong neighbours live. Both numbers are asserted, each in its own unit. | divergence | done (M16.1) |
| 10 | **The atlas cannot be knowledge the engine does not read.** §4.1 has the atlas raise `read-band-x` on each stake so *the card's prose then says the one on the right*; prose is static and `every_flag_an_event_sets_is_read_by_something` refuses a flag nothing consults — the `Outcome::Xp` bug. So the reading is on the **sheet**, where a survey belongs, and it opens the three bays that hold: one card instead of three stakes. The plan's fifteen fatigue is charged, in one go. | divergence | done (M16.1) |
| 11 | **A no-op choice can be behind a requirement, and `no_choice_that_does_nothing_spends_a_door` did not know it.** The compass reading is `Outcome::Nothing` and wants an instrument, so a fresh game could not take it and the lint failed on the refusal rather than on the rule. `common::with_instrument` is the fixture that fixes it — the parts read off the recipe table, never a list of names. | finding | done (M16.1) |
| 12 | The pockets are `PlaceKind::Boss` with no drops, so a pocket is a **certainty** rather than a 260‰ roll. Six boss tiles on one floor takes the game's boss-tile count from 9 to 15, which M16.3's *the boss refusal is the tile's and not the name's* rule (`fight::instant`) now applies to. **Worth checking in M16.3** that marking an Iron Abbot met in a pocket does not make the Sands' own Iron Abbot instant-able in a way that reads wrong. | worklist | open |
