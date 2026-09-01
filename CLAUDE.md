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

## The economy, and why the shelf stopped rolling

Three changes that are one change: **a character starts with almost nothing, a
town sells a fixed shelf, and a town asks you for something.**

- **The starting kit is `Oak Handle` + `Iron Blade`.** It was eleven components
  — most of a helmet, a pair of molds and a whole weapon — which made the shop
  decoration for the first hour. Two pieces assemble one weapon that beats a
  Cave Rat and a Bog Toad and loses to a Bone Archer, which is the opening.
- **The Iron Blade is seated turned, and has to be.** It is one cell wide and
  **four tall**; a starting weapon frame is three rows. Upright it does not fit
  anywhere, the weapon assembles nothing, and a character who cannot win cannot
  earn — the M4 soft-lock, exactly. The fifth field of a `STARTER` row is the
  rotation and this is what it is for.
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
- Reroll and pinning are gone with the random shelf. A town that sells
  something different every visit is not a place, and three of them are one
  slot machine in three costumes.

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

## Your figure is your class's

`art.player` is the Sprocketman — who you are before anybody has decided what
you are. The fork is where that stops being true and it does not come off, so
the panel draws `art.classes[canonical]` from then on. Repainted on every
`paintPanel`, so a loaded save arrives wearing its own figure rather than
waiting for the next fork.

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
| Board rebuilt against the original | 411 passing |
| The other side's gear, and a tree that says what it does | 419 passing |
| Shops, errands and a replay of both sides | 425 passing |
| Components that show their shape and explain themselves | 427 passing |
| The sheet, and a fight that opens holding what it holds | 429 passing |
| Catalogue | 528 components |
| Ladder | 50 creatures |
| `crates/core` | ~33k lines, down from ~50k |
| wasm | 888 KB |
| Save format | v1 |
| Map | 20×20, 5 regions, 3 towns, 6 events |
| Bestiary | 50 creatures, rated 16 to 2958 |
| Starting kit | **2 components**, 28 Fnorp, 1 assembled weapon |
| Towns | 3, fixed shelves of 11 / 15 / 17, no reroll |
| Errands | 3, one to a town |
| Boards | 6×3 at level 1, one row a level, 6×8 ceiling |
| Level 5 | ~27 fights, mean of nine seeded walks |
| Skill trees | 13 base nodes + 3 × 8 class nodes, every one stating its effect |
| Figures | 13 family drawings + 4 drawn for themselves + 3 classes → 71 SVGs |
| Art coverage | 50 of 50 creatures, 3 of 3 towns, 3 of 3 classes, and you |

Note the catalogue is **528**, not the 374 the retheme document counts — it
grew upstream after that document was written. Any content work that quotes a
catalogue size should quote this one.

## Open questions the human has not answered

Listed in `PLAN.md` §6. None block M1.

**Answered:** the repo is `sgilson7/gear-master-2d`, public, Pages served from
Actions.

**Still open**, with the default in force: losing costs nothing but the walk
back; the content charter is binding; invented proper nouns fail the M2 lint.

**No longer open:** errands exist, as `crates/core/src/quest.rs` — a new module
rather than upstream's, which was a chain of receipts along a road. `town.rs`
stays dropped: a town is a place on the map plus a shelf in `shops.json`, and
does not need a module.
