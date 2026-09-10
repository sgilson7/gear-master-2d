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
