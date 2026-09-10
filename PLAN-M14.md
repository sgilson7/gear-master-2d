# PLAN-M14 — Down twice, and the country under the country

*Two long dungeons with a puzzle on every floor, the Treyway's south, and the
map with the third town in it. Written against the M13 tree as deployed at
`ac2256c7`: `world.rs` (`PlaceDef`, `Floor`, `Drain`, `leave_the_sitting`),
`event.rs` (`Requirement`, `Outcome`), `survey.rs` (`mods_for` ignores the
map, which is what makes a second surveyable map a data drop), eleven maps in
`data/maps/`, and the door under the lake, which is where the writing stopped.*

---

## 0. The one-paragraph version

The Treyway gets a **south**: thirteen rows of shore below the coast, cut
off by two tiles of sea that go out when the tenth cairn is built on the
Reach — *the reach was wider on the way back*. On that shore is **the
Wextreen Sump**, the second surveyable place, and it goes **down**: four
floors, an instrument reads each, a puzzle on each, and the Ninth Surveyor at
the bottom, who did not give up. Behind the door under the lake is **the
Silt Stair**, cut for people carrying something: four floors down, a puzzle
on each, and at the bottom the thing Marbulon has been sitting with her back
to for eleven years. Both dungeons end at a door onto the same map. **The
Undercountry** is twenty by twenty, has one town on it, and the town is
empty, because what is in it is another plan. Marbulon's door in the
shallows opens onto it too, once both are down — *nothing is behind the
door* was true, and now it is not.

---

## 1. Decisions

### 1.1 Puzzles are monotone, because flags are

`WorldState.flags` and `answered` only ever grow. A puzzle whose wrong move
locks the right one is a puzzle the save cannot come back from, and a
one-shot wheel that floods the channel you needed is exactly that. So:

> **Every puzzle in this block is solved by discovering something, never by
> avoiding something.** A wrong move costs a fight, a walk, or a component —
> never the solution.

That is not a limit on the puzzles, it is what kind of puzzles this game
has. Its real puzzle is the board: `LooseItemOfSize`, `AlignedItems`,
`AssembledOfRarity` are locks a player opens by *packing*, and the survey
instruments are locks a player opens by *reading*. Eight floors, eight
locks, and every one is one of those two things.

### 1.2 Every puzzle is solvable blind, in bounded moves

`make play` walks the game with a walker that takes choices. A puzzle only a
person can solve is a puzzle the gate cannot walk, and a gate that stops at
floor two is a gate that never sees floor four.

> **Every floor has a blind solution in at most 45 event visits**, and a core
> test proves it by exhaustion (§7). Reading the floor with the right
> instrument is what makes it *short*, never what makes it *possible*.

Nine cairns visited in every order is 45; that is where the number comes
from, and it is the largest floor.

### 1.3 A stack that goes down, and is not one sitting

`PlaceDef::floors` already opens a door onto the first floor whose `cleared`
id is not in `answered`. Two things this block reads differently:

- **`cleared` is the floor's stair, not a boss.** A floor is done when its
  stair down has been answered, which is the puzzle solved. Only the bottom
  floor has a boss.
- **No `outside`.** `leave_the_sitting` only fires where a map names one, so
  a floor without it keeps you where you saved. Each floor has a stair back
  up, and the way back in lands on the first unsolved floor. A player who
  needs a 3×2 from the van can go and get it; what it costs is the walk,
  which is the currency this game's dungeons charge.

The Stack is one sitting because its budget is fatigue. These two are not,
because their budget is *what you brought*.

### 1.4 The south is found, not built

The Treyway's rows 15 onward are drawn in the same file at 16×26. **One
country, one file** — the note on the map already says West Bambulon is a
tile of it, and a second file would be two places to keep identical. Nothing
at rows 0–14 moves, so no `at_to` on any gate into the Treyway changes.

The two sea tiles at column 8, rows 15–16, are drawn as a new terrain,
**`tide`**: impassable, sea-coloured, and `drains` to `coast` when
`built-the-tenth` — the last flag on the Reach — is set. The tide goes out
on the day the tenth cairn goes up, and a player standing on the south coast
at row 14 has been able to see it the whole time.

### 1.5 The third town is a place with nothing in it, and it says so

`the-undercountry` ships with a `town` place, a name, a start tile, and a
region with enemies. The town screen shows a shop with no shelves and no
errands, and the town's prose says the writing stops here now — the same
sentence the door under the lake carried, moved one map down. **It is not a
placeholder drawn as a town; it is a town drawn honestly.** What goes in it
is PLAN-M15's.

### 1.6 Two ways down, three ways in

Both dungeon bottoms carry a door onto the Undercountry, and each is
`hidden_until_all` both bosses — so the first dungeon you finish shows a
sealed door with the other dungeon's name in the refusal, and the second one
you finish opens both. Marbulon's door in the shallows is the third, and it
is the one you will use: it is on the starting map.

---

## 2. What the engine has not got, and what it grows

Five additions, all small, and each read by both dungeons.

### 2.1 `Requirement::Surveying(&'static str)`

```rust
/// The instrument on the character's frame is this one.
///
/// Read off `Rule::Survey { kind }`, the same way a `needs_survey` gate is
/// answered in the shim — a map does not know about rules, and neither does
/// an event; the shim asks the character and hands the answer down. Unlike
/// `needs_survey` it names a *kind*, because a floor that any instrument
/// reads is a floor no instrument is for.
///
/// Never the only way through a door. §1.2: an instrument makes a floor
/// short, not possible.
Surveying(&'static str),
```

`Requirement::check` refuses a kind not in `INSTRUMENTS`, like `Rule::Survey`.

### 2.2 `Drain.tiles: Option<Vec<[u8; 2]>>`

```rust
/// Restrict the drain to these cells. Absent, it is the whole map, which
/// is what the lake needed and no floor with two channels does.
///
/// Cells are checked at parse time against the map's bounds and against
/// `from` — a drain naming a cell that is not the terrain it drains is a
/// drain that will fire and change nothing, which is the failure this
/// project keeps finding one milestone late.
#[serde(default)]
pub tiles: Option<Vec<[u8; 2]>>,
```

### 2.3 `PlaceDef::hidden_until_all: Vec<String>`

```rust
/// Not here until every id is in `answered` or `flags`.
///
/// `hidden_until` stays as the one-id case and is not deprecated: a door
/// behind one boss is most doors. The refusal, when some but not all are
/// met, names the ones that are not — "the Sump is down and the Stair is
/// not" — because a sealed door that says nothing is a bug report.
#[serde(default)]
pub hidden_until_all: Vec<String>,
```

`no_flag_is_waited_on_forever` extends to it.

### 2.4 Two terrains

| id | glyph | passable | cost | encounter ‰ | what it is |
|---|---|---|---|---|---|
| `tide` | `t` | no | — | — | sea that goes out; only ever `drains` to `coast` |
| `silt` | `s` | yes | 3 | 200 | what a room is floored with after it has been under water |

`lakebed` already exists and is drawn in no file; the Silt Stair's gallery
drains to `silt` rather than `lakebed` because eleven inches of it were
banked against the door and the door is the point.

### 2.5 `puzzle::solvable_blind(map) -> Result<usize, Unsolvable>`

Not a game rule; a test harness in core, so both the suite and the walker
share it. Given a floor, it takes every available choice at every event in
every order until the stair's `hidden_until`/`hidden_until_all` is met,
and returns the worst-case visit count. Board requirements
(`LooseItemOfSize`, `AlignedItems`, `Holding`) are satisfied by a fixture
that owns one of each footprint the floor asks for, because *having the
thing* is the player's problem and *finding the door that wants it* is the
puzzle's.

---

## 3. The Treyway's south — the Low Water

Rows 15–25 appended; rows 0–14 untouched. Bracketed **17–20**, above the
Reach's band and below both dungeons.

```
row 14  S:::--------:::S     (existing coast)
row 15  SSSSSSSStSSSSSSS     t = tide, drains to coast on built-the-tenth
row 16  SSSSSSSStSSSSSSS
row 17  S::::::::::::::S
row 18  S:--,,,,,,,,--:S
row 19  S:-,,%%,,,%%,,-S
row 20  S:-,,%,,,,,%,,-S
row 21  S:-,,,,,,,,,,,-S
row 22  S:-,%,,,,,,,%,-S
row 23  S:--,,%%%%%,,--S
row 24  S::,,,,,,,,,,::S
row 25  SSSSSSSSSSSSSSSS
```

| at | kind | id | what |
|---|---|---|---|
| [8, 14] | event | `the-tideline` | Visible from the first visit: a post at the water with the tide marked on it in eleven years of notches, and the note that the reach was wider on the way back. Sets `read-the-tideline`. Foreshadows; gates nothing. |
| [7, 21] | gate | `the-lip-of-the-sump` | `needs_survey: true`, `floors:` the-sump-1..4 keyed on each floor's stair (§4). Refusal is the Reach's: nothing to read it with. |
| [3, 19] | event | `the-ninth-clipboard` | The clipboard missing from the post on the Reach. Reading it sets `read-the-ninth`; the Cairnfield's golem hint (§4.3) is worded off it. |
| [12, 22] | event | `the-low-water-mark` | Prose and a `Counter` arm: `walked-the-low-water`. Nothing reads it this block; it is the watcher pattern, planted. |

Region `the-low-water`, bounds `[[1,17],[14,24]]`, enemies drawn from the
Reach's inner pool plus one — Rimefather at the top so it is the rarest.

---

## 4. The Wextreen Sump

The second surveyable place, and it is a pit. Four maps, `the-sump-1` (the
lip) to `the-sump-4` (the floor), each 12×12, each static, each read by
`survey::mods_for` unchanged. Bracketed **19–23**. No `outside`; every floor
has a stair up.

An instrument reads each of the first three floors, in the order the
instruments are learned — compass, atlas, golem — and each floor's puzzle is
short with it and possible without it.

### 4.1 Floor 1 — The Lip (the compass) — *the sluices*

**Redrawn after the artifact pass.** The first draft put wheel B one step from
the arrival tile, which is not a puzzle, and hung the stair behind all three
wheels, which makes the redundant wheel compulsory. Both are fixed below: the
wheels sit *below* the channels they drain, in the order you meet them, and the
redundant one is the one that costs a component.

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,==,,,,^     [5,1] stair down (the-sump-1-stair, hidden_until sluice-b)
   2    ^~~~~~~~~~~^     channel B
   3    ^,,,,,,,,,,^     [1,3] the bearing plate · [5,3] wheel B
   4    ^,%,,,,,,%,^
   5    ^~~~~~~~~~~^     channel A
   6    ^,,,,,,,,,,^     [1,6] wheel A
   7    ^%,,,,,,,,%^
   8    ^~~~~~~~~,,^     channel C — dry at [9,8] and [10,8]
   9    ^,,,,,,,,,,^     [10,9] wheel C
  10    ^,,,,==,,,,^     [5,10] start / stair up
  11    ^^^^^^^^^^^^
```

Three channels, three wheels, **each wheel drains one channel** —
`Drain { when: sluice-x, from: water, to: silt, tiles: [...] }`, monotone,
never refloods. The stair is behind channel B alone.

**The puzzle is the map, and one of the three wheels is not needed.** Channel C
is walked round at its dry east end, so wheel C buys a crossing that is already
there — and wheel C is the one that costs you something you do not get back.
The floor is a tax on not looking:

| wheel | requires | what it costs |
|---|---|---|
| C [10,9] | `LooseItemOfSize { 2, 2 }` | **a 2×2 jams it open and stays in it** — for a channel crossable at [9,8] |
| A [1,6] | `Holding("Map Shard")` | nothing; you keep the shard |
| B [5,3] | `Flag("read-the-lip")` — set by the bearing plate at [1,3], which `requires Surveying("compass")` **or** `Took("Pace it out")`, a second choice costing a fight and 12 fatigue | the compass, or a walk |

The compass's own reading (quiet, −20%) is what makes pacing it out survivable
at 260‰ slag, so the instrument pays twice on this floor and neither payment is
the door. Blind solution: three wheels at ≤ 3 visits each, plus the plate —
**≤ 10**. With the compass: **1**.

### 4.2 Floor 2 — The Shelf (the atlas) — *the weighed door*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,,D,,,,,^     [5,1] the door (the-sump-2-stair)
   2    ^,,%%,,,%%,,^
   3    ^,%,,,,,,,%,^
   4    ^,,,,L,,,,,,^     [4,4] the lintel (event)
   5    ^,%,,,,,,,%,^
   6    ^,,%%,,,%%,,^
   7    ^,,,,,,,,,,,^
   8    ^%%,,,,,,,%%^
   9    ^,,,,,,,,,,,^
  10    ^,,,,==,,,,,^     start / stair up
  11    ^^^^^^^^^^^^
```

The door is a `door` place whose event has three choices:

| choice | requires | outcome |
|---|---|---|
| Put something in the slot | `LooseItemOfSize { 3, 2 }` | the component stays in the door; `the-sump-2-stair` answered |
| Walk two of you through | `AlignedItems(2)` | free; answered |
| Try the slot with what you have | `none` | `unmet`-style prose: *it is not that shape*, and nothing else |

The lintel at [4,4] `requires Surveying("atlas")` and says the shape — *three
by two, and it does not care which way up*. Without the atlas the third
choice is the only clue, and there are eleven footprints in the catalogue;
a player cycles their tray. Blind: **≤ 11 visits**, and the fixture owns a
3×2 (there are a hundred).

The atlas's reading — louder, paying — is why the shelf's enemies are the
Reach's inner pool at +10%: the floor that costs you a component is the
floor that pays for it.

### 4.3 Floor 3 — The Cairnfield (the golem) — *the order of the nine*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,,,,,,,,^
   2    ^,4,,,,,,7,,^     nine cairns, one event each: the-cairn-1..9
   3    ^,,,,,,,,,,,^
   4    ^,,,,1,,,,,,^     digit = which surveyor's, and this is NOT written
   5    ^,8,,,,,,,3,^     on the map — only the height is in the prose
   6    ^,,,,,T,,,,,^     [5,6] the tenth cairn = stair (hidden_until cairn-9)
   7    ^,2,,,,,,,9,^
   8    ^,,,,,,,,,,,^
   9    ^,,,,5,,,6,,^
  10    ^,,,,==,,,,,^     start / stair up
  11    ^^^^^^^^^^^^
```

Nine cairns. Each has one choice, *Add a stone*, which `requires
Flag("cairn-N-1")` — the previous surveyor's — and sets `cairn-N`. The
first requires nothing. The prose gives each cairn's **height** and nothing
else; the heights are not in surveyor order and are not in height order —
the third surveyor's is the tallest, as the Reach already said, and the
ninth's is the shortest, because she went down.

The order **is** findable: `the-ninth-clipboard` on the Low Water (§3) lists
nine heights in surveyor order. A player who read it walks the field once.
The golem reads it for you: an event at [5,3], *the stones*, `requires
Surveying("golem")` and says which cairn is next. Blind: try each unanswered
cairn until one takes — **≤ 45**, the number §1.2 is built on.

`unmet` on a cairn is *there is no stone under this one yet* — it does not
say which cairn is under it, because that would be the clipboard for free.

### 4.4 Floor 4 — The Sump Floor — *the Ninth Surveyor*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^~~~~~~~~~~~^
   2    ^~~~,,,,,~~~^
   3    ^~~,,,,,,,~~^
   4    ^~,,,,B,,,,~^     [5,4] boss: The Ninth Surveyor
   5    ^~,,,,,,,,,~^     [6,4] door onto the-undercountry (hidden_until_all)
   6    ^~~,,,,,,,~~^
   7    ^~~~,,,,,~~~^
   8    ^~~~~,,,~~~~^
   9    ^~~~~~,~~~~~^
  10    ^~~~~~=~~~~~^     start / stair up
  11    ^^^^^^^^^^^^
```

One boss, on an island in the water at the bottom of the reach. **The Ninth
Surveyor** is new in `enemies.json`, rated one notch above Sootmother
(recon: by DPS bracket against the M13 walker's level-22 board, not by
copying 4910/124 and adding). Drops are catalogue: *Map Shard*, *Astrolabe*,
and a third the recon picks from the instrument components so the golem is
buildable off this floor by anybody who reached it.

The door at [6,4] is `hidden_until_all: [the-ninth-surveyor, the-bottom-of-the-bottom]`.

---

## 5. The Silt Stair

Behind the door under the lake. Four maps, `the-silt-stair-1..4`, 12×12,
bracketed **18–22** — a step below the Sump, because the lake is reachable
earlier than the Low Water. No `outside`; every floor has a stair up, and
the top of floor 1 is the door under the lake, so the way out is still two
hundred and six steps. `no_homeward` on all four, for the lake's reason.

The motif is *carrying* — the stair was cut for people carrying something —
and Marbulon, whose door this is the back of.

### 5.1 Floor 1 — The Landing — *the manifest*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,,,,,,,,^
   2    ^,%%%,,,%%%,^
   3    ^,%,,,,,,,%,^
   4    ^,%,,M,,,,%,^     [4,4] the manifest (event)
   5    ^,,,,,,,,,,,^
   6    ^,%,,,,,,,%,^
   7    ^,%,,,,,,,%,^
   8    ^,%%%,S,%%%,^     [5,8] the stair down (the-silt-stair-1-stair)
   9    ^,,,,,,,,,,,^
  10    ^,,,,==,,,,,^     start; stair up = the door under the lake
  11    ^^^^^^^^^^^^
```

The stair is cut for something **one wide and four long**, and it will not
take you down without one. Two choices:

| choice | requires | outcome |
|---|---|---|
| Lay it in the stair | `LooseItemOfSize { 1, 4 }` | it stays; answered |
| Carry the lens down | `Holding("The Cracked Lens")` | you keep it; answered — *it was carried down once already* |

Nineteen catalogue pieces are 1×4, most of them grips and blades. The
Cracked Lens is Sootmother's drop, so a player who came through the lake
the intended way holds the key already, and the manifest at [4,4] says so:
a list of what was carried down, in a hand that stops after *a lens, cracked*.
Blind: **≤ 2**.

### 5.2 Floor 2 — The Chair Room — *the chair*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,,,,,,,,^
   2    ^,,,,,,,,,,,^
   3    ^,,,,,,,,,,,^
   4    ^,,,,,D,,,,,^     [5,4] the door (the-silt-stair-2-stair, hidden_until chair-9)
   5    ^,,,,,,,,,,,^
   6    ^,,,,,C,,,,,^     [5,6] the chair (event)
   7    ^,,,,,,,,,,,^
   8    ^,,,,,,,,,,,^
   9    ^,,,,,,,,,,,^
  10    ^,,,,==,,,,,^     start / stair up
  11    ^^^^^^^^^^^^
```

An empty room with a kitchen chair in it facing a door. One event, three
choices, always all three offered: **Turn it to face the door. Count to
four. Turn it back.** Nine flags, `chair-1..9`, each choice's outcome gated
by `Took` of the right previous choice — the sequence is *face, four, back*,
three times, which is exactly what Marbulon does in front of her own door on
the first map, twice more while you stand there. A wrong choice is
`FightInstead` — the room's enemy — and no flag; the sequence does not
reset, because flags do not.

Blind: 3 choices × 9 steps, worst case **27**, and each wrong one is a
fight. A player who watched Marbulon does it in nine.

### 5.3 Floor 3 — The Drowned Gallery — *the two chains*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^,,,,,,,,,,,^
   2    ^,=========,^     the gallery: road that floods (drain flood, tiles rows 2–8 col 2–10)
   3    ^,=,,,,,,,=,^
   4    ^,=,,,,,,,=,^
   5    ^A=,,,,,,,=B^     [0,5] chain A · [11,5] chain B (events, in the wall)
   6    ^,=,,,,,,,=,^
   7    ^,=,,,,,,,=,^
   8    ^,=========,^
   9    ^,,,,,S,,,,,^     [5,9] stair down — under the gallery; hidden_until drained
  10    ^,,,,==,,,,,^     start / stair up
  11    ^^^^^^^^^^^^
```

Two chains in the walls. **Chain A floods the gallery** — `Drain { when:
gallery-flooded, from: road, to: water, tiles }`. **Chain B drains it to
silt** — `Drain { when: gallery-drained, from: water, to: silt, tiles }` —
and chain B's choice `requires Flag("gallery-flooded")`; pulled first, it
*will not move*, and says so. The stair at [5,9] is `hidden_until:
gallery-drained` — it was under the floor, and the floor had to go under
water and come back up to show it.

Two solutions, one puzzle: flood then drain, or flood and **wade** — the
Toad's Own Frame's `Rule::Wade` walks the flooded gallery, and a player
wearing it reaches the stair on water before chain B, because `Wade` opens
the whole body since M11.4. The stair's `hidden_until` is still `gallery-
drained`; the wading route reaches chain B faster, not the stair. (Recon:
whether a wading shortcut is worth drawing at all — if it saves nothing it
is cut.)

Blind: A, B — or B, A, B — **≤ 3**.

### 5.4 Floor 4 — The Bottom of the Bottom — *what Marbulon faced away from*

```
        0123456789AB
   0    ^^^^^^^^^^^^
   1    ^ssssssssss^
   2    ^s,,,,,,,,s^
   3    ^s,,,,,,,,s^
   4    ^s,,,,B,,,s^     [5,4] boss: What Marbulon Faced Away From
   5    ^s,,,,D,,,s^     [5,5] door onto the-undercountry (hidden_until_all)
   6    ^s,,,,,,,,s^
   7    ^s,,,,,,,,s^
   8    ^ssssssssss^
   9    ^,,,,,,,,,,^
  10    ^,,,,==,,,,^     start / stair up
  11    ^^^^^^^^^^^^
```

A silt floor and one thing on it. **What Marbulon Faced Away From** is new
in `enemies.json`, rated a notch above the Ninth Surveyor — it is the deeper
of the two by one map's worth of walking, and the last boss before the
country under the country. Drops: *Cosmic Orb*, *Map Shard*, and a third
chosen so the pair of bosses between them drop every instrument component
once. Its prose is Marbulon's: *she did not need to look at it. She needed
you to.*

The door at [5,5] is `hidden_until_all: [the-ninth-surveyor, the-bottom-of-the-bottom]`.

---

## 6. The Undercountry

`the-undercountry`, 20×20, bracketed **23+**. Three ways in, one town, and
the town is empty.

| at | kind | id | what |
|---|---|---|---|
| [10, 10] | town | `the-third-town` | Name is the human's (§9). Shop with no shelves, no errands, and prose that says the writing stops here. |
| [3, 17] | gate | `the-way-up-the-sump` | to `the-sump-4` [6,4] — you arrive beside the door you came through |
| [16, 17] | gate | `the-way-up-the-stair` | to `the-silt-stair-4` [5,5] |
| [10, 2] | gate | `marbulons-door-from-below` | to `west-bambulon` [3,10] |

And on West Bambulon, `marbulons-door` gains a gate beside it:
`[3, 9]`, `gate`, `the-door-in-the-shallows`, to `the-undercountry` [10,2],
`hidden_until_all` both bosses — and Marbulon's event gains a third choice,
`requires Flag`-both, *Ask her what is behind it now*, which is the one line
the map's prose has been waiting for.

One region, `the-undercountry`, enemies from the two bottoms' pools with a
third new creature at the top for rarity (recon). Terrain is the Treyway's
vocabulary at the Treyway's scale — it is a country — and the layout is the
human's to draw or the builder's to draw plainly; **twenty tiles of it are
road between the three gates and the town, and nothing else on it is
load-bearing.**

---

## 7. Milestones

Order: **M14.0, M14.1, M14.2, M14.3, M14.4, M14.5.** Deploy points after
M14.2 and after M14.4. M14.3 could ship before M14.2 — the Silt Stair does
not depend on the Low Water — but the Sump exercises `Surveying` and the
drain `tiles` harder, and the block wants the harder one first.

| # | Milestone | Deliverables | Acceptance | Deploys |
|---|---|---|---|---|
| **M14.0** | **The primitives** | `Requirement::Surveying`; `Drain.tiles`; `PlaceDef::hidden_until_all`; `tide` and `silt` in `terrain.json`; `puzzle::solvable_blind` in core; the refusal for a partly-met `hidden_until_all` that names what is missing. **Nothing a player can see.** | `a_survey_requirement_reads_the_frame`; `a_drain_with_tiles_leaves_the_rest`; `a_drain_naming_a_cell_it_cannot_drain_does_not_load`; `hidden_until_all_names_what_is_missing`; `no_flag_is_waited_on_forever` extended and negative-tested; `solvable_blind` over a fixture floor with a known count | no |
| **M14.1** | **The Low Water** | `the-treyway.tiles.json` to 16×26; four places; region; `the-lip-of-the-sump` gate with `needs_survey` and four `floors` (onto maps that exist as **empty 12×12 stubs** with a stair up and nothing else, so no gate points at a hole). Theme entries. | Reachability over every tile still derives; `the_tide_goes_out_on_the_tenth_cairn` (drain fires on the flag, not before); `no_gate_into_the_treyway_moved`; the walker reaches row 21 in a transcript | no |
| **M14.2** | **The Wextreen Sump** | Four floors as §4 draws them; three puzzles; the Ninth Surveyor in `enemies.json` with drops; every event and refusal string. | `solvable_blind` ≤ 10 / 11 / 45 / — on the four floors, **each asserted at its number**, not `< 100`; `each_sump_floor_is_short_with_its_instrument` (with the compass, atlas, golem the count is 1 / 1 / 9); `the_lip_needs_one_wheel` (a walk that turns only B reaches the stair); `the_shelf_takes_a_3x2_and_keeps_it`; `a_cairn_says_nothing_about_which_is_under_it`; `the_ninth_surveyor_is_beatable_by_the_walker_at_22`; the sealed door names the Stair | **yes** |
| **M14.3** | **The Silt Stair** | Four floors as §5 draws them; three puzzles; What Marbulon Faced Away From in `enemies.json`; the manifest; the chair's nine flags; `no_homeward` on all four. | `solvable_blind` ≤ 2 / 27 / 3 / —; `the_chair_is_marbulons_sequence` (face-four-back ×3 opens it in nine; anything else fights); `chain_b_will_not_move_dry`; `the_gallery_drains_to_silt_and_not_lakebed`; `a_wader_reaches_chain_b_on_water`; `the_lens_carries_you_down`; the sealed door names the Sump | no |
| **M14.4** | **The Undercountry** | `the-undercountry.tiles.json`; the town, empty and honest; three gates in, three back; Marbulon's door and her third choice; `hidden_until_all` on both bottoms. Theme entries. | `both_bottoms_open_together_and_not_before`; `marbulons_door_opens_onto_the_same_map`; `the_third_town_has_no_shelves_and_says_so`; reachability derives over twelve maps; `every_gate_lands_beside_its_door` (arrive on a tile adjacent to the gate that brought you) | **yes** |
| **M14.5** | **The gate, the walk, the handoff** | Browser gate checks (below); a `make play` transcript that reaches the third town; `HANDOFF-M14.md`; `CLAUDE.md` updated with the maps table at twenty maps, the divergences, the numbers. | The five checks, **each negative-tested**; the transcript's step count and level at the third town written into `CLAUDE.md` | no |

**Browser gate, M14.5** — five checks sharing one character, three engines:

| check | what only a browser can answer |
|---|---|
| `check_the_tide_is_drawn_before_it_goes_out` | the two `tide` tiles are on the page from the first visit, and after `built-the-tenth` they are coast and steppable |
| `check_the_sump_refuses_without_an_instrument` | the lip's refusal is the Reach's sentence, and an assembled compass opens it |
| `check_a_wheel_says_what_it_wants` | wheel A's refusal names 2×2, and a 2×2 in the tray turns it and is gone from the tray |
| `check_the_chair_fights_you_wrong` | a wrong choice at the chair raises a fight and the door stays hidden; nine right ones show it |
| `check_the_third_town_is_empty_on_purpose` | the town screen opens, shows no shelves, and its prose is the stop-line |

---

## 8. Numbers, in one place

| Thing | Number |
|---|---|
| New maps | 9 (4 + 4 + 1); maps in the game 20 |
| Treyway | 16×16 → 16×26; rows 0–14 unmoved |
| Floors with a puzzle | 6; floors with a boss 2 |
| Blind-solution ceilings | Sump 10 / 11 / 45; Stair 2 / 27 / 3 |
| With the right instrument | Sump 1 / 1 / 9 |
| New requirements | 1 (`Surveying`) |
| New `PlaceDef` fields | 1 (`hidden_until_all`) |
| New `Drain` fields | 1 (`tiles`) |
| New terrains | 2 (`tide`, `silt`) |
| New creatures | 2 bosses + 1 for the Undercountry's rarity slot |
| New components | **0** — every drop and every key is catalogue |
| Brackets | Low Water 17–20 · Silt Stair 18–22 · Sump 19–23 · Undercountry 23+ |
| Ways into the Undercountry | 3 |
| Shelves in the third town | 0 |

---

## 9. Decisions that are the human's

1. **The third town's name.** `the-third-town` is the id; the name on the
   tile and in the theme table is not the builder's to invent. The plan
   ships it as *a town with no name on the post yet* if unanswered, which
   is true and in register.
2. **The Undercountry's layout.** §6 asks only that the road exists. If the
   next plan wants the country a particular shape — the Stack drawn from
   below, a coast, a second reach — that is cheaper to draw now than to
   redraw.
3. **Whether the Reach's `built-the-tenth` is the tide's flag**, or the
   ninth clipboard's `read-the-ninth` should be. The first ties the south
   to finishing the Reach, which is the recommendation; the second lets a
   player who skipped the cairns find it, at the cost of the line about the
   way back.
4. **The wading shortcut on floor 3 of the Stair** (§5.3). It is one
   sentence of design and zero code, and it may be worth nothing; recon
   decides whether it is drawn or cut.
5. **Whether the two bosses' third drops complete the instrument set.** The
   plan says yes — a player who did both dungeons should be able to build
   the golem — but that is a claim about the catalogue the recon has to
   count.
