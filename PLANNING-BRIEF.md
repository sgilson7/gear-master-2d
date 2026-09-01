# Gear Master 2D — planning brief for the Claude Code agent

This document does not build the game. It tells a Claude Code agent how to **produce the
plan** that builds the game, what that plan must contain, and what "done" means at every
step. Put it at the root of the new repo as `PLANNING-BRIEF.md`, then start the agent with:

> Read PLANNING-BRIEF.md. Execute Part A to produce PLAN.md. Stop and show me PLAN.md before
> executing any milestone.

---

## 0. What we are making

A 2D, tile-based, open-world RPG in the shape of early Dragon Quest / Final Fantasy, built on
the assembly-puzzle gear system from `sgilson7/gear-master`. The player walks a map, gets
into fights decided by the gear-board auto-battler, levels up, grows their boards, spends
skill points, and at level 5 picks a class. It is meant to be **long and grindy**, which is
why save-to-file is not a feature but a precondition.

**MVP is finished when all of these are true in the deployed browser build:**

1. A leveling system exists, and **leveling up adds a row to a gear-slot board** (which slot
   per level is the agent's design call — see §A.3).
2. A base skill tree exists and the player gets **one point per level** to spend on it.
3. At **level 5** the player chooses a **class**, which unlocks a class-specific skill tree.
4. The game can **download a save file at any moment** and **load one back**, restoring the
   exact game state including the random-encounter RNG. This is a hard gate; a build that
   cannot round-trip a save is not shippable, no matter what else works.
5. The world is a **20×20 tile map** with terrain, tile-bound events, and random combat whose
   chance is a function of the tile's terrain and its danger level.
6. It is hosted on GitHub Pages like the original, in a **separate repo**.

---

## Part A — Produce the plan (the agent's first job)

The agent's first deliverable is `PLAN.md`. It must not write game code until PLAN.md is
approved. PLAN.md is produced by doing the following, in order.

### A.1 Read the sources

| Source | What to extract |
|---|---|
| `sgilson7/gear-master` | The engine crate (`crates/engine`, ~79k lines, **zero dependencies**). Catalogue the public API of `run.rs`, `loadout.rs`, `combat.rs`, `piece.rs`, `naming.rs`, `rating.rs`, `rng.rs`, `event.rs`. Note `Run::grow_boards(by)` (grows *every* slot) and the snapshot/undo code — both are the seeds of what we need. |
| `sgilson7/gear-master` | The **event prose** in `event.rs` (TWO BY TWO, THE CASINO, GERALD, etc.). This is the house voice. Extract 10 concrete rules from it (see A.2). |
| `sgilson7/pdf-redactor`, `sgilson7/perturbation-workbench` | The house web architecture: `crates/core` (Rust + serde) → `crates/wasm` (wasm-bindgen) → `web/app.js` + `web/index.html`, no bundler. Download = `Blob` → `URL.createObjectURL` → `<a download>`; upload = `<input type=file>` → `file.arrayBuffer()`. Copy this pattern exactly. |
| `PLANNING-BRIEF.md` §C | Known engine bugs to fix before reuse. |
| `tikz_figure_prompt.md` | The only sanctioned way to produce art (§B, M6). |

### A.2 Resolve the tone reference

The requested tone is "**fully turtle dick derived, but more specific**." The agent must
**not guess** what this refers to. Resolution order:

1. Look for a file, repo, or gist under `sgilson7` whose name or README matches the phrase.
2. If not found, **ask the user for the reference document** before writing any prose.
3. Once located, write `TONE.md`: 10–15 concrete, checkable rules derived from the reference
   *and* from the existing event prose, each with a one-line example. "More specific" means
   every rule must be something a reviewer can point to in a sentence and say yes/no about —
   e.g. "characters count things and report the count," "the narrator never explains a joke,"
   "no adjective that a monster could not itself use." No rule may be a vibe word.

`TONE.md` is a deliverable of M0 and every event, enemy description, town, and NPC line
written later is checked against it in review.

### A.3 Make the design decisions the brief leaves open

PLAN.md must state a decision and a one-paragraph rationale for each of these. The brief's
recommendation is in italics; the agent may deviate but must say why.

| Decision | Recommendation |
|---|---|
| Engine reuse | *Fork `crates/engine` into `crates/core` as a git subtree (not a dependency — we will edit it). Keep it graphics-free. Add `serde` derives; that breaks the zero-dep purity, and the house repos already accept serde in core.* |
| Rendering | *Canvas 2D in vanilla JS per the house pattern. No framework, no bundler. Tiles as 32 px squares; 20×20 map = 640×640 viewport.* |
| Save format | *Versioned JSON: `{ "format": "gm2d-save", "version": 1, "state": {...} }`. `version` bumps on any incompatible change and every version has a migration or an explicit "cannot load" message. The wasm boundary exposes exactly `save_json() -> String` and `load_json(s) -> Result`.* |
| Board growth schedule | *Boards start at 6×3 per slot (half the original's 6×8). Each level adds one row to a slot in a fixed rotation: weapon → chest → helmet → gloves → greaves → repeat. Rotation is deterministic so a save's board sizes are derivable from level, which makes the save robust. Requires a per-slot `grow_board(slot, by)` — the engine only has all-slots.* |
| XP curve | *Exponential-ish: `xp_to_next(L) = 20 · 1.35^(L−1)`, rounded. Combat XP = enemy rating from `rating.rs`. Tile events may grant XP. Must be tuned so level 5 lands around 25–35 fights.* |
| Danger model | *Each tile has `terrain` (base encounter rate) and belongs to a `region` with an enemy pool. `danger = mean(rating.rs score of each enemy loadout in the pool)`. Encounter chance on entering a tile = `terrain.base × f(danger)` with `f` monotone; enemy drawn weighted by rating so hard enemies are rarer in every region. All draws from `rng.rs`, seeded, state saved.* |
| Losing | *No unconditional bounty (see §C). Loss returns the player to the last town with gold intact but no reward. Whether loss costs XP is a tuning knob, default no.* |
| Classes | *Three at MVP. Each is a tree of 8–12 nodes. Class choice is permanent within a save.* |

### A.4 Write PLAN.md in this shape

```
1. Decisions (A.3 table, filled in, with rationale)
2. Repo layout and conventions (§B.0)
3. Milestones M0–M6 (§B), each with:
     - Goal (one sentence)
     - Deliverables (files/features, checkable)
     - Acceptance tests (things `make test` or a human can verify)
     - Deployable? (yes/no, and what the deployed page lets a visitor do)
     - Risks specific to this milestone
4. Save-format schema, version 1, as a JSON example
5. Data authoring formats (tiles, events, enemies, skills) as JSON/RON examples
6. Open questions for the user (things the agent could not decide)
```

Then stop and present it.

---

## Part B — Milestone skeleton the plan must follow

The agent fills in detail but must keep this order and these deployment gates. A milestone
is not complete until `make test` passes, `make web` builds, and the deploy gate (where
present) is live at `https://sgilson7.github.io/<repo>/`.

### B.0 Repo layout and conventions (applies to all milestones)

```
<repo>/
  PLANNING-BRIEF.md   this file
  PLAN.md             the agent's plan (Part A output)
  TONE.md             tone rules (M0)
  CLAUDE.md           agent operating notes, kept current
  Cargo.toml          workspace: crates/core, crates/wasm
  crates/core/        forked engine + world + progression; serde; no graphics; no wasm
  crates/wasm/        wasm-bindgen shim only: save_json, load_json, step, render-state getters
  web/                index.html, app.js, style.css, assets/ — no bundler
  data/               tiles.json, events.json, enemies.json, skills.json (authored content)
  art/                *.tex TikZ sources (M6) and compiled SVGs in web/assets/
  docs/               GitHub Pages output, produced by `make publish`
  Makefile            make test | check | web | serve | publish | art
```

Rules:
- `crates/core` must never import wasm-bindgen or anything DOM-shaped. Every rule is
  testable with `cargo test -p core` in a few seconds, as in the original.
- Every milestone adds tests. Save round-trip tests run from M1 onward on every commit.
- `make publish` rebuilds `docs/` and pushes. Only the agent's human runs it, not the agent.
- Content lives in `data/*.json`, never in Rust literals, so tone edits are data edits.

### M0 — Foundation and tone
**Goal:** an empty page deploys; the engine compiles as `core`; the voice is written down.
- Deliverables: workspace with `core` (forked engine, §C bugs fixed, serde on `Run` and
  everything it owns), `wasm` shim exporting `hello()`, `web/` that loads wasm and prints the
  engine's piece count; `TONE.md`; `CLAUDE.md`; CI running `make test`.
- Acceptance: original engine test suite passes in the fork; the three §C regressions have
  named tests; `make web` yields a page that says "core: N pieces".
- **Deploy gate 1:** yes — a visitor sees the page load and the wasm respond.

### M1 — Save and load, before anything else
**Goal:** prove the round-trip on the smallest possible state so every later system is built
on top of it, not retrofitted.
- Deliverables: `SaveFile` v1 with `format`/`version`/`state`; `save_json`/`load_json` across
  the wasm boundary; download button (Blob → `<a download>`) and load input
  (`file.arrayBuffer()`), both copied from the house repos; a *convenience* autosave to
  `localStorage` that is clearly labelled as not the real save; a migration stub for v1→v2.
- Acceptance: property test — for any reachable `Run`, `load(save(r)) == r` including RNG
  state; loading a file with a wrong `format` or a future `version` fails with a sentence, not
  a panic; the download works in Chrome, Firefox, Safari.
- **Deploy gate 2:** yes — a visitor can change a number, download, reload the page, upload,
  and see the number come back. Boring on purpose.

### M2 — The world
**Goal:** walk a 20×20 map with terrain, events on tiles, and encounter rolls.
- Deliverables: `data/tiles.json` (20×20 grid: terrain, region, optional event id);
  `data/events.json` (tile-bound events in the engine's existing `LadderEvent` shape so the
  old event machinery is reused, prose per `TONE.md`); terrain table (base encounter rate,
  passable, movement cost); regions with enemy pools and computed danger; player movement
  (arrow/WASD, one tile per press); encounter roll on tile entry per A.3; a debug overlay
  showing each tile's terrain/danger/roll; one starting town tile with a shop (reusing
  `shop.rs`) and a rest point.
- Acceptance: a seeded walk of a fixed path produces the same encounter sequence every run;
  danger numbers are derived from `rating.rs`, not typed; every event id in `tiles.json`
  exists in `events.json` (test); map is fully reachable from the start tile (test).
- **Deploy gate 3:** yes — walk around, hit events, encounters show a placeholder "a fight
  would happen here" card. Save/load restores position and RNG.

### M3 — Fights on the map
**Goal:** an encounter opens the gear board, runs the deterministic fight, and resolves.
- Deliverables: encounter → loadout screen (port the board UI to Canvas; drag, rotate, drop;
  keep the green/red fit preview and gold/red assembly outline) → fight replay with the
  per-item cooldown bars → result; `data/enemies.json` with each enemy as a real loadout
  (keep the original's "monsters wear the catalogue" rule and its assembles-or-fails test);
  gold on win only; loss returns to last town; shop restock rules.
- Acceptance: the fight log for a given loadout/enemy is byte-identical to the original
  engine's for the same inputs (golden tests); the `hit_for` display equals the first hit
  in the log (this is the §C.2 regression test, now at the UI level).
- **Deploy gate 4:** yes — the full loop. This is the first build worth sending to a friend.

### M4 — Leveling and the base skill tree
**Goal:** the grind has a reason.
- Deliverables: XP and level on the character; `grow_board(slot, rows)` in core; level-up
  applies the rotation from A.3 and shows *which* board grew and by how much; `data/skills.json`
  base tree (10–15 nodes; each node is a stat change or a rule change expressed in engine
  terms, e.g. "+1 accessory allowed in the weapon recipe"); one point per level; skill screen;
  all of it in the save.
- Acceptance: board dimensions are a pure function of level (test); a level-5 save loaded
  into a fresh session has the same boards and the same spent points; no skill node can be
  bought twice or without its prerequisite.
- **Deploy gate 5:** yes.

### M5 — Classes (MVP complete)
**Goal:** level 5 forks the character.
- Deliverables: class-choice screen at level 5 (three classes, each described in `TONE.md`
  voice, each with a one-line mechanical promise); three class trees in `skills.json`;
  class-locked nodes; class recorded in the save; a save from before level 5 loads and still
  prompts at level 5.
- Acceptance: the MVP checklist in §0 is walked by hand and every line is a yes; a full
  play-through from level 1 to a class choice is scripted as an integration test using the
  seeded RNG so it is reproducible.
- **Deploy gate 6:** yes — tag `v0.1.0-mvp`.

### M6 — Art and a tone pass (post-MVP, optional)
- Deliverables: for each enemy, town, and NPC that needs a picture, one `.tex` file in `art/`
  written by filling in `tikz_figure_prompt.md` exactly (standalone class, tunables at the
  top, `\foreach` for repeats, self-check at the end). `make art` runs `pdflatex` +
  `pdftocairo -svg` if present; otherwise the README tells the human to compile in Overleaf
  and drop the SVG into `web/assets/`. Until then the game uses geometric placeholders.
- A second pass over every string in `data/` against `TONE.md`.
- Deploy gate: yes, but not a version bump.

---

## Part C — Engine bugs to fix in M0 before reuse

Found and verified by playing the original headlessly. Each gets a regression test.

1. **Bounty is paid on a loss.** `run.rs`: `self.gold += bounty` executes before the
   win/loss `match`. With Grinder mode's one-rung knockback this is an unbounded, risk-free
   gold farm (measured: +17 gold per lose/win cycle, forever). In an open world this becomes
   the whole game. Fix: pay inside the `Victory` arm only.
2. **Displayed damage is double-scaled.** `loadout.rs`: `ItemProfile.stats` is stored
   pre-multiplied by power (`scaled_stats`), then `hit_for()` multiplies by power again.
   A weapon that hits for 30 in the log displays "hits 46". Fix: `hit_for` must use unscaled
   flat damage, or `stats` must stop being pre-scaled; pick one and test that the display
   equals the log's first hit.
3. **Purchase message quotes the catalogue price, not the charged price.** `cli/main.rs`
   prints `registry.def(id).price`; the shop lists and charges a run-scaled price. Not a
   core bug, but the new UI must display the charged amount.

Also inherit, deliberately: no RNG in combat; 50 ms ticks; monsters are loadouts; the naming
system. Do not "simplify" any of these — they are the reason to keep the engine.

---

## Part D — Guardrails for the agent

- Do not start any milestone until the previous one's deploy gate is live and the human has
  seen it. Long grindy games are killed by systems that were never played in sequence.
- Never write a game string without `TONE.md` open. If `TONE.md` does not exist yet, you are
  in M0 and should be writing it.
- Save/load is tested on every commit from M1 on. A red round-trip test blocks everything.
- Keep `core` graphics-free and wasm-free. If you find yourself importing web-sys in core,
  stop.
- Prefer editing `data/*.json` to editing Rust when the change is content.
- When the brief and PLAN.md disagree after approval, PLAN.md wins; note the divergence in
  CLAUDE.md.

## Part E — Open questions to put to the user in PLAN.md

- What is the "turtle dick" tone reference? (Blocking for TONE.md.)
- Should losing a fight cost anything beyond the missed reward — XP, gold, gear?
- Are the three MVP classes named by the user, or should the agent propose them from the
  existing catalogue's natural groupings (physical / spell-book / crystal-ball)?
- Does "download at any time" include mid-fight? (Recommendation: yes, by saving the
  pre-fight state and seed; the fight replays identically on load.)
