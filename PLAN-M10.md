# PLAN-M10.md — where an ench comes from, and one swing that ends a bout

Written 2026-09-02, against `8d1e38f`. `PLAN.md` remains the plan of record for
M0–M7, `PLAN-M8.md` for M8 and `PLAN-M9.md` for M9; this is the plan for the
next block and follows their conventions.

**Four milestones, each deployable on its own.** M10.0 changes where an ench
comes from, M10.1 teaches the fight that an item can break, M10.2 is the class
that is built on the two of them, and M10.3 is the standing "play it" gate.
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
| A | An ench comes from a **named vendor**, not from every town | M10.0 |
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

**Theme:** an ench stops being a shelf item and becomes something you went and
got.

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

1. **A bench is a town's, and it is content.** `shops.json`'s town entries gain
   `enchs: [id]`, beside the `stock` they already carry. `EnchDef::price` stays
   as *what it costs*; **where** is the town's list. Same division the components
   already make: the catalogue says what a thing is, the shelf says who sells it.
   - **Append, never insert**, and the save carries `WorldState::bought` by
     index — so the bench needs the same treatment the shelf has or a save that
     bought ench two comes back having bought a different one. Decide this
     explicitly: the recommendation is a **separate `bought_enchs` list keyed by
     `(town, ench id)`**, because an ench is bought by name and not by shelf
     position, and reusing the index would be one list answering two questions.
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
3. **`Rule::check`'s sibling for enchs.** `SkillsData::parse` refuses a node
   naming an ench `enchs.json` has not got, and `ShopsData::parse` refuses a town
   selling one. Both are the guard `Rule::check` already is, and both exist
   because *content nobody can reach is the thing nothing else in the game says
   anything about.*
4. **A lint per source.** `every_ench_comes_from_somewhere` walks `enchs.json`
   and refuses one that is on no bench, in no node and paid by no errand. That is
   the lint this milestone is *for*: the point of narrowing availability is that
   an ench now has a place it comes from, and an ench with none is an orphan.
5. **The bench says whose it is.** A town that sells no ench shows no bench, the
   same way a town with no errand shows no board.

### Tests

- `every_ench_comes_from_somewhere`, and its inverse — no ench is sold by every
  town, which is the ask.
- `a_node_that_names_no_ench_is_refused`, `a_bench_that_names_no_ench_is_refused`.
- `a_granted_ench_is_derived_and_not_banked`: take the node, assert the ench is
  there; **change nothing in the save and retune the node in the data**, and
  assert the character's answer moved.
- `a_save_from_before_the_bench_moved_still_opens` — the four priced enchs are
  still buyable somewhere, and a character who already owns one keeps it.
- `check_the_bench_is_that_towns_bench` (browser).

### Deploy point

After 5. Nothing new exists; four things moved. Worth shipping on its own
because it is the milestone most likely to be *wrong about the map* — one town
is placed, so "specific vendors" is currently a list of one, and that is a
finding a player will have an opinion about immediately.

### Risks

- **One town is placed.** Kettleworks and High Wick are written, shelved and on
  no map (`PLAN.md` §6a row 1). If the four existing enchs are spread across
  three benches, three quarters of them become unreachable in the shipped
  build. **The plan is that the pit's bench keeps two and the tree grants one,
  and the other two move to the staged towns** — which turns §6a row 1 from a
  deferred nicety into the thing standing between the player and content. Say so
  in the commit; do not quietly keep all four in the pit.

---

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

After 6, with the ench on a bench and no class attached to it. **The mechanic
gets a player's opinion before a class is built on it**, which is the ordering
M9 wished it had for the drop rate.

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
| Classes offered | 4 | 5 |
| Skill trees | 5 | 6 — a tree of eight for the Bill |
| Classes whose power reaches code | 4 of 4 | 5 of 5, **and a lint that says so** |
| Towns selling every ench | all of them | none |
| Save format | v1 | v1, and every M9 file still opens |

## 5. Open questions for the human

1. **Is `The Chonga Swing` the name?** Sourced to p. 41, and the class it
   belongs to is already Top of the Bill in the shipped theme. `PLAN.md` §6.4 —
   whether invention is allowed at all — is still open, and this is a
   construction from a sourced proper noun the way *The Ponkey Turn* is.
2. **Does it break for the fight, or for good?** Planned: **for the fight.**
   Combat is a pure function of the board and a mid-fight save carries a
   creature name and a tile; a component destroyed for good would make the fight
   write to the character, and it would be the first thing in the game that
   takes a component away — with locks, enchs, Auto-pack and the save all
   needing an answer. Permanent is a much larger and much more interesting
   mechanic, and it should be its own block if it is wanted.
3. **Does the ench break, or the item?** Planned: the item, for the fight. An
   ench that was consumed would be a consumable, and the game already has a word
   for those.
4. **Which vendor sells what?** Planned: the pit keeps two, the tree grants the
   Swing, and the last two go to Kettleworks and High Wick — which are on no map.
   That makes `PLAN.md` §6a row 1 load-bearing rather than deferred. The
   alternative is that the pit keeps all four and "specific vendors" means one
   vendor, which is not much of a rule.
5. **Is `+200%` the right number?** It is exactly "3× more powerful" as asked,
   and nothing in the catalogue moves power by half that. Deferred to M10.3, when
   somebody has played it — the same way M9 deferred the drop rate, and that
   deferral was right.
6. **Should the Bill's tree be able to make the Swing survivable?** A node that
   let a fragile item fire twice would be the class's whole build. It is the
   most obvious node in the tree and it is also the one that could delete the
   mechanic. Planned: **no** — the tree tunes what the swing is *worth*, never
   how many it gets.
