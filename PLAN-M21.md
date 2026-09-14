# PLAN-M21 — Three benches: the Plot, the Kennel, the Stall

**The frame.** `PLAN.md` wins where it and this disagree, and a divergence from
this goes in `CLAUDE.md`'s table with its reason, in the commit that makes it.
Written against `40e3121`: M20 through M20.9 landed, the cart on the flat, five
more enchs, twenty-nine maps, brewing as the template.

## What was asked for

> all these ideas are great actually, write out a plan and a prompt for a model
> in claude code to execute this; for any new sprites, use the latex sprite
> making prompt, and for new ench ideas, look for games as inspiration for new
> ideas. i want the plan to be written out as milestones, with different
> deliverables per milestone, and after each milestone, a summary table is
> output showing how far along the work is. the model should not ask for
> approval at any time. it should also keep a notebook of second order effects
> that it notices, and add additional milestones at the end to address those
> second order effects

The three ideas are `SYSTEMS-PITCH.md`'s: **the Plot** (Stardew Valley, Rune
Factory), **the Kennel** (Dragon Quest V, Shin Megami Tensei), **the Stall**
(Moonlighter, Recettear). Each is a system in brewing's shape — a bag that
opens in town, a thing with a shape that is not a component, a bizarre-shaped
bench, a `C(8,2)` table a lint proves complete, one door into the character,
a clock — and each carries one specialization.

## Decisions taken before anything is built

1. **Nothing here is a component.** Seeds, crops, kennelled creatures and buyers
   are shapes in their own modules, as ingredients are in `brew.rs`. The save
   fingerprint does not move; a player mid-run keeps their run. The Stall is the
   one place a real `CATALOG` piece is placed by shape outside the five grids,
   and it is placed *from the tray*, so it is still the same piece.
2. **Build order is Plot → Kennel → Stall, and it is not negotiable.** The Plot
   feeds the larder; the Kennel eats the larder; the Stall sells what comes
   back. Built in this order each system has something to consume from the
   last. Built in any other order the Kennel arrives hungry, which is a system
   that looks broken on the day it ships.
3. **The clock is the bell.** The game has no days. A crop grows a stage per
   fight won; a customer comes per fight won; a kennelled creature eats per
   fight fought. Every constant on these clocks is a named `pub const` with a
   glossary shelf, because the pace is the risk in all three and a number in a
   shelf is a number the human can point at.
4. **One door each.** The Plot lands in the larder and, for six pairs, in the
   barrel's ench list. The Kennel lands in `simulate_party` as a second
   `Combatant`. The Stall lands in the purse and, for bargains, in `EVENT_ONLY`.
   No new combat code except the Kennel's second combatant, which goes through
   a function that has existed for it since the party fixtures.
5. **Three specializations, one slot.** `ClassKind::Specialization` already
   holds one and pairs with nothing. The Grower, the Handler and the Factor
   join the Apothecary as four papers on the same counter; a player holds one.
   Every node is `tunes` on its own bench's knobs, which is `expert_nodes_touch
   _only_the_expert`'s rule read across to specializations, and the lint is
   widened rather than copied.
6. **Every `C(8,2)` is authored and proved.** Twenty-eight companion plantings,
   twenty-eight run companions, twenty-eight buyer pairs. `every_pair_is_authored`
   is one generic lint over three tables. A pair that exists only in a
   creature's silhouette and not in a table is a pair nobody can make.
7. **Sprites are TikZ, from the prompt, and nothing else.** `CLAUDE.md`'s
   *TikZ or nothing* rule holds. Every new figure is `art/*.tex` written by
   filling in `tikz_figure_prompt.md` with the game's conventions in
   `behemoth.tex` (`\Unit`, `\Edge`, `main`/`dark`/`accent`/`ink`, primitives
   only) and compiled by `make art`. The figures this block needs are counted in
   §M21.10 and no raster is produced anywhere.
8. **New enchs come from other games, and each one says which.** Six new ench
   effect kinds, each the smallest version of a mechanic another game made
   famous, each landing as a `Plot` ench seed or a `Stall` bargain so there is a
   reason it is new. §M21.6 lists them with their sources.
9. **The builder does not ask.** Every decision the plan leaves open has a
   recommendation beside it, and the builder takes the recommendation, says so in
   the commit, and records it in the notebook. There are no deploy-point
   questions: the builder deploys at the two points named and reports.
10. **The notebook is a worklist.** `SECOND-ORDER-M21.md` is written as the work
    goes, in `SECOND-ORDER-M20.md`'s format — numbered rows, `open`/`closed` —
    and **M21.12 onward are milestones that do not exist yet**: the builder
    writes them from the notebook's open rows when it gets there, numbers them,
    executes them, and closes the rows.
11. **A status table after every milestone.** The shape is fixed (§Status
    table) and it is appended to `MILESTONES.md` under an `M21` heading after
    every milestone's commit, with the test count and the commit hash, so the
    human can read how far along the work is without reading a commit.

---

## Milestones

Order: **M21.0 → M21.11**, then whatever the notebook adds. Deploy after
M21.5 and after M21.11.

### M21.0 — The seed drawer, and eight seeds

*The Plot's bag.*

| deliverable | what |
|---|---|
| `plot.rs` | `Seed` (eight, `from` an art family the way `brew.rs` derives ingredients), `Crop` (a seed planted: `family`, `stage: 0..=2`, `at`), `Bed`, `Stage` shapes |
| `data/plot.json` | eight seeds; per family the three stage shapes — a `1×1`, a `2×1` or `1×2`, and the harvest polyomino (three to five cells, family-distinct) |
| `WorldState::seed_drawer` | a bag, town-only, in the save with `serde(default)` |
| the drop | a seed alongside the ingredient on a win, at `SEED_PER_MILLE` (350), through `pay_a_win`'s own door — not if you already hold six of that seed |
| lints | `every_family_has_a_seed`; `every_stage_shape_grows` (stage n+1 contains stage n's cells) |

Acceptance: `a_win_drops_a_seed_at_its_rate`; `the_drawer_opens_only_in_town`;
`a_save_without_a_drawer_opens`. **Nothing a player can see.**

### M21.1 — The bed

*The Plot's bench and its clock.*

| deliverable | what |
|---|---|
| `Bed` per town | a bizarre-shaped mask per town, in the map file as `bed: [[x,y],...]` — Kettleworks long and gapped, the End of All Gears round with a stone in it, the third town's the largest. Three beds, eleven to fourteen cells each. |
| planting | `Game::plant(town, seed, at)` — refuses if the harvest shape would not fit from `at`; a stunted plant is not a thing this game has, so the refusal names the cells it lacks |
| the clock | `plot::tick` in `settle` on a win: every crop everywhere grows one stage; `STAGES = 3`; a stage-2 crop is ready |
| harvest | `Game::harvest(town, at)` → ingredients into the larder, `HARVEST_YIELD` (2) per crop, family-matched |
| the screen | the town screen gains **the bed** beside the retort: the mask drawn like the retort, crops drawn at their stage, a *plant* and a *pull* control, and the tape line *the works' row came up* |

Acceptance: `a_crop_grows_one_stage_per_win_anywhere`; `a_harvest_shape_that_
does_not_fit_is_refused_by_cell`; `a_harvest_fills_the_larder`; `two_beds_grow_
at_once`.

### M21.2 — Companion planting, and the ench seeds

*The Plot's table and its second door.*

| deliverable | what |
|---|---|
| `data/plot.json` gains `companions` | twenty-eight pairs of families; each a `yield` modifier: `double`, `potency +N`, `second_ingredient`, or `ench_seed` |
| adjacency | two crops at harvest stage whose polyominoes touch edge-on; `PerAdjacent`'s test read on the bed |
| six ench seeds | six of the twenty-eight pay an **ench seed**, which is planted like a crop and harvests as one of the six new enchs in §M21.6, straight into `enchs_owned` |
| lint | `every_pair_is_authored` (generic, over `companions`); `six_pairs_pay_an_ench_seed`; `no_ench_seed_pays_an_ench_the_barrel_sells` |

Acceptance: `touching_at_harvest_pays_the_pair`; `not_touching_pays_nothing`;
`an_ench_seed_harvests_an_ench`.

### M21.3 — The Grower

*The Plot's specialization.*

| deliverable | what |
|---|---|
| `SpecPower::Grower { stages, beds, yield_pct, bed_cells, pairs_reach }` | knobs, declared |
| the tree | six nodes as `SYSTEMS-PITCH.md` §1.2, `tunes` only, costs 1/1/2/2/3/4 like the Apothecary's ladder |
| the paper | on the same counter as the Apothecary's; one slot |
| the fork | the specialization card grid at two |

Acceptance: `spec_nodes_touch_only_their_bench` (the widened lint) over both;
`forced_under_glass_harvests_at_two`; `a_second_bed_is_two_towns`.

### M21.4 — The kennel, and the offer

*The Kennel's bag and its clock.*

| deliverable | what |
|---|---|
| `kennel.rs` | `Kennelled { family, spec: &'static str, wins_together, out: bool }`; `WorldState::kennel: Vec<Kennelled>` |
| the offer | on the **sixth** win against a creature (`OFFER_AT = 5` prior wins, read off the tally that already exists), the creature's card gains a choice *Take it along*; one per family; a boss is never offered — `no_boss_is_kennelled` reads `rank` |
| the feed | a kennelled creature that is *out* eats one family-matched ingredient off the larder at the bell; **no ingredient, it stays in** and the tape says so; `FEED_EVERY = 1` |
| the yard | `Run` — a bizarre-shaped mask per town, in the map file as `run: [...]`; a kennelled creature occupies its **family silhouette** as a polyomino (eight shapes, in `data/kennel.json`, cut from the TikZ families' bounding cells) |
| the screen | the town screen gains **the run**: the kennel list, the yard, *put out* / *bring in* |

Acceptance: `the_sixth_win_offers_once_per_family`; `a_boss_never_offers`;
`a_creature_out_eats_at_the_bell`; `an_empty_larder_keeps_it_in`; `a_silhouette
_that_does_not_fit_the_run_is_refused_by_cell`.

### M21.5 — The second combatant

*The Kennel's door into the fight — the one place this block touches combat.*

| deliverable | what |
|---|---|
| `simulate_party` fed | a second `Combatant` built by `Combatant::monster_at(&spec, difficulty)` for the creature out, capped: `KENNEL_CAP` = the **region's** bracket stats, not the creature's own |
| targeting | it takes the second foe when there are two, the same when there is one |
| the tally | `wins_together += 1` on a win; every `TALLY_STEP` (10) is `+TALLY_PCT` (5) to its stats |
| the loss | a fight lost with it out puts it back in the kennel at `wins_together = 0`, and the card says so |
| companions | `data/kennel.json` gains twenty-eight pairs of families that touch edge-on in the run, each a pair bonus expressed in the existing rule vocabulary (`Beacon`, `SpinExtra`, a `stat`); `every_pair_is_authored` over it |
| the replay | the second combatant on the bar, drawn with its family sprite; `Event::Companion` for what it did |

Acceptance: `a_creature_out_fights_and_is_capped_at_the_region`;
`a_lost_fight_resets_it`; `ten_wins_together_pay_five`; `two_touching_in_the_
run_pay_the_pair`; `the_replay_draws_two`. **Deploy.**

### M21.6 — Six enchs from six games

*What the ench seeds grow into and what bargains pay. Each is a new effect kind
on the ench, the smallest honest version of a mechanic another game made
famous.*

| id | effect kind | what it does | from |
|---|---|---|---|
| `the-bramble-coat` | `returns { pct: 35 }` | damage this item's armour absorbs is returned to the attacker at 35% | **Diablo II** — the Thorns aura, and Terraria's Thorns potion: the wall that bites back |
| `the-tithe-ring` | `leeches_mana { pct: 12 }` | 12% of what this item deals comes back as mana | **Diablo II** — mana leech; the item that pays for the next cast |
| `the-one-page` | `once { times: 5 }` | this item fires once a fight, at five times, and is spent until the bell | **Slay the Spire** — Exhaust: a card that is enormous because it is gone |
| `the-first-word` | `ambush { pct: 100 }` | the item's first activation of a fight is doubled; every other one is what it was | **Hades** — the opening-hit boons: the fight is decided in the first second or it is not |
| `the-slow-match` | `coats { curse: "searing" }` | every activation lands searing at its base duration | **Path of Exile** — Ignite: the hit that keeps hitting |
| `the-long-count` | `ramps { pct: 4, cap: 40 }` | each activation adds 4% to this item's power, to 40, for the fight | **Risk of Rain 2** — the stacking-speed items; a fight that gets worse for them the longer it goes |

| deliverable | what |
|---|---|
| `ench.rs` | six `Effect` kinds; `scaled()` arms (`ramps` and `ambush` scale their pct; `once` and `coats` do not, for `Trigger::scaled`'s reason) |
| `combat.rs` | each read where it lands: `returns` in `take_physical`, `leeches_mana` and `ramps` and `ambush` and `once` in the activation path, `coats` beside `CurseOnActivate` |
| `data/enchs.json` | six entries, priceless (`price: null`) — none on the barrel; three are ench seeds (§M21.2) and three are Stall bargains (§M21.8) |
| the glossary | six shelves on **G**, every number read from its constant |
| lint | `every_new_ench_is_paid_by_a_bench` — each of the six is reachable from exactly one of the Plot or the Stall |

Acceptance: one behavioural test per kind, each asserting a number
(`bramble_returns_thirty_five_of_what_it_absorbed`, `the_one_page_fires_once
_at_five_and_is_spent`, …); `every_new_ench_is_paid_by_a_bench`.

### M21.7 — The Handler

*The Kennel's specialization.*

| deliverable | what |
|---|---|
| `SpecPower::Handler { offer_at, mouths, feed_pct, run_cells, tally_pct }` | knobs |
| the tree | six nodes as `SYSTEMS-PITCH.md` §2.2; *A Second Lead* is the only route to two out |
| the paper, the fork | three cards |

Acceptance: `spec_nodes_touch_only_their_bench` over three; `a_second_lead_
fields_two`; `fed_from_the_hand_eats_every_other_fight`.

### M21.8 — The counter, and eight buyers

*The Stall's bench and clock, in the third town.*

| deliverable | what |
|---|---|
| `stall.rs` | `Stall { shelf: Shelf, stock: Vec<(PieceId, price)>, ledger: Vec<Sale> }`; `WorldState::stall` |
| the shelf | a bizarre-shaped mask, `shelf: [...]` on the third town's map, nine to twelve cells; a component from the tray is **placed by its own shape** and leaves the tray |
| pricing | a price per item against its `rating`; the card prints the barrel's figure beside yours |
| buyers | eight, one per family, in `data/stall.json`; each `wants` two slots and a rarity floor |
| the clock | one buyer per win anywhere (`CUSTOMERS_PER_BELL = 1`), drawn from the eight; a buyer whose want is on the shelf at a **fair** price (within `FAIR_PCT` 20 of the barrel's) buys; at **high** buys one time in three (`HIGH_ODDS`); at **low** buys at once and the ledger prints what was left |
| bargains | `C(8,2)` = 28 buyer pairs — two buyers in a row who are kin pay a **bargain**: an `EVENT_ONLY` component or one of the three Stall enchs from §M21.6, onto the shelf, free |
| the screen | the third town's screen gains **the counter**: shelf, stock, price card, ledger. **The third town has shelves now, and they are the player's.** |

Acceptance: `a_piece_on_the_shelf_leaves_the_tray_and_keeps_its_shape`;
`a_fair_price_sells_on_the_next_bell`; `a_high_price_sells_one_in_three`;
`a_low_price_sells_at_once_and_the_ledger_says_so`; `every_pair_is_authored`
over buyers; `a_bargain_is_never_on_the_barrel`.

### M21.9 — The Factor, and the four papers

| deliverable | what |
|---|---|
| `SpecPower::Factor { customers_per_bell, margin_pct, shelf_cells, bargain_pct, stalls }` | knobs |
| the tree | six nodes as `SYSTEMS-PITCH.md` §3.2 |
| the fork | four specialization cards; the paper counter at four |
| `a_second_counter` | `stalls +1` opens a stall in Kettleworks, drawn with the third town's shelf mask rotated — a second shelf mask is a second thing to author, and the rotation is the plan's answer |

Acceptance: `spec_nodes_touch_only_their_bench` over four; `the_thumb_on_the_
scale_pays_a_tenth_over`; `a_second_counter_is_kettleworks`.

### M21.10 — The figures

*Every new sprite, as TikZ, from the prompt.*

| figure | count | notes |
|---|---|---|
| crop at harvest, per family | 8 | `art/crop-<family>.tex`; the family's palette; a plant that looks like what dropped the seed |
| sprout and shoot | 2 | generic stage 0 and stage 1, ink only |
| ench seed | 1 | |
| buyer, per family | 8 | `art/buyer-<family>.tex`; a person standing the way the family's creature stands |
| the three benches | 3 | bed, run, counter — drawn as the retort is |
| the three papers | 3 | Grower, Handler, Factor — the class-paper drawing in three more colourways, as M19.7 did the experts |
| **total** | **25** | |

Each `.tex` is `tikz_figure_prompt.md` filled in with: *facts the figure must
get right* = the family's `\Main`/`\Dark`/`\Accent`, `\Unit` 0.055cm, `\Edge`
2.4pt, primitives only, the bounding box the polyomino implies; *audience* =
a tile on a 380px canvas. `make art` compiles all twenty-five;
`check_the_palette_holds` (M19.15's) is extended to the eight crops and eight
buyers so nothing draws magenta.

Acceptance: `make art` clean; every figure referenced by `data/art.json`;
`every_figure_is_a_standalone_tikz_from_the_prompt` (a lint that reads the
header comment `Written by filling in tikz_figure_prompt.md`).

### M21.11 — The gate, the walk, the glossary, the deploy

| deliverable | what |
|---|---|
| browser checks | `check_a_seed_is_in_the_drawer_and_not_the_tray`; `check_the_bed_grows_on_a_win`; `check_a_crop_that_will_not_fit_names_the_cells`; `check_the_sixth_win_offers`; `check_a_creature_out_is_on_the_bar`; `check_the_shelf_takes_a_piece_by_shape`; `check_a_sale_prints_on_the_ledger`; `check_four_papers_one_slot`; `check_the_new_enchs_have_shelves_on_g`; each negative-tested |
| the walk | `make play` from the run fixture plants, kennels and stocks; the transcript's harvest count, offer, and first sale written down |
| the glossary | every constant named in this plan on **G** |
| `HANDOFF-M21.md`, `CLAUDE.md` | the three benches in `CLAUDE.md`'s systems section; twenty-nine maps unchanged; `data::MAPS.len()` untouched |
| **deploy** | `make publish`, report the hash |

### M21.12 — The offer reaches the player, and the receipt can be read

*Asked for mid-block, in the human's words: "make all the text in the after
battle screen larger and easier to read, and hook the kennel into the game
because there is currently no way for an enemy to offered for recruitment into
the kennel".*

**The second half is a bug and it is mine.** `Game::kennel_offer` and
`Game::take_along` shipped in M21.4 with seven tests and **no caller outside
them** — so the run, the yard and the feed are all live at `77bb4b19` with no
way to put anything in the kennel. That is the Apothecary's own failure, in a
block that opened by fixing it: *a lint that reads a list rather than the
behaviour is the failure it exists to catch*, and nothing was asking whether
the offer was reachable.

| deliverable | what |
|---|---|
| the offer | the result screen of a won fight offers *Take it along* when `kennel_offer` says yes, and says why not when it is close — the refusal already counts down |
| `offer_json`, `take_along_here` | two exports; the decision stays in core, the shim moves strings |
| the reachability lint | `every_way_into_the_kennel_is_reachable` — a core rule with no caller is content reachable from nowhere, and this is the second time in two blocks |
| the receipt | the after-battle screen at a size somebody can read: the result title, the receipt lines and the tape, up from 11–13px |

Acceptance: `check_a_won_fight_offers_the_creature`, negative-tested;
`check_the_receipt_is_legible` measures the computed font size rather than
reading the stylesheet.

### M21.13 — The larder is earned, not found

*Asked for: "ingredients should be much rarer so that you have to use the
growing system".*

Today a win pays an ingredient **certainly**, so the Plot is a second source
for something you already have enough of — and the Kennel eats out of the same
larder, so nothing is ever short. A roll makes fighting the trickle and growing
the supply, which is what the Plot was built to be.

| deliverable | what |
|---|---|
| `INGREDIENT_PER_MILLE` | the drop becomes a roll, off `game.rng` like every other, with a glossary shelf |
| the measurement | what a win pays against what a bed pays, written into the commit — the Plot's yield is `HARVEST_YIELD` per crop per `STAGES - 1` wins and that is the number the rate is set against |
| the Kennel | a creature out eats out of the same larder, so this is also what makes the feed cost something — notebook row 13, answered |

Acceptance: `a_win_pays_an_ingredient_at_its_rate`; `growing_out_pays_more_than
_fighting_does`, which is the ask stated as a number.

### M21.14 — The town is a street

*Asked for: "the town screen should be reworked to be more like a bunch of tabs
you can select from, and when you press the tab button, the functionality for
that system appears. so each tab can be presented as another building in the
town, somewhat like the way gear master 1 presents its towns".*

The town screen is now the shelf, the barrel, the order book, the tins, the
bank, the errands, the brewing bench, the bed and the run, stacked down one
column. That is nine systems in one scroll.

| deliverable | what |
|---|---|
| the street | one tab a building, drawn as a row of buildings rather than a tab strip — `.treetabs`' own shape, which the errand log already reuses |
| what is behind each | the panels that exist, moved rather than rewritten; nothing changes about what any of them does |
| which buildings | a town has the ones it *has*: no bed, no bed tab. The third town has no shelf and says so |
| the figure | one TikZ figure a building, counted into M21.10 |

Acceptance: `check_the_town_is_a_street` — every system reachable in one press,
and a town missing one does not draw its building.

### M21.15 → — Whatever the notebook says

**These milestones do not exist yet.** When M21.11 is done, the builder reads
`SECOND-ORDER-M21.md`, takes every row marked `open`, groups them into
milestones of the same shape as the ones above — a title, a deliverables
table, an acceptance line — numbers them M21.12, M21.13, … in the order that
closes the most rows soonest, appends them to this file under this heading,
executes them, closes the rows, and writes the status table after each. A
row that is a finding rather than work (*the yardstick moved* is a finding;
*re-measure the four floors* is work) is closed with the finding written in
`CLAUDE.md`. The block ends when the notebook has no open rows, and the last
status table says so.

---

## Status table

After **every** milestone's commit, this table is appended to `MILESTONES.md`
under `## M21 — three benches`, one row per milestone done or pending, and
the same table is printed at the end of the milestone's final message:

```
| # | milestone | deliverables | tests | commit | state |
|---|---|---|---|---|---|
| M21.0 | The seed drawer | plot.rs · plot.json · seed_drawer · the drop · 2 lints | 1004 (+13) | a1b2c3d | done |
| M21.1 | The bed | … | … | … | in progress |
| M21.2 | Companion planting | … | | | pending |
| … | | | | | |
| notebook | rows open / closed | 3 / 2 | | | |
```

`tests` is the count `make test` reports and the delta from the previous row.
`state` is one of `done`, `in progress`, `pending`, `added` (a notebook
milestone), `dropped` (with the notebook row that says why). The notebook line
is always last.

---

## Numbers

| thing | number |
|---|---|
| systems | 3; specializations 4 (with the Apothecary) |
| bags | 2 new (seed drawer, kennel); the shelf is the stall's own |
| benches | bed ×3 towns, run ×3, counter ×1 (+1 with the Factor) |
| `C(8,2)` tables | 3, one lint |
| clocks | 3, every constant a shelf on G |
| new enchs | 6; new effect kinds 6; games cited 6 |
| new sprites | 25, all TikZ |
| new components | **0** |
| new combat code | the second combatant, through `simulate_party` |
| deploy points | 2; approvals asked **0** |
| milestones written here | 12; milestones the notebook adds | as many as it has open rows for |

## Decisions that would have been the human's, and the recommendation taken

| decision | recommendation, taken without asking |
|---|---|
| `STAGES` — fights per harvest | 3, so a harvest is three wins; the Grower's first real node makes it two. The pitch said seven and seven is a session for a new player. |
| `KENNEL_CAP` | the region's bracket, never the creature's own. A boss on your side is the balance risk and this is the cap. |
| bosses in the kennel | never. `rank == boss` is not offered. |
| the Stall's second counter | Kettleworks, with the third town's shelf mask rotated a quarter turn. |
| `FAIR_PCT` | 20 either side of the barrel's figure. |
| which three enchs are seeds and which three bargains | seeds: the bramble coat, the slow match, the long count (things that grow); bargains: the tithe ring, the one page, the first word (things that are traded). |
| whether a kennelled creature eats when it is *in* | no. Only out, only at the bell. |
| whether to ship the Grower before the Kennel exists | yes — the Plot is complete on its own and its ench seeds are its reason; the Kennel is the next thing it feeds. |

---

## M21.16 — An errand for every bench

**Asked for**, in the human's own words:

> I also want to add a quest chain for each new system, that the quests are
> predicated upon using the system. an example would be brew a potion that
> gives at least 300 max health and give it to the quest giver, have two
> specific enemies as companions in your kennel, etc. add this as a new
> milestone at the end

Four benches shipped in M20 and M21 — the retort, the bed, the run, the counter
— and **not one errand in the game is about any of them.** That is the M21.4
failure one level up: a system a player can reach and no screen that *points*
at it. The quest log is the one screen in this game that says *something has
opened and it is somewhere else*, and it has never once said it about a bench.

### The goal kinds

`Goal` has four arms — `Slay`, `Bring`, `Word`, `Clear` — and **none of them
can ask about a bench**, for the reason `Clear` had to be written in the first
place: a `Bring` is answered by a component in the bag and a potion is not a
component, a `Slay` is answered by a creature dying and a companion is a
creature that did not, and a `Word` is answered by standing somewhere.

So the block adds **one** new arm and not four, because four arms asking four
nearly-identical questions is how they drift:

```rust
Goal::Show { what: Shown }
```

where `Shown` is what a bench can be asked to produce:

| `Shown` | asked of | answered by |
|---|---|---|
| `Brew { stat, at_least }` | the pack | a potion in `Character::potions` whose `Gives` reaches the figure |
| `Grown { seed, n }` | the larder | `n` of what that seed crops into, held at once |
| `Kennelled { creatures }` | the kennel | every named creature in `Character::kennel` |
| `Together { creatures }` | the run | every named creature **out** at once |
| `Sold { at_least }` | the ledger | one sale at or over the figure |

**Read, never banked.** Every one of the five is a question asked of what the
character is holding *now*, the way `Clear` reads `answered` and `holding`
reads the bag — so an errand taken after the fact is `Ready` the moment it is
taken, which is right: you did the thing.

**And the figure is derived.** `Brew { stat: "max_health", at_least: 300 }` is
answered by running `brew::gives` over the potion, which is the same function
the pack's card prints from — a threshold checked against a second sum would be
a second rulebook.

### The chains

Four chains, one a bench, **three rungs each**, given at the counter the bench
is at and turned in there:

| chain | root | then | then |
|---|---|---|---|
| **the retort** | brew anything at all | brew one that gives 300 max health | brew one out of two three-cell ingredients |
| **the bed** | grow anything | hold four of one crop at once | harvest a companion pair |
| **the run** | kennel anything | kennel two named creatures | have both out together |
| **the counter** | sell anything | sell something for 400 | sell at a high ask |

Every rung is a thing the bench already does; **nothing new is built in any
bench for this.** That is the constraint — an errand that needed a new
mechanic would be a milestone about the mechanic.

### What they pay

Gold and components, and **no rows, no enchs and no new components** — M21.8's
own rule and the catalogue's. The last rung of each chain pays the one thing
that bench wants: **a seed** for the bed, **an ingredient** for the retort, a
**bargain component** for the counter, and for the run the only thing a kennel
can use, which is a larder full.

### Deliverables

| | |
|---|---|
| `Goal::Show { what: Shown }`, five arms, in `quest.rs` | read at `stage()` and nowhere else |
| `quest::guide` answers for each — a bench is a *place* | so the log points at the town, which is what the log is for |
| twelve errands in `data/quests.json`, four chains of three | `granted`, so a rung you have not reached is not on a counter |
| `every_bench_has_a_chain` | a lint over the four benches, not a list of four |
| `every_rung_is_a_thing_the_bench_already_does` | no rung needs a mechanic that does not exist |
| the ask line for each, unthemed, TONE 13a | *brew one that gives 300 max health* is the engine's sentence |
| a browser check: a rung goes `Offered` → `Ready` when you use the bench | the one thing `cargo test` cannot ask |

**Acceptance.** Twelve errands, four chains, one new `Goal` arm with five
variants. `make play` shows at least the root rung of one chain handed in. Every
lint green, and breaking each new one names the bench it is about.
