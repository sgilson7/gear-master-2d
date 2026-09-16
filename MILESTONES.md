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
| **M19.0** | The ball slides | clock-driven and interpolated between ticks; the trail grows behind it | `27d0542` · green |
| **M19.1** | A diamond catches the ball | `PlaceKind::catches`, `Contact::Caught`; hitting a gate is entering it | `27d0542` · green |
| **M19.2** | Five obstacles, five marks | a boulder, a hole, teeth, a drift and a lane — nine places had worn the event mark | `27d0542` · green |
| **M19.3** | The long cart | a ride between towns you have stood in, 40 Fnorp, from the town screen | `27d0542` · green |
| **M19.4** | The furnace reaches a board that swings | the Stoker dealt 746 against a classless 746; its stacks pay both lanes now | `27d0542` · green |
| **M19.5** | The furnace on the bar | `Event::Burned` was in the replay's `_` arm and nothing drew it | `27d0542` · green |
| **M19.6** | The glossary | five shelves on **G**, every number read from the constant that decides it | `27d0542` · green |
| **M19.7** | Twenty-one expert papers | one drawing, twenty-one colourways — an expert is a pair, so the figure is | `27d0542` · green |
| **M19.8** | Two browser checks | the furnace on the bar, and the glossary. 90 green | `f2196a0` · green |
| **M19.9** | The cue draws the road ahead | the whole predicted path, the tile it stops on, and a box round what it hits — because an event is one tile and nothing helped you aim | ✓ |
| **M19.10** | A shot prints a refusal | `shoot` threw `blocked`/`refused_by` away; `walk` has printed them since M15. The third *"the land is pink"* | ✓ |
| **M19.11** | Instant battle on a table | `try_shoot` ran neither `rout` nor `instant`; one `settle_without_a_screen` called from both | ✓ |
| **M19.12** | A sealed door that says what it wants | `needs_all` rather than `hidden_until` on the way under the flat — and `sealed_because` had been called by nothing since M14 | ✓ |
| **M19.13** | Only the boss ends the sitting | `leave_the_sitting` fired on every win on a Stack floor; three tests covered the function and none the trigger | ✓ |
| **M19.14** | The Wextreen deep | ten enemies on her board, two TikZ families, `The Unwritten` on the Undercountry | `cc5c0bc` |
| **M19.15** | Nothing draws magenta | `quick`, `silt` and `tide` had no palette entry; a check now holds the palette against the engine's list | `cc5c0bc` |
| **M19.16** | A surveying door that asks | the frame opens every time, names its own door, and signs the figure | `cc5c0bc` |

---

# M21 — three benches

*`PLAN-M21.md`: the Plot, the Kennel, the Stall — three systems in brewing's
shape, each with a specialization, each with a `C(8,2)` table a lint proves
complete. `SECOND-ORDER-M21.md` is the notebook, and M21.12 onward are read
off it.*

| # | milestone | deliverables | tests | commit | state |
|---|---|---|---|---|---|
| M21.0 | The seed drawer | `plot.rs` · `plot.json` · `seed_drawer` · the drop · 2 lints · 5 acceptance | 1,044 (+7) | `b53964e` | done |
| M21.1 | The bed | bed masks ×3 · `Game::plant` · `plot::tick` · harvest · the screen · 6 acceptance | 1,044 | `2ab7c62` | done |
| M21.6 | Six enchs from six games | 5 `Effect` kinds · **4 were mechanics the engine already had** · 3 entries (3 held for the Stall) | 1,053 (+9) | `6226982` | done · **moved before M21.2** |
| M21.2 | Companion planting | 28 pairs · adjacency at harvest · 3 ench seeds · load lint · `every_pair_is_authored` | 1,053 | `6226982` | done |
| M21.3 | The Grower | `SpecPower::Grower` · six nodes · the paper · the fork at two | | | pending |
| M21.4 | The kennel, and the offer | `kennel.rs` · the offer at six wins · the feed · the yard · the screen | | | pending |
| M21.5 | The second combatant | `simulate_party` · targeting · the tally · 28 pairs · the replay | | | pending · **deploy** |
| M21.6 | Six enchs from six games | six `Effect` kinds · `combat.rs` · six shelves on G | | | pending |
| M21.7 | The Handler | `SpecPower::Handler` · six nodes · three cards | | | pending |
| M21.8 | The counter, and eight buyers | `stall.rs` · the shelf · pricing · eight buyers · 28 bargains | | | pending |
| M21.9 | The Factor, and the four papers | `SpecPower::Factor` · six nodes · four cards · a second counter | | | pending |
| M21.10 | The figures | 25 TikZ figures · `make art` · the palette lint | | | pending |
| M21.11 | The gate, the walk, the deploy | 9 browser checks · `make play` · the glossary · `HANDOFF-M21.md` | | | pending · **deploy** |
| notebook | rows open / closed | 2 / 7 | | | |

### after M21.8 — the counter

| milestone | what it is | tests | Δ | commit |
|---|---|---|---|---|
| M21.0 | the seed drawer, and eight seeds | 1,013 | +9 | `9bcbfb1` |
| M21.1 | the bed, and a crop that turns | 1,022 | +9 | `2ab7c62` |
| M21.6 | six enchs, four of which the engine already had | 1,029 | +7 | `6226982` |
| M21.2 | twenty-eight companion pairs | 1,029 | — | `6226982` |
| M21.3 | the Grower, and a percentage that rounded to nothing | 1,034 | +5 | `f535c38` |
| M21.4 | the kennel, the offer, the feed and the yard | 1,046 | +12 | `1e502ec` |
| M21.5 | a creature out fights, as gear, capped at the region | 1,053 | +7 | `572796d` |
| M21.7 | the Handler | 1,053 | — | `1d13985` |
| M21.12 | the offer is reachable, and the result screen is legible | 1,053 | — | `1d13985` |
| M21.13 | an ingredient is a roll at 250‰, not a certainty | 1,055 | +2 | `1d13985` |
| **M21.8** | **the counter, eight buyers and twenty-eight bargains** | **1,069** | **+14** | **`1d13985`** |
| M21.9 | the Factor | pending | | |
| M21.10 | twenty-five figures | pending | | |
| M21.11 | gate, walk, glossary, deploy | pending | | |
| M21.14 | the town is a street | pending | | |
| M21.16 | an errand for every bench | pending | | |

Browser gate: **104 `ok:` lines**, chromium. Suite green in **99 binaries**.
Notebook: **29 rows, 22 closed, 7 open.**

### after M21.16 — an errand for every bench

| milestone | what it is | commit |
|---|---|---|
| M21.0–M21.5 | drawer, bed, enchs, pairs, Grower, kennel, companion | `572796d` |
| M21.7 · .12 · .13 | Handler · the offer and the result screen · a rarer ingredient | `1d13985` |
| M21.8 | the counter, eight buyers, twenty-eight bargains | `1d13985` |
| M21.14 | the town is a street of seven buildings | `cfc9f14` |
| — | **reported from play**: a specialization tree that refused every node, and a class lane no screen drew | `093e851` |
| **M21.16** | **an errand for every bench — `Goal::Show`, twelve errands, four chains** | **`f3c75f0`** |
| — | the specialization bug, as a gate check in its own words | `84fcfda` |
| M21.9 | the Factor | pending |
| M21.10 | twenty-five figures | pending |
| M21.11 | gate, walk, glossary, deploy | pending |

**1,097 tests in 94 core binaries**, counted with `packaging/count-tests.sh` —
which is the only way to get the number back, because `cargo test` interleaves
its summary lines and cannot be summed. **The per-milestone deltas above this
block were estimates and are gone**: a number nobody can reproduce is not a
number, and estimating one in a table headed *tests* is that rule broken in the
file that records it.

Browser gate: **107 `ok:` lines**, chromium. Suite green in **100 binaries**
across the workspace, `cargo test --workspace` exit 0.
Notebook: **43 rows, 38 closed, 5 open.**

### after M21.11 — the figures, the glossary, the record

| milestone | what it is | commit |
|---|---|---|
| M21.0–M21.8 | drawer, bed, enchs, pairs, Grower, kennel, companion, Handler, counter | `1d13985` |
| M21.14 | the town is a street of seven buildings and four doors | `cfc9f14` |
| M21.16 | an errand for every bench — `Goal::Show`, twelve errands, four chains | `f3c75f0` |
| M21.9 | the Factor, and four things reported from play | `ed86846` |
| — | promises rewritten mechanically, and the pools called one thing | `2479a80` |
| **M21.10** | **thirteen figures from two drawings** | **`7c18b73`** |
| **M21.11** | **the glossary's sixth shelf, the divergences, the record** | this |

`packaging/count-tests.sh` is the only way to get a test total back. Browser
gate: **108 `ok:` lines**, chromium. Suite green, `cargo test -p gm2d-core`
exit 0.

### M21 — done

| milestone | what it is | commit |
|---|---|---|
| M21.0–M21.8 | the drawer, the bed, the enchs, the pairs, the Grower, the kennel, the companion, the Handler, the counter | `1d13985` |
| M21.14 | the town is a street of seven buildings and four doors | `cfc9f14` |
| M21.16 | an errand for every bench — `Goal::Show`, twelve errands, four chains | `f3c75f0` |
| M21.9 | the Factor, and four things reported from play | `ed86846` |
| M21.10 | thirteen figures from two drawings | `7c18b73` |
| M21.11 | the glossary's sixth shelf, the divergences, the record | `0043982` |
| **M21.17** | **the notebook emptied: the walk, the bargain's owner, three findings written down** | this |

Suite green, `cargo test -p gm2d-core` exit 0. Browser gate: **108 `ok:` lines**,
chromium. `packaging/count-tests.sh` is the only way to get a test total back.

Notebook: **63 rows, 0 open.**

## M22 — High Wick, and the table

| # | milestone | deliverables | tests | commit | state |
|---|---|---|---|---|---|
| M22.0 | The measure | 6 numbers; 4 of them disagree with the plan and it is corrected | 1107 (+0) | this | done |
| M22.1 | A wing is a shelf with a host | wing_of · arrives · name · met/DONE · counters_at · shelves_among · 10 checks | 1117 (+10) | `9d2434a` | done |
| M22.2 | The clerk comes down | send-for-the-clerk (a Slay, not a Bring) · the-clerks-desk · the mirror errand re-keyed · 3 avail lints widened | 1120 (+3) | `c2060ba` | done |
| M22.3 | The arcane shelf, and the post gets its name | high-wick as a wing · cut-the-post · PlaceDef::named · the gate south | | | pending |
| M22.4 | The gate, the walk, the deploy — the town | check_the_third_town_fills_up · in-the-third-town.json · deploy | | | pending |
| M22.5 | A pocket can go somewhere | Pocket honours to/at_to · the tape · the table's id in the lint | | | pending |
| M22.6 | The table | the-lower-table · the cup as a map of its own · 4 lints | | | pending |
| M22.7 | What stands on it | The Twelfth Name · one art colourway · the pool | | | pending |
| M22.8 | Three errands you do with a cue | a chain of three off the desk | | | pending |
| M22.9 | The gate, the walk, the glossary, the deploy — the table | 5 browser checks · on-the-lower-table.json · the stale rows · deploy | | | pending |
| notebook | rows open / closed | 6 open / 14 closed | | | |
