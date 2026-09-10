# PLAN-M15 — a fight you have already had

*Three asks from the human, one of them a screen and two of them curves, plus
the recon each needs. Written against the M14 tree as deployed at `4d065d1d`:
`fight.rs` (`rout`, `run`, `settle`, `pay_a_win`), `progression.rs`
(`XP_TO_NEXT`, `MAX_LEVEL`, `XP_DIVISOR`), `shop.rs` (`Paper`, `StockGate`),
twenty-one maps in `data/maps/`.*

---

## 0. The one-paragraph version

**Instant Battle.** Beat a creature five times and you may mark it. From then on
meeting it settles where it stands: the fight is simulated in full, banked in
full, and **never drawn** — the result arrives on the strip and in the history
like everything else the game says. It still costs you four percent. A menu
lists what you have beaten five times and lets you turn each one on and off.
**The experience curve is flattened**: quadratic to level 50 and exponential
after it, against today's `1.35^(L−1)` from level one, which is why level twenty
costs two hundred and nineteen thousand. **And the second class comes off the
tree**: Spike's second paper is gated by its price and nothing else.

---

## 1. Instant Battle

### 1.1 It is the rout, and the rout is already written

`fight::rout` is the whole precedent and it is worth reading before anything
else is typed. It is the one path in this game where **an encounter is settled
without a screen**, it has two users already — the Rat King's Mandate and the
survey golem — and its own comment says why they are one mechanism:

> **Two ways an encounter is settled without a fight**, and they are one
> mechanism because they are one thing: something happened that meant the fight
> did not.

An instant battle is the **third**, and it is a different thing in exactly one
respect: *the fight did happen.* So it does not belong in `rout` — a rout pays
the plain bounty, skips the speed bonus and **costs no tiredness**, and every
one of those three is wrong here.

### 1.2 What it actually is: `run` then `settle`, with nothing drawn

The two halves already exist and the shim already calls both:

```rust
fight::run(game, difficulty) -> Option<CombatLog>   // simulate from game.encounter
fight::settle(game, &log, difficulty) -> Option<Settlement>
```

`settle` pays the bounty with the class's arm on it, rolls the drops, counts the
errand tally, banks the streak — and **tires you four percent**, at
`fight.rs:343`. So *"you should still lose tiredness"* costs nothing to
implement: it is what going through `settle` already does. **Do not reimplement
the settlement.** A second answer to *what a win pays* is the thing this project
has paid for six times.

What is new is only that nothing opens. In `crates/wasm/src/lib.rs`, `walk`
already reports three shapes and the page already branches on them:

```js
if (r.routed) log(r.routed.receipt.join(' '));      // settled, nothing drawn
else if (r.encounter) openFight();                  // drawn
```

Add a fourth: `instant`, carrying the same receipt shape, logged the same way.
**`log()` is the one door everything the game says goes through** — it lands on
`#tape` and is kept in `#history`, which is precisely what the ask describes.

### 1.3 Five wins, and the count is already in the save

`fight::pay_a_win(game, creature, receipt)` is the one place a win is paid,
called by both `settle` and `rout`. `WorldState::bump` writes a counter, and
`WorldState::counters` is `#[serde(default)]` and already round-trips.

So **the tally is one line in `pay_a_win`** and no new save field:

```rust
game.world.bump(&format!("beat:{creature}"));
```

`Game::beaten(name) -> u32` reads it. **Derived, never banked** — the eligible
list is *"every creature whose counter is at least five"*, worked out fresh, not
a second list kept beside the first.

A rout counts. It is a win, it goes through `pay_a_win`, and a player who routs
a rat five times has met the rat five times.

### 1.4 What *is* new state, and it is one field

```rust
/// Creatures set to settle where they stand, by canonical name.
#[serde(default)]
pub instant: Vec<String>,
```

On `WorldState`, not `Character`: it is a standing instruction about the world
rather than a fact about you — the same division `bought_licence` makes in the
other direction. Empty by default, so **no seam**: every save that opens on M14
opens on this.

### 1.5 The four rules it must obey

- **A boss is never instant.** `rout` refuses one by name (`boss_at`) and this
  refuses one for the same reason: a creature standing on a tile is the end of a
  dungeon, and the whole of it is the fight.
- **Nothing may be marked that has not been beaten five times**, and the check
  is in core. A shim that decided it would be a second rulebook.
- **A defeat is still a defeat.** `run` simulates honestly and `settle` settles
  honestly, so an instant battle you *lose* walks you home and takes what you
  were carrying. That is the risk the feature is paid for with, and the receipt
  has to say so plainly on the strip because there is no screen to see it on.
- **It still tires you**, which `settle` already does. `an_instant_battle_costs_
  what_a_fight_costs` is the check, and it must compare against a *fought*
  fight rather than against the number 4 — a test that hardcodes `PER_FIGHT` is
  a second copy of it.

### 1.6 The screen

A menu, reachable from the map screen, listing every creature beaten five times
with its count and a switch. It is a `.screen` — read *Screens, and the three
times one covered another* in `CLAUDE.md` before adding one, and give it a tier
in the z-index table.

**It says what a mark costs you**, in the engine's words: an instant battle
cannot be watched, cannot be fled, and cannot be slowed down and read. Somebody
who marked a creature and then wanted the log has spent something.

### 1.7 What only a browser can answer

That the fight screen **does not open**, and that the result reaches the strip
and survives into the history. `cargo test` can prove the settlement is
identical; it cannot prove nothing was drawn.

---

## 2. The experience curve

### 2.1 What it is now, measured

`progression.rs` holds `XP_TO_NEXT`, a table of thirty-two generated from
`round(20 · 1.35^(L−1))` and checked against the formula by
`the_table_matches_the_formula`. It is **exponential from level one**:

| to leave level | costs | total banked |
|---|---|---|
| 1 | 20 | 20 |
| 5 | 66 | 198 |
| 10 | 298 | 1,133 |
| 14 | 989 | 3,822 |
| 20 | 5,989 | 23,244 |
| 32 | 219,471 | ~860,000 |

The shipped walk ends at **level 14**, and the M14 start lines are level 20 only
because they were handed twenty thousand experience. Levels past twenty are not
a difficulty curve, they are a wall.

### 2.2 What is asked for

**Quadratic to 50, exponential after 50.** Two consequences the plan has to
state rather than discover:

- **`MAX_LEVEL` is 32.** A curve that changes shape at fifty needs a table that
  reaches at least fifty, and past it there must be somewhere to go or the
  change of shape is decoration. The table is a `const` array sized by
  `MAX_LEVEL`, so this is a number and a regeneration.
- **`XP_DIVISOR` is set by a test, not by taste** — `level_five_lands_where_the_
  plan_says` walks the shipped map and demands level five in 25–35 fights.
  Flattening the curve moves that band. **The band is the contract**; if the new
  curve puts level five at nineteen fights, the divisor moves, not the test.

### 2.3 The anchor, which is the human's and is answered

> level 20 should be about 150 fights, and it should be less than it is now by
> about half. take whichever is lower

**Two bounds, and both are checkable.** They are stated separately because one
needs a walk and the other does not:

| bound | value | how it is checked |
|---|---|---|
| **the half** | `xp_to_reach(20) ≤ 8,526` | arithmetic, against the recorded 17,053. `cargo test`. |
| **the hundred and fifty** | level 20 in **130–170 wins** on the shipped map | `make play`, the way level 5's 25–35 already is |

*Take whichever is lower* means both are asserted and the design must satisfy
both — if the walk lands 150 fights above the half, the half wins; if the half
still leaves 150 out of reach, go under it.

### 2.4 Measured, so the milestone does not start from nothing

**Today, on the shipped map**, from `testing/transcripts/m14-a-new-game.txt`:

| level | wins to reach it | banked |
|---|---|---|
| 5 | 32 | 132 |
| 10 | 67 | 793 |
| 11 | 74 | 1,091 |
| **20** | **never reached** | **17,053** |

The walk plateaus at eleven. Level 20 today is somewhere between three and five
hundred fights depending where they are had, which is the thing the ask is
about.

### 2.5 The shape

Quadratic to 50, joined so the curve does not step:

    xp_to_next(L)  =  A · L²  +  B · L  +  C            for L < 50
                   =  xp_to_next(50) · G^(L − 50)       for L ≥ 50

What is **not** negotiable is the joint: `xp_to_next(50)` computed both ways
must agree, or the curve has a cliff at the one level a player will be watching
for.

**Three constraints fix A, B and C**, and the third is the human's number:

    xp_to_next(1) = 20        the first level costs what it always has
    xp_to_reach(5) = 132      the one measured contract, kept exactly
    xp_to_reach(20) = target  the ask

Solved, with `reach(5)` held at 132 so **`XP_DIVISOR` need not move and
`level_five_lands_where_the_plan_says` should still pass untouched** — which is
the reason to pin the first two rather than fit freely:

| target | A | B | C | first ten levels | reach(10) |
|---|---|---|---|---|---|
| 5,500 | 2.127 | −0.552 | 18.425 | 20 26 36 50 69 92 119 150 186 226 | 748 |
| **6,000** | **2.420** | **−1.819** | **19.400** | **20 25 36 51 71 96 125 160 199 243** | **783** |
| 7,000 | 3.005 | −4.354 | 21.349 | 20 25 35 52 75 103 138 179 226 278 | 853 |
| 8,526 | 3.897 | −8.221 | 24.324 | 20 23 35 54 81 115 158 208 266 332 | 960 |

All four are monotone with positive terms, and all four hold `reach(5) = 132`.

**6,000 is the recommendation and not the answer.** It clears the half-bound
with room, and it puts 150 fights at forty experience a win — which is a
plausible average over a climb that starts in the pit at three and ends in the
Treyway at two hundred, and is exactly the sort of plausible that a walk
disproves. **The walk settles it**, and if 150 fights overshoots twenty, the
target comes down rather than the band moving.

### 2.6 The trap

`XP_TO_NEXT` is `[i32; MAX_LEVEL]`. Exponential growth past fifty overflows an
`i32` quickly: `20 · 1.35^49` is already 2.3 billion. **Pick the base against
the type, or change the type, and say which in the commit.** An overflowing
table is a level that costs a negative amount, and `[profile.test]` has overflow
checks on, so it will fail loudly in a test and silently in a release build.

---

## 3. The second class comes off the tree

### 3.1 One line, and it is `shop.rs`

```rust
Paper::Second => Some(StockGate::TreesFinished(1)),   // becomes None
Paper::Expert => Some(StockGate::TreesFinished(2)),   // unchanged
```

The expert paper is **not** in this ask. It stays behind two finished trees,
because the twenty-four points are its price — that is what makes it free.

### 3.2 What has to move with it

- `papers.rs::the_second_paper_wants_one_finished_tree_and_five_thousand`
  becomes a test about five thousand.
- `a_gate_counts_and_says_so` must keep a user: the expert line still counts,
  and the check must not go vacuous.
- `check_the_papers_are_drawn_and_refused` in `testing/drive.py` reads the
  second line's refusal.
- Spike's van is `hidden_until_level: 10`, which becomes the **only** thing
  between a new character and a second class besides the money. Say in the
  commit whether that is intended; it is the one place the change could go
  further than the ask.

### 3.3 What must not move

`Character::choose_second_class` refuses without `second_paper`, and the fork is
still permanent and still cannot be re-raised. **The paper is spent on the
choice, not on the purchase** — M13.7's divergence, and it is why the fork opens
from the sheet rather than after every fight.

---

## 4. Milestones

| # | Milestone | Deliverables | Acceptance | Deploys |
|---|---|---|---|---|
| **M15.0** | **The tally** | `pay_a_win` counts by creature; `Game::beaten`; `WorldState::instant`; `Game::mark_instant`/`unmark`, refusing a boss and anything under five. **Nothing a player can see.** | `a_win_is_counted_by_creature` (and a rout counts); `nothing_under_five_can_be_marked`; `a_boss_is_never_instant`; the save round-trips an empty and a non-empty `instant`; **no seam** | no |
| **M15.1** | **The battle nobody watches** | `fight::instant` or the `walk` arm that runs `run` + `settle` and draws nothing; the shim's fourth shape; `app.js` logging it through `log()`. | `an_instant_battle_pays_what_a_fought_one_pays` (bounty, experience, drops, errand tally — compared against a fought fight, not against constants); `an_instant_battle_costs_what_a_fight_costs`; `an_instant_defeat_still_walks_you_home` | no |
| **M15.2** | **The menu** | The screen, its tier in the z-index table, the switch, and the sentence saying what a mark costs. | Browser: the menu lists what is eligible and nothing else; marking one and walking into it **does not open `#fight`** and does put the result on `#tape` and in `#history`; unmarking it opens the screen again | **yes** |
| **M15.3** | **The curve** | `MAX_LEVEL`, the piecewise `xp_to_next`, the regenerated table, `XP_DIVISOR` re-anchored only if it has to be. | `the_table_matches_the_formula` over both pieces; `the_curve_has_no_step_at_fifty`; `the_curve_never_overflows`; **`the_climb_to_twenty_is_at_most_half_what_it_was`** (≤ 8,526, arithmetic); `level_five_lands_where_the_plan_says` still passes **untouched** — the curve is fitted to hold it, so a divisor that moved means the fit is wrong; and a `make play` transcript putting **level 20 in 130–170 wins** | no |
| **M15.4** | **The second paper** | The gate comes off; the strings and the tests follow. | `the_second_paper_wants_five_thousand_and_nothing_else`; `a_gate_counts_and_says_so` still has a user; the browser check reads the new line | **yes** |
| **M15.5** | **The notebook executed** | `SECOND-ORDER-M15.md` turned into milestones and run, `HANDOFF-M15.md`, `CLAUDE.md`. | The transcript's numbers written into `CLAUDE.md` | no |

---

## 5. Decisions that are the human's

1. **How far past fifty?** `MAX_LEVEL` has to reach at least 50 for the ask to
   mean anything. 60 is a suggestion and not an answer.
2. ~~**What a level twenty should cost**, in fights, on the shipped map.~~
   **Answered:** *about 150 fights, and less than it is now by about half —
   whichever is lower.* Both are bounds and both are asserted; see §2.3. The
   half is arithmetic (≤ 8,526) and the 150 is a band on the walk (130–170), the
   way level 5's 25–35 already is.
3. **Whether Spike's van stays behind level ten.** With the tree gate gone it is
   the only thing left besides five thousand Fnorp.
4. **Whether an instant battle may be *lost*.** The plan says yes — it is what
   the feature is paid for with — but a player who marked something and then
   out-levelled their own board would lose a walk's worth of carried experience
   with no screen to see it coming. The alternative is refusing to settle a
   fight the simulation loses, which is a lie about a fight that happened.
