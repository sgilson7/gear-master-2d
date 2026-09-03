# PLAN-M11.md — past the door

Written 2026-08-31, against the live page and a fresh clone of `main`.
`PLAN.md` remains the plan of record for M0–M7, `PLAN-M8.md` for M8,
`PLAN-M9.md` for M9; M10 is being written elsewhere and this block does not
depend on anything in it. This is the plan for what is on the other side of the
western door.

**Ten milestones, each behind the standing gate:** `make test`,
`make test-ui` in three browsers, `make play` read by a person. Six of them
are also **deploy points** (§5). Two documents are born in this block and live
past it: `SECOND-ORDER-M11.md` (§6) and `PLAYTEST-M11.md` (§7).

---

## 0. The ask

> A DQ-style overworld behind the door, with overworld battles controlled by
> region, and two maps you can walk into. One map contains the next town and is
> designed in the Look Outside dense style, with a massive cheese tower in the
> middle: every time you clear a floor, the tower drops a level and you are
> kicked outside, and the next entry is different. When the tower has fully
> dropped after 5 levels, the lake in the center of the first map (The End of
> All Gears) drains and opens a new dungeon with a boss. With the toad gear you
> can travel into the water early — entering the middle of the lake in toad
> gear drops you into the cave to fight the boss early. The second town map
> incorporates quest lines. The other map is survey-based: a new item type,
> **map shards**, combined on a gear board using recipes in the weapon style —
> compass (map shard + glass lens + magnet → basic survey, augmented by gear),
> atlas (2 map shards + glass lens + cosmic orb + cosmic alignment), summon
> survey golem (3 map shards + 2 living earth). Each survey item gives its own
> unique bonuses to the surveyed map. One surveyable map for now, but the
> architecture must scale so mapping can affect them. Surveyable maps are
> **static, not rolled per survey** (unlike PoE2), so they can carry
> questlines.

And, added after the first draft of this plan:

> New enemies for the new maps, authored with the monster tools, with set gear
> on three more first-map creatures, three tower creatures, and one overworld
> creature whose set bonus teleports you back to town at the cost of a
> restorative. And the text for things like being stopped at a barrier or
> quest movement currently prints below the save area; it should have its own
> game-output log, with a button to see the history.

## 1. What recon found, and what it changed

Measured before writing, in the order it mattered.

**The multi-map plumbing already exists.** `WorldState.map` is a saved field
with a backward-compatible empty default; the Great Gear Cave is already a
second map; `World::load` takes any terrain/tiles pair; `world::overworld()`
returns `"west-bambulon"` and is the only place that name is decided. Adding
maps is authoring, not architecture. What does **not** exist is a map at a
different scale, or a transition that remembers where you stood on the map you
left — `at` is one pair, not one per map. M11.1 fixes that in the save, which
is a new field, which is a compile error until `save.rs` carries it. Good.

**The output problem is the shared-slot problem, and the original already
solved it.** Refusals, quest movement and pickups render below the save area
today — a slot that exists because nothing owns it, which is the exact class
of thing `SECOND-ORDER` notebooks exist for (M8's invisible feature shared a
message slot nobody had thought about). The original gear-master ships the
answer: a quiet three-line strip during play and a FULL LOG overlay for the
whole transcript. M11.0 ports that pattern rather than inventing one, and it
goes **first**, because every milestone after it writes into it.

**The lake and the toad gear are real names.** The lake sits in the region
`The End of All Gears`; the Bog Toad already drops `Toad Frame` and `Toad Hide`,
and the assembled Toad set already grants the rule *the lake's rim becomes
ground*. So the early-access path is not a new system — it is one more arm of a
rule two systems already read (`skills.rs` and `loadout::set_of` both grant
`Rule`s, `Allowances::of` matches them exhaustively). The design question is
whether the Toad set's rule *widens* (rim → the whole lake) or a deeper-water
rule joins it. §8 row 1; this plan assumes widening, because a set a player
ground for should get better when the world gets bigger, and the second-order
entry for it is mandatory (§6).

**Two towns are written and on no map.** Kettleworks and High Wick have
shelves and errands since M8. The dense town map lands one of them (§8 row 2
picks which) and inherits its content for nothing — this is `PLAN.md` §6a
row 1 finally paid.

**The survey components are partly unknown.** The catalogue has 536 pieces
under canonical names. Whether `Glass Lens`, `Magnet`, `Cosmic Orb`,
`Cosmic Alignment`, `Living Earth` exist, half-exist under other names, or are
new is a grep, and M11.5 does the grep before adding anything — *before adding
a system, grep for it* is the standing rule, and it goes for pieces too.
**Every genuinely new piece moves the catalogue fingerprint and invalidates
old saves, by design.** So the block's save seams are exactly two, both named
in §5, and no other milestone touches the catalogue.

**The monster tools exist and come over.** The original's GUI has `make pack`
(*dress creatures by hand — the game, editing somebody else's board*) and its
`crates/lab` has a `dress` binary that searches the catalogue for a loadout
hitting a target rating. Both of the original's guarantees hold here already —
**monsters wear the catalogue** and the assembles-or-fails test — and M11.9
extends them over everything it adds.

**Floodline is the playtest pattern.** `packaging/browser/` there has the
three parts M11.8 ports: an `AGENT-BRIEF.md` that tells the playing agent
everything a player could know and forbids it the source ("the run measures
your reading of `balance.rs` and not the game"); a `driver.py` giving hands
and eyes one process per turn; and a `PLAYTEST-M*.md` that writes down what the
run found in the players' own words. GM2D's `testing/drive.py` is already most
of the driver.

**Floodline also has the notebook.** `SECOND-ORDER-M12.md` exists there
because a reviewed feature shipped invisible — the feature was reviewed, the
message slot it shared was not. §6 adopts the format whole.

## 2. The shape of the block

```
M11.0  the output log           deploy point A
M11.1  the overworld            deploy point B
M11.2  the town map, dense
M11.3  the cheese tower         deploy point C
M11.4  the lake drains
M11.5  map shards + instruments (the first seam)
M11.6  the surveyable map       deploy point D
M11.7  bug and triage pass
M11.8  the subagent playtest    deploy point E
M11.9  the bestiary             deploy point F = the block ships
SECOND-ORDER-M11.md runs the whole length (§6)
```

Order is load-bearing three times. M11.0 goes first because every milestone
after it emits text, and text moved once — before content is authored against
the wrong slot — beats text moved at the end and re-verified everywhere.
M11.4 needs M11.3's `tower_dropped` flag, and M11.6 needs M11.5's
instruments. M11.9 sits after the playtest on purpose: the run plays the
*systems* against creatures borrowed and re-dressed from the shipped roster,
and the bestiary then replaces the borrowings — new pieces move the
fingerprint, so the block's second and final save seam is its last act, not
its middle (§5).

---

## 3. The milestones

### M11.0 — the output log

**Goal.** One place the game talks. Refusals, quest movement, pickups and
everything else that prints below the save area moves to its own log, with a
button for the history.

**Deliverables.**
- A log panel in `app.js`: the last three or four lines, always visible,
  in a region that is the log's own — new id, new class, because trap 6 is
  two reused names old and both cost a day. A **HISTORY** button opens the
  session transcript as a scrollable overlay; `Escape` closes it. This is the
  original's quiet-strip-plus-FULL-LOG, re-cut for GM2D's screen.
- **One emit path.** A single `log()` in `app.js` that every message goes
  through — crossing refusals, quest taken and handed in, a drop, a tin
  drunk, a level banked, the save written. A message that can miss the log is
  the shared-slot bug waiting to happen again; one function owns the slot.
- The below-save slot is **removed, not shadowed** — the elements go, so a
  stray writer fails loudly in `make test-ui` instead of printing somewhere
  nobody looks.
- Core sends what it sends today; the log is presentation (rule 5 — the page
  draws what core sent it, and now it draws it in one place). Entries carry
  no clock, just order; the transcript reads top to bottom like the fight
  replay's does.
- **History is the session's, not the save's.** A save carries the world, not
  what the screen said about it; a transcript in the save file would be a
  seam and a diary. §8 row 10 confirms.
- The planted browser checks that read the old slot retargeted; a new check
  plants a refusal and finds it twice — in the strip, and in the history.
  Broken first, watched failing, then trusted, per the standing habit.

**Acceptance.** No message renders below the save area, and the old elements
are gone; a crossing refusal, a quest hand-in and a drop each land in the
strip and the history in all three browsers; the HISTORY overlay opens, holds
the whole session, and closes; `make play`'s transcript reads from the log
region.

### M11.1 — the overworld

**Goal.** The western door opens onto a DQ-scale map where West Bambulon is
one tile, and walking it can mean fighting.

**Deliverables.**
- `data/maps/overworld.tiles.json` (new per-map layout: `data/maps/<id>.tiles.json`;
  the existing `tiles.json` moves there with a shim so nothing else changes).
  A coarse map — recommend 16×16 — with its own terrain vocabulary (plains,
  range, coast, the lake seen from outside) and 3 enterable places: the door
  back into West Bambulon, the town map (M11.2), the survey reach (M11.6,
  present but refusing entry until an instrument exists, with a sentence —
  which now lands in the log).
- Overworld regions with their own pools; `encounter_per_mille` and
  `draw_enemy` already read per-region, so overworld battles being
  "controlled by region" is content, not code. 2–3 new overworld creatures
  drawn from the unthemed catalogue, themed per `TONE.md`, rated to bracket
  levels 5–9.
- Per-map position: `WorldState` grows `positions: Vec<(String, [u8;2])>` so
  stepping through the door returns you to the tile you left, both ways.
  Exhaustive destructure in `save.rs` extended; old saves (which have never
  been past the door) default to each map's `start`.
- The door event rewritten to actually cross, in `TONE.md` voice: the
  anthology has been telling you Bambulon was the world; the overworld is
  where it stops pretending.

**Acceptance.** A seeded walk across the overworld replays; the door
round-trips position; a save written on the overworld reopens there;
`make play` extended to step through the door and back.

### M11.2 — the town map, dense

**Goal.** One enterable 20×20 in the Look Outside register: small, thick with
things that answer, nothing decorative.

**Deliverables.**
- `data/maps/towerfield.tiles.json` (working id; the theme names it). One of
  the shelved towns (§8 row 2) placed with its shelf and errands live.
- Density budget, enforced by test: **≥ 40 of 400 tiles** carry a place, a
  once-event, or an examinable — an `Event` with no choice, one paragraph, in
  voice. Look Outside's lesson is that a map this size carries a game when
  everything answers; the budget is the checkable version.
- The cheese tower's footprint in the middle — visible, entrance refusing
  until M11.3 with a sentence that promises it.
- **Two quest lines** in `data/quests.json` using the existing three goal
  kinds, each 3–4 errands long with at least one step that crosses maps
  (town map ↔ overworld ↔ West Bambulon), because quests that cross maps are
  what make the world one place. One line touches the tower (a giver who wants
  floors down), so M11.3 has a witness.

**Acceptance.** Density test green; every quest step completable in
`make play`'s extended walk; every event id on the map exists; both shelved
towns' data still parses (the unplaced one stays shelved, not deleted).

### M11.3 — the cheese tower

**Goal.** Five floors. Clear one, the tower drops a level, you are put out,
and the next entry is a different tower. Fully dropped, it flags the lake.

**Deliverables.**
- Five floor maps, `data/maps/cheese-tower-5.tiles.json` down to `-1` — small
  (recommend 10×10), each with a floor boss (existing creatures re-rated or
  new, themed; the tower is *cheese* and the tone writes toward that with a
  straight face). Floor N is reachable only while `tower_floors_cleared`
  (a `WorldState` counter — the generic mechanism, no new field) equals 5−N.
- The drop: beating a floor boss bumps the counter, plays a one-paragraph
  event (the tower settles; the anthology counts the remaining floors and
  reports the count), and **moves the player to the tower's outside tile** —
  the kick is a position write, not a death.
- The entrance resolves which floor map you enter off the counter — so "the
  next time you enter is different" is literal: it is a different map. Floors
  already cleared do not re-open; the tower is shorter, not deeper.
- At 5, flag `tower_dropped` set; the entrance becomes a stump with an
  examinable; the M11.2 quest line's tower step completes.
- Tower floors are the primary **map shard** faucet (the drop is authored in
  M11.5's data but the hook lands here as a no-op drop table entry).

**Acceptance.** Counter drives which map the entrance opens, tested at all
six values; save mid-tower reopens outside (a floor is one sitting — §8
row 3 confirms); kick never lands the player in scenery (`World::repair`
covers the outside tile); golden fight fixtures for the five bosses.

### M11.4 — the lake drains

**Goal.** `tower_dropped` empties The End of All Gears; a dungeon and a boss
are under it; the Toad set gets you there early through the lake's middle.

**Deliverables.**
- West Bambulon's lake tiles become drained-bed terrain when `tower_dropped`
  — terrain resolution consults the flag the way `place_is_there` already
  consults state (small `World` change, one function, tested).
- `data/maps/under-lake.tiles.json`: the dungeon, entered from the drained
  bed. One boss, new, rated above the tower's fifth floor; drops feed the
  survey economy (a shard or an instrument piece) and a set piece per M9's
  conventions.
- **The early way.** The Toad set's rule widens: the lake's rim *and body*
  become ground while the set is whole. The lake's centre tile is a trapdoor
  place: step on it and a one-choice event drops you into `under-lake` —
  which, entered wet, is the *undrained* variant: same map, `flooded` flag,
  a harder frame for the same boss (recommend: the boss keeps its loadout,
  the arena's water rows are impassable, so the fight is positioned worse —
  content, not a second boss). Beating it early sets the same flags a drained
  kill would, minus `tower_dropped`; the tower still stands and still wants
  dropping. Second-order entry mandatory: widening a shipped set rule moves
  every place water gates anything (§6).
- Killing the boss either way opens whatever M12 wants behind it; this plan
  writes the ending event and assumes nothing further, per the standing rule.

**Acceptance.** Flag flips terrain and the walker sees it; toad-set entry
reaches the flooded variant, drained entry the dry one, from the same files;
un-toaded water still refuses; both boss fights have golden fixtures; a save
written inside `under-lake` reopens there.

### M11.5 — map shards and the instruments *(the first seam)*

**Goal.** The new item type and the three recipes, on a gear board, in the
weapon style.

**Deliverables.**
- Recon first, in the commit message: grep the catalogue for `Glass Lens`,
  `Magnet`, `Cosmic Orb`, `Cosmic Alignment`, `Living Earth`; reuse what
  exists, add what does not. **All of this milestone's new pieces land
  together** — one fingerprint move, called out on the page the way the
  catalogue seam always is.
- `Map Shard`: a new component category (`survey` core, the way `handle` is
  the weapon's), dropped by tower floors (hook from M11.3 goes live),
  the under-lake boss, and one quest reward. Interpretation on record:
  *"using three weapons like the recipe"* is read as **three recipes in the
  weapon-recipe style** — a core piece plus named supporters, assembled on
  the weapon board, mutually exclusive with a weapon in that grid the way
  book and blade already exclude each other. §8 row 4 confirms.
- The three recipes, as data in `piece.rs` + recipe tables:
  - **Compass** — 1 map shard + 1 glass lens + 1 magnet.
  - **Atlas** — 2 map shards + 1 glass lens + 1 cosmic orb + 1 cosmic
    alignment.
  - **Survey Golem** — 3 map shards + 2 living earth.
- Assembling one grants a `Rule` (`Rule::Survey(kind)`) — the third granter
  of rules after the tree and the sets, through the same exhaustive enum.
  What each kind *does* is M11.6's; here they exist, name themselves on the
  item card, and are saved.
- `explain.rs` and the item card learn the new category (M8's lesson: a
  system nobody is told about is a bug report).

**Acceptance.** Old saves refuse with the fingerprint sentence (by design,
and the deploy note says so); each recipe assembles and names itself;
a board holding an instrument refuses a weapon core and says why, in the log;
catalogue shape tests extended.

### M11.6 — the surveyable map

**Goal.** One static map that only an instrument opens, and that the
instrument *changes*.

**Deliverables.**
- `data/maps/the-reach.tiles.json`: static, authored, 20×20, with its own
  region pools, a short quest line (giver on the town map, so surveying is
  somebody's errand), and set-bearing drops. **Static is the point**: the same
  tiles every survey, so quests can name places on it.
- The survey act: at the overworld's survey reach, with an instrument
  assembled, an event opens the map and records `active_survey: (map, kind)`
  in `WorldState`. The instrument is **not consumed** (§8 row 5); leaving the
  reach closes the survey; re-entering re-reads whichever instrument is on
  the board now.
- **Survey modifiers**, the scalable part: a `SurveyMod` set computed from
  the active instrument and applied at `World::load` time — the map is
  static, the *lens* varies:
  - **Compass** — the honest read: encounter per-mille −20%, and the mod
    scales off worn gear (*augmented by gear*: recommend + per assembled
    item, so a packed board surveys better — the game's thesis again).
  - **Atlas** — the cosmic read: drops +N per-mille and XP +M% on the reach,
    numbers set in data; the map fights back harder (encounter +10%) because
    an atlas is a promise and the reach heard it.
  - **Survey Golem** — the accompanied read: a golem walks the survey. In
    every fight on the reach the golem stands as **one extra pre-assembled,
    read-only item on your side** — an ally row in the replay, reusing
    `theirs.js`'s read-only board. Riskiest deliverable in the block;
    fallback on record: if the ally row fights the replay layout, the golem
    instead takes the first encounter of each entry entirely (it "handles
    one") and the ally row moves to M12.
- Architecture note, enforced by a test: `SurveyMod` application is a pure
  function of `(map, kind, character)` — no survey state leaks into the map
  files, so a second surveyable map is a data drop.

**Acceptance.** The reach refuses without an instrument, in voice, in the
log; each of the three kinds measurably changes the same seeded walk
(encounter counts differ, drops differ, golem row present); quest line on the
reach completable; save inside a survey reopens inside it with the same mods.

### M11.7 — the bug and triage pass

**Goal.** The block's debt paid before anybody outside is asked to play it.

**Deliverables.**
- A sweep: `make test-ui` in all three browsers watching the console for
  anything above a log; the eight traps in `HANDOFF.md` §5 walked against the
  new code by hand (locks, ids, cache-busting, planted checks); every derived
  number added this block found *on a screen* (the M8 rule — four skills once
  worked perfectly and were reported broken because nothing printed them).
- `TRIAGE-M11.md`: every finding, one line each — severity (blocks the
  playtest / wrong but survivable / cosmetic) × cost. **Blockers fixed in
  this milestone; the rest carried into the playtest openly**, listed in the
  agent brief's "known" section so the playtest spends its attention on the
  unknown.
- A tone pass over every string the block added, `TONE.md` open.

**Acceptance.** Zero known blockers; the triage file exists and every line
has a disposition; `make play` reaches both endings (tower-dropped and
toad-early) from a fresh seed.

### M11.8 — the subagent playtest

**Goal.** Somebody who is not the builder plays the block, and what they hit
is written down. Floodline's pattern, ported.

**Deliverables.**
- `testing/AGENT-BRIEF-M11.md`: everything a player could know about the new
  content — the door, the overworld, the tower's promise, the shard recipes
  as a shop-poster would print them — and the two prohibitions, verbatim in
  spirit from floodline: **the agent does not read `crates/core`, the data
  files, the tests, or the probes**, and it plays the deployed build, not a
  local one, so it also checks delivery (*a deployed fix is not a delivered
  fix*).
- `testing/agent_driver.py`: `drive.py`'s machinery re-cut as hands and eyes,
  one CDP command per invocation, because an agent's turns are separate
  processes — `look` (screenshot), `panel`, `log` (the strip), `history`
  (the overlay), `key`, `click`, `save-file` (downloads and checks the save
  round-trips mid-run).
- The run: a fresh Claude agent, brief only, plays from a new game through
  the door, drops the tower, drains the lake, builds at least one instrument
  and surveys the reach. Budgeted at two sittings; the save file is the
  pause button, which is itself under test.
- `PLAYTEST-M11.md`: the findings in the run's own words, floodline-style —
  what was fun, what was invisible, what was wrong, each with the moment it
  happened. Findings feed `TRIAGE-M11.md` dispositions and the block's close.

**Acceptance.** The run completes or the reason it could not is itself the
headline finding; every brief instruction was executable as written; at least
one save/load happened mid-run and held.

### M11.9 — the bestiary *(the second seam)*

**Goal.** The new maps stop borrowing. New creatures authored for the town
map, the tower and the reach; seven of the world's creatures get sets; every
new face gets a picture.

**Deliverables.**
- **The authoring tools first.** Port the original's pair per §1: the GUI's
  `make pack` hand-dressing mode, and the lab `dress` search as
  `make dress RATING=n` emitting a `MonsterSpec`. Keep both guarantees:
  **monsters wear the catalogue** (a creature is a loadout, never a stat
  block) and the assembles-or-fails test, extended over every spec this
  milestone adds — a typo would silently ship a harmless monster, and that
  test is why it never has.
- **New creatures**, replacing M11.1/M11.3/M11.6's re-dressed stand-ins where
  a stand-in was used: 2–3 ambient for the town map's outskirts, distinct
  bosses for tower floors that shipped on borrowed frames, 2–3 for the
  reach's pools. Each dressed by tool, rated to its region's bracket, themed
  with `TONE.md` open, canonical name + themed name per the M9 convention.
- **Seven sets**, each per M9's rules — one grid's whole recipe, agreement
  and completeness through `loadout::set_of`, granting a `Rule` no stat could
  express, dropped in per-mille rates bounded by
  `a_set_is_a_few_hours_and_not_a_lifetime`:
  - **Three on West Bambulon creatures that have none** — the pit's three are
    already paid (rat, toad, wallspider), so these come from the map's other
    regions; recon lists the roster and §8 row 7 picks. Spread across the
    level gradient so each gate has a set worth farming behind it.
  - **Three on cheese tower creatures** — recommend floors 1, 3 and 5, so
    the tower pays at the start, the middle and the top, and a set is a
    reason to *want* the next entry to be different.
  - **One on an overworld creature**, and its rule is the block's one piece
    of new travel: **assembled and whole, it returns you to your last town,
    and the trip costs one restorative** — the tin is consumed on departure,
    and with no tin in the bag the set refuses, in voice (the gear knows the
    way home; it does not know it sober). Cross-map by construction: it
    writes `map` and `at` through M11.1's positions, and it is the one
    legitimate way off a map that is not a step or a defeat.
- **The faces.** Every new creature gets `art/<name>.tex` written by filling
  in `tikz_figure_prompt.md` exactly — tunables up top, `\foreach` for
  repeats, the self-check at the end — compiled by `make art` to
  `web/assets/*.svg`; the geometric placeholder stands wherever a `.tex` has
  not compiled, declared in the deploy note rather than discovered.
- **The seam, called.** The set pieces are new components; the fingerprint
  moves a second time. Deploy point F's page note says so, the same sentence
  D used.
- A one-sitting agent spot-run against the deployed build — brief appendix,
  not a second full playtest: meet one new creature per map, farm one set far
  enough to see a piece drop, fire the homeward set once with a tin and once
  without. Findings appended to `PLAYTEST-M11.md`.

**Acceptance.** Every new spec assembles or the suite is red; each of the
seven sets grants a distinct rule that names itself on the item card; the
homeward set moves the player to `last_town` across maps, consumes exactly one
restorative, and refuses without one, in the log; drop-hours test bounds all
seven; golden fixtures for every new creature; `make art` output present or
placeholders listed in the deploy note.

---

## 5. Deploy points

Every milestone passes the standing gate; these six also push. The agent
does not `git push` or `make publish` without the human's word — the check
before each: `git log origin/main..HEAD`.

| point | after | a visitor can | note on the page |
|---|---|---|---|
| **A** | M11.0 | read everything the game says in one place, and its history | a pure improvement to the live page; nothing else changes |
| **B** | M11.1 | walk through the door, cross the overworld, fight by region, walk back | town and reach visibly refusing, in voice — a promise, not a hole |
| **C** | M11.3 | enter the town map, run its quests, drop the tower twice | the lake not yet draining is fine: the tower says five |
| **D** | M11.6 | the whole arc — tower, lake, boss, shards, all three surveys | **the first seam lands here**: the note says old saves are done, and why |
| **E** | M11.8 | the block, triaged and playtested | `PLAYTEST-M11.md` linked from the note |
| **F** | M11.9 | the bestiary — new faces, seven sets, the way home | **the second seam**: same sentence as D, and the spot-run's findings appended |

The seams are at D and F and nowhere else, on purpose: A, B and C change no
pieces, so every pre-door save keeps working through them, and the playtest
at E ships between the seams so its saves survive into the spot-run.

## 6. SECOND-ORDER-M11.md — the notebook

Created at M11.0, in floodline's format, kept for the same reason floodline
keeps theirs: a reviewed feature once shipped invisible because it shared a
slot nobody thought about. **Every milestone lands at least one entry; an
entry has three parts** — *the change*, *follows from* (what it necessarily
drags with it), *watch* (what would show it going wrong, and where). A change
claiming no second-order effects gets an entry saying so, because that is a
claim and a claim can be wrong.

Seeded at birth with the block's six known ones:

1. **One log slot** (M11.0) touches every planted browser check that read
   the old strip, and every future feature that says anything — which is the
   point. Watch: a message emitted by a path that bypasses `log()`, which is
   why the old elements are removed and not shadowed; the symptom would be a
   feature working perfectly and reported broken, which is M8's symptom.
2. **Per-map positions** (M11.1) touch loss-return (`last_town` is on
   another map now) and the repair-on-load path. Watch: a defeat on the
   overworld, and where it puts you.
3. **Widening the Toad set rule** (M11.4) touches every place water gates
   anything — the rim rule's original sites, `World::repair`, the walker.
   Watch: `make play` pathing through water it used to route around.
4. **A new component category** (M11.5) touches every exhaustive match on
   category — the board's refusals, `explain.rs`, the item card, drops.
   Watch: a shard seating where only a weapon core should.
5. **The ally row** (M11.6) touches the replay's two-board layout and rule 5
   (*the page draws numbers core sent it*) — a third board is a third set of
   numbers the page must not invent. Watch: absorbed damage on the golem.
6. **The homeward set** (M11.9) touches the travel economy the block
   otherwise leaves alone — the walk home is what makes a tin worth drinking
   and a town worth reaching, and a teleport priced in tins re-prices both.
   Watch: whether the spot-run and `make play` stop walking home at all, and
   whether tins get hoarded as fare rather than drunk as medicine — either is
   the rule's price set wrong, and the knob is the tin count, not the rule.

**Questions that come up at deploy points go in the notebook too**, dated,
under the point that raised them — the notebook is where E's playtest
findings get their *follows from* written before anybody fixes anything.

## 7. What this block does not do

No procedural surveys (static is the point, and the ask says so). No second
surveyable map (the architecture scales; the content waits). No chart items,
no coaches, no vehicles beyond the Toad widening and M11.9's homeward set —
travel economy stays fatigue and tins, which is exactly why the one teleport
is priced in a tin. Nothing past the under-lake boss: what its death opens is
M12's, and nothing in this block may assume an answer.

## 8. The human's calls, before the milestone that needs them

| # | needed by | question | recommendation on record |
|---|---|---|---|
| 1 | M11.4 | Widen the Toad set's rule to the whole lake, or add a second deep-water rule? | widen — a ground set improving with the world beats a second rule beside the first |
| 2 | M11.2 | Kettleworks or High Wick on the town map? | Kettleworks — its errands read more portable; High Wick keeps its shelf for M12 |
| 3 | M11.3 | Is a tower floor one sitting (save inside reopens outside)? | yes — floors are 10×10 and the kick is the loop |
| 4 | M11.5 | Instruments on the weapon board (excluding a weapon), or a sixth board? | weapon board — surveying costs your sword arm, which is a decision; a sixth board is UI the block does not need |
| 5 | M11.6 | Instruments consumed per survey, or durable? | durable — shards are the grind (three faucets), the instrument is the achievement |
| 6 | M11.6 | Golem ally-row fallback acceptable if the replay fights it? | yes, stated in advance so it is a decision and not a retreat |
| 7 | M11.9 | Which three West Bambulon creatures get sets? The pit's three are paid; recon lists the rest of the map's roster (the north holds Lord Drabley Henpeck and his bracket) | one per gated region, lowest first, so each crossing has a set behind it — Henpeck last, if at all, since a set behind the map's rarest fight is PLAN.md §6b's problem again |
| 8 | M11.9 | Which tower floors' bosses carry the three sets? | 1, 3 and 5 — first taste, mid-climb reason, top prize |
| 9 | M11.9 | May the homeward set fire from inside the tower or the under-lake? | yes from the tower (it is five entries by design and the kick already moves you), no from under the lake — a dungeon you can post yourself out of is not under a lake |
| 10 | M11.0 | Does the log history belong in the save file? | no — session-only; a save carries the world, not what the screen said about it, and a transcript in every save is a seam and a diary |
