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
