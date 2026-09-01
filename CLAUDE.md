# CLAUDE.md — operating notes

Kept current. If something here is out of date it is a bug in this file.

## What this is

A 2D tile-based open-world RPG built on the gear-assembly auto-battler forked
from `sgilson7/gear-master`. `PLANNING-BRIEF.md` is the brief; `PLAN.md` is the
plan and **wins where the two disagree**; `TONE.md` governs every string a
player reads.

**Every milestone is done.** M0–M5 shipped the MVP, tagged `v0.1.0-mvp`; the
board was then rebuilt against the original's colourblind design; M6 added the
art and the tone pass. Live at <https://sgilson7.github.io/gear-master-2d/>.

**No rest point, and there should not be one.** M2 carried it forward; M3 is
where it turns out to be nothing. Combat health resets every fight, so a rest
would restore something that was never spent. If M4 gives damage a way to
persist between fights, the town is where the rest goes.

## Rules

- `crates/core` never imports `wasm-bindgen`, `web-sys`, or anything
  DOM-shaped. If you are reaching for one, stop.
- `crates/wasm` is a shim. It moves strings across the boundary and decides
  nothing. A rule decided there is a rule the test suite cannot reach in
  seconds, and then there are two rulebooks.
- Content lives in `data/*.json`. **If you are editing a `.rs` file to change
  what a player reads, you are in the wrong file.**
- Never write a game string without `TONE.md` open.
- **Save round-trip tests run on every commit. A red round-trip blocks
  everything.** `tests/save.rs` is that suite; `testing/drive.py` walks the same
  property through three real browsers.
- **Adding a field to `Game` is a compile error until the save carries it.**
  `SaveFile::of` and `into_game` destructure exhaustively. Two fields are
  skipped on purpose and each says so where it is skipped. Do not "fix" a
  destructure by adding `..`.
- **The agent does not run `git push` or `make publish`.** Only a human
  deploys. (The one exception was the repo's creation and first push, which the
  human asked for explicitly. The rule is back in force.)
- Do not start a milestone before the previous gate is live and the human has
  seen it.

## Commands

    make test          # the engine suite, native, seconds
    make check         # fast type-check
    make web           # build dist/web/
    make test-ui       # drive the built page in a real browser
    make serve         # build and open locally
    make test-ui-setup # one-time: venv + headless chromium

Rebaseline the golden combat fixture, and say in the commit what started
fighting differently:

    REBASELINE_GOLDEN_COMBAT=1 cargo test -p gm2d-core

## Divergences from the brief

`PLAN.md` wins. These are the places it does, and why.

| # | Divergence | Where |
|---|---|---|
| 1.1 | Fork by copy with a provenance file, not a git subtree. And the campaign is dropped, not carried: eleven modules deleted, `Run` replaced by `Character`. | `PLAN.md` 1.1, `crates/core/UPSTREAM` |
| 1.7 | §C.1 is a **design change, not a bug fix**. Upstream paid the bounty on a loss deliberately and its reasoning was sound *on a ladder*. GM2D is not a corridor, so the justification goes and the exploit stays. | `crates/core/src/reward.rs` |
| 1.9 | The theme becomes data. `theme.rs` already treats a name as a key rather than a label; moving its tables to `data/` is where they belong. | `PLAN.md` 1.9 |
| 1.10 | Actions builds and publishes to Pages. No `docs/`, no human-run `make publish` rebuild. The brief described gear-master, which predates both house web repos and ships macroquad. | `.github/workflows/deploy.yml` |

Also true, and not in the brief because it could not have been:

- **§C.1's code was gone before the fix was written.** The bounty was paid in
  `Run::settle`, which left with the campaign. The rule now lives in
  `reward.rs` and M3's encounter resolution calls it.
- **§C.3 is not a code fix.** It was a fault in a CLI GM2D does not ship. It
  survives as a UI rule: **the shop screen displays the price actually
  charged, never `registry.def(id).price`.** It becomes a test in M3.

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

## Classes

- **Three, and they are upstream's.** Gorillathon, Funnel Sergeant and
  Worm-Fact Keeper are `Berserker`, `Hexweaver` and `Bloodletter` with the
  theme talking, so the powers — Leeching, Contagion, Bloodscent — are already
  tuned and already tested. Nothing new was invented in combat at the last
  milestone, deliberately.
- **The promise is the rule.** Each class's one-line mechanical promise is
  `ClassPower::describe()` put through `theme.retell`, so it cannot go stale and
  it speaks the game's language rather than the engine's.
- **The fork is permanent and offered until answered.** There is no path that
  clears a class; the screen is the only one in the game that does not take
  Escape. A save made at level three arrives at five and is asked, and one made
  at nine without a class is still asked — the question was never answered
  rather than declined.

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
- `data/art.json` maps a canonical creature name or place id to a figure. **A
  subject with no entry draws nothing**, which is what makes the file safe to be
  incomplete — and it will be, because there are fifty creatures and seven
  figures.

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

## Inherited on purpose — do not "simplify"

No RNG in combat. 50 ms ticks. Monsters are loadouts wearing catalogue pieces.
The naming system. These are the reason to keep the engine.

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
| Catalogue | 523 components |
| Ladder | 50 creatures |
| `crates/core` | ~33k lines, down from ~50k |
| wasm | 502 KB |
| Save format | v1 |
| Map | 20×20, 5 regions, 3 towns, 6 events |
| Bestiary | 50 creatures, rated 16 to 2958 |
| Starting kit | 11 components, 28 Fnorp, 4 assembled items |
| Boards | 6×3 at level 1, one row a level, 6×8 ceiling |
| Level 5 | ~27 fights, mean of nine seeded walks |
| Skill trees | 13 base nodes + 3 × 8 class nodes |

Note the catalogue is **523**, not the 374 the retheme document counts — it
grew upstream after that document was written. Any content work that quotes a
catalogue size should quote this one.

## Open questions the human has not answered

Listed in `PLAN.md` §6. None block M1.

**Answered:** the repo is `sgilson7/gear-master-2d`, public, Pages served from
Actions.

**Still open**, with the default in force: losing costs nothing but the walk
back; the content charter is binding; invented proper nouns fail the M2 lint;
`quest.rs` and `town.rs` stay dropped.
