# TRIAGE-M11.md — everything the block's own sweep found

Written at M11.7, before anybody outside was asked to play it. Every line has a
**severity**, a **cost** and a **disposition**, and the rule the plan sets is
that blockers are fixed here and the rest is carried into the playtest *openly*
— listed in the agent brief's "known" section, so the run spends its attention
on the unknown.

**Severity** is one of three, and they are about the player rather than about
the code:

- **blocks** — a player cannot get past it, or gets past it and the game is
  wrong afterwards.
- **wrong but survivable** — it is not what was meant, and you can play on.
- **cosmetic** — it reads badly, costs nothing.

**Cost** is what fixing it would take, in the only unit that has ever been
accurate here: whether it is content, a number, a function, or a design
decision that is the human's.

---

## Fixed in M11.7

| # | severity | what | cost | disposition |
|---|---|---|---|---|
| 1 | **blocks** | **The tower could not be dropped.** The second floor's boss (The Tallow Saint, 1223) beat the best board the game hands out — both shelves bought out, every errand reward, every set piece, auto-packed. So the tower could not come down, the lake could not drain, and the ending was unreachable. 597 tests were green through it, which is the M4 soft-lock's exact shape one block out. | content | **Fixed.** Floor two's boss is Rimefather. And the thing that would have caught it is now a test: `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten`. |
| 2 | **blocks** | **Seven of nine new pools had a most-drawn creature the same board could not beat.** `draw_enemy` weights a pool so its *easiest* member is its commonest, so this was not the region's teeth — it was the fight you meet three times in five. Measured, not guessed: the whole ladder simulated against `common::geared_from`. | content | **Fixed.** Every pool retuned off the measurement. The same lint covers it. |
| 3 | **blocks** | **A Living Earth was 754 wins.** Bone Cantor became the hardest of three in the Stack's Shadow, so it became the rarest, so the golem's parts were behind the rarest fight on the map. `PLAN.md` §6b row 1, paid a third time. | content | **Fixed.** A fourth creature stands above it; the earth is twelve wins. |
| 4 | **wrong but survivable** | **The walk could not rest before going under the lake.** Under the lake it turned round at forty percent worn; on the Bambulon side nothing stopped it walking straight back down. One run thrashed the grating six thousand times, finished pinned at the sixty percent fatigue cap with 382 health, and carried 25,237 experience it could never bank. | the walker | **Fixed.** A branch that sends you somewhere has to agree with the branch that sends you back. |
| 5 | **cosmetic** | Two strings broke `TONE.md`. `the-cold-anvil` said "a great many times" where rule 4 wants a number, and `the-nine-surveys` had "says only", which is rule 10's speech-tag adverb wearing a different hat. | prose | **Fixed** in the tone pass. |

## Carried into the playtest, openly

| # | severity | what | cost | disposition |
|---|---|---|---|---|
| 6 | **wrong but survivable** | **The round trip is the block's real cost.** There is one town past the door — Kettleworks — and a character who arrives under-geared cannot catch up there, because its shelf is the last shop in the game. Most of a run is spent walking between the pit and the tower. | design, and the human's | **Carried.** The obvious fixes are a shelf on the Treyway or a second town, and both are content decisions rather than bugs. In the brief's known list. |
| 7 | **wrong but survivable** | **The gear ceiling arrives before the difficulty ceiling.** Boards stop growing at level six and the shelves run out at Kettleworks, so everything past the door is fought with the same board — the only progression left is drops. A level-nineteen character and a level-fourteen one pack the same grid. | design, and the human's | **Carried.** It is why the pools had to be tuned against one fixed board rather than against a curve. |
| 8 | **wrong but survivable** | **A player can build all three instruments at once.** Six shards exist and the three recipes want six, and nothing stops all three sitting in the weapon grid together. `survey_kind` then takes whichever rule comes first, which is an arbitrary answer to a question the player thinks they are choosing. | a function, or a design decision | **Carried.** Making the reach ask *which* instrument is the better answer and is a screen; refusing the third is the cheap one. Neither is a bug today: the parts are scarce enough that nobody will have six shards before the ending. |
| 9 | **wrong but survivable** | **Auto-pack does not build an instrument, so a walk never surveys.** The run collects five map shards off the tower and a sixth under the lake, and packs none of them: Auto-pack seeds on a core and grows what improves a *rating*, and a compass rates worse than the blade it would replace. So the reach is content `make play` cannot reach, and the browser gate is the only thing that walks it. | a function, or nothing | **Carried.** It is the same shape as M10.3's finding that Auto-pack never bolts an ench on (`PLAN.md` §6c row 3), and the same answer: the button is not an optimiser and must not become one. What it might reasonably do is *offer*. |
| 10 | **cosmetic** | **`make play` still walks back and forth across the door.** Both sides now share a give-up guard and the churn is much smaller, but a losing walk still crosses once before it settles. | the walker | **Carried.** It is the harness and not the game. |
| 11 | **cosmetic** | **`PlaceKind::Door` has one user and `#ending` has one caller.** Both were unused for three milestones between M11.1 taking the ending off the wall and M11.4 putting one under the lake. | nothing | **Carried, as a note.** Stated so that scheduled dead code is not mistaken for forgotten dead code. |
| 12 | **wrong but survivable** | **The Toad set's widening makes the lake a shortcut.** `Rule::Wade` opens the whole body now, so a player carrying the set crosses West Bambulon diagonally in a way the map was drawn against. It is the reward working, and it also means the two crossings can be walked round on the water. | a number, or nothing | **Carried.** Notebook entry 3's watch; the playtest is the right place to find out whether it reads as clever or as broken. |

## What the sweep checked and found nothing

- **The eight traps in `HANDOFF.md` §5**, walked by hand against everything this
  block added. Locks, `PieceId`-by-name, `map()` never reaching for the game,
  every relative import stamped by pattern, no reused id, every planted browser
  check stripping all five grids, and `set_of` still the one answer.
- **The console**, in all three engines, through the whole gate: nothing above a
  log, and nothing off-origin.
- **Every derived number this block added has somewhere it is shown**: the
  floors left (in the drop's own paragraph), the stump (in the log), the survey
  and its three numbers (on the standing panel), the instrument's rule (on the
  sheet), what an instrument's part is for (on its card), and who fought the
  golem's fight (in the receipt).
- **`TONE.md`'s eight machine-checkable rules**, over 56 events, 19 errands and
  every place that speaks — plus a hand sweep of all 76 new player-facing
  strings against the rules a machine cannot check.
