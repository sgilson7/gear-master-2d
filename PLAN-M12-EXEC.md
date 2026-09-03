# PLAN-M12-EXEC.md — the execution plan

`PLAN-M12.md` is the frame, written 2026-08-31 while M11 was in flight. **This
is the execution plan and it wins where the two disagree**, the same way
`PLAN.md` wins over `PLANNING-BRIEF.md`. Everything the frame decided stands
unless a row in §7 says otherwise and says why.

Written 2026-09-03, with M11 live at `6b7abe3e` and 630 tests green.

**Six milestones, five deploy points.** The frame has five and four; the sixth
is **M12.5 — events that pay something and say what they pay**, added on the
human's ask and argued for in §2 rather than smuggled in.

---

## 1. What changed between the frame and now

The frame assumed M11 might slip. It did not: M11.0 through M11.9 are live,
and three things it could not know are now facts.

- **The log exists and is load-bearing.** M11.0 landed, so the frame's "every
  sentence goes through `log()`" is not a dependency any more, it is the house
  style. Good: this block adds a lot of sentences.
- **The catalogue is 568, not 536**, and the block's seam arithmetic changes
  with it. M11 took two seams (544 → 550 → 568). §7 row 1 hardens the frame's
  recommendation from *preferred* to **binding**: M12 adds no components. A
  third and fourth fingerprint move inside two blocks is not rude, it is a
  player's save refused twice in a fortnight.
- **`data/` grew a maps directory and nine maps.** The frame's "no new maps"
  holds and gets easier — every lever in this block is economy, not geography.

And one thing the frame could not have known because it was found today:

- **The block's own thesis has a third faucet nobody costed.** Cells outnumber
  pieces; the frame's answer is a barrel and a ledger. But the game already
  has 56 event tiles, **47 of which pay nothing at all**, and events are the
  one content type that can hand over a piece for free at exactly the moment a
  frame has a hole in it. That is M12.5, and it is a throughput lever wearing
  a content coat.

## 2. Why the events milestone belongs in this block

Two asks arrived together and they are the same ask.

> *"do they hint at some puzzle? or did you increase interactions by
> functionally doing nothing and populating the area with worthless text?"*

> *"we also need to improve the rewards for events across the board, currently
> they are just fnorp and experience which sucks, and you cant predict what
> youre gonna get."*

**The measurement, taken rather than remembered:**

| | |
|---|---|
| event tiles in the game | 56 |
| that offer a choice | **9** — 7 on West Bambulon, 2 on the Treyway |
| that are prose and nothing else | **47** — 41 on the Kettleworks field, 6 on the Reach |
| of those 47, that do anything at all | **6**, and only as errand waypoints — a `word` goal you stand on |
| distinct outcome kinds used, across 21 choices | 4: `xp` ×20, `flag` ×18, `gold` ×5, `give` ×2 |
| events that open another event | **0** |
| choices that say what they pay | **0** |

So the honest answer to the first ask is: **no, they do not hint at a puzzle.**
There are no flags on them, no `hidden_until`, no cross-references and no
chain. The nouns they share are "Stack", "Kettleworks" and "Somebody". Six of
the forty-seven carry an errand waypoint and the rest are prose. A map whose
texture is *reading things* is a legitimate content decision — it is the one
`CLAUDE.md` recorded and congratulated — but texture only survives contact
with a player if something on the map rewards having read it, and nothing
does.

It belongs in **this** block rather than M13 for a reason that is about
pressure and not about tidiness: an event that hands over a component is
piece throughput, which is the block's whole subject. The barrel fills frames
with junk you *buy*; a commission fills one with the thing you *chose*; an
event fills one with something you did not expect and did not pay for. Three
faucets, three feelings, one thesis. And the third is the only one that can
put a piece in your hands at the exact moment you are standing somewhere
interesting.

**The recon says most of it is content, not engine.** `Outcome::Give(name)`
hands over a component and has since M2. `Outcome::Flag` sets a world flag and
`PlaceDef::hidden_until` reads one, which is how M8.7's door in the wall
works — so **an event chain has been possible in the data format for four
blocks and has never once been used.** What is missing is three outcome kinds,
one derived describer, one box on a card, and content.

### The outcomes box, and why it is a port and not an invention

The ask is *"maximal mechanical clarity, much like the clarity of the skill
tree"*, and that is exactly right, because the skill tree already solved this
and the solution is written down as `TONE.md` rule 13a:

| | written by | speaks |
|---|---|---|
| `label`, `blurb` | a person, in `data/events.json` | the book |
| **`Outcome::describe()`** | **derived in core from the outcome** | the engine |

Derived, never typed — a spec nobody writes by hand cannot disagree with the
thing it describes, so retuning an outcome retunes its box. Unthemed, because
somebody choosing between two halves of an event is comparing numbers and a
number wearing a joke has to be translated first.

**And the describer already exists on the wrong type.** `event::Outcome` — the
cut campaign's, still in `event.rs` — carries a full `describe()` whose doc
comment is this design verbatim:

> Static: what this outcome *is*, for a tooltip before it is taken. What it
> *did*, with the run's own numbers in it, is `Run::receipt`.

`tile_event::Outcome`, the one the game uses, has none. So M12.5 ports a
pattern rather than inventing one, and `CLAUDE.md`'s *Two types called
Outcome* records the trap for whoever greps next.

---

## 3. The shape of the block

```
M12.0  the measure               (no push — instrumentation)
M12.1  the bargain barrel        deploy point A
M12.2  commissions               deploy point B
M12.5  events that pay           deploy point C   <- new, and it moves
M12.3  slower cells              deploy point D   (scope set by the measure)
M12.4  triage, the friend, close deploy point E = the block ships
```

**M12.5 goes third, before the cells.** The frame's ordering argument is that
throughput comes before the schedule change so M12.3's scope can be decided by
evidence. Events are throughput. Putting them after commissions and before the
cells keeps that argument intact and strengthens it: the probe that decides
whether M12.3 ships at all is then measuring a game with **all three** faucets
open, which is the game the human would actually be judging.

Putting it last instead would mean deciding the cells question against a game
missing a third of its piece supply, and then discovering the answer had
changed. Putting it first would mean authoring event rewards before the barrel
exists, and every reward would be priced against a game that is about to move
underneath it.

---

## 4. The milestones

M12.0 through M12.4 are the frame's, executed as written except where §7 says
otherwise. Only their deltas are restated here; **read `PLAN-M12.md` §3 for
the deliverables and acceptance of those five.** M12.5 is written in full
because it is new.

### M12.0 — the measure *(unchanged)*

The probe, the target curve (fill ≥ 70% by level 3, ≥ 80% by level 6, bench
depth ≥ 2 by level 5), and a committed baseline. No push.

**One addition.** The probe reports a fourth number: **pieces acquired, by
source** — shelf, barrel, commission, event, drop, quest. The block opens
three faucets and closes by claiming pressure moved; a claim about throughput
that cannot say *where the pieces came from* is a claim nobody can act on when
the next block wants to tune one of them.

### M12.1 — the bargain barrel *(unchanged, seam now binding)*

As the frame. §7 row 1 makes "reuse the catalogue" binding rather than
recommended, so **this milestone can no longer be a seam.** If the grep cannot
find 10–14 usable smalls, the barrel ships smaller — a nine-piece barrel is a
barrel, and a fourth fingerprint move in two blocks is not.

### M12.2 — commissions *(unchanged)*

As the frame: the ledger in `data/shops.json`, the clock in fights,
`WorldState::commissions` destructured in `save.rs` with an empty default, one
open order per town, and the voice.

**One addition, from a rule this project learned the hard way.**
`every_commission_reaches_something` — the `every_offered_class_reaches_
something` shape, and it must **call rather than declare**: place an order,
tick the clock, collect, and assert the piece is in the bag. A lint that reads
the ledger and says "yes, that piece exists" is the failure it exists to
catch, one level up.

### M12.5 — events that pay something, and say what they pay *(new)*

**Goal.** An event stops being a paragraph with a dismiss button and becomes a
decision with legible stakes and a real payout — and the field stops being
forty-one things that happen to you for no reason.

#### 5a. The outcomes box

- **`tile_event::Outcome::describe(&self) -> Vec<String>`**, in core, derived,
  unthemed, one line per concrete delta — ported from `event::Outcome::
  describe`, which is the same function on the cut campaign's type.
- The shim puts it on each choice as `outcome: [String]`; `showCard` grows a
  third element under the blurb, styled as a spec and not as prose. The board
  already has a visual language for *this is the engine talking* — the skill
  node's `line()` — and the box reuses it rather than inventing a look.
- **`no_mechanical_line_speaks_the_theme` extends over it.** That lint exists
  and already guards exactly this text on skill nodes; an outcomes box that
  said "you feel the Nut Freeze lift" would be the same bug in a new room.
- **A requirement is one condition and an outcome is however many things
  happen**, which is why the campaign's `describe` returns a `Vec` and its
  `Requirement::describe` a `String`. Inherited distinction; keep it.
- **`Requirement::describe` is the second half of the port, and it is the more
  useful half.** Its doc says why, and it is a distinction the live type has
  never had:

  > Not the same thing as `Choice::unmet`, and both are needed. `unmet` is
  > flavour written for the moment after you have tried; this is the plain
  > statement *before* an attempt.

  So a locked choice gets two lines and they do different jobs: **what it
  wants** (unthemed, derived, before you try) and **what it says when you
  try** (the author's, in voice). Today it has only the second, which is why
  a refusal reads as a wall.
- **Every choice shows its box, including the ones you cannot take** — what it
  wants *and* what it would pay, or a locked choice is a dead end instead of a
  target to come back to.

#### 5b. Three new outcome kinds, and no more

Held to three, and each has to earn its arm in a match that is exhaustive
everywhere it is consumed.

| kind | what it does | why it is not something that exists |
|---|---|---|
| `Supply { id, n }` | hands over restoratives | `Give` is components only; a tin is not a component and never has been — no shape, no grid, spent rather than worn |
| `Tire(pct)` | costs fatigue | the only currency the road has, and without a cost an event is a vending machine. Positive only; a *negative* tire is a tin and tins are bought |
| `Warp { map, at }` | puts you somewhere else | the human's ask, and the one genuinely new verb |

Deliberately **not** added, each for a stated reason:
- *An ench.* `Quest::enchs` already pays one and the rule that an ench is not
  a component is why it is a separate field there; an event wanting one is a
  sign it should be an errand.
- *A skill point.* Points come from levels and levels come from banking. A
  faucet outside that loop is a second answer to "what is a point worth".
- *A row.* M12.3 is about rows being earned; a row from a card is the lottery
  ticket that milestone exists to refuse. Said out loud because it will be
  tempting.

#### 5c. Warp, and what it costs

The ask names it: *"weird events like being teleported somewhere you shouldn't
be early, like the lake cave."*

**This is coherent with M11's design rather than a violation of it**, and the
reason matters. M11.4 already ships an early way under the lake — the Toad's
Own Frame lets you wade out to the grating before the tower falls — and it
already answered what an early entry costs. It is not a harder fight, because
combat has no board and a position cannot cost anything. **It is fatigue**:
entered early, the map's own middle rows are still flooded and the way down is
twenty-one tiles of slag against eleven of road.

A warp inherits that answer exactly. It puts you somewhere out of order; the
map you land on charges you the walk. Rules:

- **A warp is one way and never a shortcut home.** It moves you *out*, never
  back. `Rule::Homeward` is the thing that takes you home and it costs a tin.
- **It lands you somewhere `World::repair` agrees with**, allowances in hand,
  and `every_warp_lands_somewhere_you_can_stand` is the check — the exact
  shape of `every_gate_leads_somewhere_you_can_stand`, which exists because a
  gate whose far side is a wall strands a player and nothing else would say so.
- **It writes `remember_at` on the way out**, like every other crossing, so the
  map you left knows where you were standing.
- **The box says so in advance, in the plainest words the game has.** This is
  the one outcome where surprise is the content and clarity is still not
  optional: *"You are put somewhere else. It is a long walk back."* Naming the
  destination is the human's call — §7 row 5.
- **It is rare and it is authored.** Two or three warps in the block, not a
  new furniture type. A map that teleports you every tenth tile is a map you
  stop walking.

#### 5d. Chains, which cost no engine at all

`Outcome::Flag` sets a flag; `PlaceDef::hidden_until` names one; `Requirement::
Flag` gates a choice on one. **All three have shipped for four blocks and no
event has ever used them together.** So a chain is content:

- A choice sets a flag. A second event's tile is `hidden_until` that flag, or a
  second event's *choice* requires it.
- **Three chains, two to four steps each**, at least one of them running
  through the Kettleworks field so the field's density starts paying.
- **A chain's last step pays a component**, because a chain that pays Fnorp is
  a longer way to earn Fnorp.
- **`every_flag_an_event_sets_is_read_by_something`** — the lint, and it is
  the `every_ench_comes_from_somewhere` shape. Eighteen `flag` outcomes exist
  today and this check will fail on its first run, which is the point; the
  ones that turn out to be orphans get read or get cut.

#### 5e. The forty-one

Not deleted. The prose is written and it is good, and deleting it to make a
metric move would be the wrong lesson. But a category needs a name and a
budget:

- **A `note` is a legitimate event and stays one** — read once, then quiet,
  which is now true as of the M11 follow-up. It is the map's furniture.
- **A `note` must be outnumbered by decisions on its own map, or it is
  wallpaper.** `a_map_is_not_mostly_wallpaper`: on any map, events that ask
  something ≥ events that do not. Kettleworks field today is 0 vs 41 and will
  fail on its first run, as it should.
- **Meeting it is content, not deletion.** Roughly twenty of the field's
  forty-one gain a choice and a payout; the rest stay notes, and the map ends
  around twenty-one decisions to twenty notes. The Reach's six are six notes
  on a map with no decisions at all and get three.
- **The six that are errand waypoints keep their job** and are the model for
  the rest: a tile worth standing on is a tile something asked you to stand on.

#### 5f. What the nine existing events get

They are the game's oldest events and they are all `xp` and `flag`. Each gains
an outcomes box for free (derived), and their payouts are re-cut so that **at
least three of the nine hand over a component**, because those nine sit on
West Bambulon where a level-two frame has the most holes in it and the block's
thesis is loudest.

**Acceptance.**
- Every choice in the game shows a derived outcomes box; the lint refuses a
  themed one; a greyed choice shows both its refusal and its payout.
- The three new outcome kinds each round-trip a save and are each honoured by
  a test that *calls* rather than reads.
- Every warp lands somewhere standable on every map, checked over all eleven.
- Three chains complete end to end in a seeded run; no flag is set that
  nothing reads.
- `a_map_is_not_mostly_wallpaper` green on all eleven maps.
- The probe shows **pieces-from-events > 0 by level 4** and overall fill up
  from the post-M12.2 reading.
- Browser: an event card's box matches what core said it would pay, and what
  the receipt says afterwards matches the box — read, never recomputed, which
  is the page's oldest rule and the one an outcomes box is most likely to
  break.

### M12.3 — slower cells *(unchanged, gate now measures three faucets)*

As the frame, including the retirement of `PLAN.md` M4's row-per-level, rows
as skill nodes, rows as quest-line rewards, the per-slot ledger, and the bake
of old saves. §7 row 4 restates the frame's gate zero with the events counted.

### M12.4 — triage, the friend, and the close *(unchanged, one addition)*

As the frame: `TRIAGE-M12.md`, the curve diff, the agent spot-run, the
friend's verdict verbatim.

**The spot-run gains a fifth errand**, and it is the one this block's newest
milestone most needs a stranger for: *read three events and say, before you
choose, what each half will give you.* An outcomes box that a builder can read
is not a box that works.

---

## 5. Deploy points

Every milestone passes the standing gate — `make test`, `make test-ui` in
three engines, `make play` read by a person. Five push, on the human's word,
`git log origin/main..HEAD` checked first, and the live check is
`GM2D_ORIGIN=… drive.py` now rather than a table somebody types.

| point | after | a visitor can | note on the page |
|---|---|---|---|
| **A** | M12.1 | fill their frames from the barrel, cheaply, today | **no seam** — §7 row 1 is binding; old saves sail through |
| **B** | M12.2 | order a piece and fight their way to it | old saves load; the ledger explains itself on first sight |
| **C** | M12.5 | read an event that says what it pays, and be paid in gear | old saves load; a chain half-finished before the update stays finishable |
| **D** | M12.3 | earn a row, with a point or off a quest line | old saves keep every row, baked as earned; the note says levels no longer grant rows |
| **E** | M12.4 | the block, triaged, measured, friend-tested | the curve diff and the friend's verdict |

**No seam anywhere in this block**, which is the difference between this plan
and the frame: M11 spent two and a player's saves have been refused twice
already.

## 6. SECOND-ORDER-M12.md — the entries this block starts with

The frame seeds four. M12.5 adds three, and the third is the one to watch.

5. **An event that pays gear moves the same floor the barrel moves** (M12.5).
   Regions and crossings are rated against a level's expected board, and the
   block now has three faucets feeding it. Watch: the golden fixture at levels
   1–4, and whether the level-5 crossing stops being a wall. The knob is what
   events pay, not region ratings — the same knob the barrel has, which means
   **the two can mask each other**, and the probe's by-source counts are how
   you tell them apart. That is what they are for.
6. **A warp is a new way to be somewhere** (M12.5). Every question that has
   ever been answered with "you got here by walking" now has a second answer:
   arrival prose, first-visit flags, `remember_at`, the quest log's guide, and
   `World::repair`'s idea of home. Watch: a warp into a map whose gate carries
   a paragraph — does the paragraph fire, and should it?
7. **An outcomes box is a promise on a screen** (M12.5). This project has
   shipped four promises that reached nothing — `Showstopper`, `Recycler`, the
   ench rack, and event experience — and every one was a screen saying a thing
   the engine did not do. A box is that failure mode with a printing press.
   Watch: the browser check that the box, the receipt and the character's
   actual state all agree, and treat any disagreement as a **fifth** instance
   rather than a rendering bug.

## 7. Divergences from `PLAN-M12.md`

| # | the frame said | this plan says | why |
|---|---|---|---|
| 1 | barrel may contain new pieces if unavoidable (§8 row 1, recommendation: reused only) | **binding: no new components anywhere in M12** | M11 spent two seams. A third and fourth inside two blocks is a save refused twice in a fortnight. A nine-piece barrel is still a barrel |
| 2 | five milestones, four deploy points | **six and five**, M12.5 inserted third | the human's ask, and it is a throughput lever, which is the block's subject — §2 |
| 3 | M12.3's gate zero re-runs the probe after M12.2 | **after M12.5** | the cells question should be decided against a game with all three faucets open, not two |
| 4 | probe reports fill and bench depth | **and pieces by source** | three faucets that can mask each other need to be told apart before anything is tuned |
| 5 | — | **a warp names its destination or does not: the human's call** | naming it makes it a choice; not naming it makes it an event. §8 row 9 |
| 6 | the spot-run has four errands | **five** | the fifth tests the outcomes box on somebody who did not write it, which is the only way that feature can be tested at all |

## 8. The human's calls, before the milestone that needs them

Rows 1–8 are the frame's and stand as written; row 1's recommendation is now
binding per §7. These are the new ones.

| # | needed by | question | recommendation on record |
|---|---|---|---|
| 9 | M12.5 | Does a warp's outcomes box **name where it sends you**? | **no** — "You are put somewhere else, and it is a long walk back" is honest about the cost, which is the part that matters, and keeps the surprise that makes it worth writing. Naming it turns a weird event into a fast-travel menu |
| 10 | M12.5 | How many of the field's 41 notes become decisions? | **about twenty**, leaving the map roughly even. Enough that reading pays; not so many that a dense map becomes a dense quiz |
| 11 | M12.5 | May an event's warp send you somewhere the level gates would refuse — the far side of a crossing, or under the lake before the tower falls? | **yes, and that is the point** — but only where the map itself charges for it, which today means under the lake and nowhere else. A warp past a crossing hands a level-five character a region rated for twelve, and that is a wall, not a surprise |
| 12 | M12.5 | Do the nine existing events keep their current payouts on top of new ones, or are they re-cut? | **re-cut** — three of the nine pay a component instead of some of their Fnorp. They sit where the frame has the most holes in it, and adding on top inflates a curve this block is trying to measure |

## 9. What this block still does not do

The frame's §7 stands entire: **no reroll in any costume**, no harvesting off
beaten creatures, no world-clock restock, no new maps, creatures, sets or
rules, no change to what the shelf itself stocks.

M12.5 adds three of its own:

- **No new components**, which is §7 row 1 and is what makes the block
  seamless. Events pay out of the 568 that exist.
- **No event that pays a row, a skill point or an ench.** Each has a home and
  a reason, listed in §5b.
- **No rewriting the forty-one.** Their prose stays. What changes is that
  twenty of them start asking something, and that the map around them stops
  being only notes.
