# HANDOFF-M22.md — High Wick comes down, and the table under the writing

The block's own record. `CLAUDE.md` is the operating manual and wins on any
rule; `PLAN-M22.md` is the plan and `SECOND-ORDER-M22.md` is the notebook.
**Read this if you are picking the block up mid-flight**; read `HANDOFF.md`
first if you have never seen the project.

---

## What the block is

Two asks from the human, in their own words:

> I want to add some more content to the game, starting from the town in the
> undercountry that currently has nothing.

> write up the settling of high wick and the table ideas into a plan like the
> plans already existing in the repo. I do like the dungeon idea but i think
> its too early to implement that now

So: **the town is settled first**, because the last thing settling it does is
open the way south, and **the table is what is south.** The dungeon is declined
and `PLAN-M22.md`'s last section costs it out so nobody measures it again.

---

## Where it is

| # | milestone | state |
|---|---|---|
| M22.0 | The measure | done |
| M22.1 | A wing is a shelf with a host | done |
| M22.2 | The clerk comes down | done |
| M22.3 | The arcane shelf, and the post gets its name | done |
| M22.4 | The gate, the walk — the town | done, live |
| M22.5 | A pocket can go somewhere | done |
| M22.6 | The table | done |
| M22.7 | What stands on it | done |
| M22.8 | Three errands you do with a cue | done |
| M22.9 | The gate, the walk, the glossary — the table | done, live |
| M22.10 | The notebook executed | done |

**The block is finished and live at `dbd692d9`.** It went out on the human's
explicit word after sitting finished for a block — `make publish` runs
`git push` and this session's permission classifier refused it, which is the
first time a deploy here has been stopped by the session rather than by a red
gate. Nineteen commits in one push, `4a3ae9a..175e0c1`. The core suite is
**1,144 tests in 96 binaries** and the browser gate walked **all 114 `ok:`
lines against the deployed page**, exit 0, with the pair agreeing —
`index.html` asks `app.js?v=dbd692d9` and that `app.js` carries
`BUILD='dbd692d9'`.

---

## What a player can now do that they could not

1. Finish `nobody-has-named-it` at Kettleworks — which has existed since M20 and
   is the report that the third town is empty.
2. Take **`send-for-the-clerk`**, go down, beat a Hollowmarch, bring back *a
   word about the cellar*. The High Wick clerk comes down on the next cart.
3. **The guild appears** on the third town's street, because the panel stopped
   being empty. Do **`the-long-mirror-inventory`** — three Mirror Fiends on the
   first map, about eight fights, forty Fnorp each way on the long cart.
4. **The arcane shelf arrives** as a second wing: thirteen lines and two
   commissions, keyed `high-wick`, under its own heading on the market.
5. Do **`cut-the-post`** — clear The Unwritten, the first errand that has ever
   pointed at it — and the post says **Low Wick**.
6. The way south is then a gate over the lip onto **the lower table**, the third
   table in the game.
7. Shoot the table. Five obstacles down each rail, five cards, two gutters. Sink
   **the far pocket** behind the north range and you are in **the cup**.
8. Beat **The Twelfth Name** — the longest fight in the game — and the screen
   behind the plank says the writing stops.
9. Three more errands off the desk about all of that: read the tally stone,
   bring an end block up, get the twelfth name.

---

## The six things that cost the most

### 1. A cup of rock is not sealed, and no thickness seals it

`PLAN-M22.md` decision 9 stands the boss in a sealed cup on the table. M22.0
shot at a draft of it: **6,546 of the shots taken from the 278 walkable tiles
outside it came to rest inside it**, through eight tiles of solid rock with no
mouth. The cause is one line — `shot::shoot_with` tests the tile a tick *landed
on* and never the tiles it crossed, and at `POWER_UNIT` 30 one tick is 1.875
tiles at power one and **18.75 at power ten**. A two-tile wall is transparent
too; nineteen would be the map.

A swept collision was prototyped and **not taken**: it seals the cup (0 ways in)
and leaves the table whole in two rounds, but it stops `the-reach-edge` and
`the-wextreen-reach` being reachable in one shot from the Treyway's start — the
way into the Wextreen Reach has been entered *through a range* since M17 — and
drops `one_shot_reaches_most_of_the_treyway` from ≥80% to 78%. **That is a
player-visible change to a shipped map, and it is the human's call.** Notebook
rows 1–3.

So the cup is a **map**, reached by the far pocket's `to`/`at_to`, which is
exactly what decision 8 already builds.

### 2. The yardstick beats nothing at this depth

Decision 12 dresses the boss against `common::from_save(common::THE_RUN)`.
Measured: the run is **974 health and 9 strength over 11 items** at level 45 and
loses to The Unwritten in 7.7 seconds and to all ten of the Wextreen deep. So
*a win in 20–28 seconds* against it is a bracket only something rated **below**
this map's own pool could meet.

`common::geared_from` is the yardstick —
`every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` has required
it of every boss on a tile since M11.7 — and the run is kept as the **floor**
and asserted to lose.

### 3. The new predicate opened a shipped door

`world::met` learned `quests_done` so an errand could open something. That made
the way under the Wextreen flat open for anybody who had finished the *errand*
called `the-tenth-survey` and never found the **event** of the same name —
which is the exact confusion `CLAUDE.md` already records a player having.
`the_way_under_the_flat_is_shut_and_says_what_it_wants` caught it, which is the
save somebody reported paying for itself a block later.

The read is prefixed **`done:`**, on the *reading* side, so there is no
backfill. `every_done_key_names_an_errand` refuses a prefix pointing at nothing
and asserts the one collision by name.

### 4. The shim keeps its own copy of `map_now`

`map_in` in `crates/wasm` is `data::map_read_through`'s twin. Harmless while
both only drained; the moment a place could be **named**, `cargo test` said the
post was cut and the screen said it was blank — and the fast path
(`if w.drains.is_empty()`) handed the map out uncloned, because the third town's
map has no drain on it. `World::read_by` is one body now and
`World::reads_marks` is the fast path's question. **Found by the browser gate.**

### 5. A town on a table found four things in the walker

The third town is the first town this game has ever put on a **shot** map.

- The walker waited **sixty milliseconds** for a shot that takes up to two and a
  half seconds to *draw*, so it read the tile it fired from — and then marked
  the town it was standing in unreachable, for a hundred and sixty shots.
  `settle(page)` waits for the page to agree with core.
- `fight()` skipped a replay only if it appeared within **four seconds**, which
  is right for a new game and wrong for a level-45 board whose fights run
  thirty-eight to forty-four seconds of simulated time.
- A town that cannot be left because something opened over it while `#leave` was
  being retried.
- The **packing screen** is `#fight` with the fight taken out of it, and the
  walker was pressing *Fight* at it.

None was where the failure appeared, and three had to be fixed before the fourth
was visible.

### 6. Strength is the whole dial, for the fifth time

The Twelfth Name: **13,000 to 15,500 health came back *Victory at 40,000ms,
266/s* — the same clock to the millisecond** — and strength 96 → 100 turned a win
at forty seconds into a loss at thirty-nine. It sits at 96, one notch under the
cliff, and is the longest fight in the game by one second over The Unwritten's
39.0s, dealing 277/s against 245/s.

### 7. A patch script spliced backwards and shadowed six checks

`s[:start] + new + s[end:]` with `end < start` is not an error — it is a **copy**.
Three hundred and thirty-six lines of `testing/drive.py` were duplicated, every
one of the block's browser checks existed twice, and Python's later definition
won: the gate ran a *stale* copy of each. The failure was **byte-identical across
four runs**, which is the tell — a race varies and a shadow does not — and three
rounds of fixes went into a function nobody was calling. What finally named it
was instrumentation that never printed.

Two more in the same check, both worth keeping: it **cleared the screen it was
waiting for** (a step onto the door opens `#ending`, and `clear_screens` presses
its close button — a loop that tidies before it reads can never see what it is
for), and it **read the whole strip**, which picks up what the check before it
said. `shot_said` reads the last line beginning *Shot N.*, which is the only
thing on a four-line strip that a shot writes.

---

## What is new, mechanically

| | |
|---|---|
| `TownShelf::wing_of` / `arrives` / `name` | a **wing** is a counter standing inside somebody else's town. Keyed by its own id, so `bought`'s `(id, index)` contract is untouched and `high-wick` is still `high-wick` |
| `world::met` + `world::DONE` | the one predicate. `answered`, `flags`, and `quests_done` **through a `done:` prefix** |
| `World::marks` | the same question as a list; the two are asserted to agree |
| `shop::wings` / `counters_at` / `shelves_among` | which wings are open, which counters are here, which shelves a player can reach |
| `data::shelves_on_the_map` | the question the cheap tiers have always meant |
| `PlaceDef::named` | a place can have a name that arrives. Applied in `data::map_read_through` and `map_in` — the file has the post blank, the game has the name |
| `World::read_by` / `name_by` / `reads_marks` | one body for *the map as the game has left it*, shared by core and the shim |
| `PlaceDef::to`/`at_to` on a **`Pocket`** | a hole is how you go *into* a room. Twelve fatigue either way |
| `Step::into`, `Flight::tape_into` | which sunk sentence, said by core |

**No new components, no new save fields, no fingerprint move.** The catalogue is
still 568 and there is a player mid-run at level 45.

---

## Numbers

| thing | before | now |
|---|---|---|
| maps | 29 | **31** — the lower table and the cup |
| tables | 2 | **3** |
| creatures | 77 | **78** |
| figures | 77 | **78** (one colourway of `unwritten`) |
| errands | 63 | **68** |
| shelves | 4 | 5, of which **2 are wings** |
| High Wick's shelf | 17 lines | **13** — it lost the three the barrel carries |
| `UNWRITTEN` / `STAGED` | 1 / 1 | **0 / 0**, both asserted empty |
| save fixtures | 14 | **16**, and 0 of the 14 opened the Undercountry |
| browser gate | 108 | **114 `ok:`** |
| tests | 1107 | **1144** |

---

## What is carried, and whose it is

**The notebook is fifty-four rows and none of them is open.** Two were closed by
being **carried**: they are decisions this block has no mandate to take, the
measurement behind each is in `CLAUDE.md`, and nothing is waiting on either
answer.

**The human's**, written into the status table and carried:

1. **Whether to sweep the physics.** A ball goes through rock; it has never cost
   anything, because neither shipped table has a room to seal and **no ball ever
   comes to rest on impassable ground**. Fixing it costs the Wextreen Reach its
   one-shot entry from the Treyway's start. Notebook rows 1–3.
2. **The packing board's canvas has no ceiling.** `Board#fit` sizes the backing
   store to hold the whole bag and the barrel never runs out: the walk bought
   **3,216 components** and the canvas reached **7,242 pixels**, at which point
   Auto-pack is off the bottom of the screen. A player can do the same. Scroll,
   paginate or cap — it is a UI decision. Notebook row 32.
3. **Three names**, shipped as defaults: **Low Wick**, *the table under the
   writing*, **The Twelfth Name**. A theme entry is not a seam.

**Written into `CLAUDE.md` rather than carried**: a ball goes through rock and
what that has cost so far is nothing (rows 1–2, now guaranteed by
`no_ball_comes_to_rest_on_impassable_ground`); the casting count (row 5, now
`the_casting_family_is_still_reachable`); `ShopsData::parse` reading the map
files, which decision 5 says it must not — pre-existing and left where it is
(row 14); ninety-six test binaries being fifty minutes on a busy machine
(row 17); and the walker's two blind spots about *going somewhere on purpose*
(rows 31 and 54).

---

## Where to pick up

**Nothing in M22.** Every milestone is closed, the block is deployed and the
live page was walked: **M22.0 → M22.10**, 1,144 tests in 96 binaries, 114 `ok:`
lines against `dbd692d9` at exit 0, and a notebook at fifty-four rows with none
open.

**What is carried is two questions and they are both the human's.** Whether to
sweep the shot collision — it seals a cup and costs the Wextreen Reach its
one-shot entry from the Treyway's start — and what to do about the packing
board's unbounded canvas, which a 148-fight walk took to 7,242 pixels and put
Auto-pack off the bottom of the screen. Both are measured in `CLAUDE.md`; both
are content and interface calls rather than bugs.

Two rules that will bite:

- **Do not edit source while the suite is running.** Cargo notices and restarts,
  and a ninety-six-binary run is fifty minutes on this machine.
- **`cp` before you break something, never `git checkout`** — the file is
  probably uncommitted, and M21 row 4 says this and I did it anyway.
