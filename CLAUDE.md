# CLAUDE.md — operating notes

Kept current. If something here is out of date it is a bug in this file.

**How to read it.** The top is the part you have to know before you touch
anything: what this is, the rules that are load-bearing, and the commands. Then
six parts, one a system, each holding that system's design decisions *and* the
mistakes it cost to arrive at them. The mistakes are kept on purpose — every
one of them is a day somebody spent — but they are filed under the thing they
happened to rather than in the order they were found, which is how this file
was arranged until it was long enough that the arrangement mattered.

`HANDOFF.md` is the short door in. Read that first if you have never seen this.

## What this is

A 2D tile-based open-world RPG built on the gear-assembly auto-battler forked
from `sgilson7/gear-master`. `PLANNING-BRIEF.md` is the brief; `PLAN.md` is the
plan and **wins where the two disagree**; `TONE.md` governs every string a
player reads.

**Every milestone is done, M0 through M13.** M0–M5 shipped the MVP, tagged
`v0.1.0-mvp`; the board was rebuilt against the original's colourblind design;
M6 added the art and the tone pass; M7 the shops, errands and the first
dungeon; M8 curses made visible, a quest log, enchs, a fourth class, and a door
at the end of it; M9 what a creature leaves behind — three sets, two rules no
stat could express, and two crossings that make the north a decision; M10 where
an ench comes from, an item that fires once, and a fifth class; **M11 what is
through the door** — nine new maps, a tower that comes down, a lake that
empties, and three instruments that read a map you cannot otherwise enter;
**M12 board pressure** — a save that could not be played, three counters where
there was one, events that pay something and say what they pay, and a row you
earn instead of one a level hands you; **M13 what two classes reach** — a
second paper on Spike's counter, ten experts one for each pair, a tree apiece
that moves nothing but its own promise, and four new rules to move it with; and
**M14 down twice, and the country under the country** — the Treyway grew a
south, two four-floor dungeons with a puzzle on every floor went into it and
under the lake, and behind both of them is a country with one town on it that is
empty and says so.
<https://sgilson7.github.io/gear-master-2d/>

**M11 is live**, deployed at `43804e49` on the human's word. Its live check is
the first one this project has not written by hand: `GM2D_ORIGIN=… drive.py`
walks all forty-two gate checks against the deployed page, three engines. See
*A deployed fix is not a delivered fix* for why that is a separate step from
the deploy going green.

**M12 is done and live**, verified the way this file has demanded
since M8: `GM2D_ORIGIN=… drive.py` walked all forty-six checks in three engines
against the deployed page, and the player's own frozen save was loaded on it and
walked. Eight milestones — M12.B (a save that could not be played), M12.0 (the
measure), M12.1 and M12.1a (the barrel and the price tiers), M12.2
(commissions), M12.5 (events that pay), M12.3 (a row is earned) and M12.6 (the
chain you can see). `TRIAGE-M12.md` is the sweep, and **two of its rows are not
the builder's**: an agent spot-run — `testing/AGENT-BRIEF-M12.md` is written and
there is now a deployed build to run it against — and the friend whose one
sentence started the block. **M12.0 deployed nothing** — its whole job
was to make board pressure a number before anything tried to move it, and the
number is in `testing/transcripts/m12.0.txt`. `PLAN-M12.md` is the frame,
written while M11 was in flight; **`PLAN-M12-EXEC.md` is the execution plan and
wins where the two disagree**, the same way `PLAN.md` wins over
`PLANNING-BRIEF.md`; its §10 carries the baseline. It carries one milestone the frame does not have —
events that pay something and say what they pay — added on the human's ask.
`PLAN-M9.md`, `PLAN-M10.md` and `PLAN-M11.md` are done; `PLAN.md` §6d is what
M11 left open, §6c is M10.3's, §6b is M9.4's and §6a is M8.8's.

**M19 is five things reported from play, and three of them were bugs.** The ball
slides (a flight is twenty ticks and the page advanced eight a frame, so a shot
was over in fifty milliseconds); a diamond catches the ball, so hitting a gate
is entering it; five obstacles were wearing the event's mark; the long cart runs
between towns you have stood in; and **the Kettle-Stoker did nothing at all** —
it dealt 746 against a classless character's 746, because empowerment scales
magic hits and the board was swinging a blade. There is a **glossary** on `G`
now, with every number in it read from the constant that decides it. See *The
furnace reached nothing* and *Everything you need to play, in one place*.

**M17 is done and live**, deployed at `fac56a16` and verified the way this file
has demanded since M8: `GM2D_ORIGIN=… drive.py` walked **all eighty-five checks
against the deployed page**, and the pair agrees. **The overworld is a table.** The Treyway and the Undercountry
carry `traversal: "shot"` — the arrow keys aim a cue, space fires, the ball
runs to rest under integer physics in `core::shot`, and *where you stop is
where you are*. Every other map is still walked a tile a press. Six
milestones, `PLAN-M17.md` is the frame, `HANDOFF-M17.md` is the block's record,
and `SECOND-ORDER-M17.md` is its notebook — **forty rows, all closed**, and
rows 16 to 19 are what the one worklist row turned into once it was built and
found to be wrong. **M18 is the notebooks executed**: every worklist row in
`SECOND-ORDER-M16.md` is resolved or marked as the human's, and there are two
of the second kind. See *Two countries are tables* in Part one.

**The block's own fault was found by the browser gate and not by `cargo test`**,
which is the argument for the gate in one sentence: a gate offered from beside
it opened a bar still under nine feet of water, because **the tide crossing has
never carried a condition of its own — the impassable ground *was* the
condition.** Nothing in core could see it, because every core test that could
have asked was asking *is the gate offered* rather than *should it be*.

**M16 is done and live**, deployed at `e3a2193f` and verified the way this file
has demanded since M8: `GM2D_ORIGIN=… drive.py` walked **all seventy-eight
checks against the deployed page**, and the pair agrees — `index.html` asks for
`app.js?v=e3a2193f` and that `app.js` carries `BUILD='e3a2193f'`. Seven
milestones, `288810c` through the gate, and the block is **three maps, two base
classes, eleven experts and no new components**.

**The Eleven Reefs** is the shape: three floors under the Wextreen Sands behind
a gate that is not there until you have taken the tenth surveyor's sheet and is
shut until something on the frame can read the flat. Nine stakes over three
bands where the one that leads on is a different one each time; three doors of
which the second **takes what the third needs**; four sinkholes, two of them
false, and four levers that open each other's alcoves. On the plate at the
bottom is **the Tenth Surveyor, wearing a real player's board** — thirty-eight
components in the cells the human seated them in and all six of that character's
enchs, which is the first creature in the game to carry one.

**And the block's largest finding is one the plan could not have had: no board
this game hands a player reaches Rare.** `PLAN-M16.md` §4.2 hangs two doors on
`AssembledOfRarity`; the level-45 run tops out at an item rating of **50**
against a `RARE_AT` of 90, and Auto-packing everything it owns does not move it.
`geared_from` — both shelves and every errand, at level ten — reaches **135, which
is Epic**. So a door at legendary would have been a wall with a sentence on it
and a door at epic would have refused the deep board while passing the shallow
one. See *No board a player can build reaches Rare*.

`PLAN-M16.md` is the frame, `HANDOFF-M16.md` is the block's own record and
`SECOND-ORDER-M16.md` is its notebook — **forty-one rows**, and the last
milestone is that notebook executed.

**M15 is done and live** — six milestones and one fault reported
from play, `745ba01` through the notebook, and **three asks from the human**:

| | |
|---|---|
| `745ba01` | the shore said what a cliff says, and two gates stand on water |
| `b788b98` | M15.0: the tally, and a boss is a tile rather than a name |
| `7904bee` | M15.1: the battle nobody watches, and one answer to where a defeat puts you |
| `af4f5c5` | M15.2: the menu |
| `0519668` | M15.3: the curve is quadratic to fifty, and the plan's target was 40% too high |
| `b48fb27` | M15.4: the second paper is gated by its price and nothing else |

**Instant Battle** is the block's shape: beat a creature five times and you may
mark it, and from then on meeting it settles where it stands — simulated in
full, banked in full, and **never drawn**. It is `fight::run` then
`fight::settle` with nothing opening, so it costs the four percent and rolls the
drops because that is what going through the settlement already does. **No new
settlement code and no save seam.**

**The curve is quadratic to fifty and exponential after it**, and the block's
largest single finding is that `PLAN-M15.md` §2.5's recommended target was 40%
too high: it reasoned *150 fights at forty experience a win* and the measured
mean over a walk's first 150 wins is **28.7**. `xp_to_reach(20)` is **4,298**
against 17,053, and `xp_to_reach(5)` is **132 exactly** so `XP_DIVISOR` never
moved and `level_five_lands_where_the_plan_says` passed untouched.

**And the second class comes off the tree.** Spike's second paper is gated by
five thousand Fnorp and nothing else; the expert paper keeps its two finished
trees, because that is what makes it free.

`PLAN-M15.md` is the frame, `HANDOFF-M15.md` is the block's own record and
`SECOND-ORDER-M15.md` is its notebook — **fifteen rows, six of them worklist,
and M15.5 is that worklist executed.**

**M14 is live**, at `4d065d1d`. Nine milestones and two faults reported from
play, `402d89f` through `ff9ba52`, walked against the deployed page in three
engines.
`PLAN-M14.md` is the frame, `HANDOFF-M14.md` is the block's own record and
`SECOND-ORDER-M14.md` is its notebook — **twenty-one rows, four of them
worklist, and M14.6 is that worklist executed.**

**The block's thesis is that this game's puzzle is the board.** Its real locks
are `LooseItemOfSize`, `AssembledOfRarity` and the survey instruments — things a
player opens by *packing* and by *reading* — and eight floors are eight of those
two things. Every one is **monotone**: flags only grow, so no move on any floor
can make the way on unreachable, and every floor has a blind solution the
harness counts rather than the plan asserting it.

Before it: `0aae36b` at `5fb93594` — **spells, books, crystal balls, inks and
alignments are buyable.**
106 casting components, six of them reachable and all six errand rewards —
because `roll_barrel` named its kinds by hand and covered one of the weapon's
three recipes, and because the arcane shelf is on a map that does not exist. See
*The barrel could not hold a book* and *A reward you could buy, eighteen times
over*.

Everything below shipped between M12 closing and M13
opening — all of it reported from play, all of it on `main`, all of it walked on
the live page:

| | |
|---|---|
| `d45643e` | a hover no longer moves the board, and what a grid takes moved onto the grid |
| `2cfb8f6` | the frozen-save gate check waited on a word that was already on the strip |
| `9f7ef4c` | a swing is not a constant, and the replay row said it was |
| `b3b296f` | the swing check raced the playback it was scrubbing |
| `d03d8c1` | the Kettleworks was a wall, and the wall was the gear |
| `f114cdf` | a defeat costs you your place, and the door is the door again |
| `98ff7cf` | a card you cannot leave, and a card carrying somebody else's errands |
| `3d5c059` | a chain errand you can finish, and a fight you can slow down and read |
| `eabc973` | a locked choice names the chain, a bank, a door that survives a reload, and a page that notices a new build |
| `5ff7eb4` | an instrument has a frame of its own, and surveying no longer costs your sword arm |
| `f439274` | a row is bought all the way up to the old size, and the tree's wires are drawn where they are |

And since M13 shipped:

| | |
|---|---|
| `f2d1bd4` | M13: what two classes reach — live at `ac2256c7` |
| `6790385` | the deploy note for `ac2256c7` |
| `0aae36b` | the barrel could not hold a book — live at `5fb93594` |

**Two of them touched the save and none of them the catalogue**, and neither did
M13. The bank adds `Character::banked`, which defaults empty and is skipped when
it is; the instrument frame adds a sixth board, and a file naming five gets one
at the height a player's frames are. A component changing which grid it goes in does
not change its name and `catalog_fingerprint` hashes names, so the catalogue is
still 568, there is still no seam, and every file that opened on M12 opens on
this — `Character::repair_boards` lifts an old build's instrument out of the
weapon grid on the way in. The stamp a deploy leaves is a record of that deploy and not
a claim about now, the same way the M12 and M13 stamps below are; **what has to
agree is the pair**, `index.html` asking for `app.js?v=X` and that `app.js`
carrying `BUILD='X'`. The live one as this was written is `5fb93594`, and it
will be wrong by the next deploy whatever that deploy is for.

**M14 is done and live**, deployed at `4d065d1d` on the human's word and
verified the way this file has demanded since M8: `GM2D_ORIGIN=… drive.py`
walked **all seventy-eight checks in three engines against the deployed page**,
and the pair agrees — `index.html` asks for `app.js?v=4d065d1d` and that
`app.js` carries `BUILD='4d065d1d'`. Nine milestones, `402d89f` through
`ff9ba52`, plus **two faults reported from play** — the barrel that showed one
thing and sold another, and the map that came out the wrong shape. The suite is **832
passing** and the browser gate walks **all three engines at 78 `ok:` lines**,
five of them M14.5's and every one negative-tested — **two of those five found
faults on a green build**, which is the reason the gate exists. `PLAN-M14.md` is
the frame, `HANDOFF-M14.md` is the block's own record, and `SECOND-ORDER-M14.md`
is its notebook: **twenty-one rows, four of them worklist, and M14.6 is that
worklist executed.**

**Nine of `PLAN-M14.md`'s decisions came out differently and every one is in the
divergence table**, because the floors were drawn before anybody walked them —
and the largest is that §1.1 is written against the *cut campaign's*
`Requirement`, which is `CLAUDE.md`'s own *grep for it, and then check which of
the two you found* arriving on schedule.

**M13 is done and live**, deployed at `ac2256c7` on the human's word, verified
the way this file has demanded since M8: `GM2D_ORIGIN=… drive.py` walked all
sixty-three checks in three engines against the deployed page, and every screen
the block added was driven on it by hand as well — see the note in *A deployed
fix is not a delivered fix*. Ten milestones, and the block's frame is
`PLAN-M13-2.md` — there is no `PLAN-M13.md`; the file on disk is the `-2`.
`HANDOFF-M13.md` is the block's own door in and `SECOND-ORDER-M13.md` is its
notebook: **twenty-three rows, one a consequence the plan did not ask for and
the work made true anyway**, written when noticed rather than at the end, and
read by the last milestone as its own worklist. That convention paid for itself
— seven of the rows were marked as candidates and all seven were executed in
M13.9, and three of them turned up something that was actually wrong.

**Where the next block goes is a convention and worth keeping:** a block's frame
is `PLAN-M15.md`, and if the frame turns out to be a different document from the
one you execute, `PLAN-M15-EXEC.md` **wins where the two disagree** — the same
relationship `PLAN.md` has with `PLANNING-BRIEF.md` and `PLAN-M12-EXEC.md` had
with its frame. Divergences from a plan go in the table at the bottom of this
file with their reasons, in the commit that makes them, because a divergence
nobody wrote down is indistinguishable from a mistake. **And keep the
notebook**: it is the cheapest thing in this block and it is what made the last
milestone a worklist rather than a guess.

Three things are outstanding rather than open: the two rows of `TRIAGE-M12.md`
that are not the builder's — an agent spot-run against a deployed build, and the
friend — and `PLAN-M12-EXEC.md` §8 row 13, which is M12's own biggest miss
written down as a decision.

**And one is M14's, and it is the human's**: `PLAN-M14.md` §9 decision 1, the
third town's name. It ships as *a town with no name on the post yet*, which is
what the plan says to do if unanswered and is true and in register.
`common::UNWRITTEN` is where the emptiness is declared, and the day somebody
names it, the name goes in the map file and nothing else moves.

**M12's thesis is board pressure.** Cells outnumber pieces, so a board reads
as inventory space rather than a puzzle, and there is no moment where putting
one thing down means taking another up. Everything in the block is a lever on
that: a bargain barrel and commissions raise throughput, earned rows slow
cells, and events that pay gear are a third faucet.

**"No reroll" was the block's founding decision and M12.6 reversed it, narrowly
and on the human's ask.** `PLAN-M12.md` §0 declined the friend's reroll by name
and its reasons still hold **for the shelf**, which is untouched: a town that
sells something different every visit is not a place. What turns over is the
floor under the shelf and the menu above it — the barrel and the order book —
and neither is a place's character. `n * n` a roll, counted per type so the bin
never prices the book, reset every tenth level, and **the order being made is
never rerolled**, because you paid for it and its clock is running.

**And the whole economy was multiplied by five**, on the human's word, with the
enchs held at 2,000 and **income deliberately unchanged**. The first attempt
scaled the bounties to match and was wrong: real play reaches the Drambus Stack
holding about three thousand Fnorp, so the rise is a *correction* for income the
tests had been undervaluing. The thing undervaluing it was
`a_restorative_costs_less_than_the_walk_home`, which priced tins against "the
pit pays about six a win" — the poorest fight in the game, on the first map, at
level one, where nobody buys a tin. **When a test disagrees with a cost, suspect
the test's idea of income first.**

**The demo ends under the lake now**, at a door behind the thing at the bottom
of it, and getting there means dropping a five-floor tower or walking on water.
What is past *that* door is not written — not hidden, not locked, not saved for
later — and the ending screen says so in as many words. The game's overall
structure past that point is still the human's to decide; `PLAN-M8.md` §5.6 is
where the question was first written down and it is still the live one.

**M13's thesis is that a class is a thing you finish.** Five classes have been
a fork at level five since M5 and nothing else; finishing one has meant nothing,
because there was nothing past it. Now **a finished tree is a countable fact**
— `SkillsData::tree_finished` — one finished tree buys a second class off
Spike's counter and two finished trees hand over the **expert** that pair
reaches, free, because the twenty-four points were the price. Ten pairs, ten
experts, `C(5,2)` and no more: `every_pair_of_offered_classes_reaches_an_expert`
is what says the table is complete, because a list of ten written by hand is a
list that can be nine.

**A character holds up to three classes and all three are live.** The five
shipped powers touch five different rules and no pair of them collides; the ten
experts are written to the same constraint and `no_pair_of_live_powers_disagrees`
is what keeps it true. Nothing arbitrates, because nothing has to.

**M11 was two seams and M12 and M13 are none.** M11.5 moved the catalogue 544 → 550 and
M11.9 moved it 550 → 568, so a save written before *that* block is refused by
name — the design, said out loud in both commits, with the number living in
`a_save_from_before_this_block_is_refused_by_name`. **Nothing M12 or M13 added moved
it.** Every field either block introduced defaults, and the fivefold price rise
is seam-free because `catalog_fingerprint` hashes names and not prices. A save
that opened on M11 opens on M13 — and it opens **lighter**, because M13.9 took a
field back out: see *Derived, never banked, and that includes the last one*.

**No rest point, and there still should not be one.** Combat health resets
every fight, so a rest would restore something that was never spent. What a
town does instead is where the design landed: it is the only place experience
becomes a level, everything you are carrying is lost if you fall before you
reach one, **and it takes the tiredness off** — which is the one thing a fight
does spend for good. The thing the old note was looking for was never a rest;
it was a stake.

## Rules

Break one of these and the failure is silent and expensive. `HANDOFF.md` picks
five of them out as load-bearing — core stays graphics-free, the shim decides
nothing, content lives in `data/`, a new field is a compile error until the save
carries it, and the page never recomputes a number. The rest are here because
something cost a day.

- `crates/core` never imports `wasm-bindgen`, `web-sys`, or anything
  DOM-shaped. If you are reaching for one, stop.
- `crates/wasm` is a shim. It moves strings across the boundary and decides
  nothing. A rule decided there is a rule the test suite cannot reach in
  seconds, and then there are two rulebooks.
- Content lives in `data/*.json` — the map, the events, the tree, **the town
  shelves (`shops.json`) and the errands (`quests.json`)**. **If you are editing
  a `.rs` file to change what a player reads, you are in the wrong file.** The
  two exceptions are inherited and known: the component catalogue is `piece.rs`
  and the theme tables are `theme.rs` (mirrored into `data/theme.*.json`, which
  is generated — `REBASELINE_THEME_DATA=1`).
- **A new component needs a themed name in the same change.**
  `the_turtle_theme_covers_the_catalogue` fails otherwise, and it is right to:
  a piece nobody has named reaches the player in the engine's words.
- **Adding to `CATALOG` changes the save fingerprint**, and older saves are
  refused with a sentence naming both catalogues. That is the design; say so in
  the commit when it happens.
- Never write a game string without `TONE.md` open.
- **A choice at a chain root hands over an errand, and no two of them the same
  one.** Reported as *"its either 12 experience or 20 experience, so they'd
  always take the 20"* — both halves opened a different chain and neither was
  visible, so it was not a decision, it was a smaller number beside a larger.
  An errand is the one thing this game has that says *something has opened and
  it is somewhere else*: it lands in the log and the log points at the map.
  `every_choice_at_a_chain_root_starts_its_own_errand` refuses the next one that
  is only a number, and `Quest::granted` keeps the branch you did **not** take
  from sitting on the same tile a moment later offering itself.
- **When a test disagrees with a cost, suspect the test's idea of income
  first.** The fivefold price rise looked like it broke the economy and had not:
  `a_restorative_costs_less_than_the_walk_home` priced tins against *the pit
  pays about six a win*, which is the poorest fight in the game, on the first
  map, at level one, where nobody buys a tin. Real play reaches the Stack
  holding about three thousand Fnorp. **The shipped game's income is not scaled
  to make a test happy** — the measuring stick is.
- **A second copy of a constant goes stale in three engines at once.** The gate
  carried its own `12` for the barrel's price ceiling, and every price in the
  game was multiplied by five. It reads `shop::BARREL_CEILING` off the payload
  now. Same shape as the `EVENT_ONLY` list read with a regex — twice — which put
  a set piece in the barrel and was caught by a test rather than by rereading.
- **The suite's cost was runtime, not debuginfo.** The fork's
  `[profile.test] debug = "line-tables-only"` was already carried across; what
  it did not cover was `opt-level`, and this suite simulates a great deal of
  combat. At `opt-level = 2` a warm run is fourteen seconds instead of minutes,
  with debug assertions and overflow checks still on.
- **Save round-trip tests run on every commit. A red round-trip blocks
  everything.** `tests/save.rs` is that suite; `testing/drive.py` walks the same
  property through three real browsers.
- **Adding a field to `Game` is a compile error until the save carries it.**
  `SaveFile::of` and `into_game` destructure exhaustively. Two fields are
  skipped on purpose and each says so where it is skipped. Do not "fix" a
  destructure by adding `..`.
- **The agent does not run `git push` or `make publish` on its own judgement.**
  The default is that only a human deploys, and it holds even when the work is
  green and the human is clearly going to want it. **The exception is an
  explicit ask** — "deploy this", "push it" — and it has been taken twice: the
  repo's creation, and M8's thirteen commits. Neither was inferred. When it is
  taken, the deploy is not finished until it has been verified *against the live
  page*, which is a separate step and has its own section below.
- Do not start a milestone before the previous gate is live and the human has
  seen it.
- **A refusal names the thing in the way, and a place standing on ground you
  cannot walk on knows more about it than the terrain does.** `world::step`
  refuses on `walkable` before anything asks the place, so a gate on impassable
  terrain can never say why it is shut. Reported from play at the shore: *"the
  land is pink and it says no way through."*
  **And the condition is not "on impassable ground", it is "somewhere a player
  can be refused"** — which the first version of the lint got wrong. Two gates
  stand on ground nobody can walk on and only one of them has a **standable
  neighbour**: the grating is in open water in the middle of the lake, so
  without `Rule::Wade` you cannot reach a tile beside it and with it the water
  is walkable and the gate opens. A `shut` there is a sentence no player can
  ever read, which is the dead content this block had just caught in a skill
  node's hover, so it was written and then deleted.
  `a_place_you_can_be_refused_at_says_why` asks the sharper question over every
  place on every map, **and asserts the one exception by name** the way
  `common::UNWRITTEN` does — a list that quietly grew is a list that has gone
  stale.
- **A slice of a capped list is a comparison that quietly stops being about
  anything.** `#tape` keeps the last few lines and drops the rest into the
  history, so a browser check written as `tape(page)[before:]` goes on returning
  the last one or two however many fights have happened. It is the *compares
  zero with zero* failure with a scrollback in it — the check would have gone on
  passing on a build where the receipt landed and then scrolled away.
- **A sentence nobody proof-reads is a sentence that can say anything.**
  `no_sentence_has_a_gap_in_the_middle_of_it` was written for a formatting nit —
  nine engine strings carrying runs of eighteen to twenty-six spaces, the
  wreckage of a `\` continuation a scripted edit ate — and what it found was
  `Effect::GrowSlotRows`'s hover telling players a row arrives *"on top of the
  row that grid gets when the level rotation reaches it."* **M12.3 deleted the
  rotation three blocks ago.** Same failure as the `STARTER` comment this file
  quoted as live fact for five blocks and the controls blurb M12.B deleted,
  except player-facing. The lint's corpus is the maps — **which nothing had ever
  linted** — plus every sentence the engine composes.
- **A page that draws a world has to be told which world, every time it can
  have changed.** Three instances now and one rule: a defeat carrying you to
  another map (M11), a save being restored (M12), and — M14 — **a choice being
  taken.** Answering an event raises a flag, a flag is what a `hidden_until`
  reads, so the third turn of the chair opens the door in the north wall, and
  the page went on drawing an empty room because `paintPanel` re-reads only when
  the *map id* moves. Found by the browser gate, and it could only be: nothing
  in `cargo test` can see a place that is there and not drawn.
  **Four instances now**, and the fourth is M15's: an **instant battle that is
  lost** walks you home across maps, and it happens *inside* `walk()` — forty
  lines after the panel was painted against the map you stepped on. A rout
  cannot lose and moves nobody, so it has no equivalent hole; that was checked
  rather than assumed.
- **The page draws numbers core sent it, and never recomputes one.** Violated
  three times and invisible every time: the replay once subtracted damage from
  a health total it kept itself and ignored `absorbed`; it once opened every
  fight with an empty armour bar because nothing *announces* a balance nobody
  had to earn; and a curse's countdown was nearly divided out of the playback
  head, which would have drawn a shape the fight never had.
- **A set is the set or it is gear.** `loadout::set_of` is the one answer to
  "is this the Mandate", and both the item's name and the rules it grants read
  it. Two conditions: every component in the item names the same set, and every
  component that names the set is in the item. Agreement alone lets two thirds
  of a three-piece set call itself whole, because most recipes have an optional
  slot; completeness alone would let a stranger in.
- **A map does not know about bags, or about levels.** `world::step` takes a
  `world::Allowances` — a handful of bools and a number the caller fills from
  the character — and never the character. Same division a gate's key makes.
  `Allowances::of` matches `Rule` exhaustively, so a new rule is a decision
  about walking rather than a silence.
- **One puzzle is not monotone, and it is safe because it forgets.** The
  Drowned Gallery's three stones are pushed onto three marks, and *a stone
  pushed into a corner is exactly the move that makes the way on unreachable* —
  Sokoban is the one shape the rule below forbids. So the stones are a fact
  about **this visit**: `WorldState::blocks` carries the floor they belong to
  and `WorldState::go_to` forgets them, so walking up the stair and back down
  reseeds the room. **Reseeding on the map *id* is not enough** and a test
  caught it: you leave and come back to the same floor, so what matters is that
  you *left*. `go_to` is the one door onto `map` and it exists because there
  were seven callers — a gate, a warp, two walks home, a defeat, a floor's kick
  and a charm — and the next thing that must be forgotten on leaving cannot be
  forgotten in six of them.
- **A puzzle is monotone, because flags are — and that is a guarantee about
  flags, not about doors.** `WorldState.flags` and `answered` only ever grow, so
  a puzzle whose wrong move locks the right one is a puzzle the save cannot come
  back from. **Growing is what broke it once**: `answer_event` wrote the event
  into `answered` whatever the choice did, so *Try the slot as you are* — the
  one choice in the game that does nothing at all — shut the Wextreen Sump's
  weighed door for good and made the floor unfinishable. Reported from play by
  somebody standing in it. **A choice that changes nothing is not an answer**
  now, `no_choice_that_does_nothing_spends_a_door` is the rule over every event
  there is, and `world::reopen_doors_a_no_op_shut` unsticks the saves it already
  happened to. `puzzle::solvable_blind` could not have caught it: a blind solver
  never takes a choice that does nothing, which is the one move a model of a
  good player will not make. Every one in this game is solved by
  *discovering* something and never by avoiding something — and that is not a
  limit, it is what kind of puzzles this game has. `puzzle::solvable_blind` is
  the proof and `every_floor_in_the_game_can_be_solved_blind` runs it over every
  floor there is, because a list of six written by hand is a list that can be
  five. **The corollary bit in M14**: nine moves in an order over three
  always-offered labels is not expressible, because a choice carries one
  requirement and one outcome. See *The Silt Stair*.
- **A drain may take ground away only if a later one gives it back.** Every
  drain before M14 opened something — a lake into bed, a tide into coast, a
  channel into silt — and one into a wall would be a lake that empties into
  rock. The Drowned Gallery floods a road on purpose, and what makes that safe
  is that the chain after it in the same list turns the water it made into silt:
  flags only grow, so somebody who pulled A can always pull B. **The pairing is
  the monotone rule written where it can be checked.**
- **A repeating event may never pay.** `TileEvent::repeats` is the third kind of
  event, after the card that is answered once and the note that is only read,
  and it exists because a *sequence* is not a decision — the chair at the bottom
  of the Silt Stair is three moves at one object. What stops it being a faucet
  is at load: a repeating event may raise a flag, cost you fatigue, or nothing,
  and gold, a component, experience, a tin, an errand and a warp are all
  refused. That list is what a puzzle is made of.
- **Two lints asking one question is how they drift.** M14 hid a stair behind a
  flag in a game that had only ever gated on `answered`, and **three** separate
  checks went red, each for the right reason and the wrong question, each
  counting only place ids. Two were retired into
  `no_flag_is_waited_on_forever`, which asks it over every mark and every reader
  at once. The third —
  `every_flag_an_event_sets_is_read_by_something` — asks the *other* direction
  and stayed, and once it learned about `hidden_until_all`, `needs_all`,
  `floors[].cleared` and drains it was still right about two live faults.
- **Before adding a system, grep for it.** `explain.rs` was written with a
  duplicate `Action::describe` and `Trigger::describe` already in `piece.rs`,
  and M8 opened with a request to add curses to a game that has had 59
  cursing components since the fork.
- **Break a new check and watch it fail before you keep it.** Three checks have
  shipped vacuous. A check that compares zero with zero is not a check.
- **A derived number needs somewhere it is shown**, or it cannot be told from a
  bug. Four skills worked perfectly and were reported as broken because nothing
  printed them. **And a number that is shown needs somewhere it is read**:
  `Outcome::Xp` wrote into a counter nothing consulted for four blocks, so nine
  events printed a receipt for experience that never existed.
- **Check the second visit.** Three faults found after M11 shipped were all one
  fault — a key that stayed in the bag, an event that re-opened for ever, and a
  door that had to keep being unlocked. Everything in this game is walked over
  more than once, and every check that plants a state and steps once is asking
  about the first time only.
- **A promise on an irreversible screen must reach something.** Two shipped
  classes advertised a number and delivered nothing — `Showstopper` for two
  milestones and `Recycler` for one — and the fork does not come off, so a
  player who took either spent the one choice the game does not let them retake.
  `every_offered_class_reaches_something` is the guard and it has to **call
  rather than declare**: its first version matched the variant and named where
  the power was honoured, which a stubbed payout passed cleanly. *A lint that
  reads a list rather than the behaviour is the failure it exists to catch, one
  level up.*
- **Derived, never banked — and that includes a thing, not only a number.** A
  node that hands over an ench does not write it into the save; the save carries
  the node. `Character::enchs` reads it fresh, the same way `player_stats` reads
  the tree, so retuning what a node awards retunes every character who took it.
  M11 extends it to a *place*: how many floors of the Drambus Stack are gone is
  how many of its boss ids are in `answered`, and there is no counter. **M13.9
  finished the job**: `Loadout::assembly_pct` was the last banked derived number
  in the game, written into every save and then thrown away on the way in — a
  number that is stored and ignored is a number somebody will one day believe.
  It is derived on load and nothing writes it to a file. See *Derived, never
  banked, and that includes the last one*.
- **Ask the recipe table, never a list of kinds.** `roll_barrel` named thirteen
  kinds by hand under a comment claiming it covered *the five recipes*; there
  are seven, because a weapon is a blade **or a book or a crystal ball**, and
  the list covered the blade. A hundred and six casting components were on no
  counter in the game for as long as the barrel has existed.
  `shop::barrel_wants` is derived, with the counts the recipes ask for, and
  `every_recipe_assembles_out_of_the_barrel_alone` is the check that its
  grid-shaped neighbour could not make. **Sixth time a hand-written list has
  cost this project something.**
- **A shelf nobody can walk up to undercuts nothing.** The barrel and the order
  book refuse what a town stocks, so the cheap tier cannot undercut the
  authored one — and `shops.json` carries a shelf for High Wick, which is on no
  map and is the *arcane* one. It was holding three of the five barrel-priced
  spells out of the cheap tier on behalf of a counter nobody has stood at.
  `data::towns_on_the_map` is the question both pools ask now.
- **A knob is a player-facing string, and so is a canonical name.** An expert's
  tuning prints the bare knob — `Effect::Tunes::line` — so a knob called
  `harvest` promises the nature pool, which is the theme's word for it and not
  the class's. Two lints, and both read the theme rather than a list somebody
  typed: `no_knob_or_line_speaks_a_word_a_theme_would_produce` walks the
  **right-hand side** of `Theme::vocabulary` over every node line and every
  knob, and `every_class_name_is_one_a_player_could_read` refuses a canonical
  that reaches a screen as a squashed variant name unless every theme renames
  it. The first found the ten expert promises saying *Funny* where they meant
  mana, on the sentence somebody reads before an irreversible choice.
- **A promise is printed from the tuned power, never from the roster.**
  `class::CLASSES` is the classes as written, before a point is spent;
  `Character::class_defs` is the classes you *are*, with the expert's knobs
  turned, and it returns owned definitions for exactly that reason — a
  `&'static` cannot carry a tuning. Thirty-eight of the sixty expert nodes cost
  points and changed nothing until M13.6 found it. The one place that still
  reads `CLASSES` directly is the fork screen, and it should: nobody choosing a
  class has spent a point in its tree yet.
- **A map file is content and so is a map file's terrain, but which map file
  you are reading is the game's.** M11 needed a lake that is water until a
  tower falls and lakebed after. That is not a second map and it is not a grid
  in the save: `TilesData::drains` names rows and the flag that empties them,
  and `data::map_now` reads the map *through* the game. `data::map_at` is the
  file, `map_now` is the game — the same split `place_at` and `place_now` made,
  for the same reason.
- **What you carry may change what a map says, and it must never change what a
  map is.** `survey::mods_for` is a pure function of (map, instrument, how many
  items are assembled) and returns a `SurveyMod`. Nothing about surveying is in
  a map file, because an instrument is the character's and a map is the
  world's — the identical division `Allowances` makes for a crossing.
- **Every walkable question takes the allowances.** `World::walkable` was
  `passable || (wade && shallow)` and M11.4 widened it to the whole body of
  water. The check that made that safe is older than the change:
  `an_allowance_never_shuts_anything`, over every tile of every map — which is
  what let eleven maps be added without re-deriving reachability by hand — and
  another nine in M14 on top of that.

## Commands

    make test          # the engine suite, native, 717 tests, ~14s warm
    make check         # fast type-check
    make web           # build dist/web/
    make test-ui       # drive the built page in three real browsers
    make play          # play the demo start to finish and read every screen
    make serve         # build and open locally
    make art           # compile art/*.tex to web/assets/*.svg
    make dress         # search the catalogue for a creature near a rating
    make read          # print an existing creature's board and its rating
    make test-ui-setup # one-time: venv + headless chromium

**Two environment variables, both added in M11 and both about *which page*.**

    GM2D_WEB=dist/web-gate ./packaging/package-web.sh   # build somewhere else
    GM2D_WEB=dist/web-gate testing/drive.py chromium    # and gate that build
    GM2D_ORIGIN=https://sgilson7.github.io/gear-master-2d/ testing/drive.py …

`GM2D_WEB` is honoured by the build script, the gate and the walker, and it
exists because M11.8 put a long-running agent playtest on `dist/web` while
M11.9 was moving the catalogue. Rebuilding to run the gate does not merely swap
the page under a run in progress — it moves the save fingerprint, so the run's
own save stops loading and the sitting is over. Three tools that each hardcoded
one directory was one directory too few.

`GM2D_ORIGIN` skips the local server entirely and walks a page that is already
up, which in practice means **the live one**. That is the *verify against the
live page* step this file has demanded since M8, done by the thing that already
holds all fifty-five questions instead of by a person remembering four.

**`make test-ui` and `make play` are different tools.** The first walks a route
chosen to exercise checks and asserts; the second starts a new game and plays
it, and its output is a transcript rather than a verdict. The second is the one
that found an Auto-pack seating the starting kit for the whole game and a class
fork opening underneath the town — both of which the first was green through.

**And `make play` is not the instrument the pacing bands are set with**, which
M15.3 had to find out. `level_five_lands_where_the_plan_says` walks a **fixed
east-west patrol on the pit road over nine seeds and asserts on the mean** — a
controlled measurement of the map's pacing. The walker wanders, goes north,
loses, and drops what it is carrying: it misses that same 25–35 band by three
fights on its own transcript, and nobody has ever thought that a fault. So a
band written *"exactly the way level 5's already is"* cannot be handed to
`make play`. What M15.3 used instead is a **replay of a transcript's own
payouts** — every `+N experience, carried` summed as though it were all banked —
which measures the curve against real fights and separates it from the walker's
banking problem. It reproduces 25–35 on both shipped transcripts, which is how
it earned the right to answer a question the walker cannot.

**`make test-ui` builds and `testing/drive.py` does not.** Running the driver
directly walks whatever is in `dist/web`, which is how a new `Event` variant was
reported missing for two runs in M10.1. If a check is failing on something you
have just written, rebuild before you believe it.

**There is a third tool now, and it is not ours.** `make play` is a walker
somebody who built the game wrote, and it can only find what its author thought
to look for. M11.8 added `testing/agent_driver.py` and
`testing/AGENT-BRIEF-M11.md`: one command per turn against a browser that stays
open between them, driven by an agent that has been **forbidden the source**
and given only what a shop poster could tell it. See *Somebody who did not
build it*.

**It reaches the ending, and the ending moved.** M9.4 taught the walker two
things a player already knew — that a road refused three times is a road you
stop walking at, and that the way out of a dungeon is a target like any other —
and M11 taught it five more, written down in `testing/transcripts/README.md`.
The transcripts are in `testing/transcripts/`. The current one reaches the door
under the lake at step 4406, level 14, 342 wins and 170 losses.

**The walker is not deterministic and two runs of it disagree.** One M11.9 run
finished; the next spent 240 cycles walking out of the pit town, losing in the
Kettleworks Field, and walking back — never banking, because its destination
was on another map and only a town spends what you carry. That is a finding
about the walker rather than the game, it is in `PLAN.md` §6d, and the general
version is worth more than the instance: **a walker with a destination stops
being a player.** A run that loops is still a transcript; read where it loops.

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

    REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core

---

# Part one — the world you walk on

## Two countries are tables, and you shoot across them

**M17.** `TilesData::traversal` is `step` or `shot`, and the Treyway and the
Undercountry are the second kind. The arrow keys aim a cue instead of taking a
step, space fires, and the ball runs to rest — *where you stop is where you
are*.

- **The physics is integers and it is in core**, for the reason every roll is
  per-mille: a seeded walk has to produce the same flight in every browser, and
  a float is the one thing that rounds differently in three engines. Sixteenths
  of a tile, seventy-two five-degree angles read out of a `(cos, sin)` table,
  powers one to ten, and **no `f32`, no `sqrt` and no trigonometry at
  runtime** — `the_physics_has_no_floating_point_in_it` is a lint over the
  *source*, because an `f32` produces a flight that is *nearly* right and a
  hash cannot tell you about that until somebody in another browser reports it.
- **The shim animates what core returns and decides nothing.** A `Flight` is a
  list of sub-cell positions and the page's only job is to walk it. A physics
  loop in JavaScript would be the first thing here that ran differently in
  three engines, and `check_a_shot_animates_to_where_core_said` compares the
  drawn path against `preview_shot`'s.
- **`POWER_UNIT` is 30 and the plan said 22**, and the property that decides it
  is not in the plan: **distance has to be monotone in power.** A cue where
  pulling back harder lands you *nearer* is a cue nobody can aim, and at 22 —
  and at every restitution under 80 — it is not, because a fast ball spends its
  extra speed on ricochets and a bounce that costs too much makes a strong shot
  die at the wall it hit.
- **A bumper adds a fixed kick and is counted, and both halves cost a draft.**
  Thirty *percent* compounds: reflected at four fifths and boosted a third, a
  ball between a bumper and a wall gains four percent a round trip for ever,
  and shots ran to `MAX_TICKS`. A fixed kick settles at about `2.2k` — **and
  still never stops**, because **friction is charged per tile crossed and a
  ball bouncing in place crosses none.** So the pump is counted: three, and
  after the third a bumper is a wall, which is what it already is to a ball
  approaching it.
- **A landing is `world::arrive_at`, which is `step`'s own second half.**
  Extracted rather than written twice, and `step` calls it too, so there is one
  answer to *what is here*. It rolls at `LANDING_MULT` — 150% — because a
  landing is a longer stay than a step and a shot map has far fewer of them.
- **`shots-taken` is bumped and `tiles-walked` is not**, and the strip says
  *shots* rather than *walked* on a table. Which of the two it is comes from
  core, because *which map you are on* is the game's.

### A gate beside you offers its refusal, never the way through

The block's most expensive hour, and the cause is one sentence: **the tide
crossing has never carried a condition of its own, because the impassable
ground *was* the condition.** Every other gate in this game is answered by the
shim asking the bag; that one was answered by `walkable` refusing the step
before anything asked the place. So a rule that reached it *without stepping*
bypassed the only lock it had, and the first draft of the beside-rule walked a
player over a bar still under nine feet of water.

- **Once the tenth cairn drains it the tile is `coast`**, which is ground a
  ball can come to rest on, so the way south is entered by landing on it like
  every other gate. That is what makes it safe for the beside-rule to hand back
  a refusal and nothing else.
- **`world::here` needed it too, and that is a soft-lock rather than a
  tidy-up.** On a table you come to rest beside a gate, so somebody who landed
  next to the Reach's edge, built a compass in the frame it opened and pressed
  *Go in* was refused for ever. `gate_beside` is one function and both doors
  onto a tile ask it.
- **The reachability lint could not have found it, because it took "beside" for
  an answer.** A lint that accepts *landable or landable beside* cannot tell a
  gate you can enter from one you can only be turned away from — this file's
  *a lint that reads a list rather than the behaviour is the failure it exists
  to catch, one level up*, in a new coat. It asks the sharp question now: **can
  a ball come to rest on this tile, in some state of the world** — flooded
  **and** drained, because a map is not one grid and a drain turns `tide` into
  `coast`. And it asks an **obstacle** the question that is true of one, which
  is whether any shot *hits* it: a bumper is a tile no ball can ever rest on,
  and that is what a bumper is.
- **Found by the browser gate in three engines**, which is the argument for it.

### A diamond catches the ball, and the two rules are for two cases

Reported from play: *"if you just simply hit the diamonds to enter a zone, you
should enter it, it shouldnt have to perfectly land on it"* — and a cue that
demands a tile exactly demands a hole in one. `PlaceKind::catches` is **gates
and bosses**, the two things the map draws as a diamond and the two that are a
way *into* somewhere; they end the flight where they are hit.

- **Not towns and not events.** The Undercountry's town stands in the middle of
  its one fast lane, and a town that caught every shot down that road would be
  a road nobody can use. A signpost you can roll past is a signpost, and a cart
  that snatched the ball out of the air would be a toll booth.
- **Not the tile you shot from**, or a ball leaving a gate it was just refused
  at is put straight back — a soft-lock made of one rule.
- **A diamond on ground the ball cannot enter is answered by the beside-rule
  instead**, which hands back its refusal. The tide crossing is on `tide` until
  the tenth cairn goes up two maps away; nothing can land on it or fly into it.
  The test asserts both by asking `walkable` rather than listing the exception.
- From the Treyway's start the road west goes from **55 exact landings to 66
  shots caught**.

### Five obstacles were drawn as events

Reported as a question, which is the tell: *"there are a bunch more event
diamonds, are they actual events or are they obstacles?"* Only gates and bosses
had a draw arm, so all nine obstacles fell through to the generic small diamond
— five physical behaviours wearing one mark. `look.rs`'s rule is that **a mark
is a shape nothing else draws**, and it had been kept for everything on the map
except the things the ball actually hits.

A boulder is a filled disc with a ring, the one round *solid* thing here. A
pocket is the same circle inverted — dark, empty, lit rim — because one throws
the ball away and the other keeps it. Spikes are teeth on a groundline, open at
the top, because you go *through* them. Sand is low banked lines with stipple.
A chute is two rails and an arrowhead, the only mark on any map that points.

### You can watch the ball slide

A flight is 4 to 47 physics ticks and a median one is **twenty** — and the page
advanced *eight a frame*, so the ball was drawn about three times and the whole
shot was over in fifty milliseconds, with the full path painted before it had
travelled any of it. Reported as *"you should be able to watch the ball slide"*,
which is exactly what it was not doing.

The clock drives it now rather than the frame counter, and the ball is drawn
**between** ticks, which is what makes it slide rather than hop. `TICK_MS` is 55
so a median shot is about a second. The trail grows behind it and stays drawn
after it lands.

### A tile one step away is a tile you cannot shoot to

From directly adjacent every power overshoots or bounces off the wall behind,
so three gate checks that planted themselves beside their target and fired
never arrived — the bar of shingle, the third town, and the screen after it.
That is not a fault; it is what a cue *is*, and the fix is a tee rather than a
nudge. **Worth knowing before somebody puts two places next to each other on a
table.**

### You pull the cue; you do not tap the table

A click on the canvas fired a full-power shot from wherever the pointer
happened to be, because `pointerdown` set the cue and `pointerup` fired it — so
somebody clicking the map to focus it for the keyboard took a shot. A shot
needs a **drag** now, thresholded in CSS pixels because it is about the hand
and not the map, and a press that never travels leaves a cue set with the keys
alone.

- **The trail is cleared where a shot starts, not where it lands.** A line
  wiped on landing is a shot you cannot look back at, and on a table looking
  back at the last one is how you take the next — the strip's argument for
  keeping the last thing said. It also makes reduced motion mean what it says:
  **do not move things is not tell me less**, so the ball is at rest on the
  next frame with the path still drawn.
- **A shot can end on any screen, so a check that fires one must tidy any of
  them.** `dismiss_card` and `close_fight` cover two of six; a check whose
  `finally` covered two left a town over the page, the next check's first click
  timed out, and the whole failure list went unprinted. `clear_screens` is the
  one door.
- **`check_a_floor_still_steps` is the hardest check in the block to
  negative-test**, and the reason is the finding: every lie about *the arrows
  mean two things now* takes the whole gate down before the check runs, because
  a floor that grows a cue is a dungeon nobody can walk out of.

### The walker asks core where a ball goes

`shot::aim_at` sweeps the seventy-two by ten and returns the gentlest shot that
lands on a tile, and it is the **one** answer: the reachability lint floods
with it, the browser gate crosses with it, and `playthrough.py` aims with it. A
pathfinder in Python would be a second answer to *where does a ball go*, and it
would be the first thing here that disagreed with the engine about the map.
`make play` crosses the Treyway in **three** shots against the plan's nine, and
everything on either table is reachable in **two** rounds.

## The world

- **Danger is measured, not typed.** A region's danger is the mean of
  `rating::creature_rating` over its enemy pool.
  `tests/world.rs::no_data_file_types_a_danger_number` fails the build if a
  number ever appears in a data file. Tuning the map means moving creatures
  between pools; typing a number would be tuning the ruler.
- **Every roll is integer per-mille.** A seeded walk has to produce the same
  encounters in every browser, and float rounding is the one thing that breaks
  that silently — the symptom would be a save that replays for the person who
  wrote it and not for the person they sent it to.
- **A blocked step draws nothing.** Bumping into a cliff must not advance the
  stream, or a replay would depend on the player's mistakes rather than their
  path.
- **The map is never saved.** `WorldState` holds a position, an answered set
  and flags. The grid is `data/tiles.json`, and content is not state — the
  discipline is borrowed from upstream's `county.rs`.
- **The page draws numbers core sent it.** An earlier draft recomputed the
  encounter chance in JavaScript for the debug overlay, which put the formula
  in two languages with only one of them tested.

## The first map has one town

One town and a great deal of wilds. Kettleworks and High Wick are written,
shelved and given errands, and are **not placed** — they belong on maps that do
not exist yet.

- `towns_on_this_map_all_trade_and_all_want_something` replaces a set equality
  that held only while one map carried every town. The direction that still
  holds is that a town you can walk into trades and wants something; the other
  direction is a named `STAGED` list, which is the point: **content waiting for
  a map is fine and content waiting for nothing is an orphan, and the only
  difference is somebody having written the name down.** A third stray shelf
  fails there.
- A save can now remember a town this map does not have — a file from before
  they moved, or from a map this build does not ship. `World::repair` falls
  back to the start, and `world.rs` tests exactly that; without it such a save
  would put a player nowhere.

## A save never places you where you cannot stand

`WorldState` is `#[serde(default)]` in the save so that files written before M2
still open — and a default `WorldState` stands at `(0, 0)`, which on this map is
rock. A player carrying an autosave from an older build spawned inside it and
could not move in any direction.

**Anything that loads a position runs `World::repair`.** It puts the player at
their last town if one is known and walkable, and at the map's start otherwise.
Two core tests and one browser check hold it, the last of which plants the exact
file rather than waiting for one.

The general rule: a field defaulted for backward compatibility is a field that
will arrive wrong, and the loader is where that is caught.

`try_step` repairs too, not only `load_json`. A position you cannot stand on is
a dead end rather than a glitch — there is no key that gets you out of it — so
the first keypress fixes it whatever put it there.

## A coordinate means nothing without the map it came from

**The worst bug this project has shipped, and the one it was hardest to
believe.** Reported from a real save: after clearing the first floor of the
Drambus Stack, the character could make **one move per page load** and click
**one button per page load**, and the file loaded onto West Bambulon when it
should have been on the Treyway. A new game had none of it.

It was a panic. In a wasm build a panic is an `unreachable` on the console and
nothing else, and because the page's handlers each call into the shim, the
first one to touch the fault poisons the module and every handler after it is
dead — which is exactly what *one move and then nothing, until you reload*
looks like from a chair.

The cause is one sentence: **a 16×16 map's coordinates were being read against
a 20×20 map's grid.** `WorldState::at` is a position and `WorldState::map` is
the map it is a position on, and every lookup in `world.rs` took the first and
assumed the second. `region_of[260]` on a 256-tile grid is a panic; `[190]` is
not, and quietly answers about somewhere else.

- **Found by bisecting the shim's exports against the deployed page**, not by
  reading. The save was replayed in a browser with one export stubbed at a
  time until the panic moved. It was `quest_log_json` — the quest log asks
  where an errand points, which asks every map about a position, which is the
  one code path in the game that hands a coordinate to a map that did not
  produce it.
- **`idx` returns `Option<usize>` now**, and everything downstream of it does
  too: `terrain_at`, `region_index`, `region_at`. An out-of-bounds coordinate
  is a question with no answer, not a slot in a vector.
- **And `crossing_between` checks the map id before anything else.** Bounds
  alone would have turned the panic into a wrong answer — a crossing on West
  Bambulon refusing a step taken in the Stack — which is the failure that does
  not announce itself. *A guard that converts a crash into a silent lie is half
  a fix.*
- **`terrain_name` answers `""` off-map** rather than refusing, because it
  feeds a label and a label has somewhere to put nothing.
- The save is in the repository and the gate plants it:
  `check_the_frozen_save_is_playable` loads that exact file, walks it, and
  opens the log. **A bug reported with a save attached should never be fixed
  without that save becoming a check** — everything else about this one was
  reconstruction.

## Twenty-one errands that could be taken and never finished

Reported from play: *"i have the quest what is behind the door, and when I try
to turn it in to marbulon, I am unable to as she does not have a button in her
event to submit this new quest completion. this is probably a greater issue
across the quest chains."* It was: **all twenty-one chain errands, over ten
turn-in places.** The whole of what M12.5 added could be started and none of it
could be handed in.

`QuestsData::at` filtered every `granted` errand out of the list a place is
concerned with. The intent is right and is written above it — *the branch you
did not take must not be sitting on the tile offering itself* — and it was the
wrong instrument, because **`at` answers a different question from the one that
rule is about**:

| question | whose | where it lives |
|---|---|---|
| which errands is this place concerned with | the data's | `QuestsData::at` |
| which of them will it talk to *you* about | the character's | `quest::shown_at` |

- **The filter was never what kept a chain off the counter.** `stage` already
  answers `Locked` for one nobody has been handed — *"a granted errand that has
  not been granted is not offered, it simply is not yet"* — and `Locked` is
  what the new filter hides. So the rule is unchanged and the hand-in comes
  back. The filter's only effect was the bug.
- **The shim held half the rule and could not see the other half.** It was
  already filtering `Offered | Locked` at a place that is not the giver, which
  is the same kind of judgement; the two halves were in two crates and neither
  knew about the other. `shown_at` is both, in core, once — and the shim calls
  it instead of assembling its own answer.
- **A rule with two homes is a rule with two answers**, which is the thing the
  shim is not allowed to do and this is what it looks like when it happens by
  accretion rather than by decision.
- **Both directions are tested**, because fixing this by deleting the filter
  outright would have put every unearned branch back on the counter. The core
  test hands one chain errand over and then asserts the other twenty are still
  invisible at their own givers — and it asks `shown_at` rather than `stage`,
  because testing the rulebook is not testing the screen.

## A card you cannot leave, and a card carrying somebody else's errands

Two faults reported together, both on the event card, and one of them is a
soft-lock:

> *"events do not have a way to close them without making a decision; some
> events in the kettleworks are locked, and so you get trapped in the event
> screen, and have to reload the browser page to get out."*

**`showCard` hid the action bar whenever an event had choices** — a decision
is a decision — and **sixteen events have exactly one choice, gated behind a
flag.** They are the second rung of a chain: locked until you have taken the
first, which is the whole design of M12.5's roots. Walk onto one without the
flag and the card is an unclickable button with no exit. `the-quench-pond`,
`the-hooper`, `the-scrap-line` and thirteen more.

**Escape had closed the card the whole time**, and that is the other half of
the finding rather than a mitigation: *a thing that works and cannot be seen
is a thing that does not work* — the fourth time that sentence has been the
answer here. The way out is on the screen now, on every card, and **walking on
does not answer the event**: it is not marked, and the tile offers it again,
which is what makes leaving one a choice rather than a way to lose it.

- **`#card-bar` is unconditional.** Not "shown when nothing is takeable",
  which would be a second rule about what a card is for and would leave the
  next kind of unanswerable event to find out about the hard way.

And the second, which is the Marbulon report finally caught:

> *"you see her quests below the text box when you walk through the gate to
> the kettleworks as well as the overworld."*

**`#card-errands` was painted by `openEvent` and cleared by nobody.** The door
in the western wall shows its paragraph through `showCard` *directly*, so
Marbulon's errands from two tiles back were still in the card when the border
drew its own. **`showCard` owns every part of the card now** — title, prose,
choices, receipt, errands, bar — which is the only version of that rule that
cannot go stale, and `openEvent` paints the errands *after* it rather than
before. The same shape as `paintPanel` having to be told which map, every
time.

- **I looked straight past this twice.** Two sittings of "cannot reproduce"
  read the card's text truncated to four hundred characters, and the errands
  are *below* the prose. The reporter's own words said "below the text box"
  and I read them as "instead of the text". **When a report says where on the
  screen a thing is, that is the instruction, not the decoration.**
- The check reproduces both in the reporter's words with the fixes reverted:
  *"MARBULON'S DOOR" has 2 choices (2 takeable) and no way out of it*, and
  *the door's card is carrying the last place's errands*.
- And it must not need the button it is testing. The first version clicked
  `#card-close` to get on with its next assertion, so on a broken build it
  hung for thirty seconds and reported a traceback instead of the finding it
  had already made. `leave_the_card` takes the button if it is there and
  Escape if it is not.

## A defeat costs you your place

Reported from play: *"when you die there, and you return to the overworld
through a door, you appear back exactly where you died in the overworld,
instead of at the door to the overworld."*

The bookmark is real and is the right idea: `WorldState::positions` remembers
where you were on each map you have left, so **a gate that names no landing
tile lands you where you left off** — which is what makes the Treyway a
country rather than a chute, and it is written in the map file rather than
branched on in the shim.

**What it did not distinguish is walking off a map from being carried off
one.** The defeat path wrote the bookmark down with a comment arguing that
coming back should put you *"into the fight they lost"*. It reads well and it
plays badly, and the report is the answer: a border you re-enter in the middle
of is not a border.

- **`WorldState::forget` is the rule and it is core's**, called by the shim's
  walk-home. Forgetting rather than overwriting, so `World::arrival` falls
  through to the map's own start — which on the Treyway is the tile the door
  put you on the first time. *"At the door"* is then a fact the map states
  rather than a coordinate somebody wrote down twice.
- **Only the map you fell on.** Dying on the Treyway must not lose your place
  on a map you have every right to still be standing on, and the core test
  says so in its third assertion.
- **Riding home on the Drover's Stride still remembers.** That is a decision
  and this was not: you paid a tin to leave, and coming back where you left is
  what you paid for. A defeat takes what you were carrying, and now it takes
  your place as well.
- **The engine half and the half that was wrong are in different crates.**
  `tests/world.rs` covers the rule; the walk home lives in the shim, which is
  where the bookmark was being written, so `check_a_defeat_costs_you_your_place`
  is the one that would have caught it. It plants a death on the Treyway, walks
  back through the door and reads the tile — and with the old line put back it
  says *came back in at [4, 4], which is where the player died*.

## The north is a decision, not a slope

Nothing stopped a level-one character walking fifteen tiles north into a region
of two-thousand-rated creatures. The gradient was a gradient and not a gate.
`PlaceKind::Crossing` is the gate — a `Gate`'s sibling, on this map rather than
onto another one.

- **A crossing guards a region, not its own tile**, which is a divergence from
  `PLAN-M9.md` and the map is the reason: rows four to fifteen are open ground
  twelve tiles wide, so a crossing that refused only the square it stands on
  would need a dozen of them across a row — which is the wall the plan
  rejected, drawn in places instead of in rock. The place stands on the **near**
  side of what it guards, so it is a milestone you can walk up to and read.
- **It is the first thing gated on what you *are*.** A key is in the bag and a
  level is not, so the number travels in `Allowances` with everything else the
  map may not go and read.
- **Before the step, not after it.** Every other place does its work once you
  are standing on it; this one refuses, so it happens first — and a refused
  crossing draws nothing and counts no tile, the same rule a cliff obeys.
- **A threshold, not a cage.** A step that stays inside the guarded region is
  never refused and neither is one out of it, so a save planted on the far side
  is somebody who can still walk. And **going home is never refused**: the walk
  after a defeat is a placement, so `World::repair` consults no crossing.
- **The refusal is two registers.** `shut` is the world's and lives in
  `tiles.json`; the number is the engine's and is derived in
  `crossing_refuses`. A `shut` line quoting its own level would be a second
  copy of `needs_level` two lines above it, and the lint refuses a digit there.
- **The quest log says when a road is shut.** `Guide::shut`, found by playing
  it: the M9.4 walk pressed north into the first crossing for nine thousand
  steps because the log went on pointing at an errand behind it without a word.
  A log that points somewhere you cannot go and says nothing is a log that is
  wrong rather than a road that is shut.
- Its own mark, and neither of the two it is nearest: a post with a bar across
  it, the one shape on this map that is a line rather than a body.

## The lake has a rim

`Rule::Wade` opens water that touches land. **Measured before it was written:**
on this map that is 14 of the lake's 28 tiles, so row 9 becomes crossable end to
end and the middle 14 stay shut. No new terrain and no repaint.

- `World::shallow` is the ground's half and `World::walkable` is the game's.
  Orthogonal only — a diagonal touch is a corner, and a corner is not somewhere
  to put a foot down on the way in.
- **`World::repair` reads the allowances going in and ignores them coming out.**
  What counts as *standing somewhere* has to know about the set, or the next
  keypress walks a wading player home out of a lake they were legally in; where
  a repair *puts* somebody must not, because a rim tile is only a place to
  stand while the set is on the board.
- Wading only ever **adds**. `an_allowance_never_shuts_anything` says so over
  every tile of every map, which is what makes the reachability tests still
  mean what they meant.

## The first dungeon

`data/dungeon.json`, nine by five, one way in and one thing at the end of it.
Short on purpose: fatigue is the budget, and the walk back out is part of it.

- **Maps are a list.** `data::MAPS` is `(id, json)` and `WorldState::map` says
  which one you are on. A map this build has not got falls back to the
  overworld rather than panicking — a save can name one, and `World::repair`
  then finds them somewhere to stand.
- Two new `PlaceKind`s. A **Gate** is a way onto another map, and whether it
  opens is answered in the shim rather than in `World`, because it depends on
  what is in the bag and a map does not know about bags. A **Boss** is a
  creature standing on a tile rather than one the ground rolled.
- **A boss drop is looked up by the tile, not the creature.** The same Rust
  Colossus stands in a region's pool; beating one in a field must not hand over
  the way to the next map.
- **Dying in a dungeon walks you home across maps.** It used to leave you
  standing on the boss's own tile, because the walk home looked for a town on
  the map you were on and a dungeon has none.
- `every_gate_leads_somewhere_you_can_stand` is the one that matters: a gate
  whose far side is a wall strands a player on a map with no way off it, and
  nothing else in the game would say so.

### `map()` must not reach for the game

`map` used to look `WORLD` up as a single static. Following the player means
knowing which map they are on, and the first version read `GAME` to find out —
which is a `RefCell` double borrow at nearly every call site, because they are
all already inside `with` or `with_mut`. In a wasm build that is a bare
`unreachable` on the console and **nothing else to go on**.

`map_for(g, …)` takes the game. Where the closure also mutates it, the id is
resolved first and `map_named` is used, because borrowing the game to find the
map and then mutating it inside the closure is the same fault one level up.

## The door in the wall, and the screen that is not a loop

The key from the bottom of the Cave had nothing to open until M8.7. Three
firsts, and each one is a rule:

- **A place can be conditional, and it is still content.** `PlaceDef::hidden_until`
  names an id that has to be in `answered` or `flags`; until then the place is
  not drawn, not steppable and absent from `World::place_now`. Spawning one at
  runtime was the other option and is rejected for the reason the map is not in
  the save: **places are content and content is not state.**
- **`place_at` is the file; `place_now` is the game.** The first answers
  questions about the data — where the last town is, whether the map is well
  formed. Everything a player can see or walk into goes through the second.
- **What a door *wants* is still the shim's.** `hidden_until` reads `answered`
  because a `World` does not know about bags; `needs` names a component and is
  answered where the bag is, exactly as a gate's key is.

`Quest::requires_answered` is the other half: an errand gated on something that
is not another errand. A boss tile writes its own id into `answered` when it is
cleared, so "once the Cave is done" is one field rather than a second kind of
prerequisite.

The ending screen does **not** take the fork's treatment. You can back out of
it, because the world is still there behind you and there is an errand about
the door to hand in. What it must not do is pretend there is more.

## Twenty maps, and where they live

M11 took the map count from two to eleven, and the first thing it had to do was
move the two. **M14 took it to twenty-one** and had to move nothing, which is
the return on that: ten new files in the same directory, ten lines in
`data::MAPS`, and **no map changed shape.** The Treyway grew a south in its own
file — one country, one file — and then gave it back, for a reason that is not
about content at all: see *A country half again as tall as it is wide*. `data/tiles.json` and `data/dungeon.json` are
`data/maps/west-bambulon.tiles.json` and
`data/maps/the-great-gear-cave.tiles.json`; every map is one file in one
directory named for the id it registers under, and `data::MAPS` is still the
list. **Two maps could be two nouns in a data directory. Eleven cannot**, and
the rename was cheaper before the nine than after.

| map | size | what is on it |
|---|---|---|
| west-bambulon | 20×20 | the pit, the only starting town, 2 crossings, the bench, 3 gates |
| the-great-gear-cave | 9×5 | one boss, one way in |
| **the-treyway** | 16×16 | the country behind the door; West Bambulon is one tile of it |
| **kettleworks-field** | 20×20 | Kettleworks, and 41 events — one tile in ten answers |
| **the-drambus-stack-5 … -1** | 10×10 each | five floors, one boss each, one sitting each |
| **under-the-lake** | 13×9 | what the lake was on top of, and the door the demo ends at |
| **the-reach** | 20×20 | the same map every time; what changes is the instrument |
| **the-low-water** | 16×11 | the Treyway's south, over a bar of shingle the tide leaves |
| **the-sump-1 … -4** | 12×12 each | the Wextreen Sump: three puzzles and the Ninth Surveyor |
| **the-silt-stair-1 … -4** | 12×12 each | the Silt Stair: three puzzles and what Marbulon faced away from |
| **the-undercountry** | 20×20 | the country under the country, and one town with nothing in it |
| **the-wextreen-sands** | 18×12 | the second surveyable map, where the needle is no use and the paper survey is |
| **the-reefs-1 … -3** | 16×15, 16×8, 17×12 | the Eleven Reefs: nine stakes, three doors, four sinkholes and the Tenth Surveyor |

- **The Treyway brackets levels twelve to sixteen, not five to nine.** The plan
  asked for five to nine and that number was written before anybody counted the
  road: the door is behind the Cave and the Cave is behind a crossing that asks
  for nine, so the earliest a player reaches it is twelve. A continent
  bracketed below the map it opens off is a continent nobody fights in.
- **A dense map is a different kind of map, and it needed no new code.** The
  Kettleworks field is 41 events on 400 tiles. West Bambulon is 7 on 400. The
  difference is entirely in the file, which is the point of content living in
  `data/`.
  **And that is the whole of what it is, which is the problem.** Asked
  directly whether the field's events hint at anything, the measurement is:
  **six of the forty-seven do something** — they are errand waypoints, a
  `word` goal you stand on — and **forty-one are prose and nothing else.** No
  flags, no choices, no `hidden_until`, no cross-references, no chain. The
  nouns they share are "Stack", "Kettleworks" and "Somebody". There is no
  puzzle in them because none was written.
  A map whose texture is *reading things* is a legitimate content decision;
  forty-one dismissals that pay nothing and ask nothing are texture only if
  something on the map rewards having read them, and nothing does. `PLAN-
  M12-EXEC.md` §M12.5 is where that is answered rather than defended.

## The Kettleworks was a wall, and the wall was the gear

Reported from play: *"the monsters in area 2 the kettleworks are like insanely
difficult to defeat by the time you get there ... the only one I can reliably
kill is the thing in the fortieth kettle due to timing it out."* Both halves
were true, and the second is the tell — **a win at the sudden-death clock is
not a win a board earned.**

**The yardstick is the human's and it is now a test.** Level ten, any class,
**one assembled item to a grid**, on the three-row frames a level ten stands up
in, owning both shelves and everything the errands pay. Generous about gear and
mean about space, which is the safe direction. Measured before anything moved,
that board beat *nothing* in the field: The Curator, the Pale Twin and the
Kettle Wight all ran to the buzzer and both hounds killed it in eight seconds.

- **The body numbers are not the lever, and this is the finding.** Health,
  strength, regen and both resistances scaled to **seventy percent** changed
  not one outcome. Almost all of what a Kettleworks creature *rates* — and all
  of what it does to you — is the gear it wears, which is also what
  `creature_rating` is mostly counting. The Kettle Wight has no weapon at all:
  it is a wall of 82 armour an activation, and cutting its health to 60% still
  ended every fight at the buzzer because the clock was doing the killing
  either way.
- **`gear_offset` is the dial, and its own doc says what it costs to move.**
  *Move one off zero only with evidence from a densely packed profile* — six
  creatures are off zero now and the evidence is written where the dial is. It
  steps every piece down its own footprint family, so a board still packs
  exactly as authored.
- **The dial saturates.** The Hoop Hound's families bottom out at −8% and the
  Kettle Wight's at −9%, so four of them needed a body trim on top to reach the
  band that was asked for. Every creature the field actually deals is down 12
  to 16%.
- **Two creatures were moved rather than tuned.** Lord Drabley Henpeck is off
  the first map entirely and kept for a boss fight somebody has still to place;
  The Rice Criers moved from the first map into the Kettleworks, which is what
  a 471 belongs beside. West Bambulon's deepest region would have been a pool of
  one, so it took Rust Colossus as well — *content waiting for a map is fine
  and a region that deals the same fight for ever is not*.

### And easing a creature makes its neighbour rarer

**The fourth day this project has lost to `draw_enemy`'s weighting**, and the
first time it was self-inflicted. A pool's weight is `(max + 1 − rating)`, so
the hardest member is the rarest — which means **lowering one creature promotes
whoever is left at the top into being almost never drawn.** Twice in one
afternoon:

- Easing The Curator and the Pale Twin made **The Gearwright** the ceiling of
  the first Treyway, and the Drover's Stride comes off it: 25% of draws became
  0%, and `a_set_is_never_behind_the_rarest_fight_in_its_region` said so.
- Easing the **Ruin Hound** made the **Slag Warden** the ceiling of the Kolok
  Downs, and an instrument part went from 25 wins to **647**.

The rule that comes out of it: **ease a pool, not a creature.** A whole pool
stepped together keeps its order and its shares; one member stepped alone
reshuffles who is rare, and what is hung off the rare one goes with it. The
Gearwright came down with its poolmates. The Ruin Hound went back to zero
instead — it is the field's ceiling and is dealt **0% of the time there**, so
easing it bought a player nothing and cost a set its owner.

## The Drambus Stack, and the counter that is not there

Two hundred and ten feet of cheese with a door in the south face. Five floors,
and **the tower comes down as you clear it**: beat a floor's boss and you are
put outside, and the next time you go in it is the floor below, because there
is one fewer above it.

- **How many floors are gone is derived.** `PLAN-M11.md` asked for
  `tower_floors_cleared` in `WorldState`; a boss already writes its own tile id
  into `answered` when it is beaten, so the count is how many of those ids are
  there. A counter would be a second answer to a question the save already
  answers, and the two would part the first time a save was edited. **The tower
  is the levels rule again**: derived from what you did, never stored.
- **Floor one is the bottom and its boss being answered *is* the tower being
  down**, which is the flag the lake reads. No `tower_dropped` field either.
- **A floor is one sitting.** There is no walking out, and a save taken inside
  reopens outside — `leave_the_sitting` in `world.rs`. That is not a
  restriction on the player, it is what makes the fatigue budget mean anything:
  a dungeon you can leave and re-enter at will is a dungeon with no budget.
- `PlaceDef::floors` is how one place declares the stack it is the door to, and
  `World::arrival` answers which one you get. The five files are separate
  because they are five different rooms, not one room with a number on it.

## The lake empties, and it is still one map

When the Stack is down, the lake in the middle of West Bambulon drains, and
there is a ring of cut stone in the middle of it with a grating in it.

- **One map, read twice.** `TilesData::drains` names the rows and the flag, and
  `data::map_now` applies it. The plan proposed a second map variant; a second
  file would have been two places that had to be kept identical everywhere they
  were not deliberately different, which is how a map and its copy drift.
- **And there is an early way in, which costs the walk.** The Toad's Own Frame
  has let you wade since M9; M11.4 widened `Rule::Wade` from *the rim* to *the
  whole body*, so a player wearing the whole set can walk out to the middle and
  go down before the tower falls. The plan wanted the early way to make the
  fight worse; combat has no board, so a position cannot cost anything. What it
  costs instead is **fatigue**: entered early the map's own middle rows are
  still flooded, so the way down is twenty-one tiles of slag against eleven of
  road. Fatigue is the only currency a dungeon here has, and it is the honest
  one to charge.
- **`walkable` widened and nothing had to be re-derived**, because
  `an_allowance_never_shuts_anything` has held over every tile of every map
  since M9.2. An allowance that only ever adds is an allowance you can widen.

## A country half again as tall as it is wide

Reported from play, and the whole of it is one line of CSS that predates the
block:

> *the bottom map in the overworld should be a separate map, accessible in the
> same way as currently via a little land bridge, but not all rendered in the
> same map, cause the resolution for the overworld looks all messed up now*

**`fitMap` sizes the canvas's backing store to the map** — twenty by twenty is
640 by 640, the Cave's nine by five is 288 by 160 — and its own comment says
why: *"a canvas pinned to the larger left the cave floating in a screen of
nothing."* That has been right since M8. What was wrong is the line under it:
`#map { width: 640px; height: 640px }`, a **fixed square in CSS**, so the
browser scaled a non-square backing store to a square box on both axes
independently.

**Every map that is not square has therefore been drawn at the wrong aspect
ratio for as long as there has been a second map.** Nobody noticed because all
of them were *wider* than they were tall — a room stretched to a square still
reads as a room. M14.1 drew a country sixteen by twenty-six, which is scaled
1.25× across and 0.77× down, and it came out crushed.

Two things came out of it and they are different kinds of thing.

**The split is the design call, and it is the human's.** The Treyway is sixteen
by sixteen again and the shore is `the-low-water`, its own file at sixteen by
eleven. *One country, one file* is still the right instinct — it is why West
Bambulon and the Treyway share a note and why the lake is one map read twice —
and it is not a rule that beats a map you cannot look at.

**The bar is a gate now, and it was two tiles of the same grid.** One tile of
`tide` at column 8 on the Treyway's last row, drawn from the first visit,
impassable, and `coast` once the tenth cairn goes up on the Reach. The crossing
stands on it — so the shore is not behind a *hidden* place and not behind a
refusal; it is behind a tile you cannot stand on yet, which is what a land
bridge is. `wading_does_not_move_a_place_or_a_region` had to learn the
difference: **ground the world opens is not ground a set opens**, and the check
it exists for — a place three players in four never find because it is behind a
rule they did not know to build for — is untouched.

**The CSS is the fault underneath, and the split alone would have left it.**
`width: 100%; max-width: 640px; height: auto` takes the ratio from the backing
store, so the Cave is a wide short room rather than a stretched square and the
map under the lake stops being taller than it is. `check_the_tide_is_drawn_
before_it_goes_out` measures it on both sides of the crossing — the backing
store against the drawn rectangle, on a square map and on one that is not —
because **only a browser can say what shape a canvas came out.** Negative-tested
by putting the square back: *"the shore's canvas is 640x640 for a 512x352 grid,
which is a different shape."*

## Down twice, and the country under the country

M14 is eight floors with a puzzle on each, and the whole block hangs off one
sentence that is a property of the engine rather than a taste:

> **`WorldState.flags` and `answered` only ever grow**, so a puzzle whose wrong
> move locks the right one is a puzzle the save cannot come back from.

So every puzzle in the game is solved by **discovering** something and never by
avoiding something. That is not a limit on the puzzles, it is what kind of
puzzles this game has — and what it has instead is the board:
`LooseItemOfSize`, `AssembledOfRarity` and the three survey instruments are
locks a player opens by *packing* and by *reading*, which is the thing the whole
game is already about.

### `puzzle::solvable_blind`, and why it is in core

A floor only a person can solve is a floor `make play` cannot walk, and a walker
that stops at floor two is a gate that never sees floor four. So every floor has
a **blind solution** and this counts it — in core rather than in `tests/`, for
the reason `pressure.rs` is: *a number a design stakes itself on that is worked
out by the thing measuring it is the page recomputing a total, one level up.*

**A visit is one card read**, and the model is a sweep against an adversary that
arranges the floor as badly as it can be arranged: everything that could still
raise a flag is *remaining*, the solver walks all of them, the one that opens is
visited **last**. Nine cairns is `9 + 8 + … + 1`, which is **45** — the number
`PLAN-M14.md` §1.2 builds its whole ceiling out of, reproduced by the model
rather than assumed by it.

Two things it learned the expensive way, and both changed a floor's number:

- **A card on the far side of a channel is not a card yet.** The first version
  counted every event whether or not the solver could reach it, which measures
  the flag chain and not the floor. It re-drains the world at every position and
  floods from the arrival tile now — and on a floor drawn slightly worse that is
  the difference between *solvable* and *the wheel that drains the channel is
  behind the channel*.
- **A card this floor can never open is read once.** The Cairnfield's slab wants
  a golem, or nine heights off a clipboard two hundred paces away on the shore.
  Counting it as remaining nine times over made the field 54.

**And `solvable_blind_with` was written and then deleted**, which is worth
knowing because the idea is obvious and wrong: a blind sweep carrying a golem
comes out at **54** against 45 without one, because the instrument puts a tenth
card on the floor worth walking to. True, and a number that moves the wrong way
is a number somebody will one day quote. The comparison in one unit is
`solvable_knowing(None)` against `solvable_knowing(Some(k))` — nine moves across
the field, or one on the slab.

### An instrument makes a floor short, and it charges in three currencies

`PLAN-M14.md` §1.2 promises *short, never possible*, and the second half is a
lint: `an_instrument_is_never_the_only_way_through` walks the flag graph with
every `Surveying` choice deleted and asks whether every hidden place is still
reachable. **Stated over the place rather than over the choice**, because that
is where it matters — the lintel on the Shelf wants an atlas and nothing else,
and the lintel is not a door.

The first half does not survive as the plan wrote it, and the reason is the
finding: **the three floors charge in three different currencies.**

| floor | blind | what it charges | what the instrument takes off |
|---|---|---|---|
| the Lip | 8 | **12 fatigue** | the compass reads the bearing off the plate |
| the Shelf | 1 | **a component** | the atlas reads the shorthand and sets the catch |
| the Cairnfield | 45 | **forty-four extra card reads** | the golem stands on the slab |

Only the third moves the count. The plan's *1 / 1 / 9* survives there and
nowhere else.

### The Wextreen Sump

Four floors down a hole eleven feet across on the Treyway's new southern shore,
behind a gate that wants an instrument the way the Reach's edge does.

- **The Lip.** Three channels, three wheels, and the wheels sit *below* the
  channels they open in the order you meet them. **One of the three is not
  needed**: channel C is walked round at its dry east end, so the wheel that
  opens it buys a crossing that is already there — and it is the only wheel in
  the game that keeps what you feed it. The floor is a tax on not looking.
- **The Shelf.** One door on a counterweight, three ways through: leave a
  three-by-two in the slot, read the lintel with an atlas, or stand something
  **epic** in it. Epic and not rare because the board a player actually has
  holds exactly one epic item and twenty commons.
- **The Cairnfield.** Nine heaps, **one title between them**, and the digit is
  on neither the map nor the place ids: which surveyor's heap a cairn is, is the
  puzzle. The nine heights are distinct and the ninth clipboard on the shore
  lists them in surveyor order, so a person can solve it by hand. **A cairn's
  refusal names a cairn and never which one** — TONE rule 12 and the block's own
  *a cairn does not say which cairn is under it* meeting in the same sentence.
- **The Sump Floor.** The Ninth Surveyor, on an island in the water.

### The Silt Stair

Behind the door under the lake, which is a **way on** now: *nothing is behind
the door* was true and is not, and the two paragraphs saying nobody had decided
went with the writing, one map further down.

- **The Landing.** The stair is cut one wide and four long. Lay something that
  shape in it and it keeps it, or carry down The Cracked Lens — which is
  Sootmother's own drop, so somebody who came through the lake the intended way
  is already holding the key, and the manifest on the wall says so in a hand
  that stops after *a lens, cracked*.
- **The Chair Room.** One chair facing a door, and **the only repeating event in
  the game**. Three moves in an order: face, four, back — which is what Marbulon
  does in front of her own door on the first map, and has done since M8.
- **The Drowned Gallery.** Two chains: A floods it and B drains what A made, and
  B will not move in a dry room and says so. **The first drain in the game that
  takes ground away**, and what makes that safe is that the one after it in the
  same list gives it back — flags only grow, so somebody who pulled A can always
  pull B. The stair was under the floor the whole time.
- **The Bottom of the Bottom.** What Marbulon Faced Away From, on silt.

**`no_homeward` on all four**, for the lake's own reason one map further down:
it is the one place where the walk *is* the content, and a set that posted you
out of it would delete what the two hundred and six steps cost.

### Neither is one sitting, and that is a different budget

The Drambus Stack is one sitting because its budget is **fatigue**. These two
are not, because their budget is **what you brought**: no floor names an
`outside`, every floor has a stair back up, and a save taken on one reopens on
it. A player who needs a three-by-two from the van can go and get it, and what
that costs is the walk.

`PlaceDef::floors` is what makes going back in land on the first *unsolved*
floor — and `Floor::cleared` reads `marks()` rather than `answered` now, because
three of the Sump's four floors have no boss at all and are done when their
puzzle is solved, which is a flag.

### A rating predicts nothing about whether a fight is winnable

The recon both new bosses were dressed against, measured against
`common::geared_from` — this repository's answer to *the board a player actually
has* since M11.7:

| it beats | it loses to |
|---|---|
| Sootmother 1670, Anvilheart 1803, The Last Light 2031, **Francis 2958** | **Cairn Chorus 1141**, The Tallow Saint 1223, The Ground Floor 1507, Gilt 2489 |

It beats a 2958 and loses to a 1141. What decides it is **damage per second**:
Cairn Chorus deals 206 and kills it in thirteen seconds, Sootmother deals 19.7
and loses at the buzzer.

**And what a creature deals is mostly how many items its board makes.** Two
drafts of two bosses were wrong the same way and neither was about a number:

- The Ninth Surveyor's first weapon grid was a hilt and two accessories, which
  **assemble nothing**. She dealt 7.8 damage a second — and exactly 7.8 at
  strength 152 and at 320, because strength pays a swing and there was no swing
  to pay. A sweep of seven healths against four strengths came back Victory in
  all twenty-eight, which is the Kettleworks finding again: *the body numbers
  are not the lever.*
- What Marbulon Faced Away From's first board made **three** items where the
  other makes eight, because a hilt, a key and a charm in one row touch and
  merge.

**The coordinates are the dial and the piece names are the costume**, which is
upstream's *monsters wear the catalogue* arriving from the other side.

### The Undercountry, and a town drawn honestly

Twenty by twenty, three ways in, one town, and the town is **empty**: no shelf,
no errands, and a screen one tile south of it that says the writing stops here.
It is not a placeholder drawn as a town; it is a town drawn honestly, and what
goes in it is the next plan's.

Three lints refused it, all correctly — a town on a map sells something, wants
something, and is not the same shop as another. `common::UNWRITTEN` is the
mirror of `avail.rs`'s `STAGED`: **a shelf with no ground under it and ground
with no shelf on it, and both are fine only because somebody wrote the name
down.** Every one of the three exceptions is *asserted* rather than skipped — an
unwritten town that quietly grew a shelf is a list that has gone stale.

**The two doors at the two bottoms are `needs_all` and Marbulon's is
`hidden_until_all`**, and the difference is which side you are standing on. A
door at the bottom of a dungeon that is not there is a room you walk out of
thinking the dungeon ended; a door in a shallows on the starting map that is
there from the first afternoon is a secret with a signpost on it. So the first
one you finish shows you a sealed door with the other dungeon's name in the
refusal — `Game::sealed_because`, two registers on one line, `shut` from the
map file and *which dungeon is still standing* derived off the places so it
cannot go stale when a boss is renamed.


## No board a player can build reaches Rare

**The largest thing M16 found, and it is a measurement rather than a bug.**
`PLAN-M16.md` §4.2 hangs the Assay's first door on `AssembledOfRarity("epic")`
and its third on `"legendary"`, on the strength of M14's note that *the board a
player actually has holds exactly one epic item and twenty commons*. That note
is about the Shelf and it is not what the numbers say.

| board | best item | rarity |
|---|---|---|
| the run, as the human left it | 50 | Common |
| the run, Auto-packed out of everything it owns | 50 | Common |
| `geared_from` — both shelves, every errand | 135 | **Epic** |

`RARE_AT` is 90, `EPIC_AT` 130 and `LEGENDARY_AT` 170, and **562 of the 568
components rate Common on their own.** The thresholds *are* reachable — the best
item on the whole creature ladder is Francis's at 343, and 76 creature items are
Rare or better — because a creature's board is packed with the best components
in the game and a player's is what they could buy.

So a door at legendary is a wall with a sentence on it, and a door at epic
refuses a **level-45 caster board** while passing a level-ten shopper's. The
Assay asks in footprints and money, and keeps `epic` on door one as the
*shortcut* rather than the lock. `no_board_a_player_can_build_reaches_rare` is
the lint, and it exists so the next door is not written against a tier nobody
reaches.

**The question underneath it is the human's**: `Rarity` describes creature
boards and not player boards, because `item_rating` prices pieces and cadence and
an Ink is a multiplier that rates nothing on its own. Nothing in the game turns
on it today — the Shelf's shipped epic door is passable by `geared_from` — and it
is the kind of thing that is wrong the day somebody hangs a fifth door on it.

## A lane you can commit to is never one you can be shut out of

`RESIST_CAP` has been 95 since the fork, with its reason written beside it:
*enough to matter against a build that has committed to one resistance, and
never enough to make committing pointless.* `MIND_CAP` was a hundred, with the
opposite reason: *those two lanes can be shut out completely, and full immunity
is a thing a creature is allowed to have.*

Both were true, and the second was written when **nothing in the game was built
on either lane**. Then M16 measured it:

- **twenty-three of the sixty creatures** are at or past a hundred curse resist,
  base plus gear, up to Nine of Ashes at 233;
- **thirty-one** are at or past a hundred mind resist, **every deep boss among
  them** — What Marbulon Faced Away From at 128, the Ninth Surveyor at 126, The
  Rust Parliament at 200.

At a hundred, `landing_ms` returns **zero milliseconds** and
`mind_damage_after_resist` returns **nothing**. So the Whisperer — a base class
whose whole promise is the mind lane — and four curse-shaped experts would have
dealt exactly nothing at the bottom of every dungeon in the game. That is
`every_offered_class_reaches_something` failing one level up: a class that
reaches something against a rat and nothing against the fight it was bought for.

**`stats::LANE_CAP` is 95** and nothing under ninety-five moved, so no creature
was retuned. `PLAN-M16.md` §5.3 asks for the two M14 bosses to be brought under
the cap by **re-dressing**; the recon found twenty-three rather than two and
neither of the plan's numbers, and re-dressing a quarter of the ladder to move a
constant is *ease a pool, not a creature* two levels up.

- **`no_creature_is_immune_to_a_curse_or_a_whisper` calls rather than
  declares.** It asks `landing_ms` and `mind_damage_after_resist` with each
  creature's own summed resistance, over all four curse kinds, rather than
  reading a number off a sheet. Broken by putting the cap back, it names **124
  shut creature-lanes**.
- **Three unit tests pinned *fully resisted, never lands*** and were updated
  rather than deleted, each with the reason where it stands. A test that pins
  the behaviour you are changing is a test to change in the commit that changes
  it.
- **`MIND_CAP` keeps the job it is actually right for**, which is piercing and
  hardening: a hundred percent of those means something.

## Quicksand, and why a floor that opens can be monotone

M14's rule is that `flags` and `answered` only ever grow, so a puzzle whose
wrong move locks the right one is a puzzle the save cannot come back from. The
Flat Below wants terrain that *closes* — that is what sand does — and it cannot
have it.

**`quick` is the terrain that can only be opened.** Impassable, rolls nothing,
and `a_quick_cell_only_ever_drains_to_silt` refuses a drain that makes it, over
every map there is. Nine stakes each open one segment of one band and pulling all
nine opens everything, so a blind walk can never strand. What it costs is
forty-five fatigue — three-quarters of `CAP`, asserted against it — and six
fights in pockets that lead nowhere.

**The compass is offered at every stake and sets nothing**, which is a lint
rather than an oversight: the Sands' own prose says there is iron under it and a
compass will tell you about every reef at once. *A lie that paid would be a
hint.*

**And the blind ceilings are in card reads, not in moves.** `PLAN-M16.md` counts
*pulls* and *drops*; `puzzle::solvable_blind` counts card reads, which is M14's
unit and the unit the Cairnfield's forty-five is in. The Flat Below is **36**
against the plan's 9 — thirty-six rather than the eighteen a first estimate
gives, because the adversary opens each band *early* and leaves its two wrong
neighbours live while the next band's three arrive. The Assay is **3**, exactly
as written. The Needle Room is **7** against 8.

## A warp is a lock, and an alcove is two tiles

`PROMPT-M16.md` asks for it to be reported first if `Outcome::Warp` cannot land
you in a walled alcove. **It can.** What could not see it was
`puzzle::reachable`, which floods terrain — so the Needle Room, whose four
alcoves touch the ring only on the diagonal, measured `Stuck`. It follows a warp
now, **on an unconditional choice only** — whether a gated one can be taken is a
question about a `Position` that function has not got — and **only onto the map
it is already on**, because a warp to another map is a way out rather than a way
across.

What nearly shipped instead is a lever nobody could ever pull. **`warp_to`
resolves no place on the landing tile**, and it is right not to: every other warp
in the game lands you on ground. So a one-tile alcove with the lever *on* the
tile you fall onto is a card that opens for nobody — you land, nothing opens it,
and there is nowhere to step off and back on. `an_event_that_asks_something
_still_reopens` caught it looking for a walkable neighbour and finding none.

**An alcove is two tiles**, the one you land on and the one the lever is on.
Fixed in the map; `warp_to` is untouched. The general shape is worth keeping:
there are three kinds of arrival in this game — a step, a gate and *being put
down by something that is neither* — and only the first two resolve a place.

**And the sinkholes do not repeat**, which `PLAN-M16.md` §4.3 says they should.
`a_repeating_event_may_never_pay` refuses a repeating warp, for a reason that is
still right. They do not need to: both ungated levers sit behind a true
sinkhole, each lever drains the way out of its own alcove, so each hole is
needed exactly once and no order strands anybody.

## A creature wearing a real player's board

The Tenth Surveyor is thirty-eight components in the cells the human seated them
in and all six of that character's enchs — `MonsterSpec.enchs`, and the first
creature in the game to carry one. `the_tenth_surveyor_wears_the_run` compares
her board against `common::from_save`'s by name and count per slot.

- **`MonsterSpec.items` is a chunk list, so the gear array is in *item*
  order** — which a board's placement order is not. The run's first helmet item
  is board entries 0, 3 and 5 and its second is 1, 2 and 4.
  `Character::item_partition` returns the pair, because *the boss's `items` field
  is not a number anyone should type* is true of the order as well.
- **An ench lands on the profiles and nowhere else**, through `ench::apply` —
  the door the player's go through, because two answers to *what an ench does* is
  the mistake this project has paid for six times. A creature gets no `enched`
  flag and no beacon: those are read by rules a character holds.
- **`data/enemies.json` went lossy the moment a creature carried one**, and
  `the_bestiary_file_reads_back` passed anyway because `EnemyData` had no such
  field. It has one now, added in the commit that made it necessary.

**Her numbers are found, and the plan's method could not have found them.**
`PLAN-M16.md` §5.2 asks for a win rate between 55% and 70% *over a loop of
seeds*, and **combat has no RNG** — a loop over seeds counts the same fight every
time, which is why a mid-fight save carries a creature name and a tile. What
varies between two players meeting her is the **board**, so the bracket is over
boards. Against `common::geared_from`:

| | deals | |
|---|---|---|
| The Ninth Surveyor | 122.1/s | Victory |
| What Marbulon Faced Away From | 138.7/s | Victory |
| **The Tenth Surveyor** | **221.2/s** | **Victory** |
| Gilt | 428.3/s | Defeat |
| Nine of Ashes | 531.0/s | Defeat |

Her first draft was 236 strength and dealt **807.9/s**, which killed that board
in three seconds. **Health barely moves a fight at this depth and strength is the
whole dial**, which is the Kettleworks finding a third time: across a sweep of
5,200 to 8,800 health not one outcome changed, and across 40 to 100 strength it
went from a win at the buzzer to a loss at fourteen seconds.

**And the run loses to her.** `geared_from` is twice the board the reconstruction
is — 1,942 health and 21 items against 974 and 11 — and beats both M14 bosses
while the run beats neither. The run's gear is cheap early components packed into
a caster board, so *"an almost finished run that is very powerful"* is not what
§5.1 transcribes. The block is bracketed against the board that can actually
reach her.

## The two classes GM2D wrote

Every class before M16 is upstream's with the theme talking, which was the right
call while the powers were already tuned and already tested. The **Kettle-Stoker**
and the **Whisperling** are the first two written here, and neither has an
upstream to borrow from.

- **The Stoker buys empowerment with somebody else's pool.** A held pool pays a
  standing bonus and mana empowerment scales off the mana left, so stacking it
  hard drains the very pool it multiplies. The Stoker burns rage, faith or
  nature instead — **the largest one**, which is what makes the class a decision
  about which hopper to fill rather than a number that goes up.
  `pools_worth_holding` was checked rather than assumed: it returns exactly
  those three.
- **The Whisperer makes the mind lane a way to finish.** `Combatant::is_down`
  has fired at `max_health <= 0` since the fork, so a mind build could already
  kill anything in principle and in practice never did. The class moves where
  the line is rather than inventing a second way to die — and it is only true at
  all because of `LANE_CAP`.

**A base class has knobs now**, which is the widening M16 needed and the first
thing it found: `Effect::Tunes` was checked against `ExpertPower::knobs` and
refused everything else, so a tree could not tune its own class's power.
`ClassPower::knobs` and `::tune` are the fix, and `Character::class_defs` tunes
a base power the way it tunes an expert's — **M13.6's thirty-eight dead nodes,
one level down**.

- **And widening it opened a collision the old reader could not see.**
  `per_stack` is the Stoker's *and* the Patented Funnel's, so a character who was
  both would have had each tree's tuning applied to the other's power — exactly
  what the parse-time check exists to make impossible, arriving from the other
  side. `tunings_for(class, taken)` is scoped to the tree; `tunings_from` stays
  for the callers that mean all of them.
- **`ExpertPower::step` read the knob's *name* and needed to read the power.** A
  knob ending `_ms` is printed in whole seconds by the two experts it was written
  for and to a **tenth** by the four furnace ones, where eight hundred
  milliseconds is 4.0s becoming 3.2s.

## The tree could not grant a pool, and `Stat { mind }` was dead as written

`Effect::Stat` read five fields and `Effect::StartWith` two, so
`stat { nature: 20 }` and `start_with { insight: 20 }` would have parsed, cost
points and changed nothing — the *eight skill nodes* failure, caught by
`every_effect_key_is_one_the_engine_actually_reads` before it shipped. `Stat`
grows `mind`; `StartWith` grows rage, faith, nature, insight and dread, which is
what *what you are already holding when the bell goes* has always meant.

**And `Effect::Stat { mind }` was dead the moment it was added.** Mind damage is
read off `ItemProfile::stats.mind` and a character-level `Stats.mind` reaches no
item. It cannot be read off `player_stats` either, which sums every item's stats
and would pay the board's own mind damage once per item. It travels through
**`Held::mind`** — the one door for *what the character contributes, once, beside
the item*, which is where the tree's armour, its mana and its granted rules
already go. Found by `every_point_in_an_expert_tree_buys_something`, which is the
lint that exists to find exactly this.

## Twenty-one experts, and what it cost to make every point buy something

`C(7,2)` is twenty-one and the table held ten. The eleven grant three new rules
between them — `burn_keeps_bonus`, `burn_carries`, `mind_pierce` — and **each is
granted by more than one tree**, which is the shape `Spread` and `Beacon` already
have: a rule granted by exactly one tree is a knob that has been given a second
name.

**An expert's power is self-contained**, and six of these would not have been. A
Fired Funnel's promise is *every stack the furnace buys is also mana*, and a
furnace is the Stoker's — so without one the whole tree read dead.
`light_the_furnace` gives all six their own; you cannot hold one without being a
Stoker, so it is belt-and-braces, and it is what lets the lint ask its question
of the expert **alone**. Giving the *fixture* both parents instead was tried and
buried five of the original ten under two parent powers' worth of noise.

The lint then found five more things and every one of them was real:

- **three tuning defaults chosen so the knob never bound** — Bare Furnace's
  `cap` was never reached by two bare frames' worth of stacks, Fired Funnel's
  `per_fight` was never spent, and Told Once's `per_curse` and `cap` were **raw
  health points** where the plan means percentage points of a maximum;
- **`purse_sweep` could not see a two-second move**, so Flash Powder's window at
  ten and at twelve were the same string — the *compares zero with zero* failure
  with a stopwatch in it;
- and **a board that deletes a maximum in one blow cannot watch a threshold
  move**, so Told Once's board takes two of the Whisperer's nodes rather than all
  nine. The general form: *a fixture strong enough to win instantly is a fixture
  that measures nothing.*

**`every_offered_class_reaches_something` grew a fourth arm**, and it had to:
M16's two are the first powers on the fork that change nothing a fighter *walks
in* with — a furnace runs at the tick and an unmaking moves where a fight ends —
so the three places that power could be honoured were all the same on both sides
and the class was still real. The honest question for those is *did the fight
differ*, which is what `experts_reach.rs` has asked of the experts all along.

**The fork draws seven cards in two rows of four and three**, measured in a
browser at 1280×720, and still refuses Escape. Three tracks would have wrapped
seven as 3 / 3 / 1 — three rows, the last of them one card wide, on a screen
whose whole job is to be compared across.

## The Wextreen Reach, and reading a map through what you carry

At the north edge of the Treyway the plain stops. With an instrument assembled
you can go in; without one there is nothing to read it with, and the refusal
says so.

**It is the same map every time.** What changes is the instrument, and the
whole of that lives in `crates/core/src/survey.rs`:

    survey::mods_for(map, kind, items_assembled) -> SurveyMod

- **A pure function, and nothing about surveying is in a map file.** An
  instrument is the character's and a map is the world's — the identical
  division `Allowances` makes for a crossing, and the reason a map still does
  not know about bags.
- **The compass quiets the map** (−20%, and −3% an assembled item down to a
  −45% floor), **the atlas pays** (+120‰ drops, +40% experience) **and is
  louder for it** (+10%), and **the golem handles one fight an entry**.
- **The golem's fallback was taken, and the plan named it in advance so that
  taking it would be a decision.** It was to be a third board in the replay;
  that is a third set of numbers the page must not invent, and the honest
  version is a third combatant in `combat.rs` — new combat code in a block that
  added none. One fight an entry is the version that is true.
- **`items_assembled` is in the signature because the compass reads it.** The
  quiet scales with how much of a board you gave up to carry an instrument,
  which is the trade the whole system is about.

## An instrument has a frame of its own

**It used to take the sword arm, and that was the wrong trade.** Three recipes
— compass, atlas, survey golem — were appended to the Weapon slot, and a weapon
grid held gear or an instrument and never both. The cost was deliberate,
written down in `PLAN-M11.md` §8 row 4 and defended here for two blocks.
Reported from play:

> *"the implementation for the surveying should not require a weapon ... it
> makes any fight you would reach on the other side impossible"*

Which is the answer. **What is through the Reach is a map you have to fight
on**, so a cost paid in your only weapon is not a cost, it is a wall — and the
one thing this project keeps learning is that a wall and a price look identical
from a chair until you are standing at it.

- **`SlotKind::Instrument` is a sixth grid and is deliberately not in
  `SlotKind::ALL`.** That exclusion is the whole design. `ALL` is the gear a
  character wears, and thirty-one places walk it to ask what the boards are
  worth — `combat_items`, `total_stats`, `pressure::of`, Auto-pack, the packing
  screen. Every one of them is right about an instrument **without being
  touched**, because an instrument is a tool and not gear. `SlotKind::EVERY` is
  for the two callers that mean all six: building a loadout, and writing one
  down.
- **So `Character::instrument` is asked separately**, and `rules()` adds its
  `Rule::Survey` from there. `item_rules` walks `reports`, which walks `ALL`,
  which no longer reaches the frame — the same division everywhere else in the
  change: what a board is *worth* never counts the instrument, and what reads a
  map is only ever the instrument.
- **`RuleError::MixedGrid` is gone**, along with the sentence about what
  surveying costs you. The grids do not mix because they are two grids;
  `PieceDef::fits` is the whole rule now.
- **One instrument, and it is stated rather than drawn.** The frame is six by
  three and a golem is twelve cells, so one is what it holds comfortably — but
  two compasses are ten cells, and geometry cannot enforce a rule when the
  largest instrument is bigger than two of the smallest. `Character::instrument`
  answers with the first, **and the screen prints which**, because a player
  carrying two must not be left to guess. Nothing grows the frame:
  `resize_boards` walks `ALL`, and no node or errand names it.
- **`PieceKind::Shard` is still deliberately not a core.** `is_core` is the
  *item-split anchor*, and it mattered more when the two shared a grid — but it
  is still what keeps a frame holding two instruments from splitting on a shard
  in a way nobody authored. The exclusion is commented where it is made,
  because the next person to add a `PieceKind` will read that list.
- **`Orb` and `Alignment` fit both grids**, which is the reason they were
  reused rather than invented: a cosmic orb in a ball is a crystal ball and one
  in an atlas is an atlas. `shared()` still walks `ALL`, so they keep the
  weapon's hue and a map shard is not "shared" — it has exactly one home.
- **The hue is the widest gap left on the wheel, not an Okabe-Ito colour.** The
  palette's two unused entries are orange and blue, three hundredths from
  greaves and fifteen thousandths from helmet; neither is a channel. The
  instrument takes 0.732, the middle of the largest unused arc, and its own
  motif — a compass rose. It is the one grid never drawn beside the other five,
  and separating it properly cost nothing.

### A save written before the frame existed

**`Slot::place` does not validate**, because the loader hands it what the file
says — so a character who had built a compass would have opened with map shards
stranded among their blades: cells taken, no instrument granted, and no screen
saying why.

`Character::repair_boards` lifts anything out of a grid it does not belong in
and returns it to the bag. It is the board's `World::repair`, and it is the
same rule: **a field carried across a build change is a field that will arrive
wrong, and the loader is where that is caught.** What comes out goes to the bag
rather than to another grid, because where a component belongs is the packing
screen's question and it is still owned.

**No seam.** Nothing moved the catalogue — a component changing which grid it
goes in does not change its name, and `catalog_fingerprint` hashes names. A
file that names five boards gets a sixth at the height a player's frames are.

### The door is a bench, not a wall

**A gate that wants an instrument is the only shut door in the game whose
answer the player may already be carrying the parts for.** So it opens the
frame instead of printing a refusal — asked for in as many words: *"you are
shown a screen with a single gear slot, which you must build the compass and
other mapping based items within"*.

- **It is a board like any other**, so it is the same `Board` class driven by
  the same exports, handed one grid instead of five. `boards_json(kinds)` is
  one payload builder for both screens, because a second would be a second
  answer to *what is on a grid*.
- **`Board#slotOrder` is derived from the payload now.** It was five names
  written out, which is a second copy of what grids exist; `SLOT_ORDER` is a
  display *preference*, and anything the payload carries that it does not know
  about is drawn after rather than dropped.
- **`world::here` is standing still and letting the door answer again.** The
  refusal leaves you on the gate's own tile, so repeating the step you were
  turned away from walks you *past* it along the row. `here` reports the gate
  and nothing else — no roll, no tile counted, and no town, event, boss or
  bench re-running, because those happen on arrival and have already happened.
  A gate is the one place whose answer can change while you stand on it,
  because the answer is a question about you.
- **The screen states the trade in core's numbers.** `kit_reading_json` runs
  `survey::mods_for` against the map through the door, so what it promises is
  what that map will actually be read with.
- **`id="kit"` was already taken.** The pack in the map panel has it, and the
  new screen took it too — so `walk()` saw a screen that was not hidden and
  refused every keypress, which reads exactly like a frozen game. *Do not reuse
  a name* has cost this project a `.card` collision, a `.tabs` collision and
  now an id; the screen is `#instrument`.

---

# Part two — the board, and the fight it runs

## Things the fork learned the expensive way

Three facts about the engine that are invisible until they cost you a day. All
three were found by the golden fixture in M0, and all three are fields M1's
save file has to carry.

1. **`Loadout::locks` is state, not geometry.** Two pieces that touch are one
   item unless a lock says otherwise, and which locks exist depends on the
   order the player built in. Re-deriving them gives a different board: the
   first fixture rebuild came back with more items than it went in with.
2. **`Loadout::name_seed` seeds the name hash.** Drop it and every stat
   survives a round trip while every item is renamed — "Resonant Sliver" comes
   back as "Resonant Thorn" and nothing else looks wrong.
3. **`PieceId` is an index into `PieceRegistry`.** The registry is saved whole
   and in order, by canonical catalogue *name* — never by catalogue index,
   which is only stable while catalogue order is.

And one from upstream, inherited deliberately: **lock each item as it
assembles, not once at the end.** A finished board is packed to within a cell
or two of full, so deriving items in a single pass at the end asks which pieces
are connected and gets "most of them". `share.rs` learned this when nineteen
weapon pieces came back as one item.

## Inherited on purpose — do not "simplify"

No RNG in combat. 50 ms ticks. Monsters are loadouts wearing catalogue pieces.
The naming system. These are the reason to keep the engine.

## The fight

- **Combat has no RNG**, which is why a mid-fight save carries a creature name
  and a tile and nothing else. `PLAN.md` §6 proposed storing the pre-fight state
  and the seed; the engine made both unnecessary.
- **The page decides nothing about the board.** The green fit preview *is*
  `legal_anchors` rendered. `testing/drive.py` picks a piece up and compares
  what the board painted against what core returned, so a page that started
  computing its own answer would be caught rather than trusted.
- **The auto-pack button seats only what you own.** It briefly handed out any
  missing component, which made it a supply of free gear and the shop
  pointless.
- **A loss pays nothing and walks you home.** Visible in play now, not just in
  `reward.rs`.

## Auto-pack packs what you own

**The bug M8.8 found by playing, which every test was green through.**

Auto-pack seated a fixed list of twenty-two component names and skipped
anything not owned — and it took that branch only when the weapon frame had
reached eight rows, which is level twenty-something. Below that it seated the
two-piece *starting kit*. So for essentially the whole game, a player who
pressed the button they were given with a bag full of gear got an Oak Handle
and an Iron Blade.

Even past the gate, five of the eleven things the only town on the map sells
are not on that list. With every component the map can hand out, the board came
to **two assembled items of five frames** — and two items lose to the Cave's
boss, so the key never dropped, the door never appeared, and the demo could not
be finished.

The list was not wrong when it was written. It was written against a starting
kit of eleven components and outlived it by three milestones. **A list of names
is a second copy of what the shops sell**, and the two drifted the moment the
shelves became content.

What replaced it, and why each part of it is there:

- **Seed on a core, grow what improves.** Two components that touch are one
  item, so *packing more is not packing better*: a seven-row weapon frame
  packed solid is one enormous group that assembles nothing at all. The first
  rewrite did exactly that and handed a character carrying every reward on the
  map a weapon frame full of books and no weapon — it lost to a Cave Rat.
- **Every placement after the seed must strictly improve** `(items assembled,
  what they rate)`, and is taken straight back out otherwise.
- **A seed that led nowhere is taken back out too.** A lone core is a component
  doing nothing in a cell somebody else could have used.
- **Deterministic.** Best-rated first, ties by id. A seeded walk that repacked
  differently on two machines would be a seeded walk that fought different
  fights.
- **It is not an optimiser and must not become one.** The whole game is the
  arrangement; a button that packed perfectly would be a button that played for
  you. What it has to do is leave nothing obvious in the bag.

`tests/common/mod.rs::build_full_loadout` is the old arrangement, kept as a
**fixture**. Four tests wanted "a known full board" and were reaching for
Auto-pack to get one; they are about recipes and about `hit_for`, not about the
button, and sharing one list meant a change to how the button packs broke tests
about what an assembly bonus does.

## Board pressure, as a number

M12's thesis is that cells outnumber pieces, so a board reads as inventory
space rather than as a puzzle and nothing you pick up ever costs you something
you already had. That was an argument until M12.0 measured it. It is
`crates/core/src/pressure.rs`, `testing/playthrough.py` prints it at every
level, and the baseline is `testing/transcripts/m12.0.txt`.

- **Two numbers, and they are a pair.** **Fill** is cells under something;
  **bench** is owned components that fit nowhere at all. High fill alone is a
  board that never grew; a full bench alone is a bag of things in the wrong
  slot. Tension is both at once — no room, *and* something worth making room
  for — so nothing reduces them to one score, because a single figure hides
  exactly the case the block is trying to produce.
- **Bench depth is core's answer, not the walker's.** `Character::fits_anywhere`
  is the rulebook; the probe prints it. A number a design stakes itself on —
  M12.3 stakes one on bench depth — that is worked out in Python by the thing
  measuring it is the page recomputing a total, one level up.
- **Every turn, not the one the piece happens to be wearing.** A component that
  would seat if you turned it is not benched, because the player turns it. That
  is why `Slot::can_place_shape` and `Character::can_equip_shape` exist:
  answering by *rotating* the piece four times mutates a registry and pushes
  four undo entries to answer a query. The shape-taking pair is for asking
  only — `equip` goes through `can_equip`, which reads the shape the piece is
  actually in, and a test says so.
- **A quest item is carried, never benched.** `can_equip` refuses
  `PieceKind::Quest`, so a tally of toad eyes and two keys would otherwise sit
  in the number for ever — and it would climb every time an errand was taken,
  which is the one metric the block stakes a decision on moving for the *right*
  reason. The exclusion is in `pressure::of`, where the meaning is, and not in
  `fits_anywhere`, whose honest answer about a toad eye is still no.
- **Fill is cells, not components.** A four-cell blade fills four and a ring
  fills one; the difference between a packed board and a tidy one is entirely
  this.
- **An enchantment is under the grid and is not counted.** Gear sits on top of
  it, so it takes no cell away from anything, and counting it would report a
  board as full whose every cell is free.
- **The curve lives in `pressure::target`**, so the engine and the close-out
  cannot disagree about what was aimed at. They are **targets, not
  assertions** — today's game does not meet them, which is the premise of the
  block, so nothing there fails a build.
- **Pieces are counted by source** — shelf, barrel, commission, event, drop,
  quest. Three faucets open in M12 and they can mask each other; a claim that
  the curve moved is a claim nobody can act on if it cannot say which tap did
  it. Attribution is by what the bag gained around a known action, in the
  walker, because *where a component came from* is not something the engine
  should carry in a save for ever to answer a question only a probe asks. A
  nonzero `elsewhere` is a gap in the probe and is reported as one.

**What it found on its first run, and two of the four were not in the plan:**

1. Fill never passes **51%** before level fifteen, against 70% by three.
2. **Fill falls as you level** — 43% at five, 37% at eight. `PLAN-M12.md`
   called a scheduled row "dilution on a timer"; this is that sentence as a
   measurement, and it is the strongest evidence in the block for M12.3.
3. **The greaves grid is 0% for fourteen levels.** Not thin — empty. That is a
   fifth of the canvas contributing nothing for the whole playable game, and it
   is not a content shortage: the pit sells two greaves-capable pieces and six
   of the nine sets drop greaves. Auto-pack never seats one, because a Mold
   without a Material assembles nothing and the shared Materials go to the
   gloves first.
4. **Bench is 0 for fourteen levels**, so the decision this game is made of has
   never once been posed. It reaches ten at fifteen, when the Drover's Stride
   drops.

And **events pay 0 components**, which is M12.5's premise confirmed rather
than assumed.

## What you are about to fight

Only the Cave Rat has innate attacks. **All forty-nine other creatures fight
purely out of their gear** — so a fight screen that printed a name and a rating
was hiding the entire fight. `encounter_json` had carried the creature's item
names since M1 and the page rendered none of them.

- The creature's cards come off the same `item_card` in `crates/wasm` that the
  player's do, and render through the same `cards()` in `app.js`. Two copies
  would be two answers to "is cork a standing stat", which is the question the
  two halves exist to settle.
- `web/theirs.js` draws its board read-only. It imports `paintMotif` from
  `board.js` rather than reimplementing it — the motif is the *shape* half of
  the colourblind triple-encoding, and everything that draws a cell must draw
  the same one.
- **Every relative import in every shipped module is stamped, by pattern.**
  The stamping was a list of module names written out by hand, and both times a
  module was added it was left off — `theirs.js` first, then `shape.js`, each
  importing `board.js` two hops from the entry point, which is exactly where a
  stale mix hides because the page itself looks fresh. `package-web.sh` now
  rewrites every `from './x.js'` it finds and **dies if any bare import
  survives**, which is the check that catches the next one rather than the last
  one.
- `#made` holds two panels now, so anything querying `.made-item` must scope
  itself — an unscoped query lit a creature's card when you pointed at your own
  blade.

## Watching a fight

- **Both boards tick.** Only the Cave Rat has innate attacks, so for the other
  forty-nine a replay showing one side's cooldowns was showing half the fight
  with no way to tell which half.
- **Rows are HTML, bars are canvas.** A row you can point at is a row the
  browser can tell you about; 11px canvas text can be hovered by nothing. Same
  lesson the item list learned when it came off the board canvas.
- **Nothing is computed that the log reports.** Armour comes off
  `Hit::target_armor` and `GainArmor::total`; the four pools come off
  `GainResource::total`, `GainMana::total` and every spend's `remaining`. This
  is the health bug generalised — that one subtracted `damage` from its own
  total and ignored `absorbed`.
- **The armour bar wraps, it does not clamp.** Lifted from the original with
  its reasoning: the two bars read as a pair because they are the same
  measurement, so a full armour bar is as much armour as you have health and a
  pixel is the same number of points in both. Past full each complete bar is
  another layer drawn darker than the one under it. Clamping made every amount
  from "exactly enough" to "four times over" draw an identical bar.
- **The armour label is haloed, not coloured.** The ground under the middle of
  that bar is whatever layer the wrap landed on — the palest shade and the
  empty track are both possible under the same text, and no single ink reads on
  both.
- **The replay panel draws on its own dark ground and uses its own ink**, the
  same as the board. Taking the page's ink put dark labels on a near-black
  panel every time the viewer was in light mode.
- One `oneCard` in `app.js` renders an item for the packing panel, the
  creature's panel and both sides of the replay. Four places, one answer to "is
  cork a standing stat".

## The furnace reached nothing, and nothing drew it either

Reported from play: *"for the kettle stoker, mana empowerment does not show on
the bar in battle, and seemingly does nothing for my attacks."* Both halves
were true and the second is the worse one.

**`magic_empower` is `stacks × 5 × mana`, and it scales a *magic* hit and
nothing else.** Empowerment is upstream's caster mechanic. So every stack the
Kettle-Stoker's furnace bought on a board holding a blade was a number that
could never be read — measured against `common::geared_from`, a Stoker dealt
**746** and a classless character dealt **746**, over one burn. Not close;
identical.

- **`Combatant::burn_stacks` is counted apart and pays both lanes.** That is a
  change to what a *Stoker* gets rather than to what empowerment means: nothing
  but `stoke` ever writes it, so a Chronomancer's stacks are still the
  caster's. The class is GM2D's own and nothing obliged it to inherit a
  restriction its promise does not mention. The same fight now ends in 4,200ms
  against 5,000.
- **A first draft also had the furnace bank the mana it shovelled**, which made
  the class work and quietly took **Fired Funnel's whole promise** — *every
  stack the furnace buys is also mana* is not a promise if the furnace already
  does it. `every_point_in_an_expert_tree_buys_something` said so on the next
  run and named the node. *An expert's power is its own*, and the lint is the
  thing that keeps saying so.
- **`every_offered_class_reaches_something` passed it**, because its fixture
  casts. *A fixture that can hold every branch of a mutually exclusive choice
  is measuring a game nobody plays* — this is that sentence about a **lane**.
  `the_furnace_reaches_a_board_that_swings` asks the shopped board instead.
- **And `Event::Burned` was in `fight_json`'s `_ => ("other")` arm**, exactly
  where `Cursed`, `Warded` and `Stunned` were before M8.2, so a class whose
  whole identity is a number had no screen that printed it. It is on the pool
  row now — beside the pools rather than as a chip, because a chip is a thing
  that is *on* you with a clock running and empowerment is a thing you *have*.
  **Sixth time** *a derived number needs somewhere it is shown* has been the
  answer here.

## A glossary is a proofreading surface

The unintended half of the last section, and it earned its place on the first
read. **The glossary is the first screen in this game that prints all
twenty-eight promises together**, and four of them turned out to be
ungrammatical: *"you may hold 1 enchs a component"*, *"1 stacks of mana
empowerment"*. That is the kind of thing nobody notices in a match arm and
everybody notices in a column.

`expert::enchs` and `expert::stacks` are the one answer, and
`no_promise_is_ungrammatical_about_a_count` is over **every** promise rather
than the three that happened to be wrong — **it found the fourth the moment it
existed**. A list of three written by hand is a list that can be two, and this
is that argument settled in about ninety seconds.

**And it called one class two names.** An expert's aside said *what Berserker
and Stoker reach together* under an entry titled **Gorillathon** — a canonical
engine name on a player-facing screen. `Theme::retell` swaps whole words and
does not cover the class table, so the pair travels beside the sentence on
`Entry::pair` and the shim puts the player's word in: core writes the sentence,
the shim translates the names in it. Found by *looking at it*, which is what
`make art`'s *draw it, then look at it* says about anything drawn — and a
screen is drawn.

**One thing is left as it is and is the human's.** Three expert promises read
oddly at their *untuned* values — *"a cast refunds 0% of what it cost"* —
because the knob is zero until points are spent in its tree. Accurate for
somebody who has just taken it, and it reads as broken. `describe` is shared
with the fork card and the tree tab, where naming a knob at zero is how you
know the knob is there, so hiding it is a decision rather than a tidy-up.

## Everything you need to play, in one place

`crates/core/src/glossary.rs`, on **G** or a button. Five shelves in the order
the game teaches itself: getting about, the board, the fight, what you can
become, what you carry.

Reported from play, and the example given is the argument: *"a game glossary
that explains everything you need to play the game, with stuff like what mana
empowerment does"*. Nothing had ever said what empowerment does, so a player
watching the stacks climb could not tell a mechanic they did not understand
from one that was broken. It was broken.

- **Derived, never typed.** Every number is read from the constant that decides
  it, every class describes itself with `ClassPower::describe`, and the pools
  come off `Combatant::pool_pays` through `Stats::parts` — the same function
  the standing panel draws. A glossary with its figures written out by hand is
  a second rulebook with a slower feedback loop than the first.
- **A first draft listed the pool fields by hand and dropped rage**, which is
  the pool every player meets first. Two answers to *what does a pool pay* is
  exactly the shape this file keeps recording.
- **`the_numbers_are_read_and_not_typed` took three goes to stop being
  vacuous**, and only breaking it found the first two. `contains("4")` passes
  with the fatigue figure hardcoded to *nine*, because the cart's forty-Fnorp
  fare has a 4 in it. Whole-number matching over the whole glossary passes too,
  because a class promise says *every 4 seconds*. **A figure has to be checked
  in the entry that says it.**
- **Unthemed, TONE 13a**, except a class's *name*, which is the world's word
  and goes through the theme — the same split the standing panel makes.

## The speed of a fight, and a log you can read

Both ported from the original, and the interesting thing about the port is how
little of it was engine: **`CombatLog::describe` has written a sentence for
every event since the fork and `combat::tally_items` has answered which lines
belong to which item, and neither was read by anything.** The whole of this was
interface — two more derived answers that had nowhere they were shown.

**The speed control steps 1 → ½ → ¼ → 2**, which is the original's cycle and is
a *slow-down*: the reason to reach for it is always that something went past
too fast to read, so the first press has to make it slower rather than faster.
**Settable before the fight as well as during it**, which is the original's
note and the better half of the idea — a replay you slowed down after it
started is one you already missed.

- **A step is to the next thing that happened, not a slice of time.** Stepping
  by fifty milliseconds walks a player through a second of nothing to reach the
  blow they were waiting for; the log is a list of moments and the step goes to
  the next one. The check compares against the log's own next entry.
- Space pauses, right steps, up and down change the rate — the original's keys,
  guarded on the replay stage being up, because those arrows walk the map
  everywhere else.

**The log screen is the transcript with both boards flanking it**, and the
arrangement is the whole idea rather than decoration. The original writes it
down: *the transcript is true and unreadable — forty lines of consequence, and
the question a player has is what did that piece do.* So clicking an item
narrows the list to that item's own lines and prints its account: activations,
goofs, seconds stopped, and what it put in.

- **Which lines are an item's own is core's answer.** `ItemTally::entries` is
  documented as *"the interface shows the log filtered to these"* — written for
  an interface that did not exist here until now. The page filters on that list
  and never works out ownership itself.
- **The page prints the sentence and does not compose one.** The gate compares
  every rendered line against `entries[].text` and every narrowed list against
  the tally, so a page that started writing its own account of a fight would be
  caught — it is the *"the page draws numbers core sent it"* rule applied to
  prose.
- `describe` matches the event enum exhaustively, so a new variant is a compile
  error there rather than a blank line on the screen.

## Both boards, and the jolt

- **A fight is two boards.** The replay drew neither; it now draws both,
  read-only, through the same `Theirs` painter the creature panel uses.
  `side_slots` in the wasm shim builds them for the panel and for both sides of
  the replay — one builder, so three screens cannot disagree about a cell.
- **What fires jolts.** A decaying wobble, 260ms, driven off the same
  activation times the cooldown bars are: six items on two boards all coming
  round at their own rates is unreadable, and movement says *that one, now*
  where a colour change would be five things happening at once.
- The shake is set from outside — `Theirs.shaking` is a list the replay writes
  and the painter reads. The painter decides nothing about when.
- **An innate attack has no cells.** A creature's bite stands on no gear, so
  nothing on a board moves for it; the browser check skips a fight where only
  the bite went off rather than failing one. Which activations are shakeable is
  a property of the fight.

## A swing is not a constant, and the row said it was

Asked directly: does fury give strength, devotion resistances, harvest regen —
the way the original does? It does, and the code is **byte-identical**:
`Combatant::held_bonus` in `combat.rs` diffs clean against
`sgilson7/gear-master`, and so does `stats::after_defences`. Measured rather
than read, end to end in GM2D fights:

| one point of | pays | measured |
|---|---|---|
| **fury** (rage) | +1 physical damage | a swing went 30 → 40 → 42 → 44 → 46 as it banked |
| **devotion** (faith) | +2 physical *and* +2 magic resist | 12 faith turned a 40-damage bite into 30 |
| **harvest** (nature) | +1 regen | 2 nature is a 1-point heal every half second |

Resistance and hardening are in and are the original's: resist cuts the blow,
piercing cuts the resistance, hardening cancels the piercing, and resist clamps
at 95. Twenty-seven components grant physical resist, thirty magic resist, five
physical hardening, four magic hardening, and all fifty-eight creatures carry
resistances.

**What had not come across was the screen.** The original draws a panel headed
*what a banked pool pays, per point*, built from `Combatant::pool_pays` so the
drawing cannot disagree with the rulebook. GM2D had `pool_pays` in core and
**nothing read it**, so a board banked fury for a whole fight, the replay
printed `fury 8`, and no screen anywhere said the 8 was eight more damage on
every swing. Fourth time this shape has been found here, after four skill
nodes, the opening armour bar and the ench rack.

- **The panel is derived twice over.** `pool_pays` gives the rates and
  `Combatant::pools_worth_holding` gives the list: a pool has to be one the
  catalogue can *grant* and one that pays something for being *held*. Mana and
  insight fail the second — they are spent, and empower or wound rather than
  paying a wearer for sitting on a pile — and the three fusions fail the first,
  because nothing in this catalogue makes one. So the panel is three lines, and
  a component that starts granting a pool puts it there without anybody
  remembering to.
- **Two registers on one line, TONE 13a.** The pool's *name* is the world's
  word and goes through the theme; what it *pays* is the engine's, unthemed and
  with the number in it, because somebody comparing two pools is comparing
  numbers.

**And the number beside an item in the replay was the opening estimate, for the
whole fight.** Held fury is added to every swing and a spin adds to the item's
own power, so what an item hits for at the tenth second is not what it hit for
at the first — and the row printed `hit_for` off the pre-bell stats and never
moved. The fight had always been right; the row was describing a different one.

- **`Event::Hit` carries the item that threw it.** `by_item` indexes the
  swinger's own item list — the same index `Activate` reports — so a screen can
  put a number beside the row that earned it. The three push sites all had it
  in scope already: `activate` has `idx` and `apply` takes `owner`.
- **Read, never derived.** The row shows the last swing the log attributes to
  that item at or before the playback head, and the opening estimate until it
  first swings. Scrubbing backwards puts the old number back, so it reads the
  whole list every frame rather than remembering what it drew last.
- **`damage` is the swing, before the defender's answer**, and that is
  deliberate and old: *a hit that is turned aside completely still has to show
  up, or a player stacking resistance sees nothing happening at all.* It cost
  an afternoon here — the first end-to-end measurement of devotion read
  `Event::Hit.damage`, saw 40 either way and nearly reported the mechanic
  broken. What a blow actually cost is `target_health` on the same entry.
- **The serialised `amount` for a hit was `damage + absorbed`** — a swing plus
  part of what that swing lost to armour, which is not a quantity anything
  could use. Nothing read it. It is the swing now.
- **The golden fixture prints `Hit` by hand, and only `Hit`.** That fixture is
  a character-for-character comparison against a transcript captured from
  upstream, so a field GM2D adds to an event upstream also has cannot appear in
  it. Every value upstream printed is still printed, so a swing that lands
  differently still fails; what is dropped is one field that did not exist when
  the capture was taken. **A second hand-written arm there should be argued for
  the same way this one was.**

## Curses were always there, and nothing said so

Reported as *"are curses in the game? if not, they need to be added"*. They
were, and always had been: **59 of the then-536 components apply one**, six are on a town
shelf and two are on the *starting* shelf at three Fnorp each. What was missing
was every screen that should have mentioned them.

Before adding a system, grep for it. This is the second time this project has
nearly built something twice — `explain.rs` was written with a duplicate
`Action::describe` and `Trigger::describe` already in `piece.rs`.

Three screens, and each was a different way of saying nothing:

- **The item card had no arm for `Action::Curse` at all.** It has a third group
  now, off `explain::curse_lines`, which is `Trigger::describe` filtered through
  the engine's own `walk_actions`. The sentence names who it lands on, so a
  piece that curses its own wearer reads as the downside it is.
- **`Event::Cursed`, `Warded` and `Stunned` fell into `fight_json`'s
  `_ => ("other", …)`.** A Whisperling could stack frost on you for a whole
  fight and the panel did not move.
- **Nothing showed a curse that was up.** The replay draws chips now, per side,
  beside the pools: the curse, its stacks, what it is *doing* — "30/s", "-75%",
  "1 in 2", off `CurseKind::effect_at`, which reads the same constants the
  simulation does — and a countdown.

**Read, never derived**, the rule the health bar and then the armour bar each
had to learn. A chip is `{kind, stacks, until}` where `until` is the event's own
timestamp plus the duration the event reported. Expiry produces no event, so a
chip is dropped when the clock passes it — pruned once by the entry's time and
again by the playback head, which covers the gap between two entries.

**A stun is its own event because it rides on one named item.** Two items
stopped at once is two chips, not two stacks, and a check that only ever saw a
curse would let that arm rot.

## The soft-lock M4 shipped and then found

For an afternoon the game was **unwinnable from its own first tile**, and every
test passed.

`apply_preset` is an eight-row arrangement and `Balanced Grip` is one cell wide
and four tall, so on a three-row starting frame the weapon had no handle and
assembled nothing. A starting character walked out of the pit with one glove,
lost every fight, and — because a loss pays neither gold nor experience — had
no way to buy or grind out of it.

Two things now stop it happening again:

1. `a_starting_character_can_win_in_the_pit` asserts the starting kit assembles
   a weapon and beats something in the region it starts in.
2. The calibration test **fights for real** instead of assuming every encounter
   is a win. The version that assumed wins measured how much the map offers
   rather than how much a player gets, and would have gone on passing.

---

# Part three — what a character becomes

## Levels

- **The level is derived from experience, never stored.** Two numbers that
  could disagree is two answers to one question, and a hand-edited save should
  produce a consistent character rather than a contradictory one.
- **Board size is a pure function of level plus granted rows.** So it can be
  checked rather than trusted. `resize_boards` only ever grows: a board that
  got shorter would drop whatever was seated in the rows it lost, silently.
- **A skill's *effect* is not state — the node is.** The tree is re-read on
  every load and every stat query, so retuning a node retunes every save that
  took it.
- **`XP_DIVISOR` is set by a test, not by taste.** It is 5 because that puts
  level 5 at a mean of ~27 fights across nine seeded walks of the pit. Moving
  the map's regions moves this; the band is the contract.

## A row is earned, not scheduled

Every level added one row to one grid, in a fixed rotation, from M1 to M12.
`PLAN-M12.md`'s thesis is that a board reads as inventory space rather than a
puzzle, and M12.0 went and measured it before anything tried to move it:

> **Fill goes *down* as you level.** 43% at level five, 37% at eight. Rows
> arrive on a clock and components do not, so levelling dilutes you — and a
> board with room for everything is a board that never asks which thing.

So a scheduled row is dilution on a timer, and the rotation is gone.
`progression::rows_for` and `grows_at` are retired; there is `base_rows()`,
which is three for every frame for ever, and `board_rows(granted)`.

- **A row is a thing you buy or a thing you finish.** Eleven skill nodes grant
  one and two errands do. M12.3 wrote seven of the eleven — one for each of the
  five frames, and a second for the weapon and the chest — and M13 added the
  four that carry a frame the rest of the way to the original six by eight, on
  a spine that costs more the deeper you go. See *A row is bought all the way
  up to the old size*. Every level now poses the game's own question with
  the player's hands on it — power on the board you have, or a bigger board —
  which is the pressure the whole block is about.
- **Nothing is banked, and the save needed no migration**, which is a
  divergence from `PLAN-M12.md`. `BoardSave::rows` is already written and
  restored verbatim and `resize_boards` only ever grows, so an old file keeps
  every row it ever earned without a ledger. Where a row *came from* is derived
  from `skills_taken` and `quests_done`, the same way a node's effect and the
  tower's fallen floors are.
- **`Quest::rows` is a `RowReward` and `Quest::granted` is a different
  question.** The first is what finishing pays; the second is whether the
  errand is ever offered at a counter at all. Both arrived in M12 and they are
  not the same field wearing two hats.
- **Two pillar tests were retired rather than repaired**, and that is the
  honest move: they asserted that level *N* implies a particular board, which
  was the guarantee this milestone deliberately removed. A test that pins the
  behaviour you are changing is a test to delete in the commit that changes it,
  with the reason in the message.

## Experience is carried, and a town is the bonfire

**A fight pays into your pocket. A town is the only thing that turns it into a
level. A defeat takes everything you are carrying and nothing you have spent.**

- `Character::xp` is what has been **spent**, and the level is derived from it
  and nothing else — the old rule holds, it just names a different number.
  `Character::carried` is what is on you. Two numbers, and not two answers to
  one question: one is what you have become and the other is what you are
  going to become.
- `carry` on a win, `bank` in a town, `drop_carried` on a defeat. **Nothing on
  the road calls `gain_xp`** — that is the spending primitive and `bank` is its
  only caller.
- `Settlement` lost `levels` and `grew`, because a fight cannot produce either
  any more. They are `fight::Banking`, which is where they happen.
- **Banking spends the whole pocket at once**, so it can cross several levels
  in one go: a character who walks home carrying a hundred and forty arrives at
  level five having passed two, three and four on the way. Anything asserting
  "the fork opens at exactly five" is wrong now — it opens at five *or more*,
  at the banking that crossed it.
- The class fork is offered from the town, because that is where a level lands.
- **Health was already free.** Combat health resets every fight — `Combatant`
  is built from `Stats` at the bell and nothing persists — so "health recovers
  after a fight" needed no change and is not one. What the souls rule adds is
  the thing that *is* at stake, which is what the note at the top of this file
  meant by there being no rest point: there was nothing to restore. Now there
  is something to lose.

### What it did to the browser walk

Two failures worth keeping, because both were the walk telling the truth:

- **The patrol cannot get you home.** `PATROL` is six steps east and six west;
  from more than six tiles away it never finds the town again. That did not
  matter while a fight levelled you on the spot, and it is the whole loop now.
  The grind heads for the nearest town once it is carrying enough
  (`head_for_town`), which is what a player does. It failed on one run and
  passed the next before that — a flaky gate is worse than a red one.
- **A level lands wherever you happen to be standing in a town**, not in the
  fight that earned it, so the receipt naming the frame is the town's.
  `BANKINGS` records every banking receipt as the walk makes them, and the
  level-up check reads that rather than the receipt of one particular fight.

## Classes

- **Four, and every one of them is upstream's.** Gorillathon, Funnel Sergeant,
  Worm-Fact Keeper and the Kaklon Licensee are `Berserker`, `Hexweaver`,
  `Bloodletter` and `Recycler` with the theme talking, so the powers — Leeching,
  Contagion, Bloodscent, Recycler — are already tuned and already tested.
  **Nothing new has been invented in combat for a class**, twice over now: M5
  took three and M8.4 took a fourth, and the fourth's identity is the ench rack
  rather than a new rule in the fight.
- **A promise must describe the game the player is in.** `Recycler`'s said
  "for each stack of Recycler you are carrying. Five stacks is half again on
  all five slots" — upstream handed the same class out repeatedly and a promise
  had to say what a second one bought. GM2D asks once, at level five, and the
  answer does not come off. `no_class_on_offer_promises_a_stack` is the lint.
- **The promise is the rule.** Each class's one-line mechanical promise is
  `ClassPower::describe()` put through `theme.retell`, so it cannot go stale and
  it speaks the game's language rather than the engine's.
- **The fork is permanent and offered until answered.** There is no path that
  clears a class; the level-five screen is the only one in the game that does
  not take Escape. A save made at level three arrives at five and is asked, and
  one made at nine without a class is still asked — the question was never
  answered rather than declined.
- **Three, since M13, and all three are live.** `Character::classes` yields
  zero, one, two or three in the order they were paid for — the fork, Spike's
  second paper, and the expert the pair reaches. The three places a power is
  honoured each fold over a slice where they used to read an option, and
  **nothing arbitrates**: the five shipped powers touch five different rules,
  the ten experts are written to the same constraint, and
  `no_pair_of_live_powers_disagrees` is what says it stays true.

## What two classes reach

Five classes were a fork at level five and nothing else. Finishing one meant
nothing, because there was nothing past it. M13 is what is past it, and the
whole block hangs off one countable fact:

    SkillsData::tree_finished(class, taken) -> bool

- **Finished is every node, not most of them.** Nine of ten does not unlock a
  thing — `a_finished_tree_is_every_node`. And it is derived off `skills_taken`
  like everything else, so nothing is banked and retuning a tree retunes who has
  finished it.
- **One finished tree buys a second class. Two hand over an expert.** The
  second paper is 5,000 Fnorp, matching the Patent, because Spike does not price
  by what a thing is worth to you. The expert paper is **free**: the
  twenty-four points were the price.
- **Ten experts, `C(5,2)` and no more.** `expert::EXPERTS` pairs them and
  `for_pair` is order-insensitive, because which class you took first is a fact
  about your afternoon and not about what the pair is. A list of ten written by
  hand is a list that can be nine, so
  `every_pair_of_offered_classes_reaches_an_expert` counts.
- **`ClassPower::Expert(ExpertPower)` is one arm, not ten.** Ten variants on
  `ClassPower` is ten new arms in every exhaustive match in the engine, and
  there are three of those with thirty arms already. Matching `Expert(e)` and
  then matching `e` is still two exhaustive matches.
- **Nothing new was invented in combat for a class, and that finally broke.**
  Six of the ten experts are read at the tick and four of the new knobs needed
  code that was not there — see *Four rules for a class to move*. What did not
  break is the older half of the rule: no expert is a new `ClassPower`, and no
  expert tree grants a stat.

### An expert tree moves its own promise and nothing else

The constraint that gives the block its shape, and it is enforced rather than
intended: **every node of an expert tree must reach that expert's power.** A
`tunes` of a knob the power declares, a `grants` of a rule the power is kin to,
or a `gives_ench` inside the power's licence. No flat stats, no `start_with`, no
`grow_slot_rows`, no bare `assembly_pct`.

- A `+12 strength` node would be a node you could take without noticing which
  class you were in. The five base trees are allowed to be a mix because they
  are the character's first shape; an expert tree is the argument for its own
  promise, six nodes long. `expert_nodes_touch_only_the_expert` is the lint.
- **`Effect::Tunes` is the seventh effect kind**, and the knob it names is
  checked at parse time **against the tree's own class** — a tree cannot tune a
  knob its class has not got, and `SkillsData::parse` refuses a tuning of zero
  and a tuning finer than the knob is printed at. `ExpertPower::step` declares
  the granularity off the knob's name: a node moving `window_ms` by 500 would
  cost two points and change no sentence, which is the *eight skill nodes*
  failure with a decimal point in it.
- **The promise is re-read from the tuned value**, so twelve points of tuning
  cannot go stale — and the tab prints it at its head, which is where every node
  under it is a footnote to that line.

### Every expert reaches something, and thirty-eight of them did not

`every_offered_class_reaches_something` has been a lint since M10.2 and it holds
five classes to *called, not declared*. M13.6 is the same question asked of
sixty **knobs**, and it is the most expensive check in the block for a reason:
on its first honest run it reported **thirty-eight of the sixty expert nodes as
points the tree sells and the engine never reads**. Four of the causes were the
engine.

1. **The fight was handed the untuned power.** `class_defs` returned
   `&'static ClassDef` off `CLASSES`, which is the roster before a point is
   spent — so the tunings reached `start_with` and `combat_items` and reached
   neither `combat.rs` nor `reward.rs`. See the rule in *Rules*.
2. **Two knobs were aimed at a kill, and this game deals one foe.**
   `PLAN-M13-2.md` §3.1 D and §3.7 C both write *a kill inside the fight*; a
   brawl has those and `fight::run` builds a single `MonsterSpec`, with
   `check_down` breaking the loop on the tick it falls. So strength refunded on
   a corpse and a free-cast window reopened after the only foe is dead both
   bought nothing. `combat::the_fight_turned` is what replaced it: **every
   quarter the enemy loses**, capped at three so the corpse is not one.
   **Quarters and not halves is `encore` deciding it** — an encore is a *count*
   and the tree sells two, so a milestone that can happen once is a count that
   can only ever be one.
3. **A percentage off three rounds to nothing.** A cast costs
   `SPELL_MANA_COST = 3`, so *twenty percent less* is nought point six and
   integer division makes that nought. `ExpertPower::cast_price` is the one
   place the sum is done, rounded the payer's way, and `describe` prints **the
   price** rather than the percentage — the two are different sums and only one
   of them is the one the fight does.
4. **`racks` was Full Bill's whole promise and `attach_ench` had never heard of
   it.** See *One ench a component, and two for a Full Bill*.

**The fixture is most of the work, and it has to be.** The first draft asked all
ten against one Auto-packed board and reported that seven reached nothing: it
has no empty frame for an Overwound Arm to turn, fifty-eight finished items
where a Standing Fact wants four, nothing that spins, and every one of its casts
was the enemy's. There are five boards now and each poses the question its
experts are about — *a check that needs something to happen has to make sure it
can* — and **a `put` whose answer is ignored is a fixture that silently does
nothing**, which is why every seating in that file is asserted.

**And the question the check asks is *what does this point buy*, asked where a
player buys it.** The primary comparison is the node's own prerequisite chain
against that chain plus the node, which is a build somebody can be standing in —
and it asks a threshold at the bottom of its range, where it is legible. Forty
against seventy is a different fight; seventy against a hundred is the same
fight twice when nothing in the game can spend seventy. The whole tree minus one
node is the second look, asked only of what the first cannot see.

### Four rules for a class to move

`Rule` kinds went 9 → 13, and these are the first since M9 that needed code in
the fight rather than a translation at the bell.

| rule | what it is | read by |
|---|---|---|
| `Spread { every_turns }` | an enchantment copies itself onto a bare frame | `fight::settle` → `Character::spread_underlay` |
| `RowHarvest { per_cell }` | a filled row pays mana at the bell | `Character::row_harvest` → `Held::mana` |
| `Beacon { pct }` | an ench lends a share of itself to what touches it | `Character::combat_items` → `ench::broadcast` |
| `Productivity { every, slower_pct }` | every nth activation of an enched item runs twice, and it is slower for the rest of the fight | `combat::activate` |

- **Spread works on the diagonal, and the plan said orthogonally.**
  `Slot::enchant_is_live` pays an enchantment nothing while another touches it
  edge-on, so a copy laid beside its source **kills both** and the node would be
  a point spent on making yourself worse. A corner is the tightest spread this
  board allows and the borrowed idea survives: what is next to what still
  decides what you get.
- **Spread is settled at the end of a fight, not during one.** Combat is a pure
  function of what it was handed — that is why a mid-fight save carries a
  creature name and a tile, and why `Effect::Fragile` breaks an item *for the
  fight*. A rule writing to the loadout mid-tick would undo that.
- **`RowHarvest` pays at the *start*.** *At the bell* means the start
  everywhere in this engine: `RunningItem` is rebuilt at every bell, `Held` is
  translated at the bell, `CombatLog::player` is the fighter as the bell went.
  The plan put it in `fight::settle`, which is the one place its mana could not
  be spent.
- **A beacon never chains.** The lends are gathered before the first one is
  written, so what a neighbour is given is never given onward — a packed chest
  broadcasting its own broadcast would reach a fixed point, and the fixed point
  would be the game. And **only the two enchs that are a number lend**: forty
  percent of a switch is not a thing, and lending `Fragile` would break every
  neighbour, which is a beacon that punished packing rather than paying for it.

## Derived, never banked, and that includes the last one

`Loadout::assembly_pct` is the extra percent every assembly bonus counts for,
which is the Kaklon Licensee's whole power and four of the base tree's nodes. It
lives on the loadout rather than being passed in because `report` is called from
a hundred and eight places — the sheet, each item card, the shop's comparison,
the fight — and every one of them has to see the same number; a parameter
through all of them is a parameter somebody forgets in one place, and the bug
that makes is an item card that disagrees with the fight.

**It was also written into every save and thrown away on the way in.** The
loader has re-derived it since M10.2, so what the file said was overwritten a
hundred lines later — and *a number that is stored and ignored is a number
somebody will one day believe.* M13.0 caught the shape from the other side: an
expert taken without the re-derivation came back from a round trip carrying a
figure it did not go in with.

- **The file does not carry it.** `#[serde(default, skip_serializing)]`, so
  every older file still opens and nothing new writes one.
  `the_assembly_bonus_is_not_in_the_file` plants a nine-thousand into an older
  file and watches it be ignored.
- **`Character::assembly_pct_of` is the one place the sum is done**, and
  `a_class_taken_any_way_re_derives_the_bonus` walks every door that sets a
  class and compares against it.
- **With the Kaklon Licensee on both sides of the pairing.** It is the only
  class whose power moves the number, so a door it is not standing at is a door
  that can forget the re-derivation and change nothing — a check that would pass
  on a game that was broken. The run is done twice for that reason alone.
- **One door the check cannot see, and it says so.** `take_expert` refreshes
  too, and no `ExpertPower` is a `Recycler` — `expert_nodes_touch_only_the_expert`
  refuses a bare `assembly_pct` in an expert tree. What holds that line is the
  lint rather than the call, and the comment beside the call names it.

## Eight skill nodes that cost a point and did nothing

Found while making the tree describe itself, and the reason that job was worth
doing properly.

`Effect::Stat` carried `armor` and `mana`. Both are **grants an item makes on
its own tick** everywhere else in the engine — `RunningItem` pays them on every
activation — so a *character-level* total of them has no tick to hang off, and
`Combatant::player` had always started both at zero and thrown the total away.
Eight nodes granted one or the other. They parsed, they cost points, they
showed as taken, and they changed nothing: `Corked`, `Funnel Drill`,
`Bedazzled Plaid`, `The Five`, and the whole spine of the Hexweaver tree —
`Army Issue`, `The Banana Standard`, `Anvil, Own Foot`, `A Funny Undone`.

The fix is a separate effect that says what it means:

- `Effect::StartWith { armor, mana }` — what you are already holding at the
  bell — and `combat::Held`, passed beside `Stats` rather than inside it,
  because folding it in would pay every item's armour again as a balance.
- One more rung on the simulate ladder (`simulate_holding` /
  `simulate_party_holding`), which is how every other run-only concern has been
  added: the existing signatures are untouched and no test had to say it holds
  nothing.
- `Node.effects` is a list now, since four of the eight granted a stat **and**
  a balance. It reads as one object or an array in the JSON, because most nodes
  do one thing.

**Why serde let this happen, and the lint that catches the next one:**
`deny_unknown_fields` is a container attribute, not a variant one, so it cannot
be put on `Effect::Stat`. serde therefore drops a key it does not know without
a word. `every_effect_key_is_one_the_engine_actually_reads` in
`tests/skills_read.rs` reads the raw `data/skills.json` and refuses any effect
key outside the vocabulary. Reading the parsed struct could never have found
this — the whole failure is that the parse succeeded.

## A skill has to say what it does

The tree described itself only in the world's words. *"Nine hundred feet of
Deep Chocolate mine, and you never once came up early"* is a good sentence
about a character and tells nobody it is sixty max health. Reported by the
human as *"completely unintelligible as to what they do"*.

Two registers, kept apart, and `TONE.md` rule 13a is the written version:

| | written by | speaks |
|---|---|---|
| `name`, `blurb` | a person, in `data/skills.json` | the book |
| `Node::line()`, `Node::detail()` | **derived in core from the effect** | the engine |

- **Derived, never typed.** A spec nobody writes by hand cannot disagree with
  the effect it describes. Retuning a node retunes its description.
- **Unthemed on purpose** — the one exception to rule 13. Somebody choosing
  between two nodes is comparing numbers, and a number wearing a joke has to be
  translated first. `no_mechanical_line_speaks_the_theme` enforces the inverse
  of rule 13 over exactly this text.
- `line()` is the one-liner on the button; `detail()` explains the words in it
  and appears on hover **and on focus**, so a keyboard reaches it.
- The class fork prints `power.describe()` raw. It used to go through
  `theme.retell`, which turned the one sentence somebody reads before an
  irreversible choice into a sentence about the Roast and the Nut Freeze.
- Check every number you put in a description. `SPELL_MANA_COST` is **3**, not
  30; the first draft of the mana line said "that many casts", which is not a
  number at all.

## A skill that works and cannot be seen is a skill that does not work

Reported from a real session: four nodes taken — `Corked`, `Funnel Drill`,
`Cave Lungs`, `Handspan` — and *"I am receiving none of the start of combat
bonuses and I cannot tell whether I have received the strength or not."*

**Every one of them was working.** Twelve armour soaked blows for the whole
fight. The engine was right and the screen said nothing, which from where the
player sits is the same thing as a broken skill.

Two faults, and they are the same fault twice:

1. **The fight opened at zero.** `fight_json` seeded its running snapshot with
   `armor = 0` and empty pools, then updated on events — and *nothing announces
   a balance nobody had to earn.* The only armour event reports what is **left
   after a hit**. So the bar sat empty until something took a swing at it.
   `CombatLog::player` is `start_player`, the fighter as the bell went, and it
   has carried the answer all along; the snapshot seeds from it now.
2. **Nothing showed the character sheet.** `character_json` had emitted stats
   since M5 and no line of code read them. +6 strength and +60 max health
   landed in a number no screen printed. `#sheet` prints it, and prints what
   the tree says you begin a fight holding.

Rules that came out of it:

- **A derived number needs a place it is shown**, or it cannot be told from a
  bug. The test that would have caught this is not a unit test — core was
  correct — it is `check_the_sheet_says_what_you_are`, which fails when core
  reports a non-zero stat the sheet drops.
- **The sheet speaks the node's words**, not the theme's: a node reading "start
  every fight with 12 armor" against a sheet reading "12 cork" is one number
  with two names, and the whole job of the line is to let somebody confirm they
  got what was promised. An item card still says Cork — a card is about the
  item, not about a promise being checked.
- **A check that compares zero with zero is not a check.** The first version of
  `check_a_starting_balance_is_on_the_bar` compared the log's opening armour
  with the bar's, and a character on the gate's walk holds nothing, so a build
  with the bug hard-coded to zero passed it. It feeds a log back with a balance
  on it and reads the opening row now. Negative-test every new check by
  breaking the thing it guards.

## A row is bought all the way up to the old size

M12.3 made a row a thing you buy and gave the base tree seven nodes that sell
one. That took a frame from three rows to four, or to five for the weapon and
the chest, and stopped — well short of the six-by-eight the original game
ships. Asked for:

> *"add another 5 rows to the sprocketmans craft tree; the skill bonuses should
> include things like more rows to all of the gear slots, but get progressively
> more expensive per additional row you add, up to the original gear master
> size"*

Five tiers, and they are a spine rather than a rack: The Fourth Course evens up
the three frames M12.3 left behind, and the three above it each give **a row on
every grid**, at three, four, five, six and seven points.

- **Every frame lands on exactly eight, and not one grant further.** That is
  what `every_frame_can_be_walked_to_the_old_size` is for, and the second half
  of it is the half worth having: `board_rows` clamps at `MAX_ROWS`, so a tree
  that over-grants breaks nothing — it quietly sells a point for nothing, which
  is precisely the failure eight nodes shipped with for two milestones. **A
  grant past the ceiling is a promise that reaches nothing.**
- **The escalation is read off the tree, not listed in a test.** A row node
  deeper than another may never cost less, and depth is what the prerequisites
  already say — a second copy of the order would go stale the first time a tier
  was re-parented.
- **The whole ladder is twenty-eight points**, against a `MAX_LEVEL` of 32 and
  a demo that ends around level fourteen. So the last two courses are an
  endgame the shipped content does not reach, which is what *progressively more
  expensive* buys and is worth saying out loud rather than discovering.
- **The capstone grants no row, because there are none left**, and says so.
  Five tiers were asked for and four carry the rows; a fifth that granted a
  sixth row on a five-row ceiling would be the exact thing the test above
  refuses.
- **A node that grows every frame says so once.** Listing the five separately
  came to a hundred and thirty-three characters, half again over what
  `a_mechanical_line_stays_short_enough_to_read_at_a_glance` allows — and a line
  nobody reads is a line that is not there. `Node::line` collapses it to *"+1
  row on every grid"*, derived, so a tier that stops covering all five goes back
  to naming them and cannot quietly claim the set.
- **The blurb still names every frame it grants**, which is
  `a_row_granting_skill_names_its_frame` and is not negotiable: a blurb that
  overstates its effect is the worst kind, because the player finds out by not
  getting it.

## The tree's wires were drawn on a screen nobody had laid out yet

Reported from play: *"there is also a bug in the way the lines are drawn in the
skill tree between connecting skills; they only appear after you make a skill
purchase"*, which is the whole diagnosis.

`openTree` called `paintTree()` and *then* set `$('tree').hidden = false`. The
wires are **measured** — the rows are flex and wrap, so where a node actually
is is the only thing that can be trusted — and a hidden screen is
`display: none`, where every rectangle is zero. So the first open drew all
seventeen wires as `M 0 0 V 0 H 0 V 0` on an svg zero wide. Taking a node
repaints while the screen is up, which is why the lines turned up on the first
purchase and never before.

- **You cannot measure an element that is not laid out.** The fix is the order:
  show it, then paint it.
- **The gate was green through it, and that is the finding.** Its tree check
  counted `#nodes .wires path` against the number of prerequisites and got
  seventeen for seventeen. **A check that counts elements is not asking whether
  they are drawn** — the "compares zero with zero" shape, one level along. It
  measures now: the svg has a width, and no path is at the origin, asked before
  anything on that screen has been clicked.

## The tree is a tree

It was one flat rack of buttons, which told you what existed and nothing about
what led to what.

- **Rows are depth, and depth is core's.** `Tree::depth_of` is 0 for a node
  with no prerequisite and one past the deepest thing it needs otherwise;
  `Tree::rows` groups by it. A screen working its own layering out would be a
  second answer to "what has to come first", and the two would part the first
  time a node gained a second prerequisite — `w-law` already has two.
- The top row is exactly **what you can spend a point on with an empty sheet**,
  which is the question somebody opening the screen is asking.
- **Within a row, order by the average position of the parents.** The cheapest
  thing that keeps the lines from crossing, and it puts a node over the things
  that need it.
- **Wires are measured, not computed.** The rows are flex and wrap, so where a
  node actually *is* is the only thing that can be trusted; `drawWires` reads
  `getBoundingClientRect` after layout and redraws on resize. Elbows rather
  than diagonals — a straight line through three rows of buttons is
  unreadable.
- A wire into a node whose prerequisite is taken is lit; the rest is
  scaffolding. `.open` outlines a node you could take right now, because the
  tree is mostly locked at any moment and the few open doors are what wants
  finding.

**One tab a tree**, and it is built for a list rather than a pair: a character
has the base tree plus whichever class trees they have unlocked, and there will
be more than one of the second kind. `all_trees_json` already returns exactly
the trees a character may spend in, so the tabs are however many that is.

Two things this broke, both worth knowing:

- `#tree-tabs` carries `class="tabs"` and is **not** inside a `.made` panel,
  and the tab styling was written as `.made .tabs`. It inherited nothing and
  the buttons stacked. A style scoped to a container is a style the next user
  of that class name will not get.
- The fork's browser check counted `#nodes .wares` and asserted "more than the
  base tree". Only the open tree is drawn now, so that question stopped
  meaning anything; it counts tabs and opens the class one instead.

## Skills that grant rules, not numbers

The tree could grant a stat, a starting balance, a row on a frame and a
percentage on every assembly bonus. All four are arithmetic. `Effect::Grants`
is the fifth kind and the first that says the game works differently for you
now.

**`Rule` is an enum, not a string**, and there are three locks on it, because
this project has shipped the other thing:

1. An exhaustive match wherever a rule is consumed.
2. `deny_unknown_fields` on the enum — a container attribute, which is exactly
   why it could not go on `Effect::Stat`.
3. `Rule::check`, run by `SkillsData::parse`, which refuses a rule naming a
   grid or a curse the engine has not got, or a tuning that tunes nothing.

**A granted rule is a fight input, not a mutable global.** It reaches combat
through `Held` — the same door the tree's armour and mana go through — and is
translated into a `Combatant` field at the bell, the way a `ClassPower` is.
Combat stays a pure function of what it was handed, which is the property a
mid-fight save carrying a creature name and a tile rests on.

`Rule::CurseOnActivate` fires in `activate` beside the item's own triggers,
**not folded into the profile**: a profile is the *board's* answer and this is
the *character's*, so two players with the identical board do not have the
identical fight, and an item's card must not start claiming a curse the item
does not own.

**`Rule::Scout`, and `#numbers` is gone.** That button was a debug overlay that
shipped, and it handed the region's danger and every tile's odds to everybody
for nothing — which makes a node granting them a node granting nothing. The
figures are `null` until the reading is earned and the panel says "you could not
say" rather than printing a zero: **zero is a number and would be a lie, and a
screen cannot tell a lie from a bug.**

The plumbing was built two milestones before the Kaklon Patent wanted it, and
that is the whole reason M8.6 could write eight nodes instead of writing them
twice.

### And an item grants them too

M9.0 widened `Effect::Grants` from the tree to an assembled item, which is what
every set in that block goes through. A type two systems grant is a type neither
of them owns, so `Rule` moved to `crates/core/src/rule.rs`; `skills.rs` keeps a
`pub use` and nothing about `data/skills.json` changed.

- **The strings are `Cow<'static, str>`.** A rule arrives from two places now:
  parsed out of JSON, where a name has to be owned, and written into `CATALOG`,
  where it has to be a compile-time constant. `Cow` is the one type that is
  both. The alternative was a second enum for the catalogue's half, which is two
  rulebooks.
- **`Character::rules()` is the tree's plus every rule an assembled item
  grants**, read fresh every time for the reason a node's effect is read fresh.
  It is on `Character` and not on `Loadout` for the reason enchs are.
- *An unassembled set grants nothing* is not a check anywhere: only assembled
  groups are walked, so it is the shape of the loop rather than a condition in
  it.
- **A rule pays off the whole set**, gated by `loadout::set_of` — see *Rules*.
  A rule off one component would be a rule off one component, and both M9's
  would ride into any glove or chest that happened to hold the piece carrying
  them.
- **Three consumers, each where it can honestly answer.** The combat rules go
  through `Held` as they always did; `Rule::Rout` is settled by `fight::rout`,
  before there is a fight to put it in; `Rule::Wade` is answered by
  `world::step`, where a wall is refused. The match in `combat.rs` has an arm
  saying the last two are not combat's, which is the arm existing so that adding
  a rule is a decision rather than a silence.
- **No `Rule::describe` was written.** `line` and `detail` already exist and a
  second describer is the mistake `explain.rs` made from the other direction.
  The sheet prints `Rule::line`, unthemed, TONE 13a — a rule moves no bar and
  prints no number of its own, so without somewhere it is shown there is no way
  to tell one that works from one that does not.

## Where an ench comes from

Every trading town kept a bench until M10 and sold every priced ench to any
licensee. That was right when enching was one class's whole identity and the
worry was stranding a licensee from their own class; it made an ench *a thing
you bought* rather than a thing you went and got, which is the reason the
shelves stopped rolling in M7.

**Three sources now, and a lint per source.** A tree awards one, an errand pays
one, and what neither does is sold by one man on the Verge road who is not there
until level ten. `every_ench_comes_from_somewhere` refuses an orphan;
`no_town_sells_an_ench` is the ask; `a_price_means_somebody_charges_it` refuses a
price nobody charges.

- **`Effect::GivesEnch` is derived, never banked**, like every other effect.
  `Character::enchs()` is what was bought or paid over *plus* what the taken
  nodes grant, read fresh — so retuning which ench a node hands over retunes
  every save that took it.
- **An expert tree is the one place a *second* source is right**, which is the
  exception M13.5 had to write. All eight enchs already have an owner, so any
  `gives_ench` in an expert tree duplicates one — and for Full Bill, whose
  promise *is* rack count, a second copy **is** the promise. So the lint exempts
  expert trees and asserts the opposite for them: an expert node may only hand
  over an ench something else already gives, because **a source hidden behind
  two finished trees is worse than a duplicate.**
- **`enchs_owned` changed meaning, and the loader carries old saves across.** It
  was *loose*, and `attach` moved an entry out of it — which cannot work once a
  derived list can grant one, because there is nothing to take an entry out of.
  It is *banked* now and `enchs_loose` subtracts what is on the board.
  `repair_enchs` runs on load, and `detach` no longer pushes, which would have
  handed out a second copy of every ench anybody ever unbolted.
- **Closing the towns nearly made the Patent inert.** Three of its eight nodes
  tune the spin, and the spin is not a stat — it is The Ponkey Turn, which was
  ninety Fnorp off a bench. A class offered at level five would have had an
  identity and three eighths of a tree that did nothing until ten. **Bench
  Rights grants it**, which is the node at the root of the spin spine with no
  prerequisite, already named for the thing it now hands over.
- **`PlaceKind::Bench` is a place and not an event**, and the reason is
  measured: `answer` refuses a second choice and writes the whole event id into
  `answered`, so a card could sell one thing once. A bench sells each line once
  and keeps the sold ones on the table, greyed — the town shelf's rule, and the
  gap is the memory of what you took.
- **A place can be hidden until a level**, and it reads the level the way a
  crossing does — through `Allowances`. Writing `level-10` into `flags` when it
  was reached would have worked and is refused: the level is derived from
  experience and never stored.
- **A level that opens a place says so.** The map redraws when you bank, which
  M10.0 had to be told to do — and a redraw is not a sentence. The M10.3 walk
  reached level twelve and never met the man on the road, so banking now names
  what the level opened.

## An item that fires once

**The first new rule in the fight since the fork**, and the smallest one the
game could be given. It is invented for an *ench* rather than for a class — the
fifth class reuses a power that was already tuned — but the distinction is thin
and is worth knowing.

- **`broken` is its own field**, not `stun_ms = u32::MAX` and not `has_fired`. A
  stun is a curse somebody put on you: aimed, resisted, counted, and it ends.
  This is the gear. And `has_fired` asks *was this the first?* while this asks
  *is it finished?* — one flag answering two questions is how the next person
  gets it wrong.
- **It breaks at the end of the activation**, after everything that activation
  pays. That is the bargain: the swing that finishes the item lands in full and
  nothing after it does. Before the payout it would be an ench that triples an
  item's power and never lets it use any.
- **For the fight, not for good.** `RunningItem` is rebuilt at every bell. A
  component destroyed permanently would be the fight writing to the character,
  and a mid-fight save carries a creature name and a tile because it does not.
- **`Effect::Fragile { pct }` is one effect, not two.** The power and the cost
  are one bargain, and two effects that could be attached separately would let
  somebody take the good half.
- **A stun does not aim at a broken item.** The rule was that stunning what is
  already stopped is the one outcome an aimed stun must not have, as a
  *tie-break* behind the rating. A stun on a stopped item is wasted for a
  second and one on a broken item is wasted for the fight, so it sorts ahead of
  the rating now.
- **An ench does not move what an item is worth.** `item_rating` prices the
  pieces and the cadence, and `ench::apply` runs over the profiles afterwards —
  so a Chonga'd blade is not suddenly the best item in the game by the shop's
  reckoning. Pinned in a test, because the day spent finding that out should not
  be spent twice.

## Two promises that reached nothing

M10.2 was going to add a fifth class. Its first two commits were not that.

**`ClassPower::Showstopper`** — *a fight won in under ten seconds pays fifty
percent more* — existed, was tuned, was themed as Top of the Bill, and was read
by nothing. `combat.rs` ignores it on purpose and correctly, because it is a
settlement rule; and `fight::settle` never looked at the class. It lands in
`reward::bounty_with_class` now, where the argument about what a fight pays
already lives.

**`ClassPower::Recycler`** — found by the lint written for the first one, on its
first run. The Kaklon Patent had promised *"+N% assembly bonuses"* since M8.4 and
delivered the tree's half and nothing else: `apply_skills` only ever read
`Effect::AssemblyPct`, and `combat.rs` skips the class saying it is a board rule
already in the profiles, which nothing put there.

- **The lint calls rather than declares.** Its first version matched the variant
  and said "the purse", which a stubbed payout passed cleanly — *a lint that
  reads a list rather than the behaviour is the failure it exists to catch, one
  level up.* `every_offered_class_reaches_something` now proves each of the three
  places a power can be honoured: the purse moves, the fighter at the bell
  differs, or the board differs.
- **The roster is `class::OFFERED`, in core.** It was a `const` in the shim and
  a second copy in `tests/classes.rs` whose own comment admitted it.
- **The licence is a list.** Two classes may ench, and what separates them is
  *which* enchs they can get — which is what taking the bench off every town
  made possible.
- **The window is a condition and not a formality**, measured: Top of the Bill is
  paid on **66%** of the wins a full board takes off the ladder, and the M10.3
  playthrough was paid on 65 of 99. The two agree, which is the point of taking
  both.

## Enchs, and the spin

**An ench is not an enchantment.** `PieceKind::Enchantment` is thirteen
catalogue pieces laid *under* the grid so gear sits on top of them — upstream's
terrain model, and a different mechanic. The book has its own word for the other
thing (the ench economy, p. 119), so the two words stay two words: no rename, no
migration, and nobody has to work out which of two meanings a sentence is using.

An ench is not a component either, for the three reasons a restorative is not
one: no shape, no grid, attached rather than worn.

- **The attachment is to the piece, not to the cell.** `Ench::on` is a
  `PieceId`, so an ench survives being picked up, turned, moved to another grid
  and put back down. A cell would have meant it falling off on every repack.
- **Both effects are numbers the engine already had.** `power` and
  `cooldown_ms` are what `PieceDef::power_bonus` and `speed_bonus` already move.
- **They land in `Character::combat_items`, not in `Loadout`.** A profile is the
  board's answer and an ench is the character's; a loadout that knew about enchs
  would be a loadout that knew about a licence.
- **The class is the gate**, not a node inside it. Enching is what the Kaklon
  Patent *is*, and a class whose identity waited on a point spent is a class you
  could take and not notice you had taken.
- **The mark is the fourth channel**, after motif, luminance and hue, and it had
  to be told from the lock's gold outer edge and the assembled item's pulsing
  white one. So it is drawn *inside* the component, where neither of those goes.
- **A priceless ench is on nobody's bench.** `price` is optional, and
  `QuestsData::parse` refuses an errand that pays one that has a price — a
  reward you could have bought makes the errand a slow way to shop.

### One ench a component, and two for a Full Bill

One, deliberately: two is a bigger design space and a much bigger interface, and
for fourteen of the fifteen classes neither has earned its place. The exception
is the expert whose whole promise **is** the second rack — Full Bill, both
licences on one counter — and it moves the number rather than the rule.

- **`Character::ench_racks` is the one answer and `attach_ench` is the one place
  it is enforced**, so the screens still assume nothing. *"One ench a
  component"* had been written into the refusal **as a rule**, so the expert
  selling two points of a second rack sold a rack the engine would not give —
  found by M13.6, which is the milestone that exists to find exactly this.
- **Nothing refuses a second copy of the same ench by name**, and that is not an
  oversight: `enchs_loose` has already refused unless you own two, and owning
  two is what Full Bill's own tree hands over. Which is also why
  `every_ench_comes_from_somewhere` had to learn an exception — see *Where an
  ench comes from*.
- **A rack is a stack.** `detach_ench` and `toggle_ench` take a piece **and an
  `nth`**, resolved once by `Character::nth_ench`, and past the end is the last
  one on. A screen drawing four rows whose buttons all reached the same ench
  would be three controls doing somebody else's job.
- **The card names all of them.** `ench_json` carries `more`, because a card
  naming the first of two is a card that is wrong about the item it is
  describing.
- **`tidy_enchs` retains rather than detaching once.** It used to drop *an*
  attachment to a component that was gone; a rack holds more than one, and the
  rest would have been left pointing at nothing.

### The spin

> *"if they are blocked and cannot rotate, then they do not move"*

**Rotation is decided on the board and banked in the fight.** Combat has no
board — `ItemProfile` is a flat snapshot, which is why a mid-fight save carries
a creature name and a tile and nothing else. So `Slot::turn_cycle` works out at
pack time which of the four orientations an arrangement can reach *in place*,
and the fight ticks through a list it was handed.

- **Deduplicated by the cells produced.** A one-by-four turned twice lands on
  itself; that is not a second orientation and would have paid a stack for a
  turn nobody can see.
- **Leaving room to turn costs you cells**, which is a real packing decision of
  the kind `PerAdjacentEmpty` already trades in. The spin is not free power; it
  is power bought with space.
- **The spend is the cap.** `SPIN_PCT_PER_TURN` is uncapped because a slow item
  stacks more and fires less; a ceiling would have had to be tuned against every
  cadence in the game.
- **`Event::Turned` and `Event::Spun` are logged rather than left to a clock.**
  A frosted item turns slower for the same reason it fires slower, so a screen
  dividing the playback head by a second would draw a shape the fight never had.
  The packing board has no fight to read, so it turns on the wall clock, which
  is the honest one there.

---

# Part four — the road, and what is on it

## The economy, and why the shelf stopped rolling

Three changes that are one change: **a character starts with almost nothing, a
town sells a fixed shelf, and a town asks you for something.**

- **The starting kit is `Oak Handle` + `Iron Blade`.** It was eleven components
  — most of a helmet, a pair of molds and a whole weapon — which made the shop
  decoration for the first hour. Two pieces assemble one weapon that beats a
  Cave Rat and a Bog Toad and loses to a Bone Archer, which is the opening.
- **The Iron Blade has to end up turned.** It is one cell wide and **four
  tall**; a starting weapon frame is three rows. Upright it does not fit
  anywhere, the weapon assembles nothing, and a character who cannot win cannot
  earn — the M4 soft-lock, exactly. **Auto-pack is what turns it**, and the
  board starts empty: the kit is *given into the bag*.
  `character.rs`'s `STARTER` constant and its `seat` method were an
  *arrangement* — eleven components with a cell and a rotation each — and they
  are **deleted**, in `64533fb`, with a comment where they stood saying so.
  Their own comments had described a game that seats the kit, which this one
  has not been for some time, and this file quoted that comment as live fact
  for five blocks until M12.4 read it — `TRIAGE-M12.md` row 12. **A constant
  nothing calls is a comment nothing checks** — and so is a comment beside a
  constant that moved. M12.6 multiplied every price by five and left two
  `_note` fields in `data/shops.json` describing the old ones: the barrel's
  said *twelve Fnorp or under* where `shop::BARREL_CEILING` is 60, and the pit
  shelf's said *everything on it is under six Fnorp. A starting character has
  twenty-eight* where that shelf is 75 to 125 and the purse is 140. **That is
  the gate's hardcoded `12` a third time**, in the one place nothing could
  catch it: a `_note` key is not read by the parser, so no test can disagree
  with it. Both corrected, and the ratios they describe never moved — the whole
  economy scaled together.
- **A shelf is content.** `data/shops.json` holds each town's stock and it never
  changes; the save carries `WorldState::bought`, which is a town id and an
  index. Same discipline as the map. `Game::shop` and `ShopSave` are gone, and
  a save written before this still opens — serde ignores the key it no longer
  knows, and the shelves it arrives to are the shelves everybody has.
- **The index is the identity**, so a bought entry is greyed and left where it
  was. Dropping it would renumber the list and a save saying "bought number
  three" would come back pointing at something else. It also just reads better:
  the gap is the memory of what you took.
- **Append to a town's stock, never insert.** Same reason.
- Reroll and pinning are gone **from the shelf**, and that has not changed: a
  town that sells something different every visit is not a place, and three of
  them are one slot machine in three costumes. What M12 put under the counter
  turns over, and the shelf above it still does not — see *Three tiers*.

## Three tiers, and an order that arrives on the world's clock

The shelf was the only way to buy anything, and a shelf that never changes is a
shelf you have finished. M12 put two more counters under it, and the three are
one design: **luck is cheap, choice is dear, and the middle is a shelf.**

| tier | price | how it is chosen |
|---|---|---|
| the bargain barrel | ×1 of catalogue, nothing over 60 | rolled, 13 lines, the same in every town |
| the shelf | ×5 | authored, per town, never rerolls |
| the order book | ×10, nothing under 65 | rolled, per town, and it does not arrive when you pay |

- **A commission is not a purchase, it is a wait.** You pay, and the piece
  arrives after three to ten **fights** — `shop::fights_for`, derived from the
  price, so a dearer thing takes longer. Not a timer and not a step count:
  fights are the thing this game actually spends, so an order is paid for in
  the currency the rest of the game is denominated in.
- **One order per town at a time**, and the refusal names what is already on
  the bench. Three towns is three orders, which is a decision about where you
  are going rather than a queue.
- **The barrel holds nothing the shelf holds, nothing off a creature, and
  nothing you carry rather than wear.** That list is checked at load in
  `ShopsData::parse` and again on every roll — *the rules are the rules whoever
  rolled it*, so a rerolled barrel is held to what the authored one passed.
  **The `EVENT_ONLY` list was read with a regex, twice, and under-captured
  both times**, which is how a Gold Chip reached the barrel; it is selected
  through the engine's own constant now.
- **Everything costs five times what it did**, on the human's instruction, and
  the starting purse moved with it (28 → 140). The catalogue's own `price`
  field did not move — `shelf_price` and `commission_price` are percentages of
  it — so the fingerprint is untouched and **no save was refused for this.**
- **The income was not scaled**, and the correction is the finding: see the
  rule *When a test disagrees with a cost, suspect the test's idea of income
  first.*

### The barrel could not hold a book, and nothing said so

Reported as *"make spells generally more available, so spells, crystal balls,
books, alignments, inks"*. Measured before anything moved, and the measurement
is the finding: of the **106** components in the casting family — 18 books, 31
spells, 19 inks, 26 orbs, 12 alignments — **six** were reachable, and all six
were errand rewards. Nothing on either placed town's shelf. Nothing in the
barrel. Nothing in an authored order. Nothing off a creature or out of an event.

Two causes, and both are lists.

**`roll_barrel` named its kinds by hand.** Thirteen rows — Handle, Damaging,
Frame, Plating, Base, Layer, Material, Mold, Material, Mold, Ring, Accessory,
Crest — under a comment reading *one of each kind the five recipes need*. There
are **seven** recipes across the five worn grids, because `piece::recipes` has
said since before the fork that a weapon is a blade **or a book or a crystal
ball**, and the list covered the blade. So a rerolled barrel could not produce a
book either, and the two casting ways of building a weapon assembled nothing
from any counter in the game.

`shop::barrel_wants` is derived from the recipe table now, with the counts the
recipes ask for — **a book wants one spell and a ball wants two**, so a barrel
holding one spell is a barrel that cannot finish a ball. Fourteen cores and two
extras rolled from the kinds a recipe will *take* rather than require, which is
what a reroll is for. **A list of kinds written by hand is the sixth of these
this project has paid for**, after `package-web.sh`'s modules, the `EVENT_ONLY`
regex twice, the gate's own ceiling and Auto-pack's twenty-two names.

**And High Wick is the arcane shelf, on no map.** It is the only counter in the
game that sells a book, an ink and three spells — and `barrel_pool` and
`ledger_pool` both refused anything *any* town stocked, so a town nobody can
walk into was holding three of the five barrel-priced spells out of the cheap
tier on its own behalf. The rule is about **undercutting**, and a shelf with no
ground under it undercuts nothing: `on_a_shelf_you_can_reach` asks
`data::towns_on_the_map`, and the day High Wick is placed its stock leaves the
cheap tiers by itself.

**89 of the 106 are buyable somewhere now**, and the opening barrel — ×1 of
catalogue, under every counter, from the first afternoon — carries a Chapbook,
two spells and a Clouded Orb, so all three ways of building a weapon can be
finished out of it.

- **`every_recipe_assembles_out_of_the_barrel_alone` is the check that was
  missing.** Its neighbour asks whether each of the five *grids* makes
  something, and the weapon grid always did. Asking the recipe table instead
  cannot go stale the next time a grid grows a second way of being built —
  break the barrel back to blade-only and it names both missing ways.
- **A rolled barrel is held to the same shape**, off `barrel_wants`, because
  the authored one passing while every rerolled one lost a way to build a
  weapon is exactly the shape of the bug that was there.
- **Neither placed town gained arcana, on purpose.** The pit is cheap basics
  and Kettleworks is metalwork; a town is its character, and the barrel is
  under every counter and is nobody's character. An ink *was* put on the pit's
  shelf and taken back off: `the_floors_cost_more_than_the_things_at_the_end_of
  _them` went red, because the errand chain already pays a book and a spell and
  one cheap ink to multiply them measurably raised the endgame board. **The ask
  was availability, not power.**
- **The bin got junkier to make room.** `the_barrel_is_a_floor_and_not_a_ceiling`
  holds the barrel's mean piece rating under the pit shelf's, and four casting
  cores pushed it over; a lower-rated layer, greaves material and sole bring it
  back to 3.87 against the pit's 3.90. **Inks and alignments cannot be in the
  authored barrel at all** — they are multipliers and the cheapest rates 11
  against the pit's ceiling of 10 — so they arrive by reroll and by commission,
  where nothing holds a floor.

### A reward you could buy, eighteen times over

Found by nearly authoring it: the first draft of the barrel's spell line was
**the Warding Sigil**, which an errand pays.

*A reward you could have bought makes the errand a slow way to shop* has been
written down since the errands were built, and it was enforced for the shelf
only. `EVENT_ONLY` holds every set piece and every chain reward off every
counter — and **eighteen ordinary errand rewards were never on that list**, so
the barrel could roll seven of them and the order book eleven more. Both pools
read `quests.json` now, and `no_cheap_tier_sells_what_an_errand_pays` is the
lint, over the rolled pools *and* the authored barrel.

### Turning one over

The barrel and the order book reroll. The shelf does not, for the reason it
never did.

- **`n*n` for the nth**, counted **per type** and reset every ten levels in
  every town. The curve is the point — the first is a shrug and the eighth
  costs sixty-four — and without the reset it is a wall by level fifteen.
  `reroll_band(level)` is `level / 10` and the counters clear when it moves,
  which is derived from experience like everything else and not a stored date.
- **The thing you are waiting for is never rerolled.** `roll_commissions`
  takes a `keep`, re-derives its fights from its price like any other line, and
  fills around it. A reroll that could delete a paid order would make the
  button a trap.
- **A refusal spends nothing**, pinned in a test, because the first thing
  anybody does with a refused button is press it again.
- **`rolled_barrel` and `rolled_ledgers` default**, so a save from before M12
  opens on the authored barrel with the counters at zero — which is what that
  character had.

### A licence you can buy

Spike Kaklon sells the Kaklon Patent's paper for **5,000 Fnorp** to anybody
whose class did not come with it, and each ench on his table is **2,000**.

- **`Character::licensed()` is the class *or* the paper**, one function, so
  every screen that asks whether enchs are yours asks it once. `bought_licence`
  is the only new field, and it is a `bool` on the character rather than a
  flag in the world, because it is a thing about you.
- **The class is still the identity and the paper is still not.** A licensee
  gets the Patent's tree, its two awarded enchs and its spin nodes; a buyer
  gets the rack and the bench. Five thousand is priced against that gap
  deliberately — it is late money, and what it buys is the ability to use what
  the game already paid you.

### Three papers, and two of them are refused

Since M13 the van sells three, **all visible from the first time you walk in**
and refused until their condition is met.

| paper | price | wants |
|---|---|---|
| the Kaklon Patent's licence | 5,000 | — |
| **the Second Paper** | 5,000 | one finished class tree |
| **the expert paper** | **nothing** | two finished class trees |

- **Visible-but-refused is the whole point.** *A locked line on a shelf you can
  read is a goal; an absent line is a secret* — the same argument the Reach's
  door makes when it opens the instrument frame instead of printing a refusal.
- **The refusal counts.** `StockGate::refusal` writes *"2 finished class trees,
  and you have finished 1 of the 1 you are"*, because a button that greys with
  no reason is a button a player reports as a bug — a sentence this file has
  now written down five times.
- **`StockGate` is one field with two arms.** The van already gated its stock on
  a level; this gates on a fact about the trees, and the *line stays drawn*
  either way.
- **The third line prints the expert's own promise**, so a player choosing a
  second class at level twelve can see what that pairing eventually reaches.
  That is the pairing decision made in daylight, which is what a fork screen is
  for — and the second fork's own cards say it too.
- **Two of the three are bought and the third is taken.** `Game::buy_paper`
  refuses in named ways and **spends nothing when it does**, which is the
  reroll's rule and the bank's: the first thing anybody does with a refused
  button is press it again.

### The second fork is the same screen, and it can be slept on

The level-five fork is reused rather than rebuilt, with two changes and no
others: it offers **four** cards — the roster minus what you already are — and
**it takes Escape**.

- **The paper is spent on the choice, not on the purchase.** Until it is
  answered it sits in the pack, and `class_offer_json` keeps offering it.
- **And it must not nag.** `offerClass` is called after every fight, after every
  banking and on every load, because that is what an unanswered question needs.
  Wired to the same three, the second fork came back after every single
  fight — which is the game refusing to let you sleep on it. It opens where a
  player asks for it: on the purchase, and from the line on the character sheet
  that says the paper is in the pack. **That line is the other half** — a thing
  in your pack that no screen mentions is a thing you have forgotten you own.
- **Each card names the expert that pairing reaches**, with its promise. The
  pairing is the whole of the decision and this is the only screen where it is
  made.
- **Choice is permanent, same as the first.** No path clears either.

## The bank, and what "banked" costs you

Asked for as *"make it so you can place your items from your bag into a bank
accessible from any town, its the same bank for all towns with infinite size"*.
One vault, every town, no limit — and the interesting half is not the size.

- **Banked is not carried, and that is the whole feature.** `Character::deposit`
  moves the id **out of `owned`** and into `banked`. So a banked component does
  not pack, does not bench, is not a key you are holding, and cannot be handed
  over a counter — three consumers that all read `owned` and are therefore all
  right for free. The alternative, a second list that still counts as yours, is
  a bigger bag with a screen in front of it, and a bigger bag is not a decision
  about anything.
- **It is on the character, not in the world**, for the reason `bought_licence`
  is: what you have put away is a fact about you rather than about a place. A
  vault per town would be a thing you had to remember the location of, which is
  bookkeeping rather than a choice — and `bank_json` takes no town id, because
  there is nothing about a place in the answer.
- **A seated component is refused by name and nothing moves.** Banking happens
  in a town, where the board is not on the screen, so lifting a piece off a
  grid here would break an item somewhere the player cannot watch it happen.
  `spend_one` does lift; the difference is that handing in a tally is forced
  and this is a choice. *A refusal spends nothing* — the reroll's rule, pinned
  the same way.
- **The registry keeps a deposited piece**, so a `PieceId` in the vault stays
  valid and comes back the same component with its ench still on it, because an
  ench names a piece and not a cell.
- **`Game::eq` had to learn about it.** That operator is hand-written and lists
  fields by name, and `banked` is not in `owned` — so a save that dropped it
  would have round-tripped green and quietly emptied somebody's vault. The
  comment above it warns about exactly this, one field earlier.
- **No seam.** `banked` is `#[serde(default)]` and skipped when empty, so no
  fingerprint moved and every older save opens with an empty bank, which is
  what those characters had.
- **It takes pressure off the bench, deliberately.** M12's whole thesis is that
  a board reads as inventory space; a vault is a place to put what you are not
  using, so `pressure::of`'s bench falls when you use one. That is the feature
  working, not a regression in the measurement — but the number in
  `testing/transcripts/m12.0.txt` was taken before there was anywhere to put
  anything, and a later run that banks is not comparable with it.

**Two towns is the check.** `tests/bank.rs` proves a deposit leaves the bag in
milliseconds; what only a browser can answer is whether the vault the *second*
town opens is the same one — a list per town would pass every unit test in the
repository and lose your gear the moment you walked east. So the gate banks
something at the pit, plants itself onto Kettleworks, and asks there.

## Errands

`crates/core/src/quest.rs`, `data/quests.json`. **Not** upstream's `quest.rs`
(a chain of receipts along a road, deleted in `48203ee`) and **not**
`piece::Quest` (a component that transforms after N activations). Three things
called quest; this is the only one a town hands out.

- **The tally is a bag item, not a counter.** Beating a toad gives you a Toad
  Eye and the eyes sit in your bag until you carry them back. A counter would
  be simpler and would also mean the errand had no middle.
- **A drop is gated on the errand, not on the creature.** Nothing falls before
  it is asked for and nothing falls after the fifth: a bag filling with eyes
  nobody wants is litter, and a sixth eye is a thing that cannot be handed in.
- Handing in unseats the tokens first. A component handed over the counter and
  still occupying a cell is a component in two places.
- The **ask** is derived and unthemed — `beat 5 × Bengulon Jungle Toad, then
  hand in 5 × Bengulon Toad Eye` — and the **brief** is the world's. Rule 13a
  again. `×` rather than a plural because a creature's name is a proper noun
  and some of them are already plural: The Rice Criers, The Drowned Court.
- **No two errands share a tally.** `holding` counts a token by name across the
  whole bag, so two errands wanting the same one would each see the other's —
  take both, kill five toads, hand in twice.
- **Every town has one**, and every errand names a creature that is actually in
  some region's pool. Both are tests: a town that wants nothing is a shop, and
  an errand naming a creature that is nowhere cannot be finished and nothing
  else in the game would say so.
- **A reward has to be usable.** The first errand pays a book *and* a spell,
  because a book with no spell assembles nothing;
  `what_the_errand_pays_assembles_into_a_weapon` seats both on a starting frame
  and checks a weapon comes out.

## Errands are not a town's

`giver` asks and `turn_in` takes it back, which makes "go and tell them in
town" one errand rather than two; `requires` makes a questline. Three goals:
slay something, bring something, or go somewhere and report.

- **Arriving is the doing.** `quest::on_arrival` runs on the step, so walking
  over the tile and carrying on still counts. The marker goes in
  `world.answered` — the same set a tile-event writes to, so a word and a door
  are remembered the same way.
- `Bring` names **a component or a restorative**, resolved by looking in both
  drawers. The alternative was a second goal kind asking the same question of
  a different list. It also had to be: the shelf sells each entry **once**, so
  "bring me four of a shop item" is impossible and "bring me four tins" is not.
- **No two errands share a tally**, or handing in one would empty the other.
- An errand shows at its turn-in only **once it is on you**: a clerk who has
  not been told about the heap has nothing to say about it.
- **Every reward is unique and on no shelf.** They are in `EVENT_ONLY` too, or
  the creature stepper walks into them — a Harvest Crest turned into
  Marbulon's glass before that was noticed. A reward you could have bought
  makes the errand a slow way to shop.
- **An errand can pay an ench**, in `Quest::enchs` rather than in `reward`,
  because an ench is not a component: no shape, no grid, and it goes in a rack.
  The same rule holds and is enforced at load — `EnchDef::price` is optional,
  and `QuestsData::parse` refuses an errand paying one that has a price. It is
  handed over whether or not the character is licensed: an errand does not know
  what you became, and a reward that vanished for three players in four would
  be worse than one they cannot use yet.

## A locked choice was a wall, and is a target now

Reported from play, standing at the wall an errand had sent them to:

> *"I'm trying to submit the quest 'the cork you took' at the cork boundary,
> but there is no button in the event for me to turn it in ... when I was doing
> the part in the kettleworks, i had the strip of cork in my loose bag but I
> was not able to start the quest at the event you had to go to."*

**Two faults, and only the first was mine.** The missing button is the granted-
errand filter — fixed in the commit before this one, verified on the live build
with THE CORK YOU TOOK sitting on the boundary's counter marked ready.

The second is the chain itself. The cork ladder is four rungs and two maps:

| rung | where | needs | hands over |
|---|---|---|---|
| the cork boundary | west-bambulon | — | `has-cork`, and the errand |
| the standing frame | west-bambulon | `has-cork` | `corked-the-frame` |
| the rind wall | kettleworks | `corked-the-frame` | `corked-the-wall` |
| the crumb field | kettleworks | `corked-the-wall` | — |

And the errand's own brief says *"You have a strip of boundary cork ... somebody
at Kettleworks has built a wall out of the same stuff and left a gap in it.
Take the strip and see where it goes."* It sends you to the third rung holding
the first one's key. The wall then refuses, correctly, and the plain statement
before the attempt read **"Requires: corked the frame"** — the name of a fact,
with nothing anywhere saying where a frame might be.

- **`Requirement::wants` names the event that hands the flag over**, so the
  line reads *"Requires: corked the frame — THE STANDING FRAME"*. Sixteen
  events in this game have exactly one choice and it is gated on a flag; every
  one of them says where to go now.
- **Looked up, never listed.** Whichever choice sets the flag is the one that
  opens the door, so the events are asked and the answer cannot go stale when a
  chain is re-authored.
- **It takes the events rather than reaching for `data::events()`.** This is
  called while that data is being read, and a lazy static that asks for itself
  is a deadlock.
- `Requirement::describe` is *the plain statement before an attempt* and
  `unmet` is the flavour after one — the split `event.rs`'s dead type wrote
  down and M12.5 ported. This is that split finally paying: with only the
  flavour a refusal is a wall, and the statement is what makes it a target.

**What is still open is a content call and it is the human's.** The brief points
at the wall and does not mention the frame. Either the brief should say so, or
the wall should take `has-cork` and the ladder should be three rungs. Both are
defensible and neither is a bug in the engine; the check that the ladder *has*
a bottom is `the_cork_chain_is_a_ladder_and_not_a_wall`.

## Forty-one dismissals, and the choice nobody would take

This file used to say the Kettleworks field's forty-one events were prose and
nothing else, and that the answer was owed rather than written. M12.5 is that
answer, and it took two passes because the first one was not enough.

**The first pass gave the events something to do.** Nine of the fifty-six asked
a question before; forty-three do now, over seventy-three choices. Four
outcomes were added to `tile_event::Outcome` and every one is a thing the
engine could already do somewhere else:

| outcome | what it is |
|---|---|
| `Supply { id, n }` | tins, so a map can restock you |
| `Tire(u32)` | fatigue, so a choice can cost the only thing a fight spends |
| `Warp { map, at }` | put down somewhere else — including under the lake, early |
| `Errand(String)` | hand over an errand |

- **`Outcome::describe` and `Requirement::describe` are ports, not
  inventions.** Both existed on `event::Outcome` — the campaign's dead type —
  with the doc comments that are the design, and this file's *Two types called
  Outcome* section is where that was written down as a debt. It was paid by
  moving the pattern to the live type, which is what that section said to do.
- **A warp is never a way home.** `a_warp_is_never_a_way_home` refuses one that
  lands in a town, because the Drover's Stride is what a ride home costs and an
  event that undercuts it makes a whole set a curiosity.
- **Every flag an event sets is read by something.** A `Flag` outcome that
  nothing consults is a promise into a counter, which is exactly the shape of
  the `Outcome::Xp` bug that went four blocks unnoticed.

**The second pass is the one that matters, and it came from the human playing
it:**

> *"at THE SHALLOWS MARKER, to the user its either 12 experience or 20
> experience, so they'd always take the 20 cause why would you take the smaller
> number."*

Which is right, and it is not a tuning problem. Both branches opened a
different chain of content and **neither branch said so**, so the screen was
offering a small number beside a large one. A choice whose consequence is
invisible is not a choice.

- **A root choice hands over an errand, and no two branches the same one.**
  That is the fix and it is structural: an errand is the one thing this game
  has that says *something has opened and it is somewhere else* — it lands in
  the log, and the log points at the map. **Ten roots, twenty-one chains.**
- **The lint allows a third answer, and naming it is the whole of what makes
  it usable.** A choice at a root may start a chain, or **continue** one — an
  event is often both, and The Standing Frame begins two chains while being the
  second rung of a third, its cork branch gated on the cork you took at the
  boundary. What `every_choice_at_a_chain_root_starts_its_own_errand` refuses is
  the third kind: an ungated choice at a root that goes nowhere, which is
  exactly the smaller number sitting beside the larger one.
- **A chain errand is `granted`.** It never appears at a counter, because the
  branch you did *not* take must not be sitting on a shelf a moment later
  offering itself for the asking. `quest::at` filters them and `stage()`
  reports one you were never given as `Locked`.
- **The chains pay things, not points.** Enchs, map shards, rows, a ride under
  the lake — `the_chains_pay_more_than_experience` is the lint, because
  experience is the reward that reads identically whichever branch paid it.
- **`geared_from` had to learn that chains are exclusive.** The fixture gave
  itself all twenty-one chain rewards, which is a character no player can be,
  and it broke a reachability check by making the tower's floors look cheap.
  One reward per root now. *A fixture that can hold every branch of a mutually
  exclusive choice is measuring a game nobody plays* — the same failure shape
  as measuring a pool by what it contains rather than what it deals.

## A log that points at the map

The errands existed, there was nowhere to see them all, and no way to find out
where a Whisperling lives.

- **Where an errand points is core's.** `quest::guide` takes the errand and
  every shipped map and answers in ids: an untaken errand points at whoever
  asks, a full tally at whoever takes it back, a word at the tile you have to
  stand on, and a slaying at every region whose pool holds the creature. A page
  working that last one out would be a second copy of "what lives where", and
  the pools are the one thing on this map that gets retuned.
- **The pin is state.** `WorldState::pinned`, one at a time, off by being pinned
  again, dropped by `hand_in`. It goes in the save for the reason the feature
  exists: a highlight that died with the screen would be a reference, and the
  value is entirely in the walking.
- **The highlight is motion, not a fifth hue.** The map already carries terrain
  hue, region shade, place marks and the player. So the region breathes and the
  ring's dashes march, both in the gold already on the map — and the redraw loop
  runs only while something is pointing.
- **`quest_log_json` is a different question from `quests_json`**: what is on
  you versus what this place wants.

The log is a screen of its own rather than a tab inside the tree. `#tree-tabs`
answers "which tree", and a strip answering two different questions is the
`.card` collision in a new coat.

## What a creature leaves behind

Three sets, one a pit creature, and each is exactly one grid's recipe — so three
drops make one finished item and nothing has to be bought to complete it.

| set | creature | grid | what it does |
|---|---|---|---|
| The Rat King's Mandate | Cave Rat | gloves | `Rule::Rout` — an A. Rat gives up |
| The Toad's Own Frame | Bog Toad | chest | `Rule::Wade` — the lake has a rim |
| The Wallspider Weave | Bone Archer | helmet | `Rule::CurseOnActivate` |

- **The third set does not change the world, and that is the point of it.**
  `PLAN-M9.md` proposed a combat bonus through `AssemblyBonus.triggers`; that
  cannot be a *set* bonus, because an assembly bonus's triggers pay on any
  assembled item holding the piece. So the Weave grants a rule the Patent's
  nodes have granted since M8.3, which combat already translates at the bell —
  still no new combat code, and `Rule` kinds went 5 → 7 rather than the plan's
  8.
- **A set's name is not themed.** Every other name in `piece.rs` is canonical
  and the theme gives the player's word for it; these are the player's word
  already. A name somebody wrote is a proper noun and a proper noun is not
  translated. They are constants because a set name is the key
  `loadout::set_pieces` matches on, and `set_pieces` derives what is in a set
  from the catalogue rather than listing it — the last list of names written by
  hand here was `package-web.sh`'s modules and it was missed twice.
- **`data/drops.json` is creature, component, per-mille**, keyed canonically
  like a `Slay` goal, refused at load for a creature the ladder has not got, a
  component the catalogue has not got, or a rate outside 1..=500. A certainty is
  not a drop, it is a boss's `drops` field.
- **The roll is in `fight::settle`, on a victory, off `game.rng`.** `fight.rs`
  did not touch the stream before M9.1, so a won fight now costs one draw per
  drop entry. `a_seeded_walk_replays` walks a fixed path with two seeds and
  settles nothing, so it holds — re-read rather than assumed.
- **Every entry is rolled whether or not the piece is already in the bag**, and
  the refusal happens after. Skipping the draw would make the stream a function
  of what the player is carrying rather than of the fights they had.
- **`pay_a_win` is one function**, because a rout pays what a win pays and two
  copies of "what a win pays" is two answers to one question.
- **A rout is settled where the encounter is.** A fight decided before its first
  tick is a fight the replay has to draw, and there is nothing to draw. It pays
  what a win pays, costs **no** tiredness — nothing was fought, and the receipt
  says so because a player will check — and **a boss is never routed**, the same
  rule that looks a boss drop up by the tile.
- **A drop you could buy is worse than a quest reward you could buy.** All eight
  components are `EVENT_ONLY`, which also keeps them out of every footprint
  family `stepped_component` walks — so no creature can ever be dealt one, which
  is why arming three assembly bonuses did not move the ladder.
- **A component's card names its set.** Found by playing it: the M9.4 walk
  picked up all three of the Mandate's components and was never told they made
  anything, because Auto-pack packs for a rating and a set is only the set.

### The rate, and the thing the rate cannot fix

Set by a test, like `XP_DIVISOR` and `PER_FIGHT`. What that test *measures*
changed in M9.4 and the change is the finding:

**Wins in the region, not wins against the creature.** `draw_enemy` weights a
pool so its hardest member is its rarest, and the pit's 16 / 80 / 95 deals
80 / 16 / **1** out of 97. Measured per creature, all three rates looked fine
and the Wallspider Weave was three and a half thousand fights away. Measured the
way a player counts — draw the pool the way the map draws it, count every win —
the three sets are 40, 120 and 242 wins, against a full playthrough of 159.

So the rates are 50 / 80 / 500 per-mille and they are *not* the same on purpose:
the rate carries what the draw does not. **The pool weight is the proper fix and
it is a person's** — `PLAN.md` §6b, row 1.

## Six more sets, and one of them is a bus fare

M11.9 took the sets from three to nine. The M9 conventions are unchanged and
did not need revisiting, which is the return on having written them down: three
drops make one finished item, every piece is `EVENT_ONLY`, nothing is on a
shelf, the rule rides one component and pays off the whole set through
`loadout::set_of`.

- **Five of the six rules are instances the engine already had.** `Rule` kinds
  went 7 → 9, not 7 → 13. `CurseOnActivate` on gloves with frost is not the
  Wallspider Weave's, and `Rout` on The Curator is not the Rat King's — a
  different instance is a different set bonus, and the alternative was
  inventing six combat mechanics for six sets. **Nothing new has been invented
  in combat for a set**, which now holds across all nine.
- **`a_set_is_never_behind_the_rarest_fight_in_its_region` is `PLAN.md` §6b row
  one obeyed rather than paid off.** `draw_enemy` makes a pool's hardest member
  its rarest; this block found that the hard way three separate times. The
  check refuses a set whose owner is drawn less than a fifth of the time.
- **The tower's set is exempt and is the reason the check names the others.**
  The Curd Mantle is three certainties off floors five, three and one, so
  climbing *is* the grind. And a floor is one sitting, so a set off a floor's
  *pool* would be unfarmable — which is why the tower carries one set and not
  the three the plan asked for.
- **`every_set_is_one_creatures_and_one_grids` had to widen to "one creature
  *or* one stack of floors."** A rule that was exactly right for three sets was
  not a rule, it was a description of three sets.

### The Drover's Stride pays a tin and walks you home

`Rule::Homeward` is the sixth, and it is not a combat rule at all. Assembled
whole, the greaves take you to your last town for the price of one restorative.

- **`Game::go_home` refuses in four named ways** — not wearing it, nowhere to
  go back to, no fare, and not from under the lake. Each names the thing in the
  way, because a button that greys out with no sentence is a button that reads
  as broken.
- **Not from under a lake**, and that is the design rather than an omission: it
  is the one map where the walk *is* the content, and a set that posted you out
  of it would delete the thing the early way costs. From inside the Stack it
  works, because the tower already kicks you out every floor.
- **It drinks the cheapest tin and only one.** Choosing which tin to burn is a
  decision nobody wants to make twice a session, and a fare paid out of small
  change is a fare.
- **A refusal spends nothing.** Pinned in a test, because the first thing a
  player does with a refused button is press it again.

## The long cart, and why it does not undercut the Stride

Reported from play: *"there should be a way to teleport between towns."* From a
town, the cart runs to any town you have stood in, for `game::CART_FARE` — 40
Fnorp, two cheap tins.

- **It runs counter to counter, which is the whole reason a second kind of
  travel is allowed to exist.** The Drover's Stride's job is getting you *out
  of the wilderness*: it works from anywhere and costs a tin. You have to
  already be somewhere safe to board the cart, so it can never be the thing
  that saves a run. What it sells is the walk between counters, which is
  bookkeeping rather than a decision. `every_ride_begins_and_ends_at_a_counter`
  is that stated as a property.
- **Where you have been is a counter**, `stood:<town>`, bumped in
  `arrive_in_town` — the `beat:` and `met:` pattern a third time. Counters
  already round-trip, so remembering every town a run has stood in costs **no
  save seam at all**, and `cart_stops` works the list out fresh.
- **It charges, because free fast travel is the one thing that would flatten a
  map.** And *a refusal spends nothing*, pinned the same way the reroll and the
  bank pin it.
- **Not the counter you are leaning on.** A timetable offering the town you are
  standing in would be a fare for nothing.

## Fatigue is what a fight actually spends

Health resets at every bell, which is why a rest had nothing to restore.
**Every battle takes four percent of your maximum health for good, won or
lost.** Two things give it back and they are not the same thing:

- **A town takes all of it off**, on arrival, in the town's own voice —
  `Game::arrive_in_town`, which is core's because "a town mends you" is a rule.
  It is still not a rest: health was always free, and what a town undoes is the
  one thing a fight *does* spend. A defeat walks you home, and home is a town,
  so a lost fight ends rested — the wear happened and the walk undid it.
- **A tin takes some of it off wherever you are standing**, which is the
  decision this mechanic exists to create: another fight, open the tin, or turn
  round. The town is what makes the walk home worth taking rather than a
  formality.

The shelf was retuned when the town started mending — 6/16/40 → 4/11/28, and
**20/55/140 since M12.6 put every price in the game up fivefold**. A tin no
longer buys back a fight, it buys the walk home, so
`a_restorative_costs_less_than_the_walk_home` prices it under what the fights it
undoes pay rather than at several times over, with a floor, because a tin that
costs nothing is not a decision.

- A **percentage**, so it means the same thing at level one and at twenty.
  Twelve points is a third of a starting character and a rounding error later.
- Applied **last, on the total**, in `player_stats`. Taking it off the base and
  adding gear back would make a helmet cure tiredness.
- Capped at 60. A maximum that can reach zero is a character who can neither
  fight nor mend, which is a game over with no screen for it.
- `PER_FIGHT` is set against the pit by
  `a_full_expedition_is_a_budget_and_not_a_wall`, which walks twelve fights and
  refuses a number that makes the second unwinnable or the tenth free.
- Restoratives are **not components**: no shape, no grid, spent rather than
  worn. Three good reasons not to force them into `PieceDef`, where each would
  have been a special case. `data/supplies.json`, and every town sells them —
  a place that had run out of the only thing that undoes tiredness is a place
  you could strand yourself at.
- Drinkable **from the standing panel**, not in town. The decision this exists
  to create is the one on the road: another fight, open the tin, or turn round.

---

# Part five — how it looks

## How the board reads

Lifted from upstream's `crates/gui`, which had a documented, tested,
colourblind-safe design GM2D's first board ignored. **Three channels, any two of
which can be lost:**

| channel | carries | where |
|---|---|---|
| a motif stamped on every cell | the slot | `look::motif` |
| brightness | the role — **cores darkest** | `look::kind_luminance` |
| an Okabe-Ito hue | the slot again | `look::slot_hue` |

- **The palette lives in `core::look`, not in the page.** It is numbers and an
  enum, not graphics, so core stays graphics-free — and the accessibility
  contract is enforced by `cargo test` rather than by looking at a screenshot.
  `tests/look.rs` is ten tests, seven of them ported near-verbatim from upstream.
- **The one number: `ROLE_SEPARATION = 0.08`.** Consecutive role steps must
  differ by that much in luminance *in every hue*. It is why `slot_color`
  bisects for a brightness target instead of picking three HSL lightnesses —
  the same lightness lands at wildly different brightness per hue, and yellow
  flattens its top two steps into one.
- **Assembled versus not is brightness and weight, never gold against red.**
  That pair is the one distinction red-green colour blindness is worst at, and
  the gold collides with the greaves hue. GM2D shipped the rejected pairing for
  two milestones before the original's comment was read.
- **A component is one shape, not a row of tiles.** Cells fill edge to edge; the
  dark edge traces only the true boundary. So a four-cell blade reads as one
  blade, and the lines inside an item are the seams between its parts.
- **A shared component is grey until it is placed**, and takes its grid's colour
  and mark as it crosses in — which shows the rule without stating it.

## Board rendering — the rules that were learned by breaking them

- **Never cache what core can be asked.** The held component is looked up by id
  every frame, not copied at pick-up. The copy went stale the moment the player
  turned it: core rotated correctly and the board kept drawing the old shape.
- **The drag footprint is painted last, over the pieces.** It used to go onto
  the empty grid before anything was drawn on it, so every occupied cell covered
  it — and occupied cells are exactly where a drop fails and an answer is
  wanted.
- **The ghost on the cursor is translucent and offset.** At 92% alpha sitting
  square on the target it hid the green-or-red answer at the moment it was
  being asked.
- **The canvas sizes its own backing store to its box.** A fixed intrinsic width
  is a fixed width *scaled by CSS*: 1240 displayed at 800 turned every 34px cell
  into 22px and left a third of a screen empty underneath.
- **Text belongs in HTML, not on the canvas.** The item list was 11px canvas
  text crammed under each grid, where a second item overlapped and a third was
  cut off.
- **The replay reads health, it does not compute it.** The log reports
  `target_health` on a hit and `health` on a burn or a regen. Subtracting
  `damage` from a running total ignores `absorbed`, so armour soaked a blow, the
  bar dropped anyway, and both sides could sit at zero for the rest of a fight
  that was still going. `fight_json` carries a snapshot per entry.
- **An item card has two halves and which stat goes in which is not a
  presentation choice.** *Standing still* is what the item contributes whether
  or not a fight is happening — health, strength, power, regen, resists, pierce,
  harden. *Every activation* is what one tick does — damage, cork, the Funny,
  fury, devotion, harvest, plus any unconditional pool gain folded in from a
  trigger. Cork resets every fight; listing it beside max health told the player
  they were wearing armour they were not. `testing/drive.py` checks the split.
- **Do not reuse a class name.** `.card` is the event dialog — `position: fixed`,
  `inset: 0`, `z-index: 10`. The item cards were given the same class and every
  one of them became a full-viewport overlay pinned over the game. Found by
  measuring `elementFromPoint`, not by reading.

## A component is a shape

Everywhere a component appears it now shows the shape it takes up and the kind
of thing it is, and explains itself on hover.

- **Two blades at one price are not the same purchase** when one is four cells
  in a line and the other is a cross. The shelf gave a name, a slot and a
  price, which is everything about a component except the thing you are buying.
- The bag under the board drew a **one-cell swatch for everything**, so a ring
  and a twelve-cell base looked identical — hiding the only property of a loose
  component that decides where it can go.
- `explain::piece_lines` is what a hover reads. It uses `Action::describe` and
  `Trigger::describe`, **which already existed in `piece.rs`** — the first
  draft of `explain.rs` wrote both again, which is the "engine owns the
  sentence" principle failed from the other direction. Check before writing a
  describer.
- `every_component_says_something_about_itself` covers the catalogue. It skips
  quest tokens (a tally does nothing on purpose) and the six `EVENT_ONLY`
  relics — **whose value lived in `relic.rs`, deleted with the campaign.** They
  are on no shelf and no surviving event grants one, so they are unreachable
  content rather than a lint to satisfy with invented stats.
- **Two answers on one hover, and neither replaces the other.** The panel card
  is about the *item*, because pointing at a blade is asking about the weapon;
  the hover card is about the *component*, because that is what you are about
  to pick up. `board.onpoint` and `board.onpiece` are both reported.
- `shape.js` and `Board.thumb` both draw through `paintMotif`. The mark is the
  shape half of the colourblind triple-encoding, so everything that draws a
  cell draws the same one — at 34px on the board, 11px in the bag, 14px on a
  shelf.

## A grid says what it takes

The packing screen showed what you had built and never what a grid wanted, so
the recipes were a thing you learned by trying combinations or by reading
`piece.rs`. `recipeBox` prints them, and **it prints them at the grid**: a `?`
beside the frame's own name on the board, which opens a card pinned to the
viewport.

- **Derived in core, never typed.** `piece::recipe_parts` reads the recipe
  table, so retuning a recipe retunes the line — the same reason `Node::line`
  is derived rather than written into a blurb. Unthemed, TONE 13a: somebody
  comparing what two grids want is comparing counts.
- **The way's name is printed only where there is a choice.** The weapon grid
  has six ways of being built and the other four have one each; naming the way
  on a grid with one way is a label that carries nothing.
- **It was in the panel first, and the panel was the wrong place.** M12.B put
  it above each grid's cards in the right-hand list, which is five boxes down a
  column and a long way from the empty greaves frame the question is actually
  about — and the list is the thing a hover scrolls. Asking at the frame is
  what the panel was standing in for. **An empty grid is skipped in the list
  again**, because the reason it stopped being skipped went with the recipe.
- **A button, not a hotspot painted on the canvas.** It takes focus, so the
  recipe is reachable from a keyboard the way a skill node's detail is — and a
  control drawn into the canvas would be a second thing hit-testing the board's
  pixels. The canvas draws the label, so `Board#helpSpots` **measures** where
  that label ends and reports a spot; `Board#onlayout` fires on every fit, so
  the controls follow a frame that grew a row rather than being placed once.
- **Canvas pixels are not always CSS pixels.** `Board#fit` floors the backing
  store at 560 wide and pins the CSS height to the backing height, so a narrow
  column scales the two axes by different amounts. `frameHelp` asks for both
  and adds `clientLeft` — the canvas's own border, which is a pixel of drift on
  every button if it is left out.

**And the controls blurb at the top of the page is gone**, on the human's ask —
four lines of screen that were read once and then held that space for the rest
of the sitting. It had also started to lie: *"every level adds a row to one
frame"* is exactly what M12.3 retires. The canvas keeps its `aria-label`, which
is where a control belongs for somebody who cannot see the board. **A paragraph
of instructions is a paragraph that has to be maintained like any other string,
and nothing was checking that one.**

## Art

- **TikZ or nothing.** Every figure in `art/` is a standalone document written
  by filling in `tikz_figure_prompt.md`, and the reason is not ceremony: a
  figure that is text can be reviewed, diffed and corrected in one line, and a
  figure that is a PNG can only be re-rolled and hoped over.
- `make art` compiles to `web/assets/*.svg`. **The SVGs are checked in**, so a
  deploy never needs LaTeX; missing tooling prints what to install and exits 0.
  `standalone.cls` is not in BasicTeX and is the usual reason it fails —
  `tlmgr init-usertree && tlmgr --usermode install standalone`.
- **The house style, which is the prompt's "audience" field:** flat fills, heavy
  outlines, no gradients; a figure must read at 64px on the map and again at 4×
  in a panel.
- `data/art.json` maps a canonical creature name or place id to a figure. A
  subject with no entry draws nothing.
- **The creature half of that file is generated — do not hand-edit it.**
  `art/creatures.json` says which family drawing each creature is cut from and
  in what colours; `make art` compiles a figure per creature and rewrites
  `data/art.json` from it. Deriving the map from the manifest is the point: the
  map and the files it names cannot drift, because only one of them is written
  by a person.
- **Families, not fifty drawings.** Thirteen silhouettes — sentinel, bone,
  wisp, hound, idol, mirror, clergy, crown, court, wright, ash, rime, vermin,
  plus the four drawn for themselves — each compiled once per creature with
  `\def\Main{...}\def\Dark{...}\def\Accent{...}` on the pdflatex command
  line, against a `\providecommand` default inside the figure. Two creatures in
  a family share a silhouette and never a palette.
- **`.tex` count ≠ `.svg` count, and that is fine.** A creature whose slug
  equals its family name (Francis) compiles twice to the same file. The check
  that matters is `every_creature_has_a_figure_and_every_figure_has_a_file`.
- **Draw it, then look at it.** Three of the thirteen compiled cleanly and did
  not read: `bone`'s ribs came out as a spring, `clergy` collapsed into a single
  triangle because the mitre sat straight on the robe, `ash` was a stack of
  circles. A figure that compiles is not a figure that works — rasterise the
  set and put your eyes on it.

## Twenty-one expert papers, from one drawing

Reported from play: *"there should be a sprite for all classes and all expert
classes"* — and there were seven against **twenty-eight** things a character can
end up being. So the panel drew a base class's portrait for somebody who had
become an expert, and the second fork named what each pairing reaches and showed
nothing of it.

**An expert is literally a pair, so the drawing is a pair.** `art/expert.tex` is
a stamped paper with two wax seals at the foot of it, one the colour of each
parent class, compiled twenty-one times off `art/experts.json` — the
thirteen-creature-families argument applied to a thing that is a pair by
construction. Nothing about the sheet changes between them.

- **One colour a class, chosen apart** rather than taken from each figure's
  darkest ink: two muddy seals side by side are two blobs. Twenty-one pairs from
  seven colours are all distinguishable by construction.
- **The stamp is the office's own ink and neither parent's.** A stamp wearing
  one parent's colour would say the paper belongs to that half of it.
- **The map is written from the manifest**, so the file and the names cannot
  drift — `art-manifest.py` owns the expert half of `art.json["classes"]` and
  the seven hand-drawn ones survive.
- **`every_class_the_game_offers_has_a_figure`** is the lint, and it is *offered
  classes and experts only*: `class::CLASSES` is the inherited roster and
  carries names GM2D offers from nowhere, so drawing a portrait for a class no
  player can take is art shipped for nobody — the creature half of that file's
  failure, upside down.
- **The panel draws the deepest class you have become.** `classes` is what you
  *are*, in the order they were paid for, so the last of them is the one a
  portrait should be of. Reading `class` drew the level-five figure for somebody
  who had finished two trees and taken the paper.

## The art was drawn and shown nowhere

Reported by the human as *"the png representation of them that we built;
nowhere ever shows it"*, and they were exactly right.

`data/art.json` shipped mapping **three creatures out of fifty**. So a portrait
appeared on the fight screen roughly one time in twenty, and `art.player` —
`sprocketman.svg`, compiled and deployed since M6 — was read by no line of code
at all. Nothing was broken; the map was just almost empty, and an empty map is
indistinguishable from a feature that does not exist.

Two things came out of it, and the second is the one that matters:

1. Every creature has a figure now, and the player's own is in the panel that
   is always up.
2. **Coverage is a test.** `every_creature_has_a_figure_and_every_figure_has_a_file`
   fails when a creature is added without art, when the map names a file that
   is not there, and when the map names a creature that is not in the ladder.
   `check_the_portrait_shows` says the same thing from the browser, including
   `naturalWidth != 0` — a portrait that 404s is not a portrait.

## An ench you were paid and cannot use yet

Reported as *"the quest the frame that stands did not pay the yodregar index"*.
It did. Core handed it over, the save carried it, the town's receipt named it,
and `an_errand_pays_its_ench_to_a_character_who_cannot_use_one` says so. The
rack is the only screen in the game an ench appears on, and it was `hidden`
outright to anybody without the Kaklon Patent — so the errand paid, and no
screen anywhere would show it.

**The engine was right and the screen was the bug**, which is the third time
that shape has been found here: four skills worked and were reported broken, a
starting balance was on the bar and the bar opened at zero, and this. The rule
is the same one and it keeps earning its place — *a thing that works and cannot
be seen is a thing that does not work.*

- **The design already said so and only did half of it.** `quest.rs` hands an
  ench over regardless of licence because *"a reward that vanished for three
  players in four would be worse than one they cannot use yet"* — and that
  sentence only holds if the player can see they have one. This is the other
  half of it.
- **Shown, and not offered.** The rack lists what you own read-only and says
  what it wants: the licence. `attach_ench` refuses an unlicensed character in
  core, and offering a click that is going to be refused is a worse screen than
  not offering it.
- **Still hidden when you hold none**, which is what hiding it was for: a rack
  of nothing on a screen you cannot use is noise. The condition is *unlicensed
  **and** empty*, not *unlicensed*.

## Your figure is your class's

`art.player` is the Sprocketman — who you are before anybody has decided what
you are. The fork is where that stops being true and it does not come off, so
the panel draws `art.classes[canonical]` from then on. Repainted on every
`paintPanel`, so a loaded save arrives wearing its own figure rather than
waiting for the next fork.

## A key that never left the bag, and an event that never shut up

Three faults reported after M11 went live, and all three are the same fault:
**nothing had ever checked what happens on the second visit.**

### A key is spent opening its lock, and the lock stays open

Both halves are load-bearing and the second is the one that matters. There are
exactly two keys — the Witch's Key at the Cave mouth and the Deep Gate Key at
the door in the wall — and the bag was only ever *asked* about them, so a key
you had used sat there for ever.

- **`Game::unlock` is the whole answer**, and it is core's. Whether a gate
  opens used to be decided in the shim, on the grounds that a `World` does not
  know about bags — which is true, and is an argument for not putting it in
  `World`. It was never an argument for putting it in a shim: **a `Game` is
  exactly the thing that holds both a bag and a world**, and *a key is spent*
  is a rule the fast suite has to be able to reach.
- **The lock stays open, and that is not a convenience.** The door in the wall
  is the only way to the back half of the game, and a defeat in the Treyway
  walks you home to West Bambulon. Spent *and* re-locked would end the run
  there, and there is no second key anywhere in the game.
- **An instrument is asked for every time and is never spent.** The Reach's
  whole design is that what you carry changes what you read, so a survey gate
  is never written into `answered`. Only a key is.
- **Read `answered` before the key turns.** `unlock` writes the place down, and
  the gate's paragraph uses that same set to mean *you have been through here*.
  Asking afterwards opens the door and eats the speech in the same step.
- `Character::spend_one` is one function because there are two callers, and
  what it means to give something up must not have two answers: an errand
  handing in a tally, and a key turning in a lock. `quest::hand_in` wrote those
  eight lines first.
- **A key is carried, never worn** — `PieceKind::Quest`, refused by `can_equip`
  with *"that is a quest item - it is carried, not worn"*. So spending one
  cannot strand a cell. An errand's tally *is* seatable, which is why
  `spend_one` takes it off a board first anyway.

### An event that asks nothing is read once

`answer(id, n)` takes the index of the choice you picked, so **an event with no
choices could never reach `answered`** — it re-opened its modal on every step
onto the tile, for ever.

That was invisible while every event in the game asked something. It is 7 of 7
on West Bambulon and 2 of 2 on the Treyway. M11 then added **41 on the
Kettleworks field and 6 on the Reach, and not one of them asks anything** — so
one tile in ten became a toll booth, on the map whose whole texture was
supposed to be reading things.

- **Marked in `world::step`**, beside the line that already computes `spent`,
  because *what counts as having read something* is a rule.
- **Reported as nothing at all the second time**, rather than as a spent event,
  because a page cannot decline to draw a card it was handed. An event that
  *does* ask something still comes back, so you can re-read it and see it is
  spent — that is the nine on the hand-built maps and it is deliberate.
- The design note this file carried was wrong and is corrected above: a dense
  map is a different kind of map, but 41 dismissals that pay nothing and ask
  nothing are not texture.

### The experience nine events promised and none of them paid

`Outcome::Xp` wrote into `world.counters["xp"]`, **which is read by nothing.**
Nine events promise up to 359 points between them and not one ever landed. The
comment above the line dated itself — *"M4 is what turns this into a level"* —
and M4 put experience on `Character` four blocks ago.

It is `character.carry(n)` now, the same primitive a win pays into, and the
receipt says *"+N experience, carried"* in the same words the fight uses. The
playtester had noticed those two phrasings differed and that only one of them
worked, which is the finding underneath the finding.

**Found by somebody who was not allowed to read this file.** That is what
M11.8's harness is for, and it earned its place on its first run: a fault three
milestones old, in a line every reader of the source had skimmed past because
the comment beside it explained why it was right.

## Two types called Outcome, and the describer is on the dead one

**`event::Outcome` is the campaign's and `tile_event::Outcome` is the game's.**
The campaign was cut in `48203ee` and `event.rs` survived it, so the module
still carries upstream's ladder outcomes — `FightAsWritten`, `BuyOff`,
`Claim`, `Step` — with a full `describe()` on them. The type the game actually
uses is `tile_event::Outcome`: `All`, `Gold`, `Flag`, `Give`, `Xp`, `Nothing`,
and it has **no describer at all**.

So the pattern the tile events want was written years ago, by somebody else,
on the wrong type — and its doc comment is the design:

> Static: what this outcome *is*, for a tooltip before it is taken. What it
> *did*, with the run's own numbers in it, is `Run::receipt`.

**`Requirement` is the same story and the more useful half.** The campaign's
carries a `describe` whose doc names a distinction the live type has never
had:

> Not the same thing as `Choice::unmet`, and both are needed. `unmet` is
> flavour written for the moment after you have tried; this is the plain
> statement *before* an attempt.

Which is why a locked choice in GM2D reads as a wall rather than as a target:
it has the flavour and has never had the statement.

That is the split `Node::line()` and a level-up receipt already make, and it
is why a tile event's outcomes box is a port rather than an invention. **Grep
for it, and then check which of the two you found** — this file's rule about
building something twice has a second edge, which is finding the thing and
attaching to the wrong copy.

The live type also already has more reach than any content uses: `Give` hands
over a component and `Flag` sets a world flag that `PlaceDef::hidden_until`
reads, so **gear rewards and event chains have been possible in the data
format since M2 and are used twice and never respectively.** What is missing
is not machinery, it is content and a sentence.

## The game talks in one place

Before M11.0 the game said things in five places and kept none of them. A
receipt scrolled past in the fight panel, a town's sentence lived on the town
screen until you shut it, an errand's brief was in a card you dismissed, and a
tile's event was a modal. **Nothing a player was told outlived the screen that
told them**, so the only way to check what you had just been given was to have
been reading at the time.

- **`log()` is the one door.** Everything the game says goes through it, lands
  on `#tape` — a strip that is always up — and is kept in `history`. `#history`
  is the whole sitting, on the tier above the map.
- **The strip is not a feed, it is the last thing said.** A log nobody can
  reach the top of is a log; a strip that holds three lines and a way in to the
  rest is a game talking.
- **Trap 6, and it took four coats.** `.panel dl > div { display: flex }` beats
  `[hidden]` on specificity, so a row hidden by `el.hidden` stayed laid out and
  every map read "surveying with —". This is the same fault as
  `.screen.framed` vs `.screen[hidden]` two milestones earlier: **anything that
  sets `display` on an element it also hides needs its own
  `[hidden] { display: none }`.** It was found by the agent playtest driver
  before the playtest — the driver reads the panel as text, and text does not
  care what you meant.

## A stale map, shipped since M8

`paintPanel` drew the map the page had last read. Walk into the Cave, lose, and
you are walked home across maps — and the page went on drawing the Cave with
your marker somewhere in the middle of West Bambulon. Nobody had noticed
because before M11 there were two maps and one way between them.

The fix is the class rather than the instance: **`position()` carries the map
id, and `paintPanel` compares it with the map it is holding and re-reads when
they differ.** A page that draws a map has to be told which map, every time,
because the map is the one thing about the player's position that can change
without a keypress.

Two more of the same shape, found in the same sweep: the odds overlay and the
walker's pathfinder each carried their own hardcoded list of impassable terrain
(`rock`, `water`). Both read core's `walk` grid now. **A second list of what
you cannot stand on is a second answer to a question `World::walkable` already
answers**, and M11.4 changed that answer.

## A reveal scrolls everything above it

Reported from a real session: pointing at a component on the packing board made
**the board itself jump up the screen**, so a grid you were editing walked out
from under the cursor. Reported as random; it was not — it happened on every
hover that lit a card the panel had to scroll to.

`lightCard` finished with `target.scrollIntoView({ block: 'nearest' })`, and
**`scrollIntoView` scrolls every scrollable ancestor, not the nearest one.** On
this screen there are two: the card list on the right, which is the box that
wanted to move, and `.stages`, which is the box the board is standing in.
Measured against the old build at 1280×620, one hover took the canvas from
249px down the viewport to 69 — 180 pixels, in the middle of a drag.

- **`revealInside` is `block: 'nearest'` written out for one box**, and
  `scrollBoxOf` finds it. Do nothing when the element is already readable;
  otherwise move the least that makes it so, capped so a card taller than the
  box arrives top-first.
- **Whether the box is overflowing right now is not the question.**
  `scrollBoxOf` matches on `overflow-y` alone: asking `scrollHeight >
  clientHeight` walks straight past a panel that happens to fit and hands back
  the stage behind it, which is the bug with an extra step.
- **`overflow: hidden` does not mean unscrollable.** It means the *player*
  cannot scroll it. A script still can, and nothing scrolls it back — which is
  why `.screen.framed`'s hidden overflow was no protection here.
- The history list had the same call for the same reason and goes through the
  same door. **Fix the class, not the instance** — there is one reveal now, and
  the next list that wants one will not have to rediscover this.

## The world the page is holding is not the world it loaded

`main()` read the world into the page and then restored the autosave, in that
order, and never read it again. So the copy every screen drew was the **fresh
game's** — the one the module starts with, where `answered` is empty and every
`hidden_until` place is therefore hidden.

Reported from a real save: *"when I reloaded the browser, the door to the
treyway from the end of all gears disappeared, so i cant leave anymore in my
save until I go into another menu like the tree, then it reappears."* Which is
exactly right, and the second half is the diagnosis: every screen in this game
re-reads the world on its way out, so opening and closing anything put the door
back. A player who never opened the tree could not leave the map.

- **The same fault as the stale map, one step earlier.** That one was a page
  drawing the map it last read while the player was moved to another; this is a
  page drawing the world it read before the save was loaded. Both are the rule:
  **a page that draws a world has to be told which world, every time it can
  have changed** — and *a save being restored* is the largest change there is.
- **It is invisible to anything that plays a new game.** The gate walks from a
  fresh start, where the page's copy and core's agree, and every check was
  green through it for as long as the door has existed.
  `check_a_door_survives_a_reload` plants a save with the Cave answered,
  reloads, and asks the page — **before touching anything**, because a check
  that clicks first cannot tell a page that had the door from one that went and
  fetched it.

## Screens, and the three times one covered another

Three bugs, one shape, and **not one of them was visible by reading the
source**. All three were found with `document.elementFromPoint`.

1. **`.card` is the event dialog** — `position: fixed`, `inset: 0`. The item
   cards were given the same class and every one of them became a
   full-viewport overlay pinned over the game.
2. **`.screen.framed` and `.screen[hidden]` have equal specificity**, so the
   later rule won and a *hidden* fight screen stayed laid out over the whole
   page. The town's Spend it button was visible, enabled, and swallowed by
   `#run` from a screen nobody could see. Anything that sets `display` on an
   element it also hides needs its own `[hidden] { display: none }`.
3. **Every `.screen` sat at `z-index: 20`**, so which covered which was decided
   by the order of the file — and the town comes after the fork. A level lands
   when you bank, banking happens with the town up, so the class fork opened
   *underneath* it: four cards on screen, none of them clickable, and the game
   unfinishable from level five. Taking a class then opens the tree, which was
   under the town for the same reason.

The stack is written down now, and each tier is a sentence about what a screen
is:

| z-index | what it is | which |
|---|---|---|
| 20 | where you are | the fight, the town, the map's card |
| 30 | what you opened from there | the tree, the log, the ending |
| 40 | the one that does not come off | the fork |

**A screen you cannot dismiss must be the top-most thing on the page.**
`check_the_fork_is_on_top` is what stops a fourth.

Three about the harness rather than the game, and the last two cost a deploy.

**A planted board check has to strip every grid, not the one it plants on.**
These run late in the walk on a character who has fought fifty times, and since
M9 a character who has fought can have *earned* a set — the drops are the
block's whole point. So `check_a_set_reads` asked `Character::rules` a question
about the board it had just planted and got an answer about the board the walk
had built: "two thirds of the set still grants 1 rule" was the Toad's Own Frame
sitting in a chest nobody had looked at, and the lake let a dry character
through for the same reason. Both were green on a laptop and red in CI, because
what a walk earns depends on the seed. `strip_the_boards` is the fix and it is
the rule: **a planted check is about what was planted.**

**The fork grind never went home, and had not since M5.** `PATROL` is six east
and six west, and a blocked press does not move — so from the town's own tile
the westward half is spent against the map's edge and the walk drifts east a
tile at a time until it is fifteen away and can never find the town again. A
fight used to level you on the spot, so that cost nothing; a town has been the
only place experience becomes a level since M8, and the instrumented run that
found this ended **carrying 1,115 experience at level 2 after 255 fights**.
`head_for_town` was written for exactly this and wired into one of the two
loops. It reported as *"never reached the class fork (level 3)"*, which reads
like a flake and is not one — the lesson is that **a gate failure that looks
random is a gate failure nobody has instrumented yet.** Raising the press budget
was the wrong first move and made it look worse.

**`plant` waits for the load rather than sleeping through it.** The page's file
handler is `async`; four hundred milliseconds is a guess, and a guess that is
wrong reads every assertion after it against the previous game.

**A check that needs something to happen has to make sure it can.** The first
version of the broken-item check walked into whatever the ground rolled — and a
weapon's bar is two to four seconds against a rat that dies in less, so the item
under test never came round. It passed once and then failed in two browsers of
three. It plants the fight now: bolt the ench on, download *that* save, and
re-plant it with a Rust Colossus standing in front of you.

**Taking a class off a save is not the same as not being a licensee.** A
level-five character with no class is *owed* one, so the load opens the fork —
the one screen that does not come off — and every click after it lands in a
modal. A plant that wants an unlicensed character sets a class that is not the
Patent's, rather than removing the field.

And the old one: **a check that opens a screen has to close it on every path
out.** A check that appended a failure and
returned early left the screen up, the next check died on a click it could not
land, and the whole failure list went unprinted — so the run reported a
Playwright traceback and not the one sentence that said what was wrong.
`walk_the_gate` takes its `fails` list from the caller now, so a crash cannot
take the findings with it.

---

# Part six — shipping it

## A deployed fix is not a delivered fix

Pages serves `index.html` with `Cache-Control: max-age=600` and everything else
is content-hashed, so a browser holding a stale entry point keeps loading the
**old** `app.js` and the **old** wasm from URLs that are served forever. The
position-repair fix was live, verified against the deployed site, and still had
not reached a player whose tab was pinned to the previous `index.html`.

`app.js` carries the build stamp it was packaged with, fetches `index.html` once
with a cache-busting query, and if the stamps differ navigates to
`?v=<live>` — a **different URL**, not `location.reload()`, which is allowed to
re-serve the same cached document and would loop. `sessionStorage` guards
against a genuine mismatch looping anyway.

`packaging/package-web.sh` fails the build if the stamp is not applied.

**And the self-heal only ran once, which is the other half of the same
problem.** `freshEnough` fires at load: it covers the tab opened *after* a
deploy and none of the ones that were already open. A sitting is hours and a
deploy is six minutes, so the common case is somebody looking at a page that
was current when they started — which from a chair is indistinguishable from
the fix having never shipped. It was reported that way twice in one block:
*"there is no button in the autobattling view that allows you to set the speed
of battle and view the combat log"*, against a build that had shipped all four
buttons, measured on screen at every viewport from 800×900 up.

So the page keeps asking — every five minutes, and on the way back to a
foreground tab, because somebody returning after an hour away is the likeliest
person in the world to be holding an old page. **What it must not do is
navigate.** A page yanked out from under somebody mid-fight loses the fight,
and a stale page is a smaller problem than that; it says so instead, once, on
the strip through `log()`, and then stops asking.

**And a check that reaches a button by id is not asking the question the player
asked.** The playback controls were driven by `page.click("#combat-log")`,
which passes just as happily on a control that has wrapped off the bottom of
the bar. It measures the rects now — un-hidden, laid out, inside the window —
and returns before the clicks if any of them is unreachable, because a click
that times out ends a check with a Playwright traceback instead of the sentence
that says what is wrong.

**Deploying is three things, and finishing the first is not finishing.**
`make publish` runs the engine suite and pushes; Actions then runs the suite
again, builds, **walks the full gate in three browsers**, and only then uploads
the Pages artifact — so a red gate stops a deploy rather than publishing one.
That took 6m02s for M8. The third thing is a person, or a script, loading the
live URL and asking a player's question of it. M8's was:

```
live build 7808e551
  ok  Errands button    ok  ench rack      ok  ending screen
  ok  log screen        ok  scout button   ok  one action bar
  ok  #numbers gone     console errors: none
```

M10's was:

```
live build f5654c7e
  town bench gone             yes     van below level 10       no
  van at level 10             yes     walking on opens a bench yes
  he has                      Plug Energy Tap, Grungo Elastic Band, Sneel Bearing
  five classes offered        yes     all five fit 1280x720    yes
  console errors: none
```

M12's was the same tool pointed at the same place, and the interesting part is
what it did **not** need doing by hand:

```
live build ccfeb16d                   and d8965cf7 after the doc sweep
  index.html asks app.js?v=<X>        app.js carries BUILD='<X>'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok
  46 checks, three engines            the frozen save loads and walks
  the barrel is under the counter     a grid says what it takes
  console errors: none                off-origin requests: none
```

The first deploy after the block was two reported UI faults, and its table is
the shortest one here because the gate asked forty-six of the questions:

```
live build 07a29306
  index.html asks app.js?v=07a29306   app.js carries BUILD='07a29306'
  chromium walked the gate    ok      firefox  walked the gate    ok*
  webkit   walked the gate    ok
  pointing at a seated item: board top 249 -> 249, and its card lit
  five ? beside five frames           each says what its frame takes
  console errors: none                off-origin requests: none
```

**\* and the asterisk is the finding.** Firefox failed one check on that walk
and passed it on a re-run a minute later, which is the shape this file has
called a flake worse than a red. It was not the page:
`check_the_frozen_save_is_playable` uploaded the fixture and then waited for
**any** `#tape` line reading *Loaded* — and the upload block two steps above it
loads the walk's own save, which logs exactly that, on a four-line strip. So
the wait was satisfied by the previous load before the file had been parsed,
and the map read a moment later was the walk's own.

- **Wait for the thing you are asserting.** It waits for the position to be on
  the field now, with the same ten seconds to get there, and one sentence
  covers both of the old branches — a file that never loaded and a file that
  loaded somewhere else are the same sentence with a different map in it.
- **The check raced on every run in every engine and usually won.** Nothing
  about firefox was wrong; what it did differently was fetch the file over a
  network while the assertion after it did not. **A green suite is not evidence
  a check is not racing**, and the only reason this one was ever seen is that
  the live walk is slower than the local one.
- Nine passes against the live page since — three walks, three engines — and no
  recurrence. That is evidence and not proof; what makes it a fix is that the
  race no longer exists to lose.

The deploy after that one is four reported faults and a feature, and its table
is short for the same reason — the gate asked fifty-four of the questions, in
three engines, against the live page:

```
live build c236ca4c
  index.html asks app.js?v=c236ca4c   app.js carries BUILD='c236ca4c'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      54 checks, no failures
  town buttons: 'Inventory / Character Sheet' and 'Level up'
  the bank: 2 loose in the bag, the vault opens, and Kettleworks opens the same one
  the inventory sheet: 3 rows, and they move when you pack
  playback controls, board stage: speed only — the other three are the replay's
  playback controls, replay stage: all four, un-hidden and inside the window
  console errors: none                off-origin requests: none
```

**The door check is the one that matters here**, and it is in the gate rather
than in this table: it plants a save with the Cave answered, reloads, and asks
the page before touching anything. That is the reported fault reproduced, and
it is green on the live build.

The deploy after that is the Reach's own frame, and its table is the first one
here whose interesting line is a *zero*:

```
live build 502ea52d
  index.html asks app.js?v=502ea52d   app.js carries BUILD='502ea52d'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      55 checks, no failures
  the edge refuses            and opens the frame, carrying the shut prose
  the frame's bag             3 parts, and nothing that cannot go on it
  a compass built there       reads "-20% on how often the ground stops you"
  the weapon grid             0 pieces moved
  going in                    the-reach, panel reads "compass — -20% encounters"
  console errors: none                off-origin requests: none
```

**The zero is the report.** Building an instrument took nothing off the board
you fight with, which is the whole of what was asked for.

And the one after it is the tree, whose interesting line is *seventeen*:

```
live build a418bea0
  index.html asks app.js?v=a418bea0   app.js carries BUILD='a418bea0'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      55 checks, no failures
  the tree                    22 nodes over nine tiers
  its wires, on first open    17 drawn, 17 with real coordinates
  the new tiers read          "+1 row on every grid", collapsed by Node::line
  console errors: none                off-origin requests: none
```

**Seventeen with real coordinates is the whole of the wire fix**, and the
number the old gate could not tell from seventeen at the origin.

**The deploy after M13 is the barrel, and its table is one list.** The gate asked
sixty-three of the questions against the deployed page in three engines; what a
person went and looked at is whether the casting family had actually reached a
counter, read off the page's own `__shopJson` rather than off the data file:

```
live build 5fb93594
  index.html asks app.js?v=5fb93594   app.js carries BUILD='5fb93594'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      63 ok lines, no failures
  the barrel, in the pit      16 lines, and four of them are the casting cores:
                              Guidance Sheet 25, Minus One Degrees 45,
                              Copy Paste Race 50, Blizzard Globe 35
                              — a book, two spells and a ball, at x1
  console errors: none                off-origin requests: none
```

**Whether they *assemble* is core's question and stays there.**
`every_recipe_assembles_out_of_the_barrel_alone` seats them and asks the recipe
table; a browser cannot say anything about that a hundred and fifty milliseconds
of `cargo test` does not say better. What only the live page can answer is
whether the counter is carrying them, which is what the list above is.

**The deploy after that is five asks and nothing is free any more.** Ninety-six
of the questions are the gate's, in three engines against the deployed page.

```
live build e44f5a7b
  index.html asks app.js?v=e44f5a7b   app.js carries BUILD='e44f5a7b'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      96 ok lines, no failures
  the pack, three kinds       Cork Tea — Takes 10% of the tiredness off.
                              The Quiet Word — Running from the next fight
                                costs you nothing. Spent when you run.
                              The Short Way Back — Puts you in the last town
                                you stood in.
  the Drowned Gallery         stones [[3,6],[6,4],[8,7]] on marks
                              [[3,2],[8,2],[5,8]]; the stair is not there; one
                              push moves a stone to [3,5] and the player in
                              behind it
  the Cairnfield              walked onto a cairn you cannot use: 55% -> 65%,
                              which is past the sixty a fight stops at, and the
                              strip says "Nothing here for you, and the walk
                              cost 10%."
  console errors: none
```

**The Gallery's line is the one worth reading twice.** It is the only puzzle in
this game that is not monotone — a stone in a corner is exactly the move the
rule forbids — and what makes it safe is that the room forgets: `go_to` is the
one door onto a map and it drops the stones, so walking up the stair and back
down reseeds them. Reseeding on the map *id* was not enough, and a test caught
it before a player could.

**The deploy after M15 is four asks and two faults, and the second fault was a
soft-lock somebody was standing in.** Ninety of the questions are the gate's, in
three engines against the deployed page; the hand-written half is the four
things a gate cannot ask.

```
live build b09d185a
  index.html asks app.js?v=b09d185a   app.js carries BUILD='b09d185a'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      90 ok lines, no failures
  the bestiary                two met, two listed; the Iron Abbot's entry is
                              six defences, four item cards and a board
  the shore                   "…nobody has cut the tenth. It is THE NINE
                              SURVEYS that opens it, through the edge of the
                              Wextreen Reach."
  the cart                    three stops in fourteen steps, which is the
                              five-step stay
  the weighed door            a save with the bottom option taken loads with
                              `answered` empty, the card reopens, and "Set the
                              counterweight off" takes and sets the flag
  console errors: none
```

**Two faults were found by hand-checking a deploy that had already gone out**,
which is the argument for the hand-written half of this table existing at all:
the bestiary was printing *144% mind resist* against a fight that clamps at 100,
and the Wextreen Sump's weighed door could be shut for good by the one choice in
the game that does nothing. Neither is something the gate had a question for
until it had happened.

**M15's is the first table here whose hand-written half found a fault in the
block that had just shipped.** Eighty-one of the questions are the gate's, in
three engines against the deployed page, so what a person went and looked at is
the reported fault, the curve, and the one thing a gate cannot ask: whether the
sentence the report was about actually reaches a player.

```
live build f4db47e5 (and a0dc7fe0 before the correction)
  index.html asks app.js?v=f4db47e5   app.js carries BUILD='f4db47e5'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      81 ok lines, no failures
  the shore, south from       "The water is over the bar and the bar is nine
  the tideline                 feet down. Eleven years of notches on the post
                               say it goes out the year somebody cuts the
                               tenth, and nobody has cut the tenth."
                              — which is the reported fault, answered
  the curve, off the page     131 banked -> level 4     4297 -> level 19
                              132 banked -> level 5     4298 -> level 20
                              the level-five contract exact, and level twenty
                              at a quarter of what it cost
  console errors: none
```

**And the same sitting found the block's own mistake.** The probe for the
*lake's* refusal planted the character on `[8, 10]`, the page put them back at
the pit, and the reason is that `[8, 10]` is water: **the grating is in open
water and has no standable neighbour**, so a `shut` on it is a sentence nobody
can ever read. The lint that made it exist was asking *"is this on impassable
ground"* where the question is *"can a player be refused here"*. Corrected the
commit after the deploy — the dead sentence is deleted, the lint is sharpened
and asserts its one exception by name, and the claim that *two* gates were
silent is corrected to one. **The deployed build carries an inert paragraph in a
map file and nothing else.**

*A lint that forces content into existence has to be sure the content can be
reached* is the general form, and it is new here.

**M17's is the first deploy this project has had *stopped by its own gate*,
and the second table here where every question was the gate's.** Eighty-five
checks in three browsers against the deployed page, and what a person went and
looked at afterwards is the pair and nothing else, because the gate asked all
of it.

```
live build fac56a16
  index.html asks app.js?v=fac56a16   app.js carries BUILD='fac56a16'
  chromium walked the gate    ok      85 ok lines, no failures
                                      (the same 85 green in firefox and
                                       webkit before the push)
  the first attempt            FAILED — and it never reached the page:
                               "FAIL: webkit: pulling the cue back drew
                               nothing", one engine of three, on a check
                               that passes standalone in that engine
```

**The webkit failure is the block's best finding and it is about the harness.**
`page.mouse.move` takes **viewport** coordinates, so after seventy-eight checks
have scrolled the page an end of the cue drag is off-screen — **webkit clamps
it and chromium does not** — the pull lands under the drag threshold, and no
cue is drawn. Standalone, in either engine, it passes. `pull_to` scrolls the
map into view and refuses the drag outright if either end is outside the
window, with the numbers in the message. *Measure, do not assume*, which is the
same sentence the playback controls earned.

**And a push during a deploy cancels it.** `94efff9`'s Pages run was
`cancelled` when `34f2dcd` was pushed on top of it, which cost one three-browser
walk and nothing else — both commits carry identical engine and page code. The
deployed sha is the later one. **Read the run you mean**: `gh run list --limit
1` returns the most recent run of *any* workflow, which here was `test`, and a
watcher pointed at it reported a green deploy while the deploy was still
building. That is the *read the exit code, and never a pipeline's* rule with a
second workflow in it.

**M19's is five things reported from play, and the table is one line per ask
because the gate asked all of them.** Ninety checks against the deployed page.

```
live build d2bc84c1
  index.html asks app.js?v=d2bc84c1   app.js carries BUILD='d2bc84c1'
  chromium walked the gate    ok      90 ok lines, no failures
  the ball slides             drawn on 19 frames, the trail growing 1 to 15
  a diamond catches           the-road-west, and it let you in to
                              kettleworks-field
  the long cart               runs to a town you have stood in, 40 Fnorp, and
                              the page arrives with it
  the furnace on the bar      10 burns off faith, up to x10
  the glossary                opens on G — 5 shelves, 29 things you can become,
                              and the fight shelf says what empowerment does
  console errors: none
```

**Three of the five were bugs rather than missing features**, and the worst of
them was invisible for a milestone: the Kettle-Stoker dealt **746** against a
classless character's **746**.

**M16's is the first table here where every question was the gate's.** Seventy-
eight checks in chromium against the deployed page, six of them the block's own
and every one negative-tested; what a person went and looked at afterwards is
the part a gate cannot be, which is whether the numbers under the floor are the
numbers the block claims.

```
live build e3a2193f
  index.html asks app.js?v=e3a2193f   app.js carries BUILD='e3a2193f'
  chromium walked the gate    ok      78 ok lines, no failures
  the way under the flat      not drawn at all before THE TENTH SURVEY is
                              answered; a gate on [11,5] after it
  a stake                     two choices — "Pull it" and "Read the flat with
                              the compass" — and the compass sets nothing
  the north sinkhole          put the player at [15, 1], which is an alcove
                              with no walkable neighbour but the lever beside it
  the Tenth Surveyor          her panel drew 12 item cards off the run's own
                              thirty-eight components
  the fork                    seven cards in 2 rows, every one clickable where
                              it is drawn, and Escape still does nothing
  a Stoker's replay           said what the furnace took and what it bought
  console errors: none                off-origin requests: none
```

**Three numbers a person checked by hand**, because they are claims about a
fight rather than about a screen: the run repacked out of everything it owns
still tops out at an item rating of **50**, `geared_from` reaches **135**, and
the Tenth Surveyor deals **221.2 damage a second** against that board — between
Marbulon's 138.7 and Gilt's 428.3, which is the band where a fight is a fight
rather than a wall.

**M14's is the first table here whose hand-written half is a walk rather than a
list.** Seventy-eight of the questions are the gate's, in three engines against
the deployed page, so what a person went and looked at is the thing a gate
cannot be: whether the two new dungeons can be *played*.

```
live build 4d065d1d
  index.html asks app.js?v=4d065d1d   app.js carries BUILD='4d065d1d'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      78 ok lines, no failures
  the Wextreen Sump           walked end to end by `make play`:
                              wheel C jammed with a Morning-Rush Mold — the
                              wheel that is not needed — then wheel A, the
                              bearing off the plate with a compass, wheel B;
                              the weighed door taken with an epic item; the
                              cairnfield walked once with the clipboard
                              THE ONE WHO WENT DOWN, 2053, three drops
  the Silt Stair              the groove takes a 1x4 and keeps it; the chair
                              goes face, four, back and the north wall opens;
                              chain A floods it and chain B brings it back
                              WHAT SHE FACED AWAY FROM, 2123, three drops
  the shore                   its own map since the report; the bar of shingle
                              is drawn from the first visit, is a wall until
                              the tenth cairn, and crosses to `the-low-water`
  the canvas                  the same shape as the map it is drawing, on a
                              square one and on one that is not — which it has
                              not been since the Great Gear Cave
  console errors: none                off-origin requests: none
```

**The transcripts are the deliverable and they are checked in** —
`testing/transcripts/m14-*.txt`, three of them, and the first is the finding:
`make play` from a new game **plateaus at level eleven against the Drambus
Stack's fourth floor** and never reaches a thing this block added. That is
`PLAN.md` §6d row 3 and it is a fact about the walker.

**M13's is the shortest table here and the most of it is not in the table.**
Sixty-three of the questions are the gate's, walked against the deployed page in
three engines, so what is written out by hand is only the part a person went and
looked at:

```
live build ac2256c7
  index.html asks app.js?v=ac2256c7   app.js carries BUILD='ac2256c7'
  chromium walked the gate    ok      firefox  walked the gate    ok
  webkit   walked the gate    ok      63 ok lines, no failures
  the van, one tree finished  three papers drawn; the expert one greyed,
                              "He wants 2 finished class trees, and you have
                              finished 1 of the 1 you are."
  the second fork             four cards, each naming what its pairing reaches
  Escape                      closes it, and the paper is still in the pack
  reload                      it does not come back; the sheet offers it
  two trees finished          "The LoudCalculation paper", nothing to pay
  taking it                   a fourth tab, promise at its head
  one point on lc-rate-card   "2.0 strength a point of mana" -> "1.6"
  a Full Bill's rack          "Two enchs a component", and two went on
  console errors: none                off-origin requests: none
```

**The promise moving is the line worth reading twice.** It is the whole of what
M13.6 fixed one layer down — `CLASSES` is the roster before a point is spent, and
a tab printing it would have said the same sentence after twelve points as
before them. And *"a point of mana"* is M13.9's: that sentence said **Funny**
until a lint read the theme instead of a list, and Funny is that theme's word
for magic.

**A stamp is not a commit and this block moved it twice.** M12.6 deployed
`ccfeb16d`; deleting `STARTER`, `seat` and `ROTATION` a commit later rebuilt
the wasm and the stamp became `d8965cf7` — *dead code is still bytes the
browser caches.* So a sentence in this file naming the live stamp goes stale on
the next deploy whatever that deploy was for, and the two named here are a
record rather than a claim about now. **What has to agree is the page with
itself**, which is why the table above asks for the pair and not the number.

**Most checks pass silently.** Fifty-three `ok:` lines came back for
forty-six checks, which does not divide and is not meant to — and sixty-three
come back now for the same reason: only some of them announce, and the four M12
added and two of M13.8's five are among the quiet ones. **Read the exit
code, and never a pipeline's** — `drive.py … | tail -30` reports `tail`'s
status, which is zero whatever the gate found. The first live walk of
`d8965cf7` was run that way and had to be run again to mean anything.

**The barrel check is the one worth reading twice.** It carried its own `12`
for the dearest thing the barrel may hold, and every price in the game had just
gone up fivefold — so it failed thirteen lines in three engines, thirty-nine
red lines for one stale constant. It reads `shop::BARREL_CEILING` off
`window.__shopJson()` now. **A gate that hardcodes a number core owns is a
second rulebook with a longer feedback loop than the first.**

M11's was the first that nobody assembled by hand. `GM2D_ORIGIN` points the
gate at the deployed page, so the live check is all forty-two questions in
three engines rather than the four or five somebody thought of on the day:

```
live build 43804e49
  index.html asks app.js?v=43804e49   app.js carries BUILD='43804e49'
  chromium walked the gate    ok
  firefox  walked the gate    ok
  webkit   walked the gate    ok
  the eight new figures       all served
  surveying, with none        hidden
  console errors: none
```

**The hand-written table is not obsolete and should still be written**, because
the gate asks the questions somebody already thought to encode and a deploy
note should also record what you went and looked at. What changed is that the
floor is now fifty-five rather than zero.

M9's was:

```
live build 20ac2295
  crossings on the map        2       what they ask  [5, 9]
  the north is shut           yes     the set names its item   yes
  the card says what it does  yes     the sheet says it too    yes
  console errors: none
```

**The live stamp will not match the one your laptop built**, and that is not a
fault: the hash covers the built wasm, which is not bit-identical across
toolchains, so CI's number is CI's. What has to match is the page against
*itself* — `index.html` asking for `app.js?v=X` and that `app.js` carrying
`BUILD = 'X'`. When those two disagree the self-heal navigates; when they agree
a pinned tab is carried forward. Check that pair, not the number.

The failure this catches is the one that has happened here: **the work was
never deployed at all.** M8.0 through M8.8 sat local for a whole block while
`origin/main` was still on the plan document, and the first anybody knew was
the human saying they could not see the quest log. `git log origin/main..HEAD`
is the check, and it costs nothing.

**The stamp hashes everything the browser caches**, and two holes have been
found in that line by hand rather than by a test:

1. The modules were listed by name, so `shape.js` was left out when it was
   added — and its own import of `board.js` with it.
2. `index.html` and `styles.css` were left out, so a markup- or CSS-only change
   produced an **identical stamp**. `styles.css?v=…` kept its old URL and was
   served from cache for ever, and the entry point's self-heal never fired
   because the two stamps matched. A CSS fix could ship and never reach anybody
   who had already loaded the page.

The hash is taken **before** stamping, which is what makes it stable — the
`?v=` values and `__BUILD__` are written into those files afterwards.

## The block was unfinishable, and 597 tests were green

The single most expensive thing M11 found, and it was found by measuring rather
than by playing. Every milestone from M11.1 to M11.6 had shipped content, every
suite was green, and **the block could not be completed from its own first
tile.** Floor two's boss and seven of the nine new pools could not be beaten by
the best board the game hands out.

The reason the suite was green is the finding, and it generalises past this
game entirely:

> The old reachability assertion was `(2..5).contains(&taken)` — *between two
> and four of these fights are winnable*. **A range whose lower bound is "not
> everything works" cannot tell a cost from a wall.** It passed on a tower
> nobody could climb, and it would have passed on one nobody could enter.

What replaced it measures the whole ladder against `common::geared_from` — the
board a player actually has at that point, not a fixture and not a full one —
and asserts against that. Retuning the block around the result is what M11.7
was.

Three things worth keeping out of it:

- **A content block needs a reachability measurement, not a reachability
  opinion.** The check has to name the board it is measuring against, or it is
  measuring the fixture.
- **This is the third time `draw_enemy`'s weighting has cost this project a
  day.** A pool's hardest member is its rarest, so *what a region contains* and
  *what a region deals you* are different questions, and every check that asks
  the first one is answering about a game nobody plays. M9.4 found it for drop
  rates, M11.7 for difficulty, M11.9 for sets.
- **Fixing it retuned content, not the engine.** Nothing in `combat.rs` moved.
  A block that has to change the fight to be finishable has a different problem.

## Somebody who did not build it

M11.8's deliverable is not a feature. It is `testing/agent_driver.py` and
`testing/AGENT-BRIEF-M11.md`: a way to hand the built game to an agent that has
been **forbidden the source, the data, the tests, the plan, this file and the
git log**, and given only what a shop poster could tell it.

- **The prohibition is the instrument.** A run that reads `world.rs` measures
  the reader's understanding of `world.rs`. The only file in the repository the
  playtester may open is its own brief, and if it wants a number it cannot get,
  *that is the finding* — write down that you wanted it.
- **One command per turn, browser open between them.** `start`, `look`,
  `panel`, `buttons`, `key`, `click`, `save`, `load`, `stop`. `look` is a real
  screenshot, because the map, the board and the fight are canvases and reading
  the DOM cannot see them.
- **It runs cross-process against the served build**, over CDP, which is why
  `GM2D_WEB` had to exist: the gate rebuilding `dist/web` underneath a playtest
  moves the save fingerprint and ends the sitting.
- **`buttons` before you click** — it lists every clickable thing on the
  topmost screen with its selector. A driver that makes the agent guess a
  selector produces findings about the driver.

The findings land in `PLAYTEST-M11.md` and the triage in `TRIAGE-M11.md`, which
is twelve findings scored severity × cost; five were fixed in M11.7 and seven
are carried with the reason written down.

## Tone, as a lint

`tests/tone.rs` holds the eight rules from `TONE.md` a machine can check. Not
the ones about register — those need a reader — but the ones that are facts
about a string. Every one caught something on its first run:

- **Rule 13** found a blurb saying "armour" twice where the game says Cork.
- **The blurb/effect check** found a node promising a row on two frames and
  granting one. A blurb that overstates its effect is the worst kind: the
  player finds out by not getting it.
- **Rule 12** was itself wrong first, and failed two lines that were perfectly
  clear — "Forty Fnorp" names forty, and spelling small numbers out is the
  house style. The lint learned to read numbers as prose.

## Divergences from the brief

`PLAN.md` wins. These are the places it does, and why.

| # | Divergence | Where |
|---|---|---|
| 1.1 | Fork by copy with a provenance file, not a git subtree. And the campaign is dropped, not carried: eleven modules deleted, `Run` replaced by `Character`. | `PLAN.md` 1.1, `crates/core/UPSTREAM` |
| 1.7 | §C.1 is a **design change, not a bug fix**. Upstream paid the bounty on a loss deliberately and its reasoning was sound *on a ladder*. GM2D is not a corridor, so the justification goes and the exploit stays. | `crates/core/src/reward.rs` |
| 1.9 | The theme becomes data. `theme.rs` already treats a name as a key rather than a label; moving its tables to `data/` is where they belong. | `PLAN.md` 1.9 |
| 1.10 | Actions builds and publishes to Pages. No `docs/`, no human-run `make publish` rebuild. The brief described gear-master, which predates both house web repos and ships macroquad. | `.github/workflows/deploy.yml` |
| 9.1 | **A crossing guards a region, not its own tile.** `PLAN-M9.md` §M9.3 wrote it as a tile you may pass; the map is twelve tiles of open ground across every boundary, so that is a dozen crossings in a row, which is the wall the same section rejects. | `crates/core/src/world.rs`, `PlaceKind::Crossing` |
| 9.2 | **The third set grants a `Rule` rather than an assembly trigger.** §M9.2 proposed the trigger because it costs nothing; it also pays on any assembled item holding the piece, which is not a *set* bonus. The rule it grants is one the tree has granted since M8.3, so it still costs no combat code. `Rule` kinds went 5 → 7, not the table's 8. | `crates/core/src/piece.rs`, `WEAVE` |
| 10.1 | **No divergences from `PLAN-M10.md`.** The row is here because its absence is worth stating: every deliverable landed as written, the three questions the human answered are folded in, and the two decisions the plan delegated — where the van's stock lives, and whether he restocks — were made in the commits that made them. What the plan did **not** contain is `ClassPower::Recycler` being dead, which is not a divergence but a discovery: see *Two promises that reached nothing*. | — |
| 11.1 | **The overworld brackets levels 12–16, not 5–9.** `PLAN-M11.md` §M11.1 asks the Treyway's pools to bracket five to nine; that number was written before recon. The door is behind the Cave and the Cave is behind a crossing that asks for nine, so a player who reaches it is level twelve at the earliest — bracketing at five would have put a whole continent below the map it opens off. | `data/maps/the-treyway.tiles.json` |
| 11.3 | **No counter, and no `tower_dropped` flag.** §M11.3 asks for `tower_floors_cleared` in `WorldState`. Beating a boss already writes its tile id into `answered`, so *how many floors are gone* is how many of those are there — derived, never banked. And floor one is reachable only once the four above it are down, so its boss being answered **is** the tower being down; M11.4's lake reads that id. | `crates/core/src/world.rs`, `PlaceDef::floors` |
| 11.4 | **The flooded under-lake is one map read twice, and what the early way costs is the walk.** §M11.4 proposed a second variant whose water rows make the fight "positioned worse"; combat has no board, so positioning cannot cost anything. The map drains its own middle rows on the same flag, so entered early the straight run is shut and the way down is twenty-one tiles of slag against eleven of road — which is fatigue, and fatigue is the only currency a dungeon here has. | `data/maps/under-the-lake.tiles.json` |
| 11.6 | **The golem's fallback was taken**, which §8 row 6 named in advance so that taking it would be a decision rather than a retreat. It handles one fight an entry rather than standing as a third board in the replay. The reason is not the layout — it is rule 5: a third board is a third set of numbers the page must not invent, and the honest version is a third combatant in `combat.rs`, which is new combat code in a block that has added none. | `crates/core/src/survey.rs` |
| 11.9 | **Six sets, not seven, and the tower keeps its borrowed bosses.** §8 row 8 puts three sets on tower floors; a floor is one sitting, so a set off a floor's *pool* is unfarmable — the tower carries one set instead, three certainties off floors five, three and one, so climbing is the grind. And §M11.9 asks for distinct bosses on the floors that shipped on borrowed frames: M11.7 measured every one of those frames against the board the game hands out and retuned the block around the result; re-dressing them threw that measurement away, and the first attempt produced a boss nothing could beat. The eight new creatures stand in *pools*, where a new face costs nothing that has to be re-measured against reachability. | `crates/core/src/combat.rs`, `crates/core/src/piece.rs` |

| 12.3 | **No ledger for a granted row, and two MVP pillar tests retired.** `PLAN-M12.md` asks for granted rows to be banked in the save; `BoardSave::rows` already is, and `resize_boards` only ever grows, so an old file keeps what it earned without a migration. What a row came *from* stays derived from `skills_taken` and `quests_done`. The two tests asserting *level N implies board B* are gone rather than repaired — that guarantee is what the milestone removes. | `crates/core/src/progression.rs` |
| 12.5 | **Every root choice hands over its own errand**, which the plan does not ask for. §M12.5 asks events to pay something and say what they pay; that was built and was still not a decision, because both branches of a root opened invisible content. The errand is the visibility, and it is why chain errands had to become `granted` — an unoffered kind of errand the plan has no row for. | `crates/core/src/quest.rs`, `Quest::granted` |
| 11.6a | **An instrument has a frame of its own, and `PLAN-M11.md` §8 row 4 is reversed.** That row asked for the instrument to live in the weapon grid — *surveying costs your sword arm* — and it was taken, defended and shipped. It was wrong, and the report is the argument: what is through the Reach is a map you have to fight on, so a cost paid in your only weapon is a wall rather than a price. `SlotKind::Instrument` is a sixth grid, deliberately outside `SlotKind::ALL` so that nothing which asks what a board is *worth* ever counts it. `RuleError::MixedGrid` is gone with the rule it enforced. | `crates/core/src/piece.rs`, `SlotKind::Instrument` |
| 12.6 | **Rerolls, which `PLAN-M12.md` §0 declines by name.** The block's founding decision was no reroll, on the grounds that a shelf which changes every visit is not a place. That still holds and the *shelf* still never rolls; what turns over is the barrel and the order book, which are rolled to begin with. The reversal is the human's, narrowed to the two tiers where "give me a different one" is not the same as "give me a different town". | `crates/core/src/shop.rs` |
| 13.1 | **`Rule::Spread` works on the diagonal, and `PLAN-M13-2.md` §4.2 writes it orthogonally.** `Slot::enchant_is_live` pays an enchantment nothing while another touches it edge-on, so a copy laid beside its source destroys what it copied and the node is a point spent on making yourself worse. A corner is the tightest spread this board allows and the borrowed idea survives: what is next to what still decides what you get. | `crates/core/src/rule.rs`, `Rule::Spread` |
| 13.2 | **Spread is settled at the end of a fight, not during one.** Combat is a pure function of what it was handed — that is why a mid-fight save carries a creature name and a tile, and why `Effect::Fragile` breaks an item *for the fight* rather than for good. A rule writing to the loadout mid-tick would undo the property the whole save format rests on. Turns are `duration_ms / SPIN_EVERY_MS`. | `crates/core/src/fight.rs`, `settle` |
| 13.3 | **`RowHarvest` pays at the *start* of a fight.** §4.3 puts it in `fight::settle`, which is the one place its mana could not be spent — mana is spent inside a fight and gone when it ends. *At the bell* means the start everywhere else in this engine. | `Character::row_harvest` → `Held::mana` |
| 13.4 | **`rebate` and `encore` fire on the fight *turning*, not on a kill.** §3.1 D and §3.7 C are both written against a kill inside the fight, and **GM2D deals one foe**: `fight::run` builds a single `MonsterSpec` and `check_down` breaks the loop on the tick it falls, so strength refunded on a corpse and a free-cast window reopened after the only foe is dead both bought nothing. `the_fight_turned` reports **every quarter the enemy loses**, capped at three so the corpse is not one — quarters rather than halves because an encore is a *count* the tree sells two of, and a milestone that can happen once is a count that can only ever be one. | `crates/core/src/combat.rs`, `the_fight_turned` |
| 13.5 | **`after` prints the price, not the percentage.** A cast costs three, so a fifth off it is nought point six and integer division makes that nought — `on-standing-discount` sold *twenty percent less* and took nothing off. `ExpertPower::cast_price` rounds the payer's way and is the one place the sum is done; `describe` reads it, so the promise cannot be a different sum from the one the fight does. | `crates/core/src/expert.rs`, `cast_price` |
| 13.6 | **An expert tree *may* hand over an ench something else already gives**, which §1.6 does not allow for. All eight enchs have an owner, so any `gives_ench` in an expert tree duplicates one — and for Full Bill a second copy **is** the promise. `every_ench_comes_from_somewhere` exempts expert trees and asserts the opposite for them: a source hidden behind two finished trees is worse than a duplicate. | `crates/core/tests/ench_sources.rs` |
| 13.7 | **The papers stand on Spike's van**, `the-kaklon-van` at `[4, 6]` on west-bambulon, behind `hidden_until_level: 10`, which §8.1 leaves open. The van is already the counter the Patent's paper is sold from, and a second counter would be a second place to remember. | `data/maps/west-bambulon.tiles.json` |
| 13.8 | **The second fork does not re-raise itself.** §1.3 says the paper is spent on the choice and a player may sleep on it; `offerClass` is called after every fight, after every banking and on every load, because that is what the level-five fork needs. Wired to the same three, the second fork came back after every fight — the game refusing to let you sleep on it. It opens on the purchase and from the line on the sheet that says the paper is in your pack, which is what §1.3's *"opening it in the pack re-raises the screen"* means on a page with no pack. | `web/app.js`, `offerClass` |
| 13.9 | **`Loadout::assembly_pct` came out of the save**, which the plan does not mention because it is older than the plan. It was written into every file and thrown away on the way in, so what the file said was a number nothing read and anything could believe. Derived on load, and guarded at every door that sets a class. | `crates/core/src/save.rs` |
| 14.1 | **`PLAN-M14.md` §1.1 is written against `event::Requirement`, which is the cut campaign's type.** It names `LooseItemOfSize`, `AlignedItems` and `AssembledOfRarity` as the locks a player opens by packing, and §4.1, §4.2 and §5.1 hang three doors on them; all three are `Copy`, `&'static str` and unreachable from a data file. Two were ported. **`AlignedItems` could not be**: its doc means *assembled items sharing one alignment word*, which is upstream's naming system and did not survive the fork — `PieceKind::Alignment` is a *component kind* wearing the same noun. `AssembledOfRarity` is the plan's own third named lock and reads the same live board. | `crates/core/src/tile_event.rs` |
| 14.2 | **Three of six blind counts moved, and one did not.** Measured rather than asserted: the Lip is 8 against 10, the Shelf 1 against 11, the Landing 1 against 2, the Chair Room 3 against 27, the Gallery 3 against 3 — and **the Cairnfield is 45 against 45**, which is the number §1.2 builds its whole design on and the reason to believe the other five. The Shelf's eleven is *"a player cycles their tray"*; there is no cycling, a footprint requirement is met or it is not, and what that floor charges is a component. | `crates/core/src/puzzle.rs` |
| 14.3 | **The chair is three moves and §5.2 asks for nine.** Nine rungs is not expressible in monotone flags over three always-offered labels, and §1.1 is what says so: a choice carries one requirement and one outcome, so *"Turn it to face the door"* raises one flag and cannot be the first, fourth and seventh move. Nine choices puts the answer on the card as a list of labels; a counter with a modulus in it is a flag that goes down. The sequence is Marbulon's and she does it three times because she is nervous. | `data/events.json`, `the-chair` |
| 14.4 | **`needs_all` rather than `hidden_until_all` at the two bottoms.** §4.4 and §5.4 hide the doors and §1.6 says finishing the first dungeon shows *"a sealed door with the other dungeon's name in the refusal"* — which a hidden door cannot do. Two fields, two jobs: one decides whether a place is *there* and the other whether it *opens*. | `crates/core/src/world.rs`, `PlaceDef::needs_all` |
| 14.5 | **`Outcome::Counter` is cut and the low-water marker is an examinable.** §3 asks for it explicitly *"nothing reads it this block; it is the watcher pattern, planted"*. `Outcome::Xp` wrote into a counter nothing consulted for four blocks and nine events printed a receipt for experience that never existed; planting one deliberately is that bug with a note beside it, and `every_flag_an_event_sets_is_read_by_something` refuses it by name. | `data/events.json`, `the-low-water-mark` |
| 14.6 | **The stop-line is on a `Door`, not on the town.** §1.5 says the third town's prose says the writing stops here; a `TownShelf` is an id, a stock list and a commission list and has never had prose. So it is on the one kind the game already has for a screen that is not a loop, one tile south of the counter — which also gives `PlaceKind::Door` back the user M14.3 took off it when the door under the lake became a gate. | `data/maps/the-undercountry.tiles.json` |
| 14.7 | **Marbulon's third answer is the gate's own paragraph, not a third choice on her card.** §6 asks for the choice; her event is spent the moment you take either of her errands, and her errands are the questline that unlocks the Cave — so a third choice on it is a choice nobody can reach. | `data/maps/west-bambulon.tiles.json`, `the-door-in-the-shallows` |
| 14.8 | **`the_ninth_surveyor_is_a_fight_the_board_wins`**, not `..._beatable_by_the_walker_at_22`. A level-22 board is not one this game produces — the shipped transcript ends at fourteen — and `common::geared_from` is what M11.7 established as *the board a player actually has*. **Both bosses were dressed by damage a second and not by rating**, which is what §4.4 asks for and which the recon justifies: that board beats Francis at 2958 and loses to Cairn Chorus at 1141. | `crates/core/tests/sump.rs` |
| 14.10 | **The Treyway's south is its own map, and for one milestone it was not.** M14.1 drew it into `the-treyway.tiles.json` at 16x26 on the *one country, one file* principle — the right instinct, and the wrong call for a reason that is not about content: **`#map` has been a fixed square in CSS since the first map**, so a grid half again as tall as it is wide came out squashed. Reported from play. `the-low-water` is 16x11, and the bar of shingle the tide leaves is a **gate** on one tile of `tide` rather than two tiles of the same grid. The CSS is fixed too, because the split alone would have left every non-square map — the Cave, the map under the lake — still stretched. | `data/maps/the-low-water.tiles.json`, `web/styles.css` |
| 14.11 | **The shore has a road down it and no Cairn Chorus in its pool.** Open scrub at 140 per mille under a pool of mean rating twelve hundred is 350 after the danger multiplier — one step in three — and `common::geared_from` loses to Cairn Chorus at 1141. `make play` crossed the shore twenty-eight times, was beaten on twenty-seven, and never once got down the hole. **A shore you cannot cross is a dungeon gated behind a draw**, and every other approach in this game is a road. | `data/maps/the-low-water.tiles.json` |
| 15.1 | **A boss is refused by the tile and not by the name**, which `PLAN-M15.md` §1.5 writes the other way — it says `rout` *"refuses one by name (`boss_at`)"* and `boss_at` refuses by tile. The difference is invisible until you count: **eight of the nine creatures standing on a boss tile also stand in a region pool**, so a refusal at the name would take seven ordinary field encounters off the menu on behalf of a room the player has not reached. `fight::instant` puts the rule exactly where `rout` puts it, and `mark_instant` refuses on the count alone. | `crates/core/src/fight.rs`, `instant` |
| 15.2 | **`reach(20)` is 4,300 and §2.5 recommends 6,000.** The recommendation reasons that 150 fights at forty experience a win is plausible, and says in as many words that this is *"exactly the sort of plausible that a walk disproves"*. Replaying the shipped walk's own payouts, the mean over its first 150 wins is **28.7**, so 6,000 puts level twenty at 184 fights and 5,500 at 175 — both outside the band. The plan's own instruction covers it: *if the half still leaves 150 out of reach, go under it.* | `crates/core/src/progression.rs`, `CURVE_A` |
| 15.3 | **The 130–170 band is not measurable to the precision it was written at, and `make play` is not the instrument.** §2.3 asks for it *"exactly the way level 5's 25–35 already is"*; that band is set by a **fixed patrol over nine seeds**, not by the walker, and the walker misses it by three fights on its own transcript. Two walks on the shipped curve put level twenty at **150 and 192 wins** — a 28% spread around a ±13% band. The number is kept and the spread is written down rather than tuned away, because `CLAUDE.md` already says the walker is not deterministic and two runs of it disagree. | `SECOND-ORDER-M15.md` rows 9, 13, 14 |
| 15.4 | **`Step::crossing` is `Step::refused_by`**, which no plan asked for. The field's own doc has always described the class — *"this says which kind of refusal it was, so the page can put it where a player will read it"* — and its name described the one instance it had. Renamed while fixing the shore, because restoring the channel under the old name would have put the tide's sentence in the one-second flash the report says it does not belong in. | `crates/core/src/world.rs`, `Step` |
| 14.9 | **The wading shortcut on the Gallery is drawn, and saves eight tiles.** §9 decision 4 leaves it to the recon — *"if it saves nothing it is cut"*. The chains are in opposite walls, so a flooded gallery is seventeen tiles round and nine across. **Flooding the room makes the walk worse**, which is the design rather than an accident: chain A costs you the crossing you had and the Toad's Own Frame is what gives it back. | `data/maps/the-silt-stair-3.tiles.json` |

Also true, and not in the brief because it could not have been:

- **§C.1's code was gone before the fix was written.** The bounty was paid in
  `Run::settle`, which left with the campaign. The rule now lives in
  `reward.rs` and M3's encounter resolution calls it.
- **§C.3 is not a code fix.** It was a fault in a CLI GM2D does not ship. It
  survives as a UI rule: **the shop screen displays the price actually
  charged, never `registry.def(id).price`.** It becomes a test in M3.

## Deleted, and where to find it

Eleven modules and 57 test files went in `48203ee`. Everything is in the
history at `78e40eb` if a question about the old behaviour needs answering.

Dropped modules: `county`, `dungeon`, `route`, `quest`, `relic`, `pedestal`,
`rumour`, `bestiary`, `town`, `share`, `run`.

Dropped tests, all of them testing something GM2D no longer does — fountains
and axis thresholds, the road, receipts and choices, share codes — or leaning
on a helper that does: `packing`, `validity`, `classes`, `casino`, `chain`,
`francis`, `phase_two`, `prose`, `structures`, `two_voices`, `insight`,
`tallies`, `vip`, `completable`, `two_runs`, `taller_boards`, `sudden_death`,
`fight`, `reference_builds`, and the campaign half of `tooltips`.

Four classes went with the dungeons that were their only source: `Ascendant`,
`Threshold-Sighted`, `Prospector`, `Wumpus Hunter`. Their `ClassPower`s survive
and M5's trees may spend them again.

`CLASS_ORDER` and its append-only test were the share-code wire format.
`share.rs` is gone, so the constraint is gone.

## Numbers, so a regression is visible

Every figure below was re-measured for M12.6 rather than carried forward.

| | |
|---|---|
| Upstream suite, pristine fork | 1075 passing |
| After the campaign was cut | 128 passing |
| After the simulation tests were ported to `Character` | 329 passing |
| M1 | 346 passing |
| M2 | 359 passing |
| M3 | 369 passing |
| M4 | 382 passing |
| M5 / MVP | 391 passing |
| Board rebuilt against the original | 411 passing |
| The other side's gear, and a tree that says what it does | 419 passing |
| Shops, errands and a replay of both sides | 425 passing |
| Components that show their shape and explain themselves | 427 passing |
| The sheet, and a fight that opens holding what it holds | 429 passing |
| A tree drawn as a tree | 431 passing |
| Souls experience, and one town on the map | 432 passing |
| Fatigue, errands, and the first dungeon | 447 passing |
| M8.0–M8.1: a door you can go back to, and a log that points | 453 passing |
| M8.2: curses on the card, in the replay, and a bar that stops moving | 456 passing |
| M8.3: skills that grant rules | 459 passing |
| M8.4: enchs, and the class that grants them | 469 passing |
| M8.5: the spin | 474 passing |
| M8.6: the Kaklon Patent | 477 passing |
| M8.7: the door in the wall | 482 passing |
| M8.8: played, triaged, written down | 483 passing |
| M9.0: a rule an item grants, and an item with a name | 494 passing |
| M9.1: a creature leaves something behind | 503 passing |
| M9.2: three sets | 518 passing |
| M9.3: the north is a decision | 526 passing |
| M9.4: played to the ending, triaged, written down | 526 passing |
| An ench you were paid and cannot use yet | 527 passing |
| M10.0: where an ench comes from | 537 passing |
| M10.1: an item that fires once | 545 passing |
| M10.2: Top of the Bill, and two promises that reached nothing | 553 passing |
| M10.3: played as the new class, triaged, written down | 556 passing |
| M11.0: one place the game talks | 556 passing |
| M11.1: the overworld behind the door | 566 passing |
| M11.2: the dense map, and Kettleworks on the ground | 573 passing |
| M11.3: the Drambus Stack, five floors and one sitting each | 583 passing |
| M11.4: the lake drains, and there was always something under it | 590 passing |
| M11.5: map shards and the three instruments (the first seam) | 597 passing |
| M11.6: the surveyable map, read through what you carry | 608 passing |
| M11.7: the block was unfinishable, and every test was green | 609 passing |
| M11.8: hands, eyes, and a brief for somebody who is not the builder | 609 passing |
| M11.9: what the new maps leave behind, and the long way back | 619 passing |
| A key that never left the bag, and an event that never shut up | 630 passing |
| M12.0: board pressure, measured before anything tries to move it | 638 passing |
| M12.B: a coordinate is only meaningful with the map it came from | 643 passing |
| M12.1: the bargain barrel | 651 passing |
| M12.1a + M12.2: three tiers, and an order on the world's clock | 662 passing |
| M12.5: events that pay something, and say what they pay | 670 passing |
| M12.3: slower cells — a row is earned, not scheduled | 671 passing |
| M12.4: played to the ending, triaged, written down | 671 passing |
| **M12.6: a chain you can see, a licence you can buy, and prices that mean it** | **687 passing** |
| A swing is not a constant, and the row said it was | **691 passing** |
| The Kettleworks was a wall, and the wall was the gear | **697 passing** |
| A defeat costs you your place | **698 passing** |
| Twenty-one errands that could be taken and never finished | **700 passing** |
| The speed of a fight, and a log you can read | **701 passing** |
| A locked choice was a wall, and is a target now | **703 passing** |
| A bank, a door that survives a reload, and a sheet on the screen that changes it | **710 passing** |
| An instrument has a frame of its own | **715 passing** |
| A row is bought all the way up to the old size | **717 passing** |
| M13.0: a finished tree is a countable fact | 728 passing |
| M13.1: ten experts, thirty-one knobs, one `ClassPower` arm | 742 passing |
| M13.2: `Effect::Tunes`, checked against the tree's own class | 745 passing |
| M13.3: four rules a class can move | 760 passing |
| M13.5: the ten trees land — 16 trees, 124 nodes | 760 passing |
| M13.4: three papers, and two of them refused | 775 passing |
| **M13.6: every expert reaches something — and thirty-eight did not** | **781 passing** |
| M13.7: the screens — a second fork, a sixth tab, a rack that holds more than one | 781 passing |
| M13.8: five browser checks, all five negative-tested | 781 passing |
| **M13.9: the notebook executed, and the suite from a minute to 34 seconds** | **785 passing** |
| M14.0: five small things, and the plan was written against the dead type | 802 passing |
| M14.1: the Treyway grows a south, and three narrow lints were one lint | 806 passing |
| M14.2: the Wextreen Sump — four floors, three puzzles, the Ninth Surveyor | 814 passing |
| The barrel showed you one thing and sold you another | 815 passing |
| M14.3: the Silt Stair, and the chair is three moves because nine is not monotone | 823 passing |
| M14.4: the Undercountry, and a requirement is two things now | 828 passing |
| M14.5: five browser checks, and both of the things they found were real | 828 passing |
| M14.6: the notebook executed, and six floors became every floor | 831 passing |
| **M14.8: two dungeons walked end to end, and the floors were the thing that was wrong** | **832 passing** |
| M15: the bestiary, the cart, the sands, instant battle, the curve | 913 passing |
| M16.0: the run as a fixture, and two lanes nobody could be shut out of | 919 passing |
| M16.1: the way under the flat, and nine stakes that are not labelled | 927 passing |
| M16.2: the Assay, and two doors the plan hung above what this game builds | 933 passing |
| M16.3: the Needle Room, and something on the plate wearing your own board | 941 passing |
| **M16.4 + M16.5: two classes GM2D wrote, and the twenty-one pairs they make** | **950 passing** |
| M16.6: six browser checks, a walk that loops, and the block written down | 950 passing |
| M17.0: a table nobody can see yet, and a cue you could not have aimed | 960 passing |
| M17.1–M17.5 + M18: the overworld is a table, and the notebooks executed | 976 passing |
| **M19: a ball you can watch, a diamond you can hit, and a glossary** | **1,137 passing** |

**M13.5 adds none and M13.7 and M13.8 add none, and all three are honest.**
M13.5 lands ten trees into a data file and the three lints it needed were
written in M13.2 and M13.1; M13.7 is the screens, and a screen is checked in a
browser rather than in `cargo test`; M13.8 *is* that browser check, five of
them, which the count below has as sixty-three `ok:` lines rather than as tests.
**M13.6 adds five and is the block's largest milestone by a distance** — the
five are what found thirty-eight dead nodes and four engine faults.

Note M12.4 adds none, and neither did M11.0 or M11.8 — all three are honest.
M12.4 is a playthrough, a triage and a brief; its deliverable is
`TRIAGE-M12.md`'s thirteen rows and the finding that two of them are not the
builder's. M11.0 moved every string
the game says through one door and changed no behaviour the suite could see;
M11.8's deliverable is a harness for somebody who is not allowed to read the
suite. M11.7 adds one — a block that was unfinishable was fixed by *retuning
content*, and one check now measures what a range used to guess at.

| | |
|---|---|
| Catalogue | **568 components, and neither M12 nor M13 moved it** — every save that opened on M11 opens on M13. M11's two seams (544 → 550 → 568) are the last there have been. **Prices are ×5 as of M12.6** and that is seam-free: `catalog_fingerprint` hashes names only |
| Pieces that apply a curse | 59 of 568, 4 kinds, 2 on the starting shelf |
| Sets | **9**, of three components each bar the Toad Frame's two — every piece `EVENT_ONLY`, off one creature **or one stack of floors**, in one grid |
| Ladder | **61 creatures**, and the newest is wearing a real player's board — the Tenth Surveyor, thirty-eight components in the cells the human seated them in and all six of that character's enchs, at 15,000 health and 64 strength because **strength is the whole dial at this depth and health barely moves a fight**. Before her: **60 creatures**, rated 16 to 2958, and the two new ones are the deepest fights in the game — the Ninth Surveyor at 2053 and What Marbulon Faced Away From at 2123. **Both were dressed by damage a second against `common::geared_from` and not by rating**, because that board beats a 2958 and loses to a 1141; see *A rating predicts nothing*. Six are stepped down: the Kettleworks field's five and The Gearwright, at `gear_offset: -2` plus a body trim where the footprint families ran out — 12 to 16% each |
| `crates/core` | **~49k lines**, down from ~50k at the fork and up 1.5k over M14 — `wc -l` over every `.rs` under `crates/core/src`. The method is named because the figure carried here through M12.6 was 42.4k while the code had moved under it |
| wasm | **1660 KB**, up from 1539 KB at M13 — `dist/web/pkg/gm2d_wasm_bg.wasm` after `make web`. CI builds its own and the two are not bit-identical, which is why the *stamp* is checked against itself and never against a number |
| Save format | v1. **No seam, still, and M14 adds no field at all** — nine maps, eight floors, two creatures, two terrains and four new `Requirement`/`Outcome` arms, and not one of them is in the save: a map is content, an event's shape is content, and what a run has done was already `answered` and `flags`. Every save that opened on M11 opens on this. Before it: **M13 is the first block to take a field *out*.** Five new `Character` fields, every one `#[serde(default)]` and skipped when empty — `second_class`, `expert`, `second_paper`, `fast_wins`, `told_curses` — so an older file opens as one class with no paper and nothing following it out of the last fight, which is what those characters had. **`assembly_pct` is gone from the file**: it was written and then thrown away on the way in, and *a number that is stored and ignored is a number somebody will one day believe*. A save now carries **six boards**; one naming five gets an instrument frame at the base height, and `repair_boards` lifts an old build's instrument out of the weapon grid on the way in — the loader is where a field carried across a build change is caught. `banked`, `commissions`, `rolled_barrel`, `rolled_ledgers`, `rerolls` and `bought_licence` all default the same way |
| Tables | **2** — the Treyway and the Undercountry, `traversal: "shot"` in the map file. Everything on either is reachable in **two** rounds of shots and `make play` crosses the Treyway in **three**, against `PLAN-M17.md` §2.7's ceiling of nine. `only_the_country_maps_are_tables` asserts the list, so a third is a decision somebody makes there |
| Maps | **25**, in `data/maps/*.tiles.json` — west-bambulon 20×20, the-great-gear-cave 9×5, the-treyway 16×16, kettleworks-field 20×20, five Drambus Stack floors 10×10, under-the-lake 13×9, the-reach 20×20, **the-low-water 16×11, four Wextreen Sump floors 12×12, four Silt Stair floors 12×12, the-undercountry 20×20** |
| Places | **196 over twenty-five maps**: 3 towns, 107 events, 43 gates, 16 bosses, 8 caravan stops, 2 crossings, 1 bench, 1 door, and **15 obstacles** — 4 bumpers, 5 drifts of sand, 3 spikes, 2 pockets and a chute, which are M17's and are the first places in the game that a *foot* never touches. 41 of the events are the Kettleworks field alone, and the one door is the last screen in the game, on the Undercountry |
| Events | **80 placed: 64 ask something and 16 are notes, over 102 choices.** **21 chains from 10 roots**, every root choice handing over an errand. **One of the eighty repeats** — the chair at the bottom of the Silt Stair, which is three moves at one object and the only event in the game that is not spent when it is answered |
| `PlaceKind` | **13**: town, event, gate, boss, door, crossing, bench, caravan, **bumper, spike, pocket, chute, sand**. The five new ones are M17's and `is_obstacle()` is what separates them: an obstacle is hit **in flight**, which is the one thing a step has nowhere to happen. `catches()` is the other question, and it is a different five: a gate or a boss **stops the ball**, so hitting a diamond is entering it. Each of the thirteen has its own mark on the map, which for a milestone the five obstacles did not — they wore the event's. The Stack is still `PlaceDef::floors` on a gate rather than a kind |
| Effect kinds | **7**: stat, start_with, grow_slot_rows, assembly_pct, grants, gives_ench, **tunes** — the seventh is M13.2's, and the knob it names is checked at parse time against the tree's own class |
| Ench effect kinds | 4: power, haste, spin, fragile — **unchanged** |
| `Rule` kinds | **16**: curse_on_activate, spin_extra, spin_keep, spin_every, scout, rout, wade, survey, homeward, **spread**, **row_harvest**, **beacon**, **productivity**, **burn_keeps_bonus**, **burn_carries**, **mind_pierce**. The last three are M16's and each is granted by more than one expert tree; the four before them are M13.3's and are the first since M9 that needed code in the fight rather than a translation at the bell |
| Surveyable maps | **2** — the Wextreen Reach and **the Wextreen Sands**, and `survey::mods_for`'s `map` argument is finally read. There is iron under the sand, so **the instrument that reads the Reach best reads the Sands worst**: a compass quiets the Reach by 20% and is 25% *louder* on the flat, and the atlas is the other way round. That is the whole return on a second one — *which* instrument you built becomes a question about where you are going. The door states the trade in these numbers before you take it, because `kit_reading_json` runs `mods_for` against the map on the far side |
| Instruments | 3 — compass, atlas, survey golem, all three on **their own frame**: `SlotKind::Instrument`, six by three, outside `SlotKind::ALL` so nothing that asks what a board is worth ever counts it. It never grows, and one instrument is what it holds |
| Data files | **29** — 9 in `data/` and 20 in `data/maps/`; `data::FILES` is the list `data_is_current` walks, and adding a file to it is the second half of adding one to `data::MAPS` |
| Starting kit | 2 components, **140 Fnorp**, 1 assembled weapon. The purse moved ×5 with the prices; at 28 a beginner could afford three of thirteen barrel lines and no helmet, and both M4 soft-lock guards said so |
| Towns | **3 placed** (the pit, Kettleworks and the third town) and 1 staged, and **the third one is empty on purpose** — `common::UNWRITTEN` is where that is declared, the mirror of `avail.rs`'s `STAGED`: a shelf with no ground under it and ground with no shelf on it, and both are fine only because somebody wrote the name down. Of the two that sell anything: fixed shelves of 11 / 15 / 17 that **still never reroll**; none sells an ench, and neither placed one sells arcana — a town is its character. Under each counter: a **16-line barrel** and an **order book** (8 lines over 3 towns), and those two *do* turn over. **High Wick is the arcane shelf and it is the staged one**, which is why the barrel had to be what carries the casting family |
| Errands | **40** — 19 authored, and **21 chain errands a choice hands over**. A chain errand is `granted`: never offered at a counter, because the branch you did not take must not be sitting on the tile a moment later |
| Enchs | **8** — 3 on the van's table at **2,000 each**, 2 awarded by a class tree, 1 off an errand, and **2 written for the ends of chains**. The van also sells **a licence for 5,000** to anybody whose class did not come with one |
| The pack | **5 things, and three kinds.** Three tins at **20 / 55 / 140**, and two charms: **The Quiet Word** at 40, which pays for one running-away, and **The Short Way Back** at 300, which puts you in your last town. `SupplyDoes` is the kind and it defaults to `restore`, so the tins did not move and there is no seam. **The Quiet Word's price is arithmetic**: running away costs 20%, the cheapest tin covering 20% is 55, so a charm dearer than that is a charm nobody buys — it shipped at 90 in its first draft and a test caught it |
| Two fatigue caps | **`CAP` is 60 and `HARD_CAP` is 99**, and the difference is the whole of what a penalty is. Wear stops at 60 because a fight is a budget; the things you do *instead of* fighting do not — **running away is 20% and walking off a cairn is 10%**, both through `tire_hard`. Ninety-nine and not a hundred, because `worn` floors a maximum at one point of health and a hundred would be a character alive by a rounding rule. **It is never a dead end**: walking is free, every map has a way up, and a town takes all of it off from 99 as readily as from 60 |
| The counters | shelf **×5** of catalogue, order book **×10**, barrel **×1**. The barrel holds nothing dearer than 60 and the book nothing cheaper than 65, so the three tiers cannot overlap. **None of the three sells what an errand pays** — eighteen rewards were buyable until a lint read `quests.json` |
| The barrel | **16 lines**: fourteen cores and two extras, and its shape is `shop::barrel_wants` **derived from the recipe table** rather than a list of kinds. It covers all three ways of building a weapon — a blade, a book and a crystal ball — where a hand-written list covered the blade and left a hundred and six casting components on no counter in the game |
| Casting components | **106** — 18 books, 31 spells, 19 inks, 26 orbs, 12 alignments. **89 are buyable somewhere**, up from **6**, all six of which were errand rewards. The opening barrel carries a book, two spells and an orb, so both caster ways finish out of it on the first afternoon |
| A reroll | `n*n` Fnorp for the nth, counted **per type**, wiped every ten levels in every town. The line you have on order is never rerolled out from under you |
| Board pressure | fill **43%** at level five and **37%** at eight before M12 — *down*, because rows arrived on a clock and components did not. `pressure::of` is the measurement and `pressure::target` is what it is aimed at |
| Boards | **Six frames**: five worn, 6×3 at level 1 and 6×8 once the tree has been walked all the way up, and the instrument's, 6×3 for ever.  **A row is no longer a thing a level hands you** — it is a skill point or a finished errand, **11 nodes and 2 errands**, and M12.0's measurement of why is in *A row is earned, not scheduled* |
| Level 5 | ~27 fights, mean of nine seeded walks |
| The Treyway | brackets levels **12–16**, not the plan's 5–9 — the door behind it is behind a crossing that asks for 9 |
| A whole playthrough | **342 wins, 170 losses, level 14, 4,406 steps** to the door under the lake — **on M11's curve, and that number is now about a different game.** M15.3 flattened the curve and the same walker from a new game reaches **level 16 in 210 wins** where M14's reached eleven in 1,151. It still stops at the Drambus Stack, which is a fact about the walker rather than about the maps: `PLAN.md` §6d row 3, since M11.9. `GM2D_FROM` is what walks the rest; `testing/transcripts/m14-*.txt` is the old curve and `m15.3.txt` is this one |
| The curve | **Quadratic to fifty, exponential after it**, since M15.3 — `xp_to_next(L) = 1.4257·L² + 2.4885·L + 16.0858` up to the joint, and `xp_to_next(50) · 1.35^(L−50)` past it, the two arms agreeing *at* fifty rather than near it. The base is not a new number: it is the `1.35` the whole curve used to run on, taking over where the quadratic stops. **`xp_to_reach(20)` is 4,298 against the old 17,053** — a quarter, not the half the anchor allowed — and `xp_to_reach(5)` is **132 exactly**, because the fit is pinned to it so `XP_DIVISOR` need not move. `MAX_LEVEL` is 60; the first level whose cost will not fit an `i32` is **95** |
| The cart | **A travelling caravan on the Kettleworks field**, eight authored stops and `WorldState::caravan` saying which one it is at — because *places are content and content is not state*, so a thing that moves is stops that are content and a choice that is state. Five of your steps on that map, never onto the stop it is at and never under your feet. It sells **survey gear at shelf prices**, spent once each and keyed by the cart rather than the stop, because moving a cart is not restocking it. It exists because the cheapest instrument wanted a Magnet and **nothing in the game sold one** |
| The bestiary | **Meet something once and it is in the book for good.** `met:<canonical>` beside `beat:`, and deliberately a second counter rather than a reading of the first: a creature that killed you four times is one you have met four times and beaten none. `Game::encounter_with` is **the one door** — an encounter was set in two places in the shim and a book populated at one of them is a book with no bosses in it. The entry is `creature_json`, which is **one builder for two screens**, so the glossary cannot disagree with the fight panel about what a creature is. Ordered by the ladder, because a list that reorders itself as you fight is a list you cannot find anything in twice |
| What a creature resists | **Shown, since the bestiary.** `physical_resist`, `magic_resist`, `mind_resist`, `curse_resist`, both pierces, both hardenings and `reflect` have been on `Stats` since the fork and **nothing had ever printed one** — reported as *"currently you cannot see enemies stats / resists"*. Zeroes are dropped rather than printed: on a defence, nought is the ordinary case rather than a claim. Sixth time *a derived number needs somewhere it is shown* has been the answer here |
| Instant Battle | **Beat something five times and you may stop watching it.** `fight::instant` is `run` then `settle` with nothing drawn — no new settlement code, because a second answer to what a win pays is the mistake this project has paid for six times. It pays the speed bonus, rolls the drops, ticks the order book and costs the four percent, all of which `fight::rout` deliberately does none of. The tally is one `bump` in `pay_a_win` and the mark is one `#[serde(default)]` field on `WorldState`, so **no seam**. **The boss refusal is the tile's and not the name's**: eight of the nine creatures on a boss tile also stand in a region pool |
| Skill trees | **29 trees, 208 nodes.** The base's 22, seven classes' 8 / 8 / 10 / 8 / 8 / 9 / 9, and **twenty-one expert trees of six each**. (Was: **16 trees, 124 nodes.**) The base's **22 over nine tiers**, the five classes' 8 / 8 / 10 / 8 / 8, and **ten expert trees of six each** — two roots, three, and a capstone, every node of which must reach that expert's own power. **11 of the base's grow a row** — M12.3's seven, plus a five-tier spine at 3/4/5/6/7 points that walks every frame to the original **six by eight**. Twenty-eight points for that ladder alone, against a `MAX_LEVEL` of **60** since M15.3 — it was 32, and a table that stops at 32 stops nine levels before the curve changes shape. An expert node costs **2** |
| Class figures | **28 — seven hand-drawn and twenty-one colourways of one drawing.** An expert is a pair, so its figure is the paper Spike hands over with one wax seal per parent class. `every_class_the_game_offers_has_a_figure` is the lint, over the offered seven and the twenty-one and nothing else |
| Classes offered | **7 on the fork, 28 in the game.** The Kettle-Stoker and the Whisperling are M16's and are **the first two classes GM2D wrote rather than inherited**; the twenty-one experts are `C(7,2)`, one a pair, and none is on any list a player picks from. **Every one of the twenty-eight reaches something and so does every one of the 126 expert nodes**, and both are lints that *call* rather than declare. (Was: **5 on the fork, 15 in the game.**) The ten experts are `C(5,2)`, one a pair, and none is on any list a player picks from — you finish two trees and the pair decides. **Every one of the fifteen reaches something and so does every one of the sixty expert nodes**, and both are lints that *call* rather than declare |
| Experts | **21**, carrying **64 knobs**. Eleven are M16's and they grant three new rules between them — `burn_keeps_bonus`, `burn_carries`, `mind_pierce` — each granted by more than one tree, which is the shape `Spread` and `Beacon` already have. Six of them are about a furnace, and every one of the six carries its own: *an expert's power is self-contained*. (Was: **10**, carrying **31 knobs**.) Six are read at the tick, two settle in the purse, one is the board's, one crosses a fight boundary. A character holds **up to three classes** and all three are live |
| The papers | **3** on Spike's van, all drawn from the first visit: the Patent's licence at 5,000, **the Second Paper at 5,000 behind nothing at all**, and the expert paper at **nothing** behind two finished trees — the twenty-four points are the price, which is what keeps a free paper from being a fourth class on the fork. M15.4 took the tree gate off the second paper on the human's ask; the level that puts the van on the road is what is left |
| Figures | 27 `.tex` → **83 SVGs** (13 family drawings, 4 drawn for themselves, 5 classes, 3 towns, you) |
| Art coverage | **60 of 60 creatures**, 3 of 3 towns, 5 of 5 classes, and you. The set pieces, the instruments and the enchs have no art and want none — a component has never had a figure |
| Browser gate | **90 `ok:` lines in one engine**, five of them M19's — the ball slides and the trail grows behind it, a diamond catches, the long cart runs between towns, the furnace shows on the bar, and the glossary opens on G. One of the five *passed while printing the wrong thing* (**9 burns off None**, reading `what` where an event's subject rides in `item`), which is the argument for a check that prints what it found. Before it: **85 `ok:` lines**, seven of them M17's and every one negative-tested — the cue snaps to what core takes and pulling further pulls harder, a shot flies the path core returned, four keys aim and space fires with no pointer, a spike takes its percent and says so, a ball in the pocket wakes up in town, a floor still steps and draws no cue, and reduced motion is at rest with the trail still drawn. **The hardest of the seven to break is the floor one**: every lie about *the arrows mean two things now* takes the whole gate down before the check runs. Before it: **78 `ok:` lines**, six of them M16's and every one negative-tested — the way under is silt until the sheet, a stake offers the pull and a compass that lies, a sinkhole drops you in an alcove nothing walks into, the Tenth Surveyor's panel draws the run's own items, the fork is seven cards in two rows, and a Stoker's replay says what the furnace took. Before it: **96 `ok:` lines over 3 engines** — which is 67 in any one of them, not 81; the count is a total and reading it as per-engine is wrong by fourteen. The newest is M15.2's, and it is the only one that can answer a *negative*: that a fight you have already had is settled and **never drawn**. The newest five are M14.5's: the tide is drawn before it goes out and walkable after, the lip of the Sump refuses in the Reach's words and opens the frame, a wheel that keeps what you feed it says what shape it wants, the chair is three moves in an order **and comes back**, and the third town is empty with the screen after it saying so. **All five were negative-tested, and two of the five found faults on a green build** — see *A stack gate that wants an instrument* |
| The suite | **976 passing** after M18, and a `data/` touch costs about **three minutes**, not ten: **127 seconds relinking 83 test binaries and 47 running**, measured on an idle machine. The ten is a cold `--workspace`, which adds the lab and the shim on top of both. **Measure on a quiet machine or not at all** — one attempt at this read `real 1279.89` against `user 63.37`, which is twenty-one minutes of wall clock for a minute of work, because it was queued behind three browser gates. `include_str!` is not the thing to change — loading from disk in the test profile would make the tested path differ from the shipped one, which is two rulebooks — and the fix, if one is ever wanted, is **fewer test binaries**, which is a trade against one file per concern that nobody should make to save two minutes. `SECOND-ORDER-M16.md` row 17 is where that is measured. Before it: **950 passing** after M16. Before it: **913 passing, and ~33 seconds warm** after M15, the bestiary, the cart and the sands; **832 and 27.5s** after M14 — measured after M14, and the ten slowest files are the ten that were slow at M13: `drops.rs` at 11.0s and `experts_reach.rs` at 6.3s, neither of them M14's, and nothing this block added is above 0.4s. **`SECOND-ORDER-M14.md` row 17 was written claiming it had slowed to minutes and is corrected there**: what is minutes is rebuilding sixty test binaries after a change to `combat.rs`, which is a fact about editing the engine. Before M14 it was **788 passing, and 34 seconds warm.** It was a minute through most of M13 and `rules_m13.rs` was 29.6s of it: `beacon_board` ran Auto-pack over the whole catalogue on twenty-row grids, four times, because it was the only fixture in the repository with two items that touch. `common::items_in_a_row` is what replaced it — **0.03s** — and `experts_reach.rs` went 9.6s → 6.5s by measuring once per *set* of nodes rather than once per question. `drops.rs` at 11.3s is now the slowest file and is untouched. `[profile.test] opt-level = 2` since M12.6, with debug assertions and overflow checks still on — this is the `test` profile, not `--release` |
| Floors with a puzzle | **6**, and floors with a boss **2**. Every one is monotone — flags only grow, so no move can make the way on unreachable — and `puzzle::solvable_blind` counts the worst case rather than the plan asserting it |
| Blind-solution ceilings | Sump **8 / 1 / 45**, Stair **1 / 3 / 3**. The plan guessed 10 / 11 / 45 and 2 / 27 / 3; **the Cairnfield's forty-five came back exactly**, which is the reason to believe the other five. `every_floor_in_the_game_can_be_solved_blind` holds every floor there is under 45 |
| `Requirement` kinds | **8**: none, gold, flag, holding, **loose_item_of_size**, **assembled_of_rarity**, **surveying**, **all**. Three of them are ported from `event::Requirement`, which is the cut campaign's type — `PLAN-M14.md` §1.1 names them and they were unreachable from a data file |
| `Outcome` kinds | **11**, the newest being **give_up** — the other half of `LooseItemOfSize`, and a separate arm because a requirement is a question and an outcome is what happened |
| Terrains | **17**, the newest two being **`tide`** (sea that goes out, drawn only where something drains it) and **`silt`** (what a room is floored with after it has been under water). `silt` is not `lakebed`, and the difference is eleven inches |
| New components | **0**. Every key, every drop and every thing a door wants in M14 is in the catalogue already, so the fingerprint is untouched and every save that opened on M11 opens on this |

Note the catalogue is **568**, not the 374 the retheme document counts — it
grew upstream after that document was written, and three times here. Any
content work that quotes a catalogue size should quote this one.

## Open questions the human has not answered

Listed in `PLAN.md` §6. None block M1.

**Answered:** the repo is `sgilson7/gear-master-2d`, public, Pages served from
Actions.

**Still open**, with the default in force: losing costs nothing but the walk
back; the content charter is binding; invented proper nouns fail the M2 lint.

**Answered by `PLAN-M9.md` §5, with the plan's proposal taken in each case:** a
rout pays what a win pays and costs no tiredness; the crossings ask for level 5
and level 9; the drop rate started at one in twenty and was retuned in M9.4
against what a player counts. The Wallspider Weave was left unnamed by the human
and is named here; say the word for a different one.

**Answered by the human for `PLAN-M10.md`:** an item with the Swing on it breaks
**for the fight** and not for good; **no town sells an ench**; and what a tree
does not award is sold by one vendor who is not there until level ten. Everything
else in that plan's §5 was taken as proposed — the Chonga Swing's name, where
the van stands, `+200%`, no node that makes a fragile item fire twice, and no
restocking.

**Answered by the human for `PLAN-M11.md`**, or taken as proposed where the
plan proposed: the five decisions §8 delegated were made in the commits that
made them, and the four the block *diverged* on are in the divergence table
above with their reasons rather than here.

**`PLAN.md` §6b** is what M9.4's playthrough left open — the pool weight that
makes one set three times dearer than the others, and whether Auto-pack should
know what a set is. §6b row one stopped being advice in M11.9 and became
`a_set_is_never_behind_the_rarest_fight_in_its_region`; **the underlying pool
weight is still a person's decision and is still not made.** **§6c** is M10.3's,
and its top row is the only number in either that nobody has argued about: Top
of the Bill's ten-second window is open two thirds of the time.

**`PLAN.md` §6d is M11's**, and there are three of them:

1. **What is past the door under the lake.** Same shape as the question
   `PLAN-M8.md` §5.6 asked about the door in the wall, one map further on. The
   ending screen says nobody has decided, which is true and is the only honest
   thing it could say.
2. **The pool weight, again**, now that nine sets rather than three depend on
   it. Three separate days have been spent on the gap between what a region
   *contains* and what it *deals you*, and the check written in M11.9 obeys the
   weighting rather than fixing it.
3. **`make play` is a walker with a destination, and a walker with a
   destination stops being a player.** One M11.9 run reached the ending; the
   next looped 240 times because a cross-map goal outranked going home to bank.
   The walker is our instrument and M11.8 built a second one that is not — how
   much more to invest in the first now that the second exists is a real
   question and not a bug report.

**Answered by the human for `PLAN-M12.md`**, and three of them reversed what
the plan had written down: **rerolls come back** for the barrel and the order
book but not the shelf; **every price goes up fivefold and the income does
not**; and the licence is a thing you can buy for 5,000 rather than a thing
only a class carries. The rest of §8 was taken as proposed or decided in the
commit that needed it, and the three the block *diverged* on are in the
divergence table above.

**`PLAN-M12-EXEC.md` §8 row 13 is the one live question this block leaves**, and
it is the block's own biggest miss written down as a decision rather than
defended: **should Auto-pack seat filler into the cells it has finished with?**
The button refuses any placement that does not strictly improve `(items
assembled, what they rate)`, so the barrel fills the *bag* and not the board —
203 owned components fitting nowhere at the end of the closing run, which is why
the fill target is missed and why nothing else in the block can reach it.
`PLAN.md` already says the button must "leave nothing obvious in the bag" and it
is leaving two hundred; against that, a final seating pass raises the floor of
player power across the whole early game, which is a judgement about how the
game should feel and not a bug. `TRIAGE-M12.md` rows 8, 9 and 11 are the
adjacent numbers — the loss rate is up a fifth since the shelf went ×5, and
whether `pressure::target`'s 70% at level three was ever the right number is a
question the block never asked.

**Answered by the human for `PLAN-M13-2.md`**, or taken as proposed where the
plan proposed. The nine the block *diverged* on are in the divergence table
above with their reasons rather than here, and every one of them was a case
where the plan described a game this one is not: a brawl it does not deal, an
orthogonal neighbour that would have killed what it copied, a percentage of
three that rounds to nothing.

**M13 leaves one live question, and the code asks it rather than a person.**
`Character::grown_health` is a save field **nothing in the shipped game ever
writes**. `rested_stats` adds it, `save.rs` round-trips it, `Game::eq` compares
it and `tests/save.rs` plants a twelve in it to prove the trip — and no line
anywhere sets it to anything but zero. It is inherited: upstream grew a run's
health as the campaign went, and GM2D's levels grow a *board* instead. Deleting
it is a design decision — if health-per-level is ever wanted, that is the field
it goes in — and it costs nothing where it is. **What it must not do is be
mistaken for a live number**, and the field now says all of that where it lives.

**No longer open:** errands exist, as `crates/core/src/quest.rs` — a new module
rather than upstream's, which was a chain of receipts along a road. `town.rs`
stays dropped: a town is a place on the map plus a shelf in `shops.json`, and
does not need a module.

**And no longer open: what finishing a class tree is for.** It was nothing for
eight blocks. It is a second class and then an expert, and the countable fact
underneath it — `SkillsData::tree_finished` — is one function.
