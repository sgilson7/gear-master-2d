# The prompt

Copy everything between the rules into a fresh session, in the repository root.

---

You are picking up an existing, deployed game project to execute its next
block of work. Everything you need is written down; nothing has been started.

**Read `HANDOFF-M13.md` first, entire**, then `CLAUDE.md` from "Eleven maps,
and where they live" to the end of "The Wextreen Reach". Those sections are
the machinery this block is built on — `PlaceDef::floors`, `Drain`,
`leave_the_sitting`, `survey::mods_for` — and every one of them was shaped by
a mistake the handoff describes. Then read `PLAN-M14.md`, which is the plan
you are executing.

Execute the block in the order `PLAN-M14.md` §7 gives: **M14.0, M14.1,
M14.2, M14.3, M14.4, M14.5.** The Sump comes before the Stair on purpose —
it leans harder on the primitives, and finding them wrong on the Sump is
cheaper than finding them wrong on both.

Work milestone by milestone. For each one:

1. Do the recon the milestone asks for **before** writing content, and put
   what you found in the commit message. Three things in this plan are
   guesses the recon has to replace with counts: the two bosses' numbers
   (by DPS bracket against the walker's board, never by adding to
   Sootmother's), whether the third drops complete the instrument set, and
   whether the wading shortcut on the gallery floor saves a single tile.
2. Build it, in core wherever it is a rule. `crates/wasm` decides nothing.
   The only thing the shim answers this block is `Requirement::Surveying`,
   and it answers it the way it answers `needs_survey` — by asking the
   character and handing the answer down.
3. Write the tests the plan's acceptance column names, **break each one and
   watch it fail before you keep it**, then restore. The blind-solution
   ceilings are asserted **at their number** — `<= 45`, not `< 100` — because
   a ceiling nobody can hit is a ceiling nobody checked.
4. Run `make test`, then `make web` and `make test-ui` (three engines), then
   `make play` and actually read the transcript. From M14.2 on, the
   transcript has to show the walker *solving a floor*, and if it stops at
   one, the floor is wrong and not the walker.
5. Commit with a message in the house style — read the last ten with
   `git log -10` first. They explain *why*, name what was rejected and why,
   and say out loud when something cost a day.
6. **Stop at each deploy point and ask.** You do not `git push` or
   `make publish` on your own judgement, ever, even when the work is green
   and obviously wanted. Say what `git log origin/main..HEAD` would send.

Five standing constraints for this block:

- **No new components.** Every key, every drop, every thing a door wants is
  in the catalogue already. `PLAN-M14.md` §8 says zero and the recon has to
  keep it there.
- **Every puzzle is monotone.** Flags only grow, so no move on any floor may
  make the stair unreachable. `puzzle::solvable_blind` is the proof and it
  runs on every floor.
- **An instrument makes a floor short, never possible.** `Surveying` is
  never the only requirement on the only door.
- **Every player-visible sentence goes through `log()`**, and **never write a
  game string without `TONE.md` open.** Two bosses, six puzzles, four
  refusals and a town with nothing in it are a lot of sentences to get wrong.
- **A refusal names the thing in the way.** A wheel says what shape it wants;
  a sealed door says which dungeon is still standing; a chain that will not
  move says the room is dry. A cairn does **not** say which cairn is under
  it, because that is the puzzle.

`PLAN-M14.md` §9 lists five decisions that are the human's. Where a row
records a recommendation and the work cannot start without an answer, take
the recommendation, say in the commit that you took it, and flag it. Where
it does not, ask. The town's name is the one you must not invent: ship the
line the plan gives if it is unanswered.

If you find something the plan got wrong — and you will, because the floors
were drawn before anybody walked them — say so, propose the change, and
record it as a divergence with its reason the way `CLAUDE.md`'s divergence
table does. In particular: if a floor's blind count comes out above its
ceiling, the ceiling is not the thing to change.

Start with M14.0. It ships nothing a player can see and does not deploy; its
whole job is to make five small things true in core before nine maps are
drawn on top of them.

---

## Notes for whoever hands this over

- The prompt assumes the working directory is the repo root and the tree is
  clean at `f2d1bd4` or later, deployed at `ac2256c7`.
- `make test-ui-setup` is a one-time venv + browser install. If `make test-ui`
  fails with a missing Playwright, that is the fix.
- If a long-running playtest is on `dist/web`, everything that builds must use
  `GM2D_WEB` — see `HANDOFF-M12.md` §2. A rebuild moves the save fingerprint
  and ends the other run.
- The riskiest milestone is **M14.2**, and not because the Sump is hard: it
  is the first content in the game that asks the walker to solve something
  rather than survive it. If `make play` never leaves floor one, the whole
  block's gate is blind past that floor, and M14.5 cannot be honest about a
  transcript it does not have. Watch the count on the Cairnfield especially —
  forty-five is the plan's number, and it is the one most likely to be wrong.
- `HANDOFF-M13.md` §6 leaves one decision open — `Character::grown_health` —
  and it is still open. This block does not touch it.
