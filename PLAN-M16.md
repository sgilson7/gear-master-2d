# PLAN-M16 — The Eleven Reefs

*A three-floor dungeon under the Wextreen Sands, a puzzle on every floor that
is harder in kind than M14's, and at the bottom a creature wearing a real
player's board. Written against the tree at `b3fdb4f` (M15 done and not yet
deployed; the Sands shipped in M15's bestiary block): twenty-one maps, 892
tests, `survey::mods_for` reading its map argument at last, and a save from
the human's own run at level 45 that this block uses as its yardstick and its
boss.*

*Ships with `testing/saves/the-run-20260910.json` — the save, unmodified —
and the boss's gear block in §5, transcribed from it.*

*§10–§13 fold in the second ask: two new base classes — one that burns held
pools into mana empowerment, one that makes mind damage a way to kill — and
the eleven expert classes the roster needs to stay C(n, 2) at seven.*

---

## 0. The one-paragraph version

The Wextreen Sands is already surveyable — `the-edge-of-the-sands` has
`needs_survey: true` and refuses in the Reach's sentence — and it stays that
way. What it gains is a **way under**: a gate on the flat that is not there
until you have taken the tenth surveyor's sheet, and needs an instrument on
the frame to open. Under it are three floors. **The Flat Below** is the eleven
reefs the Sands' prose promises, read by the atlas and lied about by the
compass; **The Assay** is three doors that are answered by packing, and the
second one takes a piece off you that the third one needs; **The Needle
Room** is four sinkholes and four levers around a plate where a compass has
been spinning for eleven years. In the middle of the plate stands **The Tenth
Surveyor**, and she is wearing your board — the human's own run, seated where
the human seated it, enchs and all — because she has been watching you pack
for four hundred paces.

---

## 1. Decisions

### 1.1 The run is the yardstick, and it is in the repository

`testing/saves/the-run-20260910.json` is a level-45 character with 475 wins,
33,904 Fnorp, three classes (Gorillathon, Top of the Bill, Short Programme),
six enchs seated, a caster's weapon board (two alignments, three inks, the Bog
Census), and every dungeon before this one done except the Sump's last floor.
**This block is bracketed against that save, not against a walker's board.**
`HANDOFF-M14.md` §2.7 is why: *a level-22 board is not a board this game
produces*, and the last two bosses were rated against one. A fixture that is
a real run is a fixture nobody can argue with.

`common::from_save(path)` loads it into a `Character` for the suite. It is one
new helper, and `geared_from` stays for the early game.

### 1.2 Puzzles harder in kind, not only in count

M14's six puzzles measured at 8 / 1 / 45 / 1 / 3 / 3 blind. Three of them are
one visit. *"More complicated"* here means each floor is a **compound**: two
or three mechanisms on one floor whose flags meet at the stair, with a
different mechanism per floor —

| floor | mechanisms | what it charges blind | with the atlas |
|---|---|---|---|
| The Flat Below | terrain that opens (`Drain.tiles` to a new impassable terrain), decoy pockets, an instrument that lies | 9 pulls · 45 fatigue · 6 dead-end fights | 3 pulls · 15 fatigue |
| The Assay | three board locks in sequence, and the second **takes** what the third **needs** | a component, or 9,000 Fnorp, and a repack | the office names the shape; the repack is still yours |
| The Needle Room | `Outcome::Warp` as a puzzle piece — four sinkholes, two of them false — and four levers chained through drains | 4 sinkholes · 16 fatigue lost to the false ones · 4 levers | 2 sinkholes · 4 levers |

Every one is monotone (§1.3) and every one has a counted blind ceiling
(§7), which are M14's two rules and are not relaxed.

### 1.3 Monotone, still — and quicksand is why it can be

M14 §1.1: flags only grow, so no move may make the stair unreachable. The
Flat Below wants terrain that *closes* as well as opens — that is what sand
does — and it cannot have it. What it has instead is **quicksand**: a new
impassable terrain that only ever drains *to* silt, never from it. Nine
stakes each open one segment of one band, and pulling all nine opens
everything, so a blind walk cannot strand. The puzzle is that six of the nine
open pockets that lead nowhere and cost five fatigue each, and the sheet
says which three do not.

### 1.4 The gate is hidden until the sheet, and shut until an instrument

`the-way-under-the-flat` is `hidden_until: the-tenth-survey` — you have to
have taken the unsigned survey off the folding table — **and**
`needs_survey: true`. Both, in that order: the sheet is how you know there is
an under, and the instrument is how you go into it. The refusal is the edge
gate's own sentence, because it is the same refusal.

The sheet gives a Map Shard, which is the atlas's component; the needle gives
a Magnet, the compass's. The Sands hands you both instruments' parts and then
tells you in its own prose that one of them is no use down here. The dungeon
holds it to that: `Surveying("compass")` is offered on every floor and **never
sets a flag** — it says what a compass says over iron, which is everything at
once.

### 1.5 Three floors, not one sitting

As M14: `floors` on the gate with `cleared` = each floor's stair; no
`outside`; a stair up on every floor; `no_homeward` on all three because a
dungeon under a tidal flat is under the tide. The way back in lands on the
first unsolved floor.

### 1.6 The boss wears the run

`MonsterSpec` grows one field, `enchs`, so a creature can carry an ench on a
gear entry the way a player carries one on a piece — and the Tenth Surveyor
carries all six of the run's. **Classes do not apply to creatures** and are not
made to: she has the board and the enchs, not the Gorillathon's naked rule or
the Short Programme's window. What she has instead is a boss's health and
strength, set by DPS bracket against the save (§5.2), so the fight is *your
board, on something that does not die when you do*.

---

## 2. What the engine grows

| addition | what it is | why the block cannot be data without it |
|---|---|---|
| **`quick`** terrain | glyph `q`, impassable, 0‰; drains only to `silt` | the Flat Below's bands; `tide` is the precedent |
| **`MonsterSpec.enchs`** | `&'static [(&'static str, usize)]` — ench id, gear index; applied in `loadout_at` after the piece is seated, before the lock | the boss carries six; nothing in the bestiary carries one |
| **`common::from_save(path)`** | a `Character` from a `gm2d-save` file, catalogue-checked | §1.1 |
| the items partition from a save | `Character::item_partition()` — the `items` list a creature needs, read off the save's locks | the boss's `items` field is not a number anyone should type |

Four things. `Outcome::Warp`, `Outcome::GiveUp`, `Outcome::Tire`,
`Requirement::Gold`, `Requirement::AssembledOfRarity`,
`Requirement::Surveying`, `Drain.tiles`, `hidden_until_all`, `needs_all`,
`repeats` and `puzzle::solvable_blind` all exist and are what the floors are
built from. **No new components.**

---

## 3. The gate, on the Sands

| at | kind | id | fields |
|---|---|---|---|
| [11, 5] | gate | `the-way-under-the-flat` | `to: the-reefs-1`, `hidden_until: the-tenth-survey`, `needs_survey: true`, `floors: [reefs-1-stair, reefs-2-stair, the-tenth-surveyor]`, `no_homeward` on the far side |

[11, 5] is silt between the two reefs at [13,4]–[13,5] and the one at [9,7],
which is where the sheet's north arrow points. `shut` is the edge gate's
sentence verbatim. The Sands' region and enemies are untouched.

---

## 4. The three floors

Rows are drafts to be walked, as every M14 floor was. Glyphs: `^` rock,
`s` silt (the sand), `%` slag reef, `q` quicksand, `_` lakebed.

### 4.1 Floor 1 — The Flat Below — *nine stakes, three bands, one sheet*

```
        01234567890123
   0    ^^^^^^^^^^^^^^
   1    ^ssssssssssss^     [7,1] stair down (reefs-1-stair, hidden_until band-c-mid)
   2    ^^^^^^ss^^^^^^     the neck
   3    ^%%%%^ssss^%%%^    rooms above band C — the middle one goes on
   4    ^qqqq^qqqq^qqq^    band C: C-left [1–4] · C-mid [6–9] · C-right [11–13]
   5    ^ssssssssssss^     stakes C at [3,5] [7,5] [11,5]
   6    ^^ss^^^^^^^^^^     the neck — left
   7    ^ssss^%%%%^%%%^    rooms above band B — the left one goes on
   8    ^qqqq^qqqq^qqq^    band B
   9    ^ssssssssssss^     stakes B at [3,9] [7,9] [11,9]
  10    ^^^^^^^^^^^ss^^    the neck — right
  11    ^%%%%^%%%%^sss^    rooms above band A — the right one goes on
  12    ^qqqq^qqqq^qqq^    band A
  13    ^ssssssssssss^     start [7,13] · stakes A at [2,13] [7,13]→[8,13] [12,13] · the sheet [1,13]
  14    ^^^^^^^^^^^^^^
```

Three bands of quicksand, each in three segments walled from one another.
Below each band a corridor with **three identical stakes**; each stake drains
one segment — `Drain { when: stake-x, from: quick, to: silt, tiles }` — and
the segments are not labelled. Above each band, three rooms: one has a neck
in its ceiling that leads to the next corridor, and two are slag pockets with
a creature in them and nothing else. Which segment goes on changes per band:
**right, then left, then middle.**

A stake's card has three choices:

| choice | requires | outcome |
|---|---|---|
| *Pull it* | none | `Flag(stake-x)`, `Tire(5)` |
| *Read the flat with the atlas* | `Surveying("atlas")` | `Flag(read-band-x)` — and the card's prose then says *the one on the right*, or left, or middle |
| *Read the flat with the compass* | `Surveying("compass")` | nothing. *It points at all three of them, and at the eleven under them, and at you.* |

The sheet at [1,13] is `the-tenth-sheet-again`: with the atlas it names all
three at once (`Flag(read-band-a)`, `-b`, `-c`); without, it is a folding
table and a north arrow.

**Blind:** nine pulls, forty-five fatigue, and six pockets each holding one of
the Sands' creatures — a player who pulls everything fights six times for
nothing. `CAP` is 60, so a blind walk arrives at floor 2 at three-quarters
tired, which is the tax. **With the atlas:** three pulls, fifteen fatigue,
no pockets.

### 4.2 Floor 2 — The Assay — *three doors, and the second takes what the third needs*

```
        01234567890123
   0    ^^^^^^^^^^^^^^
   1    ^^^^^^^^^^^^^^
   2    ^sss^sss^sss^s^    [4,2] door one · [8,2] door two · [12,2] door three
   3    ^s^^^sss^sss^s^    [13,3] stair down (reefs-2-stair)
   4    ^s^^^s^^^^^^^^^    [5,4] the assay office (event, alcove off room two)
   5    ^s^^^^^^^^^^^^^
   6    ^ssssssssssss^     start [1,6] · stair up [12,6]
   7    ^^^^^^^^^^^^^^
```

A straight hall of four rooms, and every wall between them is a door that
asks the board a question.

| door | choices | what it asks |
|---|---|---|
| **one** [4,2] | *Walk through wearing it* — `AssembledOfRarity("epic")` | a live board with at least one epic item |
| **two** [8,2] | *Lay it in the door* — `LooseItemOfSize {2, 3}` → `GiveUp {2, 3}`, `Flag(door-two)` · *Pay the assay* — `Gold(9000)` → `Gold(-9000)`, `Flag(door-two)` | a loose 2×3, **which stays in the door**, or nine thousand |
| **three** [12,2], `hidden_until: door-two` | *Walk through wearing it* — `AssembledOfRarity("legendary")` | a live board with a legendary item |

The trap is the order. There are a hundred 2×3 footprints in the catalogue
and the run's chest is built of them — Chorister's Weave, Chain Layer,
Brigandine Base. A player with no loose 2×3 unseats one to pay door two, and
the item it came out of may no longer be legendary when door three asks.
**The floor is a repack**: keep a legendary standing while giving a 2×3 away,
or pay nine thousand and keep the board. The run has 33,904 Fnorp; the price
is a quarter of it on purpose.

The assay office at [5,4] `requires Surveying("atlas")` and prints, in the
catalogue's own words, *which of your loose pieces is two by three and which
of your items is legendary* — the board read back to you — so the repack is
planned rather than discovered. With the compass it prints the tray in the
wrong order.

**Blind:** one visit per door — but `solvable_blind`'s fixture owns one of
each footprint and a legendary, so what it measures is the *lock count*, not
the repack. `the_assay_costs_the_run_a_repack` (§7) is the test that measures
the repack, off the real save.

### 4.3 Floor 3 — The Needle Room — *four sinkholes, four levers, one plate*

```
        01234567890123
   0    ^^^^^^^^^^^^^^
   1    ^s^^^^^ss^^^^s^    [1,1] alcove NW · [12,1] alcove NE (levers)
   2    ^ssssss^ssssss^    [7,2] sinkhole N
   3    ^s^qqqqqqqqq^s^
   4    ^s^qqqqqqqqq^s^
   5    ^sSqqqq^qqqqSs^    [1,5] sinkhole W · [12,5] sinkhole E · [7,5] the plate — the boss
   6    ^s^qqqqqqqqq^s^
   7    ^s^qqqqqqqqq^s^
   8    ^ssssss^ssssss^    [7,8] sinkhole S
   9    ^s^^^^^ss^^^^s^    [1,9] alcove SW · [12,9] alcove SE
  10    ^^^^^^ss^^^^^^^    start [6,10] · stair up [7,10]
  11    ^^^^^^^^^^^^^^
```

A ring of silt around a square of quicksand, and in the middle of the square
the plate the needle has been spinning on. The four corner alcoves are
walled from the ring; the only way into one is to fall into the right
sinkhole.

| sinkhole | outcome | what it is |
|---|---|---|
| N [7,2] | `Warp { the-reefs-3, [12,1] }` | drops you in alcove NE |
| E [12,5] | `Warp { the-reefs-3, [6,10] }`, `Tire(8)` | drops you at the start, eight fatigue lighter — **false** |
| S [7,8] | `Warp { the-reefs-3, [1,9] }` | drops you in alcove SW |
| W [1,5] | `Warp { the-reefs-3, [6,10] }`, `Tire(8)` | **false** |

Every sinkhole `repeats` — you can fall in again — because a one-shot warp is
a flag that strands. Each true alcove holds a lever, and the levers chain
through drains:

| lever | requires | drains | and opens |
|---|---|---|---|
| NE [12,1] | none | quadrant NE of the square to silt | `tiles` also drain [12,2]→[12,3]: a path from NE's alcove **down to alcove SE** |
| SE [12,9] | `Flag(lever-ne)` | quadrant SE | — |
| SW [1,9] | none | quadrant SW | a path from SW's alcove **up to alcove NW** |
| NW [1,1] | `Flag(lever-sw)` | quadrant NW | — |

The plate at [7,5] is a `boss` place, `needs_all: [lever-ne, lever-se,
lever-sw, lever-nw]`, and the quicksand around it is what the four drains
remove. **Two sinkholes reach two alcoves, and two levers reach two more.**
The compass at the plate's edge (`the-needle-again`, `Surveying("compass")`)
says the needle points at all four, which is true and is not help. The atlas
(`Surveying("atlas")`) says N and S.

**Blind:** four sinkholes in the worst order is two false drops at eight
fatigue each, then four levers — **8 visits, 16 fatigue**. With the atlas: 2
drops, 4 levers, nothing lost.

---

## 5. The Tenth Surveyor

### 5.1 The gear, transcribed

Thirty-eight pieces across five boards, seated exactly where the run seated
them, rotations included. The instrument frame is not carried — a creature
has no survey. `items` is derived (§2), not typed.

```json
"gear": [
  ["Bone Crown","helmet",0,0,3], ["Idol's Crest","helmet",3,0,0], ["Bronze Plating","helmet",4,0,0],
  ["Bone Scale","helmet",1,1,2], ["Bronze Frame","helmet",4,1,0], ["Bone Fletch","helmet",2,2,3],
  ["Chorister's Base","chest",0,0,1], ["Chorister's Weave","chest",2,0,0], ["Chorister's Layer","chest",2,1,0],
  ["Brigandine Base","chest",4,1,0], ["Chain Layer","chest",0,3,0], ["Sprocketman's Gratitude","chest",4,3,0],
  ["Rimeglove Material","gloves",0,0,0], ["Gripping Mold","gloves",2,0,0], ["Mage's Wrapping","gloves",4,0,0],
  ["Tin Band","gloves",3,1,0], ["Oathring","gloves",1,2,0], ["Rat Signet","gloves",2,2,0],
  ["Witch's Thimble","gloves",3,2,0], ["Padded Mold","gloves",4,2,0],
  ["Plain Sole","greaves",0,0,0], ["Spun Material","greaves",2,0,1], ["Greave Mold","greaves",4,0,0],
  ["Ratskin Material","greaves",0,1,0], ["Spun Material","greaves",2,1,3], ["Sapling Mold","greaves",4,1,0],
  ["Herbal","weapon",0,0,0], ["Chain Coil","weapon",1,0,2], ["Cosmic Alignment","weapon",2,0,0],
  ["The Bog Census","weapon",3,0,0], ["Quicksilver Ink","weapon",5,0,1], ["Quicksilver Ink","weapon",2,1,1],
  ["Runewash Ink","weapon",0,2,2], ["Emberburst","weapon",2,2,0], ["Census Bolt","weapon",4,2,1],
  ["Ratchet Cog","weapon",0,4,1], ["Quicksilver Ink","weapon",2,4,0], ["Azure Alignment","weapon",4,4,0]
],
"enchs": [
  ["the-yodregar-index", 32], ["the-wextreen-correction", 33], ["the-chonga-swing", 9],
  ["sneel-bearing", 26], ["plug-energy-tap", 27], ["grungo-elastic-band", 31]
]
```

Ench indices are into `gear`: the Index on Runewash Ink, the Correction on
Emberburst, the Chonga Swing on Brigandine Base, the Bearing on Herbal, the
Tap on Chain Coil, the Band on the second Quicksilver Ink. Monster boards are
eight rows, so every placement seats without growth.

### 5.2 The numbers are found, not typed

| field | how |
|---|---|
| `health`, `strength` | by DPS bracket against `from_save`'s board: the save should beat her **between 55% and 70% of the time at Medium**, at Easy near always, at Hard rarely. `combat::simulate` is deterministic, so the bracket is a loop over seeds and a count. Start from What Marbulon Faced Away From's 7600 / 214 and move. |
| `regen` | 15, hers |
| resists | **base plus gear under 100**, every one. §5.3. |
| `rating`, `bounty` | derived, as every creature's are |
| drops | three catalogue pieces the run does not own and would seat: recon picks from the weapon orbs and the instrument components; **at least one must be `EVENT_ONLY`**, or the fight pays nothing the barrel does not |

### 5.3 The curse cap, and a lint for it

Both M14 bosses stand **past the curse cap** — the Ninth Surveyor at 114 and
What Marbulon Faced Away From at 120, base plus three gear pieces — which
means every curse in the game lands on them for zero duration and four expert
classes reach nothing on the two deepest fights. Nobody summed base and gear.

`no_creature_is_past_the_curse_cap` sums both for every creature and fails
at 100 or above. The Tenth Surveyor is authored under it, and **the two M14
bosses are brought under it by re-dressing, not by a new stat** — the
handoff for this block says which pieces moved and what their curse resist
reads now.

---

## 6. Milestones

| # | Milestone | Deliverables | Acceptance | Ships |
|---|---|---|---|---|
| **M16.0** | **The fixture and the primitives** | `testing/saves/the-run-20260910.json`; `common::from_save`; `Character::item_partition`; `quick` in `terrain.json`; `MonsterSpec.enchs` through `loadout_at`; `no_creature_is_past_the_curse_cap` (red on two creatures) | `the_save_opens_at_forty_five`; `a_quick_cell_only_ever_drains_to_silt`; `an_ench_on_a_creature_reads_like_an_ench_on_a_player` (same profile numbers as the save's own item); the curse lint red, then the two re-dressings, then green | no |
| **M16.1** | **The gate and the Flat Below** | `the-way-under-the-flat` on the Sands; `the-reefs-1` as §4.1; nine stakes, three necks, six pockets, the sheet; floors 2 and 3 as **empty stubs with a stair up** | `the_way_under_is_hidden_until_the_sheet`; `the_way_under_needs_an_instrument`; `solvable_blind` **= 9**; `the_flat_is_three_with_the_atlas`; `the_compass_sets_nothing_on_any_floor`; `every_pocket_holds_a_creature` | no |
| **M16.2** | **The Assay** | `the-reefs-2` as §4.2; three doors, the office | `solvable_blind` = 3 locks; **`the_assay_costs_the_run_a_repack`** — load the save, pay door two with a 2×3 off the chest, assert door three refuses, repack, assert it passes; `nine_thousand_keeps_the_board` | no |
| **M16.3** | **The Needle Room and the Tenth Surveyor** | `the-reefs-3` as §4.3; four sinkholes with `repeats`; four levers; the plate; the creature with §5.1's gear and §5.2's numbers; drops | `solvable_blind` = 8; `a_false_sinkhole_costs_eight_and_strands_nobody`; `the_levers_chain_through_the_drains`; **`the_tenth_surveyor_wears_the_run`** (her `outfit()` assembles the same items, by name and count per slot, as `from_save`'s `reports()`); `the_run_beats_her_between_55_and_70_at_medium` | **yes** |
| **M16.4** | **Two more base classes** (§10, §11) | `ClassPower::Stoker` and `ClassPower::Whisperer`; `OFFERED` at seven; two nine-node base trees in `skills.json`; theme names; the fork at seven cards | `every_offered_class_reaches_something` over seven; `a_stoker_burns_the_largest_pool_and_only_that_one`; `a_burned_pool_pays_no_held_bonus`; `an_unmaking_is_a_kill_and_pays_like_one`; `mind_resist_is_still_the_only_answer`; `the_fork_draws_seven_and_refuses_escape` | no |
| **M16.5** | **Eleven experts** (§12) | `EXPERT_DEFS` at twenty-one; eleven `ExpertPower` variants with knobs; eleven six-node trees at cost 2; `OFFERED_SECOND` at six | `every_pair_of_seven_has_exactly_one_expert` (= 21); `expert_nodes_touch_only_the_expert` over twenty-one; `every_expert_knob_is_declared` / `is_moved_by_some_node`; `no_pair_of_live_powers_disagrees` over every reachable triple of seven | **yes** |
| **M16.6** | **The gate, the walk, the handoff** | seven browser checks; `GM2D_FROM=testing/saves/the-run-20260910.json make play` reaching the plate; `HANDOFF-M16.md`; `CLAUDE.md` at twenty-four maps, seven classes, twenty-one experts | each check negative-tested; the transcript's fatigue at each stair written down, against §4's numbers | no |

**Browser gate, M16.6:**

| check | what only a browser answers |
|---|---|
| `check_the_way_under_is_not_drawn_before_the_sheet` | the tile at [11,5] is plain silt until `the-tenth-survey` is answered, then a gate |
| `check_a_stake_says_pull_read_or_lie` | three choices drawn; the compass one greys nothing and sets nothing; the atlas one changes the prose |
| `check_the_door_keeps_the_piece` | the tray loses the 2×3 on the door's card and does not get it back on reload |
| `check_a_sinkhole_moves_you` | the map's coordinates change on the choice; the false one also moves the fatigue figure by eight |
| `check_she_is_wearing_it` | the fight screen's enemy rating block lists her items by the run's own names |
| `check_the_fork_is_seven_wide` | the level-5 fork draws seven cards and still refuses Escape; the second fork draws six |
| `check_the_furnace_line_moves` | a Stoker's fight replay shows the pool figure fall and the empowerment figure rise on the same tick |

---

## 7. Tests and lints — the ceilings, at their numbers

- `solvable_blind`: **9 / 3 / 8**, asserted with `==`, per M14's rule.
- `the_flat_is_three_with_the_atlas` — `solvable_knowing` with the atlas on the frame is 3.
- `no_creature_is_past_the_curse_cap` — over the bestiary; red on two before M16.0 and green after.
- `the_tenth_surveyor_wears_the_run`; `an_ench_on_a_creature_reads_like_an_ench_on_a_player`.
- `the_run_beats_her_between_55_and_70_at_medium` — the bracket, as a test rather than a note.
- `a_false_sinkhole_costs_eight_and_strands_nobody`; `every_sinkhole_repeats`.
- `the_assay_costs_the_run_a_repack`; `nine_thousand_keeps_the_board`.
- `no_flag_is_waited_on_forever` over the new floors; `a_place_on_ground_you_cannot_stand_on_says_why` over the new places.
- `the_compass_sets_nothing_on_any_floor` — every `Surveying("compass")` choice's outcome is `Nothing`.

---

## 8. Numbers

| thing | number |
|---|---|
| new maps | 3 — twenty-four in the game |
| floors with a puzzle | 3; with a boss 1 |
| stakes / sinkholes / levers / doors | 9 / 4 / 4 / 3 |
| blind ceilings | 9 · 3 · 8 |
| with the atlas | 3 · 3 · 6 |
| fatigue a blind walk arrives at the plate with | 45 + 16 = **61, over the cap** — a blind run is meant to be turned back once |
| the assay's price | 9,000 Fnorp, a quarter of the run's purse |
| new terrains | 1 (`quick`) |
| new `MonsterSpec` fields | 1 (`enchs`) |
| new components | **0** |
| creatures past the curse cap after M16.0 | **0**, down from 2 |
| pieces on the boss | 38; enchs 6 |
| base classes | 7, up from 5 |
| expert classes | 21 = C(7, 2), up from 10 |
| new skill nodes | 18 base + 66 expert = **84** |
| new `ClassPower` variants | 2; new `ExpertPower` variants 11 |

---

## 9. The human's

1. **Her name.** *The Tenth Surveyor* is the plan's, on the sheet's own line
   — *the needle is no use down here, so I have used my eyes* — and it is
   the first creature named after a person the maps have talked about. If
   she is somebody else, the prose changes and nothing else does.
2. **Whether she keeps the instrument frame off.** §1.6 says creatures have
   no survey. Giving her the run's Magnet–Lens–Shard would mean a compass on
   a creature under iron, which is a joke the game could make.
3. **9,000.** A quarter of this run's purse; a different run's will differ.
   The alternative is the price scaling with level, which §1.2 of M13 argued
   against and still does.
4. **Re-dressing the M14 bosses** (§5.3) is a change to shipped content. It
   is the right fix and it is not this block's ask; it can be its own commit,
   or a row in the handoff for the next block.
5. **The two canonical names**, *Stoker* and *Whisperer*, and their theme
   names. §10 and §11 propose them; the theme table is the book's.
6. **Whether an unmaking pays drops.** §11.3 says yes — a kill is a kill —
   and it is the only place this block touches `settle`.

---

## 10. The sixth base class — the Stoker

**Canonical `Stoker`; theme *Kettle-Stoker*.** The class is built on a tension
the engine already has and nobody exploits: a held pool pays a standing bonus
(`held_bonus`), and mana empowerment is bought with mana and scales off the
mana left — *"stacking them hard drains the very pool they multiply."* The
Stoker buys empowerment with **somebody else's pool**.

### 10.1 The promise

> **Every four seconds, ten points off your largest held pool are burned into
> one stack of mana empowerment.** The pool goes down; the stacks stay for the
> fight.

`ClassPower::Stoker { every_ms: 4000, per_stack: 10 }`. Lands in the fighter.
It reads rage, faith and nature — the three pools `pools_worth_holding`
returns — and never mana or insight, which are spent already. Burning the
*largest* is the whole design: the standing bonus you were holding is what you
give up, and the class asks you to decide which pool to bank so that the
right one is the one that burns.

### 10.2 What the engine grows for it

- one tick on the fight clock beside the spin's, in `combat.rs`; `burn_ms`,
  `burned` on `Combatant`;
- `Event::Burned { pool, points, stacks }` in the log, so the replay can draw
  the pool fall and the stacks rise on the same line — the browser check in
  §6 reads it;
- nothing on the board.

**A creature is never a Stoker.** Same rule as experts.

### 10.3 The tree — nine nodes

| id | node | cost | requires | effect | what it is |
|---|---|---|---|---|---|
| A | **The Firebox** | 1 | — | `stat { nature: 20 }` | something to burn |
| B | **The Damper** | 1 | — | `stat { faith: 20 }` | a second thing to burn, so *largest* means something |
| C | **Coal by the Hundredweight** | 1 | A | `stat { rage: 24, health: 60 }` | |
| D | **A Hotter Draught** | 2 | B | `tunes { every_ms: −800 }` | every 3.2 s |
| E | **The Clinker Rake** | 2 | C | `tunes { per_stack: −3 }` | seven points a stack |
| F | **Held Over** | 2 | D | `grants { rule: burn_keeps_bonus { pct: 50 } }` | a burned pool pays half its standing bonus for the fight — the Stoker's only rule, and it is the trade made gentler |
| G | **Twice-Fired** | 2 | E, F | `tunes { every_ms: −800 }`, `start_with { mana: 10 }` | 2.4 s, and a pool of the other kind to scale off |
| H | **The Kettleworks Shift** | 2 | G | `stat { rage: 20, faith: 20, nature: 20 }`, `assembly_pct { pct: 10 }` | all three hoppers full |
| I | **Banked Fire** | 2 | H | `grants { rule: burn_carries { pct: 25 } }` | a quarter of the stacks survive to the next fight, which is the only thing in the class that crosses a bell |

Base trees are allowed flat stats; §1.6 of M13 applies only to experts.

---

## 11. The seventh base class — the Whisperer

**Canonical `Whisperer`; theme *Whisperling*** — Marbulon's chain is about
three of them and nobody has said what one is. Mind damage already eats
maximum health and `is_down` already fires at `max_health <= 0`, so a mind
build can kill anything today, in principle, and in practice never does: a
boss at 90 mind resist takes a tenth, and there is no mind pierce past the
Showstopper's `WrongSense`. The Whisperer makes the mind lane a way to
finish rather than a way to whittle.

### 11.1 The promise

> **A creature whose maximum health you have eaten down to a third of what it
> was is unmade** — dead, whatever is still in it.

`ClassPower::Whisperer { third: 33 }`. Lands in the fighter. It is an execute
threshold on the *mind* lane only: physical and magic damage move `health`,
not `max_health`, and do not count. What Dread and Insight already do —
`mind_bonus`, `DREAD_DIVISOR` — is untouched; the class changes where the
fight ends, not how fast the number moves.

### 11.2 What the engine grows for it

- `Combatant::unmade_at: Option<i32>` — set at fight start from the power,
  `max_health * third / 100`;
- one check in `take_mind_pierced` after the eat: `max_health <= unmade_at`
  → `health = 0`, and `Event::Unmade` in the log;
- **the kill is a kill.** `settle` pays bounty, drops and the tally the same
  way; the log line differs. §9.6 is the human's to veto.

**`mind_resist` is still the only answer** to the mind lane, and the test of
that name pins it: an unmaking at 90 resist is slow, not impossible.

### 11.3 The tree — nine nodes

| id | node | cost | requires | effect | what it is |
|---|---|---|---|---|---|
| A | **A Word in the Ear** | 1 | — | `stat { mind: 16 }` | |
| B | **Insight, Held** | 1 | — | `start_with { insight: 20 }` | fuel |
| C | **The Second Word** | 1 | A | `stat { mind: 14, mind_resist: 10 }` | |
| D | **Dread, Early** | 2 | B | `start_with { dread: 1 }` | the multiplier before the first hit |
| E | **Two-Fifths** | 2 | C | `tunes { third: +7 }` | unmade at 40% |
| F | **What They Heard** | 2 | D | `grants { rule: mind_pierce { pct: 20 } }` | a fifth of the resist walked through — the first mind pierce a class hands out that is not the Showstopper's |
| G | **The Long Whisper** | 2 | E, F | `tunes { third: +5 }`, `stat { mind: 12 }` | 45% |
| H | **Three of Them** | 2 | G | `start_with { dread: 2 }`, `grow_slot_rows { slot: helmet, rows: 1 }` | the helmet is the mind lane's board |
| I | **The Last Word** | 2 | H | `tunes { third: +5 }`, `grants { rule: mind_pierce { pct: 15 } }` | unmade at half, through 35% of the resist |

---

## 12. Eleven experts — the roster at seven

Seven base classes is twenty-one pairs; ten exist. The eleven below keep
`every_pair_of_seven_has_exactly_one_expert` true, and every one follows M13
§1.6: six nodes, cost 2, two roots reconverging on a capstone, and every node a
`tunes` of the expert's own knob, a `grants` of a kin rule, or a `gives_ench`
inside the licence. Nothing invented in combat that the two parents did not
already own between them.

Node shape throughout: A, B roots · C ← A · D ← B · E ← C · F ← E, D.

### 12.1 Stoker × Gorillathon — **Bare Furnace**

> An empty frame is a pool: each empty frame burns as ten points a tick.

Knobs: `per_frame`, `every_ms`, `cap`.

| # | node | effect |
|---|---|---|
| A | The Open Grate | `per_frame +5` |
| B | Draught Through the Frame | `every_ms −600` |
| C | Nothing to Slow It | `cap +4` — empty-frame stacks cap raised |
| D | The Cold Side | `grants burn_keeps_bonus { pct: 25 }` (kin: Stoker) |
| E | Stripped for Firing | `per_frame +5` |
| F | Furnace, Bare | `every_ms −600`, `cap +6` |

### 12.2 Stoker × Funnel Sergeant — **Fired Funnel**

> A stack the furnace bought may be spent as Funny: one stack is eight.

Knobs: `worth`, `per_fight`, `refund`.

| # | node | effect |
|---|---|---|
| A | The Exchange Window | `worth +4` |
| B | Standing Order, Hot | `per_fight +3` |
| C | The Boiler Receipt | `refund +25` — a cast paid in stacks refunds a quarter as mana |
| D | Twice Through the Funnel | `per_fight +3` |
| E | The Rate Goes Up | `worth +4` |
| F | Funnel, Fired | `refund +25`, `worth +4` |

### 12.3 Stoker × Worm-Fact Keeper — **Cold Stoke**

> A curse you land burns the target: ten of *their* largest pool a tick becomes your stack.

Knobs: `per_stack`, `every_ms`, `standing` (permanent curses burn double).

| # | node | effect |
|---|---|---|
| A | Their Coal | `per_stack −3` |
| B | Slow Match | `every_ms −800` |
| C | A Fact in the Fire | `standing +1` |
| D | `grants curse_on_activate { weapon, searing }` (kin: Keeper) | |
| E | Their Coal, Twice | `per_stack −3` |
| F | Stoke, Cold | `every_ms −800`, `standing +1` |

### 12.4 Stoker × Kaklon Patent — **Ponkey Boiler**

> The spin feeds the furnace: every spin stack is burned as five points.

Knobs: `per_spin`, `keep`, `licence`.

| # | node | effect |
|---|---|---|
| A | The Governor | `per_spin +2` |
| B | The Licence, Warm | `gives_ench plug-energy-tap` |
| C | Steam Kept | `keep +2` — spin stacks kept after burning |
| D | `grants spin_every { ms: 800 }` (kin: Patent) | |
| E | The Governor, Set | `per_spin +3` |
| F | Boiler, Ponkey | `keep +2`, `per_spin +2` |

### 12.5 Stoker × Top of the Bill — **Flash Powder**

> On the first activation everything held burns at once, and the stacks are gone by the tenth second.

Knobs: `all_at_once` (pct burned), `until_ms`, `pct` (the Showstopper's cut on a win inside it).

| # | node | effect |
|---|---|---|
| A | The Flash Pan | `all_at_once +25` |
| B | A Longer Fuse | `until_ms +2000` |
| C | The Poster, Singed | `pct +10` |
| D | Damp Powder | `until_ms +2000` |
| E | The Whole Pan | `all_at_once +25` |
| F | Powder, Flashed | `pct +15`, `until_ms +2000` |

### 12.6 Whisperer × Gorillathon — **Loud Doubt**

> Wearing two items or fewer, strength counts toward mind damage at one for four.

Knobs: `worn`, `rate` (quarters), `third`.

| # | node | effect |
|---|---|---|
| A | Said Plainly | `rate +1` — one for three |
| B | The Count | `worn +1` |
| C | Two-Fifths, Bare | `third +7` |
| D | `grants mind_pierce { pct: 10 }` (kin: Whisperer) | |
| E | Said Louder | `rate +1` — one for two |
| F | Doubt, Loud | `worn +1`, `third +5` |

### 12.7 Whisperer × Funnel Sergeant — **Requisitioned Silence**

> A cast that fails for want of Funny is said anyway, as mind damage equal to its cost.

Knobs: `rate` (pct of cost), `per_fight`, `third`.

| # | node | effect |
|---|---|---|
| A | The Unpaid Word | `rate +50` |
| B | The Form, Silent | `per_fight +2` |
| C | Heard Anyway | `third +5` |
| D | The Standing Silence | `per_fight +3` |
| E | The Unpaid Word, Twice | `rate +50` |
| F | Silence, Requisitioned | `third +7`, `rate +50` |

### 12.8 Whisperer × Worm-Fact Keeper — **Told Once**

> Each curse standing on the target raises the unmaking third by three points.

Knobs: `per_curse`, `cap`, `standing` (a permanent curse counts double).

| # | node | effect |
|---|---|---|
| A | Told Again | `per_curse +2` |
| B | Four Kinds | `cap +2` |
| C | A Standing Fact, Heard | `standing +1` |
| D | `grants curse_on_activate { helmet, stun }` (kin: Keeper) | |
| E | Told Louder | `per_curse +2` |
| F | Once, Told | `cap +3`, `standing +1` |

### 12.9 Whisperer × Kaklon Patent — **Licensed Rumour**

> An enched component's activation eats two percent of the target's maximum health.

Knobs: `pct` (tenths), `racks`, `third`.

| # | node | effect |
|---|---|---|
| A | The Rumour Clause | `pct +5` |
| B | The Licence, Whispered | `gives_ench the-yodregar-index` |
| C | Two a Component | `racks +1` |
| D | `grants beacon { pct: 25 }` (kin: Patent, via Full Bill) | |
| E | The Rumour, Twice | `pct +5` |
| F | Rumour, Licensed | `third +7`, `pct +5` |

### 12.10 Whisperer × Top of the Bill — **Curtain Line**

> A creature unmade inside the window pays the Showstopper's cut twice.

Knobs: `mult`, `under_ms`, `third`.

| # | node | effect |
|---|---|---|
| A | The Last Line | `third +5` |
| B | A Longer Bow | `under_ms +2000` |
| C | The Second House | `mult +50` — two and a half times |
| D | The House Clock | `under_ms +2000` |
| E | The Last Line, Held | `third +5` |
| F | Line, Curtain | `mult +50`, `third +5` |

### 12.11 Stoker × Whisperer — **Ash and Whisper**

> Every point burned in the furnace is also said: burned points count as mind damage at one for two.

Knobs: `rate` (halves), `third`, `every_ms`.

| # | node | effect |
|---|---|---|
| A | Ash in the Ear | `rate +1` — one for one |
| B | The Draught | `every_ms −800` |
| C | Two-Fifths, Burned | `third +7` |
| D | `grants burn_keeps_bonus { pct: 25 }` (kin: Stoker) | |
| E | Ash, Louder | `rate +1` |
| F | Whisper and Ash | `third +5`, `every_ms −800` |

---

## 13. What the roster change touches

| thing | before | after |
|---|---|---|
| `OFFERED` | 5 | 7 — the fork draws seven cards; the shim's card grid goes to two rows at four and three |
| `OFFERED_SECOND(first)` | `[; 4]` | `[; 6]` |
| `EXPERT_DEFS` | 10 | 21; `every_pair_of_seven_has_exactly_one_expert` |
| `ClassPower` | 30 variants | 32 |
| `ExpertPower` | 10 | 21 |
| new rules | — | `burn_keeps_bonus`, `burn_carries`, `mind_pierce` — three, each granted by more than one tree |
| `skills.json` | 16 trees | 29 |
| the paper on Spike's van | expert paper offered for the pair held | unchanged — twenty-one pairs is still one paper each |
| saves | — | no seam: a save with `class: Berserker` opens; the new names appear on the fork only |

The two base classes and the eleven experts are **M16.4 and M16.5** in §6,
after the dungeon and before the gate, so the browser gate walks all of it.
They ship after the Tenth Surveyor because the Tenth Surveyor is the ask and
the classes are the second ask, and a dungeon the human can play is worth more
than a fork with two more cards on it.
