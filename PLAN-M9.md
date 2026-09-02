# PLAN-M9.md — what a creature leaves behind

Written 2026-09-02, against `ce46848`. `PLAN.md` remains the plan of record for
M0–M7 and `PLAN-M8.md` for M8; this is the plan for the next block and follows
their conventions.

**Five milestones, each deployable on its own.** M9.0 is the one that changes a
rule everything else leans on, M9.4 is where the block ends. Every one of them
ships behind the same gate: `make test`, `make test-ui` in three browsers, and
`make play` read by a person.

---

## 0. The ask, and what recon changed about it

> *"each of the enemies currently available in the first area (which the harder
> ones like drabley henpeck should be removed) should have a unique assembled
> item related to them, that they have a small percentage chance to drop a gear
> piece from; these gear pieces have set bonuses, which apply when the assembled
> item is recombined in your inventory."*

Four things were measured before this was written, and two of them moved the
plan.

**Lord Drabley Henpeck is not in the first area.** He is `The Hollow King`,
themed, and he lives in the region `west-bambulon` — rows 1 to 3, the top of the
map. `World::draw_enemy` picks strictly from the region's own pool, so a
character standing in the pit can only ever meet the pit's three. The pit is:

| canonical | themed | rating | health |
|---|---|---|---|
| Cave Rat | A. Rat | 16 | 55 |
| Bog Toad | Bengulon Jungle Toad | 80 | 110 |
| Bone Archer | Wallspider Swarm | 95 | 120 |

So "the harder ones should be removed" is **not** about the pit's roster, which
is already gentle. It is about the map: *nothing stops a level-one character
walking fifteen tiles north into a region of two-thousand-rated creatures.* The
gradient is a gradient and not a gate. **Confirmed with the human:** the pit
keeps all three, each gets a set, and the north gets gated as its own
deliverable. That is M9.3.

**"A depth of 1" has an exact and cheap reading**, and it was measured rather
than guessed: a water tile is walkable when it touches land. On this map that
opens **14 of the lake's 28 tiles** — the rim — and leaves the middle 14 shut.

```
 8 ^.==...........,==.^
 9 ^==....@@@@....,,==^   row 9 becomes crossable end to end
10 ^=....@####@....,,=^
11 ^=...@######@....,=^   @ opened by the set  (14)
12 ^==...@####@....,==^   # still shut         (14)
13 ^,==...@@@@....,==,^
```

No new terrain, no repaint, and the lake stops being a wall at its edge while
staying one through its middle. **Confirmed with the human.** And the payoff
sits two regions north of where the set is earned, which is the right shape for
a traversal reward.

**The engine already has most of this, and it is not where you would look.**

- **A set bonus is an assembly bonus.** `AssemblyBonus` carries a `Stats` lump
  *and* `&[Trigger]` — the whole trigger vocabulary — and it only pays while the
  item is assembled. Anything a set does *in a fight* costs no new combat code.
- **The two bonuses the human named are not fight rules.** "Instantly defeat any
  A. Rat" happens before a fight starts; "walk in water" happens on a step.
  Neither is expressible as a `Trigger`, and trying would be the mistake this
  project has made twice — a rule decided in the wrong place.
- **`Effect::Grants { rule: Rule }` is the door they go through**, built in M8.3
  and currently granted only by the skill tree. M9.0 is widening that door.

**Adding to `CATALOG` changes the save fingerprint.** This block adds eight
components. Every save written before it is refused, with a sentence naming both
catalogues. That is the design, and it goes in the commit — see `CLAUDE.md`,
*Rules*.

---

## 1. What is being asked for, in one list

| # | Ask | Where it lands |
|---|---|---|
| A | A rule an **assembled item** grants, not just the tree | M9.0 |
| B | An item with a **fixed name**, not a generated one | M9.0 |
| C | A **small chance** to drop a piece, off the creature that owns it | M9.1 |
| D | Three sets, one a pit creature, named and themed | M9.2 |
| E | `Rule::Rout` — the Rat King's Mandate | M9.2 |
| F | `Rule::Wade` — the toad set, depth 1 | M9.2 |
| G | Nothing stops a level-one walking into Henpeck | M9.3 |
| H | Play it, triage it, write it down, ship it | M9.4 |

---

## 2. Ordering, and why this order

- **The door before the things that go through it.** M9.0 is the only milestone
  with no content in it at all: a rule can come from an assembled item, and an
  item can have a name somebody wrote. Both are needed by every set, and
  building them once is the difference between writing three sets and writing
  them three times.
- **The drop before the drops.** M9.1 makes a creature able to leave something
  behind at a rate. M9.2 fills that in with actual gear. Split because the
  *rate* is the thing that will be retuned and the *gear* is the thing that will
  be added to, and they should not be one commit.
- **The gate last of the features.** M9.3 is independent of the sets and could
  ship first; it is placed after them because the sets change what a character
  can survive, and a gate tuned before them would be tuned against the wrong
  game.
- **M9.4 is not a feature.** M8.8 found two blockers that 482 tests were green
  through. That is now a standing milestone and not a one-off.

---

## M9.0 — A rule an item grants, and an item with a name

**Theme:** the two seams every set needs, and no content.

### Deliverables

1. **`Character::rules()` — every rule this character has.** The tree's
   (`skills::SkillsData::rules_from`) plus every rule granted by an item that is
   *currently assembled*. One list, read fresh every time, for the reason a
   node's effect is read fresh: a bought node's effect is not state, and neither
   is a seated item's.
   - It goes on `Character` and not on `Loadout`, for the same reason enchs did:
     a loadout that knew about granted rules would be a loadout that knew about
     a skill tree.
2. **`AssemblyBonus` gains `grants: &'static [Rule]`.** Which means `Rule` moves
   out of `skills.rs` into its own module — `crate::rule` — because it is no
   longer the tree's. `skills::Effect::Grants` re-exports it and nothing about
   the JSON changes.
   - **`Rule` stays an enum and the match stays exhaustive.** That is the whole
     guard, and this milestone doubles the number of places that consume one.
3. **Three consumers, and each is in the place that can honestly answer it.**

   | rule | consumed by | why there |
   |---|---|---|
   | the existing combat rules | `Held.rules` → `Combatant` | already built, M8.3 |
   | `Rule::Rout` | the encounter, before `fight::run` | there is no fight to put it in |
   | `Rule::Wade` | `world::step`'s passability | a step is where a wall is refused |

   `world::step` takes `&mut WorldState` and **not** the character, which is
   deliberate and stays that way — *a map does not know about bags*. So it grows
   one parameter: `allowed: &Allowances`, a small plain struct the caller fills
   from `Character::rules()`. The shim already answers a gate's key this way and
   a door's; this is the same division, made once and named.

4. **A set has the name somebody wrote.** `name_item` generates
   `[Qualifier] [Base] of the [Suffix]` from a hash, which is right for the
   five hundred and thirty-six and wrong for exactly the items this block adds.
   `PieceDef::assembly_bonus` gains `names: Option<&'static str>` — when every
   piece of an assembled item agrees on one, that is the item's name.
   - **Agreement, not one piece deciding.** A set is its pieces; a single piece
     naming the item would let two of a set plus a stranger call itself the
     Mandate.
5. **`Rule::describe()`, and the sheet prints it.** Unthemed, with the number in
   it, TONE 13a — the same rule `Node::line` follows. A player holding the
   Mandate must be able to read what it does somewhere other than by meeting a
   rat.

### Tests

- `a_rule_from_an_item_reaches_the_fight`, and does not without the item.
- `an_unassembled_set_grants_nothing` — the whole point of "recombined in your
  inventory".
- `every_rule_is_described` grows to cover the new variants, and its
  zero-value forms are refused. It already exists.
- `a_named_set_needs_every_piece_to_agree`.
- `the_golden_fixture_is_unmoved` — nothing seated, nothing different.

### Deploy point

After 5. Nothing a player can see changes, which is the point: this is the
milestone that makes the next one small.

### Risks

- **`world::step`'s signature is on twelve call sites**, across the shim and
  four test files. Counted, not guessed. Change it once, in one commit, with
  nothing else in it.
- **`Rule` moving module is a wide diff and no behaviour.** Same commit as the
  move, nothing else in it, and `skills.rs` keeps a `pub use`.

---

## M9.1 — A creature leaves something behind

**Theme:** the roll, the rate, and where the stream is spent.

### Deliverables

1. **`data/drops.json`** — creature, component, per-mille. Content, and content
   is not state, so it is a data file and not a table in `piece.rs`.
   - Keyed by **canonical** creature name, like an errand's `Slay` goal, because
     that is what the engine matches on.
2. **The roll is in `fight::settle`, on a victory, off `game.rng`.**
   - **Integer per-mille**, like every other roll in this game. Float rounding
     is the one thing that breaks a seeded walk silently.
   - **`fight.rs` does not touch the rng today.** This is its first use, and it
     therefore moves the stream on every won fight. `a_seeded_walk_replays`
     compares two walks with one seed rather than pinning literals, so it holds
     — but say so in the commit, and re-read it before assuming.
   - Rolled **once per drop entry**, after the bounty and after
     `quest::on_victory`, so an errand's tally is never displaced by a drop.
3. **The receipt says so.** In the same voice the errand drop uses — *"It was
   carrying …"* is the boss's line and *"Took a …"* is the errand's; a rare drop
   wants its own and it should read like luck.
4. **A drop is refused when you already have it.** A set is three specific
   pieces, not three of a kind, and a bag filling with Cheese Touches is the
   litter `quest::on_victory` already refuses for the same reason.
5. **The rate is set by a test, not by taste.** `XP_DIVISOR` is 5 because a test
   says so; `PER_FIGHT` is 4 because a test walks twelve fights. This wants the
   same: `a_set_is_a_few_hours_and_not_a_lifetime` walks the pit and refuses a
   per-mille that makes three pieces take fewer than ~15 or more than ~120 wins.

### Tests

- `a_drop_only_falls_for_the_creature_that_owns_it`.
- `nothing_drops_twice`.
- `every_drop_names_a_creature_some_region_holds` — the same lint the errands
  got in M8.1, and for the same reason: a drop off a creature that is nowhere is
  content nobody can reach.
- `a_set_is_a_few_hours_and_not_a_lifetime`, walked.
- A dropped piece survives a round trip. It is a registry entry like any other,
  so this should be free — assert it rather than assume it.

### Deploy point

After 5. At this point the pieces drop and do nothing, which is worth shipping:
the rate is the thing that wants a player's opinion.

### Risks

- **The rate is the whole feel of the milestone** and cannot be judged from a
  test. Ship it, play it, and expect to move it in M9.4.

---

## M9.2 — Three sets

**Theme:** the content, and the two rules that are the point of it.

### Deliverables

1. **The Rat King's Mandate**, off the A. Rat. Three pieces, the human's names:
   `Cheese Touch` (gloves mold), `Cheesy Fingers` (gloves material),
   `Cheese Finder` (ring). Assembled: `Rule::Rout { creature: "Cave Rat" }`.
   - **Rout is not a combat rule and must not be one.** Meeting one is a
     victory that never becomes a fight: the encounter resolves on the step,
     pays what a win pays, and says why. Putting it in `combat` would mean a
     fight that is decided before its first tick, which is a fight the replay
     has to draw.
   - **It still costs a fight's fatigue?** No. Nothing was fought. Say so in
     the receipt, because a player will check.
2. **The Toad Hide set**, off the Bengulon Jungle Toad. Two pieces: `Toad Hide`
   (chest layer) and `Toad Frame` (chest base). Assembled: `Rule::Wade`.
   - The refusal line the game already has is
     *"you would have to swim, and you are wearing a frame"* — which is a
     sentence about the frame. A toad's frame answering it is the whole joke and
     it is already written.
3. **A third set**, off the Wallspider Swarm. The human has not named it. It
   wants a rule that is *not* a third new kind — the cheapest good one is a
   combat bonus through `AssemblyBonus.triggers`, which costs nothing, and the
   set then teaches that not every set changes the world.
4. **Every piece is themed in the same change**, or
   `the_turtle_theme_covers_the_catalogue` fails, and it is right to.
5. **Every piece is off every shelf**, in `EVENT_ONLY`, for the reason an
   errand's reward is: *a reward you could have bought makes the errand a slow
   way to shop*, and a drop you could buy is worse than that.

### Tests

- `the_mandate_routs_a_rat_and_nothing_else` — and a Bog Toad is still a fight.
- `the_toad_set_opens_the_rim_and_not_the_middle`, asserted against the map:
  14 tiles, and the fourteen named in §0 still shut.
- `a_set_that_is_not_assembled_grants_nothing`, for both.
- `every_set_piece_is_off_every_shelf`.
- `the_catalogue_fingerprint_moved` — a save from before this block is refused
  with a sentence naming both catalogues. State it, do not discover it.
- `check_a_set_reads` (browser): the three pieces in the bag, seated, and the
  card naming the item and its rule.
- `check_the_toad_walks_on_water` (browser): planted, one step onto row 9.

### Deploy point

After 5.

### Risks

- **`Rule::Wade` changes what "reachable" means**, and two tests assert the map
  is fully reachable and that every place stands on walkable ground. Neither
  should move — wading only *adds* — but read them rather than assuming.
- **A rout is an encounter that pays without a fight**, which is a shape this
  game has not had. It is not §C.1 — that divergence is about a *loss* paying —
  but it is the same family of question, and `a_lose_win_cycle_is_not_a_gold_farm`
  is the test that already guards the family. Write its sibling, and decide
  deliberately rather than by omission: an A. Rat pays six Fnorp and three
  experience, which is not a farm, but a routed rat is also a fight that costs
  nothing at all.

---

## M9.3 — The north is a decision, not a slope

**Theme:** the half of the ask that is about the map.

### The thing to settle first

Nothing stops a level-one character walking into Lord Drabley Henpeck. There are
three ways to fix that and only one of them is in keeping:

- **Rejected: scale the pool to the player.** Danger is *measured, not typed* —
  a region's danger is the mean rating of what lives there, and
  `no_data_file_types_a_danger_number` fails the build on a number in a data
  file. A pool that reads the character's level would be tuning the ruler.
- **Rejected: a wall.** The map is a place. Rows 4 to 7 are open ground and a
  cliff drawn across them to solve a pacing problem would read as one.
- **Recommended: the map already has the mechanism.** `PlaceKind::Gate` is a
  way onto another map; what this wants is its sibling — a **crossing** on this
  one, a place that refuses a step until something is true. `hidden_until`
  proved conditional places work and `needs` proved a bag-shaped condition
  belongs in the shim. A crossing is those two on a tile you walk *through*.

### Deliverables

1. **`PlaceKind::Crossing`** — a tile you may pass only when its condition
   holds. Same shim division as a gate: the map says there is one, the caller
   says whether you get through.
2. **Two of them**, on the road north out of the Slag Flats and out of the
   Burnwarp Shallows, wanting a level rather than a component — the first thing
   in this game that is gated on what you *are* rather than on what you carry.
3. **They say why**, in the world's words, and the refusal names the number.
   TONE rule 12: *"Forty Fnorp, and you have not got it."*
4. **The map draws them.** Not a diamond — that is a gate — and not an arch,
   which is the door. A crossing wants its own mark and it is the fifth on this
   map; check it against `look.rs`'s three channels before drawing it.

### Tests

- `a_crossing_refuses_and_says_why`.
- `every_region_is_reachable_at_the_level_its_crossing_asks_for` — the map's
  reachability test, made conditional rather than absolute.
- `no_crossing_shuts_a_player_out_of_a_town_they_have_used` — `World::repair`
  puts a defeated player at their last town, and a crossing between them and it
  would strand somebody.
- `check_the_north_is_shut` (browser), planted at level one.

### Deploy point

After 3. The mark can follow.

### Risks

- **A crossing can strand.** The walk home after a defeat crosses maps already;
  it must be allowed to cross a crossing. Decide that explicitly — the
  recommendation is that going *home* is never refused.

---

## M9.4 — Play it, triage it, write it down, ship it

**Theme:** the milestone that is not a feature, and it is standing now.

M8.8 found two blockers with 482 tests green: Auto-pack seated the starting kit
for the whole game, and the class fork opened underneath the town. Neither was
findable by reading and neither was covered by a check. That is the argument.

### Deliverables

1. **A real playthrough**, `make play`, new game to the ending, read by a
   person. The transcript goes in `testing/transcripts/`.
2. **A triage table** — Blocker / Wrong / Rough / Later — in the commit, with
   the Later rows written into `PLAN.md` §6 so a deferral is visible rather than
   forgotten.
3. **The drop rate judged by a person**, which is the thing M9.1 shipped
   unfinished on purpose.
4. **`CLAUDE.md` current**, every number in the closing table re-measured.
5. **Push, deploy, and verify the deploy arrived** — the build stamp live and a
   smoke pass over the deployed site. Two deploys have failed silently here.

### Tests

The suite is the test. What this adds is somebody playing it. **If the
playthrough finds something no check would have caught, that is a missing
check**, and writing it is part of the fix.

### Deploy point

The end of it.

---

## 3. Cross-cutting rules this work must not break

- `crates/core` stays graphics-free and wasm-free.
- `crates/wasm` decides nothing. The drop roll, the rout, the wade and the
  crossing are all core's; the shim asks and renders.
- **A map does not know about bags.** `world::step` takes what it is allowed to
  do, not the character that allows it.
- **Derived, never typed.** Every rule's description comes off the rule.
- **Read, never computed.** A rout that pays has a receipt, and the receipt says
  what was paid rather than the page working it out.
- A new field on `Character` or `Game` is a compile error until the save carries
  it. Do not add `..` to a destructure.
- **A new component needs a themed name in the same change**, and adding to
  `CATALOG` moves the save fingerprint. Say so in the commit.
- Every browser check must be **negative-tested**: break the thing it guards and
  watch it fail. Three checks have shipped vacuous.
- Every check that opens a screen closes it on every path out.

## 4. Numbers to watch

| | now | after |
|---|---|---|
| Tests | 483 | expect ~510 |
| Catalogue | 536 | 544 — **the fingerprint moves** |
| `Rule` kinds | 5 | 8 — rout, wade, and the third set's |
| `PlaceKind` | 5 | 6 — a crossing is not a gate |
| Data files | 13 | 14 — `drops.json` |
| Save format | v1 | v1, but old saves are refused by the stamp |
| Regions reachable at level 1 | 5 | 2 |

## 5. Open questions for the human

1. **The Wallspider Swarm's set has no name and no bonus.** M9.2 proposes a
   combat bonus through the machinery that already exists, so that not every
   set rewrites the world. Names welcome.
2. **Does a rout pay?** A win with no fight paying a full bounty is a farm, and
   `reward.rs` §C.1 is this project's one deliberate divergence about exactly
   that. Planned: it pays, because six Fnorp is not a farm and because being the
   Rat King should be worth something. Say if not.
3. **Does a rout cost fatigue?** Planned: no. Nothing was fought.
4. **What level do the crossings ask for?** Planned 5 and 9, which is roughly
   where the existing pools stop being survivable. It is one line of data.
5. **Is `~1 in 20` the right drop rate?** Planned as a starting point and
   explicitly deferred to M9.4, when somebody has played it.
6. **What is past the door.** Still open, still yours, and nothing in this block
   assumes an answer.
