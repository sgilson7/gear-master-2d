# PLAN-M8.md — the ench, the spin, and what was already there

Written 2026-09-02, against `c4e004f`. `PLAN.md` remains the plan of record for
M0–M7; this is the plan for the next block of work and follows its conventions.
Nothing here is built yet.

---

## 0. One finding first, because it changes an ask

> *"are curses in the game? if not, they need to be added; I haven't seen any
> gear that gives it."*

**Curses are in the game and always have been.** Measured, not guessed:

| | |
|---|---|
| Pieces that apply a curse | **59 of 536** |
| …on a town shelf | 6 |
| …on the **starting** shelf | 2 — `Greave Mold` and `Plain Sole`, 3 Fnorp each |
| Curse kinds | Searing, Frost, Stun, Misfire |
| Creature attacks that curse | 3 |
| Player-facing places a curse is mentioned | **1** — the component hover, added last week |

So a player has almost certainly bought, seated and fought with curse gear and
been told nothing about it, three times over:

1. **The item card never says a piece curses.** `item_card` lists physical and
   magic damage, idiot mode, cork, the Funny, fury, devotion and harvest. There
   is no arm for `Action::Curse`, so a Greave Mold's whole point is missing
   from the card that exists to explain it.
2. **The replay drops it.** `Event::Cursed`, `Event::Warded` and `Event::Stunned`
   fall into `_ => ("other", …)` in `fight_json`. A curse lands and the screen
   does not move.
3. **Nothing shows a curse that is up.** The original drew curse chips with
   stacks and a countdown beside the pools; GM2D draws the pools and not the
   chips.

This reframes the ask. The work is **not** "add curses" — it is *make the
system that exists legible*, and then *give the player a source they choose on
purpose*, which is the Worm-Fact passive. That is M8.1 and M8.2 below, and it
is cheaper and better than building a second curse system beside the first.

The same check is worth stating as a rule, because it is the second time this
project has nearly built something twice: **before adding a system, grep for
it.** `explain.rs` was written with a duplicate `Action::describe` and
`Trigger::describe` already in `piece.rs`.

---

## 1. What is being asked for, in one list

| # | Ask | Where it lands |
|---|---|---|
| A | Curses reachable and visible | M8.1, M8.2 |
| B | A Worm-Fact node: watch activations, curse of searing when a helmet goes off | M8.2 |
| C | Scouting danger becomes a class skill; remove **Show the numbers** | M8.2 |
| D | A new class, from the book, that grants **enchantments** | M8.3, M8.5 |
| E | Enchantment inventory while packing: attach to a piece, toggle, show it clearly | M8.3 |
| F | The spin: enchanted items rotate each second, stack ×0.1 power, spend on activation | M8.4 |
| G | Cursor drift — the primary button must stop moving | M8.1 |
| H | Fatigue resets in town | M8.1 |

---

## 2. Ordering, and why this order

Each milestone ends at a deployable state. The order is chosen to avoid three
specific rewrites:

- **Plumbing before content.** M8.2 builds the first skill effect that grants a
  *rule* rather than a stat. M8.5 writes eight nodes for the new class, several
  of which will want rules. Building the plumbing once, two milestones early,
  is the difference between writing eight nodes and writing them twice.
- **The model before the thing delivered on it.** The spin (F) is delivered *as*
  an enchantment. If it were built first it would invent its own attachment,
  its own save field and its own board mark, and all three would be thrown away
  when the general system (E) arrived. E first, F second.
- **The spin before the class's tree.** M8.5's nodes will tune the spin
  (a node that adds ×0.05 a turn, a node that keeps one stack through an
  activation). Writing them before the spin exists means guessing at numbers
  and retuning every one.

M8.1 is first because it is entirely presentation, ships value immediately, and
the fixed action bar it introduces makes every later browser walk simpler.

---

## M8.1 — Make what is there visible, and stop the cursor moving

**Theme:** nothing new in the engine. Three things the player cannot currently
see, and one thing that makes them chase a button.

### Deliverables

1. **Curses on the item card.** A fourth group beside *standing still* and
   *every activation*: what the item does **to them**. Sourced from the piece's
   triggers through `Trigger::describe`, which already says "apply curse of
   searing to the enemy" — the card is the only screen not reading it.
   - Watch: the split must stay honest. A curse is an *every activation* effect
     with a target, not a standing stat.
2. **The replay reports curses.** `Event::Cursed`, `Warded` and `Stunned`
   carried in the `fight_json` entry stream with `kind`, `on`, `stacks` and
   `duration_ms` — read, never derived, the same rule armour and the pools
   follow.
3. **Curse chips on the replay panel.** Beside the pools, per side: the curse,
   its stack count, and a countdown that runs off the playback head. Lifted
   from the original's status panel, which already solved the layout problem of
   pools and chips sharing a row.
4. **Fatigue resets in town.** On entering a town, `fatigue = 0`, said out loud
   in the town's own voice.
5. **A fixed action bar on the fight screen.** Today the primary button is at
   the bottom of whichever stage is showing, and the three stages are 15, 19
   and 3 lines tall — so **Fight**, **Skip to the end** and **Walk on** are at
   three different heights. One action row, in one place, for all three stages;
   the stage content scrolls under it.
   - Same treatment for the town (**Pack your frames**) and the event card
     (**Walk on**), so the whole "advance" gesture is one target.

### Tests

- `the_card_says_what_an_item_does_to_them` — every catalogue piece whose
  triggers reach `Action::Curse` produces a card line naming the curse.
- `check_the_replay_reports_a_curse` (browser) — drive a fight against a
  cursing creature, assert a chip appears and counts down.
- `a_town_takes_the_tiredness_off` (core).
- `check_the_advance_button_does_not_move` (browser) — record the bounding box
  of the primary button across the three fight stages and assert it is the same
  within a pixel or two. This is the one that keeps G from rotting.

### Deploy point

After 5. Everything in M8.1 is independent; if any one deliverable turns out to
be more than it looks, ship the rest without it.

### Risks

- **Fatigue resetting in town devalues the tins.** They become field-only,
  which is arguably their point — but `a_restorative_costs_about_what_a_fight_pays`
  is calibrated against a world where a town does not mend you. Expect to
  retune prices down, and say so in the commit.

---

## M8.2 — Skills that grant rules

**Theme:** the skill tree can currently grant a stat, a starting balance, a row
or an assembly percentage. It cannot grant a *rule*. Two of the asks need one.

### Deliverables

1. **`Effect::Grants { rule: Rule }`** — a fifth effect kind, and the first that
   is not arithmetic.
   - `Rule` is a small enum, not a string: an exhaustive match is what keeps a
     rule that nothing reads from shipping. `every_effect_key_is_one_the_engine_actually_reads`
     already refuses an unknown key in the data; this keeps the code honest too.
   - `Node::line()` and `Node::detail()` must describe it. The derived-spec rule
     (TONE 13a) applies: *"every activation of a helmet lands a curse of searing
     on them"*, unthemed, with the number in it.
2. **`Rule::CurseOnActivate { slot, curse }`** — ask B. Carried into combat the
   way class powers are, on the `simulate_holding` rung that already exists.
   `Combatant` gains a small list of granted rules; the activation path checks
   it where `Trigger::OnActivate` is already handled.
3. **`Rule::Scout`** — ask C. The world's danger and encounter figures are
   readable when it is taken, and not before.
4. **The node.** `Worm-Fact Keeper` gains the curse passive. The tree is the
   knowing-and-cursing tree already — The Worm Fact, The Ledger, Something
   Rotting — so it is where a "watch the board and curse them" node belongs.
5. **Scouting moves into a tree and `#numbers` is deleted.** Proposed home:
   also the Worm-Fact Keeper, so the button can go in this milestone rather
   than waiting for M8.5. *(One line of data to move it into the new class's
   tree instead, if that reads better once M8.5 exists.)*

### Tests

- `a_granted_rule_reaches_the_fight` — take the node, fight, assert `Event::Cursed`
  appears and does not without it.
- `every_rule_is_described` — every `Rule` variant produces a non-empty,
  unthemed, numbered line. The exhaustive match makes a new rule a compile
  error until somebody says what it does.
- `check_scouting_is_earned` (browser) — the danger figures are absent before
  the node and present after, and `#numbers` does not exist.

### Deploy point

After 5, as one piece: removing the button before the skill exists takes a
capability away, and adding the skill without removing the button leaves two
answers to one question.

### Risks

- **Combat is a pure function and must stay one.** A granted rule is a fight
  input like a class power, not a mutable global. It goes through the same
  `Held`-shaped door.

---

## M8.3 — Enchantments you attach, and the class that grants them

**Theme:** the new subsystem, and the class whose identity it is.

### A name collision to settle first

`PieceKind::Enchantment` already exists — thirteen catalogue pieces, *laid under
the grid* so gear sits on top of them. That is upstream's terrain model and it
is **not** what is being asked for. Two options:

- **Recommended:** the new thing is an **ench** (the book's own word — "the ench
  economy", "enchmatter", p. 119), a separate concept that attaches to a
  component. The old kind keeps its name and its thirteen pieces. Two words,
  two things, no rename and no migration.
- Rejected: reuse `PieceKind::Enchantment`, which would mean one word meaning
  two mechanics and a rewrite of the thirteen.

### Deliverables

1. **The model.** `data/enchs.json` — id, name, blurb, what it does. `Character`
   gains `enchs_owned: Vec<String>` and `enchanted: Vec<Ench>` where an `Ench`
   is `{ on: PieceId, id: String, active: bool }`.
   - Saved by the same discipline as everything else: component by **registry
     index**, which is what `owned` already does.
   - Adding the fields is a compile error until the save carries them, which is
     the guard already in place.
2. **The rack.** On the packing screen: what you own, what is attached, what is
   toggled on. Click an ench, then click a component → attached. Click again →
   toggled. A second click on the rack detaches.
3. **It is obvious which piece is enchanted.** A mark drawn on the component's
   cells, in the board's own language — the fourth channel after motif,
   luminance and hue, and it must not collide with the lock outline or the
   assembled ring. Proposed: a small corner glyph plus a dashed inner edge,
   greyed when toggled off.
4. **The class.** Derived from the book: **Spike Kaklon**, celebrity inventor
   (Plug Energy, the Yodregar Archives, Grungo-tree elastic), and the ench
   economy is already his world. Working name for the tree: **The Kaklon
   Patent**. Its `ClassPower` is the existing `Recycler` or `Splintered` —
   nothing new invented in combat, the rule this project has held since M5.
   - The class is what grants enching. Until it is taken, the rack is not there.
5. **Two enchs to ship with it**, so the system is usable the day it lands and
   the spin is not carrying the whole feature alone.

### Tests

- `an_ench_survives_a_round_trip` (save).
- `an_ench_follows_its_component` — detach, reseat, rotate; the attachment
  tracks the piece and not the cell.
- `nothing_may_be_enched_twice`, and `an_ench_toggled_off_changes_nothing`.
- `check_the_rack` (browser) — attach, toggle, confirm the board marks it and
  the card says so.

### Deploy point

After 5.

---

## M8.4 — The spin

**Theme:** the ench that turns an item, and the stacking power it earns.

### The one design decision

> *"if they are blocked and cannot rotate, then they do not move"*

**Rotation is decided on the board and banked in the fight; it is not simulated
mid-fight.** Combat has no board — `ItemProfile` is a flat snapshot, which is
exactly why a mid-fight save carries a creature name and a tile and nothing
else. Putting a live board into the fight would undo the property the whole
save format rests on.

So: at pack time, core works out **which of the four orientations the item can
legally take in place** — its turn cycle. That is a board question and the board
is where it is answered. The fight then ticks the cycle once a second and banks
a stack per turn; an item whose cycle has one entry never turns and never
stacks, which is precisely "blocked, so it does not move".

The good consequence: **leaving room to turn costs you cells**, which is a real
packing decision of exactly the kind `PerAdjacentEmpty` already trades in. The
spin is not free power; it is power bought with space.

### Deliverables

1. **`Loadout::turn_cycle(item) -> Vec<Rotation>`** in core — the orientations an
   item can reach in place, in order. Pure, testable, no page involved.
2. **The stack.** `+0.1×` power a turn, spent to zero when the item activates.
   Carried on `ItemProfile`/`RunningItem` beside `power`, which already exists
   and is already a percentage — `power_bonus` is the precedent.
3. **The animation.** The packing board and both replay boards turn the item's
   footprint each second, eased. The page already redraws every frame and
   already has the cells; what it needs from core is the cycle and the tick.
   - `paintMotif` and the edge tracer are shared already, so all three boards
     turn identically.
4. **The card says it.** *"turns every second · +0.1× power a turn, spent when it
   goes off · cannot turn where it is"* — the last of those when the cycle is
   one long, which is the line that tells a player to repack.
5. **The ench itself**, themed. Ponkey Dong figure-skates while deadlifting
   (pp. 20–22) and is already in the ladder as the Rust Colossus; the spin is
   his.

### Tests

- `an_item_with_no_room_does_not_turn` — box one in, assert a one-entry cycle
  and no stacks over a whole fight.
- `stacks_are_spent_on_activation` — the power returns to base on the tick it
  fires, and the damage that tick carries the stack.
- `the_golden_fixture_is_unmoved` — no ench attached, no behaviour change. This
  is the one that says the feature is additive.
- `check_the_spin_animates` (browser) — the drawn footprint differs between two
  frames a second apart, and matches a legal orientation core named.

### Deploy point

After 5.

---

## M8.5 — The Kaklon Patent

**Theme:** the class's tree, and enough enchs to spend it on.

### Deliverables

1. **Eight nodes**, in the shape the other three class trees have: two roots, a
   spine, one convergence. Rows are depth and the tab strip already handles a
   fourth tree with no work.
2. **Nodes that tune the spin** — a second stack per turn, a stack kept through
   an activation, a turn every 0.8s — now that there is a spin to tune.
3. **More enchs**, one or two of them earned rather than bought, so the class's
   tree and the map's errands point at each other.
4. **Move scouting here** if it reads better than the Worm-Fact tree. One line
   of data.

### Deploy point

After 3.

---

## 3. Cross-cutting rules this work must not break

- `crates/core` stays graphics-free and wasm-free.
- `crates/wasm` decides nothing. The turn cycle, the stack arithmetic, the curse
  rule and the scouting gate are all core's.
- **Derived, never typed.** Every new mechanical description comes off the
  effect, unthemed, with the number in it — TONE 13a.
- **Read, never computed.** The curse chips read the log the way the armour bar
  and the pools do. The replay has been wrong twice by working something out
  for itself.
- A new field on `Character` or `Game` is a compile error until the save carries
  it. Do not add `..` to a destructure.
- A new module in the web chain needs no thought now — `package-web.sh` stamps
  every relative import by pattern and dies on a bare one.
- Every browser check must be **negative-tested**: break the thing it guards and
  watch it fail before keeping it. Two checks have shipped vacuous.

## 4. Numbers to watch

| | now | after |
|---|---|---|
| Tests | 447 | expect ~490 |
| Catalogue | 536 | +2 to +4 (enchs are not components) |
| Skill trees | 3 + base | 4 + base |
| Effect kinds | 4 | 5 |
| `PlaceKind` | 4 | 4 |
| Save format | v1 | v1 — every new field defaults |

## 5. Open questions for the human

1. **Scouting's home** — Worm-Fact Keeper (available in M8.2) or the Kaklon
   Patent (waits for M8.5)? Planned as the former; one line to move.
2. **Does a town reset fatigue completely, or to a floor?** Planned as
   completely, as asked. A floor would keep the tins relevant on the way home.
3. **Can a component carry two enchs?** Planned as one. Two is a bigger space
   and a bigger UI.
4. **Is the spin's ×0.1 per turn, uncapped?** Planned uncapped, because the
   spend-on-activation is the cap — a slow item stacks more and fires less.
