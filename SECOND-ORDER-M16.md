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
