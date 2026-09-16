# The prompt

Copy everything between the rules into a fresh Claude Code session, in the
repository root, with `PLAN-M22.md` already placed there.

---

You are picking up an existing, deployed game project to execute its next block
of work, **start to finish, without asking anyone anything.** Everything you
need is written down; nothing has been started; nobody is going to answer a
question, so do not ask one. Where the plan leaves a choice open it also gives
a recommendation, and you take the recommendation, say in the commit that you
took it, and move on.

**Read `HANDOFF.md` first, entire**, then `CLAUDE.md` from "Rules" to the end
of "Commands", then these sections of it: *Two countries are tables, and you
shoot across them* (all of it — the tile a step away, the diamond that catches,
the gate beside you), *Down twice, and the country under the country*, *The
economy, and why the shelf stopped rolling*, *Errands are not a town's*, *A
refusal names the lock*, *Adjacency on a bed is usually forced*, and
*Divergences from the brief* — read a dozen rows of that table before you write
a line, because the failures rhyme.

Then read `SECOND-ORDER-M21.md` entire. **It has sixty-three rows and none of
them is open**, so you carry nothing in; what you are reading it for is the
format you will keep and the shapes that keep recurring. Rows **62** and **63**
are the newest and both bear on this block: a `.screen` left pinned over the
page fails thirty steps later as a Playwright timeout, and **Instant Battle
draws no replay**. Read `git show 4a3ae9a` for the whole of that.

Then read `PLAN-M22.md`, which is the plan you are executing, and
`testing/saves/README.md`, which is the table this block adds two rows to.

Then read the code the plan touches, before anything else: `shop.rs`
(`TownShelf`, `shelf`, `barrel_pool`, `ledger_pool`), `world.rs` (`PlaceDef`,
`place_is_there`, `opens_onto`, `arrive_at`), `shot.rs` entire (634 lines),
`game.rs` (`shoot`, `warp_to`, `sealed_because`), `quest.rs` (`hand_in`,
`stage`, `guide`), `data.rs` (`towns_on_the_map`, `MAPS`, `FILES`),
`tests/common/mod.rs` (`UNWRITTEN`, `from_save`, `THE_RUN`, `geared_from`),
`tests/shot.rs` and `tests/undercountry.rs` entire, `tests/stall.rs`'s
`a_bargain_is_never_on_the_barrel`, `testing/drive.py`'s
`check_the_third_town_is_empty_on_purpose`, `testing/playthrough.py`'s
`GM2D_FROM` block and `clear_over_the_map`, and
`data/maps/the-undercountry.tiles.json`. The plan quotes these by name; **if a
name it quotes is not there, the plan is wrong** and you write that down before
you build around it.

The block is two asks from the human, in their own words:

> I want to add some more content to the game, starting from the town in the
> undercountry that currently has nothing.

> write up the settling of high wick and the table ideas into a plan like the
> plans already existing in the repo. I do like the dungeon idea but i think
> its too early to implement that now

Execute in the order `PLAN-M22.md` gives: **M22.0 through M22.9, then M22.10
onward**, which you will write yourself from the notebook. Deploy after M22.4
and after M22.9, and after the last notebook milestone if it changed anything a
player can see. You do not ask before deploying. You run `make publish`, you
report the hash, and you keep going.

Work milestone by milestone. For each one:

1. **Do the recon before writing anything**, and put what you found in the
   commit message. M22.0 is nothing *but* recon and it is not optional: the
   plan's draft table has never been shot at, and the flood in
   `every_place_on_a_shot_map_is_reachable_by_shots` is what decides whether
   the cup is sealed, whether the far pocket can be reached, and how many
   rounds the map takes. **If the cup measures open, the rows are wrong and you
   redraw them before you author a map.** The other guesses: the casting count
   with `high-wick` placed (92), the rounds to reach everything (2), and the
   boss's time against the level-45 board (20–28 s).
2. **Build the rule in core and the picture in the shim.** A wing is core's
   list; the street draws a building because a panel is not empty, not because
   a list says the town has one. A pocket's `to` is resolved in `Game::shoot`
   and drawn by the page. Any `if` you find yourself writing in `crates/wasm`
   is a rule that belongs in core.
3. **Grep before you invent.** The plan already found that a pocket warps, that
   `PlaceDef` carries `to`, that `hidden_until` reads one closure, that
   `paintStreet` reads the panel, and that the level-45 yardstick already
   exists as `common::from_save(common::THE_RUN)`. If you reach for a new
   mechanism, search for it first; four of six "new" things in M21 already
   existed under other names.
4. **Write the tests the acceptance lines name, break each one and watch it
   fail before keeping it**, then restore — with a `cp` you took yourself,
   never `git checkout` on a file with uncommitted work in it (M21 row 4). And
   **a check whose negative test cannot be made to fail through the check is a
   check nobody has proved**: that is M21 row 23 and `4a3ae9a` is its answer.
   Every refusal this block adds belongs to exactly one of `ShopsData::parse`
   and a lint, the parse owns what a file can be asked about itself, the lint
   owns what needs the map files, and the lint's negative test hands a mutated
   `SHOPS_JSON` to `parse` and asserts the refusal — which is what
   `a_bargain_is_never_on_the_barrel` does now. Read it before you write yours.
5. **A check about a thing is an arm of the check that already reaches that
   thing**, not a second walk to the same tile — M21 row 42. The gate already
   stands in the third town: `check_the_third_town_is_empty_on_purpose` plants
   at [10, 5] and shoots to [10, 10]. You **rewrite** it into
   `check_the_third_town_fills_up`. You do not add a second one beside it.
6. **Every new sentence a player reads is `TONE.md`'s**, with the file open.
   The clerk counts volumes; the tape counts contacts; the door says which boss
   is still standing. The plan ships *Low Wick*, *the table under the writing*
   and *The Twelfth Name*, and you ship those.
7. **One new creature, dressed by damage a second against
   `common::from_save(common::THE_RUN)`** — not against `common::geared_from`,
   which loses to a 1141. `make dress` and `make read` are the tools; its art
   is one colourway of an existing family in `art/creatures.json` and `make
   art` compiles it. A figure that compiles is not a figure that works, so
   rasterise the sheet and put your eyes on it.
8. Run `make test`, then `make web` and `make test-ui`, then the walk — **and
   the walk is `GM2D_FROM=`, not a bare `make play`.** From a new game the
   walker now runs 977 fights to level thirteen and ends in the loop `PLAN.md`
   §6d row 3 names; it will never reach the Undercountry, and that is a fact
   about the walker. **No save in this repository opens the Undercountry** —
   all fourteen were read and the human's own has one of the two bottoms — so
   this block ships its start lines: `testing/saves/in-the-third-town.json` in
   M22.4 and `on-the-lower-table.json` in M22.9, each with its row in
   `testing/saves/README.md`. Then read the transcript.
9. **Nothing this block writes waits for `#stage-replay`.** Instant Battle
   settles a marked creature without drawing one, and at level 45 most of the
   ladder is marked. `fight()` in the walker already handles it; a gate check
   you write that assumes a replay will hang for ten seconds and then fail
   somewhere else.
10. **Keep the notebook.** `SECOND-ORDER-M22.md`, in `SECOND-ORDER-M21.md`'s
    format — numbered rows, bold first sentence, `open` or `closed` — starting
    at **row 1**, because M21's is closed. Write a row the moment you notice a
    second-order effect: a lint that flipped from exemption to assertion and
    caught something else, a fixture that moved when the shelf landed, a shot
    the flood found that the walker could not take, a constant whose value
    turned out to matter for a reason the plan did not give. Do not save rows
    up for the end. A row is a sentence about what you saw, not a plan to fix
    it.
11. **Print the status table.** After every milestone's commit, append the
    table in `PLAN-M22.md` §Status table to `MILESTONES.md` under `## M22 —
    High Wick, and the table`, with every milestone's row, the test count from
    `packaging/count-tests.sh` and its delta, the commit hash, and the
    notebook's open/closed count on the last line. Print the same table as the
    last thing in your message for that milestone. The human reads this table
    and nothing else to know where you are.
12. Commit in the house style — `git log -10` first. Say *why*, name what was
    rejected, and say when something cost a day.

**When M22.9 is done, write the rest of the plan.** Read the notebook. Take
every `open` row. Group rows into milestones of the same shape as the plan's —
a title, a deliverables table, an acceptance line. Number them M22.10, M22.11,
… in the order that closes the most rows soonest. Append them to `PLAN-M22.md`
under *M22.10 → — Whatever the notebook says*. Then execute them exactly as you
executed the first ten, closing rows as you go and printing the table after
each. A row that is a finding and not work is closed by writing the finding
into `CLAUDE.md`; a row that is the human's is closed by writing it into the
status table's last message as a question and carrying it. **The block ends
when the notebook has no open rows** and the final status table's last line
says `0 open`.

Nine standing constraints:

- **No new components, no new save fields.** A wing is a shelf, a table is a
  map, a name is a field on a place. The fingerprint does not move and
  `save.rs` is not touched. There is a player mid-run at level 45.
- **The town is settled before the table exists.** M22.5 is the first table
  milestone and `only_the_country_maps_are_tables` refuses the map file until
  it is there, which is the order enforcing itself.
- **`high-wick` keeps its id.** A sale is `(id, index)`; renaming the shelf or
  reordering its stock moves what somebody already bought.
- **`met` is the one predicate.** `hidden_until`, `hidden_until_all`,
  `needs_all`, `arrives` and `named.when` all read it. A second closure that
  also reads `quests_done` is two rulebooks.
- **A pocket costs twelve wherever it goes.** `POCKET_TIRES` is one constant
  with one shelf on G.
- **`PlaceKind::Door` has exactly one user** before and after, and it is the
  stop-line. `the_stop_line_is_still_one_door` goes into the suite before the
  map is authored.
- **The boss is not in a pool.** Divergences 15.1 and 20.5 between them say
  why; `the_twelfth_name_stands_nowhere_but_the_cup` asserts it.
- **`UNWRITTEN` and `STAGED` are emptied and asserted empty, never deleted.** A
  list that means *nothing is staged* is a claim only while something checks
  it.
- **Clear at the door, not at the scene.** Every screen this block opens is one
  more thing that can be left pinned over the page, and what that looks like is
  a timeout somewhere else — `clear_screens` in the gate before a check's first
  click, `clear_over_the_map` at the top of the walker's step loop. That cost
  two deploys and three walks already.

If you find something the plan got wrong — and the draft rows at least will be
— say so in the commit, make the change, record it in the notebook and in
`CLAUDE.md`'s divergence table with its reason, and keep going. You do not stop
to ask whether to diverge. The three most likely: the cup measures open from
some tile inside it, in which case the cup shrinks or the boss moves and the
far pocket's `at_to` moves with it; the far pocket cannot be reached in any
round, in which case the range in front of it is cut back a tile at a time
until the flood finds it and **never by cutting a mouth in the cup**; and the
level-45 board beats the first draft of the boss at the buzzer rather than by a
margin, in which case the dial is the gear tier and not the health, which the
Kettleworks finding says three times over.

Start with M22.0. It ships nothing a player can see and one commit whose
message is six numbers, and the first status table has one row marked done.

---

## Notes for whoever hands this over

- The tree is at `4a3ae9a`, live, with M21 closed and the notebook empty. This
  block adds one map, so `data::MAPS.len()` goes to 30 in M22.6 and
  `tests/undercountry.rs:218` moves with it; if the **fingerprint** moves,
  something has gone into the catalogue that should not have.
- **Your save cannot see any of this yet.** It carries
  `the-bottom-of-the-bottom` and not `the-ninth-surveyor`, and Marbulon's door
  wants both, so the Undercountry is shut for you. That is why the block ships
  two start-line saves — load `in-the-third-town.json` and you are standing in
  it. Nothing about the door changes.
- Three names are yours and the plan ships defaults for all three — *Low Wick*,
  *the table under the writing*, *The Twelfth Name*. Say the word for different
  ones; a theme entry is not a seam.
- The riskiest milestone is **M22.6**, the table, and the plan says why: the
  draft has never been shot at. M22.0 shoots it. If M22.0's commit says the cup
  is open, expect the rows to differ from the plan's and expect the notebook to
  say so.
- `CLAUDE.md`'s numbers table has three stale rows — Maps at 25 against 29,
  Places at 215-over-25 against 220-over-29, Data files at 29 against 45. M22.9
  fixes all three. If you want them fixed sooner, they are one commit.
- The dungeon is declined, and the plan's last section costs it so nobody
  measures the 101 call sites or the `Difficulty` constant again.
