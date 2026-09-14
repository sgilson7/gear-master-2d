# The prompt

Copy everything between the rules into a fresh Claude Code session, in the
repository root, with `PLAN-M21.md` and `SYSTEMS-PITCH.md` already placed
there.

---

You are picking up an existing, deployed game project to execute its next
block of work, **start to finish, without asking anyone anything.** Everything
you need is written down; nothing has been started; nobody is going to answer
a question, so do not ask one. Where the plan leaves a choice open it also
gives a recommendation, and you take the recommendation, say in the commit
that you took it, and move on.

**Read `HANDOFF.md` first, entire**, then `CLAUDE.md` from "Rules" to the end
of "Commands", then these sections of it: *Brewing*, *Specializations*, *TikZ
or nothing*, *The glossary*, and *Determinism*. Then read `PLAN-M20.md` and
`SECOND-ORDER-M20.md` entire — the first is the block that built the template
this block copies three times, and the second is the notebook format you will
keep. Then read `SYSTEMS-PITCH.md`, which is the design, and `PLAN-M21.md`,
which is the plan you are executing.

The block is one ask from the human, in their own words:

> all these ideas are great actually, write out a plan and a prompt for a
> model in claude code to execute this; for any new sprites, use the latex
> sprite making prompt, and for new ench ideas, look for games as inspiration
> for new ideas. i want the plan to be written out as milestones, with
> different deliverables per milestone, and after each milestone, a summary
> table is output showing how far along the work is. the model should not
> ask for approval at any time. it should also keep a notebook of second
> order effects that it notices, and add additional milestones at the end
> to address those second order effects

Execute in the order `PLAN-M21.md` gives: **M21.0 through M21.11, then
M21.12 onward**, which you will write yourself from the notebook. Deploy
after M21.5 and after M21.11, and after the last notebook milestone if it
changed anything a player can see. You do not ask before deploying. You run
`make publish`, you report the hash, and you keep going.

Work milestone by milestone. For each one:

1. **Do the recon before writing anything**, and put what you found in the
   commit message. The things this plan guessed that you must measure: every
   clock constant (`STAGES`, `FEED_EVERY`, `CUSTOMERS_PER_BELL`) against the
   run fixture's pace in `make play`; `KENNEL_CAP` against a fight the
   fixture wins narrowly; whether the eight family silhouettes cut from the
   TikZ bounding cells actually fit any of the three run masks; and whether
   `simulate_party` as it stands takes a second combatant without a change
   to targeting.
2. **Build the rule in core and the picture in the shim.** Every bench is a
   `Shape` mask and every placement is core's decision. The shim draws what
   core returns. The one combat change in the block — the Kennel's second
   combatant — goes through `simulate_party` and nowhere else.
3. **Write the tests the acceptance lines name, break each one and watch it
   fail before keeping it**, then restore. The three `C(8,2)` tables are
   proved by one generic lint; if you find yourself writing a second copy of
   it, stop and make the first generic.
4. **Every new sprite is TikZ from `tikz_figure_prompt.md`**, and nothing
   else. Fill the prompt in with the conventions in `art/behemoth.tex` —
   `\Unit`, `\Edge`, the family's `main`/`dark`/`accent`, `ink`, primitives
   only — write the `.tex` with the header comment *Written by filling in
   tikz_figure_prompt.md*, and compile with `make art`. Twenty-five figures.
   A figure that does not compile is not done. No PNG, no SVG by hand, no
   image generator.
5. **Every new ench names the game it came from** in `ench.rs`'s doc comment
   and on its glossary shelf. The six are chosen; if one of them cannot be
   built as described, replace it with another mechanic from another named
   game and say which in the notebook — do not ship five.
6. Run `make test`, then `make web` and `make test-ui`, then `make play` from
   the run fixture and **read the transcript**. From M21.1 on it has to show a
   harvest; from M21.5 on, a creature out; from M21.8 on, a sale.
7. **Keep the notebook.** `SECOND-ORDER-M21.md`, in `SECOND-ORDER-M20.md`'s
   format — numbered rows, bold first sentence, `open` or `closed`. Write a
   row the moment you notice a second-order effect: a fixture that moved, a
   property set on a container that a sibling inherits, a lint that catches
   nothing, a constant whose value turned out to matter for a reason the plan
   did not give, a screen that grew a fourth thing beside three. Do not save
   rows up for the end. A row is a sentence about what you saw, not a plan
   to fix it.
8. **Print the status table.** After every milestone's commit, append the
   table in `PLAN-M21.md` §Status table to `MILESTONES.md` under
   `## M21 — three benches`, with every milestone's row — done, in progress,
   pending — the test count and delta, the commit hash, and the notebook's
   open/closed count on the last line. Print the same table as the last thing
   in your message for that milestone. The human reads this table and nothing
   else to know where you are.
9. Commit in the house style — `git log -10` first. Say *why*, name what was
   rejected, and say when something cost a day.

**When M21.11 is done, write the rest of the plan.** Read the notebook. Take
every `open` row. Group rows into milestones of the same shape as the plan's —
a title, a deliverables table, an acceptance line. Number them M21.12, M21.13,
… in the order that closes the most rows soonest. Append them to `PLAN-M21.md`
under *M21.12 → — Whatever the notebook says*. Then execute them exactly as
you executed the first twelve, closing rows as you go and printing the table
after each. A row that is a finding and not work is closed by writing the
finding into `CLAUDE.md`. **The block ends when the notebook has no open rows**
and the final status table's last line says `0 open`.

Seven standing constraints:

- **No new components.** Seeds, crops, kennelled creatures, buyers and
  benches are shapes in their own modules. The save fingerprint does not
  move. There is a player mid-run.
- **Build order is Plot, Kennel, Stall.** The Kennel eats what the Plot
  grows; a Kennel that ships first is hungry on the day it ships.
- **The clock is the bell.** No days, no timers, no real time. Every clock
  constant is a `pub const` with a shelf on G.
- **One slot for a specialization**, and no specialization pairs with
  anything. `spec_nodes_touch_only_their_bench` is the widened lint and it
  runs over all four.
- **A boss is never kennelled, and a kennelled creature is capped at the
  region.** `no_boss_is_kennelled` and `a_creature_out_is_capped_at_the_
  region` are in the suite before the offer is written.
- **The Stall is the only place a `CATALOG` piece is placed by shape outside
  the five grids**, and it comes from the tray and goes back to the tray.
  Nothing is created; nothing is destroyed but by a sale.
- **Every player-visible sentence goes through `log()`, with `TONE.md`
  open.** Eight buyers, eight crops, twenty-eight pair lines three times over
  — count, do not dramatise.

If you find something the plan got wrong — and the clock constants at least
will be — say so in the commit, make the change, record it in the notebook
and in `CLAUDE.md`'s divergence table with its reason, and keep going. You do
not stop to ask whether to diverge. The two most likely: the family
silhouettes may not fit any run mask, in which case the silhouettes shrink
to their family's *harvest crop* shape (which you will already have drawn)
rather than the run growing; and the Stall's buyer clock at one per bell may
flood the ledger, in which case `CUSTOMERS_PER_BELL` stays one and buyers
who want nothing on the shelf are not printed.

Start with M21.0. It ships a bag nobody can open yet and eight seeds nobody
can plant, and the first status table has one row marked done.

---

## Notes for whoever hands this over

- The tree is at `40e3121`, live. This block adds no maps, so `data::MAPS.len()`
  stays and the fingerprint stays; if either moves, something has gone into
  the catalogue that should not have.
- The builder is told not to ask. That includes deploys. If you want to see
  the block before it goes live, watch `MILESTONES.md` for the M21.5 row; the
  deploy follows it in the same session.
- The notebook milestones (M21.12+) are unbounded by design. The last status
  table tells you how many there were and what they closed. If the builder is
  still going after M21.20, the notebook is finding real things and that is
  the block working, not failing.
- `make art` needs `pdflatex` and `pdftocairo`; `packaging/build-art.sh` says
  where. Twenty-five figures at a few seconds each is a couple of minutes.
- The riskiest milestone is **M21.5**, the only combat change: a second
  combatant through `simulate_party`. The party fixtures have run with one
  player and one foe for a long time and the targeting rule *"the second when
  there are two, the same when there is one"* is a sentence, not yet code.
  Watch the transcript's first fight with a creature out.
