# transcripts

Where `make play` output goes when it is worth keeping.

A transcript is not a test result. It is what one run of the game said, in the
order it said it, and its value is that a person can read it — which is the one
thing the suite cannot do. `testing/playthrough.py` writes it to stdout;
redirect it here when a run found something.

**M8.8's run is `m8.8.txt`.** It is the run that caught an Auto-pack seating the
starting kit for the whole game and a class fork opening underneath the town,
both of which 482 tests were green through. It ends at the bottom of the Great
Gear Cave rather than at the door — see `PLAN.md` §6a, row 6, which is what that
tells you.

**M9.4's run is `m9.4.txt`, and it is the first that reaches the ending.** 1,434
steps, 159 wins, 13 losses, level 14, five drops and no console errors. Getting
there took two fixes to the walker rather than to the game, and both are things
a player already knew: a road refused three times is a road you stop walking at,
and the way out of a dungeon is a target like any other. Before the first of
them it pressed north into the Slag Flats' crossing for nine thousand steps;
before the second it beat the Cave's boss, took the key, and wandered a
nine-by-five room for the rest of the run — which is what `m8.8.txt` ends on.

Read the first of those as a finding about the *game* too: the quest log was
pointing at an errand behind a shut road and saying nothing about it, which is
what `Guide::shut` fixed.

**M10.3's run is `m10.3.txt`, played as Top of the Bill.** 1,360 steps, 153 wins,
22 losses, level 13, and the class paid on 65 of 99 fights — which is the same
66% the engine reports over the ladder, from the other end.

Two things it found. The walk banked level ten and **was told** the van was
there, which is the fix: a place hidden until a level appears when you bank, the
map redraws, and a redraw is not a sentence — the run before this one reached
level twelve and never met him. And it never bolted the Swing on, because
Auto-pack does not use enchs and never has; that is `PLAN.md` §6c row 3.

**M11.1's run is `m11.1.txt`, and it is the first that goes past the door.**
2,385 steps, 277 wins, 30 losses, level 16 — through the wall onto the Treyway,
both roads on the far side read, and back. The door stopped being an ending in
this milestone, so the run's finishing condition changed with it: *crossed, read
both roads and came back* rather than *reached the ending*.

Four things it found, and three of them were in the game rather than in the
walker.

1. **A defeat that crosses maps left the page drawing the old one.** Home has
   been on another map since M8, and nothing on the loss path re-read the grid
   — so dying in the Cave put the canvas on a nine-by-five room with the player
   standing at (1, 18) of it. Seven call sites re-read the map by hand and an
   eighth did not. Fixed as a class: the cheap `position()` call carries the map
   id and `paintPanel` compares it, so every path that moves anybody is covered.
2. **Two hardcoded lists of what the ground is.** The odds overlay skipped
   `rock` and `water` by name and the walker's pathfinder did the same, and the
   Treyway added `range` and `sea` — so the walker pathed straight through a
   mountain range and pressed into it. Both read core's own passability now.
3. **The walker did not know the van existed.** M10 put a `.screen` on the
   Verge road at level ten and `walk()` refuses every keypress while one is up,
   so a walk that reached level ten and stepped on [4, 6] stopped dead and
   reported the road home as unreachable. The M10.3 run never got there on foot,
   which is why it was M11.1 that found it.
4. **Crossing a border with everything in your pocket is how you lose it.** The
   first three runs walked through the door carrying five hundred to nine
   hundred experience, lost the first fight on the other side and were carried
   home having read nothing. The walker banks and mends before it crosses now,
   which is what a person does, and the run after that read both roads.

**M11.2's run is `m11.2.txt`.** 2,252 steps, 272 wins, 48 losses, level 16 —
through the door, west along the Treyway, into Kettleworks, both its errands
taken, and home. It is the first run that reaches a *third* map and the first
that takes an errand in a town that is not the pit's.

Two things it found.

1. **The walker's give-up list was not keyed by map.** `read_over` held bare
   `(x, y)` pairs, so a tile the walk had crossed off in Bambulon crossed off a
   different tile on the Treyway — and the run spent six thousand presses
   crossing the door back and forth looking for a woman it had already decided
   was not there. Four maps make a bare coordinate four tiles.
2. **The pools beat a seven-item board.** The first drafts of the Treyway's
   arrival band and the field's outskirts were rated where the *next* thing
   should be rather than where the last one left off, and the walk crossed and
   was killed and carried home, repeatedly, having read nothing. Both were
   retuned down against what the walk actually survives, which is the only
   measurement either of them has.

**M11.3's run is `m11.3.txt`.** 20,000 presses, 1,247 wins, 1,039 losses, level
11 — three of the Drambus Stack's five floors down, and it is the first run that
enters a map it cannot walk out of.

The walker was rewritten five times to get this far, and every one of the five
was a thing a person does without thinking:

1. **Walk on the road.** Shortest-path crossed the Stack's Shadow off the road
   at twenty-eight percent a tile where the road is six.
2. **Do not walk through the shop.** `town` was in the road set, so the cheapest
   route to anywhere ran through the counter.
3. **Do not clear the give-up list every time you bank.** It cleared the mark
   saying *this errand's tile is where you already are*, and one run walked into
   Kettleworks seven hundred and forty times buying a tin each visit.
4. **A give-up that has somewhere to go is not a give-up.** The
   back-off-when-losing check fell through to a branch that sent the walk to
   the counter it was standing beside: two thousand six hundred visits.
5. **How much you carry depends on whether you are winning.** Banking at
   twenty-five meant banking after every win and never reaching the door;
   carrying four hundred through a losing patch meant banking nothing at all
   and finishing nine levels down. It reads its own last dozen fights now.

And the finding that is about the *game* rather than the walker, which is what
`TRIAGE-M11.md` will get: **the round trip is the block's real cost.** There is
one town past the door, its shelf is Kettleworks', and a character who arrives
under-geared cannot catch up there — the run forbidden from walking back lost
two thousand four hundred fights standing in a field. A thousand losses against
twelve hundred wins is what the road past the door costs at the level the door
opens at.

**M11.4's run is `m11.4.txt`.** 20,000 presses, 1,141 wins, 904 losses, level 12,
three of the Stack's five floors down — and the lake still full, because the run
ran out of presses before the fifth floor.

That is the finding, and it is a measurement rather than a fault: **the block is
longer than a walk.** Twenty thousand presses was enough to reach the door and
drop most of a tower and is not enough to drain a lake behind it, at a win rate
of about eleven to nine. `TRIAGE-M11.md` gets both halves — the length, and the
rate that makes the length what it is.

**M11.6's run is `m11.6.txt`.** 20,000 presses, 1,383 wins, 1,040 losses, level
14, three floors of five. It says the same thing `m11.4.txt` says and says it
twice, which is what makes it a measurement rather than a bad seed: **the block
is longer than a walk, at a win rate of about four to three.** The reach is
behind the lake, the lake is behind the tower, and twenty thousand presses gets
most of a tower.

Both halves go to `TRIAGE-M11.md` — the length, and the rate that makes the
length what it is.

**M11.7's run is `m11.7.txt`, and it is the first that reaches the end of the
block.** 6,546 steps, 703 wins, 187 losses, level 21 — through the door, west to
Kettleworks, all five floors of the Drambus Stack, the lake drained, and the
door under it read.

What changed between `m11.6.txt` (three floors of five in twenty thousand
presses) and this one is not the walker: it is the **measurement**. M11.7
simulated the whole ladder against the best board the game actually hands out
and found that seven of the nine new pools had a most-drawn creature that board
could not beat, and that the tower's second floor was a wall. The pools were
retuned off the measurement rather than off the ratings, and the win rate went
from four-to-three to four-to-one. `TRIAGE-M11.md` rows 1 to 3.

Two things this run does **not** do, both in the triage:

- **It never surveys.** Auto-pack collects six map shards and packs none of them,
  because a compass rates worse than the blade it would replace. The reach is
  content `make play` cannot reach; the browser gate is the only thing that
  walks it. Row 9, and the same shape as M10.3's finding about enchs.
- **It takes the dry route.** The lake is drained by dropping the tower. The
  wet route — a whole Toad set walking out to the grating before the tower falls
  — is `check_the_toad_walks_on_water` in the gate, which lands under the lake
  with the water rows still in place.

## M11.9, twice, and the two runs disagree

`m11.9.txt` is the **second** run, and it is here because it loops. The first
one reached the ending — the door under the lake at step 4,406, level 14, 342
wins to 170 losses. The second spent 240 cycles doing this:

    door       -> The End of All Gears, carrying 0
    road west  -> The First Treyway
    the stack  -> The Kettleworks Field, lose, carrying 0 again
    door       -> The End of All Gears, carrying 0

It walks out of the pit town, dies in the Kettleworks Field, is walked home,
and heads straight back out. It passes *through* a town every single cycle and
banks nothing, because it has nothing to bank — a defeat takes what you are
carrying, and its next destination is on another map, so it never once goes
home while it still has something.

**The game is fine and the instrument is not.** M11.7 proved the block is
finishable and the first run of this pair proved it again; what the second run
found is that our walker, once it has a cross-map goal, stops doing the one
thing a player does without thinking, which is cash in before pressing on.

The general version is worth more than the fix:

> **A walker with a destination stops being a player.** `head_for_town` has
> existed since M8 and is wired into the loops that have no better idea. A
> per-map branch that knows where it is going outranks it, and that is exactly
> when banking matters most, because the fights on the way are the ones you
> lose.

It is `PLAN.md` §6d row 3 rather than a fix, and the reason it is a question
rather than a bug is M11.8: there is a second playtest instrument now, and it
is not ours. How much more the first one is worth is a real decision.

**Keep both kinds of transcript.** A run that loops is still a transcript —
read *where* it loops. This one names the exact map, the exact three creatures
it could not beat at level 11 (High Cork Priest 999, What Was Left On Five 892,
and, once, Galapagos Jim at 453), and the fact that a loss in that field is a
loss of everything carried since the last town.
