# SECOND-ORDER-M11.md — what each change drags with it

Born at M11.0 and kept for the length of the block, in floodline's format and
for floodline's reason: **a reviewed feature once shipped invisible, because
the feature was reviewed and the message slot it shared was not.** The review
asks *is this right*. This asks *what else moves*.

**Every milestone lands at least one entry. An entry has three parts:**

- **the change** — what was done, in one line;
- **follows from** — what it necessarily drags with it, whether or not
  anybody wanted it to;
- **watch** — what would show it going wrong, and *where somebody would see
  that*, because a symptom nobody can see is the failure this file exists for.

**A change claiming no second-order effects still gets an entry saying so**,
because that is a claim and a claim can be wrong.

Questions raised at a deploy point go in here too, dated, under the point that
raised them.

---

## Seeded at birth — the six the plan already knew

These were written into `PLAN-M11.md` §6 before any of them was built. They are
repeated here rather than referenced, because a notebook you have to read two
documents to use is a notebook nobody opens.

### 1. One log slot (M11.0)

**The change.** Every message the game sends the player goes through a single
`log()` and lands in one strip, with a HISTORY overlay for the sitting.

**Follows from.** Every planted browser check that read the old `#says` slot
retargets. Every *future* feature that says anything inherits the slot — which
is the point of doing it first rather than last. The old element is **removed
and not shadowed**, so a writer that misses `log()` fails loudly rather than
printing somewhere nobody looks.

**Watch.** A message emitted by a path that bypasses `log()`. The symptom is a
feature working perfectly and being reported broken, which is M8's symptom
three times over — four skills, an opening armour bar, an ench you were paid
and could not see.

### 2. Per-map positions (M11.1)

**The change.** `WorldState` remembers where you stood on every map, not just
the one you are on.

**Follows from.** Loss-return: `last_town` can be on another map now, so the
walk home is a map change and not only a position write. `World::repair`'s
inputs widen with it.

**Watch.** A defeat on the overworld, and where it puts you.

### 3. Widening the Toad set's rule (M11.4)

**The change.** `Rule::Wade` stops meaning *the rim* and starts meaning *the
water*.

**Follows from.** Every place water gates anything: the rim rule's original
sites, `World::repair`'s in-and-out asymmetry, and the walker, which has been
routing round the lake since M9.

**Watch.** `make play` pathing straight through water it used to walk around,
and whether that shortens the walk enough to change what the transcript
measures.

### 4. A new component category (M11.5)

**The change.** `Map Shard` and the instruments are a category the catalogue
has not had.

**Follows from.** Every exhaustive match on category — the board's refusals,
`explain.rs`, the item card, the drop tables, the shop shelves.

**Watch.** A shard seating where only a weapon core should, or an instrument
that assembles and says nothing about itself on its card.

### 5. The ally row (M11.6)

**The change.** The survey golem stands as a third board in the replay.

**Follows from.** The replay's two-board layout, and rule 5 — *the page draws
numbers core sent it*. A third board is a third set of numbers the page must
not invent.

**Watch.** Absorbed damage on the golem. If the golem's bar moves by a number
the log did not report, the page has started computing again.

### 6. The homeward set (M11.9)

**The change.** A set that returns you to your last town for the price of one
restorative.

**Follows from.** The travel economy the rest of the block leaves alone. The
walk home is what makes a tin worth drinking and a town worth reaching, and a
teleport priced in tins re-prices both.

**Watch.** Whether the spot-run and `make play` stop walking home at all, and
whether tins start being hoarded as fare rather than drunk as medicine. Either
is the rule's price set wrong, and **the knob is the tin count, not the rule**.

---

## M11.0 — the output log

### The slot moved, and it moved before the content did

**The change.** `says()` became `log()`; the `<p id="says">` under the save
panel is gone; a `#tape` strip keeps the last four lines and `#history` keeps
the sitting.

**Follows from.**

- **Five browser checks and three transcript readers retargeted.** They read
  `#says` by id. `drive.py` grew `tape()` and `last_said()` so there is one
  reader on the test side too, and `playthrough.py` grew its own — the
  transcript now reads out of the log region, which is what a player reads.
- **`townSays` and `vendorSays` became two functions each.** The screen keeps
  its own sentence, because that is where you are standing, and the log keeps
  it too, because a message printed on a screen you walk out of is a message
  you cannot go back and read. `boardSays`, `treeSays`, `forkSays` and the
  quest log's own slot did **not** move: those are about the screen you are
  looking at rather than about the world, and putting a "does not fit there"
  into the transcript would bury the world's half.
- **The fight receipt is logged.** It is the only place a drop is ever named,
  and the result screen is walked away from. This is a *widening* — the log
  now holds things that never had a slot at all — and it is the reason the
  strip is four lines rather than one.
- **Two local variables called `log` had to be renamed** (`refreshPin`'s quest
  log and `runFight`'s combat log). Naming the emit path `log` in a file that
  already had two meanings for the word is trap 6 in miniature; the shadow was
  legal JavaScript and would have been a silent no-op message.
- **`#history` joins the z-index table at tier 30**, with the tree, the quest
  log and the ending — *what you opened from where you are*. It takes Escape
  and it blocks walking, like the rest of that tier.

**Watch.**

- A message that appears on a screen and never in the history. The check plants
  a save, reads the newest strip line, and requires the history to hold **more
  lines than the strip**, so a history that is the strip in a bigger box fails.
- The old element coming back. `check_the_game_talks_in_one_place` fails on
  `document.getElementById('says')` existing at all — it is cheaper to assert
  the absence than to find the writer that found it.
- The strip getting long enough to push the panel below the fold on a laptop.
  Four lines is a guess; if it wants to be three, that is `TAPE`.

---

## M11.1 — the overworld

### Three maps in a directory, a border, and two lists of walls

**The change.** `data/maps/<id>.tiles.json`; a sixteen-by-sixteen Treyway with
its own terrain vocabulary; `WorldState::positions`; and the door in the western
wall stops being an ending and becomes a gate.

**Follows from.**

- **Every map lint that walked one map now walks all of them.** Reachability,
  places-on-walkable-ground, encounter bounds and placed-events were written
  when there was one map and stayed that way through a second. A mistyped glyph
  on the Treyway would have been silent. `no_two_maps_name_a_place_the_same` is
  new and is the one nobody had noticed was needed: `answered`, `bought` and
  `quests_done` are one set each for the whole game.
- **`PlaceKind::Door` is now used by nothing**, and the `#ending` screen with
  it. Both stay: M11.4 has an ending to write and this is the screen for it.
  Stated here rather than discovered, because dead code that is *scheduled* and
  dead code that is *forgotten* look identical from the outside.
- **A gate's key may come off a boss now, not only off an errand.**
  `the_witchs_key_is_the_key_the_cave_wants` asserted one faucet and there are
  two. The list is closed in the test, which is where the closing is stated.
- **`at_to` became optional in meaning as well as in type.** A gate that names
  it is a dungeon's mouth; one that does not is a border. That is now a content
  decision written in the map file, which is where it belongs, and it is why
  `every_gate_leads_somewhere_you_can_stand` checks the map's `start` when no
  landing tile is named.

**Watch.**

- **A defeat that crosses maps** — entry 2's watch, and it fired on the first
  run. The page caches the grid and seven call sites re-read it by hand; the
  eighth, the loss walk home, did not, so dying in the Cave left the canvas
  drawing a nine-by-five room around a player standing at (1, 18) of it. It has
  been shipped since M8 and nothing found it because nothing had lost a fight
  in the Cave. Fixed as a *class*: `position()` carries the map id and
  `paintPanel` compares it, so every path that moves anybody is covered rather
  than an eighth site being added to a list of seven.
- **A list of terrain names written outside `terrain.json`.** Two were found in
  one run — the odds overlay's `rock`/`water` skip and the walker's
  pathfinder's — and both were correct until a map added a wall they had never
  heard of. `world_json` reports per-tile passability now and both read it.
  Anything that grows a third such list should be looked for here first.
- **A `.screen` the walker does not know about.** The van has swallowed every
  keypress at [4, 6] since M10 and nothing noticed, because no run had reached
  level ten on foot before this one. The symptom is the walk stopping dead and
  blaming the *road*: "not reachable yet" about a tile that is perfectly
  reachable. Any new screen wants a branch in `playthrough.py`'s loop in the
  same commit.

---

## M11.2 — the town map, dense

### A category of event that answers nothing, and a town that stopped being staged

**The change.** `kettleworks-field`, twenty by twenty, forty-three of its four
hundred tiles carrying something. Kettleworks placed with its M8 shelf and
errands. The Drambus Stack in the middle as its own terrain. Two questlines,
three errands each, each crossing a map.

**Follows from.**

- **`TileEvent` may have no choices now, and that is a category.** The engine
  refused one outright — *"offers no choices, so it cannot be answered"* — which
  was right while every event was a card. An **examinable** is a post or a pond
  or a wall somebody built out of rind: nothing to spend, never written into
  `answered`, and it reads the same on the ninth crossing as on the first. The
  consequence to watch is the one that cannot be tested for: a *card* written
  with its choices left off by accident is now legal content.
  `most_of_the_field_is_something_to_look_at_rather_than_something_to_answer`
  counts both kinds, which at least makes the ratio visible.
- **Kettleworks came off `STAGED`**, which is what that list was for — and the
  test that reads it had to grow from *the overworld's towns* to *the world's
  towns*, along with `every_town_has_an_errand`,
  `the_shipped_errands_parse_and_name_things_that_exist` and
  `every_errand_names_a_creature_that_is_actually_out_there`. Every one of those
  said "the map" and meant "the first map".
- **An errand may now cross a border**, and its giver, its turn-in and the place
  it sends you to can be on three different maps. `quest::guide` already
  answered across maps; nothing had ever asked it to.
- **`__standAt` grew a sibling.** It has always written an empty `map`, meaning
  *the first one*, which was invisible while there was one map and is a trap
  with four. `__standHere` keeps the map you are on. Two names rather than an
  argument with a default, because the default was the silent part.

**Watch.**

- **A slaying errand nobody can finish.** `draw_enemy` weights a pool so its
  hardest member is its rarest, which is `PLAN.md` §6b row 1 — and placing
  Kettleworks made its four-nock errand *live*, at one draw in ninety-seven,
  which is four hundred wins. Found by
  `every_slaying_errand_asks_for_something_you_actually_meet`, which is new and
  which fails at sixty expected wins. Fixed as content, the way this project
  tunes a map: **Bone Archer lives in the Slag Flats as well as the pit**, where
  it is the easiest of four rather than the hardest of three. The Flats' danger
  moved by five and the Wallspider Weave got cheaper along with the errand,
  which is the same row of §6b paying off from the other end.
- **A check that opens a screen and leaves it open.** The field check stood *on*
  a tile and stepped off and back, and the first of those two steps can roll a
  fight — which swallowed the second keypress and left the fight screen up for
  every check after it. Stand beside, take one step, `close_fight` on every path
  out. Trap eight, third coat.
- **Forty cards is not forty good cards.** The budget is a floor on *count* and
  says nothing about whether any of them is worth reading. That is a reader's
  job and M11.7's tone pass is where it happens.

---

## M11.3 — the cheese tower

### A counter that was not written, a sitting that is a map's own property,
### and a walker that had to learn how a person plays

**The change.** Five ten-by-ten floors of the Drambus Stack; the door in the
field opens onto whichever is still standing; beating a floor's boss drops the
tower and puts you out; when they are all gone there is a stump.

**Follows from.**

- **There is no `tower_floors_cleared`.** `PLAN-M11.md` §M11.3 asks for a
  counter in `WorldState`; beating a boss already writes its tile id into
  `answered`, so *how many floors are gone* is how many of those are there.
  Derived, never banked — the rule an ench, a level and a skill's effect all
  follow. It also means **there is no `tower_dropped` flag**: floor one is
  reachable only once the four above it are down, so its boss being answered
  *is* the tower being down, and M11.4's lake reads that id.
- **`TilesData::outside` is one field doing two jobs**, and they are the same
  job: where clearing the floor puts you, and where a save taken inside reopens.
  `world::leave_the_sitting` is the one function both call, so the kick and the
  reopen cannot disagree.
- **`World::arrival` had to learn about sittings.** A gate with no landing tile
  lands you where you left off, which is right for a border and wrong for a
  floor: where you left off on a floor is the tile the boss is standing on.
- **`remember_at`**, and this one was found by playing. A gate is walked *onto*,
  so by the time the crossing is handled `at` is the doorway — and the kick put
  the player back on the doorstep, one keypress from the next floor. It writes
  the tile you stepped *from* now, which is also better for the border.
- **`PlaceDef::prose` stopped being the door's.** A boss carries a paragraph
  now — the floor coming down, counting what is left — and it counts correctly
  because the order the floors come down in is fixed and written.
- **The gate's card learned its own name.** `showCard('THE DOOR IN THE WALL', …)`
  was hard-coded, so walking into a tower of cheese two maps from that wall
  announced itself as the door in the wall.

**Watch.**

- **Five things the walker had to be taught, and every one of them is a thing a
  person does without thinking.** They are listed here because each was a run
  of twenty thousand presses that measured nothing, and because the next map
  will find the sixth:
  1. **Walk on the road.** Shortest-path crossed the Stack's Shadow off-road at
     twenty-eight percent a tile where the road is six, and lost two thousand
     two hundred fights doing it.
  2. **Do not walk through the shop.** `town` was in the road set, so the
     cheapest route to anywhere ran through the counter.
  3. **Do not clear the give-up list every time you bank.** It cleared the mark
     that says *this errand's tile is where you already are*, and the walk went
     into Kettleworks seven hundred and forty times buying a tin each visit.
  4. **A give-up that has somewhere to go is not a give-up.** The
     back-off-when-losing check fell through to a branch that sent the walk to
     the counter it was standing beside: two thousand six hundred visits.
  5. **Setting out means not turning round at the first fight.** Banking at
     twenty-five carried meant banking after every win, and a run got six
     hundred and eighty-eight wins and not one floor of the tower.
- **The round trip is the block's real cost, and it is a finding about the
  game rather than the walker.** There is one town past the door, its shelf is
  Kettleworks', and a character who arrives under-geared cannot catch up there
  — the run that was forbidden from going back lost two thousand four hundred
  fights standing in a field. Going back is right and the game supports it; what
  it means is that most of a run is now spent walking between the pit and the
  tower, and the errand log keeps pointing east. **`TRIAGE-M11.md` gets this.**

---

## M11.4 — the lake drains

### Terrain that is derived, a rule that widened, and a place standing in water

**The change.** West Bambulon's lake empties when the Drambus Stack's bottom
floor is answered. There is a grating in the middle of it with two hundred and
six steps under it, one boss at the bottom, and a door behind the boss that says
the writing stops there. `Rule::Wade` widened from the rim to the whole body.

**Follows from.**

- **`data::map` and `data::map_now` are now two questions.** The first is the
  file — is the map well formed, where is the town, what does a lint see. The
  second is the file as the game has left it. Every question a *game* asks moved
  to the second, and a lint that could only see the drained lake could not see
  the undrained one, which is what everybody plays for two blocks.
- **The shim's `map_named` became `map_in` and takes the marks.** Not the state:
  nearly every call site is a closure that then *mutates* the state, so holding
  a borrow across the call is a borrow error at eight sites. `WorldState::marks`
  is the smallest thing that answers *has this happened*, and it also names the
  fact that `answered` and `flags` have always been read together.
- **`ever_walkable` is a third answer beside `passable` and `walkable`**, and it
  exists because a place may stand on ground that opens later or opens for
  somebody. The load check and two lints use it. What it still catches is the
  thing they were for: a place in rock, or in a sea nothing opens.
- **`Rule::Wade`'s line lost its number**, because the number *was* the limit.
  `the_new_rules_describe_themselves` grew a named exemption rather than a
  fabricated quantity — a spec that invents a number to satisfy a lint is the
  failure the two-register split exists to stop.
- **Three tests about the rim were rewritten and one measurement was kept.**
  `the_rim_is_shallow_and_the_middle_is_not` still measures fourteen and
  fourteen and now decides nothing; it is kept because it is the argument M9
  made, and the shape of the lake has not changed.

**Watch.**

- **Everywhere water gates anything** — notebook entry 3, and this is where it
  came due. `World::repair` still reads the allowance going in and ignores it
  coming out, which is now load-bearing in a new way: a wading player standing
  in the *middle* of the lake who unpacks the set is repaired to land, and the
  land they are repaired to is further away than it was.
- **The walker routing through the lake.** It could not before and can now, if
  it ever earns the Toad set. `make play` pathing straight across a lake it used
  to walk round is the symptom to look for, and it is not a bug — it is entry 3
  arriving.
- **A drain is a whole-map rewrite waiting to happen.** `drain_by` swaps every
  tile of one terrain for another across the map. It is right for a lake, which
  is the only water on the map it is on; it would be catastrophic on a map where
  `from` is the ground everybody walks on. `every_drain_names_terrain_that_exists`
  refuses a drain whose `from` is the terrain in the corner of the map, which is
  the cheapest available proxy for *this is not the whole map*.
- **The under-lake is one map read twice, and the difference is the walk** —
  twenty-one tiles of slag against eleven of road. That is the only currency a
  dungeon here has, and it is worth saying because `PLAN-M11.md` proposed
  positioning, and combat has no board.

---

## M11.5 — map shards and the instruments *(the first seam)*

### Six components, two meanings of "core", and a grid that has to choose

**The change.** `Map Shard`, `Glass Lens`, `Magnet`, `Cosmic Orb`,
`Cosmic Alignment` and `Living Earth` join the catalogue. Three recipes on the
weapon board build a compass, an atlas and a survey golem; an assembled one
grants `Rule::Survey`. The catalogue is 550 and every save written before it is
refused.

**Recon, as the plan asks for it in the commit:** none of the five names the ask
gives existed. `Scrying Lens`, `The Cracked Lens` and `Nine-Plane Lens` are all
in the catalogue and none of them is a `Glass Lens`; there is no magnet, nothing
cosmic and no living earth. So all six are new, which is the whole seam.

**Follows from.**

- **`is_core` had two meanings and they had to be told apart.** It reads as
  *the piece a recipe is built around* and it means *the piece an item is split
  on* — `items_with_locks` hands every other piece to its nearest one. A shard
  that anchored an item split every instrument bigger than a compass: the atlas
  came out as three items that each needed something. So `Shard` is not a core
  in that sense, reads at a core's brightness in `look::kind_luminance`, and
  `recipe_parts` looks for it *before* the core when naming a recipe — otherwise
  an atlas is called a crystal ball.
- **`Cosmic Orb` and `Cosmic Alignment` are the kinds a crystal ball uses.**
  Reused rather than invented, because a cosmic orb set into a ball is a good
  ball. That decides the shape of the exclusion below, and it means both had to
  earn a `power_bonus` — `an_orb_out_damages_a_book_for_the_room_it_costs` says
  every orb scales what a ball casts, and a zero would have been an orb that
  could not be used as one.
- **The exclusion has a gap in it, on purpose.** `is_survey` and
  `is_weapon_gear` are two lists with `Orb`, `Alignment`, `Enchantment` and
  `Quest` in neither. What is refused is a *blade* beside a shard, which is what
  the plan asks for in as many words; what is allowed is a ball's parts, which is
  what the atlas is made of.
- **`PlaceDef::drops` became a list**, and was always going to be one: the
  Stack's five floors each pay their own piece *and* a shard.
- **`a_set_is_a_few_hours_and_not_a_lifetime` grew a second band.** A set is
  three pieces and an afternoon, 25 to 400 wins; a single supporter is a *part*,
  2 to 40. One band for both would have called a magnet a broken set.
- **Two lints widened to every map** — the drop-hours test and the reward
  sources — because the instruments' parts come off creatures two maps out.

**Watch.**

- **Every exhaustive match on `PieceKind`.** Four new kinds went in and the
  compiler found `look::kind_luminance` and nothing else, which is the good
  outcome and also the thing to be suspicious of: anything matching on kind with
  a `_ =>` arm silently accepted them. `explain.rs` was widened by hand.
- **A shard seating where only a weapon core should** — notebook entry 4's
  watch, and the answer is that it can, in the sense that a shard and an orb
  share a grid. That is the atlas and it is deliberate; what to look for is the
  *other* direction, a blade sitting beside a shard, which `can_equip` refuses.
- **The seam.** Deploy point D's note says old saves are done and why. The
  number lives in `sets.rs::a_save_from_before_this_block_is_refused_by_name`
  and nowhere else, which is where it has always lived.
- **An instrument that grants nothing.** `Rule::Survey` is carried and named and
  **does nothing at all** until M11.6 — which is the one shape this project has
  shipped twice and written a lint about (`every_offered_class_reaches_something`).
  It is stated here so that it is a schedule and not a silence, and M11.6's
  acceptance is the thing that closes it.

---

## M11.6 — the surveyable map

### A map that is read rather than rolled, and a golem that fights once

**The change.** `the-reach`: twenty by twenty, authored, static. The edge of it
on the Treyway opens only for somebody carrying an assembled instrument, and
what the map *is* while you are on it depends on which one — a compass makes the
ground quieter and reads better off a packed board, an atlas pays more and stops
you more often, and a golem takes the first fight.

**Follows from.**

- **`World` grew a `survey` field and it is never in a map file.**
  `nothing_in_a_map_file_knows_about_a_survey` is the architecture note as a
  test, and the payoff is the whole reason it is written that way: a second
  surveyable map is a data drop plus an arm in `survey::mods_for`.
- **`data::map` now has two siblings.** `map_now` is the file as the game has
  left it and `map_read_through` is that, through an instrument. The board's
  count is the *caller's* — `world.rs` may not go and read a character, which is
  the division `Allowances` already makes.
- **The shim's `marks` became a `Seen`.** It was a list of strings; a map now
  needs three things about the game and all three have to be owned, because
  nearly every call site mutates the game inside the closure.
- **`drops::roll` grew `roll_with`**, and the atlas moves the *threshold* rather
  than the number of draws. A survey that skipped or added draws would make the
  stream a function of what you walked in with, and a seeded walk would stop
  replaying.
- **`needs_survey` is a fourth thing a gate can want**, beside a component, a
  level and an id in `answered`. It is answered in the shim for the reason a key
  is: a map does not know about bags, and it does not know about rules either.
- **Two events had to move one tile.** The post at the reach and the woman at
  the Kettleworks turn were both *on* the tiles that became gates, and an errand
  that says "go and read the post" must not want an instrument. That is now
  twice this block; a place that an errand names should not also be a door.

**Watch.**

- **The golem's fallback was taken, and it was a decision.** `PLAN-M11.md` §8
  row 6 named it in advance so that taking it would not be a retreat, and the
  reason it was taken is not the replay's layout: it is rule 5. A third board is
  a third set of numbers the page must not invent, and the honest version is a
  third combatant in `combat.rs` — new combat code in a block that has added
  none. **The ally row is M12's**, and this is the entry that says so.
- **Entry 5's watch does not apply and that is worth writing down.** There is no
  ally row, so there is no absorbed damage on a golem to check. What replaces it
  is `GOLEM_SPENT`: a mark in `answered` cleared at the *gate* rather than on the
  way out, because what it records is this entry. Watch for a golem that fights
  twice, which would mean the clear is happening in the wrong place.
- **A survey is in the save and has to be.** The map is what it is while you are
  standing on it, so a save taken inside one reopens inside the same one —
  `save_is_whole` carries it now. What to watch is the opposite: a survey that
  outlives the walk off the reach, which would be an atlas quietly paying forty
  percent in the pit.
- **The lens is on the panel**, because a survey moves the encounter rate, what
  falls off a win and what a win pays, and a player sees none of those directly.
  A derived number with nowhere it is shown cannot be told from a bug, and this
  block has now written that sentence four times.

---

## M11.7 — the bug and triage pass

### The block was unfinishable and every test was green

**The change.** Nine enemy pools and one boss retuned; a lint that would have
caught it; `TRIAGE-M11.md`; a tone pass; and the walker taught to rest before it
goes underground.

**Follows from.**

- **The measurement, and it is the whole milestone.** Ratings are not
  difficulty for this engine: the best board the game hands out beats
  Gallowglass at 1,534 and loses to Silence at 1,006, because matchups and burst
  decide it. So every pool in this block had been tuned against a number that
  does not predict the outcome, and the honest way to tune them was to simulate
  the whole ladder against `common::geared_from` and read the answer off.
- **`every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten`.** It
  checks the *most drawn* creature, because `draw_enemy` makes the easiest
  member the commonest — a region's teeth are supposed to be the fight you
  sometimes lose, and what must not happen is that the fight you meet three
  times in five is one you cannot win. And every boss, because a boss is not
  drawn: it stands there and there is no going round it.
- **`the_top_of_the_stack_is_reachable_and_the_bottom_of_it_is_not_free`
  became `the_floors_cost_more_than_the_things_at_the_end_of_them`.** Its old
  band said the best board should take two to four of the five bosses, which
  read as *the tower should cost something* and meant *the tower may be
  impossible*. The cost moved to where it actually is: the walk to the boss.

**Watch.**

- **A band that permits a wall.** The old assertion was `(2..5).contains(&taken)`
  and it *passed* on a tower that could not be finished. A range whose lower
  bound is "not everything works" cannot tell a cost from a wall, and this is
  the second time in the block a lint has been wrong in that direction (the
  first was `every_offered_class_reaches_something`'s ancestor, which read a
  list rather than the behaviour). **Prefer a floor to a band** wherever the
  thing being bounded is *reachability*.
- **The pools were tuned against one board and there is only one.** Boards stop
  growing at level six and Kettleworks is the last shop, so everything past the
  door is fought with the same grid — which is why a single fixed yardstick was
  the right one and is also triage rows 6 and 7. If a second town or a third
  shelf ever lands, every number in this milestone is measured against a board
  that no longer exists.
- **`make play` reaches the ending and never surveys.** Auto-pack collects six
  shards and packs none, because a compass rates worse than the blade it would
  replace. The reach is content the walk cannot reach, which means the browser
  gate is the only thing standing between it and rotting. Same shape as M10.3's
  ench finding, and the same answer: the button is not an optimiser.

---

## M11.9 — the bestiary *(the second seam)*

### A bench that had to be taught that rating is not difficulty

**The change.** `crates/lab` and `make dress` / `make read`. Eight new creatures
on the new maps' pools. Six sets of three — eighteen components, the block's
second and last fingerprint move. `Rule::Homeward`, which is the one piece of
new travel and is priced in a tin.

**Follows from.**

- **`is_core` had a third reader.** `recipe_parts` names a recipe off it, the
  item splitter splits on it, and now the bench seeds a grid with it. All three
  wanted the same thing and one of them (the splitter) wanted something
  narrower, which is why `Shard` is not one — M11.5's entry has the rest.
- **`PieceDef::drops` being a list paid off twice.** The Curd Mantle is three
  certainties off three tower floors, on top of what those floors already paid.
- **`SETS` went from three to nine and `every_set_is_one_creatures_and_one_grids`
  had to widen.** Its rule was *one creature owns the whole of a set, or it is a
  shopping list*; the Curd Mantle comes off three bosses. **One stack of floors
  counts as one owner**, because a floor is one sitting, so a set off a floor's
  *pool* would be unfarmable and a set off its bosses is the tower paying at the
  top, the middle and the bottom.
- **Five of six rules are old ones tuned to a new instance**, which is the
  block's standing rule and the reason six sets cost no combat code.
- **Two fixtures had to be rebaselined**: `gear_at.txt` (a ladder that grew) and
  `enemies.json`. Both are ratchets and both said what they would write.

**Watch.**

- **The bench will spend health to hit a number, and health is not difficulty.**
  Its first eight creatures were dressed at flat strength and resists, so every
  point of rating past the gear came out of the body — and it produced a
  thousand-rated creature that **lost to an Oak Handle and an Iron Blade.**
  `the_floors_cost_more_than_the_things_at_the_end_of_them` caught it. The bench
  scales the whole body with the target now, off the ladder's own ratios
  (a seventeenth, a twenty-second, a twentieth), and caps what it will spend on
  meat at 2.3 health a point — past which it says *give it more grids, not more
  meat* rather than quietly inflating.
- **And the second attempt made a wall.** Re-dressed at 1,507, The Ground Floor
  beat the best board the game hands out, and
  `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` refused it
  as a boss within one run. It stands in floor one's *pool* instead, where being
  a fight you sometimes lose is exactly what a rarest member is for.
- **So the tower keeps its borrowed bosses, and that is a divergence.**
  `PLAN-M11.md` §M11.9 asks for distinct bosses on floors that shipped on
  borrowed frames. M11.7 measured every one of those frames against the board
  the game hands out and retuned the whole block around the result; re-dressing
  them threw that measurement away and produced a wall on the first try. A new
  face on a *pool* costs nothing that has to be re-measured against
  reachability. **Six sets, not seven**, for the same shape of reason: three
  sets on tower floors would have to come off floor *pools*, and a floor is one
  sitting.
- **`Rule::Homeward` is the first rule that is a gesture.** Not a fight input,
  not a step, not a lens: the player asks and `Game::go_home` answers. Watch what
  entry 6 said to watch — whether tins get hoarded as fare rather than drunk as
  medicine, and whether the walk home stops happening. **The knob is the tin
  count, not the rule**, and it pays the *cheapest* tin it is carrying so that
  the fare comes out of small change.
