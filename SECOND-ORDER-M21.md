# SECOND-ORDER-M21 — the notebook

Written as it goes. `M21.12` onward are read off this file: every row marked
`open` is work somebody has to do or a finding somebody has to write down, and
the last milestones of this block are whatever closes them.

Format is `SECOND-ORDER-M20.md`'s — numbered rows, a bold first sentence, and
`open` or `closed`.

| # | row | state |
|---|---|---|
| 1 | **`Game::eq` had never learned about M20's bags.** That operator is hand-written and lists fields by name, and the comment beside `banked` says exactly why that matters — *a save that dropped it would round-trip green*. The larder, the glass, the pack, what you have drunk and what you are were all added in M20 and **none of them was added to `eq`**, so `tests/save.rs` could not have caught a loader that lost somebody's ingredients. Closed by adding all five beside the Plot's drawer, which is the sixth. The general form is worth keeping: **a new bag is three edits, not two — the struct, the save, and the comparison.** | closed |
| 2 | **The drawer went on `Character` and the plan said `WorldState`.** Every other bag in the game is on the character — `owned`, `banked`, `larder`, `potions`, `retort` — and `WorldState` holds what has *happened*: what is answered, what has drained, what a shop has sold. A drawer in the world would be the only bag in the game not on the person holding it. Divergence, recorded in `CLAUDE.md`'s table. | closed |
| 3 | **Three of the twenty-four seed-and-bed pairs fit nowhere without a turn.** Measured before the beds were authored, which is what the recon step is for: `toad-ichor` and `tallow-drip` fit no anchor in Kettleworks' gapped row and `reef-salt` fits none in the pit's round bed. `PLAN-M21.md` §M21.1 gives `Game::plant(town, seed, at)` with no turn. A crop turns now — `Crop { seed, at, stage, turn }`, which is `brew::Seat`'s own shape, because the retort has tried every rotation since M20 and *a heuristic that refuses an arrangement somebody can see with their eyes* is the thing `brew::fits` exists not to be. Divergence recorded. | closed |
| 4 | **`git checkout <file>` in the middle of a negative test threw away uncommitted work.** Backing a file up with `cp` before breaking it is the habit; on the fifth break I reached for `git checkout` instead, and `plot.rs` went back to its M21.0 commit — losing the turn on a crop, the harvest yield, the clock and both refusal helpers. Rebuilt from the conversation, no loss beyond the time. The rule is narrow and worth keeping: **while a file has uncommitted work in it, the only safe undo is the copy you took yourself.** | closed |
| 5 | **A paint that refreshes and a refresh that paints is a stack overflow**, and it does not announce itself. `Board.refresh()` ends by calling `onchange`, so wiring `onchange = () => paintBed()` while `paintBed` calls `refresh()` recurses — and what it looked like from outside was **the town screen never opening**, with no console error and the bed's own sentence painted correctly. The bench has split *open* from *paint* since M20 and the reason was not written down; it is now. Worth watching for the Kennel's yard and the Stall's shelf, which are the same `Board` and the same two functions. | closed |

