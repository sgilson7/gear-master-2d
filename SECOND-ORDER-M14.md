# SECOND-ORDER-M14.md — the notebook

*Written when noticed, not at the end. `SECOND-ORDER-M13.md` is the precedent
and M13.9 is what made keeping one worth it: seven rows marked as candidates,
all seven executed, three of them turning up something that was actually
wrong.*

**A row marked `M14.6 candidate` is worklist.** The last milestone of this
block reads this file and executes them.

---

## Rows

### 1. The plan is written against the dead `Requirement`, not the live one

`PLAN-M14.md` §1.1 names `LooseItemOfSize`, `AlignedItems` and
`AssembledOfRarity` as *"locks a player opens by packing"*, and §4.1, §4.2 and
§5.1 build three doors on them. **All three are on `event::Requirement`, which
is the cut campaign's type**, `Copy`, `&'static str`, and not deserialisable.
The type the game uses is `tile_event::Requirement`, and it has four arms:
`None`, `Gold`, `Flag`, `Holding`.

This is `CLAUDE.md`'s own warning arriving on schedule — *"Grep for it, and
then check which of the two you found"* — and it is the single largest thing
the plan understates. M14.0 grew by three ported arms.

**Status: done in M14.0.**

### 2. `AlignedItems` has nothing to read in GM2D

The dead type's own doc says it is *"at least this many assembled items sharing
one alignment word"* — upstream's naming system, where an item's generated name
carried a word other items could share. The fork kept `naming.rs` but nothing
exposes an alignment word on an assembled item, and `PieceKind::Alignment` is a
*component kind* the crystal ball and the atlas use, which is a different thing
wearing the same noun.

So §4.2's second choice cannot be built as written. `AssembledOfRarity` is the
plan's own third named lock, reads the same live board, and is what the shelf
door takes instead.

**Status: divergence, recorded. Done in M14.2.**

### 3. `no_flag_is_waited_on_forever` does not exist

M14.0's acceptance says *"extended and negative-tested"*. It is a doc comment
on `event::Requirement::Flag` describing a lint the campaign had; nothing in
`crates/core/tests` is called that. It is written rather than extended, over
the live events, and it is the reason a chain root that hands over a flag
nobody reads is caught.

**Status: written in M14.0.**

### 4. A tile event is answered once, and the chair needs nine

`Game::answer_event` refuses a second choice on an event already in
`answered` — *"a card could sell one thing once"*. §5.2's chair is one event
that wants nine answers.

`TileEvent::repeats` is what M14.0 adds. What it cannot fix is the other half:
**a sequence of nine over three always-offered labels is not expressible in
monotone flags.** See the divergence in M14.3.

**Status: primitive done in M14.0; the chair diverged in M14.3.**

### 5. `Outcome::Counter` would be `Outcome::Xp` planted on purpose

§3 asks for a `Counter` arm on `the-low-water-mark`, *"nothing reads it this
block; it is the watcher pattern, planted."* `Outcome::Xp` wrote into
`world.counters["xp"]` for four blocks and nothing read it, and nine events
printed a receipt for experience that never existed. `CLAUDE.md` has the rule
in as many words: **a number that is shown needs somewhere it is read.**

Planting one deliberately is that bug with a note beside it.

**Status: cut. Divergence recorded in M14.1.**

### 6. The instrument set is already completable, and only off a roll

Counted rather than assumed (§9 decision 5). The golem is three `Map Shard` and
two `Living Earth`. Map Shards are certainties — five off the Drambus Stack and
one off Sootmother. `Living Earth` has exactly one source in the game: **Bone
Cantor at 300 per mille**, a roll you have to win twice.

So the plan's claim is true and thin. One `Living Earth` off each of the two new
bottoms turns the golem from two rolls into a certainty, which is the version of
"a player who did both dungeons can build the golem" that is worth writing down.

**Status: answered in M14.2 and M14.3.**

### 7. Rating does not predict whether the board wins

Measured against `common::geared_from`, which is this repository's answer to
*the board a player actually has*:

| it beats | it loses to |
|---|---|
| Sootmother 1670, Anvilheart 1803, The Salt Wedding 1897, The Last Light 2031, Francis 2958 | Cairn Chorus 1141, The Tallow Saint 1223, Weeping Idol 1240, The Last Gearwright 1353, The Ground Floor 1507, Gilt 2489, Nine of Ashes 2519 |

It beats a 2958 and loses to a 1141. What decides it is **damage per second**,
not rating: Cairn Chorus kills it in 13.3 seconds and Sootmother leaves it on
1297 of 1942 after forty-three. `PLAN-M14.md` §4.4's instruction — *by DPS
bracket, never by adding to Sootmother's* — is right, and this is the
measurement behind it.

**Status: the bracket the two bosses were dressed against.**

### 8. The walker reaches level 14, not 22

`CLAUDE.md`'s current transcript is *342 wins, 170 losses, level 14, 4,406
steps*. M14.2's acceptance names `the_ninth_surveyor_is_beatable_by_the_walker_
at_22`, and a level-22 board is not a board this game produces.

`common::geared_from` is what M11.7 established as the thing to measure
against, and it is what the test measures against — under a name that says what
it measures.

**Status: divergence, recorded in M14.2.**

### 9. A negative-test harness that restores an old mtime measures the fault

The `neg.sh` loop was `cp` the file, break it, run, `mv` it back. `mv` restores
the **bytes and the old mtime**, so cargo saw nothing newer than the last build
and did not rebuild — every run after a restore measured the binary with the
fault still compiled in.

It cost a negative test its meaning before it was noticed:
`a_door_keeps_the_cheapest_thing_that_fits` was reported as *"still passed with
the fault in"*, which sent me looking at the test (where there was, separately,
a real problem — see row 10) rather than at the harness. `touch` after the
restore.

**Status: fixed, and the two findings it was tangled with are both real.**

### 10. A check that reads its answer off the thing it is checking

`a_door_keeps_the_cheapest_thing_that_fits` asked whether `give_up` took
`loose_of_size(w, h)[0]` — the same list, in the same order, compared with
itself. Reversing the sort passed it cleanly.

It works the cheapest out from the ratings now. **This is the
compares-zero-with-zero failure with an extra step**, and it is the fifth of
that shape this project has shipped.

**Status: fixed in M14.0.**

### 11. Three narrow lints were one lint, and a fourth was right

Hiding a stair behind a *flag* in a game that had only ever gated on `answered`
set off three checks, each for the right reason and the wrong question, and
**each of the three counted only place ids**:

| lint | what happened |
|---|---|
| `ending.rs::every_hidden_place_names_something_that_happens` | retired, subsumed by `no_flag_is_waited_on_forever` |
| `lake.rs::every_drain_names_terrain_that_exists` | its *"waits for something that can happen"* clause went the same way |
| `events_pay.rs::every_flag_an_event_sets_is_read_by_something` | **the mirror**, and it earned its keep twice |

The third is the one worth keeping: it reads the *other* direction — a flag
raised and never consulted — and once it learned about `hidden_until_all`,
`needs_all`, `floors[].cleared` and drains, it was still right about two live
faults. See rows 5 and 12.

**Status: done in M14.1.**

### 12. `read-the-ninth` was going to be prose

§4.3 has the ninth clipboard *"worded off"* the Cairnfield's golem hint. A
wording is not a read, and `every_flag_an_event_sets_is_read_by_something`
refused it.

What information can actually *do* in this engine is open a shorter path. So
the Cairnfield's slab takes two: stand a golem on it, or walk the field once
with the sheet in your hand. Nine moves become one either way, and the flag is
consulted rather than admired.

**Status: done in M14.1 and M14.2.**

### 13. A map file must not name the instrument that reads it

`reach.rs::nothing_in_a_map_file_knows_about_a_survey` refused the Sump's own
`_note`, which said which instrument reads each floor. It is right, and it is
M11.6's architecture stated as a lint: a map that knows which lens reads it is
a map that has to be edited to add a second one.

**Status: fixed in M14.2. Worth knowing that the lint reads the whole file,
`_note` included — which is what makes it hold.**

### 14. What a creature rates is mostly how many items its board makes

Two drafts of two bosses were dressed and both were wrong in the same way, and
neither was about a number:

- The Ninth Surveyor's first weapon grid was a hilt and two accessories, which
  **assemble nothing**. She dealt 7.8 damage a second, and dealt exactly 7.8 at
  strength 152 and at 320 — a sweep of seven healths against four strengths came
  back Victory in all twenty-eight.
- What Marbulon Faced Away From's first board made **three** items where the
  Surveyor's makes eight, because a hilt, a key and a charm in one row touch and
  merge. 114 a second, on the creature that is meant to be the deeper of the two.

**The coordinates are the dial and the piece names are the costume.** Both were
fixed by moving pieces rather than by moving numbers, which is upstream's rule
— *monsters wear the catalogue* — arriving from the other side.

### 15. `PlaceKind::Door` has one user left, and it is the ending

M14.3 turns the door under the lake into a **gate** onto the Silt Stair, so the
one `Door` in the game stopped being one. M14.4 puts the ending screen on the
Undercountry, which gives the kind a user again.

**M14.6 candidate**: if the ending is the only `Door` in the game for ever, the
kind is a `Gate` with a screen attached and it is worth asking whether it earns
a variant. Not this block — deleting a `PlaceKind` is a save-format question.

### 16. The barrel showed one thing and sold another

Reported from play mid-block. The screen drew `Game::barrel_now` — rolled if
rolled — and the shim's `buy_barrel` looked the index up in the **authored**
list out of `shops.json`. `Game::order` has asked the same question correctly
since it was written; the ledger's buy went into core with M12.6's rerolls and
the barrel's stayed in the shim.

**A rule decided in the shim is a rule the fast suite cannot reach**, and this
one sat one function away from its own twin for a whole block.

**Status: fixed, with the reporter's sentence as the test.**

### 17. The suite has *not* slowed, and measuring said so

Written first as *"`cargo test --workspace` was 34 seconds at M13 and is minutes
now"*, with twenty maps and `data::map` inside loops as the diagnosis. Measured:
**823 tests in 27.5 seconds**, and the ten slowest files are the ten that were
slow at M13 — `drops` at 11.0s and `experts_reach` at 6.3s, neither of them
M14's. Nothing this block added is above 0.4s.

What is minutes is **rebuilding sixty test binaries after a change to
`combat.rs`**, which is a fact about editing the engine and not about the suite.

The row is kept rather than deleted because the wrong version of it was about to
become an M14.6 candidate, and *the fix for a number nobody measured is a day
spent on the wrong file*. `lake.rs::every_drain_names_terrain_that_exists` does
load twenty maps per drain and it costs 0.05s.

### 18. A stack gate that wants an instrument never opened the frame

`wants_instrument = p.to.clone()` in the shim, which was right while the only
survey gate in the game opened onto **one** map. The lip of the Wextreen Sump is
a *stack* — four floors and no `to` at all — so `to` was `None`, the page had no
map to read the trade against, and **the one door in the game whose answer you
may be carrying the parts for printed a refusal and stopped.**

`p.opens_onto(&g.world)` is core's answer to *which map is through it now*, and
it is the one the display should have been asking all along.

**Found by the browser gate.** `cargo test` cannot see a screen that did not
open, and there is no core-side assertion that would have caught it: the gate's
refusal, the instrument list and the frame's contents were all correct.

**Status: fixed in M14.5.**

### 19. The world the page is holding, one step along

Answering an event raises a flag; a flag is what a `hidden_until` reads; so the
third turn of the chair opens the door in the north wall — **and the page went
on drawing an empty room.** `paintPanel` re-reads the world only when the *map
id* moves, which is right for every path that carries you somewhere and wrong
for this one.

This is the third instance of one rule, and the rule has not changed: **a page
that draws a world has to be told which world, every time it can have changed.**
The stale map was a defeat carrying you to another map; the world-the-page-is-
holding was a save being restored; this is a choice being taken.

Also found by the browser gate, and it is the same sentence as row 18: nothing
in `cargo test` can see a place that is there and not drawn.

**Status: fixed in M14.5.**

### 20. Two gate checks that read the right thing off the wrong element

Both caught by the negative pass rather than by the run:

- The lip's refusal was read off the **strip**. A gate that wants an instrument
  goes through `openKit(wants_instrument, shut)`, so the sentence is on
  `#instrument-shut` and the strip says nothing — and the Reach's own check has
  read it off the frame since M11.6.
- The empty shelf was counted as `#shelf .ware`. That is the *packing screen's*
  class; the shelf draws buttons. The count came back zero on a shelf with a
  book on it, and only the second assertion — that an empty box says *"Nothing
  for sale here"* rather than being blank — reported the fault.

**A check that reads its answer off the wrong element is green on a broken
build**, which is the compares-zero-with-zero family again and is why the
negative pass is not optional.

### 21. `PlaceKind::Door` keeps its user, and §1.5 was wrong about where

§1.5 says *the town's prose says the writing stops here*. **A town has no prose
field and never has** — a `TownShelf` is an id, a stock list and a commission
list. So the stop-line is on a `Door` one tile south of the counter, which is
the kind the game already has for a screen that is not a loop and which M14.3
had just left without a user.

Row 15's question is answered by the content rather than by deleting a variant:
the ending screen is a `Door`, there is exactly one, and it is on the last map.

---

## M14.6 — the notebook, executed

The last milestone reads the rows above and turns the ones that are worklist
into work. This is the convention M13.9 established and the reason `CLAUDE.md`
says to keep the notebook: seven of that block's rows were candidates, all seven
were executed, and three of them turned up something that was actually wrong.

**Four rows are candidates here**, and they are the four that name something
still true rather than something already fixed:

| row | what it asks | milestone |
|---|---|---|
| 15 | `PlaceKind::Door` has one user — does it earn a variant? | M14.6a |
| 17 | the suite's runtime, and the row was wrong about it | M14.6b |
| 6 | the instrument set off certainties — count it end to end | M14.6c |
| 20 | two checks read the right thing off the wrong element | M14.6d |

And two the block found and did not have a place to put:

| | what it asks | milestone |
|---|---|---|
| new | **`solvable_blind` is not run over every floor in the game** — six floors are checked by hand in two files, and the seventh, eighth and any future one are checked by nobody | M14.6e |
| new | **the walker takes the first enabled choice**, which is a trap at any card that comes back | M14.6f — done in M14.5's walker work |
