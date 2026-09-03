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
crates/core/           the engine — no graphics, no wasm, ~40k lines
  piece.rs             the component catalogue (544) + Trigger/Action + describe
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
  save.rs              the hand-written save mirror. Read its module doc.
crates/wasm/src/lib.rs the shim. One file. Big. Decides nothing.
web/                   vanilla ES modules, no bundler
  app.js               screens, panels, the map
  board.js             the packing board (canvas)
  theirs.js            a read-only board — creature panel and both replay boards
  replay.js            the fight playback
  shape.js             a component's shape, small
data/*.json            all the content
testing/drive.py       the deploy gate: walks the whole game in three browsers
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
```

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

```
REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core
```

## 5. Eight traps, each of which has already cost a day

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
- **One map with one town** plus a short dungeon, the Great Gear Cave, behind a
  gate that wants Marbulon's key. Its boss drops the key to **a door in the
  western wall**, which is not there until the boss is down and is where the
  demo ends.
- **The north is gated.** Two crossings: the Burnwarp Shallows want level 5 and
  the Bengulon Verge wants 9, and West Bambulon — and therefore the Cave — is
  behind the Verge. A crossing guards a *region*, not its own tile.
- **No town sells an ench.** A class tree awards two, an errand pays one, and
  the other three are sold by one man in a van at [4, 6] on the Verge road, who
  is not there below level ten. Banking the level that opens him says so.
- **Every creature in the pit drops pieces of a set**, at 50 / 80 / 500
  per-mille. Three sets, each one grid's whole recipe; assembled, and made of
  nothing but themselves, they grant a rule no stat could express: an A. Rat
  gives up without a fight, the lake's rim becomes ground, or every helmet
  activation lands a curse.
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

**Nothing is planned.** `PLAN-M9.md` and `PLAN-M10.md` are both done and both
deployed — M10 at `471afa8`, verified against the live page. Before you deploy
the next one, read *A deployed fix is not a delivered fix* in `CLAUDE.md`: M8.0
through M8.8 sat local for a whole block once, and the first anybody knew was
the human saying they could not see the quest log. `git log origin/main..HEAD`
is the check and it costs nothing.

### The three lists of what to do next, in order of how much they are worth

- **`PLAN.md` §6a** — M8.8's, and still the best list of what this game gets
  wrong. Row 1 is a second town, and it answers three of the other rows for
  nothing.
- **`PLAN.md` §6b** — M9.4's. Row 1 is the one that matters: `draw_enemy`
  weights a pool so its hardest member is its rarest, which is right for fights
  and is now also a *content* decision, because a set off the rarest creature is
  a set behind the rarest creature.
- **`PLAN.md` §6c** — M10.3's. Nothing in it is broken; the top row is a number
  nobody has argued about.

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
5. **`make play` reaches the ending**, ~1,400 steps. When it stops, read the
   transcript before the code: three times now the walker was the thing that was
   wrong, and every time it was wrong the way a new player would be.

### What is not scheduled, and is still the human's

- **What is past the door.** `PLAN-M8.md` §5.6. Nothing in the code assumes a
  second overworld, a chapter count, or an ending beyond the one M8.7 writes.
- **A second town.** Kettleworks and High Wick are written, shelved, given
  errands and on no map. `PLAN.md` §6a, row 1.
