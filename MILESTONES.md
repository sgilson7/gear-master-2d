# The combined block — M16, M17, M18

*Two plans and a notebook. `PLAN-M16.md` is the Eleven Reefs and two classes;
`PLAN-M17.md` is the country as a table; M18 is the two notebooks executed.*

| # | Milestone | Deliverables | Status |
|---|---|---|---|
| **M16.0** | The fixture and the primitives | the run as a save; `common::from_save`; `Character::item_partition`; `quick` terrain; `MonsterSpec.enchs`; `stats::LANE_CAP` and a behavioural curse lint | **done** `288810c` · 919 passing |
| **M16.1** | The gate and the Flat Below | `the-way-under-the-flat` on the Sands; `the-reefs-1`; nine stakes, three bands, six pockets, the sheet; floor 2 stubbed | **done** `deed458` · 927 passing |
| **M16.2** | The Assay | `the-reefs-2`; three doors; the assay office; the repack measured off the run | **done** `9301dcb` · 933 passing |
| **M16.3** | The Needle Room and the Tenth Surveyor | `the-reefs-3`; four sinkholes, four levers, the plate; the boss wearing the run; the bracket | **done** `6cef332` · 941 passing |
| **M16.4** | Two more base classes | `ClassPower::Stoker`, `::Whisperer`; `OFFERED` at seven; two nine-node trees; the fork at seven cards | **done** `651a932` · 950 passing |
| **M16.5** | Eleven experts | `EXPERTS` at twenty-one; eleven `ExpertPower` variants; eleven six-node trees; three new rules | **done** `651a932` · with M16.4 |
| **M16.6** | The M16 gate and walk | six browser checks; the walk; `HANDOFF-M16.md`; `CLAUDE.md` | **done** `56134d8` · 950 passing |
| **M17.0** | The table, in core | `shot.rs` — `Shot`, `Flight`, `Contact`, `shoot`; `traversal` on `TilesData`; integer physics | **done** `d5c86cc` · 960 passing |
| **M17.1** | Obstacles | five `PlaceKind`s; parse lints; the Treyway as a table | `084b955` · 976 |
| **M17.2** | Landing | `Game::shoot`; `Step`'s resolution at rest; the landing roll; the tape | `084b955` · 976 |
| **M17.3** | The cue | the drag, the keyboard, the arrow, the flight, the strip, reduced motion | `084b955` · 976 · **deploy** |
| **M17.4** | The walker aims | `shot::aim_at`; the reachability lint; `playthrough.py` shooting | `084b955` · 976 — the Treyway crosses in **3** shots against the plan's 9 |
| **M17.5** | The Undercountry, and the handoff | the second table; `HANDOFF-M17.md`; `CLAUDE.md` | `084b955` · 976 · **deploy** (handoff to follow) |
| **M18.0** | The scratch binary earns a name | `probe.rs` → `fan.rs`: how many distinct tiles a shot reaches, per power, per map (M16 row 7) | ✓ |
| **M18.1** | A pocket is a boss tile | `a_pocket_is_a_boss_tile_and_cannot_be_farmed_for_a_mark` (M16 row 12) | ✓ |
| **M18.2** | A fall is not a free ride | `a_warp_that_stays_on_its_own_map_is_a_fall_and_not_a_ride` — the map-aware lint (M16 row 22) | ✓ |
| **M18.3** | The dead axis says so | `Axis` and `ClassDef::requires` are read by nothing; documented rather than grown (M16 row 34) | ✓ |
| **M18.4** | The third kind of arrival | four doors onto a tile, named; `a_warp_resolves_no_place` (M16 row 28) | ✓ |
| **M18.5** | The walker reads before it bars | a road refused three times might be a road a card opens (M16 row 42) | ✓ |
| **M18.6** | The relink cost, measured | **127s relinking + 47s running ≈ three minutes**, not ten. `include_str!` is not where it goes — the fix, if ever wanted, is fewer test binaries (M16 row 17) | ✓ |
| **M18.7** | The two the builder may not decide | rarity's thresholds (row 14) and what a rating describes (row 24), written up as the human's | ✓ |
| **M18.8** | The gate's webkit find | a drag clamped off-viewport; `pull_to` measures both ends. 85 checks green in all three engines | `94efff9` · 976 |
| — | **Live** | `fac56a16`, deployed from `34f2dcd`; `GM2D_ORIGIN` walked all 85 against the deployed page | ✓ |

## M19 — five things reported from play

| # | Milestone | Deliverables | Ships |
|---|---|---|---|
| **M19.0** | The ball slides | clock-driven and interpolated between ticks; the trail grows behind it | `27d0542` · 1137 |
| **M19.1** | A diamond catches the ball | `PlaceKind::catches`, `Contact::Caught`; hitting a gate is entering it | `27d0542` · 1137 |
| **M19.2** | Five obstacles, five marks | a boulder, a hole, teeth, a drift and a lane — nine places had worn the event mark | `27d0542` · 1137 |
| **M19.3** | The long cart | a ride between towns you have stood in, 40 Fnorp, from the town screen | `27d0542` · 1137 |
| **M19.4** | The furnace reaches a board that swings | the Stoker dealt 746 against a classless 746; its stacks pay both lanes now | `27d0542` · 1137 |
| **M19.5** | The furnace on the bar | `Event::Burned` was in the replay's `_` arm and nothing drew it | `27d0542` · 1137 |
| **M19.6** | The glossary | five shelves on **G**, every number read from the constant that decides it | `27d0542` · 1137 |
| **M19.7** | Twenty-one expert papers | one drawing, twenty-one colourways — an expert is a pair, so the figure is | `27d0542` · 1137 |
| **M19.8** | Two browser checks | the furnace on the bar, and the glossary. 90 green | `f2196a0` · 1137 |
