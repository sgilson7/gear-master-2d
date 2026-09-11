# HANDOFF-M16.md — the Eleven Reefs, and two classes GM2D wrote

*`PLAN-M16.md` is the frame. This is the block's own record: what each milestone
found, and why the divergences are what they are. `SECOND-ORDER-M16.md` is the
notebook — forty-one rows, written when noticed — and the block's last
milestone is that notebook executed.*

*`CLAUDE.md` is the current account of the repo. Read that first if you have
never seen this.*

---

## 0. State in one paragraph

**All seven milestones are done and the block is live.** The suite is **950
passing** and the browser gate walks **78 `ok:` lines**, six of them M16's and
every one negative-tested. M16.0 through M16.5 are deployed at `e3a2193f`, and
the pair agrees — `index.html` asks for `app.js?v=e3a2193f` and that `app.js`
carries `BUILD='e3a2193f'`. The full gate was walked against the deployed page
with `GM2D_ORIGIN` and came back 78 for 78.

**Three new maps, two new classes, eleven new experts, and no new components.**
The catalogue is still 568. Two save fields are added — `Character::warm_stacks`
and nothing else — both `#[serde(default)]`, so **no seam**: every save that
opened on M15 opens on this.

**The block's largest finding is not in the plan.** `PLAN-M16.md` §4.2 hangs two
doors on `AssembledOfRarity`, and **no board this game hands a player reaches
Rare**: the level-45 run tops out at an item rating of 50 against a `RARE_AT` of
90. See §2.4.

---

## 1. The milestones

| # | Milestone | Status |
|---|---|---|
| M16.0 | **The fixture and the primitives** — the run as a save, `common::from_save`, `Character::item_partition`, `quick`, `MonsterSpec.enchs`, `stats::LANE_CAP` | ✅ 919 |
| M16.1 | **The gate and the Flat Below** — nine stakes, three bands, six pockets, a sheet, and a compass that lies | ✅ 927 |
| M16.2 | **The Assay** — three doors, and the second takes what the third needs | ✅ 933 |
| M16.3 | **The Needle Room and the Tenth Surveyor** — four sinkholes, four levers, a plate, and a creature wearing the run | ✅ 941 · **deploy point** |
| M16.4 | **Two more base classes** — the Kettle-Stoker and the Whisperling, `OFFERED` at seven | ✅ 950 |
| M16.5 | **Eleven experts** — `EXPERTS` at twenty-one, three new rules, sixty-six new nodes | ✅ 950 · **deploy point** |
| M16.6 | **The gate, the walk, the handoff** — six browser checks, all six negative-tested | ✅ 950 |

**M16.4 and M16.5 are one commit**, and `every_pair_of_offered_classes_reaches
_an_expert` is why: the moment `OFFERED` is seven, eleven pairs reach nobody,
and there is no green tree between the roster growing and the table catching up.

---

## 2. The divergences

Forty-one notebook rows; these are the ones that changed what shipped.

### 2.1 The save did not arrive, so the fixture is reconstructed

`PROMPT-M16.md` says to place `testing/saves/the-run-20260910.json` before
starting and to treat it as the human's own file. **It was not in the tree.**
`PLAN-M16.md` §5.1 transcribes the board in full, so the fixture is rebuilt from
that transcription by `crates/lab/src/mkrun.rs` — checked in, so the file is
something somebody can regenerate and diff.

Rebuilding it taught the block two things the plan could not have.
`Loadout::locks` is state and not geometry, so §5.1 says where the pieces sit
and nothing about when each item was fixed; the order is therefore **searched
for**, per grid, over every subset of the lock points. And `MonsterSpec.items`
is a chunk list, so the gear array has to be in **item order**, which a board's
placement order is not — the run's first helmet item is board entries 0, 3 and
5. `Character::item_partition` returns the pair.

Three of the run's weapon pieces — an Accessory, an Ink and an Alignment in one
row with no core — **assemble under no discipline at all**. Eleven items out of
twelve groups.

### 2.2 §5.3's curse-cap numbers did not reproduce, and the mind lane was worse

The plan says two creatures stand past the curse cap, at 114 and 120. Measured,
summing base and gear the way `Combatant` does: the Ninth Surveyor is at **88**,
Marbulon at **102**, and **twenty-three** creatures are past, up to Nine of Ashes
at 233. Nobody had asked the *mind* lane the same question: **thirty-one** are at
or past a hundred, every deep boss among them.

At a hundred, `landing_ms` and `mind_damage_after_resist` both return **zero** —
so M16.4's Whisperer and four of M16.5's experts would have dealt exactly
nothing at the bottom of every dungeon in the game.

Fixed in the engine, not by re-dressing: **`stats::LANE_CAP` is 95**, which is
`RESIST_CAP`'s own argument in `RESIST_CAP`'s own words. Nothing under 95 moved.
`no_creature_is_immune_to_a_curse_or_a_whisper` **calls rather than declares**,
and named 124 shut creature-lanes when broken.

### 2.3 The blind ceilings are in a different unit from the plan's

| floor | `solvable_blind` | the plan | why |
|---|---|---|---|
| the Flat Below | **36** | 9 | nine is §4.1's count of *pulls*; this counts **card reads**, which is M14's unit and the unit the Cairnfield's 45 is in |
| the Assay | **3** | 3 | exact |
| the Needle Room | **7** | 8 | eight is *drops and pulls*; a sinkhole raises no flag and is not a card read |

Thirty-six rather than eighteen because the adversary opens each band *early*
and leaves its two wrong neighbours live while the next band's three arrive.

### 2.4 No board this game builds reaches Rare

The Assay's doors are footprints and money, not `AssembledOfRarity`. Measured:

| board | best item | rarity |
|---|---|---|
| the run, as the human left it | 50 | Common |
| the run, Auto-packed from everything it owns | 50 | Common |
| `geared_from` — both shelves, every errand | 135 | **Epic** |

`RARE_AT` is 90, `EPIC_AT` 130, `LEGENDARY_AT` 170, and **562 of the 568
components rate Common on their own.** So door three at legendary would have been
a wall with a sentence on it, and door one at epic would have refused the
**level-45 run** while passing a level-ten shopper. `epic` is kept on door one as
the *shortcut* — `geared_from` reaches it — and `no_board_a_player_can_build
_reaches_rare` is the lint.

**The wider question is the human's**, and it is notebook row 14: a level-45
caster board is Common and a level-ten shopper's is Epic, because `item_rating`
prices pieces and cadence and an Ink is a multiplier that rates nothing on its
own. Nothing in the block turns on it.

### 2.5 `Outcome::Warp` works as a lock; the solver could not see it

`PROMPT-M16.md` asks for this to be reported first if a warp cannot land you in a
walled alcove. **It can.** What could not see it was `puzzle::reachable`, which
floods terrain — so the Needle Room measured `Stuck`. It follows a warp now, on
an unconditional choice only and only onto the map it is already on. The plan's
fallback (a `needs` gate keyed to each sinkhole's flag) is not needed.

What *did* nearly ship is a lever nobody could ever pull: `warp_to` resolves no
place on the landing tile, so the first draft's one-tile alcove was a card you
land on and cannot open. **An alcove is two tiles** — the one you land on and the
one the lever is on — so a sinkhole behaves like every other warp in the game.
Fixed in the map; `warp_to` is untouched.

### 2.6 The sinkholes do not repeat

§4.3 says they should. `a_repeating_event_may_never_pay` refuses a repeating
warp for a reason that is still right. They do not need to: both ungated levers
sit behind a true sinkhole, each lever drains the way out of its own alcove, so
each hole is needed exactly once and no order strands anybody.

### 2.7 The DPS bracket is not measurable as §5.2 writes it

§5.2 asks for a win rate between 55% and 70% *over a loop of seeds*, and
**combat has no RNG** — a loop over seeds counts the same fight every time. What
varies between two players meeting her is the *board*, so the bracket is over
boards. Against `common::geared_from`:

```
  The Ninth Surveyor             122.1/s   Victory
  What Marbulon Faced Away From  138.7/s   Victory
  The Tenth Surveyor             221.2/s   Victory   15,000 health, 64 strength
  Gilt                           428.3/s   Defeat
  Nine of Ashes                  531.0/s   Defeat
```

Her first draft was 236 strength and dealt **807.9/s**, which killed that board
in three seconds. **Health barely moves a fight at this depth and strength is the
whole dial**, which is the Kettleworks finding a third time.

**And the run loses to her**, which is in register: the reconstruction is a
caster board with 974 health against a shopper's 1,942. *"An almost finished run
that is very powerful"* is not what §5.1 transcribes.

### 2.8 A base class had no knobs, and widening it opened a collision

`Effect::Tunes` was checked against `ExpertPower::knobs` — *only an expert class
has knobs* — and both plan trees are built on tuning one. `ClassPower::knobs`
and `::tune` are the widening; `Character::class_defs` tunes a base power the
way it tunes an expert's, which is **M13.6's thirty-eight dead nodes one level
down**.

Then `per_stack` turned out to be the Stoker's **and** the Patented Funnel's, so
a character who was both would have had each tree's tuning applied to the
other's power. `tunings_for(class, taken)` is scoped to the tree.

And `ExpertPower::step` read the knob's *name* where it needed to read the
power: a `_ms` knob is printed in whole seconds by two experts and to a **tenth**
by the four furnace ones.

### 2.9 The tree could not grant a pool, and `Stat { mind }` was dead

`Effect::Stat` read five fields and `StartWith` two. `stat { nature: 20 }` would
have parsed, cost a point and changed nothing — the *eight skill nodes* failure,
caught by `every_effect_key_is_one_the_engine_actually_reads` before it shipped.

Worse, `Effect::Stat { mind }` was dead **as added**: mind damage is read off
`ItemProfile::stats.mind` and a character-level `Stats.mind` reaches no item. It
cannot be read off `player_stats` either, which sums every item's stats and
would pay the board's own mind damage once per item. It travels through
`Held::mind` now — the one door for *what the character contributes, once,
beside the item*.

### 2.10 An expert's power is self-contained, and six of these would not have been

A Fired Funnel's promise is *every stack the furnace buys is also mana*, and a
furnace is the Stoker's — so without one the whole tree read dead.
`light_the_furnace` gives all six their own. Giving the *fixture* both parents
instead was tried and buried five of the original ten under two parent powers'
worth of noise.

`every_point_in_an_expert_tree_buys_something` then found five more things, all
real: three tuning defaults chosen so the knob never bound, a purse sweep that
could not see a two-second move, and a board that deletes a maximum in one blow
and therefore cannot watch a threshold move.

---

## 3. What is open, and it is the human's

1. **`PLAN-M16.md` §9.1, her name.** Shipped as *The Tenth Surveyor*, themed
   **THE ONE WHO STAYED**, which is the sheet's own line answered.
2. **§9.2, the instrument frame.** She keeps it off: a creature has no survey,
   and §5.1 does not transcribe one either.
3. **§9.3, nine thousand.** Kept, and it is now the price at *two* doors — the
   cheap way through the Assay is one footprint and nine thousand, the way the
   board never moves is eighteen.
4. **§9.4, re-dressing the M14 bosses.** Not done, and it should not be: the
   recon found twenty-three creatures rather than two and neither of the plan's
   numbers, and `LANE_CAP` fixes all twenty-three without moving a creature.
5. **§9.5, the two canonical names.** *Stoker* and *Whisperer*, themed
   **Kettle-Stoker** and **Whisperling** — the second is the answer to what a
   Whisperling is, which Marbulon's chain has asked since M12 and nobody had
   said.
6. **§9.6, whether an unmaking pays drops.** Yes. `settle` is untouched:
   `is_down` has fired at `max_health <= 0` since the fork.
7. **Notebook row 14, and it is the real one**: `Rarity` describes creature
   boards and not player boards. Nothing turns on it today.

---

## 4. Numbers

| | |
|---|---|
| maps | **25**, up from 22 |
| floors with a puzzle | **9**, with a boss **3** |
| blind ceilings | 36 · 3 · 7 |
| base classes | **7**, up from 5 |
| expert classes | **21** = C(7,2), up from 10 |
| skill trees | **29**, up from 16 |
| new skill nodes | 18 base + 66 expert = **84** |
| `ClassPower` variants | 34, up from 32 |
| `ExpertPower` variants | **21**, up from 10 |
| `Rule` kinds | **16**, up from 13 |
| new components | **0** — the catalogue is still 568 |
| new save fields | **1** (`warm_stacks`), defaulted; no seam |
| creatures | **61**, and the newest is wearing a real player's board |
| browser gate | **78 `ok:` lines**, six of them M16's |
| the suite | **950 passing**, up from 941 |
