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
