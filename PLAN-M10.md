# PLAN-M10.md — where an ench comes from, and one swing that ends a bout

Written 2026-09-02, against `8d1e38f`. `PLAN.md` remains the plan of record for
M0–M7, `PLAN-M8.md` for M8 and `PLAN-M9.md` for M9; this is the plan for the
next block and follows their conventions.

**Four milestones, each deployable on its own.** M10.0 changes where an ench
comes from, M10.1 teaches the fight that an item can break, M10.2 is the class
that is built on the two of them, and M10.3 is the standing "play it" gate.

**Three of §5's questions were answered before a line was written**, and the
answers are folded in below rather than left as proposals: it breaks **for the
fight**; **no town sells an ench**; and what a tree does not award is sold by
**one vendor, who is not there until level 10.** §5 keeps the record.
Every one ships behind `make test`, `make test-ui` in three browsers, and
`make play` read by a person.

---

## 0. The ask, and the four things recon changed about it

> *"enchs should only be obtainable from skill trees and specific vendors, not
> just the town shop once you get the class. like a node on the skill tree
> should give you the ench. draw up a plan for implementing a new class designed
> around a new ench derived from TD, that gives a piece of gear the property
> that the assembled item its a part of is 3x more powerful, but breaks after 1
> activation"*

Four things were measured before this was written. Two of them move the plan and
one of them is a blocker nobody knew about.

### **Showstopper is honoured nowhere, and it is the class this wants**

`ClassPower::Showstopper { pct: 50, under_ms: 10_000 }` — *a fight won in under
ten seconds pays 50% more* — already exists, is already tuned, is already themed
as **Top of the Bill** (Hanglo Chiemstar, p. 31), and its blurb is already the
exact register for a class built on one enormous swing: *"They came to see a
bout. You gave them an incident."*

It is also **wired to nothing.** `combat.rs` ignores it on purpose and correctly
— it is a settlement rule, not a combat one — and `fight::settle` and
`reward.rs` never consult the character's class at all. Offering it today would
cost a player an irreversible choice at level five in exchange for nothing, which
is the failure that cost this project two milestones when eight skill nodes did
the same thing.

**So the first deliverable of the class milestone is not the class.** It is
`reward::bounty_for` learning that a fast win pays more, and a lint that refuses
to offer a class whose power reaches no code. That lint is the one that would
have caught this, and it does not exist.

### **The fork's roster lives in the shim, and in a test, and nowhere else**

`crates/wasm/src/lib.rs` holds `const OFFERED: [&str; 4]`, and
`tests/classes.rs` holds a second copy with a comment saying so out loud:
*"Named here rather than read from the shim, because the shim is wasm and this
is the list it holds."* That is a rule decided in the shim, which decides
nothing, written down twice. A block that adds a fifth class is the block that
has to fix it.

### **A fifth card does not fit, and this was measured a milestone ago**

`PLAN.md` §6a row 3: *"Four class cards fill a 1280×720 viewport. They fit and
they are clickable, measured. A fifth will not, and the fix then is a two-row
grid rather than a shorter portrait."* The fix is written down; this is the block
that owes it. `check_the_fork_is_on_top` measures with `elementFromPoint` and is
what will say whether the grid works.

### **No town benches means three of the Patent's eight nodes are dead**

This one is a consequence of the human's answer rather than of the ask, and it
is the sharpest thing recon found after it.

The Kaklon Patent's tree is eight nodes and **three of them tune the spin** —
`spin_extra`, `spin_every`, `spin_keep`. The spin is not a stat; it is an
*ench*, The Ponkey Turn, and today you buy it off a bench for 90 Fnorp. Take the
benches out of the towns and the Patent is a class you are offered at level five
whose identity, and three eighths of whose tree, does nothing at all until level
ten.

That is not a reason to keep the benches. It is the thing that makes the other
half of the ask load-bearing: **the Patent's tree has to grant The Ponkey Turn,**
and the node it belongs on is already called **Bench Rights** — no prerequisite,
and the root of the spin spine. The node named for the bench hands you the bench.

### **Three quarters of the ench already exists**

`Ench::Effect::Power { pct }` adds percentage points to an item's own power
multiplier, and power starts at 100 — so **`Power { pct: 200 }` is exactly "3×
more powerful"**, with no new arithmetic and nothing to retune. What does not
exist is *breaks after one activation*, and that is a new rule in the fight: the
first this project has added for a class since the fork.

`RunningItem` already carries `has_fired` (Overtake's whole condition) and
`stun_ms` (a per-item stop, checked once in the tick). **The new rule is one
field, one branch beside the stun's, and one event** — which is the smallest new
combat rule this game could be given, and it is worth saying plainly that it is
still a new one.

---

## 1. What is being asked for, in one list

| # | Ask | Where it lands |
|---|---|---|
| A | **No town sells an ench**; one vendor does, and not before level 10 | M10.0 |
| B | An ench comes from a **skill node** | M10.0 |
| C | An item can **break**: one activation, then it is finished | M10.1 |
| D | An ench that is **3× power and breaks** | M10.1 |
| E | A **fifth class** built on it, and its tree | M10.2 |
| F | The class's promise **reaches something** | M10.2 |
| G | Play it, triage it, write it down, ship it | M10.3 |

---

## 2. Ordering, and why this order

- **The source before the thing sourced.** M10.0 changes where every ench comes
  from, including the four that already exist. Doing it after the new ench would
  mean writing the new one's availability twice.
- **The rule before the class.** A class whose identity is an ench that does not
  work yet is a class that cannot be played. M10.1 ships the ench and the break
  with no class attached — obtainable, in the plan's own words, from a vendor —
  so the mechanic gets a player's opinion before a class is built on it.
- **The class last of the features**, because it is the one that cannot be
  taken back: the fork is permanent, and a fifth card is a screen change that
  the gate has to be re-measured against.
- **M10.3 is not a feature.** It is standing since M9.4, and M9.4 earned it
  twice over.

---

## M10.0 — Where an ench comes from

**Theme:** an ench stops being a shelf item and becomes something you were
awarded or went and found.

### The rule today, and why it is being changed

`EnchDef::price` is the whole of availability, and `shop_json` says so: *"Every
trading town keeps one, the same rule the tins follow."* Four of the five enchs
are priced and every town sells all four to any licensee. The fifth is priceless
and is an errand's.

That was right when enching was one class's whole identity and the worry was
stranding a licensee from their own class. It is wrong now for the reason the
shelves stopped rolling in M7: **a thing every town sells is not a thing you went
and got.**

### Deliverables

1. **No town sells an ench.** The bench comes off the town screen and
   `shop_json`; `buy_ench`'s "you are not in a town" refusal goes with it. This
   is a deletion, and it is the milestone's first commit so that everything
   after it is adding sources back rather than moving them.
2. **A node can hand one over.** `skills::Effect` gains a sixth kind,
   `GivesEnch { ench: String }`.
   - **Derived, never banked.** `Character::enchs_owned` is state; a node's
     effect is not. So the character's enchs are `enchs_owned` **plus** what the
     taken nodes grant, computed fresh on every read — exactly the way
     `player_stats` reads the tree and for the same reason: retuning a node
     retunes every save that took it. `Character::enchs()` is that list and
     `enchs_loose` counts against it.
   - **Untaking is not a thing**, so a granted ench cannot go away underneath an
     attachment. If that ever changes, `tidy_enchs` is where it lands.
   - `Effect::line` for it is the ench's own name and spec, so a node says what
     it hands over rather than naming an id.
3. **`Bench Rights` grants The Ponkey Turn.** Not a nicety — see §0. Three of
   the Patent's eight nodes tune a spin that only an ench supplies, and with the
   towns closed the class would be inert from five to ten. The node with no
   prerequisite at the root of the spin spine is the one, and it is already
   named for it.
4. **One vendor, and he is not there until level 10.** A place on the overworld
   that does not exist below the level and sells what no tree awards.
   - **A place, gated on a level.** `PlaceDef::hidden_until` reads `answered`
     and `flags`, which cannot express a level — and writing `level-10` into
     `flags` when it is reached would store a number the game derives from
     experience, which is two answers to one question and is refused by the rule
     that has held since M4. So it reads the level the way a crossing does:
     through `world::Allowances`, which has carried one since M9.3.
     `place_is_there`, `place_now` and `places_now` take it. **Seventeen call
     sites across three files — counted, not guessed.**
   - **It is a bench, not a card.** An event's choices are spent as a set:
     `answer` refuses a second choice and pushes the whole event id into
     `answered`, so a card could sell one thing once. The town bench already
     renders an ench with its spec, its price, whether it is affordable and how
     many you hold, and `buy_ench` already refuses the ways it should. So the
     vendor is `PlaceKind::Bench` and it reuses all of that; what is new is the
     tile it stands on and the level it waits for.
   - **What he sells is content**, in `enchs.json` beside the price or in his
     own list — one of the two, decided in the commit, and not both.
   - **Bought once, and remembered.** `WorldState::bought` is `(town, index)`
     into a shelf; an ench is bought by id, so this is a separate
     `bought_enchs` list of ids. One list answering two questions is how the
     shelf's index rule gets broken.
5. **`Rule::check`'s sibling for enchs.** `SkillsData::parse` refuses a node
   naming an ench `enchs.json` has not got, and the vendor's stock is refused
   the same way. Both are the guard `Rule::check` already is, and both exist
   because *content nobody can reach is the thing nothing else in the game says
   anything about.*
6. **A lint per source.** `every_ench_comes_from_somewhere` walks `enchs.json`
   and refuses one that is on no vendor, in no node and paid by no errand. That
   is the lint this milestone is *for*: the point of narrowing availability is
   that an ench now has a place it comes from, and an ench with none is an
   orphan. Its sibling, `no_ench_is_sold_by_a_town`, is the ask.

### Where the vendor stands, and why it matters

He has to be somewhere a level-ten character can reach and a level-nine one
cannot trivially stumble over, which the map now answers for free: the crossings
opened the Verge at nine, so **the Verge or West Bambulon is where he goes** —
past the last crossing, in the half of the map a player only sees once they have
earned it. That also means the walk to him is a walk, which is the whole of what
"went and found" means here.

### Tests

- `no_ench_is_sold_by_a_town` — the ask, stated as a lint.
- `every_ench_comes_from_somewhere`, and the inverse: nothing is awarded twice.
- `the_patent_is_not_inert_before_the_vendor` — a level-five Licensee who has
  spent one point can put an ench on a component. This is the test that would
  have caught the gap §0 found, and it is the reason the milestone has a
  deliverable 3.
- `a_node_that_names_no_ench_is_refused`, `a_vendor_that_names_no_ench_is_refused`.
- `a_granted_ench_is_derived_and_not_banked`: take the node, assert the ench is
  there; **change nothing in the save and retune the node in the data**, and
  assert the character's answer moved.
- `the_vendor_is_not_there_before_the_level`, and is there after — asserted
  against `place_now`, `places_now` and `step`, because a place that is absent
  from the map and steppable is half hidden.
- `a_save_from_before_the_benches_closed_still_opens`, and a character who
  already bought an ench keeps it.
- `check_the_vendor_appears` (browser): planted at nine, nothing on the tile;
  planted at ten, a bench with something on it.

### Deploy point

After 6.

### Risks

- **Seventeen call sites for one signature.** M9.0 did the same to `world::step`
  and the note it left is the one to follow: change it once, in one commit, with
  nothing else in it.
- **A place that appears is a place the map has to redraw.** `world_json` is
  sent once at startup and re-read on a gate; a place that appears on a *level*
  appears when you bank, which is a moment nothing currently refreshes the map
  for. That is a one-line fix and a very easy thing to miss — the symptom is a
  vendor who is there and invisible until you walk through a gate.
- **The errand's ench is a third source** and stays one. The Yodregar Index is
  paid by THE FRAME THAT STANDS and is on nobody's bench, which is the rule
  M8 set and this milestone does not disturb.

## M10.1 — An item that breaks

**Theme:** the first new rule in the fight since the fork, and the smallest one
that could be.

### Deliverables

1. **`ItemProfile::fragile`**, set from an ench, exactly the way `spins` is.
2. **`RunningItem::broken`**, and one branch in the tick beside the stun's.
   - A broken item's bar does not advance, does not turn and does not fire. It
     is a stun that never ends, and it is written as its own field rather than
     `stun_ms = u32::MAX` because a stun is a curse somebody put on you and this
     is a property of the gear.
   - Set at the end of the activation that fires it, so **the activation that
     breaks it still pays in full**. That is the whole bargain.
   - `has_fired` already exists for Overtake and is not reused: Overtake asks
     "was this the first?" and this asks "is it finished?", and one flag
     answering two questions is how the next person gets it wrong.
3. **`Event::Broke { side, index, item }`**, and the replay draws it. An item
   that silently stops looks like a bug in the playback — this is the same rule
   `Event::Stunned` was given its own variant for, and for the same reason: the
   interface needs to know *which* item.
4. **The ench.** `Ench::Effect::Fragile { pct }` — one effect, not two.
   - **One effect and not `Power` plus a separate `Fragile`**, because the two
     halves are one bargain: an ench that granted the power without the cost
     would be strictly better than everything on the bench, and an arrangement
     that could attach them separately would let a player have exactly that.
   - `line()` says both halves with both numbers, unthemed, TONE 13a:
     *"+200% power to the item this is on, and it breaks after its first
     activation."*
5. **Named from the book.** Proposed: **The Chonga Swing** — Jimmy Chonga's
   brutal gortball strike, p. 41, which the retheme document already cites as
   the source of the Arc Bat Grip. One swing, and the bat is finished. *(TONE
   rule 8: sourced to the book. `PLAN.md` §6.4 is still open, so a name is a
   proposal and not a decision — see §5.)*
6. **The rating knows.** `rating::item_rating` prices an item by what it does
   per second; an item that fires once is worth a fraction of what its numbers
   say. An unrated `fragile` would make a Chonga'd blade the best item in the
   game by the shop's reckoning and would move every rarity mark on the board.
   **This is the risk that will cost a day if it is not taken seriously.**

### Tests

- `an_item_that_breaks_fires_once` — the log has exactly one activation of it,
  and the fight goes on around it.
- `the_activation_that_breaks_it_pays_in_full`.
- `a_broken_item_does_not_turn`, the sibling of the stun's.
- `a_fragile_item_is_not_rated_as_a_permanent_one`.
- `the_golden_fixture_is_unmoved` — nothing seated, nothing different.
- `check_a_broken_item_reads` (browser): the bar stops, and the screen says why.

### Deploy point

After 6, with the Swing on the vendor's bench and no class attached to it yet.
**The mechanic gets a player's opinion before a class is built on it**, which is
the ordering M9 wished it had for the drop rate. It costs nothing to move it
into the Bill's tree afterwards if that reads better.

**It breaks for the fight and not for good** — answered, and the reasoning is
kept because it is what makes the deliverables above small: combat is a pure
function of the board, a mid-fight save carries a creature name and a tile, and
a component destroyed for good would be the fight writing to the character. It
would also be the first thing in the game that takes a component away, with
locks, enchs, Auto-pack and the save all owed an answer. `RunningItem` is
rebuilt at every bell, so "for the fight" is free and "for good" is a block.

### Risks

- **This is a new rule in the fight**, and `CLAUDE.md` says twice that nothing
  new has been invented in combat for a class. It is being invented for an
  *ench*, which is a different thing, and the class reuses a power that was
  already tuned — but the distinction is thin and the commit should say so
  rather than let somebody find it.
- **Three times power is a big number** and nothing in the catalogue moves power
  by that much. Expect to retune; the test that holds it is a fight, not a
  formula.

---

## M10.2 — Top of the Bill

**Theme:** the fifth class, and the promise it makes actually reaching something.

### Deliverables

1. **`Showstopper` is honoured**, before anything else in this milestone.
   `fight::settle` reads the class and `reward::bounty_for` gains the fast-win
   multiplier. The log already carries the fight's duration.
   - **In `reward.rs`, not in the shim and not in `combat`.** It is a settlement
     rule; `reward.rs` is where "what a fight pays" is argued, and §C.1 is the
     precedent for that argument living there.
   - The receipt says so, because *a derived number needs somewhere it is
     shown*: `+30 Fnorp` and `+15 for the speed of it` are two facts.
2. **`every_offered_class_reaches_something`** — the lint that would have caught
   this. A class on the fork whose power is matched nowhere outside `class.rs`
   fails the build. Written as a match over the offered powers rather than a
   grep, so it cannot rot.
3. **The roster moves out of the shim.** `class::OFFERED` in core, read by
   `class_offer_json` and by `tests/classes.rs`, which currently hold two copies
   of it. A rule decided in the shim is a rule the fast suite cannot reach.
4. **The class is `Showstopper`, themed Top of the Bill**, and it is the class
   the Chonga Swing belongs to.
   - **The licence stops being one class's.** `ench::LICENSED_CLASS` is a single
     `&'static str`; it becomes a property of the class in `class.rs` — the
     recommendation is `ClassDef::benches: bool` — so two classes can ench and
     what separates them is *which* enchs they can get, which is exactly what
     M10.0 made possible.
   - **What separates them:** the Kaklon Patent is the licensee that buys and
     tunes, and Top of the Bill is the one whose tree grants the Swing and whose
     bounty rewards ending a fight before it starts. One is an economy, the
     other is a burst.
5. **A tree of eight**, in `data/skills.json`, in the shape the other four have.
   Its spine is the Swing: a node that grants it, nodes that tune what a
   fragile item is worth, and nodes that reward the fast win the class is paid
   for. **Nothing in it invents a mechanic** — the vocabulary is the one
   `Effect` and `Rule` already have, plus M10.0's `GivesEnch`.
6. **A fifth card, in two rows.** `PLAN.md` §6a row 3's fix, and
   `check_the_fork_is_on_top` re-measured against it with `elementFromPoint`.
7. **Themed, sourced and named in the same change** — the class title is already
   in the shipped theme, which is half the work already done.

### Tests

- `every_offered_class_reaches_something`, negative-tested by unwiring the new
  arm and watching it fail.
- `a_fast_win_pays_more_and_a_slow_one_does_not`, and
  `a_lose_win_cycle_is_not_a_gold_farm`'s sibling: **a class that pays more for
  speed must not turn a rat into an income**, which is the family `reward.rs`
  already guards and the exact question M9.2 had to answer for the rout.
- `the_bill_can_ench_and_so_can_the_patent`, and nobody else can.
- `no_class_on_offer_promises_a_stack` grows to five and reads `class::OFFERED`
  rather than its own copy.
- `check_five_cards_fit_and_are_clickable` (browser), measured rather than
  eyeballed.

### Deploy point

After 7.

### Risks

- **The fork is permanent and this is a fifth irreversible choice.** A class
  that is weaker than the four is worse than no class, because it cannot be
  undone. The measurement that matters is a playthrough, and that is M10.3.
- **`Showstopper`'s ten-second window against the pit.** An A. Rat dies in under
  ten seconds to almost anything, so at level five the class pays 50% more on
  nearly every fight, and by the Verge it pays on none. That is either a
  difficulty curve or a bug depending on the numbers, and nothing currently
  measures it. `a_full_expedition_is_a_budget_and_not_a_wall` is the shape of
  the test that should.

---

## M10.3 — Play it, triage it, write it down, ship it

**Theme:** standing since M9.4, and M9.4 is why.

### Deliverables

1. **A real playthrough**, `make play`, new game to the ending, read by a
   person, **as the new class** — which means the walker has to be able to pick
   it, and `fork()` currently picks the Kaklon Patent by name.
2. **A triage table** — Blocker / Wrong / Rough / Later — in the commit, with
   the Later rows written into `PLAN.md` §6c.
3. **The Swing judged by a person.** Three times power for one activation is a
   number nobody has felt yet.
4. **`CLAUDE.md` current**, every number in the closing table re-measured.
5. **Push, deploy, and verify the deploy arrived.** Three deploys have now
   taught something here; the third one taught that a gate failure that looks
   random is a gate failure nobody has instrumented yet.

### Deploy point

The end of it.

---

## 3. Cross-cutting rules this work must not break

- `crates/core` stays graphics-free and wasm-free.
- **`crates/wasm` decides nothing** — and this block *removes* a decision it
  currently makes, which is the roster.
- **Derived, never banked.** A node's effect is not state, and that includes an
  ench it hands over.
- **Read, never computed.** The bounty a fast win pays is core's, and the
  receipt says what was paid rather than the page working it out.
- **A new field on `Character` or `Game` is a compile error until the save
  carries it.** `bought_enchs` is one. Do not add `..` to a destructure.
- **An item that breaks must be visible breaking.** A bar that stops with no
  event is the health bar bug wearing new clothes.
- Every browser check is **negative-tested** — break the thing it guards and
  watch it fail.
- **A planted board check strips every grid.** M9's lesson, and this block
  plants more than any before it.

## 4. Numbers to watch

| | now | after |
|---|---|---|
| Tests | 527 | expect ~560 |
| Catalogue | 544 | 544 — **the fingerprint does not move**; this block adds no component |
| Enchs | 5 | 6 |
| Ench effect kinds | 3 | 4 — power, haste, spin, **fragile** |
| Skill effect kinds | 5 | 6 — **gives_ench** |
| `PlaceKind` | 6 | 7 — **bench** |
| Classes offered | 4 | 5 |
| Skill trees | 5 | 6 — a tree of eight for the Bill |
| Classes whose power reaches code | 4 of 4 | 5 of 5, **and a lint that says so** |
| Towns selling an ench | 3 of 3 | **none** |
| Enchs a level-9 character can reach | all 4 priced ones | whatever the trees award |
| Places gated on a level | 0 | 1, and the crossings already gate 2 regions |
| Save format | v1 | v1, and every M9 file still opens |

## 5. Questions, and which are still open

**Answered, and folded into the milestones above.**

- **Does it break for the fight, or for good?** *For the fight.* M10.1's deploy
  point keeps the reasoning, because it is what makes that milestone small.
- **Which vendor sells what?** *No town sells an ench.* What a tree does not
  award is sold by one vendor, on a place that is not there until level 10 —
  M10.0, deliverables 1 and 4.
- **Does the ench break, or the item?** *The item.* An ench that was consumed
  would be a consumable, and the game already has a word for those.

**Still open, and none of them blocks M10.0.**

1. **Is `The Chonga Swing` the name?** Sourced to p. 41, and the class it
   belongs to is already Top of the Bill in the shipped theme. `PLAN.md` §6.4 —
   whether invention is allowed at all — is still open, and this is a
   construction from a sourced proper noun the way *The Ponkey Turn* is.
2. **Who is the vendor, and where does he stand?** The plan puts him past the
   last crossing, in the half of the map you only see once you have earned it.
   He needs a name, a figure and two paragraphs, and all three are the kind of
   thing the answer changes.
3. **Is `+200%` the right number?** It is exactly "3× more powerful" as asked,
   and nothing in the catalogue moves power by half that. Deferred to M10.3,
   when somebody has played it — the same way M9 deferred the drop rate, and
   that deferral was right.
4. **Should the Bill's tree be able to make the Swing survivable?** A node that
   let a fragile item fire twice would be the class's whole build. It is the
   most obvious node in the tree and it is also the one that could delete the
   mechanic. Planned: **no** — the tree tunes what the swing is *worth*, never
   how many it gets.
5. **Does the vendor restock?** Planned: no, once each, the rule every shelf in
   the game already follows. Worth asking because he is now the only shop for a
   whole system, and "sold out for ever" is a stronger sentence when there is
   one of him.
6. **Does closing the benches want a second town more, or less?** Less, is the
   honest answer, and it is worth writing down: `PLAN.md` §6a row 1 has wanted a
   second town since M8.8, and one of its arguments was that a bench is a reason
   to visit one. That argument is gone. The other three stand.
