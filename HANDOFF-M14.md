# HANDOFF-M14.md — down twice, and the country under the country

*`PLAN-M14.md` is the frame. This is the block's own record: what each milestone
found, and why the divergences are what they are. `SECOND-ORDER-M14.md` is the
notebook — twenty-one rows, written when noticed — and M14.6 is that notebook
executed.*

*`CLAUDE.md` is the current account of the repo. Read that first if you have
never seen this.*

---

## 0. State in one paragraph

**All seven milestones are done.** The suite is green at **831 passing**, and
the browser gate walks **all three engines** with five new checks in it —
**63 `ok:` lines to 78**. Twenty maps, nine of them new.

**It is committed, pushed and live.** `255fd55` on `main`, deployed at build
`7d9a9bad`, and verified against the deployed page: **seventy-eight gate checks
in three engines**, and both dungeons walked end to end by `make play` from a
start line.

**Every number in `PLAN-M14.md` that could be measured was, and most of them
moved.** Three of the six blind-solution ceilings were guesses written before
anybody walked the floors; one — the Cairnfield's forty-five — came back exactly
right, which is the reason to believe the other five.

---

## 1. The milestones

| # | Milestone | Status |
|---|---|---|
| M14.0 | **The primitives** — `Requirement::Surveying`, `LooseItemOfSize` and `AssembledOfRarity` **ported from the dead type**, `Outcome::GiveUp`, `TileEvent::repeats`, `Drain.tiles`, `PlaceDef::hidden_until_all` and `needs_all`, `tide` and `silt`, `puzzle::solvable_blind`, `no_flag_is_waited_on_forever` **written rather than extended** | ✅ 802 |
| M14.1 | **The Low Water** — the Treyway 16×16 → 16×26, four places, the tide, four stubs; **three narrow lints were one lint** | ✅ 806 |
| M14.2 | **The Wextreen Sump** — four floors, three puzzles, the Ninth Surveyor; the plan's three counts were three fictions | ✅ 814 |
| — | **The barrel showed one thing and sold another** — reported from play mid-block | ✅ |
| M14.3 | **The Silt Stair** — four floors, the chair, the two chains, What Marbulon Faced Away From | ✅ 823 |
| M14.4 | **The Undercountry** — one town, empty and declared; `Requirement::All`; `Flag` reads `marks` | ✅ 828 |
| M14.5 | **The gate** — five checks, three engines, and **both of the things they found were real** | ✅ 78 `ok:` |
| M14.6 | **The notebook executed** — six floors became every floor | ✅ 831 |
| M14.8 | **Two dungeons walked end to end** — and three of the four things that stopped the walker were the walker | ✅ 832 |

---

## 2. What the plan got wrong, in order of what it cost

### 2.1 It is written against the dead `Requirement` — the largest one

`PLAN-M14.md` §1.1 names `LooseItemOfSize`, `AlignedItems` and
`AssembledOfRarity` as *"locks a player opens by packing"*, and §4.1, §4.2 and
§5.1 hang three doors on them. **All three are on `event::Requirement`**, which
is the cut campaign's type: `Copy`, `&'static str`, not deserialisable. The type
the game uses has four arms.

This is `CLAUDE.md`'s own warning arriving on schedule — *grep for it, and then
check which of the two you found* — and M14.0 grew by two ported arms and an
outcome because of it.

**`AlignedItems` could not be ported at all.** Its doc says *"assembled items
sharing one alignment word"* — upstream's naming system, where a generated item
name carried a word other items could share. The fork kept `naming.rs` and
nothing exposes an alignment word on an assembled item, and `PieceKind::Alignment`
is a *component kind* wearing the same noun. `AssembledOfRarity` is the plan's
own third named lock and reads the same live board.

### 2.2 Three of six blind counts, and the reason each moved

| floor | plan | measured | what it actually charges |
|---|---|---|---|
| the Lip | 10 | **8** | 12 fatigue, or a compass |
| the Shelf | 11 | **1** | a component, or an atlas, or an epic item |
| the Cairnfield | 45 | **45** | forty-four extra card reads, or a golem |
| the Landing | 2 | **1** | a 1×4, or the lens you were given |
| the Chair Room | 27 | **3** | three moves in an order |
| the Gallery | 3 | **3** | the walk round, or a wading set |

The Shelf's eleven is *"a player cycles their tray"* — **there is no cycling**.
A footprint requirement is met or it is not and the card says which.

### 2.3 The chair cannot be nine

§5.2 asks for nine flags — face, four, back, three times — and **nine rungs is
not expressible in monotone flags over three always-offered labels**, which §1.1
is what says. A choice carries one requirement and one outcome, so *"Turn it to
face the door"* raises one flag; to be the first, fourth and seventh move it
would need three. Nine choices puts the answer on the card as a list of labels;
a counter with a modulus is a flag that goes down.

Three moves, and the fiction survives: the sequence is already written on
Marbulon's own door on the first map, and she does it three times because she is
nervous.

### 2.4 `hidden_until_all` cannot show you a refusal

§4.4 and §5.4 put the two bottom doors behind `hidden_until_all` and §1.6 says
finishing the first dungeon shows *"a sealed door with the other dungeon's name
in the refusal"*. Those are two different fields. `needs_all` is the second one,
and `Game::sealed_because` is the sentence.

### 2.5 Two things §3 and §1.5 ask for that the engine has no shape for

- **`Outcome::Counter`**, planted for nothing to read: that is `Outcome::Xp`
  deliberately. Cut.
- **the town's prose**: a `TownShelf` is an id, a stock list and a commission
  list. The stop-line is on a `Door` one tile south of the counter.

### 2.6 A third choice on Marbulon's card would reach nobody

Her event is spent the moment you take either of her errands, and her errands
are the questline that unlocks the Cave. The answer is the gate's own paragraph.

### 2.7 `the_ninth_surveyor_is_beatable_by_the_walker_at_22`

**A level-22 board is not a board this game produces** — the shipped transcript
ends at fourteen. `common::geared_from` is what M11.7 established as *the board
a player actually has*.

---

## 3. What was measured, and the two findings worth keeping

### 3.1 A rating predicts nothing about whether a fight is winnable

Measured against `common::geared_from`:

| it beats | it loses to |
|---|---|
| Sootmother 1670, Anvilheart 1803, The Last Light 2031, **Francis 2958** | **Cairn Chorus 1141**, The Tallow Saint 1223, The Ground Floor 1507, Gilt 2489 |

It beats a 2958 and loses to a 1141. What decides it is **damage per second** —
Cairn Chorus deals 206 and kills it in thirteen seconds; Sootmother deals 19.7
and loses. §4.4's *by DPS bracket, never by adding to Sootmother's* is right and
this is the measurement behind it.

### 3.2 What a creature rates is mostly how many items its board makes

Two bosses, two drafts, both wrong the same way and neither about a number:

- The Ninth Surveyor's first weapon grid was a hilt and two accessories, which
  **assemble nothing**. 7.8 damage a second — and exactly 7.8 at strength 152
  and at 320, because strength pays a swing and there was no swing to pay. A
  sweep of seven healths against four strengths came back Victory in all
  twenty-eight.
- What Marbulon Faced Away From's first board made **three** items where the
  other makes eight, because a hilt, a key and a charm in one row touch and
  merge.

**The coordinates are the dial and the piece names are the costume.**

---

## 4. What the browser gate found, and neither was findable in `cargo test`

1. **A stack gate that wants an instrument never opened the frame.**
   `wants_instrument = p.to.clone()` — right while the only survey gate opened
   onto one map, and the lip of the Sump is a stack with no `to` at all. The one
   shut door in the game whose answer you may be carrying the parts for printed
   a refusal and stopped.
2. **The world the page is holding, one step along.** Answering an event raises
   a flag, a flag is what a `hidden_until` reads, so the third turn of the chair
   opens the door — and the page went on drawing an empty room. Third instance
   of one rule: *a page that draws a world has to be told which world, every
   time it can have changed.*

---

## 5. The divergences, for `CLAUDE.md`'s table

| § | Divergence | Why |
|---|---|---|
| 1.1 | **`AlignedItems` is `AssembledOfRarity`.** | The alignment-word machinery was the campaign's naming system and did not survive the fork. |
| 1.2 | **The counts are 8 / 1 / 45 and 1 / 3 / 3.** | Measured. The Cairnfield agrees exactly. |
| 3 | **`Outcome::Counter` is cut and the marker is an examinable.** | A flag nothing reads is `Outcome::Xp` planted on purpose. |
| 4.1 | **The wheels sit below the channels they open, and wheel C is the redundant one.** | The first draft put wheel B one step from the arrival tile and hung the stair behind all three. |
| 4.2 | **The Shelf's third way is an epic assembled item, not `AlignedItems(2)`.** | Epic and not rare because the board a player actually has holds exactly one epic item — measured. |
| 4.4 | **`needs_all`, not `hidden_until_all`, at the two bottoms.** | §1.6 wants a refusal, and a hidden door cannot show one. |
| 5.2 | **The chair is three moves.** | Nine is not monotone over three always-offered labels. |
| 5.3 | **The wading shortcut is drawn and saves eight tiles.** | §9 decision 4, answered by measurement: seventeen round, nine across. |
| 6 | **Marbulon's third answer is the gate's paragraph, not a choice on her card.** | Her card is spent the moment you take either errand. |
| 1.5 | **The stop-line is on a `Door`, not on the town.** | A town has no prose field. |
| M14.2 | **`the_ninth_surveyor_is_a_fight_the_board_wins`.** | A level-22 board is not one this game produces. |

---

## 5a. `make play`, and the four things that stopped it

**The new-game walk plateaus at level eleven against the Drambus Stack's fourth
floor** and never reaches one thing M14 added. That is `PLAN.md` §6d row 3 —
*a walker with a destination stops being a player* — and it is kept as
`m14-a-new-game.txt` because a transcript that stops is still a transcript.

`GM2D_FROM` is what walks the rest: **a start line, not a plant.** `drive.py`
plants a save and asserts about the state it planted; this loads one before the
first press and then it is the same walk. The files are written by
`crates/core/tests/start_lines.rs` out of `common::geared_from`, and
`the_start_lines_open_and_stand_where_they_say` is what stops them going stale
the way a checked-in save silently does.

With it, **both dungeons walk end to end**:

- `m14-the-sump.txt` — four floors, three puzzles solved by three different
  routes (the wheel that costs a component, the epic-item door, the clipboard
  shortcut), and **THE ONE WHO WENT DOWN** beaten at 2053 with all three drops.
- `m14-the-silt-stair.txt` — the groove, **the chair in three moves**, both
  chains, and **WHAT SHE FACED AWAY FROM** beaten at 2123 with all three drops.

Getting there found four things, and **only the first was the floor**:

1. **The floors were floored with scrub.** 68 tiles at 140 per mille against a
   pool this strong is a fight every four steps; a level-twenty board wore to
   the sixty percent cap crossing one room. Every other dungeon in this game is
   road at 30 — the Cave, the Stack, the map under the lake are all cut
   passages. **This is the one `PROMPT-M14.md` said to watch for**, and it was
   right: the floor was wrong and the walker was not.
2. **A lever changes what is reachable, so a bar goes stale.** Right for a
   crossing, which refuses on what you *are*; wrong for a sluice.
3. **A card that opened is not a tile with nothing on it.** An event opens when
   you walk *onto* it, so standing on the chair after answering looked exactly
   like standing on nothing — and the walk climbed back up four floors. Six
   times.
4. **The first enabled choice is a trap at a card that comes back.** The chair's
   first move has no requirement and stays live for ever.

## 5b. And then the map was the wrong shape

Reported from play after the block shipped, with the design call attached: the
shore should be **its own map**, reached the same way, over the land bridge —
because *"the resolution for the overworld looks all messed up now"*.

**The report is about M14.1's map and the fault is older than the game's second
map.** `fitMap` sizes the canvas's *backing store* to the grid and has since M8;
`#map` pinned the *displayed* size to a 640-pixel square, so the browser scaled
a non-square backing store to a square box on both axes independently. Every
non-square map has been drawn at the wrong aspect ratio since the Great Gear
Cave — nobody noticed because all of them were *wider* than they were tall, and
a nine-by-five room stretched to a square still reads as a room.

Three changes, and they are three kinds of thing:

1. **The split**, which is the design call. The Treyway is 16x16 again;
   `the-low-water` is its own file at 16x11; the bar is a **gate** on one tile
   of `tide` rather than two tiles of the same grid. Better than it was even
   without the rendering — a shore you cross *to* is a place, and a shore drawn
   on the bottom of somewhere else is a suburb.
2. **The CSS**, which is the fault underneath and which the split alone would
   have left in place for the Cave and the map under the lake.
3. **The shore's own ground.** It was open scrub at 140 per mille under a pool
   whose mean rating is twelve hundred — 350 after the danger multiplier, one
   step in three — and its pool held Cairn Chorus, which is the creature
   `common::geared_from` loses to. `make play` crossed it twenty-eight times,
   was beaten on twenty-seven, and never got down the hole. It has a road down
   the middle now and the pool is four, which is what every other approach in
   this game looks like.

`wading_does_not_move_a_place_or_a_region` had to learn one thing from it:
**ground the world opens is not ground a set opens.** The check it exists for —
a place three players in four never find because it is behind a rule they did
not know to build for — is untouched.

## 6. What is left

**Nothing in the block.** Seven milestones, the suite green, the gate green in
three engines, and the walker's transcript in `testing/transcripts/`.

One thing is the human's and is not the builder's:

1. **`PLAN-M14.md` §9 decision 1 — the third town's name.** It ships as *a town
   with no name on the post yet*, which is what the plan says to do if
   unanswered, and it is true and in register. `common::UNWRITTEN` is where the
   emptiness is declared.

And `PLAN-M14.md` §9 decisions 2, 3, 4 and 5 are answered in the commits that
answered them: the Undercountry is drawn plainly and §9.2 is left where it was
found, `built-the-tenth` is the tide's flag as recommended, the wading shortcut
is drawn and saves eight tiles, and the two bottoms make a golem out of
certainties.
