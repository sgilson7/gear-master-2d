# HANDOFF.md — what you need before you touch anything

For whoever picks this up next. `CLAUDE.md` is the operating manual and is long
because it is a list of things that cost somebody a day; **read it, but read
this first** — it tells you what the project *is*, where the load-bearing walls
are, and which of its habits will bite you within the hour.

---

## 1. What this is, in five sentences

Gear Master 2D is a tile-based open-world RPG built on a gear-assembly
auto-battler forked from `sgilson7/gear-master`. You walk a map, meet creatures,
and fight them — but you do not fight: you **pack five grids with components**,
and the arrangement is the whole of your input. Components form *items* when
their shapes touch and satisfy a recipe; items activate on cooldowns and the
fight runs itself. Everything a player reads is themed after an anthology
called *Turtle Dick: Tales from the Crypt*, whose manuscript lives one directory
up in `TurtleRichard/`. It is live at <https://sgilson7.github.io/gear-master-2d/>.

## 2. The five rules that are actually load-bearing

Break any of these and the failure is silent and expensive.

1. **`crates/core` never imports `wasm-bindgen`, `web-sys`, or anything
   DOM-shaped.** The engine is testable in seconds because of this.
2. **`crates/wasm` decides nothing.** It moves strings across the boundary. A
   rule decided there is a rule the fast suite cannot reach, and then there are
   two rulebooks. When you catch yourself writing an `if` in the shim, the rule
   belongs in core.
3. **Content lives in `data/*.json`.** The map, events, the skill tree, town
   shelves, errands, supplies, drops. Two inherited exceptions: the component
   catalogue is `piece.rs` and the theme tables are `theme.rs` (mirrored into
   `data/theme.*.json`, which is *generated* — `REBASELINE_THEME_DATA=1`).
   `data::FILES` is the list of what is compiled in, and adding a file means
   adding it there too — `data_is_current` walked four of eleven until M9.1.
4. **Adding a field to `Character` or `Game` is a compile error until the save
   carries it.** `SaveFile::of` and `into_game` destructure exhaustively. **Do
   not "fix" a destructure by adding `..`** — that hole is the whole point.
5. **The page draws numbers core sent it.** It never recomputes. This has been
   violated twice and both times the bug was invisible: the replay once
   subtracted damage from a health total it kept itself and ignored `absorbed`;
   and it once opened every fight with an empty armour bar because nothing
   *announces* a balance nobody had to earn.

## 3. The shape of the code

```
crates/core/           the engine — no graphics, no wasm, ~42k lines
  piece.rs             the component catalogue (568) + Trigger/Action + describe
  loadout.rs           grids, items, recipes, the assembly pipeline
  combat.rs            the fight. No RNG. 50ms ticks. `simulate_*` ladder
  character.rs         the player: boards, xp, fatigue, supplies, skills, class
  world.rs             maps, terrain, regions, places, stepping
  quest.rs             errands: give / carry / hand in
  skills.rs            the tree, and `Effect` — what a node does
  rule.rs              `Rule` — what a node *or an item* grants that is not a number
  drops.rs             what a creature leaves behind, and how often
  explain.rs           what one component does, in the engine's own words
  fatigue.rs           what a fight costs beyond the fight, + restoratives
  shop.rs              town shelves, which are content and not state
  survey.rs            what an instrument does to a map you are reading
  save.rs              the hand-written save mirror. Read its module doc.
crates/wasm/src/lib.rs the shim. One file. Big. Decides nothing.
crates/lab/            the authoring bench. NOT shipped — nothing depends on it
web/                   vanilla ES modules, no bundler
  app.js               screens, panels, the map
  board.js             the packing board (canvas)
  theirs.js            a read-only board — creature panel and both replay boards
  replay.js            the fight playback
  shape.js             a component's shape, small
data/*.json            all the content
data/maps/*.tiles.json one file per map, named for its id — eleven of them
testing/drive.py       the deploy gate: 42 checks, three browsers, and it can
                       be pointed at the live page with GM2D_ORIGIN
testing/playthrough.py `make play` — a walker somebody who built it wrote
testing/agent_driver.py one command per turn, for an agent that may not read
                       the source. `AGENT-BRIEF-M11.md` is what it is told
```

## 4. The commands

```
make test        # the engine suite, native, seconds
make check       # fast type-check
make web         # build dist/web/
make test-ui     # walk the gate in chromium, firefox and webkit
make play        # play a new game to the ending and read every screen
make serve       # build and open locally
make art         # compile art/*.tex to web/assets/*.svg
make dress       # search the catalogue for a creature near a rating
make read        # print an existing creature's board and its rating
```

Two variables, both about *which page*:

```
GM2D_WEB=dist/web-gate ...          # build and gate somewhere other than dist/web
GM2D_ORIGIN=https://sgilson7.github.io/gear-master-2d/ testing/drive.py chromium
```

The first exists because a long-running playtest lives on `dist/web` and
rebuilding under it moves the save fingerprint. The second walks the **live**
page, which is the *verify against the live page* step `CLAUDE.md` has demanded
since M8.

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

```
REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core
```

## 5. Ten traps, each of which has already cost a day

1. **`Loadout::locks` is state, not geometry.** Two components that touch are
   one item unless a lock says otherwise, and which locks exist depends on the
   order the player built in. Re-deriving them gives a different board.
2. **Lock each item as it assembles, not once at the end.** A finished board is
   packed tight; deriving items in one pass at the end asks "which components
   are connected" and gets "most of them".
3. **`PieceId` is an index into `PieceRegistry`.** The registry is saved whole
   and in order, by catalogue *name* — never by catalogue index.
4. **`map()` must not reach for the game.** The shim's map lookup takes the
   `Game` as an argument. Reading `GAME` inside it is a `RefCell` double borrow
   at nearly every call site, and in a wasm build that is a bare `unreachable`
   on the console with nothing else to go on.
5. **A new module in `web/` is cache-busted by pattern**, not by a list —
   `package-web.sh` stamps every relative import and dies on a bare one. It
   also hashes `index.html` and `styles.css`, because a CSS-only change once
   produced an identical build stamp and would have been served stale forever.
6. **Do not reuse a class name or an id.** `.card` is the event dialog and
   giving item cards the same class made each one a full-viewport overlay;
   `id="pack"` is the town's button and giving the restorative rack the same id
   hung the browser walk on a hidden element.
7. **A planted browser check must strip every grid, not the one it plants on.**
   The gate's plants run on a character who has fought fifty times, and since
   M9 a character who has fought can have *earned* a set. A check that clears
   only its own grid asks about the board it made and gets an answer about the
   board the walk made — green on a laptop, red in CI, because what a walk
   earns depends on the seed. `strip_the_boards` in `drive.py`.
8. **A set bonus pays off the whole set, and `loadout::set_of` is the one place
   that decides.** Agreement *and* completeness — every component names the same
   set, and every component that names it is in the item. Agreement alone lets
   two thirds of a three-piece set call itself whole, because most recipes have
   an optional slot; completeness alone lets a stranger in.
9. **A CSS rule that sets `display` beats `[hidden]`, and it will do it again.**
   `.screen.framed` did it in M8 and `.panel dl > div { display: flex }` did it
   in M11, hiding nothing and printing "surveying with —" on every map for four
   attempts. **Anything that sets `display` on an element it also hides needs
   its own `[hidden] { display: none }`.**
10. **A reachability check whose lower bound is "not everything works" cannot
    tell a cost from a wall.** `(2..5).contains(&taken)` passed on a tower
    nobody could climb, and the whole M11 block shipped unfinishable behind 597
    green tests. Measure against the board the player actually has —
    `common::geared_from` — and assert against that. And know that `draw_enemy`
    makes a pool's *hardest* member its *rarest*, so what a region contains and
    what it deals you are different questions; that gap has cost three days.

## 6. How the game plays, as of now

- **Start:** two components — an Oak Handle and an Iron Blade, seated *turned*,
  because the blade is 1×4 and a starting frame is three rows. Twenty-eight
  Fnorp.
- **Experience is carried, not banked.** A fight pays into your pocket; a town
  is the only place it becomes a level; a defeat takes everything unbanked and
  nothing you had spent.
- **Fatigue** takes 4% of your maximum health every battle, won or lost, capped
  at 60%. Only a restorative gives it back.
- **Shelves are fixed.** Each town sells a set list, once each, no reroll.
- **Errands** have a giver and a turn-in, which may differ; three goals — slay,
  bring, or go somewhere and report.
- **A town takes the tiredness off**, and it is the only thing that does apart
  from a tin. A tin is what you drink four tiles in with something on the next
  square; the town is what makes the walk home worth taking.
- **Eleven maps and two towns.** West Bambulon is where you start; the Great
  Gear Cave is a short dungeon behind a gate that wants Marbulon's key, and its
  boss drops the key to **a door in the western wall**. Behind that door is
  **the Treyway**, a 16×16 country of which West Bambulon is one tile, and the
  road west across it reaches **Kettleworks** — the second town, on a field so
  thick with things to read that one tile in ten answers.
- **The Drambus Stack** stands in that field: five floors, a boss each, and the
  tower **comes down as you clear it**, so the next time you go in it is the
  floor below. A floor is one sitting — no walking out, and a save taken inside
  reopens outside. How many floors are gone is derived from which boss ids are
  in `answered`; there is no counter.
- **When the Stack is down the lake empties**, and there is a way down in the
  middle of it. That is where the demo ends now — a door behind the thing at
  the bottom. **Or you get there early**: the Toad's Own Frame lets you wade the
  whole lake rather than its rim, and entered before the tower falls the way
  down is twenty-one tiles of slag against eleven of road. Fatigue is what the
  early way costs, because fatigue is the only currency a dungeon here has.
- **Three instruments, and they take your sword arm.** A compass, an atlas and a
  survey golem build on the **weapon** grid out of map shards the Stack and the
  lake leave behind — and a weapon grid holds gear or an instrument and never
  both. With one assembled you can enter **the Wextreen Reach** at the north
  edge of the Treyway; without one there is nothing to read it with. It is the
  same map every time and what changes is the instrument: the compass quiets it,
  the atlas pays more and is louder, the golem fights one fight for you.
- **The north is gated.** Two crossings: the Burnwarp Shallows want level 5 and
  the Bengulon Verge wants 9, and West Bambulon — and therefore the Cave — is
  behind the Verge. A crossing guards a *region*, not its own tile.
- **No town sells an ench.** A class tree awards two, an errand pays one, and
  the other three are sold by one man in a van at [4, 6] on the Verge road, who
  is not there below level ten. Banking the level that opens him says so.
- **Nine sets.** Each is one grid's whole recipe, every piece is off a creature
  or off a tower floor and on no shelf, and assembled — made of nothing but
  themselves — each grants a rule no stat could express. The three pit sets are
  M9's: an A. Rat gives up without a fight, the lake becomes ground, every
  helmet activation lands a curse. Of M11's six, five are new *instances* of
  rules the engine already had, and the sixth is **`Rule::Homeward`** — the
  Drover's Stride pays one cheap tin and walks you to your last town. It
  refuses in four named ways, and one of them is *not from under the lake*,
  because that is the one map where the walk is the content.
- **A class is one of five**, taken at level five and permanent. **The Kaklon
  Patent** and **Top of the Bill** are the two that can bolt **enchs** onto
  components — a rack on the packing screen, one ench a component. The Patent's
  tree awards the turn that stacks power; the Bill is paid half again for a win
  under ten seconds and its tree awards **The Chonga Swing**, which triples an
  item's power and breaks it after one activation, for the fight rather than for
  good.

## 7. Where to look when something is wrong

| Symptom | Look at |
|---|---|
| Bare `unreachable` in the browser console | a `RefCell` double borrow in the shim — trap 4 |
| A save will not open | `save.rs`'s catalogue fingerprint. Adding a component invalidates old saves **by design** |
| A player is stuck in scenery | `World::repair`, called on load *and* on the first keypress |
| A fix deployed and did not arrive | the build stamp, and `app.js`'s self-heal — see *A deployed fix is not a delivered fix* in CLAUDE.md |
| The board draws the wrong shape | something cached what core can be asked. The held component is looked up by id every frame |
| A class or a node promises something and nothing happens | it is honoured in the fight, the purse or on the board, and possibly in none of them. `every_offered_class_reaches_something` |
| A browser check passes but the thing is broken | it is probably vacuous. **Break the thing and watch it fail** before trusting it |
| The map on screen is not the map you are on | `paintPanel` did not re-read. `position()` carries the map id — trap in *A stale map, shipped since M8* |
| A row you hid is still laid out | a `display` rule beat `[hidden]` — trap 9 |
| A whole region cannot be beaten and the suite is green | the reachability check is a range, not a measurement — trap 10 |
| The playtest agent's save stopped loading | somebody rebuilt `dist/web` under it and moved the fingerprint. Use `GM2D_WEB` |

## 8. The habits, in one place

- **Reproduce first, measure rather than guess.** Two bugs were found by
  `document.elementFromPoint` after reading the code produced wrong answers.
- **Fix the class, not the instance.** Removing a cache rather than resyncing
  it; stamping every import rather than the one that was missed.
- **Every derived number needs somewhere it is shown**, or it cannot be told
  from a bug. Four skills worked perfectly and were reported as broken because
  nothing printed them.
- **Two registers, kept apart.** A name and a blurb are the world's; a spec is
  the engine's — unthemed, with the number in it, and *derived from the effect*
  so it cannot disagree with what it describes. `TONE.md` rule 13a.
- **Never write a game string without `TONE.md` open.**
- **The agent does not `git push` or `make publish`** unless the human has said
  to. Check the current standing instruction before you deploy.

## 9. What is being built next

**Nothing is planned.** `PLAN-M9.md`, `PLAN-M10.md` and `PLAN-M11.md` are all
done and all deployed — M11 at `43804e49`, verified by pointing the gate itself
at the live page. Before you deploy the next one, read *A deployed fix is not a
delivered fix* in `CLAUDE.md`: M8.0 through M8.8 sat local for a whole block
once, and the first anybody knew was the human saying they could not see the
quest log. `git log origin/main..HEAD` is the check and it costs nothing.

### The four lists of what to do next, in order of how much they are worth

- **`PLAN.md` §6a** — M8.8's, and still the best list of what this game gets
  wrong. Row 1 is a second town, and it answers three of the other rows for
  nothing.
- **`PLAN.md` §6b** — M9.4's. Row 1 is the one that matters: `draw_enemy`
  weights a pool so its hardest member is its rarest, which is right for fights
  and is now also a *content* decision, because a set off the rarest creature is
  a set behind the rarest creature.
- **`PLAN.md` §6c** — M10.3's. Nothing in it is broken; the top row is a number
  nobody has argued about.
- **`PLAN.md` §6d** — M11's, three rows, and one of them is §6b row 1 again
  with nine sets on it instead of three. The other two are *what is past the
  door under the lake* and *`make play` is a walker with a destination, and a
  walker with a destination stops being a player*. `TRIAGE-M11.md` is the
  twelve-row triage behind it — five fixed in M11.7, seven carried with
  reasons.

### Five things M9 and M10 established, so you do not re-derive them

1. **`Rule` is `crates/core/src/rule.rs` and two systems grant one** — the skill
   tree and an assembled item. The enum, `deny_unknown_fields` and `Rule::check`
   are the whole guard, and every match on it is exhaustive on purpose.
2. **A set is the set or it is gear.** `loadout::set_of` is the one answer, and
   both the item's name and the rules it grants read it: agreement *and*
   completeness.
3. **A map does not know about bags, about levels, or about who is looking.**
   `world::step`, `World::repair` and `place_is_there` all take a
   `world::Allowances` the caller fills. `Allowances::of` matches `Rule`
   exhaustively.
4. **A promise is not a wiring.** Two shipped classes advertised a number and
   delivered nothing — `Showstopper` for two milestones and `Recycler` for one,
   the second found by the lint written for the first.
   `every_offered_class_reaches_something` proves the behaviour rather than
   reading a list, and that distinction is the whole of why it works.
5. **`make play` reaches the ending**, ~4,400 steps as of M11. When it stops,
   read the transcript before the code: four times now the walker was the thing
   that was wrong, and every time it was wrong the way a new player would be.
   **It is also not deterministic** — one M11.9 run finished and the next looped
   240 times — so a green run is not proof and a red one is not a bug report.

### Six things M11 established, so you do not re-derive them

1. **Eleven maps live in `data/maps/`, one file per id.** `data/tiles.json` and
   `data/dungeon.json` are gone as names; two maps could be two nouns in a data
   directory and eleven cannot.
2. **`data::map_at` is the file and `data::map_now` is the game** — the same
   split `place_at` and `place_now` made. A lake that is water until a tower
   falls is `TilesData::drains`, not a second map file and not a grid in the
   save.
3. **`survey::mods_for(map, kind, items_assembled)` is a pure function**, and
   nothing about surveying is in a map file. An instrument is the character's,
   a map is the world's — the same division `Allowances` makes for a crossing.
4. **The game says everything through `log()`**, which lands it on a strip that
   is always up and keeps it in a history you can open. Nothing a player is
   told should die with the screen that told them.
5. **`position()` carries the map id and `paintPanel` re-reads when it
   changes.** A cross-map defeat drew the wrong map from M8 until M11 because
   nobody had two ways between maps before.
6. **There is a second playtest instrument and it is not ours.**
   `testing/agent_driver.py` plus `testing/AGENT-BRIEF-M11.md` hand the built
   game to an agent forbidden the source, the data, the tests and this file. If
   it wants a number it cannot get, that is the finding.

### What is not scheduled, and is still the human's

- **What is past the door under the lake.** `PLAN-M8.md` §5.6, asked one map
  further on. Nothing in the code assumes a third overworld, a chapter count,
  or an ending beyond the one M11.4 writes — and the ending screen says as
  much, in as many words.
- **The pool weight.** `PLAN.md` §6b row 1, and it has now cost three days:
  M9.4's drop rates, M11.7's unfinishable block, M11.9's sets. Every check
  written so far *obeys* the weighting; changing it retunes every region at
  once, which is why it is still a person's.
- **High Wick.** Written, shelved, given errands, and on no map. Kettleworks
  came off that list in M11.2.
