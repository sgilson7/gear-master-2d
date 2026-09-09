# HANDOFF-M13.md — where M13 stopped, and how to pick it up

*Written mid-block, on the human's word. `PLAN-M13-2.md` is the frame (there is
no `PLAN-M13.md`; the file on disk is the `-2`). This file is the door back in —
read it, then `SECOND-ORDER-M13.md`, then carry on at §6.*

*Second sitting: **M13.6 is done.** §3 is kept as the record of what it found,
rewritten to say what was actually wrong; §6 is the live list.*

---

## 0. State in one paragraph

**Nothing is committed.** Everything below is uncommitted work on `main` at
`633f9bf`. **All ten milestones are done**, the suite is green at **785
passing** and **34 seconds warm**, and the gate walks **all three engines** with
five new checks in it.
M13.6 turned out to be the block's biggest milestone by a distance: it found
that **thirty-eight of the sixty expert nodes changed nothing**, and four
separate engine faults behind that. §3 is the account; §3a is M13.7's own.

**Do not `git push` or `make publish`.** Nothing here has been deployed. What is
left is M13.9.

---

## 1. The milestone table, as it stands

| # | Milestone | Status |
|---|---|---|
| M13.0 | **The countable fact** — `SkillsData::tree_finished`/`tree_progress`; `Character::second_class`/`expert`/`second_paper`; `classes()`, `class_defs()`, `finished_trees()`, `choose_second_class`; save round-trip + `Game::eq` | ✅ done, 728 |
| M13.1 | **The expert roster** — `crates/core/src/expert.rs`: 10 `ExpertPower`, 31 knobs, `EXPERTS` pair table, `for_pair` order-insensitive, `knobs`/`knob`/`tune`/`step`/`describe`/`short`; `ClassPower::Expert` (one arm, not ten); ten `ClassDef`s derived into `CLASSES` via `EXPERT_DEFS`; `is_earned`; both themes name all ten; `reward::AtTheBell` + `combat::CurseBill`; Short Programme and Eleventh Season paying; `Held::empty_frames` | ✅ done, 742 |
| M13.2 | **`Effect::Tunes`** — seventh effect kind; parse-time knob check **against the tree's own class**; refuses a zero move and a sub-step move; `every_expert_knob_is_declared`, `a_tuning_that_cannot_be_seen_does_not_load`, `every_declared_knob_is_moved_by_some_node` | ✅ done, 745 |
| M13.3 | **Four new rules** — `Spread` (diagonal, settled at the bell), `RowHarvest` (paid at the *start*), `Beacon` (`ench::broadcast`, no chaining, `Effect::scaled`), `Productivity` (`RunningItem.enched/fires/slowed_pct/base_cooldown_ms`, capped at `MAX_PRODUCTIVITY_SLOW_PCT`); 15 behavioural tests; `Slot::worn`; `can_take` over three classes | ✅ done, 760 |
| M13.5 | **The ten trees land** — `skills.expert.json` appended to `data/skills.json` (16 trees / 124 nodes); `expert_nodes_touch_only_the_expert` + `kin()`; `every_expert_tree_is_two_roots_and_a_capstone`; `every_expert_node_moves_a_knob_it_can_be_seen_to_move`; `finishing_an_expert_tree_changes_its_promise`. **All three new lints negative-tested** | ✅ done |
| M13.4 | **Three papers** — `shop::StockGate { Level, TreesFinished }`, `shop::Paper`; `Game::papers()` draws refused lines with the count in them; `Game::buy_paper`; `Character::take_expert`, `expert_on_offer` | ✅ done, **775** |
| **M13.6** | **Every expert reaches something** | ✅ **done, 781** — and it found four engine faults; see §3 |
| M13.7 | **The screens** — a second fork of four cards that names what each pair reaches and can be slept on; three papers on the counter with the refused one counting; a sixth tab printing the **tuned** promise; a rack that holds up to four; a sheet that says all three classes | ✅ done |
| M13.8 | **The gate** — five checks, all five negative-tested, three engines | ✅ done, **63 ok lines** |
| M13.9 | **The notebook, executed** — `assembly_pct` unbanked and every door that sets a class guarded; two theme lints; the `Slot::pieces()` sweep; `common::items_in_a_row`; `grown_health` named; **the suite from a minute to 34s** | ✅ done, **785** |

---

## 2. What is on disk

**New files** (all untracked):

| file | what it is |
|---|---|
| `crates/core/src/expert.rs` | the ten powers, their knobs, the pair table |
| `crates/core/tests/second_paper.rs` | M13.0 — 11 tests |
| `crates/core/tests/experts.rs` | M13.1 + M13.2 + M13.5 — 21 tests |
| `crates/core/tests/rules_m13.rs` | M13.3 — 15 tests |
| `crates/core/tests/papers.rs` | M13.4 — 11 tests |
| `crates/core/tests/experts_reach.rs` | M13.6 — 5 tests, five boards, three foes |
| `SECOND-ORDER-M13.md` | the notebook, **23 rows**, all seven candidates answered |

**Modified:** `character.rs`, `class.rs`, `combat.rs`, `curse.rs`, `ench.rs`,
`fight.rs`, `game.rs`, `lib.rs`, `loadout.rs`, `reward.rs`, `rule.rs`,
`save.rs`, `shop.rs`, `skills.rs`, `slot.rs`, `theme.rs`, `world.rs`,
`crates/wasm/src/lib.rs`, `data/skills.json`, `data/theme.{td,plain}.json`,
`skills.expert.json`, and eight test files whose fixtures needed a new field.

**Save shape.** Five new `Character` fields, every one `#[serde(default)]` and
skipped when empty: `second_class`, `expert`, `second_paper`, `fast_wins`,
`told_curses`. **No seam** — nothing moved `CATALOG`, so `catalog_fingerprint`
is untouched and every M12 save opens. All five are in `SaveFile::of`'s
destructure, in `into_game`, and in `Game::eq`.

---

## 3. M13.6, and the four things it found

`crates/core/tests/experts_reach.rs`, five tests, all green. What it cost is the
interesting part: **on its first honest run it reported thirty-eight of the
sixty expert nodes as points the tree sells and the engine never reads.** Two of
the causes were fixtures and four were the engine.

### 3.1 The engine handed the fight the *untuned* power

The largest of them, and it made nine of the ten trees mostly inert.
`Character::class_defs` returned `Vec<&'static ClassDef>` off `CLASSES`, which
is the roster **before any point is spent** — so `expert_power`'s tunings
reached `start_with` and `combat_items` and reached neither `combat.rs` nor
`reward.rs` nor the streak window. It returns **owned** `ClassDef`s now, with
the expert's arm carrying the tuned power, because a static reference cannot
carry a tuning; every caller already cloned into a `Vec<ClassDef>`, so nothing
downstream had to change. Notebook row 12.

### 3.2 Two knobs were aimed at a kill, and this game deals one foe

`rebate` and `encore` both fired on *a foe going down*, which
`PLAN-M13-2.md` §3.1 D and §3.7 C both write as **a kill inside the fight**. A
brawl has those; GM2D does not deal one — `fight::run` builds a single
`MonsterSpec` and `check_down` breaks the loop on the same tick — so strength
refunded onto the last tick of a fight buys nothing, and a free-cast window
reopened after the only foe is dead is a window nobody casts in.

Both are re-aimed at `combat::the_fight_turned`: **every quarter the enemy
loses**, capped at three so a corpse is not one. Quarters and not halves is
`encore` deciding it — an encore is a *count*, the tree sells two, and a
milestone that can happen once is a count that can only ever be one. Divergence
recorded in §4; notebook row 13.

### 3.3 A percentage off three rounds to nothing

`on-standing-discount` sells *twenty percent less* on a cast, a cast costs
three, and `3 * 20 / 100` is nought. `ExpertPower::cast_price` is the one place
that sum is done now, rounded the payer's way, and `describe` prints **the
price** rather than the percentage. Guarded by
`a_discount_on_a_cast_takes_something_off` over every percentage rather than the
three the tree reaches. Notebook row 14.

### 3.4 `racks` was Full Bill's whole promise and nothing read it

*"One ench a component"* was written into `attach_ench` as a rule, so the expert
that is **both licences on one counter** sold two points of a second rack the
engine would not give. `Character::ench_racks` is the answer, `attach_ench`
enforces it, `enchs_on` is the whole rack, and `detach_ench`/`toggle_ench` take
the last one on because a rack is a stack. `Refusal::AlreadyEnched` carries the
count as well as the name, because *"the Ponkey Turn is already on that"* and
*"that component holds one"* are two pieces of news. Notebook row 15.

**The shim still draws the first of them.** `ench_json` reads `ench_on`, so a
Full Bill's second ench works and cannot be seen — which is M13.7's, and is
called out in a comment where it is.

### 3.5 The fixtures, and why there are five boards

The other two causes were the fixtures, and the shape of the fix is worth
keeping because it is most of the file:

- **A `put` whose answer is ignored is a fixture that silently does nothing.**
  `bare()` asked for a cursing sole on the cell `build_full_loadout` had already
  seated a `Greave Mold` on, was refused, and went on to measure four experts
  against a board with no curse on it. Every seating is asserted now.
- **A board has to pose the question its expert is about**, and no one board
  poses ten. There are five: `bare` (four items, two empty frames, a blade and a
  curse), `caster` (casts it cannot pay for, two kinds of curse, and forty-eight
  strength to sell), `busker` (the same with the gloves left on, which is the
  trickle of Funny a *discount* decides), `bewitched` (the busy board with every
  ench in the rack, for a fight long enough to land one twice) and `enched` (the
  Auto-packed board, nineteen items and forty-four touches, for the one expert
  about what a neighbour gets).
- **Three foes, walked in order and carried out of each into the next.** A rat
  that is over in three seconds, a sentinel that is beaten slowly — the only one
  that *turns* while there is a fight left — and a Kettle Wight that runs to the
  buzzer, which is the only way a board casts thirty times and a `cap` on
  borrowing ever binds. `Character::carry_out_of` is the carry, lifted out of
  `fight.rs` onto the character it was always only touching, because `told` is
  the one knob that crosses a fight boundary and a measurement that never
  crosses one cannot see it.

### 3.6 The question the test asks, and why it changed

`every_point_in_an_expert_tree_buys_something` asked *whole tree* against *whole
tree minus this node*. It asks two things now, in order:

1. **The chain up to the node, against the chain plus the node** — which is a
   build a player can be standing in, and asks a threshold at the bottom of its
   range where it is legible. Forty against seventy is a different fight;
   seventy against a hundred is the same fight twice when nothing can spend
   seventy.
2. **The whole tree minus the node**, asked only of what the first could not
   see. Not a reachable build — dropping a root leaves its children taken — and
   that is what makes it a useful second look at a capstone's own numbers.

Every one of the four engine fixes was negative-tested by putting the fault back
and watching the right name print.

## 3a. M13.7, and the fault it introduced

The screens went in as `PLAN-M13-2.md` §1.2, §1.3 and §1.7 describe them, and
one thing had to be decided that the plan does not name.

**The second fork must not nag.** `offerClass` is called from three places —
after a fight, after banking, and on every load — because the level-five fork
is *an unanswered question that keeps being asked*. The second fork is not
that: it was bought, the paper is spent on the choice rather than on the
purchase, and §1.3 says in as many words that a player *is allowed to sleep on
it*. Wired to the same three call sites it came back after every single fight,
which is the game refusing to let you.

So `offerClass({ paper })` raises the second fork only where a player asks for
it: when they buy the paper, and from a line on the character sheet that says
it is in their pack. That line had to exist anyway — **a thing in your pack
that no screen mentions is a thing you have forgotten you own** — and it is
what §1.3's *"opening it in the pack re-raises the screen"* means on a page
with no pack.

Three other things the milestone had to do that are worth knowing:

- **`detach_ench` and `toggle_ench` take an `nth`.** A Full Bill's component
  holds up to four; a rack screen drawing four rows whose buttons all reached
  the same ench would be three controls doing somebody else's job.
  `Character::nth_ench` is the one walk, and past the end is the last one on —
  a rack is a stack.
- **The item card shows the whole rack**, not the first of it. `ench_json`
  carries `more`, and `shape.js` prints all of them.
- **`.promise` was scoped to `.wares`.** The tab head is a `<p>` in a
  `fighthead` and inherited nothing — the `.made .tabs` lesson in a new coat,
  and `#tree-promise` has its own rule now.

---

## 3b. M13.8, the gate

Five checks, in one run and sharing one character, because *what happens next*
is the question a player asks and three characters would never once ask it:

| check | what only a browser can answer |
|---|---|
| `check_the_papers_are_drawn_and_refused` | all three are drawn from the first visit, and the locked one's sentence has the count in it |
| `check_the_second_fork_can_be_slept_on` | four cards, each naming what the pair reaches, Escape puts it down, **it does not come back on a reload**, and the sheet gets you back to it |
| `check_the_expert_tab_says_what_a_point_bought` | two finished trees take the expert, a fourth tab appears, and **one point moves the sentence at its head** |
| `check_a_full_bill_holds_two_enchs` | two enchs on one component through the page's own door, and everybody else refused the second **by name** |
| `check_the_sheet_says_every_class` | all three are on the one screen that says what you are |

**Every one was negative-tested** by putting the fault back and watching the
right sentence print: the rack reverted to one, the tab printed the untuned
promise, the sheet dropped two classes, the expert paper lost its gate, the
fork was allowed to nag, and the pairing line was dropped from the card.

The gate is **63 `ok:` lines** and walks chromium, firefox and webkit.

---

## 3c. M13.9, the notebook executed

Seven rows were marked *M13.9 candidate*. All seven are answered, and three of
them turned up something that was actually wrong:

| row | what it asked | what it found |
|---|---|---|
| 1 | a guard on `assembly_pct`, or stop banking it | **Both.** The save no longer carries it — it was written and then thrown away on the way in, which is a number somebody will one day believe. `a_class_taken_any_way_re_derives_the_bonus` walks every door with the Kaklon Licensee on **both sides of the pairing**, because it is the only class whose power moves the number and a door it is not standing at is a door that can forget in silence. |
| 3 | a lint that a canonical class name is display-safe | `every_class_name_is_one_a_player_could_read`, accepting either a name a person would write or a name every theme renames. |
| 7 | a lint that a knob name survives the theme swap | **It found a real one.** Reading the *right-hand side* of `Theme::vocabulary` rather than six words somebody typed, over every node line and every knob: the ten expert promises said **Funny** for the mana a cast costs — and *Funny* is that theme's word for **magic**, not mana. Wrong twice over, on the sentence somebody reads before an irreversible choice. |
| 9 | a sweep of `Slot::pieces()` callers | **The save wrote every enchantment twice**, in `placed` and again in `enchanted`, and only `Slot::place` routing by kind kept it from being a bug. Two `loadout.rs` walks meant the underlay and now say so. And `Character::fingerprint` and `class::rank` are dead — upstream's suggest-a-class, which the fork replaced. |
| 11 | a fixture for two finished items that touch | `common::items_in_a_row(ch, n)` — a line, because *does it reach my neighbour's neighbour* needs three. |
| 16 | `grown_health` writes or goes | Named where it lives. The decision is the human's; see §6. |
| 17 | the suite's runtime | **A minute to 34 seconds**, and `rules_m13.rs` was 29.6s of it. See §6a. |

---

## 4. Divergences from `PLAN-M13-2.md`, to be written into `CLAUDE.md`'s table

| § | Divergence | Why |
|---|---|---|
| 4.2 | **`Rule::Spread` works on the diagonal, not orthogonally.** | `Slot::enchant_is_live` pays an enchantment nothing while another touches it edge-on, so a copy laid beside its source **kills both** and the node makes you worse. A corner is the tightest spread this board allows and the borrowed idea survives: what is next to what still decides what you get. |
| 4.2 | **Spread is settled at the end of a fight, not during one.** | Combat is a pure function of what it was handed — that is why a mid-fight save carries a creature name and a tile, and why `Effect::Fragile` breaks an item *for the fight*. Turns are `duration_ms / SPIN_EVERY_MS`. |
| 4.3 | **`RowHarvest` pays at the *start* of a fight.** | *At the bell* means the start everywhere in this engine. §4.3 says `fight::settle`, which is the one place its mana could not be spent. |
| 3.6 | **The `harvest` knob is `per_cell`.** | Harvest is the theme's word for the nature pool, so the node's line promised a pool the class does not touch. It now shares the name of the rule field it tunes. |
| 8.1 | **The papers stand on Spike's van**, `the-kaklon-van` at `[4, 6]` on west-bambulon, `hidden_until_level: 10`. | The van is already the counter the Patent's paper is sold from. |
| 1.6 | **An expert tree *may* hand over an ench something else already gives.** | All eight enchs have an owner, so any `gives_ench` in an expert tree duplicates — and for Full Bill, whose promise *is* rack count, a second copy is the promise. `every_ench_comes_from_somewhere` exempts expert trees and asserts the *opposite* for them. |
| 3.1 D, 3.7 C | **`rebate` and `encore` fire on the fight *turning*, not on a kill.** | Both rows are written against a kill inside the fight, and GM2D deals one foe: `fight::run` builds a single `MonsterSpec` and `check_down` breaks the loop on the tick it falls, so both paid onto a fight that was already over. `combat::the_fight_turned` reports **every quarter the enemy loses**, capped at three so a corpse is not one — quarters rather than halves because an encore is a *count* and the tree sells two, and a milestone that can happen once is a count that can only ever be one. |
| 3.7 B | **`after` prints the price, not the percentage.** | A cast costs three, so a fifth off it is nought point six and integer division makes that nought — `on-standing-discount` sold *twenty percent less* and took nothing off. `ExpertPower::cast_price` rounds the payer's way and is the one place the sum is done; `describe` reads it, so the promise cannot be a different sum from the one the fight does. |
| 1.6a | **`racks` is enforced, and a rack is a stack.** | The plan gives Full Bill a rack count and `attach_ench` had *"one ench a component"* written into it as a rule, so the promise reached nothing. `Character::ench_racks` is the number, `enchs_on` is the rack, and taking one off takes the last one on. |
| 1.3a | **The second fork does not re-raise itself, and the sheet is how you get back to it.** | §1.3 says the paper is spent on the choice and a player may sleep on it; `offerClass` is called after every fight, after every banking and on every load, because that is what the level-five fork needs. Wired to the same three the second fork came back after every fight, which is the game refusing to let you sleep on it. It opens on the purchase and from the line on the sheet that says the paper is in your pack — which is what *"opening it in the pack re-raises the screen"* means on a page with no pack. |
| — | **`Loadout::assembly_pct` is no longer in the save.** | Not a divergence from the plan, which does not mention it: a cleanup the block's own notebook asked for. It was written into every file and thrown away on the way in, so what the file said was a number nothing read and anything could believe. Derived on load, guarded at every door that sets a class. |

---

## 5. The rules this block added, and where each is read

| rule | read by |
|---|---|
| `Spread { every_turns }` | `fight::settle` → `Character::spread_underlay` |
| `RowHarvest { per_cell }` | `Character::row_harvest` → `Held::mana` |
| `Beacon { pct }` | `Character::combat_items` → `ench::broadcast` |
| `Productivity { every, slower_pct }` | `combat::activate`, off `Combatant::productivity` |

And the six fight-side experts: `pay_for_a_cast` (Loud Calculation, Opening
Number), `requisition` (Curse Requisition), `standing_fact_room` +
`land_curse_for` (Standing Fact), `overwind` (Overwound Arm), the spin tick
(Patented Funnel), the granted-curse loop (Cursed Licence). Two purse experts
are in `reward::expert_pct`. `FullBill` is the board's.

---

## 6. What is left

**Nothing in the block.** What is left is not the builder's:

1. **A human's word before anything is pushed.** Nothing here is committed and
   nothing is deployed. `git log origin/main..HEAD` is empty and the whole block
   is in the working tree.
2. **`CLAUDE.md`'s numbers**, §7 below, and a deploy note in the shape the
   others take.
3. **One decision the code asks for and does not make**:
   `Character::grown_health` is a save field nothing in the game writes.
   Deleting it is a design decision — it is where health-per-level would go if
   it is ever wanted — and `tests/save.rs` deliberately round-trips a twelve
   through it, so it is unused rather than broken. The field says all of that
   where it lives. Notebook row 16.

### 6a. The suite was a minute warm and is 34 seconds

`CLAUDE.md` says fourteen, which was true at M12 and is not now. Measured on
this machine, warm, after M13.9:

| file | tests | before | after |
|---|---|---|---|
| `rules_m13.rs` | 15 | **29.6s** | **0.03s** |
| `experts_reach.rs` | 5 | 9.6s | 6.5s |
| `drops.rs` | 9 | 11.3s | 11.3s |
| everything else | ~756 | ~10s | ~10s |
| **the suite** | **785** | **~60s** | **34s** |

**`rules_m13.rs` was one line.** `beacon_board` ran Auto-pack over the whole
catalogue on twenty-row grids, four times, because it was the only fixture in
the repository with two items that touch — which is notebook row 11, and
`common::items_in_a_row` is what row 11 asked for. `experts_reach.rs` measures
once per *set* of nodes rather than once per question; six nodes of a tree share
four chains between them.

`drops.rs` is untouched and is now the slowest file in the suite. It is not
M13's and `CLAUDE.md` already explains what it does.

## 7. Numbers to re-measure before `CLAUDE.md` is updated

Catalogue **568** (unmoved). Trees **16**, nodes **124**. Classes offered 5,
classes in the game **15**. `Rule` kinds **13**. Effect kinds **7**. Save v1,
no seam. Suite **785**, and **34 seconds warm** — not the fourteen `CLAUDE.md`
says, and not the minute it was before M13.9; see §6a. Browser gate **63 `ok:`
lines**, three engines.
