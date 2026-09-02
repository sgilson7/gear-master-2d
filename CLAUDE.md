# CLAUDE.md — operating notes

Kept current. If something here is out of date it is a bug in this file.

**How to read it.** The top is the part you have to know before you touch
anything: what this is, the rules that are load-bearing, and the commands. Then
six parts, one a system, each holding that system's design decisions *and* the
mistakes it cost to arrive at them. The mistakes are kept on purpose — every
one of them is a day somebody spent — but they are filed under the thing they
happened to rather than in the order they were found, which is how this file
was arranged until it was long enough that the arrangement mattered.

`HANDOFF.md` is the short door in. Read that first if you have never seen this.

## What this is

A 2D tile-based open-world RPG built on the gear-assembly auto-battler forked
from `sgilson7/gear-master`. `PLANNING-BRIEF.md` is the brief; `PLAN.md` is the
plan and **wins where the two disagree**; `TONE.md` governs every string a
player reads.

**Every milestone is done, M0 through M8.** M0–M5 shipped the MVP, tagged
`v0.1.0-mvp`; the board was rebuilt against the original's colourblind design;
M6 added the art and the tone pass; M7 the shops, errands and the first
dungeon; M8 curses made visible, a quest log, enchs, a fourth class, and a door
at the end of it. Live at <https://sgilson7.github.io/gear-master-2d/>.

**The demo ends at a door in the western wall**, which appears once the Cave's
boss is down and opens to the key it drops. What is past it is not written —
not hidden, not locked, not saved for later. The game's overall structure past
that point is the human's to decide and `PLAN-M8.md` §5.6 is where the question
is written down.

**No rest point, and there still should not be one.** Combat health resets
every fight, so a rest would restore something that was never spent. What a
town does instead is where the design landed: it is the only place experience
becomes a level, everything you are carrying is lost if you fall before you
reach one, **and it takes the tiredness off** — which is the one thing a fight
does spend for good. The thing the old note was looking for was never a rest;
it was a stake.

## Rules

Break one of these and the failure is silent and expensive. `HANDOFF.md` picks
five of them out as load-bearing — core stays graphics-free, the shim decides
nothing, content lives in `data/`, a new field is a compile error until the save
carries it, and the page never recomputes a number. The rest are here because
something cost a day.

- `crates/core` never imports `wasm-bindgen`, `web-sys`, or anything
  DOM-shaped. If you are reaching for one, stop.
- `crates/wasm` is a shim. It moves strings across the boundary and decides
  nothing. A rule decided there is a rule the test suite cannot reach in
  seconds, and then there are two rulebooks.
- Content lives in `data/*.json` — the map, the events, the tree, **the town
  shelves (`shops.json`) and the errands (`quests.json`)**. **If you are editing
  a `.rs` file to change what a player reads, you are in the wrong file.** The
  two exceptions are inherited and known: the component catalogue is `piece.rs`
  and the theme tables are `theme.rs` (mirrored into `data/theme.*.json`, which
  is generated — `REBASELINE_THEME_DATA=1`).
- **A new component needs a themed name in the same change.**
  `the_turtle_theme_covers_the_catalogue` fails otherwise, and it is right to:
  a piece nobody has named reaches the player in the engine's words.
- **Adding to `CATALOG` changes the save fingerprint**, and older saves are
  refused with a sentence naming both catalogues. That is the design; say so in
  the commit when it happens.
- Never write a game string without `TONE.md` open.
- **Save round-trip tests run on every commit. A red round-trip blocks
  everything.** `tests/save.rs` is that suite; `testing/drive.py` walks the same
  property through three real browsers.
- **Adding a field to `Game` is a compile error until the save carries it.**
  `SaveFile::of` and `into_game` destructure exhaustively. Two fields are
  skipped on purpose and each says so where it is skipped. Do not "fix" a
  destructure by adding `..`.
- **The agent does not run `git push` or `make publish`.** Only a human
  deploys. (The one exception was the repo's creation and first push, which the
  human asked for explicitly. The rule is back in force.)
- Do not start a milestone before the previous gate is live and the human has
  seen it.
- **The page draws numbers core sent it, and never recomputes one.** Violated
  three times and invisible every time: the replay once subtracted damage from
  a health total it kept itself and ignored `absorbed`; it once opened every
  fight with an empty armour bar because nothing *announces* a balance nobody
  had to earn; and a curse's countdown was nearly divided out of the playback
  head, which would have drawn a shape the fight never had.
- **Before adding a system, grep for it.** `explain.rs` was written with a
  duplicate `Action::describe` and `Trigger::describe` already in `piece.rs`,
  and M8 opened with a request to add curses to a game that has had 59
  cursing components since the fork.
- **Break a new check and watch it fail before you keep it.** Three checks have
  shipped vacuous. A check that compares zero with zero is not a check.
- **A derived number needs somewhere it is shown**, or it cannot be told from a
  bug. Four skills worked perfectly and were reported as broken because nothing
  printed them.

## Commands

    make test          # the engine suite, native, seconds
    make check         # fast type-check
    make web           # build dist/web/
    make test-ui       # drive the built page in three real browsers
    make play          # play the demo start to finish and read every screen
    make serve         # build and open locally
    make art           # compile art/*.tex to web/assets/*.svg
    make test-ui-setup # one-time: venv + headless chromium

**`make test-ui` and `make play` are different tools.** The first walks a route
chosen to exercise checks and asserts; the second starts a new game and plays
it, and its output is a transcript rather than a verdict. The second is the one
that found an Auto-pack seating the starting kit for the whole game and a class
fork opening underneath the town — both of which the first was green through.

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

    REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core

---

# Part one — the world you walk on

## The world

- **Danger is measured, not typed.** A region's danger is the mean of
  `rating::creature_rating` over its enemy pool.
  `tests/world.rs::no_data_file_types_a_danger_number` fails the build if a
  number ever appears in a data file. Tuning the map means moving creatures
  between pools; typing a number would be tuning the ruler.
- **Every roll is integer per-mille.** A seeded walk has to produce the same
  encounters in every browser, and float rounding is the one thing that breaks
  that silently — the symptom would be a save that replays for the person who
  wrote it and not for the person they sent it to.
- **A blocked step draws nothing.** Bumping into a cliff must not advance the
  stream, or a replay would depend on the player's mistakes rather than their
  path.
- **The map is never saved.** `WorldState` holds a position, an answered set
  and flags. The grid is `data/tiles.json`, and content is not state — the
  discipline is borrowed from upstream's `county.rs`.
- **The page draws numbers core sent it.** An earlier draft recomputed the
  encounter chance in JavaScript for the debug overlay, which put the formula
  in two languages with only one of them tested.

## The first map has one town

One town and a great deal of wilds. Kettleworks and High Wick are written,
shelved and given errands, and are **not placed** — they belong on maps that do
not exist yet.

- `towns_on_this_map_all_trade_and_all_want_something` replaces a set equality
  that held only while one map carried every town. The direction that still
  holds is that a town you can walk into trades and wants something; the other
  direction is a named `STAGED` list, which is the point: **content waiting for
  a map is fine and content waiting for nothing is an orphan, and the only
  difference is somebody having written the name down.** A third stray shelf
  fails there.
- A save can now remember a town this map does not have — a file from before
  they moved, or from a map this build does not ship. `World::repair` falls
  back to the start, and `world.rs` tests exactly that; without it such a save
  would put a player nowhere.

## A save never places you where you cannot stand

`WorldState` is `#[serde(default)]` in the save so that files written before M2
still open — and a default `WorldState` stands at `(0, 0)`, which on this map is
rock. A player carrying an autosave from an older build spawned inside it and
could not move in any direction.

**Anything that loads a position runs `World::repair`.** It puts the player at
their last town if one is known and walkable, and at the map's start otherwise.
Two core tests and one browser check hold it, the last of which plants the exact
file rather than waiting for one.

The general rule: a field defaulted for backward compatibility is a field that
will arrive wrong, and the loader is where that is caught.

`try_step` repairs too, not only `load_json`. A position you cannot stand on is
a dead end rather than a glitch — there is no key that gets you out of it — so
the first keypress fixes it whatever put it there.

## The first dungeon

`data/dungeon.json`, nine by five, one way in and one thing at the end of it.
Short on purpose: fatigue is the budget, and the walk back out is part of it.

- **Maps are a list.** `data::MAPS` is `(id, json)` and `WorldState::map` says
  which one you are on. A map this build has not got falls back to the
  overworld rather than panicking — a save can name one, and `World::repair`
  then finds them somewhere to stand.
- Two new `PlaceKind`s. A **Gate** is a way onto another map, and whether it
  opens is answered in the shim rather than in `World`, because it depends on
  what is in the bag and a map does not know about bags. A **Boss** is a
  creature standing on a tile rather than one the ground rolled.
- **A boss drop is looked up by the tile, not the creature.** The same Rust
  Colossus stands in a region's pool; beating one in a field must not hand over
  the way to the next map.
- **Dying in a dungeon walks you home across maps.** It used to leave you
  standing on the boss's own tile, because the walk home looked for a town on
  the map you were on and a dungeon has none.
- `every_gate_leads_somewhere_you_can_stand` is the one that matters: a gate
  whose far side is a wall strands a player on a map with no way off it, and
  nothing else in the game would say so.

### `map()` must not reach for the game

`map` used to look `WORLD` up as a single static. Following the player means
knowing which map they are on, and the first version read `GAME` to find out —
which is a `RefCell` double borrow at nearly every call site, because they are
all already inside `with` or `with_mut`. In a wasm build that is a bare
`unreachable` on the console and **nothing else to go on**.

`map_for(g, …)` takes the game. Where the closure also mutates it, the id is
resolved first and `map_named` is used, because borrowing the game to find the
map and then mutating it inside the closure is the same fault one level up.

## The door in the wall, and the screen that is not a loop

The key from the bottom of the Cave had nothing to open until M8.7. Three
firsts, and each one is a rule:

- **A place can be conditional, and it is still content.** `PlaceDef::hidden_until`
  names an id that has to be in `answered` or `flags`; until then the place is
  not drawn, not steppable and absent from `World::place_now`. Spawning one at
  runtime was the other option and is rejected for the reason the map is not in
  the save: **places are content and content is not state.**
- **`place_at` is the file; `place_now` is the game.** The first answers
  questions about the data — where the last town is, whether the map is well
  formed. Everything a player can see or walk into goes through the second.
- **What a door *wants* is still the shim's.** `hidden_until` reads `answered`
  because a `World` does not know about bags; `needs` names a component and is
  answered where the bag is, exactly as a gate's key is.

`Quest::requires_answered` is the other half: an errand gated on something that
is not another errand. A boss tile writes its own id into `answered` when it is
cleared, so "once the Cave is done" is one field rather than a second kind of
prerequisite.

The ending screen does **not** take the fork's treatment. You can back out of
it, because the world is still there behind you and there is an errand about
the door to hand in. What it must not do is pretend there is more.

---

# Part two — the board, and the fight it runs

## Things the fork learned the expensive way

Three facts about the engine that are invisible until they cost you a day. All
three were found by the golden fixture in M0, and all three are fields M1's
save file has to carry.

1. **`Loadout::locks` is state, not geometry.** Two pieces that touch are one
   item unless a lock says otherwise, and which locks exist depends on the
   order the player built in. Re-deriving them gives a different board: the
   first fixture rebuild came back with more items than it went in with.
2. **`Loadout::name_seed` seeds the name hash.** Drop it and every stat
   survives a round trip while every item is renamed — "Resonant Sliver" comes
   back as "Resonant Thorn" and nothing else looks wrong.
3. **`PieceId` is an index into `PieceRegistry`.** The registry is saved whole
   and in order, by canonical catalogue *name* — never by catalogue index,
   which is only stable while catalogue order is.

And one from upstream, inherited deliberately: **lock each item as it
assembles, not once at the end.** A finished board is packed to within a cell
or two of full, so deriving items in a single pass at the end asks which pieces
are connected and gets "most of them". `share.rs` learned this when nineteen
weapon pieces came back as one item.

## Inherited on purpose — do not "simplify"

No RNG in combat. 50 ms ticks. Monsters are loadouts wearing catalogue pieces.
The naming system. These are the reason to keep the engine.

## The fight

- **Combat has no RNG**, which is why a mid-fight save carries a creature name
  and a tile and nothing else. `PLAN.md` §6 proposed storing the pre-fight state
  and the seed; the engine made both unnecessary.
- **The page decides nothing about the board.** The green fit preview *is*
  `legal_anchors` rendered. `testing/drive.py` picks a piece up and compares
  what the board painted against what core returned, so a page that started
  computing its own answer would be caught rather than trusted.
- **The auto-pack button seats only what you own.** It briefly handed out any
  missing component, which made it a supply of free gear and the shop
  pointless.
- **A loss pays nothing and walks you home.** Visible in play now, not just in
  `reward.rs`.

## Auto-pack packs what you own

**The bug M8.8 found by playing, which every test was green through.**

Auto-pack seated a fixed list of twenty-two component names and skipped
anything not owned — and it took that branch only when the weapon frame had
reached eight rows, which is level twenty-something. Below that it seated the
two-piece *starting kit*. So for essentially the whole game, a player who
pressed the button they were given with a bag full of gear got an Oak Handle
and an Iron Blade.

Even past the gate, five of the eleven things the only town on the map sells
are not on that list. With every component the map can hand out, the board came
to **two assembled items of five frames** — and two items lose to the Cave's
boss, so the key never dropped, the door never appeared, and the demo could not
be finished.

The list was not wrong when it was written. It was written against a starting
kit of eleven components and outlived it by three milestones. **A list of names
is a second copy of what the shops sell**, and the two drifted the moment the
shelves became content.

What replaced it, and why each part of it is there:

- **Seed on a core, grow what improves.** Two components that touch are one
  item, so *packing more is not packing better*: a seven-row weapon frame
  packed solid is one enormous group that assembles nothing at all. The first
  rewrite did exactly that and handed a character carrying every reward on the
  map a weapon frame full of books and no weapon — it lost to a Cave Rat.
- **Every placement after the seed must strictly improve** `(items assembled,
  what they rate)`, and is taken straight back out otherwise.
- **A seed that led nowhere is taken back out too.** A lone core is a component
  doing nothing in a cell somebody else could have used.
- **Deterministic.** Best-rated first, ties by id. A seeded walk that repacked
  differently on two machines would be a seeded walk that fought different
  fights.
- **It is not an optimiser and must not become one.** The whole game is the
  arrangement; a button that packed perfectly would be a button that played for
  you. What it has to do is leave nothing obvious in the bag.

`tests/common/mod.rs::build_full_loadout` is the old arrangement, kept as a
**fixture**. Four tests wanted "a known full board" and were reaching for
Auto-pack to get one; they are about recipes and about `hit_for`, not about the
button, and sharing one list meant a change to how the button packs broke tests
about what an assembly bonus does.

## What you are about to fight

Only the Cave Rat has innate attacks. **All forty-nine other creatures fight
purely out of their gear** — so a fight screen that printed a name and a rating
was hiding the entire fight. `encounter_json` had carried the creature's item
names since M1 and the page rendered none of them.

- The creature's cards come off the same `item_card` in `crates/wasm` that the
  player's do, and render through the same `cards()` in `app.js`. Two copies
  would be two answers to "is cork a standing stat", which is the question the
  two halves exist to settle.
- `web/theirs.js` draws its board read-only. It imports `paintMotif` from
  `board.js` rather than reimplementing it — the motif is the *shape* half of
  the colourblind triple-encoding, and everything that draws a cell must draw
  the same one.
- **Every relative import in every shipped module is stamped, by pattern.**
  The stamping was a list of module names written out by hand, and both times a
  module was added it was left off — `theirs.js` first, then `shape.js`, each
  importing `board.js` two hops from the entry point, which is exactly where a
  stale mix hides because the page itself looks fresh. `package-web.sh` now
  rewrites every `from './x.js'` it finds and **dies if any bare import
  survives**, which is the check that catches the next one rather than the last
  one.
- `#made` holds two panels now, so anything querying `.made-item` must scope
  itself — an unscoped query lit a creature's card when you pointed at your own
  blade.

## Watching a fight

- **Both boards tick.** Only the Cave Rat has innate attacks, so for the other
  forty-nine a replay showing one side's cooldowns was showing half the fight
  with no way to tell which half.
- **Rows are HTML, bars are canvas.** A row you can point at is a row the
  browser can tell you about; 11px canvas text can be hovered by nothing. Same
  lesson the item list learned when it came off the board canvas.
- **Nothing is computed that the log reports.** Armour comes off
  `Hit::target_armor` and `GainArmor::total`; the four pools come off
  `GainResource::total`, `GainMana::total` and every spend's `remaining`. This
  is the health bug generalised — that one subtracted `damage` from its own
  total and ignored `absorbed`.
- **The armour bar wraps, it does not clamp.** Lifted from the original with
  its reasoning: the two bars read as a pair because they are the same
  measurement, so a full armour bar is as much armour as you have health and a
  pixel is the same number of points in both. Past full each complete bar is
  another layer drawn darker than the one under it. Clamping made every amount
  from "exactly enough" to "four times over" draw an identical bar.
- **The armour label is haloed, not coloured.** The ground under the middle of
  that bar is whatever layer the wrap landed on — the palest shade and the
  empty track are both possible under the same text, and no single ink reads on
  both.
- **The replay panel draws on its own dark ground and uses its own ink**, the
  same as the board. Taking the page's ink put dark labels on a near-black
  panel every time the viewer was in light mode.
- One `oneCard` in `app.js` renders an item for the packing panel, the
  creature's panel and both sides of the replay. Four places, one answer to "is
  cork a standing stat".

## Both boards, and the jolt

- **A fight is two boards.** The replay drew neither; it now draws both,
  read-only, through the same `Theirs` painter the creature panel uses.
  `side_slots` in the wasm shim builds them for the panel and for both sides of
  the replay — one builder, so three screens cannot disagree about a cell.
- **What fires jolts.** A decaying wobble, 260ms, driven off the same
  activation times the cooldown bars are: six items on two boards all coming
  round at their own rates is unreadable, and movement says *that one, now*
  where a colour change would be five things happening at once.
- The shake is set from outside — `Theirs.shaking` is a list the replay writes
  and the painter reads. The painter decides nothing about when.
- **An innate attack has no cells.** A creature's bite stands on no gear, so
  nothing on a board moves for it; the browser check skips a fight where only
  the bite went off rather than failing one. Which activations are shakeable is
  a property of the fight.

## Curses were always there, and nothing said so

Reported as *"are curses in the game? if not, they need to be added"*. They
were, and always had been: **59 of 536 components apply one**, six are on a town
shelf and two are on the *starting* shelf at three Fnorp each. What was missing
was every screen that should have mentioned them.

Before adding a system, grep for it. This is the second time this project has
nearly built something twice — `explain.rs` was written with a duplicate
`Action::describe` and `Trigger::describe` already in `piece.rs`.

Three screens, and each was a different way of saying nothing:

- **The item card had no arm for `Action::Curse` at all.** It has a third group
  now, off `explain::curse_lines`, which is `Trigger::describe` filtered through
  the engine's own `walk_actions`. The sentence names who it lands on, so a
  piece that curses its own wearer reads as the downside it is.
- **`Event::Cursed`, `Warded` and `Stunned` fell into `fight_json`'s
  `_ => ("other", …)`.** A Whisperling could stack frost on you for a whole
  fight and the panel did not move.
- **Nothing showed a curse that was up.** The replay draws chips now, per side,
  beside the pools: the curse, its stacks, what it is *doing* — "30/s", "-75%",
  "1 in 2", off `CurseKind::effect_at`, which reads the same constants the
  simulation does — and a countdown.

**Read, never derived**, the rule the health bar and then the armour bar each
had to learn. A chip is `{kind, stacks, until}` where `until` is the event's own
timestamp plus the duration the event reported. Expiry produces no event, so a
chip is dropped when the clock passes it — pruned once by the entry's time and
again by the playback head, which covers the gap between two entries.

**A stun is its own event because it rides on one named item.** Two items
stopped at once is two chips, not two stacks, and a check that only ever saw a
curse would let that arm rot.

## The soft-lock M4 shipped and then found

For an afternoon the game was **unwinnable from its own first tile**, and every
test passed.

`apply_preset` is an eight-row arrangement and `Balanced Grip` is one cell wide
and four tall, so on a three-row starting frame the weapon had no handle and
assembled nothing. A starting character walked out of the pit with one glove,
lost every fight, and — because a loss pays neither gold nor experience — had
no way to buy or grind out of it.

Two things now stop it happening again:

1. `a_starting_character_can_win_in_the_pit` asserts the starting kit assembles
   a weapon and beats something in the region it starts in.
2. The calibration test **fights for real** instead of assuming every encounter
   is a win. The version that assumed wins measured how much the map offers
   rather than how much a player gets, and would have gone on passing.

---

# Part three — what a character becomes

## Levels

- **The level is derived from experience, never stored.** Two numbers that
  could disagree is two answers to one question, and a hand-edited save should
  produce a consistent character rather than a contradictory one.
- **Board size is a pure function of level plus granted rows.** So it can be
  checked rather than trusted. `resize_boards` only ever grows: a board that
  got shorter would drop whatever was seated in the rows it lost, silently.
- **A skill's *effect* is not state — the node is.** The tree is re-read on
  every load and every stat query, so retuning a node retunes every save that
  took it.
- **`XP_DIVISOR` is set by a test, not by taste.** It is 5 because that puts
  level 5 at a mean of ~27 fights across nine seeded walks of the pit. Moving
  the map's regions moves this; the band is the contract.

## Experience is carried, and a town is the bonfire

**A fight pays into your pocket. A town is the only thing that turns it into a
level. A defeat takes everything you are carrying and nothing you have spent.**

- `Character::xp` is what has been **spent**, and the level is derived from it
  and nothing else — the old rule holds, it just names a different number.
  `Character::carried` is what is on you. Two numbers, and not two answers to
  one question: one is what you have become and the other is what you are
  going to become.
- `carry` on a win, `bank` in a town, `drop_carried` on a defeat. **Nothing on
  the road calls `gain_xp`** — that is the spending primitive and `bank` is its
  only caller.
- `Settlement` lost `levels` and `grew`, because a fight cannot produce either
  any more. They are `fight::Banking`, which is where they happen.
- **Banking spends the whole pocket at once**, so it can cross several levels
  in one go: a character who walks home carrying a hundred and forty arrives at
  level five having passed two, three and four on the way. Anything asserting
  "the fork opens at exactly five" is wrong now — it opens at five *or more*,
  at the banking that crossed it.
- The class fork is offered from the town, because that is where a level lands.
- **Health was already free.** Combat health resets every fight — `Combatant`
  is built from `Stats` at the bell and nothing persists — so "health recovers
  after a fight" needed no change and is not one. What the souls rule adds is
  the thing that *is* at stake, which is what the note at the top of this file
  meant by there being no rest point: there was nothing to restore. Now there
  is something to lose.

### What it did to the browser walk

Two failures worth keeping, because both were the walk telling the truth:

- **The patrol cannot get you home.** `PATROL` is six steps east and six west;
  from more than six tiles away it never finds the town again. That did not
  matter while a fight levelled you on the spot, and it is the whole loop now.
  The grind heads for the nearest town once it is carrying enough
  (`head_for_town`), which is what a player does. It failed on one run and
  passed the next before that — a flaky gate is worse than a red one.
- **A level lands wherever you happen to be standing in a town**, not in the
  fight that earned it, so the receipt naming the frame is the town's.
  `BANKINGS` records every banking receipt as the walk makes them, and the
  level-up check reads that rather than the receipt of one particular fight.

## Classes

- **Four, and every one of them is upstream's.** Gorillathon, Funnel Sergeant,
  Worm-Fact Keeper and the Kaklon Licensee are `Berserker`, `Hexweaver`,
  `Bloodletter` and `Recycler` with the theme talking, so the powers — Leeching,
  Contagion, Bloodscent, Recycler — are already tuned and already tested.
  **Nothing new has been invented in combat for a class**, twice over now: M5
  took three and M8.4 took a fourth, and the fourth's identity is the ench rack
  rather than a new rule in the fight.
- **A promise must describe the game the player is in.** `Recycler`'s said
  "for each stack of Recycler you are carrying. Five stacks is half again on
  all five slots" — upstream handed the same class out repeatedly and a promise
  had to say what a second one bought. GM2D asks once, at level five, and the
  answer does not come off. `no_class_on_offer_promises_a_stack` is the lint.
- **The promise is the rule.** Each class's one-line mechanical promise is
  `ClassPower::describe()` put through `theme.retell`, so it cannot go stale and
  it speaks the game's language rather than the engine's.
- **The fork is permanent and offered until answered.** There is no path that
  clears a class; the screen is the only one in the game that does not take
  Escape. A save made at level three arrives at five and is asked, and one made
  at nine without a class is still asked — the question was never answered
  rather than declined.

## Eight skill nodes that cost a point and did nothing

Found while making the tree describe itself, and the reason that job was worth
doing properly.

`Effect::Stat` carried `armor` and `mana`. Both are **grants an item makes on
its own tick** everywhere else in the engine — `RunningItem` pays them on every
activation — so a *character-level* total of them has no tick to hang off, and
`Combatant::player` had always started both at zero and thrown the total away.
Eight nodes granted one or the other. They parsed, they cost points, they
showed as taken, and they changed nothing: `Corked`, `Funnel Drill`,
`Bedazzled Plaid`, `The Five`, and the whole spine of the Hexweaver tree —
`Army Issue`, `The Banana Standard`, `Anvil, Own Foot`, `A Funny Undone`.

The fix is a separate effect that says what it means:

- `Effect::StartWith { armor, mana }` — what you are already holding at the
  bell — and `combat::Held`, passed beside `Stats` rather than inside it,
  because folding it in would pay every item's armour again as a balance.
- One more rung on the simulate ladder (`simulate_holding` /
  `simulate_party_holding`), which is how every other run-only concern has been
  added: the existing signatures are untouched and no test had to say it holds
  nothing.
- `Node.effects` is a list now, since four of the eight granted a stat **and**
  a balance. It reads as one object or an array in the JSON, because most nodes
  do one thing.

**Why serde let this happen, and the lint that catches the next one:**
`deny_unknown_fields` is a container attribute, not a variant one, so it cannot
be put on `Effect::Stat`. serde therefore drops a key it does not know without
a word. `every_effect_key_is_one_the_engine_actually_reads` in
`tests/skills_read.rs` reads the raw `data/skills.json` and refuses any effect
key outside the vocabulary. Reading the parsed struct could never have found
this — the whole failure is that the parse succeeded.

## A skill has to say what it does

The tree described itself only in the world's words. *"Nine hundred feet of
Deep Chocolate mine, and you never once came up early"* is a good sentence
about a character and tells nobody it is sixty max health. Reported by the
human as *"completely unintelligible as to what they do"*.

Two registers, kept apart, and `TONE.md` rule 13a is the written version:

| | written by | speaks |
|---|---|---|
| `name`, `blurb` | a person, in `data/skills.json` | the book |
| `Node::line()`, `Node::detail()` | **derived in core from the effect** | the engine |

- **Derived, never typed.** A spec nobody writes by hand cannot disagree with
  the effect it describes. Retuning a node retunes its description.
- **Unthemed on purpose** — the one exception to rule 13. Somebody choosing
  between two nodes is comparing numbers, and a number wearing a joke has to be
  translated first. `no_mechanical_line_speaks_the_theme` enforces the inverse
  of rule 13 over exactly this text.
- `line()` is the one-liner on the button; `detail()` explains the words in it
  and appears on hover **and on focus**, so a keyboard reaches it.
- The class fork prints `power.describe()` raw. It used to go through
  `theme.retell`, which turned the one sentence somebody reads before an
  irreversible choice into a sentence about the Roast and the Nut Freeze.
- Check every number you put in a description. `SPELL_MANA_COST` is **3**, not
  30; the first draft of the mana line said "that many casts", which is not a
  number at all.

## A skill that works and cannot be seen is a skill that does not work

Reported from a real session: four nodes taken — `Corked`, `Funnel Drill`,
`Cave Lungs`, `Handspan` — and *"I am receiving none of the start of combat
bonuses and I cannot tell whether I have received the strength or not."*

**Every one of them was working.** Twelve armour soaked blows for the whole
fight. The engine was right and the screen said nothing, which from where the
player sits is the same thing as a broken skill.

Two faults, and they are the same fault twice:

1. **The fight opened at zero.** `fight_json` seeded its running snapshot with
   `armor = 0` and empty pools, then updated on events — and *nothing announces
   a balance nobody had to earn.* The only armour event reports what is **left
   after a hit**. So the bar sat empty until something took a swing at it.
   `CombatLog::player` is `start_player`, the fighter as the bell went, and it
   has carried the answer all along; the snapshot seeds from it now.
2. **Nothing showed the character sheet.** `character_json` had emitted stats
   since M5 and no line of code read them. +6 strength and +60 max health
   landed in a number no screen printed. `#sheet` prints it, and prints what
   the tree says you begin a fight holding.

Rules that came out of it:

- **A derived number needs a place it is shown**, or it cannot be told from a
  bug. The test that would have caught this is not a unit test — core was
  correct — it is `check_the_sheet_says_what_you_are`, which fails when core
  reports a non-zero stat the sheet drops.
- **The sheet speaks the node's words**, not the theme's: a node reading "start
  every fight with 12 armor" against a sheet reading "12 cork" is one number
  with two names, and the whole job of the line is to let somebody confirm they
  got what was promised. An item card still says Cork — a card is about the
  item, not about a promise being checked.
- **A check that compares zero with zero is not a check.** The first version of
  `check_a_starting_balance_is_on_the_bar` compared the log's opening armour
  with the bar's, and a character on the gate's walk holds nothing, so a build
  with the bug hard-coded to zero passed it. It feeds a log back with a balance
  on it and reads the opening row now. Negative-test every new check by
  breaking the thing it guards.

## The tree is a tree

It was one flat rack of buttons, which told you what existed and nothing about
what led to what.

- **Rows are depth, and depth is core's.** `Tree::depth_of` is 0 for a node
  with no prerequisite and one past the deepest thing it needs otherwise;
  `Tree::rows` groups by it. A screen working its own layering out would be a
  second answer to "what has to come first", and the two would part the first
  time a node gained a second prerequisite — `w-law` already has two.
- The top row is exactly **what you can spend a point on with an empty sheet**,
  which is the question somebody opening the screen is asking.
- **Within a row, order by the average position of the parents.** The cheapest
  thing that keeps the lines from crossing, and it puts a node over the things
  that need it.
- **Wires are measured, not computed.** The rows are flex and wrap, so where a
  node actually *is* is the only thing that can be trusted; `drawWires` reads
  `getBoundingClientRect` after layout and redraws on resize. Elbows rather
  than diagonals — a straight line through three rows of buttons is
  unreadable.
- A wire into a node whose prerequisite is taken is lit; the rest is
  scaffolding. `.open` outlines a node you could take right now, because the
  tree is mostly locked at any moment and the few open doors are what wants
  finding.

**One tab a tree**, and it is built for a list rather than a pair: a character
has the base tree plus whichever class trees they have unlocked, and there will
be more than one of the second kind. `all_trees_json` already returns exactly
the trees a character may spend in, so the tabs are however many that is.

Two things this broke, both worth knowing:

- `#tree-tabs` carries `class="tabs"` and is **not** inside a `.made` panel,
  and the tab styling was written as `.made .tabs`. It inherited nothing and
  the buttons stacked. A style scoped to a container is a style the next user
  of that class name will not get.
- The fork's browser check counted `#nodes .wares` and asserted "more than the
  base tree". Only the open tree is drawn now, so that question stopped
  meaning anything; it counts tabs and opens the class one instead.

## Skills that grant rules, not numbers

The tree could grant a stat, a starting balance, a row on a frame and a
percentage on every assembly bonus. All four are arithmetic. `Effect::Grants`
is the fifth kind and the first that says the game works differently for you
now.

**`Rule` is an enum, not a string**, and there are three locks on it, because
this project has shipped the other thing:

1. An exhaustive match wherever a rule is consumed.
2. `deny_unknown_fields` on the enum — a container attribute, which is exactly
   why it could not go on `Effect::Stat`.
3. `Rule::check`, run by `SkillsData::parse`, which refuses a rule naming a
   grid or a curse the engine has not got, or a tuning that tunes nothing.

**A granted rule is a fight input, not a mutable global.** It reaches combat
through `Held` — the same door the tree's armour and mana go through — and is
translated into a `Combatant` field at the bell, the way a `ClassPower` is.
Combat stays a pure function of what it was handed, which is the property a
mid-fight save carrying a creature name and a tile rests on.

`Rule::CurseOnActivate` fires in `activate` beside the item's own triggers,
**not folded into the profile**: a profile is the *board's* answer and this is
the *character's*, so two players with the identical board do not have the
identical fight, and an item's card must not start claiming a curse the item
does not own.

**`Rule::Scout`, and `#numbers` is gone.** That button was a debug overlay that
shipped, and it handed the region's danger and every tile's odds to everybody
for nothing — which makes a node granting them a node granting nothing. The
figures are `null` until the reading is earned and the panel says "you could not
say" rather than printing a zero: **zero is a number and would be a lie, and a
screen cannot tell a lie from a bug.**

The plumbing was built two milestones before the Kaklon Patent wanted it, and
that is the whole reason M8.6 could write eight nodes instead of writing them
twice.

## Enchs, and the spin

**An ench is not an enchantment.** `PieceKind::Enchantment` is thirteen
catalogue pieces laid *under* the grid so gear sits on top of them — upstream's
terrain model, and a different mechanic. The book has its own word for the other
thing (the ench economy, p. 119), so the two words stay two words: no rename, no
migration, and nobody has to work out which of two meanings a sentence is using.

An ench is not a component either, for the three reasons a restorative is not
one: no shape, no grid, attached rather than worn.

- **The attachment is to the piece, not to the cell.** `Ench::on` is a
  `PieceId`, so an ench survives being picked up, turned, moved to another grid
  and put back down. A cell would have meant it falling off on every repack.
- **Both effects are numbers the engine already had.** `power` and
  `cooldown_ms` are what `PieceDef::power_bonus` and `speed_bonus` already move.
- **They land in `Character::combat_items`, not in `Loadout`.** A profile is the
  board's answer and an ench is the character's; a loadout that knew about enchs
  would be a loadout that knew about a licence.
- **The class is the gate**, not a node inside it. Enching is what the Kaklon
  Patent *is*, and a class whose identity waited on a point spent is a class you
  could take and not notice you had taken.
- **The mark is the fourth channel**, after motif, luminance and hue, and it had
  to be told from the lock's gold outer edge and the assembled item's pulsing
  white one. So it is drawn *inside* the component, where neither of those goes.
- **A priceless ench is on nobody's bench.** `price` is optional, and
  `QuestsData::parse` refuses an errand that pays one that has a price — a
  reward you could have bought makes the errand a slow way to shop.

### The spin

> *"if they are blocked and cannot rotate, then they do not move"*

**Rotation is decided on the board and banked in the fight.** Combat has no
board — `ItemProfile` is a flat snapshot, which is why a mid-fight save carries
a creature name and a tile and nothing else. So `Slot::turn_cycle` works out at
pack time which of the four orientations an arrangement can reach *in place*,
and the fight ticks through a list it was handed.

- **Deduplicated by the cells produced.** A one-by-four turned twice lands on
  itself; that is not a second orientation and would have paid a stack for a
  turn nobody can see.
- **Leaving room to turn costs you cells**, which is a real packing decision of
  the kind `PerAdjacentEmpty` already trades in. The spin is not free power; it
  is power bought with space.
- **The spend is the cap.** `SPIN_PCT_PER_TURN` is uncapped because a slow item
  stacks more and fires less; a ceiling would have had to be tuned against every
  cadence in the game.
- **`Event::Turned` and `Event::Spun` are logged rather than left to a clock.**
  A frosted item turns slower for the same reason it fires slower, so a screen
  dividing the playback head by a second would draw a shape the fight never had.
  The packing board has no fight to read, so it turns on the wall clock, which
  is the honest one there.

---

# Part four — the road, and what is on it

## The economy, and why the shelf stopped rolling

Three changes that are one change: **a character starts with almost nothing, a
town sells a fixed shelf, and a town asks you for something.**

- **The starting kit is `Oak Handle` + `Iron Blade`.** It was eleven components
  — most of a helmet, a pair of molds and a whole weapon — which made the shop
  decoration for the first hour. Two pieces assemble one weapon that beats a
  Cave Rat and a Bog Toad and loses to a Bone Archer, which is the opening.
- **The Iron Blade is seated turned, and has to be.** It is one cell wide and
  **four tall**; a starting weapon frame is three rows. Upright it does not fit
  anywhere, the weapon assembles nothing, and a character who cannot win cannot
  earn — the M4 soft-lock, exactly. The fifth field of a `STARTER` row is the
  rotation and this is what it is for.
- **A shelf is content.** `data/shops.json` holds each town's stock and it never
  changes; the save carries `WorldState::bought`, which is a town id and an
  index. Same discipline as the map. `Game::shop` and `ShopSave` are gone, and
  a save written before this still opens — serde ignores the key it no longer
  knows, and the shelves it arrives to are the shelves everybody has.
- **The index is the identity**, so a bought entry is greyed and left where it
  was. Dropping it would renumber the list and a save saying "bought number
  three" would come back pointing at something else. It also just reads better:
  the gap is the memory of what you took.
- **Append to a town's stock, never insert.** Same reason.
- Reroll and pinning are gone with the random shelf. A town that sells
  something different every visit is not a place, and three of them are one
  slot machine in three costumes.

## Errands

`crates/core/src/quest.rs`, `data/quests.json`. **Not** upstream's `quest.rs`
(a chain of receipts along a road, deleted in `48203ee`) and **not**
`piece::Quest` (a component that transforms after N activations). Three things
called quest; this is the only one a town hands out.

- **The tally is a bag item, not a counter.** Beating a toad gives you a Toad
  Eye and the eyes sit in your bag until you carry them back. A counter would
  be simpler and would also mean the errand had no middle.
- **A drop is gated on the errand, not on the creature.** Nothing falls before
  it is asked for and nothing falls after the fifth: a bag filling with eyes
  nobody wants is litter, and a sixth eye is a thing that cannot be handed in.
- Handing in unseats the tokens first. A component handed over the counter and
  still occupying a cell is a component in two places.
- The **ask** is derived and unthemed — `beat 5 × Bengulon Jungle Toad, then
  hand in 5 × Bengulon Toad Eye` — and the **brief** is the world's. Rule 13a
  again. `×` rather than a plural because a creature's name is a proper noun
  and some of them are already plural: The Rice Criers, The Drowned Court.
- **No two errands share a tally.** `holding` counts a token by name across the
  whole bag, so two errands wanting the same one would each see the other's —
  take both, kill five toads, hand in twice.
- **Every town has one**, and every errand names a creature that is actually in
  some region's pool. Both are tests: a town that wants nothing is a shop, and
  an errand naming a creature that is nowhere cannot be finished and nothing
  else in the game would say so.
- **A reward has to be usable.** The first errand pays a book *and* a spell,
  because a book with no spell assembles nothing;
  `what_the_errand_pays_assembles_into_a_weapon` seats both on a starting frame
  and checks a weapon comes out.

## Errands are not a town's

`giver` asks and `turn_in` takes it back, which makes "go and tell them in
town" one errand rather than two; `requires` makes a questline. Three goals:
slay something, bring something, or go somewhere and report.

- **Arriving is the doing.** `quest::on_arrival` runs on the step, so walking
  over the tile and carrying on still counts. The marker goes in
  `world.answered` — the same set a tile-event writes to, so a word and a door
  are remembered the same way.
- `Bring` names **a component or a restorative**, resolved by looking in both
  drawers. The alternative was a second goal kind asking the same question of
  a different list. It also had to be: the shelf sells each entry **once**, so
  "bring me four of a shop item" is impossible and "bring me four tins" is not.
- **No two errands share a tally**, or handing in one would empty the other.
- An errand shows at its turn-in only **once it is on you**: a clerk who has
  not been told about the heap has nothing to say about it.
- **Every reward is unique and on no shelf.** They are in `EVENT_ONLY` too, or
  the creature stepper walks into them — a Harvest Crest turned into
  Marbulon's glass before that was noticed. A reward you could have bought
  makes the errand a slow way to shop.
- **An errand can pay an ench**, in `Quest::enchs` rather than in `reward`,
  because an ench is not a component: no shape, no grid, and it goes in a rack.
  The same rule holds and is enforced at load — `EnchDef::price` is optional,
  and `QuestsData::parse` refuses an errand paying one that has a price. It is
  handed over whether or not the character is licensed: an errand does not know
  what you became, and a reward that vanished for three players in four would
  be worse than one they cannot use yet.

## A log that points at the map

The errands existed, there was nowhere to see them all, and no way to find out
where a Whisperling lives.

- **Where an errand points is core's.** `quest::guide` takes the errand and
  every shipped map and answers in ids: an untaken errand points at whoever
  asks, a full tally at whoever takes it back, a word at the tile you have to
  stand on, and a slaying at every region whose pool holds the creature. A page
  working that last one out would be a second copy of "what lives where", and
  the pools are the one thing on this map that gets retuned.
- **The pin is state.** `WorldState::pinned`, one at a time, off by being pinned
  again, dropped by `hand_in`. It goes in the save for the reason the feature
  exists: a highlight that died with the screen would be a reference, and the
  value is entirely in the walking.
- **The highlight is motion, not a fifth hue.** The map already carries terrain
  hue, region shade, place marks and the player. So the region breathes and the
  ring's dashes march, both in the gold already on the map — and the redraw loop
  runs only while something is pointing.
- **`quest_log_json` is a different question from `quests_json`**: what is on
  you versus what this place wants.

The log is a screen of its own rather than a tab inside the tree. `#tree-tabs`
answers "which tree", and a strip answering two different questions is the
`.card` collision in a new coat.

## Fatigue is what a fight actually spends

Health resets at every bell, which is why a rest had nothing to restore.
**Every battle takes four percent of your maximum health for good, won or
lost.** Two things give it back and they are not the same thing:

- **A town takes all of it off**, on arrival, in the town's own voice —
  `Game::arrive_in_town`, which is core's because "a town mends you" is a rule.
  It is still not a rest: health was always free, and what a town undoes is the
  one thing a fight *does* spend. A defeat walks you home, and home is a town,
  so a lost fight ends rested — the wear happened and the walk undid it.
- **A tin takes some of it off wherever you are standing**, which is the
  decision this mechanic exists to create: another fight, open the tin, or turn
  round. The town is what makes the walk home worth taking rather than a
  formality.

The shelf was retuned when the town started mending — 6/16/40 → **4/11/28**. A
tin no longer buys back a fight, it buys the walk home, so
`a_restorative_costs_less_than_the_walk_home` prices it under what the fights it
undoes pay rather than at several times over, with a floor, because a tin that
costs nothing is not a decision.

- A **percentage**, so it means the same thing at level one and at twenty.
  Twelve points is a third of a starting character and a rounding error later.
- Applied **last, on the total**, in `player_stats`. Taking it off the base and
  adding gear back would make a helmet cure tiredness.
- Capped at 60. A maximum that can reach zero is a character who can neither
  fight nor mend, which is a game over with no screen for it.
- `PER_FIGHT` is set against the pit by
  `a_full_expedition_is_a_budget_and_not_a_wall`, which walks twelve fights and
  refuses a number that makes the second unwinnable or the tenth free.
- Restoratives are **not components**: no shape, no grid, spent rather than
  worn. Three good reasons not to force them into `PieceDef`, where each would
  have been a special case. `data/supplies.json`, and every town sells them —
  a place that had run out of the only thing that undoes tiredness is a place
  you could strand yourself at.
- Drinkable **from the standing panel**, not in town. The decision this exists
  to create is the one on the road: another fight, open the tin, or turn round.

---

# Part five — how it looks

## How the board reads

Lifted from upstream's `crates/gui`, which had a documented, tested,
colourblind-safe design GM2D's first board ignored. **Three channels, any two of
which can be lost:**

| channel | carries | where |
|---|---|---|
| a motif stamped on every cell | the slot | `look::motif` |
| brightness | the role — **cores darkest** | `look::kind_luminance` |
| an Okabe-Ito hue | the slot again | `look::slot_hue` |

- **The palette lives in `core::look`, not in the page.** It is numbers and an
  enum, not graphics, so core stays graphics-free — and the accessibility
  contract is enforced by `cargo test` rather than by looking at a screenshot.
  `tests/look.rs` is ten tests, seven of them ported near-verbatim from upstream.
- **The one number: `ROLE_SEPARATION = 0.08`.** Consecutive role steps must
  differ by that much in luminance *in every hue*. It is why `slot_color`
  bisects for a brightness target instead of picking three HSL lightnesses —
  the same lightness lands at wildly different brightness per hue, and yellow
  flattens its top two steps into one.
- **Assembled versus not is brightness and weight, never gold against red.**
  That pair is the one distinction red-green colour blindness is worst at, and
  the gold collides with the greaves hue. GM2D shipped the rejected pairing for
  two milestones before the original's comment was read.
- **A component is one shape, not a row of tiles.** Cells fill edge to edge; the
  dark edge traces only the true boundary. So a four-cell blade reads as one
  blade, and the lines inside an item are the seams between its parts.
- **A shared component is grey until it is placed**, and takes its grid's colour
  and mark as it crosses in — which shows the rule without stating it.

## Board rendering — the rules that were learned by breaking them

- **Never cache what core can be asked.** The held component is looked up by id
  every frame, not copied at pick-up. The copy went stale the moment the player
  turned it: core rotated correctly and the board kept drawing the old shape.
- **The drag footprint is painted last, over the pieces.** It used to go onto
  the empty grid before anything was drawn on it, so every occupied cell covered
  it — and occupied cells are exactly where a drop fails and an answer is
  wanted.
- **The ghost on the cursor is translucent and offset.** At 92% alpha sitting
  square on the target it hid the green-or-red answer at the moment it was
  being asked.
- **The canvas sizes its own backing store to its box.** A fixed intrinsic width
  is a fixed width *scaled by CSS*: 1240 displayed at 800 turned every 34px cell
  into 22px and left a third of a screen empty underneath.
- **Text belongs in HTML, not on the canvas.** The item list was 11px canvas
  text crammed under each grid, where a second item overlapped and a third was
  cut off.
- **The replay reads health, it does not compute it.** The log reports
  `target_health` on a hit and `health` on a burn or a regen. Subtracting
  `damage` from a running total ignores `absorbed`, so armour soaked a blow, the
  bar dropped anyway, and both sides could sit at zero for the rest of a fight
  that was still going. `fight_json` carries a snapshot per entry.
- **An item card has two halves and which stat goes in which is not a
  presentation choice.** *Standing still* is what the item contributes whether
  or not a fight is happening — health, strength, power, regen, resists, pierce,
  harden. *Every activation* is what one tick does — damage, cork, the Funny,
  fury, devotion, harvest, plus any unconditional pool gain folded in from a
  trigger. Cork resets every fight; listing it beside max health told the player
  they were wearing armour they were not. `testing/drive.py` checks the split.
- **Do not reuse a class name.** `.card` is the event dialog — `position: fixed`,
  `inset: 0`, `z-index: 10`. The item cards were given the same class and every
  one of them became a full-viewport overlay pinned over the game. Found by
  measuring `elementFromPoint`, not by reading.

## A component is a shape

Everywhere a component appears it now shows the shape it takes up and the kind
of thing it is, and explains itself on hover.

- **Two blades at one price are not the same purchase** when one is four cells
  in a line and the other is a cross. The shelf gave a name, a slot and a
  price, which is everything about a component except the thing you are buying.
- The bag under the board drew a **one-cell swatch for everything**, so a ring
  and a twelve-cell base looked identical — hiding the only property of a loose
  component that decides where it can go.
- `explain::piece_lines` is what a hover reads. It uses `Action::describe` and
  `Trigger::describe`, **which already existed in `piece.rs`** — the first
  draft of `explain.rs` wrote both again, which is the "engine owns the
  sentence" principle failed from the other direction. Check before writing a
  describer.
- `every_component_says_something_about_itself` covers the catalogue. It skips
  quest tokens (a tally does nothing on purpose) and the six `EVENT_ONLY`
  relics — **whose value lived in `relic.rs`, deleted with the campaign.** They
  are on no shelf and no surviving event grants one, so they are unreachable
  content rather than a lint to satisfy with invented stats.
- **Two answers on one hover, and neither replaces the other.** The panel card
  is about the *item*, because pointing at a blade is asking about the weapon;
  the hover card is about the *component*, because that is what you are about
  to pick up. `board.onpoint` and `board.onpiece` are both reported.
- `shape.js` and `Board.thumb` both draw through `paintMotif`. The mark is the
  shape half of the colourblind triple-encoding, so everything that draws a
  cell draws the same one — at 34px on the board, 11px in the bag, 14px on a
  shelf.

## Art

- **TikZ or nothing.** Every figure in `art/` is a standalone document written
  by filling in `tikz_figure_prompt.md`, and the reason is not ceremony: a
  figure that is text can be reviewed, diffed and corrected in one line, and a
  figure that is a PNG can only be re-rolled and hoped over.
- `make art` compiles to `web/assets/*.svg`. **The SVGs are checked in**, so a
  deploy never needs LaTeX; missing tooling prints what to install and exits 0.
  `standalone.cls` is not in BasicTeX and is the usual reason it fails —
  `tlmgr init-usertree && tlmgr --usermode install standalone`.
- **The house style, which is the prompt's "audience" field:** flat fills, heavy
  outlines, no gradients; a figure must read at 64px on the map and again at 4×
  in a panel.
- `data/art.json` maps a canonical creature name or place id to a figure. A
  subject with no entry draws nothing.
- **The creature half of that file is generated — do not hand-edit it.**
  `art/creatures.json` says which family drawing each creature is cut from and
  in what colours; `make art` compiles a figure per creature and rewrites
  `data/art.json` from it. Deriving the map from the manifest is the point: the
  map and the files it names cannot drift, because only one of them is written
  by a person.
- **Families, not fifty drawings.** Thirteen silhouettes — sentinel, bone,
  wisp, hound, idol, mirror, clergy, crown, court, wright, ash, rime, vermin,
  plus the four drawn for themselves — each compiled once per creature with
  `\def\Main{...}\def\Dark{...}\def\Accent{...}` on the pdflatex command
  line, against a `\providecommand` default inside the figure. Two creatures in
  a family share a silhouette and never a palette.
- **`.tex` count ≠ `.svg` count, and that is fine.** A creature whose slug
  equals its family name (Francis) compiles twice to the same file. The check
  that matters is `every_creature_has_a_figure_and_every_figure_has_a_file`.
- **Draw it, then look at it.** Three of the thirteen compiled cleanly and did
  not read: `bone`'s ribs came out as a spring, `clergy` collapsed into a single
  triangle because the mitre sat straight on the robe, `ash` was a stack of
  circles. A figure that compiles is not a figure that works — rasterise the
  set and put your eyes on it.

## The art was drawn and shown nowhere

Reported by the human as *"the png representation of them that we built;
nowhere ever shows it"*, and they were exactly right.

`data/art.json` shipped mapping **three creatures out of fifty**. So a portrait
appeared on the fight screen roughly one time in twenty, and `art.player` —
`sprocketman.svg`, compiled and deployed since M6 — was read by no line of code
at all. Nothing was broken; the map was just almost empty, and an empty map is
indistinguishable from a feature that does not exist.

Two things came out of it, and the second is the one that matters:

1. Every creature has a figure now, and the player's own is in the panel that
   is always up.
2. **Coverage is a test.** `every_creature_has_a_figure_and_every_figure_has_a_file`
   fails when a creature is added without art, when the map names a file that
   is not there, and when the map names a creature that is not in the ladder.
   `check_the_portrait_shows` says the same thing from the browser, including
   `naturalWidth != 0` — a portrait that 404s is not a portrait.

## Your figure is your class's

`art.player` is the Sprocketman — who you are before anybody has decided what
you are. The fork is where that stops being true and it does not come off, so
the panel draws `art.classes[canonical]` from then on. Repainted on every
`paintPanel`, so a loaded save arrives wearing its own figure rather than
waiting for the next fork.

## Screens, and the three times one covered another

Three bugs, one shape, and **not one of them was visible by reading the
source**. All three were found with `document.elementFromPoint`.

1. **`.card` is the event dialog** — `position: fixed`, `inset: 0`. The item
   cards were given the same class and every one of them became a
   full-viewport overlay pinned over the game.
2. **`.screen.framed` and `.screen[hidden]` have equal specificity**, so the
   later rule won and a *hidden* fight screen stayed laid out over the whole
   page. The town's Spend it button was visible, enabled, and swallowed by
   `#run` from a screen nobody could see. Anything that sets `display` on an
   element it also hides needs its own `[hidden] { display: none }`.
3. **Every `.screen` sat at `z-index: 20`**, so which covered which was decided
   by the order of the file — and the town comes after the fork. A level lands
   when you bank, banking happens with the town up, so the class fork opened
   *underneath* it: four cards on screen, none of them clickable, and the game
   unfinishable from level five. Taking a class then opens the tree, which was
   under the town for the same reason.

The stack is written down now, and each tier is a sentence about what a screen
is:

| z-index | what it is | which |
|---|---|---|
| 20 | where you are | the fight, the town, the map's card |
| 30 | what you opened from there | the tree, the log, the ending |
| 40 | the one that does not come off | the fork |

**A screen you cannot dismiss must be the top-most thing on the page.**
`check_the_fork_is_on_top` is what stops a fourth.

And one about the harness rather than the game: **a check that opens a screen
has to close it on every path out.** A check that appended a failure and
returned early left the screen up, the next check died on a click it could not
land, and the whole failure list went unprinted — so the run reported a
Playwright traceback and not the one sentence that said what was wrong.
`walk_the_gate` takes its `fails` list from the caller now, so a crash cannot
take the findings with it.

---

# Part six — shipping it

## A deployed fix is not a delivered fix

Pages serves `index.html` with `Cache-Control: max-age=600` and everything else
is content-hashed, so a browser holding a stale entry point keeps loading the
**old** `app.js` and the **old** wasm from URLs that are served forever. The
position-repair fix was live, verified against the deployed site, and still had
not reached a player whose tab was pinned to the previous `index.html`.

`app.js` carries the build stamp it was packaged with, fetches `index.html` once
with a cache-busting query, and if the stamps differ navigates to
`?v=<live>` — a **different URL**, not `location.reload()`, which is allowed to
re-serve the same cached document and would loop. `sessionStorage` guards
against a genuine mismatch looping anyway.

`packaging/package-web.sh` fails the build if the stamp is not applied.

**The stamp hashes everything the browser caches**, and two holes have been
found in that line by hand rather than by a test:

1. The modules were listed by name, so `shape.js` was left out when it was
   added — and its own import of `board.js` with it.
2. `index.html` and `styles.css` were left out, so a markup- or CSS-only change
   produced an **identical stamp**. `styles.css?v=…` kept its old URL and was
   served from cache for ever, and the entry point's self-heal never fired
   because the two stamps matched. A CSS fix could ship and never reach anybody
   who had already loaded the page.

The hash is taken **before** stamping, which is what makes it stable — the
`?v=` values and `__BUILD__` are written into those files afterwards.

## Tone, as a lint

`tests/tone.rs` holds the eight rules from `TONE.md` a machine can check. Not
the ones about register — those need a reader — but the ones that are facts
about a string. Every one caught something on its first run:

- **Rule 13** found a blurb saying "armour" twice where the game says Cork.
- **The blurb/effect check** found a node promising a row on two frames and
  granting one. A blurb that overstates its effect is the worst kind: the
  player finds out by not getting it.
- **Rule 12** was itself wrong first, and failed two lines that were perfectly
  clear — "Forty Fnorp" names forty, and spelling small numbers out is the
  house style. The lint learned to read numbers as prose.

## Divergences from the brief

`PLAN.md` wins. These are the places it does, and why.

| # | Divergence | Where |
|---|---|---|
| 1.1 | Fork by copy with a provenance file, not a git subtree. And the campaign is dropped, not carried: eleven modules deleted, `Run` replaced by `Character`. | `PLAN.md` 1.1, `crates/core/UPSTREAM` |
| 1.7 | §C.1 is a **design change, not a bug fix**. Upstream paid the bounty on a loss deliberately and its reasoning was sound *on a ladder*. GM2D is not a corridor, so the justification goes and the exploit stays. | `crates/core/src/reward.rs` |
| 1.9 | The theme becomes data. `theme.rs` already treats a name as a key rather than a label; moving its tables to `data/` is where they belong. | `PLAN.md` 1.9 |
| 1.10 | Actions builds and publishes to Pages. No `docs/`, no human-run `make publish` rebuild. The brief described gear-master, which predates both house web repos and ships macroquad. | `.github/workflows/deploy.yml` |

Also true, and not in the brief because it could not have been:

- **§C.1's code was gone before the fix was written.** The bounty was paid in
  `Run::settle`, which left with the campaign. The rule now lives in
  `reward.rs` and M3's encounter resolution calls it.
- **§C.3 is not a code fix.** It was a fault in a CLI GM2D does not ship. It
  survives as a UI rule: **the shop screen displays the price actually
  charged, never `registry.def(id).price`.** It becomes a test in M3.

## Deleted, and where to find it

Eleven modules and 57 test files went in `48203ee`. Everything is in the
history at `78e40eb` if a question about the old behaviour needs answering.

Dropped modules: `county`, `dungeon`, `route`, `quest`, `relic`, `pedestal`,
`rumour`, `bestiary`, `town`, `share`, `run`.

Dropped tests, all of them testing something GM2D no longer does — fountains
and axis thresholds, the road, receipts and choices, share codes — or leaning
on a helper that does: `packing`, `validity`, `classes`, `casino`, `chain`,
`francis`, `phase_two`, `prose`, `structures`, `two_voices`, `insight`,
`tallies`, `vip`, `completable`, `two_runs`, `taller_boards`, `sudden_death`,
`fight`, `reference_builds`, and the campaign half of `tooltips`.

Four classes went with the dungeons that were their only source: `Ascendant`,
`Threshold-Sighted`, `Prospector`, `Wumpus Hunter`. Their `ClassPower`s survive
and M5's trees may spend them again.

`CLASS_ORDER` and its append-only test were the share-code wire format.
`share.rs` is gone, so the constraint is gone.

## Numbers, so a regression is visible

Every figure below was re-measured for M8.8 rather than carried forward.

| | |
|---|---|
| Upstream suite, pristine fork | 1075 passing |
| After the campaign was cut | 128 passing |
| After the simulation tests were ported to `Character` | 329 passing |
| M1 | 346 passing |
| M2 | 359 passing |
| M3 | 369 passing |
| M4 | 382 passing |
| M5 / MVP | 391 passing |
| Board rebuilt against the original | 411 passing |
| The other side's gear, and a tree that says what it does | 419 passing |
| Shops, errands and a replay of both sides | 425 passing |
| Components that show their shape and explain themselves | 427 passing |
| The sheet, and a fight that opens holding what it holds | 429 passing |
| A tree drawn as a tree | 431 passing |
| Souls experience, and one town on the map | 432 passing |
| Fatigue, errands, and the first dungeon | 447 passing |
| M8.0–M8.1: a door you can go back to, and a log that points | 453 passing |
| M8.2: curses on the card, in the replay, and a bar that stops moving | 456 passing |
| M8.3: skills that grant rules | 459 passing |
| M8.4: enchs, and the class that grants them | 469 passing |
| M8.5: the spin | 474 passing |
| M8.6: the Kaklon Patent | 477 passing |
| M8.7: the door in the wall | 482 passing |
| **M8.8: played, triaged, written down** | **483 passing** |

| | |
|---|---|
| Catalogue | 536 components, unchanged since M7 — the save fingerprint has not moved |
| Pieces that apply a curse | 59 of 536, 4 kinds, 2 on the starting shelf |
| Ladder | 50 creatures, rated 16 to 2958 |
| `crates/core` | ~38.6k lines, up from 33k at M7, down from ~50k at the fork |
| wasm | 1116 KB, up from 888 KB at M7 |
| Save format | v1. Every M8 field defaults, so a pre-M8 file still opens |
| Maps | 2 — overworld 20×20 (1 town, 7 events, 1 gate, 1 door) + the cave 9×5 (1 boss, 1 gate) |
| `PlaceKind` | 5: town, event, gate, boss, **door** |
| Effect kinds | 5: stat, start_with, grow_slot_rows, assembly_pct, **grants** |
| `Rule` kinds | 5: curse_on_activate, spin_extra, spin_keep, spin_every, scout |
| Starting kit | **2 components**, 28 Fnorp, 1 assembled weapon |
| Towns | 1 placed, 2 staged; fixed shelves of 11 / 15 / 17, no reroll; every one keeps a bench |
| Errands | 10: 5 at the pit, 1 roadside, 2 of Marbulon's, 2 staged |
| Enchs | 5 — 4 on every bench, 1 off an errand and on no shelf |
| Restoratives | 3, at 4 / 11 / 28 Fnorp — retuned down when a town started mending |
| Boards | 6×3 at level 1, one row a level, 6×8 ceiling |
| Level 5 | ~27 fights, mean of nine seeded walks |
| Skill trees | 13 base nodes + gorillathon 8, funnel-sergeant 8, worm-fact-keeper 10, kaklon-patent 8 |
| Classes offered | 4 |
| Figures | 26 `.tex` → 72 SVGs (13 family drawings, 4 drawn for themselves, 4 classes, 3 towns, you) |
| Art coverage | 50 of 50 creatures, 3 of 3 towns, **4 of 4 classes**, and you |
| Browser gate | 37 checks, 3 engines |

Note the catalogue is **536**, not the 374 the retheme document counts — it
grew upstream after that document was written. Any content work that quotes a
catalogue size should quote this one.

## Open questions the human has not answered

Listed in `PLAN.md` §6. None block M1.

**Answered:** the repo is `sgilson7/gear-master-2d`, public, Pages served from
Actions.

**Still open**, with the default in force: losing costs nothing but the walk
back; the content charter is binding; invented proper nouns fail the M2 lint.

**No longer open:** errands exist, as `crates/core/src/quest.rs` — a new module
rather than upstream's, which was a chain of receipts along a road. `town.rs`
stays dropped: a town is a place on the map plus a shelf in `shops.json`, and
does not need a module.
