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
   shelves, errands, supplies. Two inherited exceptions: the component catalogue
   is `piece.rs` and the theme tables are `theme.rs` (mirrored into
   `data/theme.*.json`, which is *generated* — `REBASELINE_THEME_DATA=1`).
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
crates/core/           the engine — no graphics, no wasm, ~33k lines
  piece.rs             the component catalogue (536) + Trigger/Action + describe
  loadout.rs           grids, items, recipes, the assembly pipeline
  combat.rs            the fight. No RNG. 50ms ticks. `simulate_*` ladder
  character.rs         the player: boards, xp, fatigue, supplies, skills, class
  world.rs             maps, terrain, regions, places, stepping
  quest.rs             errands: give / carry / hand in
  skills.rs            the tree, and `Effect` — what a node does
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
make serve       # build and open locally
make art         # compile art/*.tex to web/assets/*.svg
```

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

```
REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core
```

## 5. Six traps, each of which has already cost a day

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
- **A class is one of four**, taken at level five and permanent. The Kaklon
  Patent is the one that can bolt **enchs** onto components — a rack on the
  packing screen, one ench a component, and one of them turns the item it is on
  once a second for stacking power.

## 7. Where to look when something is wrong

| Symptom | Look at |
|---|---|
| Bare `unreachable` in the browser console | a `RefCell` double borrow in the shim — trap 4 |
| A save will not open | `save.rs`'s catalogue fingerprint. Adding a component invalidates old saves **by design** |
| A player is stuck in scenery | `World::repair`, called on load *and* on the first keypress |
| A fix deployed and did not arrive | the build stamp, and `app.js`'s self-heal — see *A deployed fix is not a delivered fix* in CLAUDE.md |
| The board draws the wrong shape | something cached what core can be asked. The held component is looked up by id every frame |
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

**Nothing is scheduled.** `PLAN-M8.md` is finished, all nine milestones, and
the demo ends at a door in the western wall. What is past that door is not
written, and the game's overall structure past it is the human's to decide —
`PLAN-M8.md` §5.6 is where that question is written down. Nothing in the code
assumes a second overworld, a chapter count, or an ending beyond the one the
door gives.

The two things most obviously worth doing next, and neither is a defect:

- **Place a second town.** Kettleworks and High Wick are written, shelved and
  given errands, and are on no map. The pit's eleven-line shelf is the whole
  economy, and it is why a player who buys everything still cannot fill five
  frames without errand rewards.
- **Something past the door.** See above.
