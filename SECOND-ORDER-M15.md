# SECOND-ORDER-M15.md — the notebook

Written when noticed, not at the end. M13.9 executed seven of these and three
of them were actually wrong; M14.6 did the same. **M15.5 is this file as a
worklist.**

Each row: what was noticed, why it matters, and whether it is a candidate for
the last milestone.

---

## 1. Eight of the nine boss creatures also stand in region pools — M15.0

**Noticed doing M15.0's recon**, before anything was written, and it reverses a
line in the plan.

`PLAN-M15.md` §1.5 says *"A boss is never instant. `rout` refuses one by name
(`boss_at`)"* — and `boss_at` does not refuse by name, it refuses **by tile**.
The distinction is invisible until you count:

    bosses (9): Cog Priest, Gallowglass, Iron Abbot, Mire Behemoth,
                Rust Colossus, Sootmother, The Iron Choir,
                The Ninth Surveyor, What Marbulon Faced Away From
    also in a region pool (8): all of them but Mire Behemoth

So a refusal at the *name* would take eight of the game's sixty creatures off
the menu for ever, seven of which a player meets in a field long before they
meet the tile. **The refusal is the tile's**, which is what `rout` already does
and is what the plan meant by *"for the same reason"*.

**Candidate: no.** It is a divergence rather than a worklist row, recorded in
`CLAUDE.md`'s table as 15.1. What it *creates* is a second-order row of its own,
below.

## 2. A marked creature behaves differently depending on where you met it — M15.0

Falls straight out of row 1: mark the Rust Colossus and it settles where it
stands in the Kolok Downs, and opens the fight screen on the Cave's floor. That
is correct and it is also surprising, and **a rule the player cannot see is a
rule they report as a bug** — this file's oldest sentence.

So the menu's sentence has to say it, in M15.2, and not only *"you cannot watch
it"*.

**Candidate: yes** — check in M15.5 that the sentence actually says it and that
the browser gate reads it.

## 3. The walk home had one caller and now has two — M15.1

`settle_fight` in the shim held forty lines of *where a defeat puts you*, with a
comment arguing correctly that it belongs there rather than in `fight::settle`
because the world owns where the player is. An instant battle loses the same
way, so a second copy would have been a second answer — and the failure it makes
is a player losing an unwatched fight and coming back standing on the tile that
killed them, which is `f114cdf`'s report arriving again through a door nobody
had built yet.

It is `walk_home(g)` now, one function, two callers.

**Candidate: no.** Done in the milestone that created the second caller.

## 4. `paintPanel` has to be told again, and this is the fourth instance — M15.1

The rule in `CLAUDE.md` reads: *a page that draws a world has to be told which
world, every time it can have changed.* Three instances were listed — a defeat
carrying you to another map, a save being restored, a choice being taken. **An
instant battle that is lost is the fourth**, and it is the first one that
happens inside `walk()` itself: the panel is painted near the top of that
function against the map you stepped on, and the settlement afterwards can move
you two maps away.

Handled where it happens (`if (r.instant.sent_home)`), and the rule's list in
`CLAUDE.md` wants the fourth row.

**Candidate: yes** — write the row in M15.5, and check whether the *rout* path
has the same hole. A rout cannot lose, so it should not; worth confirming rather
than assuming.

## 5. The gate prints 66 `ok:` lines per engine, and `CLAUDE.md` says 78 — M15.0

Not a fault, but `CLAUDE.md`'s *"78 `ok:` lines, 3 engines"* reads as a per-engine
count and is a **total**: 5 M14.5 lines and 1 per-engine "walked the gate" line
against 60 shared ones is 66 in one engine and 78 across three. Somebody
budgeting a check count off that sentence will be wrong by twelve.

**Candidate: yes** — one sentence in `CLAUDE.md` in M15.5.

## 6. Three plants in a row, and every one hit a documented trap — M15.2

The browser check for the menu failed four times before it passed, and **not one
of the four was a bug in the feature**. Each was a trap `CLAUDE.md` already
names, met in the order a new check meets them:

| what happened | which trap |
|---|---|
| `#instant-open` visible, click timed out for thirty seconds | *the class fork is the one screen that does not take Escape* — `xp: 4000` with no class opens it, and every click after that lands in a modal |
| `#preset` timed out | Auto-pack is on the **fight** screen, a town away; the plant seats the two starting components by hand instead, blade turned, which is the M4 soft-lock in miniature |
| `next(...)` raised on `b[0] == "Weapon"` | `save::slot_name` writes `"weapon"`, lower case |
| 240 steps and an empty tape | keypresses go nowhere without `#map` focused, **and** `tape(page)[before:]` slices a strip that is capped at `TAPE` lines, so it goes on returning the last one or two however many fights have happened |

The last one is the only one worth generalising and it is a **new** shape: *a
slice of a capped list is a comparison that quietly stops being about anything.*
It is the "compares zero with zero" failure with a scrollback in it — the check
would have gone on passing on a build where the receipt landed and then scrolled
away.

**Candidate: yes** — grep `drive.py` for other `tape(page)[` slices in M15.5.
There is at least one shape like it and it may be right or may be the same
mistake.

## 7. A `git checkout` on a file with uncommitted work — M15.2

Reverting a deliberate break with `git checkout crates/wasm/src/lib.rs` took the
milestone's *own* uncommitted exports with it. Twenty minutes to notice and
five to redo, and the reason it was not caught immediately is that the shim
compiles fine without them — the page just stops having a menu.

The habit is the fix: **copy the file before breaking it and copy it back**,
which is what the rest of this block's negative tests do. No code change.

**Candidate: no.** Recorded because it is the kind of thing that reads as a
mystery the second time.

## 8. Line continuations inside a Rust string, edited by script — M15.2

`what_a_mark_costs` was written with `\` continuations inside its string
literals. A later scripted edit rewrote the file and the backslashes went, which
is invisible in a diff read quickly and turns into **runs of eleven spaces in
the middle of a player-facing sentence.** Caught by reading the function, not by
a test: nothing in the suite reads that string, and the browser check only looks
for the word *boss*.

Rewritten with one string per line. **Candidate: yes** — a tone lint for
`  ` (two or more spaces) inside a player-facing string would have caught it,
and `tests/tone.rs` is where the eight machine-checkable TONE rules already
live.

## 9. `make play` is not the instrument the level-five band was set with — M15.3

`PLAN-M15.md` §2.3 asks for the 150-fight bound to be *"a band on the walk,
130–170 wins, exactly the way level 5's 25–35 already is."* **It is not the way
level five's is**, and the live walk proves it in its own first forty lines:

    level  5: 38 wins        <- outside its own 25-35 band
    level 14: 131 wins
    level 16: 210 wins

`level_five_lands_where_the_plan_says` does not use `make play`. It walks a
**fixed east-west patrol on the pit road over nine seeds and asserts on the
mean** — a controlled measurement of the map's pacing. `make play` wanders, goes
north, loses, and drops what it is carrying; it misses the level-five band by
three fights and nobody has ever thought that a fault.

So the 130–170 band has no instrument that can measure it: the pit road cannot
level anybody to twenty, and the walker banks a fraction of what it earns
(22,480 earned, 1,439 banked on the M14 run). **What M15.3 used instead** is a
replay of the recorded walk's own payouts, which measures the curve against real
fights and separates it from the walker's banking problem.

**Candidate: yes.** Two things worth doing in M15.5: say this in `CLAUDE.md`
beside the *`make play` and `make test-ui` are different tools* note, and
consider whether the replay-of-payouts measurement should be a checked-in tool
rather than a script that lived in one session.

## 10. The curve makes the walker a different animal — M15.3

Under the old curve the M14 walk plateaued at **level 11 after 1,151 wins**.
Under the new one it is at **level 16 after 210** and still climbing. That is
the flattening doing exactly what it was asked to do, and it is also a warning:
every number in `CLAUDE.md`'s table that was measured against a walk — *342
wins, 170 losses, level 14, 4,406 steps* — is now measured against a different
game.

**Candidate: yes** — the transcript numbers in `CLAUDE.md` want restating in
M15.5, with the old ones marked as the old curve's rather than quietly replaced.

## 11. `StockGate::TreesFinished` went from two users to one — M15.4

Not a fault, and worth knowing: the second paper's gate coming off leaves the
expert paper as the only thing in the shipped game that constructs a
`TreesFinished`. `a_gate_counts_and_says_so` tests the type directly so it is
not vacuous, and `the_expert_paper_still_wants_two_finished_trees` asserts both
halves of the pair at once so they cannot drift.

**Candidate: no.** Handled in the milestone.

## 12. Spike's van is now the only gate on a second class — M15.4

`hidden_until_level: 10`, and with the tree gate gone it is all that stands
between a new character and a second class besides five thousand Fnorp.
`PLAN-M15.md` §3.2 flags this as the one place the change could go further than
the ask. Left as it is and said so in the commit.

**Candidate: no — it is the human's.**

## 13. The replay-of-payouts instrument reproduces the level-five band; the raw walk does not — M15.3/M15.5

Row 9 said `make play` is not the instrument the 25–35 band was set with. Here
is that stated as evidence rather than as an argument, over both transcripts:

| | level 5, raw walk | level 5, banking what it earned |
|---|---|---|
| `m14-a-new-game.txt` | — | **27 wins** |
| `m15.3.txt` | **38 wins** | **28 wins** |

**The replay lands inside 25–35 on both and the raw walk misses it by three.**
That is the instrument validating itself against the one band this game already
had, which is the only way a new instrument earns the right to answer a
question the old one cannot.

## 14. Two walks disagree about level twenty by 28%, and the band is tighter than that — M15.3

The same replay, asked about twenty:

| route | wins to level 20 | mean xp/win over the first 150 |
|---|---|---|
| `m14-a-new-game.txt` | **150** | 28.7 |
| `m15.3.txt` | **192** | 16.5 |

Both are the shipped map on the shipped curve. What differs is **where the
fights were had** — the M14 walk pushed north into the Treyway sooner; this one
spent longer in the pit and then jammed against the Drambus Stack.

So `PLAN-M15.md` §2.3's band of 130–170 is **±13% around a measurement whose own
run-to-run variance is 28%.** No target satisfies both routes: 4,300 gives 150
and 192, and dropping to 3,500 would give about 130 and 172. `reach(20) = 4,300`
is kept because it is the M14 route's 150 exactly, that route is the one the
plan measured against, and `CLAUDE.md` already says in as many words that **the
walker is not deterministic and two runs of it disagree** — retuning a shipped
constant off one stuck run is the thing that sentence exists to stop.

**Candidate: no — it is the human's.** The number is defensible and the band is
not measurable to the precision it was written at. Flagged in `HANDOFF-M15.md`.

## 15. The new curve took the walker from level 11 to level 16 — M15.3

`make play` from a new game, same walker, same map:

| | old curve | new curve |
|---|---|---|
| level reached | **11**, after 1,151 wins | **16**, after 210 wins |
| where it stops | the Drambus Stack's fourth floor | the Drambus Stack |

Still the Stack, which is `PLAN.md` §6d row 3 unchanged — but five levels deeper
and five times sooner. Every walk-measured number in `CLAUDE.md` is now a number
about a different game, and they are restated in M15.5 rather than quietly
replaced.

---

# M15.5 — the worklist, and what it found

Six rows were marked **candidate: yes**. All six were executed and **two of them
turned up something that was actually wrong.**

| row | what it asked for | what happened |
|---|---|---|
| 2 | the menu's sentence has to say a boss is fought either way | **done in M15.2** — `what_a_mark_costs`'s fourth line, and the browser check reads it |
| 4 | check the rout path has the same `paintPanel` hole | **it does not.** `rout` touches none of `world.at`, `world.map`, `sent_home`, `forget`, `drop_carried` or `leave_the_sitting` — it cannot move you, so there is nothing to repaint. Confirmed rather than assumed, and `CLAUDE.md`'s rule has a fourth instance rather than a fifth |
| 5 | `CLAUDE.md` says 78 `ok:` lines and it reads as per-engine | **it was a total.** 67 in any one engine and 81 across three, since M15.2. Said in the row now |
| 6 | grep `drive.py` for other capped-list slices | **clean.** The one other slice is `tape(page)[-3:]` — the *last* few, which is correct on a capped list, and it is in a failure message rather than an assertion |
| 8 | a lint for runs of spaces in a player-facing string | **found a three-block-old lie** — see below |
| 9, 10, 14 | `make play` is not the band's instrument; restate the walk numbers | **done** — the note is in `CLAUDE.md` beside *`make test-ui` and `make play` are different tools*, and every walk-measured number in the table says which curve it is about |

## Row 8 is the one that paid for the milestone

The lint was written for a formatting nit. Nine strings in the engine carried
runs of eighteen to twenty-six spaces — the wreckage of a `\` line continuation
that a scripted edit ate at some point before this block. Chasing them meant
reading the sentences, and one of them was `Effect::GrowSlotRows`'s hover:

> A row is 6 more cells to pack the weapon grid with, granted out of turn — on
> top of the row that grid gets when **the level rotation reaches it**. No grid
> goes past 8 rows.

**M12.3 deleted the rotation.** `ROTATION`, `rows_for` and `grows_at` are all
gone, every frame starts at three rows and stays there, and *"granted out of
turn"* is meaningless when there is no turn. Eleven skill nodes have been
describing the previous game, on hover, for three blocks.

It is the same failure as the `STARTER` comment `CLAUDE.md` quoted as live fact
for five blocks and the controls blurb M12.B deleted for saying *every level
adds a row to one frame* — **except this one reached a player.**

Two other things came out of writing it:

- **`prose()` had never walked the maps.** Every place `name`, `shut` and
  `prose` line in twenty-one map files was outside every tone lint this project
  has. 108 strings.
- **Nor the sentences the engine composes** — `Node::line`, `Node::detail`,
  every class and expert promise, `what_a_mark_costs`. 363 strings, and they are
  precisely the ones nobody proof-reads because nobody typed them.

`every_sentence_a_player_can_read()` is the widened corpus and it is deliberately
**separate from `prose()`** rather than replacing it: rule 13 says content
speaks the book's language and rule 13a says a spec does not, so one list run
through both sets of rules would fail the thing it exists to protect.
`the_widened_corpus_actually_reaches_the_maps_and_the_engine` is what stops the
new list going quietly empty.
