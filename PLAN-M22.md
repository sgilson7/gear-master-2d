# PLAN-M22 — High Wick comes down, and the table under the writing

**The frame.** `PLAN.md` wins where it and this disagree, and a divergence from
this goes in `CLAUDE.md`'s table with its reason, in the commit that makes it.
Written against **`4a3ae9a`**, "M21.17: the notebook emptied": M21 is closed —
the Plot, the Kennel and the Stall, five specializations, the town a street of
seven buildings, twelve bench errands, thirteen figures, a sixth glossary
shelf, twenty-nine maps, 1,097 tests at the last count, **108 `ok:` lines**,
and a notebook of **sixty-three rows with none open**.

## What was asked for

> I want to add some more content to the game, starting from the town in the
> undercountry that currently has nothing. Please brainstorm 3 different ways
> we can add additional new content to the end of the game starting from the
> undercountry; please cite games that you draw any ideas from

and then, of the three:

> write up the settling of high wick and the table ideas into a plan like the
> plans already existing in the repo. I do like the dungeon idea but i think
> its too early to implement that now

Two of the three, in one block, in the order they compose: **the town is
settled first**, because the last thing settling it does is open the way south,
and **the table is what is south.** The dungeon is declined by the human and is
not here; §*What this block deliberately leaves* costs it out so the next plan
does not measure it again.

The games: settling is Suikoden II's castle (recruit somebody, a room opens),
Dark Cloud's Georama (rebuild a town out of what you bring up from the dungeon
under it — which is this map's own geography) and Stardew Valley's Community
Center (a shelf of asks, each answered by using a system you already have). The
table is Yoku's Island Express (pinball as the whole traversal of a country,
holes as the way *into* rooms, deliveries by flipper), Pokémon Pinball (landing
on a creature is how you get one) and Sonic Spinball (a boss reached by a
specific shot and not by walking up to it).

---

## What moved under this plan before it was executed

This plan was first written against `0043982`. One commit landed on top of it
and changed four things in it. Recorded here rather than silently patched,
because *a plan that was wrong and was corrected is worth more than one that
was never checked.*

| | what `4a3ae9a` changed | what it changes here |
|---|---|---|
| 1 | **The notebook has no open rows.** All sixty-three closed; row 41 — `make play`'s timeout at the Reach — is fixed, rows 7, 9, 12 and 23 are written into `CLAUDE.md` or answered | §M22.10 no longer carries five rows in. `SECOND-ORDER-M22.md` starts empty, at row 1 |
| 2 | **The walker gets past the Reach, and still never reaches the Undercountry.** It runs **977 fights to level thirteen** from a new game and ends in the loop `PLAN.md` §6d row 3 already names — *a walker with a destination stops being a player* | Every "the walk" deliverable becomes `GM2D_FROM=…`, and **the block ships the start-line saves**, which it did not before |
| 3 | **Instant Battle draws no replay**, and `fight()` waited ten seconds for one | Any browser check that fights on the table must not wait for `#stage-replay`. A level-45 player has marked most of the ladder |
| 4 | **`a_bargain_is_never_on_the_barrel` was re-shaped**: it mutates the file, hands it to `parse`, and makes the refusal the assertion, because *a check whose negative test cannot be made to fail through the check is a check nobody has proved* | §M22.1's parse-and-lint split is rewritten to that shape before it is built, rather than after somebody finds it. Decision 5 |

---

## What M22.0 measured, and the four things it changed

**Every number below is a command against `4a3ae9a`, run before a line was
written.** Four of them disagree with the section under this one, and the
disagreements are the milestone's whole deliverable: *a plan that was wrong and
was corrected is worth more than one that was never checked.*

| | the plan said | the measurement | what it changes |
|---|---|---|---|
| the cup | a sealed cup of rock, and the far pocket is the only way in | **6,546 of the shots taken from the 278 tiles outside the draft cup come to rest inside it.** A one-tile wall is not a wall to a ball: `shoot_with` tests the tile the tick *landed on*, and one tick is 18.75 tiles at power ten. A two-tile wall is transparent too; nineteen would be needed | **Decision 9 is rewritten.** The cup is a **map of its own** reached by the pocket, which is what decision 8 already builds — `warp_to(to, at_to)`. Maps go to **31**, not 30. Notebook rows 1–4 |
| the yardstick | the boss is dressed against `common::from_save(common::THE_RUN)` to a win in 20–28 s | **the run beats nothing at this depth.** 974 health and 9 strength over 11 items at level 45; it loses to The Unwritten in 7.7 s and to all ten of the Wextreen deep, and the only two things down there it beats take 32–33 s | **Decision 12 is rewritten.** Dressed against `common::geared_from`, which `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` has required of every boss on a tile since M11.7 and which the Tenth Surveyor was actually bracketed against. The run is kept as the **floor** and asserted to lose. Notebook row 6 |
| the fight's length | a win between 20 and 28 seconds, never at `SUDDEN_DEATH_MS` | **every fight `geared_from` wins down there is decided inside the ramp**: The Unwritten 38.0 s, the Ninth Surveyor 44.0 s, the Tenth 43.0 s. The only creature it beats in the 20–28 window is The Tailgate at 24.3 s, rated 1336 — two bands shallower | the bracket is **a victory that is not the buzzer's**, and longer than The Unwritten's 38 s. Notebook row 7 |
| the casting count | 89 → about 92 with `high-wick` placed | **89 → 89.** Everything the arcane shelf stocks was already reachable through `barrel_pool` or `ledger_pool`; the three spells leave the cheap tier and arrive on the shelf in the same move | the barrel recount in §M22.3 is a **zero**, written down rather than assumed. What placing the shelf buys is a counter a player can find, not availability. Notebook row 5 |

And three that hold:

| | |
|---|---|
| **the flood** | the draft table's seventeen non-cup places are all landable, **and it takes two rounds** — the plan's guess, and the Treyway's number. After one round only the middle event is missing |
| **the errand at 45** | `the-long-mirror-inventory` is **about eight fights**: the Mirror Fiend is 36.4% of the Bengulon Verge's draws (weight 220 of 604, behind the Warded Idol's 243), and three drops is three wins against it. At 4% a fight that is **32 points of fatigue** against a `CAP` of 60, plus 40 Fnorp each way on the long cart. A stroll, which is what it is for |
| **the check to flip** | `check_the_third_town_is_empty_on_purpose` plants at `[10, 5]`, shoots to `[10, 10]`, asserts `#shelf button` is **0** and the text says *nothing for sale*, then shoots to `[10, 11]` and reads `#ending-prose` for *decided*. Every one of those five assertions is a line M22.3 and M22.4 turn over |

**And one number the plan carried in from `MILESTONES.md` is stale**: the suite
is **1,107** in 94 binaries, not 1,097. Notebook row 8.

## Recon, done before this was written

Every number below came from a command against `4a3ae9a`; the plan is written
around them and marks what it guessed instead.

| what | number | how |
|---|---|---|
| the player's level | **45** | the repository-root `the-run-20260910.json` carries `xp: 45797`; folded through `XP_TO_NEXT` that is level 45, Berserker / Showstopper / ShortProgramme. `PLAN-M14.md` bracketed the Undercountry **23+**. `MAX_LEVEL` is 60 and the curve's exponential arm past 50 currently pays for nothing |
| **nothing in this repository has opened the Undercountry** | **0 of 14** | every file in `testing/saves/` read: not one has both `the-ninth-surveyor` and `the-bottom-of-the-bottom` in `answered`, which is what `hidden_until_all` on Marbulon's door wants. **And the human's own save has the second and not the first.** So the content this block adds stands behind a door no fixture, no walker and no shipped save currently opens — which is the argument for decision 10 |
| what the third town already has | bed (14 cells), run (14 cells), retort, bank, counter, barrel, order book, tins | `the-undercountry.tiles.json` carries `bed` and `run`; the rest is every town's. **What it lacks is a shelf and errands and nothing else** — two buildings of seven, and a name |
| the third town's bed | **15 of 28 pairs can be planted apart**, against Kettleworks' 10 and the pit's 17 | `CLAUDE.md` *Adjacency on a bed is usually forced*, new in `4a3ae9a`. The best-tempered bed in the game is in the town nobody can trade in, which is a reason to settle it and is worth one sentence of the clerk's |
| High Wick | **17 shelf lines, 2 commissions, 1 errand, 1 Stall buyer, on no map** | `shops.json` `high-wick`; `quests.json` `the-long-mirror-inventory` (giver `high-wick`, slay 3 Mirror Fiend); `stall.json` `the-woman-from-high-wick` (floor 50); `avail.rs` `STAGED = ["high-wick"]` |
| what the arcane shelf costs the barrel | 89 of 106 casting components are buyable today | `nothing_in_the_barrel_is_on_a_shelf_you_can_reach` asks `data::towns_on_the_map`, so the day the shelf has ground under it three of the five barrel-priced spells leave the cheap tier. **Guess: 89 → about 92.** The builder counts |
| Mirror Fiend | health **250**, in West Bambulon's pool | `combat.rs:1387`; `west-bambulon.tiles.json`. High Wick's one errand asks a level-45 player for three of a 250-health creature on the first map |
| the level-45 yardstick **already exists** | `common::from_save(common::THE_RUN)` | `tests/common/mod.rs:406`, `THE_RUN = "testing/saves/the-run-20260910.json"` — *"this block is bracketed against a real save at level forty-five, not against a walker's board"*. **It is a board with an empty world** (`map: ""`, nothing answered), so it is a yardstick and never a start line |
| a sale is keyed by the shelf's id | `bought: Vec<(String, u16)>` — `(town, index)` | `world.rs:1602`; `shop::shelf` marks sold entries rather than dropping them, because *the index is the identity*. **So a wing keeps its own shelf id and its own index**, and `high-wick` stays `high-wick` |
| what `place_is_there` reads | `answered` and `flags` — **not `quests_done`** | `world.rs:548`, the `met` closure; `quest::hand_in` writes `quests_done` and nothing else. **An errand cannot open a place today**; that is the one predicate this block adds to |
| the gate already stands in the third town | twice, planted at **[10, 3]** and **[10, 5]**, shooting to [10, 10] | `drive.py:1460` and `drive.py:5511`. `check_the_third_town_is_empty_on_purpose` asserts `#shelf` has **0** buttons and says *nothing for sale*, then shoots to [10, 11] and reads `#ending` for the word *decided*. **That check is the one this block has to flip** |
| the tee convention | **four tiles up the central lane** | both plants, in their own comments: *a tile one step away is a tile you cannot shoot to*, so a check that plants itself adjacent has nowhere to play from |
| a pocket already moves you off the map | to `last_town`, at **12** fatigue | `game.rs:582`, `POCKET_TIRES`; the tape says *sunk, and back to the last town you stood in* |
| what a pocket lacks | `to` / `at_to` | `PlaceDef` has both fields; only `Gate` reads them. `shot::shoot_with` reads a pocket **at rest and nowhere else**, which is what makes it a pocket rather than a hole |
| the shot | 72 angles × 10 powers, `POWER_UNIT` 30, `RESTITUTION` 80, `BUMPER_KICKS` 3, `SPIKE_TIRES` 8, `MAX_TICKS` 400 | `shot.rs`. `aim_at` sweeps the 720 gentlest-first in under a second and is the one answer the lint, the gate and the walker share |
| the lint that names the tables | `only_the_country_maps_are_tables` asserts `["the-treyway", "the-undercountry"]` | `tests/shot.rs:290`. `CLAUDE.md` calls a third *"a decision somebody makes there"*. This plan makes it |
| the stop-line | a `Door` at **[10, 11]**, one tile south of the town, the **only** user of `PlaceKind::Door` | `the-undercountry.tiles.json`; divergence 14.6 put it on a door so the kind kept a user |
| the Undercountry's boss | The Unwritten, **12,000** health, 80 strength, **no errand, no drops** | `combat.rs:4261`; nothing in `quests.json` names it. The longest fight in the game and nothing points at it |
| maps, places, data files | **29 maps**, **220 places**, **16 + 29 = 45 data files** | measured. **`CLAUDE.md`'s numbers table says 25, 215-over-25 and 29** — three rows have gone stale under four blocks, and M22.9 fixes them because a number nobody can reproduce is not a number |
| errands | **63** | `quests.json` |
| creatures | **77**, twenty art families | `art/creatures.json` |
| kennel offer | at **5** prior wins, never a boss | `kennel::OFFER_AT`; `no_boss_is_kennelled` |

**Four things the ask assumes that are not how this works**, each answered in
the decisions below:

1. *The town has nothing.* It has five of seven buildings, and the best bed in
   the game. Settling it is the market, the guild and a name — not five
   systems.
2. *High Wick becomes the third town.* It cannot without moving a save: a
   shelf is keyed by place id and a sale by `(id, index)`, and the town's name
   is the human's. High Wick's **shelf** comes down as a wing of the town; the
   town stays `the-third-town` and gets its name on the post.
3. *A pocket that leads somewhere is a mechanic.* It is one field. A pocket
   already warps you; `to` says where.
4. *Finishing an errand opens things.* Nothing reads `quests_done` but the
   quest log. One line in `place_is_there`'s `met` closure makes it true.

---

## Decisions taken before anything is built

1. **No new components, and no new save fields.** A wing is a `TownShelf`, a
   table is a map, a name is a field on a place. The fingerprint does not move
   and `save.rs` is not touched. There is a player mid-run at level 45 and this
   block is for them.
2. **Two halves, one block, in this order: the town, then the table.** The
   chain that settles the town ends at The Unwritten, and The Unwritten is what
   the gate south names in its refusal. Built the other way round the table is
   reachable from a town that still says the writing stops here.
3. **A wing is a shelf with a host and an arrival.** `TownShelf` gains
   `wing_of: Option<String>` (the host town's place id) and `arrives:
   Option<String>` (a key `met` would accept). A wing's stock, commissions and
   errands appear on its host's screen once it has arrived, keyed by the wing's
   own id — so `bought` and the index contract are untouched, and `high-wick`
   stays `high-wick`. Two wings come down: **the clerk's desk** (errands only)
   and **the arcane shelf** (High Wick's seventeen and two commissions).
4. **`met` reads `quests_done`, and that is the whole of the new predicate.**
   `hidden_until`, `hidden_until_all`, `needs_all`, a wing's `arrives` and a
   name's `when` all go through one closure; a finished errand joining
   `answered` and `flags` there is one line and no save field. A `done:` prefix
   written into `answered` at hand-in was considered and rejected: an old save
   that finished the errand before this block would need a backfill, and a
   predicate that reads the list that already exists needs none.
5. **Every refusal this block adds is owned by exactly one of `parse` and a
   lint, and the lint's negative test goes through `parse`.** This is
   `4a3ae9a`'s finding taken before it costs anything: `StallData::parse` and
   `a_bargain_is_never_on_the_barrel` asked one question, so pricing an ench to
   break the lint panicked in `data.rs` before the assertion ran. So:
   **`ShopsData::parse` owns what a shops file can be asked about itself** — a
   wing with no `arrives`, a `to` with no `at_to` — and the **lint owns what
   `parse` has no business asking**, which is whether the host is a town *on a
   map*, because that reads the map files. The lint's negative test hands a
   mutated `SHOPS_JSON` to `parse` and asserts the refusal, exactly as the
   bargain check now does.
6. **The empty town stays a decision until it is not one.** `common::UNWRITTEN`
   and `avail.rs`'s `STAGED` are both emptied **and asserted empty**, not
   deleted — the assertion is what turns *nothing is staged* from an absence
   into a claim. The three lints that exempt the third town by name flip to
   asserting it trades and wants something.
7. **A place can have a name that arrives.** `PlaceDef::named: Option<Named {
   when: String, name: String }>`, read by one function, `World::name_of(p,
   state)`; the theme table carries both names. The post says *a town with no
   name on the post yet* until the chain's last rung is handed in, and then it
   says the name the human chooses (§*What is the human's*). Nothing else in
   the game changes what it is called, and this field is the only way anything
   may.
8. **A pocket may say where it goes, and it still costs twelve.** `Pocket`
   honours `to`/`at_to` the way `Gate` does; `Game::shoot`'s sunk arm warps
   there when they are set and to the last town when they are not.
   `POCKET_TIRES` stays one number: *a pocket has to be worse than a spike or
   nobody aims around it* is still true of a pocket that is a door.
9. **The boss is behind a wall, and the way in is a pocket — and the wall is a
   map boundary, because rock is not one.** ~~The table's boss stands in a
   sealed cup of rock~~ — M22.0 measured that and **a cup of rock is not
   sealed**: `shoot_with` tests the tile a tick landed on and never the tiles
   it crossed, so at power ten a ball skips eighteen tiles and 6,546 shots from
   outside the draft cup came to rest in it. No thickness fixes that. So **the
   cup is a map of its own**, one room, and the only way onto it is the far
   pocket, whose `to` is that map and whose `at_to` is the tile you fall onto.
   That is *exactly* what decision 8 builds — `warp_to(to, at_to)`, the call
   the sunk arm already makes — so the rule is unchanged and only the
   destination is. It is still the one thing Yoku's holes do that this engine
   did not: a hole is how you go *into* a room. The lint is then over the
   **map** rather than over a flood: the boss stands on a map with one way onto
   it and that way is a pocket, and some shot on the table reaches the pocket.
10. **The block ships its own start lines, and says so in the README.** No save
    in this repository opens the Undercountry and the walker will never get
    there — 977 fights to level thirteen and then the loop. `GM2D_FROM` is the
    mechanism and `testing/saves/README.md` is the table it goes in: two new
    files, `in-the-third-town.json` and `on-the-lower-table.json`, so that a
    person — the human included — can stand in this block's content in ten
    seconds rather than behind a boss they have not beaten.
11. **The stop-line moves one map on, on the same kind.** The Undercountry's
    `the-way-on-from-here` becomes a `gate` onto the table with `needs_all:
    ["the-unwritten"]`, so it is there from the first visit and refuses naming
    the boss still standing — `Game::sealed_because`, two registers on one
    line, which is rule 8 and divergence 14.4 both. A new `Door` stands at the
    far end of the table saying what the old one said. `PlaceKind::Door` keeps
    exactly one user.
12. **The table's pool is the deep ladder, and the one new creature is dressed
    by damage a second against `common::geared_from`.** ~~the yardstick is
    `common::from_save(common::THE_RUN)`~~ — M22.0 measured it and **the run
    beats nothing at this depth**: 974 health and 9 strength over 11 items, a
    loss to The Unwritten in 7.7 seconds and to all ten of the Wextreen deep.
    A bracket against it is a bracket only something shallower than the map's
    own pool could meet. `geared_from` is what
    `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten` has
    required of every boss on a tile since M11.7, and it is what the Tenth
    Surveyor was actually bracketed against — `CLAUDE.md`'s own sentence is
    *the block is bracketed against the board that can actually reach her*. The
    run stays in the test as the **floor**: it loses, and that is asserted, so
    nothing is ever tuned down to something a weak level-45 caster board walks
    over. `make dress` and `make read` are the tools.
13. **The Kennel reaches the table for free and is not touched.** Five wins and
    the sixth offers, never a boss. The table's pool is chosen so at least one
    member is worth kennelling at level 45, which is a content choice.
14. **Every player-visible sentence is `TONE.md`'s, and every proper noun is
    the book's.** The town's name, the map's name and the boss's name are the
    human's; the plan ships descriptive defaults, as M14 did with *a town with
    no name on the post yet*.
15. **The builder does not ask.** Every open decision has a recommendation
    beside it; the builder takes it, says so in the commit, records it in the
    notebook. Deploys are at the two points named.
16. **The notebook starts empty.** M21's is closed at sixty-three rows with
    none open, so `SECOND-ORDER-M22.md` begins at row 1 and nothing is carried
    in. M22.10 onward are read off it.
17. **A status table after every milestone**, appended to `MILESTONES.md` under
    `## M22 — High Wick, and the table`.

---

## Milestones

Order: **M22.0 → M22.9**, then whatever the notebook adds. Deploy after M22.4
and after M22.9.

### M22.0 — The measure

*Nothing a player can see. Every guess in this plan replaced by a count.*

| deliverable | what |
|---|---|
| the casting count | how many of the 106 casting components are buyable with `high-wick` placed — the plan guesses 92 |
| the errand at 45 | how long `the-long-mirror-inventory` takes from the third town: the long cart to West Bambulon, three Mirror Fiends, back. A number of fights and a fatigue figure |
| the table's flood | `every_place_on_a_shot_map_is_reachable_by_shots`'s flood run over the draft rows in §M22.6, with the cup sealed — which tiles are reachable, in how many rounds, and whether the far pocket is among them |
| the cup | from every tile inside the draft cup, does any of the 720 shots come to rest on the boss's tile or catch it. **This is the number decision 9 depends on** |
| the yardstick | `common::from_save(common::THE_RUN)` rated, and its damage a second against the Undercountry's five, so a sixth can be placed beside them |
| the check to flip | what `check_the_third_town_is_empty_on_purpose` asserts, line by line, so M22.3 **rewrites** it rather than deleting it — a check about a thing is an arm of the check that already reaches that thing |

Acceptance: every number above in the commit message, and this file's own
figures corrected where they were wrong. **If the cup measures open, decision 9
is wrong and the cup is redrawn before a map is authored.**

### M22.1 — A wing is a shelf with a host

*The rule, with no content behind it yet.*

| deliverable | what |
|---|---|
| `TownShelf::wing_of`, `TownShelf::arrives` | both `Option<String>`, `serde(default)`; a shelf with a `wing_of` is a wing and one without is a town |
| `world::met` | `place_is_there`'s closure lifted out, given a name, and taught `quests_done`. **One closure, five callers** |
| `shop::wings(shops, town, state) -> Vec<&TownShelf>` | every wing whose host is `town` and whose `arrives` is met |
| `data::shelves_on_the_map()` | `towns_on_the_map()` plus every wing whose host is on a map. `nothing_in_the_barrel_is_on_a_shelf_you_can_reach` and `ledger_pool` ask this instead |
| `ShopsData::parse` | refuses a wing with no `arrives` — a wing that is always there is the host's own stock and belongs in the host's list. **And nothing else**: whether the host is on a map is the lint's, because `parse` does not read map files (decision 5) |
| the screen | the market draws the host's shelf and then each arrived wing's, under its own heading; the guild draws each arrived wing's errands. **No shim rule**: `wings_json` is core's list and the page draws it |
| lints | `a_wing_has_a_host_on_a_map`, negative-tested **through `parse`** on a mutated `SHOPS_JSON`; `a_wing_arrives_or_it_is_the_hosts_own`; `STAGED` emptied with `assert!(STAGED.is_empty())` and the reason written beside it |

Acceptance: `a_wing_is_not_on_the_counter_until_it_arrives`;
`a_wing_sale_is_keyed_by_the_wing_and_survives_the_host_selling`;
`a_finished_errand_meets_a_hidden_until`. **Nothing a player can see**, because
no wing is authored yet.

### M22.2 — The clerk comes down

*The first wing, and the errand that brings it.*

| deliverable | what |
|---|---|
| `send-for-the-clerk` | giver `kettleworks`, `requires: ["nobody-has-named-it"]` — the same counter that sent you to confirm the post was empty is the one that follows up. Goal: `Bring { item, count: 1 }` of something the Undercountry drops, so the errand is *go down and come back with proof*. Gold at tier, no gear |
| `the-clerks-desk` | a `TownShelf`, `wing_of: "the-third-town"`, `arrives: "send-for-the-clerk"`, empty stock, no commissions |
| `the-long-mirror-inventory` | giver re-keyed `high-wick` → `the-clerks-desk`. **Seam-free**: a save carries quest ids, not givers. Gold to the third town's tier; the ink and the sigil stay, because they are the reason to do it |
| the guild | appears on the street the moment the desk has arrived, because the panel is no longer empty — `paintStreet` already asks the rendered panel and not a list of towns |
| theme | the desk's name and blurb, in register: she counts volumes |

Acceptance: `the_clerk_is_not_there_until_sent_for`;
`the_third_town_wants_something_once_the_clerk_is_down` (the flipped lint);
`the_mirror_errand_is_given_at_the_desk_and_nowhere_else`.

### M22.3 — The arcane shelf comes down, and the post gets its name

| deliverable | what |
|---|---|
| `high-wick` | gains `wing_of: "the-third-town"`, `arrives: "the-long-mirror-inventory"`. Seventeen lines, two commissions, index order untouched |
| `cut-the-post` | giver `the-clerks-desk`, `requires: ["the-long-mirror-inventory"]`, goal `Clear { "the-unwritten" }` — **the first errand that has ever pointed at the longest fight in the game.** Gold at tier and one thing off no shelf |
| `PlaceDef::named` | decision 7; `named.when` is `cut-the-post`; `World::name_of` is the one reader; the map panel, the strip, the long cart's list and the theme all go through it |
| the gate south | `the-way-on-from-here` → `kind: "gate"`, `to` the table, `at_to` its tee, `needs_all: ["the-unwritten"]`; its `shut` names the post and `sealed_because` appends the boss |
| `UNWRITTEN` | emptied, `assert!(common::UNWRITTEN.is_empty())` in its place with the reason; the three exempting lints flip to asserting the town trades and wants something |
| the barrel | recount, written into the commit beside M22.0's figure |

Acceptance: `the_shelf_arrives_when_volume_ten_is_open`;
`the_post_is_named_when_it_is_cut`; `the_way_south_refuses_naming_the_
unwritten`; `no_town_is_unwritten_and_nothing_is_staged`.

### M22.4 — The gate, the walk, the deploy — the town

| deliverable | what |
|---|---|
| `check_the_third_town_is_empty_on_purpose` → **`check_the_third_town_fills_up`** | the same plant at [10, 5] and the same shot to [10, 10], rewritten: the shelf *has* things on it, the arcane wing sells under its own heading, the guild is on the street, and the post reads its name. **Rewritten rather than joined by a second walk to the same tile** — M21 row 42's rule |
| `check_the_way_south_names_the_boss` | its own line, because it is a different tile — [10, 11], where the ending screen used to be |
| the start line | **`testing/saves/in-the-third-town.json`**, both bottoms answered, standing on the tee, plus its row in `testing/saves/README.md` |
| the walk | `GM2D_FROM=testing/saves/in-the-third-town.json make play` — sends for the clerk, does the inventory, cuts the post. Steps between rungs written into `CLAUDE.md`. **A bare `make play` is not evidence about this map**: from a new game the walker reaches level thirteen and loops, which is `PLAN.md` §6d row 3 and not a bug |
| the glossary | nothing new — no constant was added, and the zero is written down rather than assumed |
| `CLAUDE.md` | *High Wick comes down* as a section; the divergences; the numbers |
| **deploy** | `make publish`, report the hash |

Both checks negative-tested, each clearing with `clear_screens` before its
first click.

### M22.5 — A pocket can go somewhere

*The one rule the table needs, built before the table.*

| deliverable | what |
|---|---|
| `Pocket` honours `to` / `at_to` | `PlaceDef::opens_onto` answers for a pocket as it does for a gate; **`parse` refuses a `to` with no `at_to` on any kind** (decision 5: the file can be asked this about itself) |
| `Game::shoot` | the sunk arm: `to` set → `warp_to(to, at_to)`; unset → the last town, as now. Twelve fatigue either way |
| the tape | `Flight::tape`'s sunk sentence names where you went, off the pocket's name — *sunk, and down the far pocket into …* |
| `only_the_country_maps_are_tables` | the list gains the table's id **now**, so M22.6's file is refused until it exists and not after |

Acceptance: `a_pocket_with_a_to_lands_where_it_says`;
`a_pocket_without_one_still_goes_home`;
`a_pocket_that_goes_somewhere_costs_the_same_twelve`;
`every_warp_lands_somewhere_you_can_stand` extended over pockets.

### M22.6 — The table

*The riskiest milestone in the block, and the recon says why: a tile a step
away cannot be shot to, and a cup that measures open is a boss with a straight
line to it.*

The draft. **Every row is a guess and M22.0's flood replaces it**; what is not
a guess is the list of properties under it.

```
    01234567890123456789
 0  ^^^^^^^^^^^^^^^^^^^^
 1  ^A,,,,^^^^^^^^,,,,A^     the cup: rock, rows 1-4, cols 6-13, no mouth
 2  ^,,,,,^,,,,,,^,,,,,^     boss [8,3]; the door [12,2]; the far pocket's at_to [11,3]
 3  ^,,,,,^,,,,,,^,,,,,^
 4  ^,,,,,^^^^^^^^,,,,,^
 5  ^,,,,,,,,,,,,,,,,,,^     bumper post [9,5]
 6  ^,,%%,,,,,,,,,,%%,,^
 7  ^,,%%,,,,,,,,,,%%,,^     the far pocket [17,2], behind the range; sand [1,2], [18,5]
 8  ^,,,,,,,,,,,,,,,,,,^
 9  ^,,,,,,,%%%%,,,,,,,^     bumpers [5,9], [14,9] flanking the island
10  ^,,,,,,,%%%%,,,,,,,^
11  ^,,,,,,,,,,,,,,,,,,^     events [3,8], [16,8], [9,11], [2,12], [17,12]
12  ^,%%,,,,,,,,,,,,%%,^
13  ^,%%,,,,,,,,,,,,%%,^
14  ^,,,,,,,,,,,,,,,,,,^     spikes [2,14], [17,14] - the gutters bite
15  ^,,,,,,,,,,,,,,,,,,^     chute [18,15] -> [18,6], the plunger lane up the right rail
16  ^^,,,,,,,,,,,,,,,,^^
17  ^^^,,,,,,,,,,,,,,^^^     gutter pockets [4,17], [15,17] (home)
18  ^^^^,,,,,,,,,,,,^^^^     tee [9,18]; the way back up [9,17]
19  ^^^^^^^^^^^^^^^^^^^^
```

| deliverable | what |
|---|---|
| `data/maps/the-lower-table.tiles.json` | 20×20, `traversal: "shot"`, `start` on the tee; `data::MAPS` and `data::FILES` both, and `MAPS.len()` to 30 in `tests/undercountry.rs:218` |
| places | 1 gate up · 1 boss · 1 door · 3 pockets, one with a `to` · 3 bumpers · 2 spikes · 1 chute · 2 sand · 5 events — **19**, asserted |
| the stop-line | `the-writing-stops-here`, `kind: "door"`, `needs_all: [the boss's tile id]`, in the cup behind the boss; the old door's three paragraphs with one fact added. The Undercountry's tile is now M22.3's gate |
| the region | one, `the-lower-table`, a pool of **four** existing Wextreen-deep creatures chosen in M22.0 by damage a second. **The boss is not in it**: a boss that also stands in a pool is dealt in the field on behalf of a cup nobody has reached — divergence 15.1's finding, applied before it is a bug |
| events | five; `Outcome::Give` on three, so the table pays gear a `Bring` can ask for; two notes; none a chain root, because the chains are the desk's |
| theme | every place named; the map ships as *the table under the writing* |
| lints | `the_boss_has_no_straight_line` (over the flood, not the file); `every_pocket_on_the_table_goes_somewhere_named`; `the_stop_line_is_still_one_door`; `every_place_on_a_shot_map_is_reachable_by_shots` over three tables |

Acceptance: the four lints; `every_gate_lands_beside_its_door` over the new
gate; `every_placed_event_exists_exactly_once`;
`reachability_derives_over_every_map` at **30**. `aim_at` finds a shot to every
landable place from the tee in **at most three rounds** — the plan guesses two,
which is the Treyway's number.

### M22.7 — What stands on it

| deliverable | what |
|---|---|
| the boss | **one new creature**, in the cup, wearing two slots of an existing creature's board so it invents no component — the Tailgate's rule. Dressed by damage a second against `common::from_save(common::THE_RUN)` to a win between **20 and 28 seconds** and never at `SUDDEN_DEATH_MS`; the body numbers are the costume. Ships as *The Twelfth Name* |
| `art/creatures.json` | one entry, a colourway of an existing family — `mirror` if it reads, for the clerk's reason. `cargo test` refuses a creature without one, and `make art`'s own rule is *draw it, then look at it* |
| the pool | four, placed; `no_boss_is_kennelled` over the new one; `a_set_is_never_behind_the_rarest_fight_in_its_region` reads it |
| the kennel | nothing built — asserted: `the_tables_pool_can_be_kennelled` walks `kennel_offer` over the four and expects `Ok` on the sixth win |

Acceptance: `the_twelfth_name_is_a_fight_the_run_wins_in_under_thirty`;
`the_twelfth_name_wears_no_new_component`;
`the_twelfth_name_stands_nowhere_but_the_cup`; `every_creature_has_a_figure`
at 78.

### M22.8 — Three errands you do with a cue

*The desk's second chain: the table as the guild's business.*

| rung | goal | answered by |
|---|---|---|
| `the-far-corner` | `Word { place: … }` | landing there — *a landing on a table is an arrival*, which is M19's rule and is already true |
| `what-the-table-pays` | `Bring { item, count: 1 }` | the third event's `Give`, carried back up |
| `the-twelfth-name` | `Clear { … }` | the cup, by the far pocket |

| deliverable | what |
|---|---|
| three errands | `granted` after `cut-the-post`, given and handed in at the desk, gold at tier, the last paying one thing off no shelf |
| `quest::guide` | points at the table for all three |
| lint | `every_rung_is_a_thing_the_table_already_does` — no rung needs a mechanic that is not on the map |

Acceptance: `the_far_corner_is_answered_by_a_landing`;
`the_tables_gear_comes_back_up_over_the_counter`;
`the_stop_line_is_behind_the_twelfth_name`.

### M22.9 — The gate, the walk, the glossary, the deploy — the table

| deliverable | what |
|---|---|
| browser checks | `check_the_way_south_opens_onto_a_table`; `check_a_shot_animates_to_where_core_said` over the new map; `check_the_far_pocket_drops_you_in_the_cup`; `check_the_gutter_pocket_sends_you_home`; `check_the_stop_line_moved`. Each negative-tested; each clearing with `clear_screens` first. **None of them waits for `#stage-replay`** — Instant Battle draws none, and at level 45 most of the ladder is marked (`4a3ae9a`) |
| the start line | **`testing/saves/on-the-lower-table.json`**, on the tee with the gate open, plus its README row |
| the walk | `GM2D_FROM=testing/saves/on-the-lower-table.json make play` — down the gate, one event, the far pocket, the boss, the door. The number of shots into `CLAUDE.md` |
| the glossary | the pocket's second sentence on its shelf; every number read from its constant |
| `CLAUDE.md`'s stale rows | **Maps 25 → 30, Places 215-over-25 → 239-over-30, Data files 29 → 46.** Three rows that went stale under four blocks, fixed here because the table exists so a regression is visible |
| `HANDOFF-M22.md`, `CLAUDE.md` | *The table under the writing* as a section; three tables; the divergences; the numbers |
| **deploy** | `make publish`, report the hash |

### M22.10 → — Whatever the notebook says

**These milestones do not exist yet, and nothing is carried in.** M21's
notebook closed at sixty-three rows with none open, so `SECOND-ORDER-M22.md`
starts at row 1. When M22.9 is done the builder reads it, takes every row
marked `open`, groups them into milestones of the same shape as the ones above,
numbers them, appends them here, executes them, closes the rows, and writes the
status table after each. A row that is a finding rather than work is closed
with the finding in `CLAUDE.md`; a row that is the human's is closed by writing
it into the last status table's notes and carrying it. The block ends when the
notebook has no open rows.

---

## Status table

After **every** milestone's commit, this table is appended to `MILESTONES.md`
under `## M22 — High Wick, and the table`, one row per milestone done or
pending, and the same table is printed at the end of that milestone's final
message:

```
| # | milestone | deliverables | tests | commit | state |
|---|---|---|---|---|---|
| M22.0 | The measure | 6 numbers, plan corrected | 1097 (+0) | a1b2c3d | done |
| M22.1 | A wing is a shelf with a host | wing_of · arrives · met · shelves_on_the_map · 3 lints | … | … | in progress |
| M22.2 | The clerk comes down | … | | | pending |
| … | | | | | |
| notebook | rows open / closed | 0 / 0 | | | |
```

`tests` is `packaging/count-tests.sh`'s number and the delta from the previous
row — never a figure read off `cargo test`, which interleaves and cannot be
summed. `state` is one of `done`, `in progress`, `pending`, `added`, `dropped`
(with the notebook row that says why). The notebook line is always last.

---

## Numbers

Every count the block commits to, in one place. **G** marks a guess the builder
replaces in M22.0; everything else is measured against `4a3ae9a`.

| thing | number |
|---|---|
| maps | 29 → **30** |
| tables | 2 → **3** |
| places | 220 → **239**; on the new map **19**: 1 gate, 1 boss, 1 door, 3 pockets, 3 bumpers, 2 spikes, 1 chute, 2 sand, 5 events |
| pockets with a `to` | **1** |
| `PlaceKind::Door` users | **1**, before and after |
| wings | **2** — the desk and the arcane shelf |
| shelf lines that come down | **17** + 2 commissions |
| errands | 63 → **68**: `send-for-the-clerk`, `cut-the-post`, three for the table; one re-keyed |
| errands pointing at The Unwritten | 0 → **1** |
| creatures | 77 → **78** |
| save fixtures | 14 → **16**, and **0 of 14 opened the Undercountry** |
| new components | **0** |
| new save fields | **0** |
| new `PlaceDef` fields | **1** — `named` |
| new `TownShelf` fields | **2** — `wing_of`, `arrives` |
| new predicates | **0** — `met` reads one more list |
| new constants | **0** |
| casting components buyable | 89 → **92 G** |
| `UNWRITTEN` / `STAGED` after | **0 / 0**, both asserted |
| rounds of shots to reach everything on the table | **2 G**; three is the ceiling |
| the boss against the level-45 board | **20–28 s G**, never 30,000 ms |
| new lints | **9** |
| browser gate | 108 → **114**: six new `ok:` lines and one check rewritten |
| notebook rows carried in | **0** |
| deploy points | **2**; approvals asked **0** |
| games cited | 6 |

## What is the human's

| decision | recommendation, taken without asking |
|---|---|
| **the town's name.** Ships as *a town with no name on the post yet* until `cut-the-post` | **Low Wick.** High Wick's people, one country down, and a wick is a thing lit at the bottom. Two of the book's words, inventing nothing |
| **the table's name.** Ships as *the table under the writing* | keep it. The stop-line's prose already calls this place *what it stops at* |
| **the boss's name.** Ships as *The Twelfth Name* | keep it. The plank on the first map burns eleven names and a twelfth with nothing after it, and this is what goes there |
| whether the Mirror Fiend errand stays a three-fight stroll on the first map | **yes.** It is the arrival tale and the long cart makes it forty Fnorp; re-keying it to a deep creature makes the clerk's own errand a wall. Gold moves to tier |
| whether the cup's pocket costs the same twelve as a gutter | **yes** — one number for a pocket; a cheaper way into the boss's room is a discount on the hardest shot on the map |
| whether the table is surveyable | **no, this block.** A third arm on `survey::mods_for` is a milestone and it is not one of these |
| whether the table's four are re-dressed or placed as they are | **as they are.** Nine of the Wextreen deep's ten already beat `geared_from`, and the level-45 board is the measure |
| whether `send-for-the-clerk` is offered at Kettleworks or at the third town | **Kettleworks**, after `nobody-has-named-it`. An empty town cannot ask for its own clerk |
| whether the Woman From High Wick's Stall blurb changes | **no.** *Nobody has been* is still true — the shelf came down, the town did not |

**And one thing that is not a decision but is yours to know.** Your own save
has `the-bottom-of-the-bottom` in `answered` and **not** `the-ninth-surveyor`,
so Marbulon's door has not opened for you and none of this block's content is
reachable from where you are standing. That is why decision 10 ships two start
lines: load `in-the-third-town.json` and you are in it, or beat the Ninth
Surveyor at the bottom of the Wextreen Sump and the door opens the way it was
written to.

## The riskiest milestone

**M22.6.** Three reasons, each already written down somewhere in this
repository: a tile a step away cannot be shot to, so every one of the nineteen
places wants a tee and the draft has not been shot at; `preview_shot` is
memoised on angle and power, and a table whose ground changes state is the tide
crossing's lesson; and a cup that measures open is a boss with a straight line
to it, which makes decision 9 a sentence rather than a design. M22.0 shoots the
draft before a map is authored, and if the flood disagrees with the rows, **the
rows are wrong and the plan is not.**

## What this block deliberately leaves

**The Crypt** — a descending post-game dungeon under the stop-line. The human
called it too early and it is; here is what it would cost so the next plan does
not measure it again. `Difficulty::factor()` is 0.5 / 1 / 3 / 9 and the whole
game runs on `Easy` (`const DIFFICULTY` in the shim, `fight.rs:200`);
`extra_items_at` steps a creature's gear a tier and copies one or two items on
Hard and Insane; `simulate_party` takes an enemy slice and has no caller in the
game. **A floor that regenerates cannot live in `answered` or `flags`**, which
only grow, so it is a save field — the first since M13 — and `Difficulty` has
to stop being a constant in the shim and become a value core hands out per
floor. And **Instant Battle draws no replay** (`4a3ae9a`), which is a feature
and is also the thing a floor counter would have to survive. Sources when it is
wanted: FFXIV's Palace of the Dead, Pokémon's Battle Tower, Diablo III's
Greater Rifts.
