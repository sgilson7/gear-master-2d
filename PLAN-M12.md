# PLAN-M12.md — the full frame

Written 2026-08-31, while M11 is in flight. This block does not depend on
M11's content — no new maps, no new creatures, no survey machinery — but it
does assume **M11.0's output log** is live (everything this block says to the
player lands there) and it must not collide with M11's two save seams, so its
own piece decisions are seam-aware (§5). If M11 slips, only that log
dependency matters.

**Five milestones, each behind the standing gate:** `make test`,
`make test-ui` in three browsers, `make play` read by a person. Four are
**deploy points** (§5). The block's product is not a feature list — it is a
felt change: *boards under pressure*. So the block opens by measuring
pressure and closes by checking the measure moved (§3, M12.0 and M12.4).

---

## 0. The ask

> A friend asked for a way to re-roll the shop in town. That goes against the
> design goals — deterministic, a designed power curve — but he raised a good
> point underneath it: until your frame is full you feel no tension and no
> need to grow it, and right now you essentially cannot fill any of the
> frames.

The decision, made in brainstorm and recorded here: **no reroll.** The
complaint is real but the reroll is the wrong cure — the disease is that
cells outnumber pieces, so the board reads as inventory space instead of a
puzzle. Tension in this game *is* board pressure, and pressure has two
levers: raise piece throughput, or slow cell growth. This block pulls both,
three ways: a **bargain barrel** (permanent cheap filler, so frames fill
early and the chase becomes *which filler dies for the good piece*),
**commissions** (order the piece you want, pay up front, it arrives on the
world's clock — the deterministic answer to "I want the thing, not the
lottery"), and **slower cells** (rows stop being a level's allowance and become
earnings — bought with skill points, won off quest lines — so growth is
something you choose over power, not something that dilutes you on
schedule).

Shelved with intent, candidates for M13, not to be smuggled in here:
harvesting pieces off beaten creatures' boards, and stock that turns over on
a world-state clock. Both good; neither in this block.

## 1. What recon found, and what it changed

**The original restocked constantly.** Worth saying out loud: gear master 1
restocked the shop after every fight and sold rerolls for 1g. The friend is
homesick for real DNA. This block's answer is still no — but the plan should
know it is declining an ancestor, not an invader, and the barrel and the
ledger are how the same appetite gets fed deterministically.

**The catalogue probably already has the filler.** 536 pieces, many
unthemed. Before authoring a single new common, M12.1 greps for existing 1×1
and 1×2 pieces cheap and weak enough to be barrel stock. Every piece reused
is seam avoided; **if new smalls are unavoidable they all land in M12.1, the
block's only possible fingerprint move**, and the deploy note says so with
the standard sentence. The strong recommendation is zero new pieces (§8
row 1) — a barrel of familiar junk is more in voice than a barrel of new
junk anyway.

**Board growth is one site, and one pillar.** Rows-per-level lands in one
place (the level-up path that calls the per-slot grow), and *board dimensions
are a pure function of level* is an existing test. M12.3 retires both — the
row-per-level loop was an MVP pillar (`PLAN.md` M4) and this block takes it
down deliberately, with the owner's name on the decision, replacing it with
rows as **earnings**: skill nodes and quest rewards. The recon question was
whether old saves survive, and the answer is baking: on first load under the
new rules, a save's current dimensions are written into a per-slot ledger as
already-earned, so nothing anyone has is taken away and the pure-function
test is succeeded by a ledger test (§3, M12.3).

**Commissions need a list in the save.** Active orders are `(piece, paid,
remaining)` — a real new `WorldState` field, exhaustively destructured in
`save.rs` like everything else, defaulting empty for old files the way
`map` did. A field, not a fingerprint: old saves load.

**The log is the mouthpiece.** Every sentence this block adds — the barrel's
stock line, the ledger entry, the arrival, the locked row's refusal and its
opening — goes through M11.0's `log()`. No new slots. That is now the
standing rule and this is the first block born under it.

## 2. The shape of the block

```
M12.0  the measure              (no push — instrumentation)
M12.1  the bargain barrel       deploy point A
M12.2  commissions              deploy point B
M12.3  slower cells             deploy point C  (scope set by the measure)
M12.4  triage + the friend      deploy point D = the block ships
```

Order is the argument: the barrel is the cheapest fix and fills frames
*today*; commissions are the mid-game sink and want the barrel underneath
them (an order matters more when the alternative is filler, not nothing);
the cells come last because they are the touchiest change and because
**M12.3's scope is decided by M12.0's probe re-run after M12.2** — if
throughput alone puts boards at target pressure, the human may shrink or
skip the schedule change entirely (§8 row 5). A block that can conclude
"the third feature was not needed" and stop is working as designed.

---

## 3. The milestones

### M12.0 — the measure

**Goal.** Board pressure becomes a number before anything tries to move it.

**Deliverables.**
- A probe in the `make play` walker: at every level gained, report **fill**
  (occupied cells / total cells, per slot and overall) and **bench depth**
  (owned pieces that fit nowhere). One line each, in the transcript.
- The target curve, written down here and testable: **overall fill ≥ 70% by
  level 3, ≥ 80% by level 6, bench depth ≥ 2 by level 5** — numbers the
  human may tune at sign-off, but numbers, because "feels tense" is not a
  gate. Tension is roughly "fill high AND bench nonempty": no room, and
  something worth making room for.
- A baseline run recorded in the plan's close-out notes: today's curve, so
  D's claim of improvement is a diff and not a mood.

**Acceptance.** Probe lines appear at every level in a fresh seeded run; the
baseline is committed; no game behavior changed (this milestone ships
nothing a player can see, and does not push).

### M12.1 — the bargain barrel *(the block's only possible seam)*

**Goal.** A permanent, fixed, cheap bin in the town shop. Frames fill early;
the tension inverts to the right direction.

**Deliverables.**
- Recon first, in the commit message: the catalogue grep for existing 1×1
  and 1×2 commons — weak stats, low price, no set membership, no rules.
  Target: **10–14 barrel pieces**, all reused if possible; any that must be
  new land here together and the deploy note carries the fingerprint
  sentence. §8 row 1 decides whether new pieces are allowed at all.
- The barrel as a fixed section of `data/shops.json`: same stock, every
  town, every visit, priced to be bought without thought (single-digit
  Fnorp). No restock logic because it never runs out and never changes —
  the barrel is furniture, and that is the point: it is the shop's floor,
  not its ceiling.
- Shop UI grows the section, visually distinct from the shelf (the shelf is
  the curated designed curve; the barrel is under the counter). Buying from
  it logs in voice — the shopkeep's opinion of the barrel is not high and
  the anthology agrees.
- Filler is *meant to die*: selling or discarding a placed barrel piece
  stays cheap and unceremonious, so replacing it with a real piece is a
  pleasure and not a decision.

**Acceptance.** Every barrel piece assembles into its slot's recipe
(assembles-or-fails extended over the barrel); the probe shows fill at
levels 1–3 up materially from baseline; barrel stock is byte-identical
across towns and visits; sell price of barrel pieces bounds the gold loop
(buying and reselling the barrel must lose money — a test, because that is a
faucet if it is wrong).

### M12.2 — commissions

**Goal.** The deterministic answer to "I want the thing, not the lottery":
order a piece, pay now, it arrives on the world's clock.

**Deliverables.**
- The ledger: a commission list per town in `data/shops.json` — pieces from
  the existing catalogue, gated the way the designed curve already gates
  (by level and by region reached), priced above shelf (ordering certainty
  costs more than finding luck). The list is authored, so the power curve
  stays authored.
- The clock: an order completes after **N fights** (§8 row 3 confirms the
  unit), N set per piece in data, ticked wherever fights resolve. Fights,
  not steps, because a step is free and a fight is the game — an order you
  can pace out by walking in circles is a wait, not a cost.
- `WorldState.commissions: Vec<(PieceId, u16)>` — paid orders and fights
  remaining — destructured in `save.rs`, empty default for old files. One
  open order per town at a time (§8 row 4), so the ledger is a choice and
  not a subscription.
- The voice: placing an order writes a ledger line in the log (the shopkeep
  counts the fights required and reports the count); each town visit with an
  order pending logs the remainder; arrival logs collection. The piece waits
  at the shop — it does not teleport into the bag, because walking back for
  it is the travel economy doing its job.
- `explain.rs` / the shop card say what a commission is the first time one
  is available (the M8 rule: a system nobody is told about is a bug
  report).

**Acceptance.** A seeded run places an order, fights N times anywhere,
collects in town; the order state round-trips the save at every stage;
refusals (insufficient Fnorp, order slot full, level gate) land in the log
in voice; the probe shows bench depth up from baseline by mid-levels.

### M12.3 — slower cells *(scope set by the measure)*

**Goal.** Rows stop arriving on a level's schedule. A row is a thing you
**earn** — a skill point spent, a quest line finished — so the board grows
when you choose growth over power, and pressure regulates itself.

**Deliverables.**
- Gate zero: **re-run the probe after M12.2 and put the numbers in front of
  the human.** If throughput alone hit the target curve, this milestone
  shrinks or is skipped outright, and the plan's close-out says so (§8
  row 5). Assuming it proceeds:
- **The level-up stops granting rows.** Levels keep everything else — the
  skill point, the banner, the curve — but the automatic row is retired.
  This retires an MVP pillar (`PLAN.md` M4's row-per-level) and the plan
  says so plainly rather than burying it: the pillar was right when pieces
  were scarce by accident; with the barrel and the ledger underneath, a
  scheduled row is dilution on a timer.
- **Rows as skill nodes.** The base tree gains **row nodes** — *Let Out the
  Helmet*, or whatever the theme calls tailoring — each granting +1 row to a
  named slot, costing the same one point a skill costs. This is the
  mechanism's whole argument: every level now poses the game's central
  question with the player's own hands — *power on the board I have, or a
  bigger board?* Recommend 6–8 row nodes in the base tree spread across
  slots, and one or two deeper in each class tree, so classes grow different
  silhouettes (§8 row 6 sets counts).
- **Rows as quest rewards.** At most one row per quest *line* (not per
  errand), named in the reward text, so a row from the world stays an event.
  The two M11.2 lines are candidates if M11 has landed; otherwise one
  existing line gets one. Rows must never enter drop tables — a row is a
  decision or an achievement, never a lottery ticket, or the whole block's
  thesis leaks.
- **The ledger and the bake.** `WorldState` grows per-slot earned-row
  counts, destructured in `save.rs`. On first load of an older file, current
  dimensions bake into the ledger as earned — nothing anyone has is taken
  away, no migration table, one branch. Dimensions become *base + earned*,
  and the retired pure-function test is succeeded by: ledger sums match
  dimensions, growth is monotonic, and every slot is capped at the
  original's 6×8 so the late game converges instead of sprawling (§8
  row 7).
- **The screens.** The level banner stops promising a row and starts
  pointing at the tree; row nodes on the skill screen show the slot and the
  new dimensions before the point is spent; quest cards name the row in the
  reward line. All of it through the log when it happens — the M8 rule,
  three ways.

**Acceptance.** A fresh save gains no row from leveling and gains one from a
row node and one from a quest line, each logged in voice; an old save loads
with its dimensions intact, baked, and its next level grants a point but no
row; ledger tests green (sums, monotonic, cap); spending a point on a row
versus a skill both round-trip the save; probe target curve met on a fresh
seeded run, and — the number this design stakes itself on — **bench depth
at the moment players buy their first row node is ≥ 1**, because a row
bought while the bench is empty means the tension inverted again.

### M12.4 — triage, and the friend

**Goal.** The block's debt paid, the measure confirmed moved, and the person
who caused the block asked whether it worked.

**Deliverables.**
- The standing sweep: consoles clean in three browsers, the `HANDOFF.md` §5
  traps walked against the new code, every new derived number found on a
  screen. `TRIAGE-M12.md` in the M11 format: every finding, severity × cost,
  blockers fixed here, the rest dispositioned openly.
- The measure, closed: baseline curve vs. final curve, side by side, in the
  close-out notes. The block claims success on the diff, not on shipping
  three features.
- A one-sitting agent spot-run against the deployed build, M11-style brief
  appendix: start fresh, fill a frame from the barrel by level 2, place and
  collect a commission, gain a locked row and open it. Findings appended to
  `PLAYTEST-M11.md`'s file or a sibling.
- **The friend plays it.** Not a formality — the block exists because of one
  sentence of his, and the exit question is specific: *did you miss the
  reroll?* His answer goes in the close-out verbatim. If it is yes, that is
  M13's opening recon, not this block's failure.

**Acceptance.** Zero known blockers; the curve diff shows the targets met;
the spot-run completed its four errands; the friend's verdict is on file.

---

## 5. Deploy points

Every milestone passes the standing gate; these four also push, on the
human's word, `git log origin/main..HEAD` checked first.

| point | after | a visitor can | note on the page |
|---|---|---|---|
| **A** | M12.1 | fill their frames from the barrel, cheaply, today | **the only possible seam**: if any barrel piece is new, the fingerprint sentence; if all reused, old saves sail through |
| **B** | M12.2 | order a piece from the ledger and fight their way to it | old saves load; the ledger explains itself on first sight |
| **C** | M12.3 | earn a row — with a point at the tree, or off a quest line | old saves keep every row, baked as earned; the note says growth changed, and that levels no longer grant rows |
| **D** | M12.4 | the block, triaged, measured, and friend-tested | the curve diff and the friend's verdict, linked |

Seam coordination with M11: M11's seams land at its points D and F. If this
block's point A ships **between** M11's seams, and M12.1 adds pieces, the
world takes three fingerprint moves in one stretch — legal but rude. The
standing preference: M12.1 reuses the catalogue and adds nothing, making
this block seamless; failing that, the human sequences A relative to M11's
F so the notes stay honest (§8 row 1 carries the decision).

## 6. SECOND-ORDER-M11.md — this block's entries

Same notebook, continued; the block adds its entries under an M12 heading.
Seeded with the four known ones:

1. **The barrel moves the floor of player power** (M12.1). Regions and
   crossings are rated against a level's expected board; a board full of
   filler is stronger than an empty one, so every early bracket eases.
   Watch: golden-fixture fight durations at levels 1–4, and whether the L5
   crossing stops being a wall — the knob is barrel stats, not region
   ratings.
2. **The barrel is a store of value** (M12.1). Anything buyable and
   sellable is a gold loop if the spread is wrong. Watch: the buy/resell
   test, and Fnorp totals in the probe run drifting up without fights.
3. **The commission clock counts fights** (M12.2). Anything else that
   counts fights — quest goals, the tower's floors, fatigue — now shares a
   heartbeat with the ledger. Watch: a fight that resolves without ticking
   (the flooded-boss variant, a quest-scripted fight), which would make some
   fights count and some not, invisibly.
4. **Rows now compete with skills for the same point** (M12.3). Everything
   tuned against "a level is a row and a point" moves: the skill trees'
   value math (every node is now priced against a row), quest lines that
   carry rows become disproportionately attractive, and every string,
   banner and errand that says "level up to grow your board" is now false,
   not half true. Watch: the banner and `explain.rs` (fixed here), any M11
   text written in flight that promises rows for levels, and the tree
   itself — if the probe shows everyone buying row nodes first and skills
   never, the row nodes are underpriced, and the knob is node depth in the
   tree, not the point cost.

## 7. What this block does not do

No reroll, in any costume — no consumable that turns the shelf over, no
"want ad," nothing. No harvest-from-creatures and no world-clock restock:
both are shelved *by name* as M13 recon, and the friend's exit interview at
D is the first input to that decision. No new maps, creatures, sets or
rules. No change to what the shelf itself stocks — the designed curve is
untouched; this block builds around it, not into it.

## 8. The human's calls, before the milestone that needs them

| # | needed by | question | recommendation on record |
|---|---|---|---|
| 1 | M12.1 | May the barrel contain new pieces (a fingerprint seam), or reused catalogue only? | reused only — a seamless block is worth more than perfect filler, and familiar junk is better voice than new junk |
| 2 | M12.1 | Barrel identical in every town, or a regional flavor line each? | identical for now — the barrel is furniture; regional barrels are an M13 idea wearing this block's clothes |
| 3 | M12.2 | Commission clock in fights or steps? | fights — a step is free and an order should cost the game, not the walking |
| 4 | M12.2 | One open order per town, per shop, or globally? | per town — it makes each town's ledger its own small promise, and the travel back is the economy working |
| 5 | M12.3 | If the post-M12.2 probe hits the target curve, does M12.3 ship anyway? | no — skip the schedule change and say so in the close-out; a block that stops early on evidence is the measure doing its job |
| 6 | M12.3 | How many row nodes, and where? | 6–8 in the base tree spread across slots, 1–2 per class tree so classes grow different silhouettes; depth in the tree is the pricing knob |
| 7 | M12.3 | Do boards cap at the original 6×8? | yes — the late game should converge on the original's canvas, not sprawl past it; lifting the cap is an M13+ decision with its own plan |
| 8 | M12.4 | Does the friend playtest before or after deploy point D? | before — his verdict belongs *in* D's note, not appended to it |
