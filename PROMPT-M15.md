# The prompt

Copy everything between the rules into a fresh session, in the repository root.

---

You are picking up an existing, deployed game project to execute its next block
of work. Everything you need is written down; nothing has been started.

**Read `HANDOFF.md` first, entire** — it is the short door in — then `CLAUDE.md`
from "Rules" to the end of "Commands", and then these four sections of it:
*Experience is carried, and a town is the bonfire*, *Levels*, *Three papers, and
two of them are refused*, and *What a creature leaves behind*. Then read
`PLAN-M15.md`, which is the plan you are executing, and skim `HANDOFF-M14.md`
for how the last block reported itself.

The block is three asks from the human, in their own words:

> I want to add a new menu called "Instant Battle", that after defeating a
> specific enemy 5 times, you can set them to instant battle, so the battle is
> resolved kind of like the rat when you have its set item; my thought is the
> battle still occurs in the background instantly, just the player is only shown
> the result through the history / ongoing event view on the map screen. having
> to click through the battle screen every time for an enemy you know for a fact
> you will kill is a little boring. you should still lose tiredness whenever you
> instant battle an enemy. also make the experience curve for levelling up less
> steep, so you level up a bit faster at higher levels. i want the experience
> needed to be quadratic up till level 50, then exponential after level 50. also
> the second class should no longer be gated behind finishing the first one,
> instead you get it whenever you can afford the 2nd paper at spike kaklons van.

Execute the block in the order `PLAN-M15.md` §4 gives: **M15.0, M15.1, M15.2,
M15.3, M15.4, M15.5.** The tally comes before the battle and the battle before
the menu, because a screen listing something the engine cannot count is a screen
built on a guess.

Work milestone by milestone. For each one:

1. Do the recon the milestone asks for **before** writing anything, and put what
   you found in the commit message. Three things in this plan are guesses that
   recon has to replace with counts: the curve's three coefficients and its
   base after fifty, what `XP_DIVISOR` becomes once the curve is flatter, and
   how far past fifty `MAX_LEVEL` should reach.
2. Build it, **in core wherever it is a rule**. `crates/wasm` decides nothing.
   Whether a creature may be marked, what an instant battle pays and what it
   costs are all rules; the shim moves the answer across the boundary and the
   page draws it.
3. Write the tests the plan's acceptance column names, **break each one and
   watch it fail before you keep it**, then restore. A check that reads its
   answer off the thing it is checking is not a check — that has been shipped
   five times here.
4. Run `make test`, then `make web` and `make test-ui` (three engines), then
   `make play` and actually read the transcript. **`make play` from a new game
   plateaus at level eleven** against the Drambus Stack's fourth floor; that is
   a fact about the walker, `PLAN.md` §6d row 3. `GM2D_FROM=testing/saves/…`
   starts a walk from a start line, and `crates/core/tests/start_lines.rs`
   writes them.
5. Commit with a message in the house style — read the last ten with
   `git log -10` first. They explain *why*, name what was rejected and why, and
   say out loud when something cost a day.
6. **Stop at each deploy point and ask.** You do not `git push` or
   `make publish` on your own judgement, ever, even when the work is green and
   obviously wanted. Say what `git log origin/main..HEAD` would send.

Six standing constraints for this block:

- **Do not reimplement the settlement.** `fight::settle` pays the bounty with
  the class's arm on it, rolls the drops, counts the errand tally, banks the
  streak and tires you four percent. An instant battle is `fight::run` followed
  by `fight::settle` with nothing drawn. **A second answer to what a win pays is
  the mistake this project has paid for six times.**
- **`fight::rout` is the precedent and not the home.** Read it first — it is the
  one path where an encounter is settled without a screen, and it has two users
  already. An instant battle differs in the one respect that matters: *the fight
  happened*, so it pays the speed bonus, rolls the drops and costs tiredness,
  and a rout does none of those.
- **Derived, never banked.** Which creatures are *eligible* is "every counter at
  five or more", worked out fresh. Only which ones are **marked** is state, and
  it is one `#[serde(default)]` field, so there is **no seam** and every save
  that opens on M14 opens on this.
- **A number that is shown needs somewhere it is read.** `Outcome::Xp` wrote
  into a counter nothing consulted for four blocks. The new per-creature counter
  is read by the menu, which is what makes it legitimate — keep it that way.
- **Every player-visible sentence goes through `log()`**, and **never write a
  game string without `TONE.md` open.** An instant battle's whole interface is
  its receipt: it is the only thing the player will see, and it has to carry a
  defeat as clearly as a win.
- **A refusal names the thing in the way.** A creature you have beaten twice
  says two of five. A boss says it is a boss. A screen that greys a button and
  says nothing is a button reported as a bug — this file has written that
  sentence six times.

`PLAN-M15.md` §5 lists four decisions that are the human's. Where a row records
a recommendation and the work cannot start without an answer, take the
recommendation, say in the commit that you took it, and flag it. Where it does
not, ask. **The curve's anchor is the one you must not invent**: there is
exactly one measured contract today — level 5 in 25–35 fights — and "less steep"
is unfalsifiable without a second. Ask for one before M15.3, and if none comes,
propose one from a `make play` transcript and say that is what you did.

If you find something the plan got wrong — and you will, because the curve was
drawn before anybody solved for it — say so, propose the change, and record it
as a divergence with its reason the way `CLAUDE.md`'s table does. **Keep a
notebook**: `SECOND-ORDER-M15.md`, written when noticed rather than at the end.
It is the cheapest thing in a block and it is what makes the last milestone a
worklist rather than a guess — M13.9 executed seven rows and three of them were
actually wrong; M14.6 did the same.

Start with M15.0. It ships nothing a player can see and does not deploy; its
whole job is to make the engine able to count what the menu is about to list.

---

## Notes for whoever hands this over

- The prompt assumes the working directory is the repo root and the tree is
  clean at `da1c045` or later, deployed at `4d065d1d`.
- `make test-ui-setup` is a one-time venv + browser install. If `make test-ui`
  fails with a missing Playwright, that is the fix.
- If a long-running playtest is on `dist/web`, everything that builds must use
  `GM2D_WEB` — see `HANDOFF-M12.md` §2. A rebuild moves the save fingerprint and
  ends the other run. Two `make play` runs cannot share a port either; kill the
  first.
- **The riskiest milestone is M15.3**, and not because the arithmetic is hard.
  `XP_TO_NEXT` is `[i32; MAX_LEVEL]` and exponential growth past fifty overflows
  an `i32` fast — `20 · 1.35^49` is already 2.3 billion. Overflow checks are on
  in `[profile.test]`, so it will fail loudly in a test and silently in a
  release build. Pick the base against the type or change the type, and say
  which in the commit.
- **The second riskiest is M15.2**, because it is a screen. Read *Screens, and
  the three times one covered another* in `CLAUDE.md` before you add one: three
  bugs, one shape, and not one of them was visible by reading the source. Give
  the menu a tier in the z-index table and do not reuse an id — `.card`, `.tabs`
  and `#kit` have each cost a day.
- **What only a browser can answer here is a negative**: that the fight screen
  *did not* open. `cargo test` can prove the settlement is identical down to the
  drop roll; it cannot prove nothing was drawn.
- `HANDOFF-M14.md` §6 leaves one decision open — the third town's name — and it
  is still open. This block does not touch it.
