# PLAN.md — Gear Master 2D

Produced by executing Part A of `PLANNING-BRIEF.md`. No game code has been written.

**Upstream read:** `sgilson7/gear-master` @ `e93a391`, `sgilson7/pdf-redactor`,
`sgilson7/perturbation-workbench`, `TurtleRichard/gear-master-td-retheme.md`,
`tikz_figure_prompt.md`.

**Target repo:** `sgilson7/gear-master-2d` → `https://sgilson7.github.io/gear-master-2d/`

---

## 0. What recon changed

The brief was written against an older `gear-master`. Nine of its factual claims are now
stale, and one of the corrections is large enough to reshape three milestones. Everything
below is verified against `e93a391`, with file:line.

| Brief says | Actually | Consequence |
|---|---|---|
| Engine is 8 modules, ~79k lines | 24 modules. `county`, `town`, `route`, `shop`, `dungeon`, `quest`, `relic`, `curse`, `class`, `pedestal`, `rumour`, `theme`, `stats`, `slot`, `shape`, `bestiary`, `share` all exist | Keep/drop decision is now the biggest scoping call — §1.1 |
| Zero dependencies | Still true (`crates/engine/Cargo.toml` has an empty `[dependencies]`) | serde is still a real, deliberate break — §1.3 |
| "Requires a per-slot `grow_board(slot, by)` — the engine only has all-slots" | `Run::grow_slot(SlotKind)` exists at `run.rs:4746`, calling `Loadout::grow_one(kind, 1)`. `Slot::with_rows(kind, rows)` exists at `slot.rs:60` | M4 gets substantially cheaper |
| §C.1 bounty-on-loss is a bug | Deliberate and documented at `run.rs:3494-3501`; the cost of losing is delegated to `Mode` | Reframed as a **design change**, not a regression fix — §1.7 |
| §C.2 damage double-scaled | **Confirmed.** `loadout.rs:1028` stores `scaled_stats = stats.times(times).powered(power)` into `ItemProfile.stats`; `hit_for` at `loadout.rs:108` multiplies by `power/100` again | Real. Named regression test in M0 |
| §C.3 shop price message | `crates/cli/src/main.rs` only; GM2D has no CLI | Becomes a UI rule, not a fix |
| "the original is hosted on GitHub Pages" via `docs/` + `make publish` | gear-master does use `docs/`, but it ships **macroquad-wasm** (`docs/gearmaster.wasm` + `mq_js_bundle.js`). The two house web repos use `packaging/package-web.sh` → `dist/web/` → `.github/workflows/deploy.yml` | Divergence from §B.0 — §1.10 |
| Board UI can be ported | The board UI is `crates/gui/src/main.rs`, **16,462 lines of macroquad**. wasm-bindgen + Canvas 2D shares no code with it | Dominant cost of M3 — §1.2 |
| TONE.md must be derived from scratch | **The Turtle Dick retheme is already shipped.** `theme.rs:313` defines `TURTLE_DICK` (id `"td"`), 2,275 lines, with `story`, `pieces`, `monsters`, `classes`, `vocabulary`, `cutscenes`, `notes`, `told`, `glossary`. `naming.rs` already has the rarity-scaling `Naming` struct with `attributives` | The retheme doc is the **authoring baseline for new GM2D content**, and the shipped `td` theme is the **voice sample** TONE.md gets its rules from |

**And one finding the brief could not have anticipated, which sets the shape of M1:**

`Run` is threaded with `&'static` references — `&'static ClassDef`, `&'static MonsterSpec`,
`&'static Town`, `&'static Dungeon`, `&'static Theme`, `&'static Brawl`, `&'static Quest`,
and roughly twenty `&'static str` / `&'static [&'static str]` fields (`run.rs:21-354`).
You cannot `#[derive(Deserialize)]` through a `&'static T` that points into a `const` table.
Serializing is easy — write the id. Deserializing requires resolving an id back to a static,
which no derive can do.

So the save layer is **not a derive**. It is an explicit owned mirror type with a hand-written
resolver in both directions. That is a designed component, not an attribute, and it is the
single best argument for the brief's instinct to put save/load at M1 rather than retrofit it.

---

## 1. Decisions

### 1.1 Engine reuse — *deviates from the recommendation*

**Decision:** copy `crates/engine` into `crates/core` as a **plain vendored fork with a
provenance file**, not a git subtree. Then delete the campaign and keep the simulation.

*Why not a subtree.* A subtree earns its ceremony when you intend to `git subtree pull`.
We do not. GM2D deletes the ladder, de-`&'static`s half the public types, and adds serde to
everything — the fork diverges totally in M0 and never converges. A subtree that is never
pulled is a worse copy with extra commands. Instead `crates/core/UPSTREAM` records
`sgilson7/gear-master @ e93a391` and the date, so provenance is a fact rather than a
mechanism.

*What is kept, dropped, and adapted.* This is the real decision. gear-master is two things
welded together: a **deterministic simulation** (boards, packing, combat, rating, naming) and
a **campaign** (a 49-rung ladder with a road, towns, a county, dungeons and quests hung off
it). GM2D wants the first and writes its own second.

| Module | Verdict | Note |
|---|---|---|
| `piece`, `shape`, `slot`, `stats`, `loadout`, `combat`, `rating`, `rng` | **Keep verbatim** | The reason to fork at all. Zero changes beyond serde and the §C.2 fix |
| `naming`, `theme` | **Keep, de-`&'static`** | Becomes data-loaded — §1.9 |
| `shop`, `class`, `curse` | **Keep, adapt** | Shop loses run-scaling by rung, gains scaling by level. `ClassDef.requires: [(Axis, i32)]` is replaced by "chosen at level 5" |
| `event` | **Keep the shape, drop the content** | `LadderEvent`/`Choice`/`Trigger`/`Requirement`/`Outcome` become `TileEvent` with `at: (u8,u8)` instead of `at: rung`. The 3,400 lines of ladder prose do not come across; the *voice* does |
| `run` | **Do not port.** Harvest it | 5,498 lines, ~90 fields, most of them ladder state (`rung`, `walking_the_parish`, `perambulation_way`, `county_*`, `outer_dungeons`, `took_exits`). GM2D writes `character.rs` + `world.rs` fresh and lifts the four things worth having: `settle()`'s combat banking, the `BoardSnapshot` undo stack (`run.rs:207`), `grow_slot`, and `restock` |
| `county`, `dungeon`, `route`, `quest`, `relic`, `pedestal`, `rumour`, `bestiary`, `town`, `share` | **Drop** | Campaign. `county.rs` is read for its *discipline*, not its code — see below |

*`county.rs` is the model to imitate.* Its header states the exact invariant GM2D's world
needs: "**It is derived, never stored.** `generate` is a pure function of a seed... the run
keeps only where you are standing and what you have cleared. Nothing here ever touches
`Run::rng`." GM2D's map is authored rather than generated, but the same split holds: the
20×20 grid lives in `data/tiles.json` and is never saved; the save holds position, the
cleared/answered set, and the RNG. That is what makes a save small and robust.

*Cost.* Deleting is fast; the test suite is the risk. 59 test files are string-keyed against
ladder content that is going away. Expect to drop roughly half of them and keep the ones that
guard the simulation (`packing`, `assembly`, `assembly_bonuses`, `validity`, `primitives`,
`catalog_shape`, `effects`, `reference_builds`, `prices`, `fight`, `two_runs`). M0 is not
done until the kept suite is green.

### 1.2 Rendering — *accepts the recommendation, and prices it*

**Decision:** Canvas 2D in vanilla JS, house pattern, no framework, no bundler. 32 px tiles;
the 20×20 map is a 640×640 canvas. A second canvas for the gear board.

*The cost the brief does not name.* `crates/gui/src/main.rs` is 16,462 lines of macroquad and
none of it survives the move to wasm-bindgen. Drag, rotate, drop, the green/red fit preview,
the gold/red assembly outline and the cooldown bars are all being written a second time.
This is the largest single line item in the plan and it all lands in M3.

*The mitigation, which is also the architecture.* JS owns **drawing and hit-testing only**.
Every question about legality goes back to Rust, which already answers all of them:
`Slot::can_place` (`slot.rs:221`), `Slot::legal_anchors` (`slot.rs:289`),
`Slot::items_with_locks` (`slot.rs:460`), `Slot::groups`, `Shape::rotated`. The fit preview is
`legal_anchors` rendered; the assembly outline is `items_with_locks` rendered. If JS ever
computes a rule, that is the bug.

*Rejected alternative:* ship macroquad-wasm like the original and skip the rewrite. It would
cost days instead of weeks, but it forfeits the whole house pattern — no DOM, no file input,
no `<a download>`, a multi-megabyte binary, and a save/load story that has to be invented
inside a game loop. Save/load is the hard gate. The house pattern exists because it makes
save/load trivial. Rewrite the UI.

### 1.3 Save format — *accepts the recommendation, and specifies the part that is hard*

**Decision:** versioned JSON, `{ "format": "gm2d-save", "version": 1, "state": {...} }`.
The wasm boundary exposes exactly `save_json() -> String` and `load_json(&str) -> Result`.
Schema in §4.

*The id-indirection layer.* `crates/core/src/save.rs` defines `SaveState` — a plain owned
struct with no lifetimes and no `&'static` anywhere — plus:

```rust
impl From<&Game> for SaveState        // infallible: statics become ids
impl TryFrom<SaveState> for Game      // fallible: ids resolve, or a sentence
```

Every `&'static T` field becomes an id string on the way out and a table lookup on the way
back. A `&'static str` flag becomes a `String`; `Game` holds `Vec<String>` rather than
`Vec<&'static str>` so nothing has to be leaked. This is mechanical but it is not free, and
it is why serde derives alone would have quietly failed at the boundary.

*Three specifics that bite.*

1. **The RNG.** `Rng { state: u64 }` is private (`rng.rs:9`). Add `pub fn state(&self) -> u64`
   and `pub fn from_state(u64) -> Rng`. Saving the *seed* is not enough — the stream has
   advanced, and the brief's requirement is the RNG state, not the seed.
2. **The registry.** `PieceRegistry { instances: Vec<Instance { def, rotation }> }`
   (`piece.rs:1371`), where `def` is an **index into `CATALOG`**. An index is stable only
   while catalog order is. Save the canonical **name** and the rotation instead, plus a
   catalog fingerprint, so a catalog edit produces "this save is from an older catalogue"
   rather than a rat wearing a crown.
3. **`PieceId` is an index into that registry.** So boards and inventory serialize as small
   integers into the saved `registry` array. Cheap, readable, and exactly the engine's model.

*Versioning.* `version` bumps on any incompatible change. `load_json` refuses a wrong
`format` or a future `version` with one sentence and no panic. A `migrate_v1_to_v2` stub with
a test ships in M1 so the second version is never the first time the mechanism runs.

### 1.4 Board growth schedule — *accepts the recommendation*

**Decision:** boards start at **6×3** per slot. Each level adds one row in the fixed rotation
weapon → chest → helmet → gloves → greaves → repeat. `Slot::with_rows` (`slot.rs:60`) and
`Loadout::grow_one` already do the work.

Board size is a pure function of level:

```
rows(slot, L) = 3 + |{ n : 2 ≤ n ≤ L, ROTATION[(n-2) mod 5] == slot }|
```

so a save carrying `level` implies its board dimensions and cannot disagree with them.
Skill nodes that grant rows out of turn (§5, `grow_slot_rows`) are the one exception, so
the saved shape is `rows(slot, level) + granted[slot]`, and `granted` is derivable from the
spent skill list. Still a pure function, still tested.

*One consequence to state.* The engine's `SLOT_H` is 8 and the catalog's enemy loadouts in
`combat.rs` place gear at coordinates up to `y = 7`. **Enemies keep full 6×8 boards**; only
the player's frames start small and grow. The original's "monsters wear the catalogue" rule
and its assembles-or-fails test carry over unchanged, and the player spends the whole early
game outgunned in board area — which is the intended shape of a grindy game, not a bug.

### 1.5 XP curve — *accepts the recommendation, with a calibration test*

**Decision:** `xp_to_next(L) = round(20 · 1.35^(L−1))` → 20, 27, 36, 49, 66, 89, 121…
Reaching level 5 costs 132 XP cumulative.

Combat XP is the beaten enemy's total loadout rating from `rating.rs`, divided by a tuning
constant `XP_DIVISOR`. Tile events may grant flat XP. `XP_DIVISOR` is set by a **calibration
test**, not by taste: a scripted seeded playthrough on the authored map must reach level 5 in
25–35 fights, and the test fails outside that band. That converts the brief's tuning target
into something CI holds.

Computed in integers throughout. `rating.rs` uses `f32` constants internally but returns
`i32`; the curve rounds once, at the table, and the table is a `const [i32; 32]` generated by
a test that also proves it matches the formula. Nothing in the XP path does float math at
runtime, so a level-up is bit-identical everywhere.

### 1.6 Danger model — *accepts the recommendation, integer-only*

**Decision:**

- Each tile has a `terrain` and belongs to a `region`.
- `terrain` carries `base` (encounter chance in **per-mille**), `passable`, and `cost`.
- `region` carries an enemy pool. `danger(region) = mean(total loadout rating of each enemy
  in the pool)`, computed at load from `rating.rs` — **derived, never typed.** A test asserts
  no `danger` field exists in any data file.
- On entering a tile: `chance‰ = clamp(terrain.base · (100 + danger·100/DANGER_REF) / 100, 0, 450)`,
  all integer. Roll `rng.below(1000) < chance‰`.
- Enemy drawn from the pool weighted by `(max_rating + 1 − rating)`, so hard enemies are
  rarer in every region, per the brief.
- Every draw comes from the saved `Rng`. No other stream exists.

*Why per-mille and not floats.* A seeded walk must produce the same encounter sequence on
every machine and in every browser. Float rounding is the one thing that can break that
silently. The acceptance test for M2 is a fixed path producing a fixed sequence; integers are
how it stays true.

### 1.7 Losing — *reframes §C.1 as a design change*

**Decision:** no bounty on a loss. A loss returns the player to the last town with gold and
gear intact, and no reward. Losing costs no XP by default; `LOSS_XP_PCT` exists as a tuning
knob and ships at 0.

*Why this is a change and not a fix.* `run.rs:3494-3501` argues, correctly, that in a ladder
run with a knockback a bounty-less loss leaves a player with no income and nothing to do but
replay a fight they know they lose. That reasoning is sound **because the ladder is a
corridor**. GM2D is not a corridor: a player who loses can walk to a lower-danger region,
farm something they can beat, and come back. The open world supplies the escape hatch the
ladder could not, which removes the justification and leaves only the exploit — the brief's
measured +17 gold per lose/win cycle, which in an open world with no rung to knock back is
unbounded and free.

So: the bounty moves inside the `Victory` arm, the regression test the brief asked for is
written, and `CLAUDE.md` records that this is a deliberate divergence from upstream's
documented intent rather than a bug we found in it.

### 1.8 Classes — *proposes three, answering Part E Q3*

**Decision:** three classes, chosen permanently at level 5, each an 8–12 node tree.

The engine's `class.rs` already has `ClassDef { name, blurb, requires: [(Axis, i32)], power }`,
but its classes are **earned by crossing stat thresholds**, not chosen. GM2D keeps the struct
and drops `requires` — choice at level 5 is the brief's mechanic and it is a different thing.

Names come from the shipped `td` theme's class table (`theme.rs:336+`), so they are already
in voice and already sourced to the book:

| Class | Groups | Mechanical promise (one line, per §B M5) |
|---|---|---|
| **Gorillathon** | physical | Weapons hit harder the fewer items you are wearing. |
| **Funnel Sergeant** | the Funny (mana / spells) | Casts that would fail for want of Funny fire anyway, once per fight. |
| **Worm-Fact Keeper** | mind / curse | Curses you land do not expire while the target is above half health. |

`Corkwright` (armour) is the alternate if playtesting says three is one too few or the mind
tree is thin. Class is recorded in the save as an id; a pre-level-5 save loads and still
prompts at 5.

### 1.9 Theme becomes data — *new decision, not in the brief*

**Decision:** `crates/core/src/theme.rs` keeps its shape and loses its `&'static`. `Theme`,
`Retold` and `Naming` become owned types, loaded at startup from `data/theme.td.json`.

*Why this is the right seam.* §B.0 requires that "content lives in `data/*.json`, never in
Rust literals, so tone edits are data edits." `theme.rs` already implements exactly that
separation in the type system — its header is worth quoting because it is the rule:

> Every name the engine works with — `"Oak Handle"`, `"Cave Rat"` — is a **key**, not a
> label. […] a theme is a lookup from the canonical name to the one on screen. […] **a theme
> cannot break the game.** A missing entry falls through to the canonical name.

That fall-through is what makes tone iteration safe: a half-finished retheme is a game with
some untranslated words, not a game that will not start. Moving the tables from `static` to
JSON changes where they live and nothing about how they work — and we are de-`&'static`-ing
these types for serde anyway (§1.3), so the two jobs are one job.

*Consequence for the retheme doc.* `gear-master-td-retheme.md` §8's checklist says to edit
374 `name:` fields in `piece.rs` and 49 in `combat.rs`. **Do not.** That checklist predates
`theme.rs`, and the work it describes has since been done properly as a theme. The retheme
doc's standing role in GM2D is what the user asked for: the **baseline artifact for writing
new story and events** — its cast, its places, its substance ladders (Cork → Vinyl → Sneel →
Time-Sap-Tempered → Ypytryktrium), its naming corpora, and above all its §2 content charter,
which is binding on every string GM2D ships.

### 1.10 Deploy — *deviates from §B.0*

**Decision:** follow the two *newer* house repos, not the original. `packaging/package-web.sh`
builds into `dist/web/`; `.github/workflows/deploy.yml` builds and publishes to Pages on push
to `main`. There is no `docs/` directory and no human-run `make publish`.

*Why.* §B.0 was describing gear-master, which predates both pdf-redactor and
perturbation-workbench and ships a macroquad bundle. Those two repos are the pattern the
brief actually wants us to copy — `crates/core` + `crates/wasm` + `web/` + Actions — and they
agree with each other. Copying the older mechanism to satisfy a sentence would be
cargo-culting the wrong half of the same instruction. `make publish` survives as an alias for
`git push` with the test suite in front of it, so the muscle memory still works.

The brief's rule that only the human deploys is kept and strengthened: the agent never runs
`git push`.

---

## 2. Repo layout and conventions

```
gear-master-2d/
  PLANNING-BRIEF.md        the brief
  PLAN.md                  this file
  TONE.md                  tone rules (M0)
  CLAUDE.md                agent operating notes, kept current
  tikz_figure_prompt.md    the only sanctioned way to produce art
  Cargo.toml               workspace: crates/core, crates/wasm
  crates/
    core/
      UPSTREAM             sgilson7/gear-master @ e93a391
      src/                 piece shape slot stats loadout combat rating naming rng
                           curse shop class theme event character world save
      tests/               inherited simulation tests + new ones
    wasm/src/lib.rs        shim only: save_json, load_json, step, render getters
  web/
    index.html  app.js  style.css  assets/
  data/
    tiles.json  terrain.json  events.json  enemies.json  skills.json  theme.td.json
  art/                     *.tex TikZ sources (M6)
  packaging/package-web.sh
  .github/workflows/{ci.yml,deploy.yml}
  Makefile                 test | check | web | serve | publish | art
```

**Rules.**

- `crates/core` never imports `wasm-bindgen`, `web-sys`, or anything DOM-shaped. If you are
  reaching for one, stop.
- `cargo test -p gm2d-core` runs in seconds and is the gate on every commit.
- Save round-trip tests run from M1 onward on every commit. A red round-trip blocks
  everything.
- Content goes in `data/*.json`. If a change is a string, it is not a Rust change.
- Never write a game string without `TONE.md` open.
- The agent does not run `git push` or `make publish`.
- Where this file and the brief disagree, this file wins and `CLAUDE.md` records why.
  The standing divergences are §1.1 (fork not subtree, campaign dropped), §1.7 (bounty is a
  design change), §1.9 (theme as data), §1.10 (Actions not `docs/`).

---

## 3. Milestones

A milestone is not complete until `make test` passes, `make web` builds, and — where there is
one — the deploy gate is live and the human has seen it. **No milestone starts before the
previous gate is live.**

### M0 — Foundation and tone

**Goal:** an empty page deploys, the fork compiles as `core`, and the voice is written down.

**Deliverables**
- Workspace with `crates/core` (vendored fork + `UPSTREAM`) and `crates/wasm`.
- Campaign modules deleted per §1.1; kept modules compile; kept tests green.
- serde derives on everything the save will touch, with the `&'static` fields already
  converted to owned ids (§1.3) — done here, not in M1, because it is a type change and
  M1 should be about the round-trip, not about lifetimes.
- §C.2 fixed: `hit_for` uses unscaled flat damage, with a test asserting the displayed number
  equals the first hit in the combat log.
- §C.1 changed: bounty moves inside the `Victory` arm (§1.7), with a test that a loss pays
  nothing.
- §C.3 recorded in `CLAUDE.md` as a UI rule: the shop displays the **charged** price.
- `Rng::state()` / `Rng::from_state()`.
- `crates/wasm` exporting `hello()` and `piece_count()`.
- `web/` that loads the wasm and prints `core: N pieces`.
- `data/theme.td.json` — the shipped `td` theme, moved out of Rust (§1.9), with a test that
  every canonical key it names exists.
- **`TONE.md`** — 10–15 checkable rules (see below).
- `CLAUDE.md`, `Makefile`, `.github/workflows/ci.yml` running `make test`.

**On TONE.md.** The reference is resolved and there is nothing to ask. It is derived from
three sources, in this order of authority:
1. `gear-master-td-retheme.md` **§2, the content charter** — binding, not advisory. What is
   excluded outright, what is renamed, what is kept, "violence stays cartoon-grade," and the
   handling of the title gag.
2. The shipped `td` theme in `theme.rs` — the **voice sample**. This is what the register
   actually sounds like in production, and rules are extracted from it rather than invented.
3. The engine's existing event prose (`event.rs`) — the deadpan-concrete house style.

Every rule must be checkable by pointing at a sentence and saying yes or no. Working examples
drawn from the material, each with its evidence:

- *Characters count things and report the count.* — "You try them twice more, which is twice
  more than you need to." (`theme.rs`, the Henpeck cutscene)
- *The narrator never explains a joke.* — Henpeck's "I am not a *retailer*" is the last word
  on the subject.
- *A reversal is delivered flat, in the shortest available sentence.* — "Then a gambler in a
  coat made of money fell through the roof of it."
- *No adjective a monster could not itself use.*
- *Scale is stated as a number, never as an intensifier.* — 1.79 trillion residents; 7,583 HP;
  the 45th annual race. Not "countless."
- *Every proper noun is sourced to the book or the CSV.* A reviewer can ask for the page.

**A register note the rules have to settle.** The engine's own event prose is ominous English
countryside (`THE THEODOLITE`, Ackworth, "THREE LINES CROSS SOMEWHERE. THE CROSSING IS NOT
MARKED."); the book is broad absurdist comedy (Ponkey Dong, the Funny, a moon dropped by a
sandwich). The shipped `td` theme has already reconciled them and TONE.md should codify how:
**the frame is deadpan and the contents are absurd.** The prose never winks; the facts inside
it are ridiculous. Anything that winks is off-register regardless of which source it came from.

**Acceptance**
- Kept upstream tests pass in the fork; the deleted ones are deleted deliberately and listed
  in `CLAUDE.md`, not silently dropped.
- Three named regression tests exist: `loss_pays_no_bounty`, `hit_for_matches_log`,
  and `shop_quotes_charged_price` (as a display-layer assertion).
- `make web` yields a page reading `core: N pieces` with the real N.

**Deployable:** yes — **gate 1**. A visitor loads the page and the wasm answers.

**Risks**
- *Deletion is the whole milestone.* 24 modules and 59 test files, heavily cross-referenced.
  The failure mode is a half-deleted campaign that still compiles. Mitigation: delete
  module-at-a-time, commit per module, and let the compiler drive.
- De-`&'static`-ing `Theme` and `Naming` touches a 2,275-line file. Mechanical, but wide.

---

### M1 — Save and load, before anything else

**Goal:** prove the round-trip on the smallest possible state, so every later system is built
on top of it rather than retrofitted into it.

**Deliverables**
- `crates/core/src/save.rs`: `SaveState` (owned, no lifetimes), `From<&Game>`,
  `TryFrom<SaveState> for Game`, the id resolvers, and the catalog fingerprint.
- Schema v1 exactly as §4.
- `save_json() -> String` and `load_json(&str) -> Result<(), String>` on the wasm boundary,
  and nothing else save-shaped.
- Download button: `Blob` → `URL.createObjectURL` → `<a download>`, copied from
  `pdf-redactor/web/app.js:94`.
- Load input: `<input type=file>` → `file.arrayBuffer()`, copied from the same file, line 183.
- A `localStorage` autosave, **labelled in the UI as a convenience and not the real save.**
- `migrate_v1_to_v2` stub with a test that exercises the dispatch.

**Acceptance**
- Property test: for any reachable `Game`, `load(save(g)) == g`, **including the RNG state**.
  Equality is derived, so a field added later and forgotten in the mirror fails the test.
- A save with a wrong `format`, a future `version`, or a mismatched catalog fingerprint fails
  with one sentence and no panic. One test per case.
- The mirror is exhaustive: a `#[test]` that construction of `SaveState` from `Game` names
  every field, so adding a field to `Game` without adding it to the save is a compile error,
  not a silent data loss.
- Download works in Chrome, Firefox and Safari (`make test-ui`, Playwright, per the house
  repos' `testing/run.sh`).

**Deployable:** yes — **gate 2**. A visitor changes a number, downloads, reloads the page,
uploads, and the number comes back. Boring on purpose.

**Risks**
- The exhaustiveness test is the load-bearing one. Without it, M4 adds a field, the round-trip
  still passes, and a level-5 character silently loads at level 1. Write it first.
- Safari's handling of `createObjectURL` downloads is the most likely browser-specific
  failure; it is why the gate names three browsers.

---

### M2 — The world

**Goal:** walk a 20×20 map with terrain, tile-bound events, and encounter rolls.

**Deliverables**
- `data/terrain.json` — the terrain table (§5).
- `data/tiles.json` — the 20×20 grid, regions with enemy pools, and placed events/towns (§5).
- `data/events.json` — tile-bound events in the adapted `LadderEvent` shape, so the existing
  event machinery is reused; prose per `TONE.md`, proper nouns from the retheme baseline.
- `crates/core/src/world.rs` — the map, movement, and the encounter roll of §1.6.
- Player movement: arrow keys and WASD, one tile per press, blocked by `passable`.
- Encounter roll on tile entry; a placeholder result card.
- Debug overlay (a key toggle) showing each tile's terrain, region, computed danger, and the
  last roll.
- One starting town tile with a shop (adapted `shop.rs`) and a rest point.

**Acceptance**
- A seeded walk along a fixed path produces the same encounter sequence every run, and the
  same sequence after a save/load in the middle of the walk. This is the test that proves
  §1.6's integer discipline.
- Danger numbers are derived from `rating.rs`; a test asserts no data file contains a
  `danger` key.
- Every event id in `tiles.json` exists in `events.json`, and every event is placed at most
  once (test).
- Every passable tile is reachable from the start tile (flood fill, test).
- Every proper noun in `events.json` appears in the retheme baseline or the CSV (a lint, so
  invented lore is caught at review rather than at ship).

**Deployable:** yes — **gate 3**. Walk around, hit events, see "a fight would happen here."
Save and load restores position, cleared set and RNG.

**Risks**
- Authoring 20×20 tiles plus events is the first real content load. It is where the schedule
  slips if TONE.md is thin, which is why TONE.md is M0.
- The temptation to reach for `county.rs`'s generator. Do not: GM2D's map is authored so it
  can be *designed*, and generation would make the danger gradient an accident.

---

### M3 — Fights on the map

**Goal:** an encounter opens the gear board, runs the deterministic fight, and resolves.

**Deliverables**
- Encounter → loadout screen → fight replay → result.
- The board UI in Canvas 2D: drag, rotate, drop, the green/red fit preview, the gold/red
  assembly outline, the per-item cooldown bars. Legality answered by Rust only (§1.2).
- Undo, ported from `run.rs:207`'s `BoardSnapshot` stack.
- `data/enemies.json` — every enemy a real loadout of catalogue pieces (§5), keeping the
  original's "monsters wear the catalogue" rule.
- Gold on a win only; a loss returns to the last town (§1.7); shop restock on level rather
  than rung.

**Acceptance**
- **Golden tests:** the combat log for a given loadout and enemy is byte-identical to the
  upstream engine's for the same inputs. Fixtures captured from `e93a391` in M0, before
  anything is deleted, and committed.
- Every enemy in `enemies.json` assembles, or the test fails loudly — the upstream
  assembles-or-fails test, carried over. A typo must not leave a monster harmless.
- The `hit_for` display equals the first hit in the log — the §C.2 regression, now asserted at
  the UI level as the brief requires.
- Mid-fight save round-trips: saving during a replay stores the pre-fight state and seed, and
  the fight replays identically on load (§6, Q4).

**Deployable:** yes — **gate 4**. The full loop. First build worth sending to a friend.

**Risks**
- **This is the milestone that can blow the schedule.** The board UI is a from-scratch
  rewrite of the largest file upstream. Mitigation: build it in the order fit-preview →
  drag/drop → rotate → assembly outline → cooldown bars, and ship the gate as soon as a fight
  can be won, with polish after.
- Golden fixtures must be captured in M0. Capturing them after the fork has diverged proves
  nothing.

---

### M4 — Leveling and the base skill tree

**Goal:** the grind has a reason.

**Deliverables**
- XP and level on the character; the curve of §1.5 as a generated const table.
- Level-up applies the §1.4 rotation and says **which** board grew and by how much.
- `data/skills.json` — base tree, 10–15 nodes, each node a stat change or a rule change
  expressed in engine terms (§5).
- One point per level; a skill screen; all of it in the save.

**Acceptance**
- Board dimensions are a pure function of level plus granted rows (test, §1.4).
- A level-5 save loaded into a fresh session has the same boards, the same spent points and
  the same gold.
- No node can be bought twice, or without its prerequisite, or without a point (three tests).
- The calibration test of §1.5: a scripted seeded playthrough reaches level 5 in 25–35 fights.

**Deployable:** yes — **gate 5**.

**Risks**
- The XP band is the first thing that will need retuning against a real map. It is a constant
  and a test, so retuning is a one-line change plus a green suite — which is the point of
  making it a test rather than a feel.

---

### M5 — Classes (MVP complete)

**Goal:** level 5 forks the character.

**Deliverables**
- Class-choice screen at level 5: three classes (§1.8), each described in `TONE.md` voice with
  its one-line mechanical promise.
- Three class trees in `skills.json`; class-locked nodes; class recorded in the save.
- A save made before level 5 loads and still prompts at level 5.

**Acceptance**
- The §0 MVP checklist is walked by hand and every line is a yes.
- A full playthrough from level 1 to a class choice is scripted as an integration test on the
  seeded RNG, so it is reproducible.
- Class choice is permanent within a save (test: no path clears it).

**Deployable:** yes — **gate 6**. Tag `v0.1.0-mvp`.

**Risks**
- Three trees is three times the content of M4's one, and it is the last thing before the tag.
  Node effects should reuse M4's effect vocabulary rather than inventing per-class mechanics.

---

### M6 — Art and a tone pass (post-MVP, optional)

**Goal:** pictures, and a second pass over every string.

**Deliverables**
- For each enemy, town and NPC that needs a picture: one `.tex` in `art/`, written by filling
  in `tikz_figure_prompt.md` exactly — `\documentclass[tikz,border=4pt]{standalone}`, every
  tunable in a `\newcommand` or `\definecolor` at the top, positions in the figure's own units
  with `x=`/`y=` set once, `\foreach` for repeats, sections in drawing order, annotations last,
  geometric primitives only, and the 3–5 line self-check at the end.
- `make art` runs `pdflatex` + `pdftocairo -svg` when present; otherwise the README tells the
  human to compile in Overleaf and drop the SVG into `web/assets/`. Until then, geometric
  placeholders.
- A second pass over every string in `data/` against `TONE.md` and the retheme charter.

**Deploy gate:** yes, no version bump.

**Note.** The prompt is written for lecture slides, so its "Audience / style" field needs a
standing GM2D answer rather than a per-figure guess. Proposed: *"a 32-px-tile 2D RPG; flat
fills, heavy outlines, no gradients; must read at 64×64 px on a 640×640 canvas and again at
4× in a bestiary panel."* The self-check requirement is what makes these reviewable, so it is
not optional here either.

---

## 4. Save format, version 1

Elided with `…` for length; the shape is complete.

```json
{
  "format": "gm2d-save",
  "version": 1,
  "catalog": {
    "pieces": 374,
    "fingerprint": "b1946ac92492d234"
  },
  "state": {
    "rng_state": 15241094284759029579,
    "theme": "td",
    "name_seed": 6510615555426900570,

    "character": {
      "level": 4,
      "xp": 71,
      "gold": 240,
      "class": null,
      "skill_points": 1,
      "skills_taken": ["frame-sense", "loose-fit"],
      "granted_rows": { "weapon": 1, "chest": 0, "helmet": 0, "gloves": 0, "greaves": 0 },
      "grown_health": 0,
      "wins": 26,
      "losses": 3
    },

    "registry": [
      { "def": "Oak Handle", "rot": 0 },
      { "def": "Tin Rim",    "rot": 1 },
      { "def": "Cork Plate", "rot": 0 }
    ],
    "owned": [0, 1, 2],

    "boards": {
      "weapon":  { "rows": 5, "placed": [ { "id": 0, "x": 0, "y": 3 },
                                          { "id": 1, "x": 1, "y": 3 } ] },
      "chest":   { "rows": 4, "placed": [ { "id": 2, "x": 2, "y": 0 } ] },
      "helmet":  { "rows": 4, "placed": [] },
      "gloves":  { "rows": 3, "placed": [] },
      "greaves": { "rows": 3, "placed": [] }
    },

    "locks": [ { "slot": "weapon", "pieces": [0, 1], "offsets": [[0,0],[1,0]] } ],

    "world": {
      "at": [7, 12],
      "last_town": "the-end-of-all-gears",
      "events_answered": ["the-cork-boundary", "the-sap-ditch"],
      "flags": ["knows-the-buyer"],
      "counters": [["fights-won", 26], ["tiles-walked", 411]]
    },

    "shop": {
      "restocks": 12,
      "shelf": ["Sneel Edge", "Vinyl Cuff", "Thrumbus Tread"]
    },

    "pending_fight": null
  }
}
```

**Notes on the shape.**

- `rng_state` is the RNG's *current* state, not the seed. This is what makes the brief's
  "restoring the exact random-encounter RNG" true rather than approximately true.
- `registry` is the ordered list of piece instances. `owned` and every `placed.id` are indices
  into it, exactly mirroring `PieceId`. Instances are stored by **canonical catalogue name**
  and rotation, never by catalogue index (§1.3).
- `catalog.fingerprint` hashes the catalogue's canonical names in order. A mismatch produces
  *"This save was made with an older catalogue (374 pieces, b1946ac9). Load it in that
  version, or start a new game."* — a sentence, not a panic.
- **`name_seed` and `locks` are not optional, and M0 found out why.** Item names
  are hashed from the arrangement *and* the seed, so a save without it restores
  every stat correctly and renames every item — the golden fixture came back
  with "Resonant Thorn" where "Resonant Sliver" went in, and nothing else
  looked wrong. Locks are state rather than geometry: two pieces that touch are
  one item unless a lock says otherwise, and a rebuild that re-derived them
  produced a board with more items than it started with. Both are in
  `CLAUDE.md` under "things the fork learned the expensive way".
- `boards[slot].rows` is stored **and** checked: `rows == 3 + rotation_rows(level) +
  granted_rows[slot]`. Storing it makes the save readable; checking it makes it honest.
- `theme` is an id. The theme's contents are content and live in `data/`, never in a save.
- `world` stores position, what has been answered, and flags. The map itself is never saved —
  it is `data/tiles.json` and it is derived, per §1.1.
- `pending_fight` is `null` outside combat. Inside it:
  `{ "enemy": "A. Rat", "at": [7, 12], "rng_state_before": 15241094284759029579 }` — the
  pre-fight state and seed, so the fight replays identically on load (§6, Q4).

---

## 5. Data authoring formats

All files carry `format` and `version` and are validated at load; a bad file names its
problem in a sentence.

### `data/terrain.json`

```json
{
  "format": "gm2d-terrain", "version": 1,
  "terrain": {
    "grass":  { "glyph": ".", "passable": true,  "cost": 1, "encounter_per_mille": 90 },
    "scrub":  { "glyph": ",", "passable": true,  "cost": 1, "encounter_per_mille": 140 },
    "wood":   { "glyph": "T", "passable": true,  "cost": 2, "encounter_per_mille": 200 },
    "slag":   { "glyph": "%", "passable": true,  "cost": 2, "encounter_per_mille": 260 },
    "rock":   { "glyph": "^", "passable": false, "cost": 0, "encounter_per_mille": 0 },
    "water":  { "glyph": "~", "passable": false, "cost": 0, "encounter_per_mille": 0 },
    "road":   { "glyph": "=", "passable": true,  "cost": 1, "encounter_per_mille": 30 },
    "town":   { "glyph": "#", "passable": true,  "cost": 1, "encounter_per_mille": 0 }
  }
}
```

### `data/tiles.json`

Rows as glyph strings so the map is editable as a picture, which is how a 20×20 map should be
authored.

```json
{
  "format": "gm2d-tiles", "version": 1,
  "width": 20, "height": 20,
  "start": [3, 17],
  "rows": [
    "^^^^^^^^^^^^^^^^^^^^",
    "^%%%,,....TTTT...,,^",
    "^%%,,.....TTT....,,^",
    "…"
  ],
  "regions": [
    { "id": "the-pit",     "name": "The End of All Gears",
      "tiles": [[0,14],[0,15],[1,14],"…"],
      "enemies": ["A. Rat", "Bengulon Jungle Toad", "Wallspider Swarm"] },
    { "id": "west-bambulon", "name": "West Bambulon",
      "tiles": ["…"],
      "enemies": ["The Crimper", "Frosty Kev", "The Brumpus"] }
  ],
  "places": [
    { "at": [3, 17], "kind": "town",  "id": "the-end-of-all-gears" },
    { "at": [9, 4],  "kind": "event", "id": "the-cork-boundary" }
  ]
}
```

`danger` is deliberately absent — it is computed from each region's enemy pool via
`rating.rs`, and a test asserts the key never appears (§1.6).

### `data/events.json`

The engine's `LadderEvent` shape with `at: rung` replaced by tile placement, which lives in
`tiles.json` so an event can be moved without editing its prose.

```json
{
  "format": "gm2d-events", "version": 1,
  "events": [
    {
      "id": "the-cork-boundary",
      "title": "THE CORK BOUNDARY",
      "trigger": "on-enter",
      "once": true,
      "blocked_by": [],
      "prose": [
        "The fence is cork. Not cork-coloured, not cork-like. Somebody has grown a fence out of it, and it has kept growing, and it is now four feet thick and going west at the rate of a hand a year.",
        "A man is cutting a doorway through it with a bread knife. He has been at it long enough to have a system. He does not look up.",
        "\"Ninth one,\" he says. \"They close.\""
      ],
      "choices": [
        {
          "label": "Take a strip",
          "blurb": "It comes away like bark and starts back while you are holding it.",
          "requires": { "none": {} },
          "outcome": { "all": [ { "flag": "has-cork" }, { "xp": 8 } ] },
          "unmet": ""
        },
        {
          "label": "Buy the knife",
          "blurb": "Forty Fnorp. He has nine more.",
          "requires": { "gold": 40 },
          "outcome": { "all": [ { "gold": -40 }, { "piece": "Sneel Edge" } ] },
          "unmet": "Forty Fnorp, and you have not got it."
        }
      ]
    }
  ]
}
```

`Requirement` and `Outcome` are externally tagged enums mirroring `event.rs`, so the existing
event machinery evaluates them unchanged.

### `data/enemies.json`

Every enemy is a real loadout of catalogue pieces. `gear` entries are
`[piece name, slot, x, y, rotation]`, matching `GearPlacement`.

```json
{
  "format": "gm2d-enemies", "version": 1,
  "enemies": [
    {
      "name": "A. Rat",
      "health": 40, "strength": 2, "regen": 0,
      "mind_resist": 0, "curse_resist": 0,
      "physical_resist": 0, "magic_resist": 0,
      "attacks": [ { "name": "bite", "damage": 3, "cooldown_ms": 900 } ],
      "gear": [],
      "items": [],
      "gear_offset": 0,
      "bounty": 6,
      "rank": "ordinary",
      "drops": []
    },
    {
      "name": "The Crimper",
      "health": 260, "strength": 9, "regen": 1,
      "mind_resist": 0, "curse_resist": 20,
      "physical_resist": 15, "magic_resist": 0,
      "attacks": [],
      "gear": [
        ["Oak Handle",  "weapon", 0, 6, 0],
        ["Sneel Edge",  "weapon", 1, 6, 0],
        ["Cork Plate",  "chest",  2, 4, 0]
      ],
      "items": [2, 1],
      "gear_offset": 0,
      "bounty": 34,
      "rank": "mini-boss",
      "drops": ["Crimping Jaw"]
    }
  ]
}
```

Enemy boards are the full 6×8 (§1.4), which is why `y` reaches 6.

### `data/skills.json`

Node effects are expressed in engine terms, never in prose.

```json
{
  "format": "gm2d-skills", "version": 1,
  "trees": [
    {
      "id": "base", "class": null,
      "nodes": [
        { "id": "frame-sense", "name": "Frame Sense", "cost": 1, "requires": [],
          "effect": { "grow_slot_rows": { "slot": "weapon", "rows": 1 } },
          "blurb": "One more row on the weapon frame, out of turn." },
        { "id": "loose-fit", "name": "Loose Fit", "cost": 1, "requires": ["frame-sense"],
          "effect": { "recipe_allows": { "slot": "weapon", "kind": "accessory", "extra": 1 } },
          "blurb": "A weapon will take one more accessory than the recipe says." },
        { "id": "thick-skull", "name": "Thick Skull", "cost": 1, "requires": [],
          "effect": { "stat": { "mind_resist": 8 } },
          "blurb": "Songil is survivable. Barely." }
      ]
    },
    {
      "id": "funnel-sergeant", "class": "funnel-sergeant",
      "nodes": [
        { "id": "issue-funnel", "name": "Army-Issue Funnel", "cost": 1, "requires": [],
          "effect": { "stat": { "mana": 15 } },
          "blurb": "Standard issue. It has somebody else's name on it." }
      ]
    }
  ]
}
```

Three effect kinds at MVP — `stat`, `grow_slot_rows`, `recipe_allows` — because a small
vocabulary reused across four trees is what keeps M5 from being four times M4's work (§3, M5
risks). New kinds are added when a node needs one, not in advance.

---

## 6. Open questions

**Answered by recon; listed so the record is closed.**

- **Q1 — What is the "turtle dick" tone reference?** Resolved. `../TurtleRichard/`:
  *Turtle Dick: Tales from the Crypt* (Dukes, Gilson & Spiess, 2019, 125 pp.), the 161-row
  `TD Titles and Characters - StoryTitles.csv`, and `gear-master-td-retheme.md`. Further,
  the retheme has **already been implemented** upstream as `theme.rs`'s `TURTLE_DICK`, which
  GM2D inherits as `data/theme.td.json` and uses as the voice sample for `TONE.md` (§1.9, M0).
  Per your instruction, the retheme doc is the baseline artifact for new GM2D story and events.
- **Q3 — Are the three classes named by the user?** Proposed, not assumed: **Gorillathon**,
  **Funnel Sergeant**, **Worm-Fact Keeper**, with **Corkwright** as the alternate (§1.8). All
  four are already in the shipped `td` class table, so they are in voice and sourced. Say the
  word if you want different ones.
- **Q4 — Does "download at any time" include mid-fight?** Yes, by the recommended mechanism:
  `pending_fight` stores the pre-fight state and seed, and the fight replays identically on
  load (§4, M3 acceptance).

**Still yours to answer. None of them block M0.**

1. **Does losing cost anything beyond the missed reward?** §1.7 ships `LOSS_XP_PCT = 0` — a
   loss costs the walk back to town and nothing else. The knob exists; the default is
   "no". Worth revisiting after gate 4, when there is something to feel.
2. ~~**Is `gear-master-2d` the repo name?**~~ **Answered.**
   `sgilson7/gear-master-2d`, public, Pages served from Actions. Live at
   <https://sgilson7.github.io/gear-master-2d/>.
3. **How binding is the retheme's §2 content charter on the *new* material?** I have treated
   it as binding — no raunch, no drugs or alcohol, no bathroom humour, no real public figures,
   cartoon-grade violence — since it was written for a publicly hosted build and GM2D is one.
   Say so if the standard has moved.
4. **May new proper nouns be invented, or only drawn from the book and CSV?** M2's acceptance
   currently includes a lint that every proper noun traces to the baseline. That is the strict
   reading and it keeps the world coherent, but a 20×20 map has more corners than 161 titles
   have names. If invention is allowed, the lint becomes a warning and TONE.md gains a rule
   for what an invented name has to sound like.
5. **Anything in the dropped campaign you want kept?** §1.1 discards `county`, `dungeon`,
   `route`, `quest`, `relic`, `pedestal`, `rumour`, `bestiary`, `town`. `quest.rs` and
   `town.rs` are the two with an obvious future in an open world, and keeping either is
   cheaper now than reviving it later.

---

## 7. What happens next

On approval: M0, in the order — scaffold the workspace, capture the golden combat fixtures
from `e93a391` **before deleting anything**, vendor the fork, delete the campaign
module-at-a-time, fix §C.1 and §C.2 with their named tests, de-`&'static` and serde the kept
types, move the `td` theme to `data/`, write `TONE.md`, then the wasm shim and the page that
says `core: N pieces`.

Then stop at gate 1 and wait for you to look at it.
