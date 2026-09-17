# Saves you can load

Drop one of these into the page's **Load** button. They exist so a person can
be standing in the thing that is being tested in ten seconds rather than in
twenty minutes of fighting — the same argument the browser gate's `plant`
makes, made for a human.

| file | where it puts you |
|---|---|
| `on-the-table.json` | **the Treyway, which is a table.** Pull back from the ball with the mouse, or aim with ← → and ↑ ↓ and press space |
| `on-the-sands.json` | **a reported save**: standing on the Wextreen Sands with THE TENTH SURVEY finished and the way under not yet open. The gate wanted the *event* of that name, two hundred paces east, and was hidden so it could not say so |
| `at-the-lip.json` | the Low Water, at the mouth of the Wextreen Sump |
| `at-the-reefs.json` | the first floor of the Eleven Reefs |
| `under-the-lake.json` | what the lake was on top of |
| `the-run-20260910.json` | the run fixture `crates/lab` rebuilds — a full board, for recon. **It is a yardstick and not a start line**: its world is empty (`map: ""`, nothing answered), so it is what a board is measured against and never somewhere to stand |
| `on-the-lower-table.json` | **the lower table, on the tee.** The whole settling done — the clerk down, volume ten open, the post cut, The Unwritten beaten — so the way over the lip is open behind you and the post says Low Wick. Pull back and fire: five obstacles down each rail, five cards, two gutters at the bottom, and **the far pocket** behind the north range, which is the only way into the cup |
| `in-the-third-town.json` | **the Undercountry, on the tee four tiles up the lane from the town.** Both bottoms answered, so Marbulon's door is open the way it was written to be; `nobody-has-named-it` handed in, so Kettleworks will talk about the post; and all three towns stood in, so the long cart runs. What is left is the chain that settles the place — send for the clerk, do the inventory, cut the post |

**No save in this repository opened the Undercountry until M22.** All fourteen
were read: not one had both `the-ninth-surveyor` and `the-bottom-of-the-bottom`
in `answered`, which is what `hidden_until_all` on Marbulon's door wants — and
**the human's own has the second and not the first**. The walker will not get
there either: from a new game it runs 977 fights to level thirteen and ends in
the loop `PLAN.md` §6d row 3 names. So the block ships its own start lines, and
`GM2D_FROM=` is what walks from one:

    GM2D_FROM=testing/saves/in-the-third-town.json make play

**A save carries the catalogue's fingerprint**, so one written before a block
that moved `CATALOG` is refused by name rather than loaded wrong. If one of
these stops opening, that is what happened and the refusal says so.
