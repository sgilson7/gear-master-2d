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
