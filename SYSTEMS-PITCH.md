# Three more benches — systems in brewing's shape, each with a specialization

*Written against `40e3121` (M20 through M20.9 landed, the cart on the flat,
five more enchs). Brewing as built is the template: a second bag that opens
only in town (`WorldState::larder`), things in it that have a **shape** but are
not components (no `CATALOG` entry, no fingerprint move), a **bench** in the
town screen that is a bizarre-shaped grid, a **pairing table** a lint proves
complete (twenty-eight brews for eight ingredients), an output that arrives
through `Held` at the bell and expires with the fight, and a
**specialization** — `ClassKind::Specialization`, one slot, no experts — whose
tree is `tunes` on the bench's own knobs (`extra`, `potency_pct`). Three
systems below are built to that template on purpose, so each is mostly data
and a bench, and each has somewhere honest for its specialization to live.*

---

## 0. What makes a system "like brewing"

Reading `brew.rs` and `PLAN-M20.md`'s decisions, the template is five things,
and a proposal that lacks one is a feature rather than a system:

| the part | brewing's | the rule it obeys |
|---|---|---|
| **a second bag** | the larder; opens in town only | not the inventory, not a component, no fingerprint move |
| **a thing with a shape** | eight ingredients, `Shape`, one per art family | the same polyomino a component is, so a bench cell is a board cell |
| **a bench** | the retort — seven cells, no symmetry | the arrangement carries information; a tidy box would be an inventory slot |
| **a table a lint proves complete** | `C(8,2)` = 28 brews | a pair nobody authored is a pair nobody can make |
| **one door into combat** | `Held` at the bell | zero new combat code; expires with the fight |
| **a clock** | none — a brew is made when you have the pair | (the three below each add one, because each is about *time*) |

And a specialization is a tree of `tunes` on the bench's knobs, held one at a
time, pairing with nothing.

---

## 1. The Plot — *things that grow while you fight*

**From Stardew Valley and Rune Factory** — a plot of beds in town, seeds
from the world, crops that take days, and companion planting where what
grows beside what decides the yield. Rune Factory is the one that put the
farm inside an RPG's loop rather than beside it; Stardew is the one that made
adjacency (sprinklers, scarecrows, trellises) the puzzle.

### 1.1 The system

- **The bag:** the **seed drawer**, in town only. **Eight seeds**, one per art
  family, dropped alongside the ingredient at a lower per-mille — the same
  `from` derivation `brew.rs` uses, so a creature cannot arrive without one.
- **The shape:** a seed has none; a **crop** does. Planted, it occupies a
  polyomino of the bed that grows with it: a `1×1` sprout, a `2×1`, then its
  full shape at harvest — three stages, and the shape at each is in
  `data/plot.json`.
- **The bench:** **the bed**, one per town you have stood in, and every town's
  bed is a different bizarre shape (Kettleworks' is long and gapped like a
  rind wall; the End of All Gears' is round-ish with a stone in it). You plant
  at a cell and the crop needs room to reach its harvest shape; a crop that
  runs out of bed is stunted and harvests at stage two.
- **The clock:** **a crop grows one stage per bell** — per fight *won*, anywhere.
  The game has no days; it has fights, and the Plot is the first thing in it
  that rewards you for what you did between visits to town. Seven wins is a
  harvest.
- **The table:** **companion planting** — `C(8,2)` = 28 pairs of crops that
  touch edge-on at harvest, each pair a **yield modifier** (double, potency up,
  a second ingredient, an ench seed). `PerAdjacent` already exists on the
  board; this is it on a bed. A lint proves all 28 authored.
- **The output:** a harvest goes **into the larder** as ingredients — the Plot
  feeds the retort, so brewing gets a second source that is not fighting the
  creature — and, for six of the twenty-eight pairs, drops **an ench seed**,
  which grows once more into one of the barrel's enchs. That is the Plot's own
  door into the character, and it is the barrel's door, not a new one.

### 1.2 The specialization — **the Grower**

`SpecPower::Grower { stages: 3, beds: 1, yield_pct: 100 }`. The tree is
`tunes`:

| node | effect | what it is |
|---|---|---|
| The First Furrow | `yield_pct +20` | |
| A Second Bed | `beds +1` — two towns' beds at once | |
| Forced Under Glass | `stages −1` — harvest at two bells | the Grower's whole point: fewer fights per harvest |
| The Long Row | `bed_cells +3` — every bed gains three cells, placed by the map | room for the big shapes |
| Companion, Thrice | `pairs_reach +1` — a crop counts its diagonal neighbours | the adjacency table read wider |
| The Whole Allotment | `beds +1`, `yield_pct +30` | |

### 1.3 What it costs, and the risk

`plot.rs` beside `brew.rs`; `WorldState::seed_drawer`, `beds: Vec<Bed>`; one
tick in `settle` on a win; a bed screen that is the retort screen with
growth stages drawn. No combat code. **The risk is the clock**: seven wins
for a harvest at the run's pace is twenty minutes, and at a new player's is a
session. The number is a constant with a glossary shelf, and the Grower's
`stages −1` is why the specialization exists.

---

## 2. The Kennel — *the creature you beat enough times comes with you*

**From Dragon Quest V**, where a defeated monster sometimes gets up and asks
to join, and **Shin Megami Tensei**, where what you have done to a demon's
kind is the negotiation. The tally is already in the game: `World` keeps how
many times each creature has been *met, won or lost or walked away*, and
`simulate_party` has run fights for a party since M-something with nobody in
it but you.

### 2.1 The system

- **The bag:** **the kennel**, in town only. A creature enters it when you
  have beaten it **five times** and then beat it once more while its tile is
  the landing — the sixth win is the offer, on the creature's own card, in
  its own register: *it does not run. It stands there, counted.* One creature
  a family, eight kennel places, so the kennel is `C(8, ·)` shaped like the
  larder.
- **The shape:** a kennelled creature is drawn as its art family's silhouette
  as a polyomino — the same TikZ families M19.14 cut — and that shape is what
  it takes up in **the run** (the bench).
- **The bench:** **the run** — a yard grid, bizarre-shaped, in the town you
  kennelled it in. A creature fits the run or it does not; two creatures that
  touch edge-on in the run are **companions** and fight better together.
  This is the retort with something that growls in it.
- **The clock:** a kennelled creature **eats one ingredient per fight it
  fights in**, off the larder, matched by family — a Kettle Scale for anything
  the works made. No ingredient, no creature at the bell. The larder is now
  fuel as well as glass, and the Plot (§1) is why you would grow it.
- **The table:** **companions** — `C(8,2)` = 28 pairs of families that touch in
  the run, each a pair bonus (the crimper beside the behemoth: the behemoth
  is slower and the crimper hits for it). A lint proves all 28.
- **The output:** **one creature fights beside you** — `simulate_party` with
  a second `Combatant` built from the creature's own `MonsterSpec`, at the
  bracket difficulty's stats, wearing what it wears. It takes the *second*
  target when there are two and the same one when there is one. It levels
  with the tally: every ten more wins together is `+5%` to its stats. It can
  be *lost* — a fight you lose while it is out puts it back in the kennel at
  zero, and the card says so.

### 2.2 The specialization — **the Handler**

`SpecPower::Handler { offer_at: 5, mouths: 1, tally_pct: 5 }`:

| node | effect | what it is |
|---|---|---|
| The Fifth Count | `offer_at −1` — the offer at four | |
| A Second Lead | `mouths +1` — two creatures out at once | the only way to field two |
| Fed From the Hand | `feed_pct −50` — a creature eats every other fight | |
| The Long Yard | `run_cells +3` | |
| Counted Together | `tally_pct +5` | every ten wins is +10% |
| The Whole Pack | `mouths +1`, `offer_at −1` | three out, and offers at three |

### 2.3 What it costs, and the risk

`kennel.rs`; `WorldState::kennel`, `run: Run`; the sixth-win offer in
`settle`; `simulate_party` fed a second combatant — **this is the one of the
three that touches combat**, and it does so through a function that already
exists for it. The risk is balance: a boss's stats on your side is a boss on
your side, so a kennelled creature is capped at the *region's* bracket, not
its own, and the cap is a constant with a shelf. The Tenth Surveyor and the
other boss creatures do not stand in region pools for farming and are not
offerable; `no_boss_is_kennelled` is the lint.

---

## 3. The Stall — *the empty town gets shelves, and they are yours*

**From Moonlighter and Recettear** — you are the shopkeeper as well as the
adventurer; what you bring back goes on a shelf; who buys it and for how much
is the game. Moonlighter's dungeon-then-shop loop is exactly this game's
walk-then-town loop with a counter added; Recettear is the one that made
*pricing* the puzzle.

### 3.1 The system

- **The bag:** none new — **the shelf is the bag**, and it is the barrel's
  cousin. The third town has had no shelves since M14 on purpose; this is
  what goes in it. You **stock** the stall from your tray: a component goes on
  the shelf and off your board.
- **The shape:** a component's own — the stall is the first thing outside the
  five grids where a real `CATALOG` piece is placed by shape, and the shelf is
  a bizarre-shaped grid where only what fits is for sale.
- **The bench:** **the counter**, in the third town only. Shelf, ledger, and a
  **price card** per item: you name a price against the item's rating, and
  the card prints how that compares to the barrel's.
- **The clock:** **customers come per bell** — each fight won anywhere brings
  one customer to the stall, drawn from the town's pool of eight *buyers*, one
  per art family, and a buyer wants what their family wants (the works buy
  cogs; the bog buys inks). A fair price sells; a high one sells one time in
  three and pays; a low one sells at once and the ledger says what you left
  on the counter. Unsold stock is stock.
- **The table:** `C(8,2)` = 28 **buyer pairs** — two buyers in a row who are
  each other's kin pay a **bargain**, an ench or a component that is not on
  the barrel, and the pair is printed on the ledger when it happens. A lint
  proves all 28.
- **The output:** **Fnorp**, which was the point, and the bargains, which are
  the barrel's kind of thing through the barrel's own `EVENT_ONLY` door. And
  one more: **a stocked shelf is a shelf**, so a player who comes back to the
  third town after a long walk finds their own goods for sale to themselves
  at the price they set — the only shop in the game whose prices are the
  player's.

### 3.2 The specialization — **the Factor**

`SpecPower::Factor { customers_per_bell: 1, margin_pct: 0, shelf_cells: 9 }`:

| node | effect | what it is |
|---|---|---|
| Open Early | `customers_per_bell +1` | two a fight |
| The Thumb on the Scale | `margin_pct +10` — every sale pays a tenth over | |
| A Longer Shelf | `shelf_cells +4` | |
| Known in the Trade | `bargain_pct +25` — pairs come round more | |
| The Second Counter | `stalls +1` — a stall in Kettleworks too | |
| The Whole Ledger | `margin_pct +15`, `customers_per_bell +1` | |

### 3.3 What it costs, and the risk

`stall.rs`; `WorldState::stall`; a customer tick in `settle`; the counter
screen, which is the town screen with a shelf you write on. **No combat
code at all** — it is the only one of the three that never reaches the bell.
The risk is the economy: `bounty_with_class` and the barrel already argue
about what Fnorp is worth, and a stall that prints it is a third voice.
The margin is capped, the customer count is capped, and both are shelves in
the glossary.

---

## 4. Side by side

| | **The Plot** | **The Kennel** | **The Stall** |
|---|---|---|---|
| from | Stardew Valley · Rune Factory | Dragon Quest V · Shin Megami Tensei | Moonlighter · Recettear |
| the bag | the seed drawer | the kennel | the shelf (no new bag) |
| the shape | a crop's, by stage | the family silhouette | the component's own |
| the bench | the bed, one per town | the run | the counter, third town |
| the clock | a stage per win | an ingredient per fight | a customer per win |
| the 28 | companion planting | companions in the run | buyer pairs |
| lands in | the larder; the barrel's enchs | **the fight**, via `simulate_party` | the purse; `EVENT_ONLY` bargains |
| touches combat | no | **yes** — a second combatant | no |
| feeds / is fed by | feeds brewing | eats brewing (and the Plot) | eats the tray |
| the specialization | Grower | Handler | Factor |
| what it answers that is open | why grow anything: brewing's supply | what `simulate_party` was for | what the third town is for |
| the risk | the clock's pace | balance of a boss on your side | a third voice on Fnorp |

**A note on order.** They chain: the Plot supplies the larder, the Kennel
eats it, the Stall sells what the Kennel and the walk bring back. Built in
that order each one has something to consume from the last, and a player who
takes the Grower first is not choosing against a system that does not exist
yet. Built in any other order, the Kennel arrives hungry.

**Also considered, and why not here.** *Trophies* (Monster Hunter; Skyrim's
Hearthfire) — a mantel of boss shapes paying standing bonuses — is a fourth
that lands in `Held` like brewing does, and it is the one to build if the
three above are too much; it is left out only because two of them already
answer the empty town and the larder, and a mantel answers neither.
