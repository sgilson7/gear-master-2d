# PLAN-M13 — The Second Paper, and the ten experts

*Second-class acquisition (mastery offers, Fnorp opens), the expert-class
roster, and sixty skill nodes. Written against the M12 tree: `class.rs`,
`character.rs`, `skills.rs`, `rule.rs`, `reward::bounty_with_class`, the
level-5 fork screen, and `data/skills.json` format 1 as shipped.*

*Ships with `skills.expert.json` — the ten trees as real data, schema-valid
against format 1, ready to append to `data/skills.json`.*

---

## 0. The one-paragraph version

**Spike Kaklon's counter carries two more papers from the first time you
walk in, and you cannot buy either of them yet.** The Second Paper is
**5,000 Fnorp** and unlocks when your first class tree is finished; buying
it reopens the level-5 fork with the four classes you did not take. The
second tree's nodes cost **2 points** where the first cost 1. Finish that
tree too — two classes completed — and the **expert paper** beside it
unlocks: one of ten, decided by which pair you hold, free, because you have
already paid twice. An expert class replaces nothing. Both parent powers
stay on, the expert's promise is a third rule that is only legible because
the first two are true, and **its tree moves nothing but its own promise.**

---

## 1. Decisions

### 1.1 "Finished" is a countable fact

A class is **finished** when `skills_taken` contains every node of its tree.
Not the base tree — the base is nobody's class — and not "most of." The
trees are 8–10 nodes; a player can count them, so the game may too.

```rust
/// In core, beside the tree it reads.
pub fn tree_finished(class: &str, taken: &SkillsTaken) -> bool
```

*Why not a level gate.* Levels measure walking; the trees measure the class.
Stranger of Paradise unlocks advanced jobs off job-tree progress rather than
character level, and it is the right read here: mastery is the points spent,
and a level-40 save that hoarded points has mastered nothing.

### 1.2 Both papers are on the counter from the first visit, and both are refused

Spike Kaklon already sells the Kaklon Patent's paper for 5,000 Fnorp. He now
sells three papers, shelved together, **visible from the first time you walk
in** and refused until their condition is met:

| Paper | Price | Unlocks when |
|---|---|---|
| Kaklon Patent (shipped) | 5,000 | — |
| **The Second Paper** | **5,000** | one class tree finished |
| **The expert paper** (whichever pair you hold) | 0 | **two class trees finished** |

*Visible-but-refused is the whole point.* A locked line on a shelf you can
read is a goal; an absent line is a secret. The refusal names what is in the
way, per TONE rule 12 — *"you have finished six of the eight, and he can
count"* — and the third paper's line **prints the expert's promise**, so a
player choosing a second class at level 12 can see all four experts that
choice would eventually reach. That is the pairing decision made in
daylight, which is what a fork screen is for and this one has no screen.

New shop machinery, and it is one field:

```rust
/// What a shelf wants before it will sell.
///
/// The van at [4, 6] already gates stock on a level; this gates on a fact
/// about the trees. One field, two readers, and the *line stays drawn* —
/// a refused entry is priced, described and greyed, because a shelf you
/// cannot see is not a shelf.
pub enum StockGate { Level(u32), TreesFinished(u8) }
```

*5,000 for the second class, matching the Patent.* The two papers cost the
same on purpose: one buys a licence and one buys a class, and Spike does not
price by what a thing is worth to you. The digit is on the shelf.

### 1.3 The fork screen is reused, not rebuilt

The second choice raises the **same screen** as level 5, with two changes
and no others:

- It offers **four cards**, the roster minus `character.class`.
- **It takes Escape.** The level-5 fork refuses Escape because an unanswered
  question must keep being asked; the second fork was *bought*, and a player
  with a paper in their pack is allowed to sleep on it. The paper is
  consumed on **choice**, not on purchase; until then, opening it in the
  pack re-raises the screen.

Choice is **permanent**, same as the first. No path clears either.

### 1.4 Three classes, all live, no arbitration invented

```rust
pub class:        Option<String>,   // unchanged, canonical name
pub second_class: Option<String>,   // serde default
pub expert:       Option<String>,   // serde default
pub fn classes(&self) -> impl Iterator<Item = &str>   // 0, 1, 2 or 3
```

The three places a power is honoured — the purse
(`reward::bounty_with_class`), the fighter at the bell, the board — each
fold over a slice where they read an option. The five shipped powers touch
five different rules and no pair collides; §6's lint pins that this stays
true as the ten experts land.

### 1.5 The second tree costs 2 a node, and the schema already says so

`SkillNode.cost` exists and is already 1 or 2 across the shipped trees.
Second-class and expert nodes are authored at **`"cost": 2`**. **No format
bump, no migration, no new field** — the ledger `can_take` already reads is
the ledger that prices this.

*Why a cost and not an earn rate.* A cost is one number on one screen. A
rate is a curve nobody can check from inside the game, and TONE rule 4
dislikes numbers you cannot point at.

### 1.6 An expert tree moves its own promise and nothing else

This is the constraint that gives the block its shape, and it is enforced
rather than intended:

> **Every node of an expert tree must reach that expert's power.** It is a
> `tunes` of a knob the power declares, a `grants` of a rule the power is
> kin to, or a `gives_ench` inside the power's licence. No flat stats, no
> `start_with`, no `grow_slot_rows`, no bare `assembly_pct`.

A `+12 strength` node would be a node you could take without noticing which
class you were in. The five base trees are allowed to be a mix because they
are the character's first shape; an expert tree is the argument for its own
promise, six nodes long, and *"the class is the gate"* becomes *"the tree is
the gate's twelve points."* `expert_nodes_touch_only_the_expert` is the
lint, and it is why §2 declares knobs before §3 spends them.

### 1.7 A sixth tab

The skills screen grows one tab, drawn like the others and only when
`character.expert` is `Some`. Base | Class | Second | **Expert**. The
expert tab prints the promise at its head — the same `describe()` string
the paper printed — because every node under it is a footnote to that line.

### 1.8 Saves load; the question keeps being asked

All three fields are serde defaults, so an M12 save opens. A save with a
finished tree walks into Spike's and finds the paper unlocked, same posture
as the level-5 rule: the question was never answered rather than declined.
No new components, so the catalogue fingerprint is untouched and this
milestone breaks no saves.

---

## 2. The ten experts, their promises, and their knobs

A **knob** is a named integer on a live power that a node may move. Knobs
are declared by the power in Rust and refused at parse time if a tree names
one that does not exist — the same guard `Rule::check` already applies to
slot and curse names.

| # | Pair | Expert class | Promise (the rule, one line) | Lands in | Knobs |
|---|------|--------------|------------------------------|----------|-------|
| 1 | Gorillathon × Funnel Sergeant | **Loud Calculation** | A cast short of Funny pays the difference in strength, at 2.0 the point, up to 40 a fight. | fighter | `rate` (tenths), `cap`, `floor`, `rebate` |
| 2 | Gorillathon × Worm-Fact Keeper | **Standing Fact** | Curses landed while wearing 2 items or fewer do not expire. | fighter | `worn`, `carry`, `bite`, `told` |
| 3 | Gorillathon × Kaklon Patent | **Overwound Arm** | An empty frame spins anyway, half a stack a turn, into your bare strength. | board | `half` (tenths), `ceiling`, `carry` |
| 4 | Gorillathon × Top of the Bill | **Short Programme** | The ten-second window widens 1 second per empty frame. | purse | `per_slot`, `floor_ms`, `pct`, `streak` |
| 5 | Funnel Sergeant × Worm-Fact Keeper | **Curse Requisition** | A cast may be paid for by consuming a curse you landed, 3 a fight. | fighter | `per_fight`, `worth`, `relist`, `pick` |
| 6 | Funnel Sergeant × Kaklon Patent | **Patented Funnel** | The spin banks Funny: 5 a stack a turn. | fighter | `per_stack`, `bleed`, `overflow`, `harvest` |
| 7 | Funnel Sergeant × Top of the Bill | **Opening Number** | Casts in the first 10 seconds cost nothing. | fighter | `window_ms`, `after`, `encore`, `bank` |
| 8 | Worm-Fact Keeper × Kaklon Patent | **Cursed Licence** | An enched component curses on every activation of its frame. | fighter | `stack` (+ kin: `CurseOnActivate`, `Productivity`) |
| 9 | Worm-Fact Keeper × Top of the Bill | **Eleventh Season** | Each curse on the fallen adds 10% to the purse. | purse | `pct`, `count_cap`, `distinct`, `posthumous` |
| 10 | Kaklon Patent × Top of the Bill | **Full Bill** | Two enchs a component, off either licence's list. | board | `racks`, `beacon_pct` (+ kin: `Beacon`, the licence) |

---

## 3. The ten trees

Sixty nodes, six a tree, **2 points each — 12 points to finish one**. Every
tree is two roots that fork and reconverge on a capstone:

```
  A ──► C ──► E ──┐
                  ├──► F  (capstone)
  B ──► D ────────┘
```

Two roots means the first point is a real choice; the reconvergence means
the capstone is the whole tree rather than one branch of it. Data ships in
**`skills.expert.json`**, validated against format 1.

Below: every node, in order, with what it does and why it is in this tree.
Tenths are noted where a knob is fractional.

---

### 3.1 Loud Calculation — *muscle is a mana pool with an exchange rate*

Base: 2.0 strength buys 1 Funny, 40 points a fight, borrowed strength
returns at the bell.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Rate Card** | `rate −4` → 1.6 | The cheapest possible statement of the class: the price of a point of Funny *is* the class, so the first node moves it and nothing else. |
| B | **Standing Order** | `cap +30` → 70 | The other axis. A better rate on 40 points is a small class; the fork is *cheaper* against *more of it*. |
| C | **The Overdraft** | `floor +20` | You may buy 20 points past empty. Strength never falls below 1 — the refusal is in the knob rather than in a special case, and the last point of you is not for sale. |
| D | **The Rebate** | `rebate +50` | A kill refunds half the bill in strength, at the bell. Rewards spending the whole order in a fight you win fast, which is the Gorillathon half talking. |
| E | **Shouted Figures** | `rate −4` → 1.2 | The A branch compounding. Two rate nodes and no third: 1.0 lives in the capstone, so the branch cannot reach parity alone. |
| F | **Both Books Open** | `rate −2`, `cap +30` | 1.0 and 100 points. One strength, one Funny — the exchange stops being an exchange, which is the promise's end state and costs all 12 points. |

### 3.2 Standing Fact — *a curse that cannot expire, if you are carrying little enough to hold it*

Base: curses landed while wearing ≤ 2 items never expire; 1 may be held.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Count** | `worn +1` → 3 | The condition, loosened. The Gorillathon parent's rule is "fewer items"; this tree's only lever on it is *how few*. |
| B | **The First Fact** | `carry +1` → 2 | The cap, raised. Without a cap a permanent curse compounds without bound; the tree spends points to lift it rather than shipping it lifted. |
| C | **Writ in the Margin** | `bite +20` | Permanent curses tick 20% harder. The one knob about the curse rather than the condition, and it needs `worn` first — a harder curse you cannot land is nothing. |
| D | **The Second Fact** | `carry +1` → 3 | Three at once. |
| E | **Bare-Armed** | `worn +1` → 4, `bite +10` | Four items is most of a build, so by the fifth node the class stops being a naked build — which is what buying it is for. |
| F | **Told Again** | `told +1` | A permanent curse outlives the fight and lands on the next creature you meet, once, before it acts. The only node in the block that reaches past the bell, and it is the pair in one sentence: the Keeper's permanence, the Gorillathon's opening. |

### 3.3 Overwound Arm — *an empty frame spins anyway* — **Drop Duchy**

Base: an empty frame banks half a stack a turn into bare strength; ceiling 8.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **Dry Bearing** | `half +3` → 0.8/turn | Rate. |
| B | **The Empty Bench** | `ceiling +6` → 14 | Depth. The same two-axis fork as §3.1 and for the same reason. |
| C | **The Clearing** | `grants Rule::Spread { every_turns: 2 }` | **The Drop Duchy node.** Every 2 turns, one bare cell of a spinning empty frame that touches worked underlay *becomes* that underlay. This is Drop Duchy's Farm-turns-Plains-into-Fields, and its Wood Clearer in reverse: buildings rewrite the terrain around them, retroactively, and adjacency decides what you get. GM2D already has the terrain half — `PieceKind::Enchantment` is an underlay laid *beneath* the grid, and `EffectKind::PerOverlappingCore` already pays "for each item built on top of it." Nothing had ever *changed* the underlay. Now an empty frame does, slowly, so an empty frame is not dead space but ground being prepared for the next time you pack. |
| D | **Counterspin** | `carry +2` | Two stacks survive an activation. Straight off `Rule::SpinKeep`'s tuning, which the Patent parent already owns. |
| E | **Both Arms** | `half +5` → 1.3/turn | An empty frame now out-spins a packed one, which is the joke the class is making, and the tree does not explain it. |
| F | **Overwound** | `ceiling +10`, `half +2` | 24 stacks at 1.5 a turn. |

### 3.4 Short Programme — *the window is as wide as you are bare*

Base: the Showstopper window is 10s + 1s per empty frame, paying 50%.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Poster** | `pct +10` → 60% | What a paid win pays. |
| B | **Quick Change** | `per_slot +1` → 2s | How wide the window gets. Rate against reach, again. |
| C | **The House Clock** | `floor_ms +2000` → 12s | The base window — the only knob that helps a *packed* build, and so the branch for a player who took the class and then geared up anyway. |
| D | **The Second House** | `streak +1` | Each consecutive paid win adds 1s to the next window, capped at 5; a slow win puts it back to nothing. The one node with memory across fights, and the cap is why it is not a runaway. |
| E | **No Encore** | `pct +15` → 75% | |
| F | **The Short Programme Itself** | `per_slot +1`, `pct +25` | Double the purse on a fast naked win, at 3s a frame. Five empty frames is a 27-second window, which is most fights, which is what 12 points should buy. |

### 3.5 Curse Requisition — *pay for a cast with a curse*

Base: 3 a fight; a consumed curse covers one cast; the cheapest goes first.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Form** | `per_fight +2` → 5 | Volume. |
| B | **Ripest First** | `pick +1` | Consume the curse with the *least time left* rather than the cheapest, so a requisition spends something about to be lost anyway. A knob that changes a decision rather than a number, and worth a point precisely because it is free value afterwards. |
| C | **Countersigned** | `relist +1` → every 3rd | Every third requisition puts the curse back, still running. The engine already re-lands curses; this schedules it. |
| D | **Two Signatures** | `worth +50` | A spent curse covers 150% of a cast and the excess banks as Funny — the Sergeant's ledger and the Keeper's feeding each other, which is the pair. |
| E | **Standing Requisition** | `per_fight +3` → 8 | |
| F | **The Whole Book** | `relist +1` → every 2nd, `worth +50` | Every second curse comes back and each covers two casts. The requisition is now cheaper than the cast. |

### 3.6 Patented Funnel — *the spin banks Funny* — **Drop Duchy**

Base: 5 Funny per spin stack per turn.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Tap** | `per_stack +3` → 8 | |
| B | **The Second Licence** | `grants Rule::SpinEvery { ms: 800 }` | An existing rule, unmodified, off the Patent's own tree. More turns is more Funny, so it reaches the promise without being new — *nothing is invented in combat for a class*, and here that is literal. |
| C | **Full Rows** | `grants Rule::RowHarvest { per_cell: 4 }` | **The second Drop Duchy node, and the more direct lift.** Drop Duchy's rows *harvest and do not clear* — a completed line converts its terrain into resources and stays exactly where it is, which is the inversion of Tetris that makes it a builder rather than a puzzle. GM2D packs five grids and has never once rewarded a row for being whole. Now a row with no gap in it pays 4 Funny a cell at the bell and is not disturbed. It re-prices packing: a 6-wide row is 24 Funny for filling the corner you would have left, and M12.3 made a row a thing you *buy*, so this pays that purchase back. |
| D | **The Flywheel Brake** | `bleed +2` | Two stacks survive an activation. |
| E | **The Overflow Pipe** | `overflow +1` | Funny past the cap lands as armour, 1 for 1 — the only sink for a class that will otherwise bank more than it can spend, and it sits on the branch that generates the most. |
| F | **The Funnel, Patented** | `per_stack +4` → 12, `harvest +2` | 12 a stack a turn and rows paying 6 a cell. |

### 3.7 Opening Number — *the first ten seconds are free*

Base: casts cost nothing for 10s; full price after.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **Longer Overture** | `window_ms +3000` → 13s | |
| B | **The Standing Discount** | `after +20` | 20% off *after* the window. The branch that admits some fights are long, which is the honest half of a class built on short ones. |
| C | **Encore** | `encore +1` | A kill inside the window restarts it. Once. Reaches the Showstopper parent without touching the purse — this class's fast win buys *time*, not money, which is what separates it from Short Programme. |
| D | **Unspent** | `bank +1` | Funny unspent when the window closes lands as armour. Rewards the player who did not need the free casts, which is otherwise the one build this class fails. |
| E | **Second Overture** | `window_ms +4000` → 17s | |
| F | **The Whole First Act** | `encore +1`, `after +30` | Two encores and half price after. Two kills inside the window is a 51-second first act, and a long fight is half price for the rest of it. |

### 3.8 Cursed Licence — *an enched component curses on every activation* — **Factorio**

Base: enched components land searing on activation; 1 curse a hit.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Searing Clause** | `grants CurseOnActivate { weapon, searing }` | The class's own rule, aimed at a frame. Four of six nodes are this rule with different arguments, and that is deliberate: the Keeper's tree already shipped one `CurseOnActivate` node, so the expert is that node's *licence* rather than a new mechanic. |
| B | **The Frost Clause** | `grants CurseOnActivate { gloves, frost }` | |
| C | **Second Reading** | `grants Rule::Productivity { every: 3, slower_pct: 15 }` | **The Factorio node.** Every third activation of an enched item lands its curse **twice**, and the item runs **15% slower** for good. This is Factorio's productivity module exactly: more output per operation, bought with speed, so it is never a free upgrade and it is the one module you have to think about. GM2D has `power` enchs and `haste` enchs and has never made you choose between them; a `haste` ench and this node are now a real pairing rather than two upgrades. |
| D | **The Stun Rider** | `grants CurseOnActivate { helmet, stun }` | Stun is capped at 3.6s by `STUN_CAP_MS`, so a helmet firing every 2s cannot lock a fight — the existing cap is why this node needs no new one. |
| E | **The Misfire Schedule** | `grants CurseOnActivate { greaves, misfire }` | |
| F | **The Licence Entire** | `stack +1` | Every enched activation lands its curse **twice**, everywhere, always — Second Reading's every-third made unconditional. The capstone generalises a node from the middle of its own tree, which is the shape of the whole class: one clause, then the whole schedule. |

### 3.9 Eleventh Season — *the fallen are billed by the curse*

Base: 10% a curse, up to 4 counted.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Ledger Column** | `pct +5` → 15% | |
| B | **The House Count** | `count_cap +2` → 6 | Rate against ceiling. A purse class has exactly these two knobs and pretending otherwise would be padding. |
| C | **Four Kinds** | `distinct +1` | Four *different* kinds on one fallen creature pays double. There are four curse kinds; this is the only thing in the game that asks you to land all of them, and the Keeper's tree is where you get them. |
| D | **The Long Run** | `posthumous +5` | A curse that expired before the bell counts half. Rescues the long fight, which otherwise pays this class least — and argues with Standing Fact about expiry from the other side. |
| E | **The Billing Review** | `pct +5` → 20% | |
| F | **Eleven Seasons of It** | `pct +10` → 30%, `count_cap +3` → 9 | Nine curses at 30% is 370% of a purse, doubled to 740% with four kinds. The largest number in the game, and it wants nine curses standing at the bell, which is a build and not an accident. |

### 3.10 Full Bill — *two enchs a component, off either list* — **Factorio**

Base: 2 enchs a component; both licences' lists.

| # | Node | Effect | Why |
|---|------|--------|-----|
| A | **The Second Rack** | `racks +1` → 3 | |
| B | **The Standing Order** | `gives_ench the-yodregar-index` | A tree handing over an ench is shipped behaviour (`Effect::GivesEnch`), and for this class an ench *is* the promise — so a licence node is on-topic where a stat node would not be. |
| C | **Ponkey Broadcast** | `grants Rule::Beacon { pct: 40 }` | **The Factorio node.** An enched component lends 40% of each ench it carries to every finished item orthogonally touching it in the same grid. This is Factorio's beacon: modules in one machine reaching the machines around it at reduced strength, which is the mechanic that turns a factory from a list of buildings into a *layout*. GM2D's packing board is already a layout and its adjacency is already computed — `loadout.rs` walks orthogonally-connected groups, `Trigger::PerAdjacentItem` reads neighbours, `Axis::Weave` scores this exact thing. What was missing is any reason for *where a finished item sits relative to another finished item* to matter to an ench. Now one enched core is worth more in the middle of a packed chest than in a corner, and the packing screen gets a reason to repack that is not fit percentage. |
| D | **The House List** | `gives_ench the-hooper-s-allowance` | |
| E | **Wider Broadcast** | `beacon_pct +25` → 65% | |
| F | **The Bill in Full** | `racks +1` → 4, `beacon_pct +15` → 80% | Four enchs a component lending 80% to every neighbour. A tightly packed chest is one where every item is carrying most of five racks, and the corner cell you left is the reason it is not. |

---

## 4. What the engine has not got yet, and what it grows

Five additions. One effect kind and four rules, and every one is read by
more than the node that grants it.

### 4.1 `Effect::Tunes` — the tree moving its own power's numbers

```rust
/// A named integer on the live power, moved by `by`.
///
/// **The seventh effect kind, and the first that cannot be read without
/// knowing which class you are in.** Every other effect changes the
/// character; this one changes *the class's own sentence*, which is what an
/// expert tree is for and what stops it becoming a second base tree.
///
/// The knob is a `Name` for the reason `CurseOnActivate`'s slot is: the
/// vocabulary lives with the power that declares it, and a second enum
/// listing rate-cap-floor-rebate would be two lists to keep in step.
/// `ClassPower::knobs()` is the list, `SkillsData::parse` checks against it,
/// and a tree naming a knob its own class has not got does not load.
///
/// Tenths where a knob is fractional — `rate: 20` is 2.0 strength a point.
/// Everything shown to a player is printed by `describe()` from the tuned
/// value, so the promise re-reads itself after every point spent and cannot
/// go stale. That was already the rule for the base classes; it is now the
/// rule for twelve points of tuning.
Tunes { knob: Name, by: i32 },
```

Lints: `every_expert_knob_is_declared` (parse-time, above) and
`every_declared_knob_is_moved_by_some_node` — a knob no node touches is a
knob that should have been a constant.

### 4.2 `Rule::Spread { every_turns }` — Drop Duchy

```rust
/// A spinning empty frame converts one bare cell of underlay, once every
/// `every_turns`, into the underlay kind orthogonally next to it.
///
/// **The first rule that writes to the board during a fight**, and it is
/// deliberately the slowest thing in the game: one cell, every other turn,
/// on a frame with nothing in it. The underlay layer has been readable
/// since it shipped (`PieceKind::Enchantment`, `PerOverlappingCore`) and
/// nothing has ever changed it.
///
/// Bounded by the frame — it never crosses a grid — and by needing a worked
/// neighbour to copy, so an empty frame with bare underlay throughout
/// spreads nothing and says so.
Spread { every_turns: u32 },
```

### 4.3 `Rule::RowHarvest { per_cell }` — Drop Duchy

```rust
/// Every completely filled row, in every grid, pays `per_cell` at the bell.
///
/// **The row is not cleared.** That is the whole borrowed idea: a filled row
/// is a machine, and clearing it would be taking the machine apart. Read by
/// `fight::settle`, where things that happen at the bell already happen, and
/// it counts rows off the loadout rather than off the packing screen — a row
/// is full or it is not, and the screen has no opinion.
RowHarvest { per_cell: u32 },
```

### 4.4 `Rule::Beacon { pct }` — Factorio

```rust
/// An enched component lends `pct` of each ench it carries to every finished
/// item orthogonally touching it, in the same grid.
///
/// Lending is not spending: the lender keeps what it has. Adjacency is
/// edge-sharing between *finished items*, which `loadout.rs` already walks
/// for its groups and `Axis::Weave` already scores, so this adds a reader
/// and not a geometry.
///
/// **It does not chain.** A lent ench is not an ench for the purpose of
/// lending it on, or a packed chest would broadcast itself to a fixed point
/// and the fixed point would be the game.
Beacon { pct: u32 },
```

### 4.5 `Rule::Productivity { every, slower_pct }` — Factorio

```rust
/// Every `every`th activation of an enched item does its thing twice, and
/// the item runs `slower_pct` slower for good.
///
/// The trade is the point and it is Factorio's: output bought with speed, so
/// it is the one upgrade you have to think about rather than take. The
/// engine has `power` enchs and `haste` enchs and has never made you choose
/// between them; this is the first thing that spends one to buy the other.
///
/// Deterministic, like `Misfire` and for the same reason: every test in the
/// suite replays a fight and expects the same answer.
Productivity { every: u32, slower_pct: u32 },
```

---

## 5. Files

| File | Change |
|---|---|
| `crates/core/src/class.rs` | `tree_finished`; `OFFERED_SECOND(first) -> [&ClassDef; 4]`; `EXPERTS: [(pair, ClassDef); 10]`, pair order-insensitive; ten `ClassPower::Expert(_)` variants; `knobs()` and a knob-aware `describe()`. |
| `crates/core/src/character.rs` | `second_class`, `expert`, `second_paper` (serde default); `classes()`; `choose_second_class`, `take_expert`; knob resolution folded over taken nodes. |
| `crates/core/src/skills.rs` | `Effect::Tunes`; parse-time knob check; `cost` unchanged. |
| `crates/core/src/rule.rs` | `Spread`, `RowHarvest`, `Beacon`, `Productivity`, and their arms in `check()` and `line()`. |
| `crates/core/src/reward.rs` | `bounty_with_class` takes the iterator; Short Programme's window and Eleventh Season's column argue here. |
| `crates/core/src/combat.rs` | Beacon in profile assembly; Productivity on the activation clock; Spread on the spin tick; folds where it read one class. |
| `crates/core/src/fight.rs` | `RowHarvest` at the bell; Standing Fact's `told` carried into the next encounter. |
| `crates/core/src/shop.rs` | `StockGate`; three papers on Spike's shelf; refused lines drawn, priced and reasoned. |
| `data/skills.json` | Append the ten trees from `skills.expert.json`. |
| `data/theme.td.json` | Ten names, ten promises, sixty node names and blurbs, three paper lines, Spike's two refusals. |
| web shim | Four-card fork mode with Escape allowed; the expert tab; three shelf lines, two of them greyed. |

---

## 6. Tests and lints

- `every_offered_class_reaches_something` over all **fifteen** — behavioural, unchanged: the purse moves, the fighter differs, or the board differs.
- **`expert_nodes_touch_only_the_expert`** — §1.6, and the block's own rule. Every node of every expert tree is a `Tunes` of a declared knob, a `Grants` of a kin rule, or a `GivesEnch` inside the licence. Reads the raw JSON, as `tests/tone.rs` does.
- `every_expert_knob_is_declared` / `every_declared_knob_is_moved_by_some_node`.
- `no_pair_of_live_powers_disagrees` — resolution is order-independent across every reachable {first, second, expert} triple.
- `beacon_does_not_chain`; `spread_stays_in_its_frame`; `row_harvest_leaves_the_row`; `productivity_is_deterministic_on_replay`.
- `second_tree_nodes_cost_two`; `finished_is_every_node` (9 of 10 does not unlock); `refused_paper_is_still_drawn`; `a_refusal_names_the_count`.
- `paper_survives_escape` — buy, Escape, reload: it is in the pack and re-raises; choosing consumes it.
- `old_save_is_asked`; and no path clears `class`, `second_class` or `expert`.
- Integration: level 1 → class → finished → 5,000 Fnorp → second class → finished → expert → capstone, on the real walk.

---

## 7. Numbers, in one place

| Thing | Number |
|---|---|
| Second Paper | 5,000 Fnorp, flat, same as the Patent |
| Expert paper | 0 Fnorp, unlocked at two finished trees |
| Papers visible from | the first visit, refused with a reason |
| Second-class / expert node | 2 points |
| Expert tree | 6 nodes, 12 points, 2 roots, 1 capstone |
| Expert classes | 10 = C(5,2) |
| New nodes this block | 60 |
| New effect kinds | 1 (`Tunes`) |
| New rules | 4 (`Spread`, `RowHarvest`, `Beacon`, `Productivity`) |
| Classes on one character, maximum | 3 |

---

## 8. Still open

1. **Where Spike stands** for the second and third papers — the same van, or a counter in a town? The shelf logic does not care; the walk does.
2. **`told`** (Standing Fact's capstone) is the one knob that crosses an encounter boundary. It wants a save field, and it is the only thing in this block that does.
3. Whether the expert paper's line prints the promise **or** the promise and the six node names. The first is a goal; the second is a plan, and a plan on a shelf may be too much shelf.
