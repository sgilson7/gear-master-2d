# HANDOFF-M12.md — executing the M12 block

For whoever picks up M12. You are inheriting a finished, deployed, 630-test
codebase with a written plan. This file is the shortest path from *nothing* to
*first commit*, and it exists because the project's own documents are long and
you should not have to read all of them before you can start.

Written 2026-09-03. M11 is live at `6b7abe3e`. Nothing of M12 is started.

---

## 0. Read in this order, and stop when you can start

1. **This file**, entire. Ten minutes.
2. **`HANDOFF.md`** §2 (the five load-bearing rules) and §5 (the eleven traps).
   Skim the rest.
3. **`PLAN-M12-EXEC.md`**, entire. It is the plan you are executing.
4. **`PLAN-M12.md`** §3 only, for the milestones the exec plan does not
   restate in full (M12.0–M12.4). The exec plan **wins** where they disagree.
5. **`TONE.md`** before you write your first player-visible string, and not
   before — but *never* write one without it open.
6. **`CLAUDE.md`** as a reference, not front to back. It is a list of things
   that cost somebody a day, arranged by the system they happened to. Grep it.

Do not read `PLANNING-BRIEF.md` (superseded), `PLAN-M8/9/10/11.md` (done), or
the transcripts unless something sends you there.

## 1. The five things that will bite you first

Every one of these has cost this project a day at least once.

1. **`crates/core` never imports `wasm-bindgen`, `web-sys`, or anything
   DOM-shaped.** The engine is testable in seconds because of this.
2. **`crates/wasm` decides nothing.** It moves strings across the boundary. If
   you are writing an `if` in the shim, the rule belongs in core. M12 has two
   places you will be tempted: the commission clock and the outcomes box. Both
   are core's.
3. **Adding a field to `Character` or `Game` or `WorldState` is a compile
   error until the save carries it.** `SaveFile::of` and `into_game` in
   `crates/core/src/save.rs` destructure exhaustively. **Do not "fix" a
   destructure with `..`** — that hole is the whole point. M12.2 and M12.3 each
   add a `WorldState` field; expect the compile error and welcome it.
4. **Adding to `CATALOG` in `piece.rs` changes the save fingerprint and
   refuses every older save.** M12 is forbidden from doing this — see §3.
5. **The page draws numbers core sent it and never recomputes one.** The
   outcomes box is the highest-risk thing in the block for this rule: it is a
   promise printed on a screen, and this project has shipped four promises that
   reached nothing.

## 2. The loop you will run a hundred times

```
make check                  # fast type-check, seconds
make test                   # the engine suite, native, ~60s, 630 passing today
make web                    # build dist/web/
make test-ui                # the gate: 42 checks, chromium + firefox + webkit
make play                   # a walker plays a new game; read the transcript
```

`make test-ui` builds first; `testing/drive.py` run directly does not, which is
how a change was reported missing for two runs once. If a check fails on
something you just wrote, rebuild before you believe it.

**Two env vars, both about which page:**

```
GM2D_WEB=dist/web-gate ./packaging/package-web.sh     # build elsewhere
GM2D_WEB=dist/web-gate testing/drive.py chromium      # and gate that build
GM2D_ORIGIN=https://sgilson7.github.io/gear-master-2d/ testing/drive.py chromium firefox webkit
```

Use `GM2D_WEB` whenever anything long-running is sitting on `dist/web` — a
rebuild does not merely swap the page, it moves the save fingerprint and kills
the other run's save. Use `GM2D_ORIGIN` after a deploy: it walks all 42 checks
against the live page, which is the *verify against the live page* step this
project has required since M8.

**Rebaseline the golden combat fixture** only when you meant to change the
fight, and say in the commit what started fighting differently:

```
REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core
```

## 3. The four rules specific to this block

- **No new components. None.** Not in the barrel, not as an event reward, not
  anywhere. `PLAN-M12-EXEC.md` §7 row 1 makes this binding where the frame
  only recommended it, because M11 already spent two save seams and a third
  and fourth inside two blocks means a player's file refused twice in a
  fortnight. **The recon is already done and it says you do not need any:**
  the catalogue holds **101 pieces of one or two cells priced at 12 Fnorp or
  under**. The barrel is a selection problem, not an authoring one.
- **No reroll, in any costume.** No consumable that turns the shelf over, no
  "want ad", nothing. The block exists *because* somebody asked for a reroll
  and the answer is a reasoned no — `PLAN-M12.md` §0.
- **Every sentence goes through `log()`** in `web/app.js`. It lands on the
  strip that is always up and is kept in the history. No new panels for
  announcements. This is the house style as of M11.0 and M12 is the first
  block born under it.
- **Do not `git push` or `make publish`.** Only a human deploys. The five
  deploy points in `PLAN-M12-EXEC.md` §5 are *requests for permission*, not
  instructions to you. Check `git log origin/main..HEAD` before asking, so you
  can say exactly what would go.

## 4. Where the code is, per milestone

Landmarks, verified 2026-09-03. Paths and names are current.

### M12.0 — the measure *(no push)*

| what | where |
|---|---|
| the walker to instrument | `testing/playthrough.py` |
| boards and what is on them | `character_json()` / the packing payload in `crates/wasm/src/lib.rs` |
| the transcripts it writes | `testing/transcripts/` |

Report per level: **fill** (occupied/total cells, per slot and overall),
**bench depth** (owned pieces that fit nowhere), and — the exec plan's
addition — **pieces by source** (shelf, barrel, commission, event, drop,
quest). Three faucets open in this block and they can mask each other; the
by-source count is how you tell them apart. Commit the baseline.

### M12.1 — the bargain barrel *(deploy point A)*

| what | where |
|---|---|
| shelves, which are content | `data/shops.json` |
| the shelf model and `shelf()` | `crates/core/src/shop.rs` |
| the catalogue you select from | `crates/core/src/piece.rs`, `CATALOG` |
| what a town sold already | `WorldState::bought` — a town id and an **index** |

Append to a town's stock, **never insert**: the index is the identity, a
bought entry is greyed and left in place, and renumbering makes an old save
point at something else.

Acceptance includes a test that **buying and reselling the barrel loses
money**. Anything buyable and sellable is a gold faucet if the spread is
wrong.

### M12.2 — commissions *(deploy point B)*

| what | where |
|---|---|
| the new field | `WorldState` in `crates/core/src/world.rs` (line ~1073) |
| the save that must carry it | `crates/core/src/save.rs`, both directions |
| where fights resolve, to tick the clock | `fight::settle` in `crates/core/src/fight.rs` (line ~277) |
| the ledger, which is content | `data/shops.json` |

The clock counts **fights, not steps** — a step is free and an order you can
pace out by walking in circles is a wait, not a cost. Empty default so old
saves load. One open order per town.

Write `every_commission_reaches_something` in the shape of
`every_offered_class_reaches_something` (`crates/core/tests/the_bill.rs`), and
note *why* that lint works: it **calls rather than declares**. Its first
version matched a variant and named where the power was honoured, which a
stubbed payout passed cleanly. Place an order, tick, collect, assert the piece
is in the bag.

### M12.5 — events that pay *(deploy point C — the new milestone)*

| what | where |
|---|---|
| the live outcome type | `crates/core/src/tile_event.rs` — `Outcome`, `Requirement`, `Choice` |
| **the describer to port** | `crates/core/src/event.rs` — `Outcome::describe` (~line 291), `Requirement::describe` (~line 3045) |
| where a choice is applied | `answer()` in `crates/wasm/src/lib.rs` (~line 742) |
| the card that must grow a box | `showCard()` in `web/app.js` (~line 590) |
| the content | `data/events.json`, and the event places in `data/maps/*.tiles.json` |
| the lint to extend | `no_mechanical_line_speaks_the_theme` in `crates/core/tests/skills_read.rs` |
| the model for warp's safety check | `every_gate_leads_somewhere_you_can_stand` in `crates/core/tests/dungeon.rs` |

**Read `CLAUDE.md`'s *Two types called Outcome* before you write a line of
this.** There are two types with that name. `event::Outcome` is the cut
campaign's and carries the describer you want; `tile_event::Outcome` is the
one the game uses and has none. Attaching to the wrong one is the trap.

**Most of this milestone is content.** `Outcome::Give` hands over a component
and `Outcome::Flag` sets a flag that `PlaceDef::hidden_until` reads — both
have shipped since M2 and are used **twice and never**. Gear rewards and event
chains need no new engine. What needs engine is three outcome kinds
(`Supply`, `Tire`, `Warp`), two describers, and a box.

### M12.3 — slower cells *(deploy point D)*

| what | where |
|---|---|
| the row-per-level grant | `character.rs::resize_boards` (~769), called from `fight.rs` (~470) |
| which slot grows when | `progression.rs::grows_at` (~99) |
| **the two tests this retires** | `board_size_is_a_function_of_level` and `a_levelled_character_has_the_boards_its_level_implies`, both in `crates/core/tests/progression.rs` |
| the ledger's new field | `WorldState` again, and `save.rs` again |

This retires an MVP pillar (`PLAN.md` M4's row-per-level) **on purpose**. Say
so plainly in the commit; do not bury it. `boards_only_ever_grow` stays and
still matters — a board that got shorter would silently drop what was seated
in the rows it lost.

**Gate zero first.** Re-run the probe after M12.5 and put the numbers in front
of the human. If throughput alone hit the target curve, this milestone shrinks
or is skipped, and the close-out says so. A block that stops early on evidence
is the measure working.

### M12.4 — triage and close *(deploy point E)*

`TRIAGE-M12.md` in the M11 format — every finding, severity × cost, blockers
fixed here, the rest dispositioned openly with the reason written down. The
curve diff. The agent spot-run (see §6). The friend's verdict, verbatim.

## 5. How to know you are doing it right

- **Break every new check and watch it fail before you keep it.** Three checks
  have shipped vacuous in this project. A check that compares zero with zero is
  not a check. The most recent one shipped vacuous **twice** in one sitting —
  `character_json` has no `bag` field, and `board.state.bag` is empty until the
  packing screen paints.
- **Every derived number needs somewhere it is shown**, or it cannot be told
  from a bug. Four skills worked perfectly and were reported broken because
  nothing printed them.
- **And every number that is shown needs somewhere it is read.** Event
  experience was written into a counter nothing consulted, for four blocks.
- **Check the second visit.** Three faults found after M11 shipped were one
  fault: a key that stayed in the bag, an event that re-opened for ever, a door
  that re-locked. Everything here is walked over more than once.
- **Grep before you build.** This project has nearly built the same thing
  twice, twice.
- **`make play` is not a verdict, it is a transcript** — and it is not
  deterministic. One run reaches the ending; the next may loop. Read *where* it
  loops. A run that loops is still a finding.

## 6. The one thing you cannot test yourself

`testing/agent_driver.py` and `testing/AGENT-BRIEF-M11.md` hand the built game
to an agent **forbidden the source, the data, the tests, the plan and every
handoff file**, and given only what a shop poster could tell it. It found a
three-block-old bug on its first run, in a line every reader of the source had
skimmed past because the comment beside it explained why it was right.

M12.4 wants a spot-run with five errands, and the fifth is the one only a
stranger can do: **read three events and say, before choosing, what each half
will give you.** An outcomes box that the person who wrote it can read is not a
box that works. Write an `AGENT-BRIEF-M12.md` in the same shape; the
prohibition is the instrument, so do not soften it.

## 7. State of the tree, right now

```
origin/main         7686004  (M12 plans + docs; nothing of M12 built)
tests               630 passing
catalogue           568 components — DO NOT MOVE IT
maps                11, in data/maps/*.tiles.json
browser gate        42 checks, 3 engines, green against the live page
live                6b7abe3e
```

Open questions that are the human's and not yours: `PLAN.md` §6a–§6d, and
`PLAN-M12-EXEC.md` §8 rows 1–12. **Ask rather than assume on any of them**,
except where a row already records a recommendation and the work cannot start
without a decision — then take the recommendation, say in the commit that you
took it, and flag it for review.
