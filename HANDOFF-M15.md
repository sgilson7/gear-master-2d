# HANDOFF-M15.md — a fight you have already had

*`PLAN-M15.md` is the frame. This is the block's own record: what each milestone
found, and why the divergences are what they are. `SECOND-ORDER-M15.md` is the
notebook — fifteen rows, written when noticed — and M15.5 is that notebook
executed.*

*`CLAUDE.md` is the current account of the repo. Read that first if you have
never seen this.*

---

## 0. State in one paragraph

**All six milestones are done and one fault reported from play was fixed
alongside them.** The suite is **854 passing** and the browser gate walks **81
`ok:` lines over three engines** — 67 in any one of them, which is worth saying
because the total is easy to read as a per-engine count.

**It is committed and it is not deployed.** `git log origin/main..HEAD` would
send seven commits: the shore fix and M15.0 through M15.5. Two of them are
deploy points the plan marks (M15.2, the menu; M15.4, the second paper) and
neither has been taken, because the agent does not `make publish` on its own
judgement.

**No save seam.** The catalogue is still 568, the one new field is
`WorldState::instant` and it is `#[serde(default)]`, and every save that opened
on M14 opens on this. `a_save_written_before_the_mark_opens_with_nobody_marked`
is that claim as a check rather than as a sentence in a commit.

---

## 1. The milestones

| # | Milestone | Status |
|---|---|---|
| — | **The shore said what a cliff says** — two gates stand on ground nobody can walk on and both were silent; `Step::crossing` → `Step::refused_by` | ✅ 842 |
| M15.0 | **The tally** — one `bump` in `pay_a_win`, `Game::beaten`, `WorldState::instant`, `mark_instant`; **eight of nine boss creatures also stand in pools** | ✅ 842 |
| M15.1 | **The battle nobody watches** — `fight::instant` is `run` then `settle`; `walk_home` extracted so a defeat has one answer | ✅ 847 |
| M15.2 | **The menu** — `#instant` on tier 30, the switch, `what_a_mark_costs` in core; four plants, four documented traps | ✅ 847 · **deploy point** |
| M15.3 | **The curve** — quadratic to fifty, exponential after; the plan's target was 40% too high | ✅ 851 |
| M15.4 | **The second paper** — the tree gate off; two tests rewritten rather than repaired | ✅ 852 · **deploy point** |
| M15.5 | **The notebook executed** — six candidate rows, and one of them found a three-block-old lie | ✅ 854 |

---

## 2. The three asks, and what each one actually cost

### Instant Battle — almost no engine

The ask reads like a feature and is mostly a `map`:

    let instant = gm2d_core::fight::instant(g, DIFFICULTY).map(|s| { … });

`fight::instant` is `run` then `settle` with nothing drawn. **There is no
settlement code in the block.** `settle` already pays the bounty with the
class's arm on it, rolls the drops, counts the errand tally, banks the streak,
ticks the order book and tires you four percent — so *"you should still lose
tiredness"* cost nothing to implement, and *"the battle still occurs in the
background"* is literally true rather than a simulation of being true.

**What it is not is `fight::rout`.** A rout and a golem are one mechanism
because they are one thing — *something happened that meant the fight did not* —
and this is the opposite. It pays the speed bonus, rolls the drops, ticks the
order book and costs the four percent, and a rout deliberately does none of
those. `an_instant_battle_is_a_fight_and_does_tick` and
`a_rout_is_not_a_fight_and_does_not_tick` are those two sentences facing each
other where they can go red.

**The receipt is the entire interface**, which is the part that needed writing
rather than wiring. There is no screen, no replay and no card, so a player who
marked something and then out-levelled their own board finds out on the strip or
not at all.

### The curve — the recon was the milestone

`PLAN-M15.md` §2.5 hands over four solved candidates and recommends
`reach(20) = 6,000`, on the reasoning that 150 fights at forty experience a win
is plausible — and says in as many words that this is *"exactly the sort of
plausible that a walk disproves"*. It is:

| target | `reach(20)` | level 20 lands at |
|---|---|---|
| today's `1.35^(L−1)` | 17,053 | 795 wins |
| §2.5's 5,500 | 5,502 | 175 wins |
| **§2.5's recommendation** | 6,001 | **184 wins** |
| the half-bound | 8,526 | 233 wins |
| **shipped: 4,300** | **4,298** | **150 wins** |

**The mean over a walk's first 150 wins is 28.7, not 40.** The half was never
the binding bound — 4,298 is a *quarter* of 17,053 — and the plan says what to
do: *if the half still leaves 150 out of reach, go under it.*

`xp_to_reach(5)` is held at **132 exactly**, which is the fit's whole point:
`XP_DIVISOR` did not move and `level_five_lands_where_the_plan_says` passed
untouched, exactly as the prompt said it should.

### The second paper — one line, and two tests rewritten

    Paper::Patent | Paper::Second => None,
    Paper::Expert => Some(StockGate::TreesFinished(2)),

The expert keeps its gate because two finished trees is what makes it **free**.
A test that pinned the old behaviour was rewritten rather than repaired, with
the reason in the commit — the move M12.3 made with the two MVP pillar tests.

---

## 3. What the block found that nobody was looking for

**A hover has been describing a deleted mechanic for three blocks.**
`Effect::GrowSlotRows`'s detail told players a row arrives *"on top of the row
that grid gets when the level rotation reaches it."* **M12.3 deleted the
rotation.** Every frame has started at three rows and stayed there since, and
eleven skill nodes went on describing the old game on hover.

It was found by a lint written for a **formatting nit**: nine engine strings
carried runs of eighteen to twenty-six spaces, the wreckage of a `\` line
continuation a scripted edit ate. Chasing the whitespace read the sentence.
`no_sentence_has_a_gap_in_the_middle_of_it` is kept, and its corpus is the maps
— **which nothing had ever tone-linted** — plus every sentence the engine
composes.

**Two gates in this game stand on ground nobody can walk on and both were
silent.** Reported from play at the shore. `world::step` refuses on `walkable`
before anything asks the place, so a gate on impassable terrain can never say
why. `a_place_on_ground_you_cannot_stand_on_says_why` is over every place on
every map, because a list of two written by hand is a list that can be one.

---

## 4. What is open, and it is the human's

1. **`MAX_LEVEL = 60.`** `PLAN-M15.md` §5 decision 1 says *"60 is a suggestion
   and not an answer"*. Taken because the work could not start without a number.
   The first level whose cost will not fit an `i32` is **95**, so there is room
   to move it.
2. **Spike's van is `hidden_until_level: 10`**, and with the tree gate gone it
   is the only thing between a new character and a second class besides five
   thousand Fnorp. §3.2 flags it; left as it is, because taking it off would go
   further than the ask went.
3. **The 130–170 band is not measurable to the precision it was written at.**
   Two walks on the shipped curve put level twenty at **150 and 192 wins**, a
   28% spread around a ±13% band, and `make play` misses the *level-five* band
   by three fights on its own transcript. `reach(20) = 4,300` is kept and the
   spread is written down rather than tuned away. If the intent was
   *"noticeably faster and roughly a third of what it was"* the number is right;
   if it was literally 150 on every route, no single number does that.
4. **The third town's name**, carried from `HANDOFF-M14.md` §6 and untouched
   here.

---

## 5. Deploying it

Two deploy points and neither taken. `git log origin/main..HEAD` sends:

    745ba01  The shore said what a cliff says, and two gates stand on water
    b788b98  M15.0: the tally, and a boss is a tile rather than a name
    7904bee  M15.1: the battle nobody watches
    af4f5c5  M15.2: the menu
    0519668  M15.3: the curve is quadratic to fifty
    b48fb27  M15.4: the second paper is gated by its price and nothing else
    <this>   M15.5: the notebook executed

**The shore fix is the one a player is waiting on.** It was reported mid-block
and it is live-affecting: the way south is behind the Wextreen Reach and the
game was refusing without saying so.

After a deploy, verify against the live page the way `CLAUDE.md` has demanded
since M8 — `GM2D_ORIGIN=… testing/drive.py chromium firefox webkit` — and check
the **pair**, not the number: `index.html` asking for `app.js?v=X` and that
`app.js` carrying `BUILD='X'`. **Read the exit code, and never a pipeline's.**
