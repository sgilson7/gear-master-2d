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
