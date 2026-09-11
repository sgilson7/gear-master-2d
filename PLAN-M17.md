# PLAN-M17 — The country is a table

*A stretch goal, architected. On the two country maps you no longer walk:
you pull back, aim, and fire yourself across the Treyway like a ball on a
table. Where you stop is where you are. Written against the tree at
`b3fdb4f` with M16 assumed landed: twenty-four maps, the Treyway at 16×16
with the tide crossing on its south edge, `world::step` as the one way a
player moves, and a walker that plays the game by stepping.*

---

## 0. The one-paragraph version

The Treyway and the Undercountry become **shot maps**. A shot is an angle and
a power; core simulates the ball across the tile grid in fixed-point integers
until it stops, bouncing off rock, range, sea and water, slowed by whatever it
crosses, and hitting whatever is in the way. Where it stops is a tile: a
place, and you enter it; ground, and the ground rolls for a fight the way a
step does. Between here and there the table has **bumpers** that throw you,
**spikes** that tire you, **pockets** that sink you back to the last town, a
**chute** that carries you the length of the map, and **sand** that stops you
dead. The regional maps — West Bambulon, Kettleworks, every dungeon floor —
keep their steps and their puzzles. The country is the only thing that was
ever big enough to be a table.

---

## 1. Where it comes from

| game | what it does | what to take |
|---|---|---|
| **Yoku's Island Express** | a whole open world traversed as pinball; the ball is a dung beetle; flippers and bumpers *are* the map | the map is the table. Obstacles are geography, not a minigame layered on it. |
| **Pinball Quest** (NES, 1989) | an RPG where you are the ball; towns and dungeons are targets; falling out the bottom is the penalty | **a pocket sends you home.** The drain is death-lite, and it has been an RPG mechanic for thirty-seven years. |
| **Kirby's Dream Course** | golf on a grid; aim, power, a bar to time; landing on an enemy is the encounter | **landing is the event.** Nothing happens in flight except geometry; everything happens where you stop. |
| **Golf Story / Cursed to Golf** | a course as a level, with hazards that cost strokes; a shot is a resource | **a shot costs something**, so the count of them is a number worth printing. |
| **Pool Panic** | a pool cue as your only verb, in a world that is all balls | one verb. The map screen has arrow keys today; on a shot map it has a cue. |
| **the helicopter game** | one button; the walls hurt; distance is the score | spikes. Touching one costs you, and you keep going. |

---

## 2. Decisions

### 2.1 Two maps are tables; every other map is a floor

`TilesData` grows one field: `traversal: "step" | "shot"`, default `step`.
**The Treyway and the Undercountry are `shot`.** Nothing else is, and nothing
else should be: a puzzle floor is a floor because you stand on one tile and
read the one next to it, and a table with nine stakes on it is a floor you
cannot solve. The two country maps are the ones that were ever big enough
and empty enough to be shot across, and they are the maps where walking is
sixteen tiles of scrub between two things that matter.

### 2.2 The physics is core's, in integers, and it is one function

```rust
/// One shot, from a tile, at an angle, with a power. Runs to rest.
pub fn shoot(world: &World, state: &WorldState, shot: Shot, allowed: &Allowances) -> Flight
```

`Shot { angle_deg: u16 /* 0..360 in steps of 5 */, power: u8 /* 1..=10 */ }`.
Seventy-two angles, ten powers, seven hundred and twenty shots from any
tile, which is what makes §2.7 possible. Positions are **sixteenths of a
tile** in `i32`; velocity likewise; one tick moves by velocity and the loop
ends at rest or at `MAX_TICKS`. No `f32` anywhere in it. The shim animates
what core returns and decides nothing — the rule since M8, and a physics
loop in JavaScript would be the first thing in the repository that ran
differently in three engines.

`Flight` is the record: every position at every tick, every contact in
order, where it stopped, and what that tile is. The replay draws it; the
tape prints it; the tests assert on it.

### 2.3 Landing is the event, and flight is geometry

Nothing happens while the ball is moving except contacts with obstacles.
When it stops:

| stopped on | what happens |
|---|---|
| a place — town, gate, event, boss, door, bench | you **enter** it, exactly as if you had stepped onto it: `Step`'s own resolution, called with the landing tile. A hidden place is ground. A shut gate refuses in its own words and you sit on its tile. |
| ground | the tile's `encounter_per_mille` rolls once, **times `LANDING_MULT` (150%)** — a landing is a longer stay than a step, and a shot map has far fewer landings than a walk has steps |
| sand (§4) | you stop there on purpose; sand rolls at its own 200‰ |

Encounters do not roll on tiles crossed. A ball crossing slag is a ball
crossing slag; the ground gets you when you are standing on it.

### 2.4 A shot is free; a contact is not

Walking costs nothing today — fatigue is a fight's. A shot is the same: **no
fatigue per shot**. What tires you is the table: a spike is `Tire(8)`, a
pocket is `Tire(12)` and the walk home, and nothing else on the table costs
health. The count of shots goes on the strip where *walked* goes, and
`shots-taken` is a counter beside `tiles-walked` for anything that wants it.

### 2.5 The pocket is the drain

A ball that stops in a **pocket** is sunk: `Warp` to `last_town`'s tile,
`Tire(12)`, and the tape says *sunk*. It is Pinball Quest's bottom-of-the-
table and it is the only obstacle that moves you off the map. Nothing on a
shot map kills you, and nothing takes your carried experience — a pocket is
a bad shot, not a lost fight.

### 2.6 Walls are walls; edges are walls

Rock, range, sea, water, curd and tide reflect the ball with `RESTITUTION`
(80%). The map's edge reflects the same way, so nothing leaves the map and
nothing can be shot into the sea. **A wall is not a place**: a gate standing
on water (the way under the lake — not on a shot map, but the rule is
general) is reached by stopping on the tile beside it and stepping, which is
what `at_to` arrival already does.

### 2.7 The walker aims by search, and so does the lint

`shot::aim_at(world, state, target) -> Option<Shot>` runs the seven hundred
and twenty shots from where you stand and returns the first that stops on
`target`, or within one tile if `near: true`. Deterministic, integer, and
about a millisecond a shot, so a full search is under a second. It is:

- the walker's move on a shot map — `make play` aims at the next place the
  same way it steps toward it today;
- **the reachability lint**: `every_place_on_a_shot_map_is_reachable_by_shots`
  replaces `every tile of every map derives` for the two shot maps — from the
  start and from every town, every non-hidden place has a chain of shots to
  it of length ≤ 6;
- and, if the human wants it, an *assist* on the screen — a ghost of the
  best shot to the place under the cursor. §10 decides.

### 2.8 Saves are between shots

`WorldState.at` stays a tile. A shot runs to rest inside one call and only
then is anything written, so a save never holds a ball in the air and no
field is added. `traversal` is on the map, not the save. Every M16 save
opens.

---

## 3. The physics, in numbers

| constant | value | why |
|---|---|---|
| `SUB` | 16 per tile | enough that a 5° step in angle changes where a ten-tile shot lands |
| `TICK` | 1 | one tick is one integration step; there is no time, only ticks |
| `POWER_UNIT` | 22 sub/tick per power | power 10 across open plain travels ~9 tiles; power 3 travels ~2 |
| `RESTITUTION` | 80% | four bounces is where a full-power shot dies |
| `STOP_BELOW` | speed < 6 sub/tick | rest |
| `MAX_TICKS` | 400 | a bound, never reached by a legal shot; a test says so |
| friction per tile crossed, by terrain | road 3 · coast 4 · plain 5 · scrub 8 · wood 12 · slag 16 · silt 18 · lakebed 14 · sand **stop** | the table is the terrain table; a road is fast and slag is slow, which is what those words already mean |
| `LANDING_MULT` | 150% | §2.3 |

Friction is subtracted from speed once per tile boundary crossed, not per
tick, so a fast ball and a slow ball lose the same across the same ground.
A bounce keeps the tangential component and reverses the normal one at
`RESTITUTION`; corners resolve on whichever axis was hit first, and a ball
that would enter a wall cell diagonally is pushed out along the shorter
axis. All of it is the same forty lines every 2D tile game has and none of
it is clever.

---

## 4. Obstacles

Five new `PlaceKind`s, drawn on the map like any place and stored in the
same list. They are places because they stand on tiles and the map file is
where things that stand on tiles live; they differ from every existing place
in that **the ball hits them in flight** rather than stopping on them.

| kind | glyph on the strip | contact | stop on it |
|---|---|---|---|
| **bumper** | `◉` | reflects like a wall, and **adds 30% speed** | cannot — it is solid |
| **spike** | `▲` | `Tire(8)`; the ball passes through, and the tape says so | ground |
| **pocket** | `○` | none — the ball goes over it at speed | **sunk** (§2.5) |
| **chute** | `▷` | the ball enters here and **exits at the chute's `to`** with its speed and heading kept | cannot — it is a mouth |
| **sand** | `∷` | none | **stops dead** the moment it enters; the tile is `silt` for every other purpose |

A bumper is Yoku's; a spike is the helicopter game's; a pocket is Pinball
Quest's; a chute is a pinball lane, and the one on the Treyway is the
drove way — the road the survey chain was about, which was for something.
Sand is mini-golf's bunker and the one obstacle you aim *for*.

### 4.1 The Treyway, as a table

Sixteen by sixteen as it stands; nothing moves and no `at_to` changes. The
obstacles go on ground:

```
        0123456789ABCDEF
    0   SSSSSSSSSSSSSSSS
    1   SS::----::--:AAS
    2   S:--AAA---AA--:S
    3   S:-,AAA==,,A--:S       ◉ [8,3] bumper at the pass
    4   S:--,,,=,,,,--:S
    5   S:-,TT==AA=,,-:S       ▲ [8,5] ▲ [9,5] spikes on the ridge · ○ [4,5] pocket in the wood
    6   S:--,T=,,,=,--:S
    7   S::--,,=,,,--::S       ▷ [2,7] chute — the drove way — to [13,2], beside the reach
    8   S:---,-=--,---:S
    9   S:--AA=,,,A,--:S       ◉ [10,9] bumper
   10   S:-,AA=,,,A,,-:S       ∷ [7,10] sand
   11   S:--,,=,,,,,--:S
   12   S:--,==,,,,,--:S       ▲ [11,12] spike
   13   S:-#=,,,,,,,-#:S       start [13,13] · the road west [3,13]
   14   S:::--------:::S       ∷ [8,14] sand, one tile above the tideline
   15   SSSSSSSStSSSSSSS
```

Eight obstacles. The reach edge at [6,1] is a shot up the middle between two
ranges with a bumper at the top of the pass, which is the shot the map is
for; the tide crossing at [8,15] is behind a sand tile, so the way south is
a stopped ball and not a lucky one; the chute is the drove way, and it
carries a ball from the south-west to the north-east in one line, which is
what a drove way is for. The pocket is in the wood, which is where things go
missing.

### 4.2 The Undercountry

Twenty by twenty, and the plan for it (M14 §6, M16) said its layout was the
human's. As a table it wants two things: the three gates in as **sand**
beside them, so an arrival is a stop and not a bounce; and the road between
them and the town as the one fast lane. Six obstacles, placed by the
builder, with the bumpers on the ranges and one pocket somewhere it would be
a shame.

---

## 5. On the screens

The map screen is the only screen that changes, and it changes only on a
shot map.

- **The cue.** Press or click on the ball and drag away from it: an arrow
  drawn back from the ball, its length the power and its direction the
  aim, exactly like every pool game since 8 Ball Pool. Release to shoot.
  The arrow snaps to 5° and to ten lengths, so what is drawn is what core
  will be asked. **Keyboard:** ← → turn by 5°, ↑ ↓ change power, space
  shoots, and the arrow is drawn the same, because a map screen that only
  worked with a mouse would be the first in the game.
- **The flight.** The shim walks the `Flight` at eight ticks a frame,
  drawing the ball and a fading trail; contacts flash their glyph; the
  strip's coordinates update at rest. Reduced-motion skips to rest with
  the trail drawn.
- **The strip.** *walked* becomes *shots* on a shot map; fatigue and danger
  stay. The danger figure is the landing tile's, times `LANDING_MULT`, so
  the number a player sees is the number the roll uses.
- **The tape.** One line a shot, in TONE: *Shot 12. Two off the range, the
  bumper at the pass, and stopped on scrub at [7, 4]. 210‰, and nothing
  came.* A spike: *— and a spike on the ridge (8)*. A pocket: *sunk, and
  back to Kettleworks with 12 off you.*
- **Nothing else moves.** Town, packing, fight, tree, sheet, fork, cards:
  untouched. A gate lands you on the far map's `at_to` tile, and if that map
  is a floor you step from there.

---

## 6. What walking meant, and what it means now

Recon owes a list, and here is its start:

| thing that reads walking | on a shot map |
|---|---|
| `tiles-walked` counter | not bumped; `shots-taken` is bumped beside it. Anything gating on `tiles-walked` (the Low Water mark's counter, per M14) is on a step map and unaffected — **recon confirms nothing on a shot map reads it** |
| `Longhaul { per_second }` | a combat rule about the fight clock; not walking. Untouched |
| `Rule::Wade` and `Allowances` | still consulted — a wader's water is ground, so on a shot map water is *not* a wall for a wader and the ball rolls across it at friction 10. The Treyway has no water tile, so this is a rule with no reader until the Undercountry has a lake |
| `no_homeward` | unchanged; the Treyway allows it |
| `every tile of every map derives` | on shot maps, replaced by §2.7's lint; the step lint keeps running on floors |
| the walker (`playthrough.py`) | on a shot map, calls `aim_at` for the next target and shoots; the transcript prints the shot line |
| `Step` | reused whole at rest; not modified |

---

## 7. Milestones

| # | Milestone | Deliverables | Acceptance | Ships |
|---|---|---|---|---|
| **M17.0** | **The table, in core** | `shot.rs`: `Shot`, `Flight`, `Contact`, `shoot`, the constants of §3; `traversal` on `TilesData`; no map is `shot` yet | `a_shot_is_the_same_in_every_engine` (a fixed shot's `Flight` hashed, checked in wasm); `a_bounce_keeps_the_tangent`; `a_full_power_shot_dies_in_four_bounces`; `no_legal_shot_reaches_max_ticks`; `friction_is_per_tile_not_per_tick`; `nothing_leaves_the_map` | no |
| **M17.1** | **Obstacles** | five `PlaceKind`s; parse-time lints (a chute's `to` is ground; a bumper is not on a wall; a pocket is not on a place); the Treyway drawn as §4.1 and set to `shot` | `a_bumper_adds_thirty`; `a_spike_tires_and_passes`; `a_pocket_sinks_to_the_last_town`; `a_chute_keeps_speed_and_heading`; `sand_stops_dead`; **`every_place_on_a_shot_map_is_reachable_by_shots`** with the chain length written down per place | no |
| **M17.2** | **Landing** | `Game::shoot` — `shoot` then `Step`'s resolution at rest; the landing roll at `LANDING_MULT`; `shots-taken`; the tape lines; hidden places are ground; shut gates refuse from their tile | `landing_on_a_place_enters_it`; `landing_on_ground_rolls_once_at_150`; `a_hidden_place_is_ground_to_a_ball`; `a_shut_gate_refuses_and_you_sit_on_it`; `a_save_between_shots_holds_a_tile_and_nothing_else` | no |
| **M17.3** | **The cue** | the drag, the keyboard, the arrow, the flight animation, the strip, reduced motion | the browser gate below | **yes** |
| **M17.4** | **The walker aims** | `shot::aim_at`; `playthrough.py` shooting on shot maps; the reachability lint reads `aim_at`; the transcript reaches the reach edge and the tide crossing by shots | `aim_at_finds_a_shot_to_every_place_from_the_start`; `aim_at_is_under_a_second`; `make play` crosses the Treyway in ≤ 9 shots and says so | no |
| **M17.5** | **The Undercountry, and the handoff** | the second table (§4.2); tuning from the transcript; `HANDOFF-M17.md`; `CLAUDE.md` with the physics table and *what walking meant* | both tables pass every lint; the shot counts across each in the handoff | **yes** |

**Browser gate, M17.3:**

| check | what only a browser answers |
|---|---|
| `check_the_cue_draws_what_core_is_asked` | drag to an angle and power; the arrow's snapped values equal the `Shot` in the request |
| `check_a_shot_animates_to_where_core_said` | the ball's final drawn tile equals `Flight.rest` |
| `check_the_keyboard_shoots` | arrows and space produce a flight with no pointer |
| `check_a_spike_flashes_and_the_fatigue_moves` | contact glyph flashes; the strip's fatigue rises by eight on that tick |
| `check_sunk_lands_you_in_town` | a shot into the pocket ends on the town screen with the tape line |
| `check_a_floor_still_steps` | West Bambulon's map screen has arrows and no cue |
| `check_reduced_motion_skips_the_flight` | with the media query on, the ball is at rest on the next frame with the trail drawn |

---

## 8. Numbers

| thing | number |
|---|---|
| shot maps | 2 of 24 |
| angles × powers | 72 × 10 = 720 |
| sub-cells per tile | 16 |
| restitution | 80% |
| new `PlaceKind`s | 5 |
| obstacles on the Treyway | 8; on the Undercountry 6 |
| fatigue: a shot / a spike / a pocket | 0 / 8 / 12 |
| landing roll | 150% of the tile |
| max chain of shots to any place, by lint | 6 |
| new save fields | **0** |
| `f32` in the physics | **0** |

---

## 9. What this does not do, on purpose

- **No fights in flight.** Landing is the event (§2.3). A creature that could
  stop a ball mid-table would make the shot a guess.
- **No flippers.** A flipper is a second verb and a timing test; the game
  has one verb per screen and the cue is the map screen's.
- **No physics on floors.** §2.1.
- **No redraw of the Treyway.** Sixteen by sixteen with sub-cells is enough
  of a table to have eight things on it; a thirty-two by thirty-two Treyway
  would be a better table and a different map, and every `at_to` into it
  would move. §10.

---

## 10. The human's

1. **The two maps.** The Treyway and the Undercountry, and no others. If the
   Low Water or the Sands should be a table too, they are step maps with
   puzzle-shaped events on them and the plan's answer is no; but the flag is
   per map and the answer is a one-line change.
2. **The assist.** §2.7 can draw the best shot to the place under the cursor
   as a ghost. It makes the table a menu; it also makes it fair. The plan
   ships without it and leaves `aim_at` public so it is one line to add.
3. **Redrawing the Treyway at 32×32.** A better table; every gate into it
   moves. Not this block, unless it is.
4. **The pocket's cost.** Twelve fatigue and the walk home. The alternative
   is the walk home alone, which is Pinball Quest's; twelve is the plan's,
   so that a pocket is worse than a spike and a player aims around it.
5. **Encounters on crossing.** The plan says landing only. If the table feels
   too safe, the alternative is a roll per *slag* tile crossed at a tenth of
   its rate; it is a one-line change in `shoot`'s contact loop and a test.
