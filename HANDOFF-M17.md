# HANDOFF-M17 — the overworld is a table

`PLAN-M17.md` is the frame. This is what was built, what came out differently,
and what is still open.

**Two countries are tables now.** The Treyway and the Undercountry have
`traversal: "shot"` in their map files: the arrow keys aim a cue instead of
taking a step, space fires, and the ball runs to rest under integer physics.
*Where you stop is where you are.*

Every other map is untouched. A floor is still walked a tile a press, and
`check_a_floor_still_steps` is what keeps that true — which turned out to be
the hardest check in the block to negative-test, because every lie about *the
arrows mean two things now* takes the whole browser gate down before the check
runs. A floor that grows a cue is a dungeon nobody can walk out of.

## The milestones

| # | what | where |
|---|---|---|
| M17.0 | the table, in core — `shot.rs`, `Traversal`, and no map set to it yet | `6b72479` |
| M17.1 | five obstacle kinds, parse lints, the Treyway drawn as a table | this commit |
| M17.2 | the landing — `Game::shoot`, `world::arrive_at`, the roll at 150%, the tape | this commit |
| M17.3 | the cue — the drag, the keys, the flight, the strip, reduced motion; seven browser checks | this commit |
| M17.4 | `shot::aim_at`, the walker aiming, the reachability lint sharpened | this commit |
| M17.5 | the Undercountry as the second table, and this document | this commit |

## The physics, in one table

| | |
|---|---|
| unit | **sixteenths of a tile**, `i32`, no `f32` anywhere in `shot.rs` |
| angles | **72**, five degrees apart, `(cos, sin)` in sixteenths from a `const fn` table |
| powers | **1..=10**, `POWER_UNIT` 30 sub-cells a tick a point |
| bounce | `RESTITUTION` 80%, tangent kept |
| friction | per **tile crossed**, off the terrain's own cost |
| at rest | speed under `STOP_BELOW` (6) |
| bound | `MAX_TICKS` 400, never reached by a legal shot |
| bumper | `BUMPER_KICK` = 3 power-units, **added**, capped at `BUMPER_CAP`, at most `BUMPER_KICKS` (3) a flight |
| spike | `SPIKE_TIRES` 8%, and the ball passes through |
| pocket | `POCKET_TIRES` 12% and the walk home |
| landing | rolls at `LANDING_MULT` (150%) of the tile's own rate |

**No trigonometry at runtime and no `sqrt`**, because a seeded walk has to
produce the same flight in three engines and a float is the one thing that
rounds differently in each. `the_physics_has_no_floating_point_in_it` is a lint
over the **source** rather than over the behaviour, and it has to be: an `f32`
produces a flight that is *nearly* right, which a hash cannot tell you about
until somebody in another browser reports it.

## What came out differently, and why

| # | divergence | where |
|---|---|---|
| 1 | **`POWER_UNIT` is 30 and §3 says 22.** The plan chose every constant on paper and says the recon has to check them. What the sweep found is a property the plan does not name: **distance has to be monotone in power**, because a cue where pulling back harder lands you *nearer* is a cue nobody can aim. At 22 it is not, and at every restitution below 80 it is not — a fast ball spends its extra speed on ricochets, and a bounce that costs too much makes a strong shot die at the wall it hit. 80 and 30 is the only pair in the sweep with no step backwards. | `shot.rs` |
| 2 | **A bumper adds a fixed kick and is counted, where §4 says +30%.** A percentage compounds: reflected at four fifths and boosted a third, a ball between a bumper and a wall gains four percent a round trip, for ever — shots ran to `MAX_TICKS` and a ricochet off the pass could outrun a full-power shot. A fixed kick settles at about `2.2k`. **And still never stops**, because friction is charged per tile crossed and a ball bouncing in place crosses none, so the pump is counted at three. | `shot.rs` |
| 3 | **Four of §4.1's eight obstacle coordinates are on impassable range** and one shares the tideline's own tile. Same class as M16's floor drawings: the ASCII in a plan is a sketch and the rows are the map. Intent kept, each moved to the nearest ground that carries it. | `the-treyway.tiles.json` |
| 4 | **What a gate beside you offers is its refusal, never the way through.** §2.6 says a wall is not a place and such a gate is reached by stopping beside it; the first draft therefore offered the gate, and walked a player over a bar still under nine feet of water. **The tide crossing has never carried a condition of its own, because the impassable ground *was* the condition** — enforced by `walkable` refusing the step before anything asked the place, so a rule that reached it without stepping bypassed the only lock it had. Once the tenth cairn drains it, the tile is `coast` and it is entered by landing on it like every other gate. | `world.rs`, `gate_beside` |
| 5 | **`Step::crossing` became `Step::refused_by` in M15** and this block is the second reader of it: a beside-refusal is a fact about where the game goes next, so it goes to the strip and not into the one-second flash. | `world.rs` |

## What only a browser could say

Seven checks, all seven negative-tested:

| check | what it caught when broken |
|---|---|
| `check_the_cue_draws_what_core_is_asked` | *pulled straight back and the cue reads 27, which is not north* |
| `check_a_shot_animates_to_where_core_said` | *the page drew 3 points and core returned 18* |
| `check_the_keyboard_shoots` | *the arrows drew no cue on a table* |
| `check_a_spike_flashes_and_the_fatigue_moves` | *a shot through a spike left fatigue at 0* |
| `check_sunk_lands_you_in_town` | *sinking did not open the town* |
| `check_a_floor_still_steps` | *an arrow on a floor drew a cue* |
| `check_reduced_motion_skips_the_flight` | *reduced motion dropped the trail as well as the flight* |

**And the block's own fault was found by the gate rather than by `cargo test`.**
The beside-gate rule opening a bar under nine feet of water failed three checks
in three engines; nothing in core could see it, because every core test that
could have asked was asking *is the gate offered* rather than *should it be*.

## Three things that cost an hour each, and are worth not paying for twice

1. **A tile one step away is a tile you cannot shoot to.** From directly
   adjacent every power overshoots or bounces off the wall behind, so three
   gate checks that planted themselves beside their target and fired never
   arrived. Not a fault — it is what a cue is — and the fix is a tee rather
   than a nudge. **Draw two places next to each other on a table knowing it.**
2. **A click on the canvas fired a full-power shot.** `pointerdown` set the cue
   from wherever the pointer was and `pointerup` fired it, so somebody clicking
   the map to focus it for the keyboard took a shot from the far side of the
   table. You pull the cue now; you do not tap the table.
3. **A shot can end on any screen, so a check that fires one must tidy any of
   them.** `dismiss_card` and `close_fight` cover two of six; a `finally` that
   covered two left a town over the page, the next check's first click timed
   out, and the whole failure list went unprinted. `clear_screens` is the one
   door.

## The lint that could not have found it

`every_place_on_a_shot_map_is_reachable_by_shots` accepted *landable **or**
landable beside*, which cannot tell a gate you can enter from one you can only
be turned away from — this repository's *a lint that reads a list rather than
the behaviour is the failure it exists to catch, one level up*, in a new coat.

It asks two sharper questions now:

- **Can a ball come to rest on this tile, in some state of the world** — the
  map flooded **and** drained, because a map is not one grid and a drain turns
  `tide` into `coast`.
- **And of an obstacle, can any shot hit it** — because a bumper is a tile no
  ball can ever come to rest on, which is what a bumper *is*. Exempting
  obstacles outright would have let an unreachable one through.

## The numbers

| | |
|---|---|
| tables | 2 — the Treyway (16×16, nine obstacles) and the Undercountry (20×20, six) |
| obstacles | 15: 4 bumpers, 5 drifts of sand, 3 spikes, 2 pockets, 1 chute |
| reachability | everything on either table, in **2** rounds of shots |
| `make play` | crosses the Treyway in **3** shots, against the plan's ceiling of 9 |
| `aim_at` | 232µs a call — the walker calls it once a move |
| browser gate | **85** `ok:` lines in one engine, seven of them this block's |
| save format | **no seam.** `shots-taken` is a counter and counters already round-trip; `traversal` is in the map file, which is content |

## What is open

- **`SECOND-ORDER-M17.md` has twenty-nine rows** and every one is closed. The
  worklist row (10) was the beside-gate rule, and rows 16–19 are what it turned
  into once it was built and found to be wrong.
- **The Treyway is a small table.** Seven hundred and twenty shots from its
  start reach 164 of its 176 walkable tiles, and everything on it is two rounds
  away. `PLAN-M17.md` §10.3 asks whether 16×16 is too small to be interesting
  and says to report rather than fix. **That is the report, and the call is the
  human's.**
- **The Undercountry's layout is still M14's guess**, which §4.2 and M14 §9
  decision 2 both leave where they found it. It is a table now and its six
  obstacles are the builder's; what goes in the empty town is the next plan's.
