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
