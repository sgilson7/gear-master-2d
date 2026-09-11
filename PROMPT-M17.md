# The prompt

Copy everything between the rules into a fresh Claude Code session, in the
repository root, with `PLAN-M17.md` already placed there and M16 landed.

---

You are picking up an existing game project to execute its next block of
work. Everything you need is written down; nothing has been started.

**Read `HANDOFF.md` first, entire**, then `CLAUDE.md` from "Rules" to the end
of "Commands", and these sections of it: *The map screen*, *Twenty-four maps,
and where they live*, *`world::step`*, *The walker*, and *Determinism*. Then
read `HANDOFF-M16.md` §2 for how the last block reported its divergences.
Then read `PLAN-M17.md`, which is the plan you are executing.

The block is one ask from the human, in their own words:

> I want the overworld to be moved around using online pool / mini golf game
> logic, where you pull an indicator and fire yourself pinball style to a
> destination; if you land in an area you have a chance for a fight, if you
> land on an overworld node you enter it. this will let traversal in the
> overworld be much more interesting. I want some pinball style obstacles
> and kinda like helicopter game, like spikes that deal tiredness damage or
> send you back to town or something

The plan reads *the overworld* as the two country maps — the Treyway and the
Undercountry — and nothing else. Every floor keeps its steps. If that reading
is wrong the human will say so at the first deploy point; do not widen it on
your own.

Execute in the order `PLAN-M17.md` §7 gives: **M17.0 through M17.5.** Two
deploy points, after M17.3 and after M17.5.

Work milestone by milestone. For each one:

1. **Do the recon the milestone asks for before writing anything**, and put
   what you found in the commit message. Three things in this plan are
   claims the recon has to check: that nothing on a shot map reads
   `tiles-walked` (§6); that the Treyway's eight obstacles leave every place
   reachable in six shots or fewer (§4.1, and the lint says the real number);
   and every constant in §3, which were chosen on paper — a full-power shot
   that dies in two bounces or twelve means the constants move, and the
   commit says which and why.
2. **The physics is core's and it is integers.** `shot.rs` has no `f32`, no
   `f64`, and no dependence on iteration order of anything but the tick.
   `a_shot_is_the_same_in_every_engine` hashes a fixed `Flight` and the
   browser gate checks the wasm build produces the same hash; if the two
   ever differ, the block stops until they do not.
3. Build the rule in core and the picture in the shim. The cue is the
   shim's; what it sends is a `Shot`; what comes back is a `Flight`; the
   shim draws the `Flight` and decides nothing. A landing is `Step`'s own
   resolution called at rest — do not write a second one.
4. Write the tests the plan's acceptance column names, **break each one and
   watch it fail before you keep it**, then restore. The reachability lint
   for shot maps is written before the Treyway is set to `shot`, so that
   setting it is the commit that turns the lint on.
5. Run `make test`, then `make web` and `make test-ui`, then `make play` and
   **read the transcript**. From M17.4 on the walker crosses the Treyway by
   shooting, and the number of shots it took goes in the commit against the
   plan's nine.
6. Commit in the house style — `git log -10` first.
7. **Stop at each deploy point and ask.** You do not `git push` or
   `make publish` on your own judgement.

Six standing constraints for this block:

- **Landing is the event.** Nothing rolls, enters, or pays in flight except
  the five obstacles' contacts. A creature that stops a ball is not in this
  plan.
- **A shot costs nothing; a contact costs what the table says.** No fatigue
  per shot. A spike is eight, a pocket is twelve and the walk home, and
  nothing on a table kills you or takes carried experience.
- **Nothing leaves the map.** Edges reflect like walls. A ball in the sea is
  a bug, and `nothing_leaves_the_map` runs every shot the suite makes.
- **Floors still step.** `check_a_floor_still_steps` is in the browser gate
  so that a regression on West Bambulon's arrow keys is red.
- **No new save fields.** A save holds a tile. If you find you need to save
  a velocity, the shot did not run to rest inside one call, and that is the
  fault.
- **Every tape line goes through `log()`, with `TONE.md` open.** A shot is a
  sentence: what it hit, in order, and where it stopped, with the per-mille.
  Counted, never dramatised.

`PLAN-M17.md` §10 lists five decisions that are the human's. Ship without
the assist (§10.2) and leave `aim_at` public. Do not redraw the Treyway
(§10.3). Take twelve for the pocket (§10.4) and landing-only encounters
(§10.5), and say in the handoff how the table felt in the transcript so
the human can move either.

If you find something the plan got wrong — and the physics constants at
least will be — say so, propose the change, and record it as a divergence
with its reason. The one to watch for: **corners.** §3 says a diagonal
entry into a wall cell resolves on the shorter axis. If that produces a
ball that sticks in a corner or tunnels through one, the resolution is the
thing to fix and `a_bounce_keeps_the_tangent` is the test that should have
caught it; write the corner case into it before fixing.

Start with M17.0. It ships a physics module nobody can see and a map flag
nobody has set, and every one of its tests is about a ball on a table that
does not exist yet.

---

## Notes for whoever hands this over

- This is a stretch goal and it is architected as one: M17.0 through M17.2
  are engine and data with no player-visible change, so the block can stop
  after any of them with nothing half-shipped. M17.3 is the first thing a
  player sees and it is the first deploy point.
- The wasm/native hash check in M17.0 is the block's real risk. Integer
  arithmetic is the same everywhere; the temptation to reach for a `sqrt`
  for the angle is not. Use a 72-entry table of `(cos, sin)` in sixteenths
  and no trigonometry at runtime.
- `playthrough.py` moves by keyboard steps and reads the strip. On a shot
  map it needs `aim_at` exposed through the wasm surface as a query, the
  same way `kit_reading_json` is; do not have the walker simulate physics in
  Python.
- The Treyway is 16×16 and stays so. The plan says in §9 why it is not
  redrawn and in §10.3 that it could be; if the transcript shows the table
  is too small to be interesting, that is the finding to report, not a
  thing to fix in this block.
