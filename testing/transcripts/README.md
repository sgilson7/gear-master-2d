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
