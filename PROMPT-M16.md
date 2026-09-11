# The prompt

Copy everything between the rules into a fresh Claude Code session, in the
repository root, with `PLAN-M16.md` and `testing/saves/the-run-20260910.json`
already placed there.

---

You are picking up an existing game project to execute its next block of
work. Everything you need is written down; nothing has been started.

**Read `HANDOFF.md` first, entire**, then `CLAUDE.md` from "Rules" to the end
of "Commands", and then these sections of it: *Eleven maps, and where they
live* (it is twenty-one now; the table is current), *The Wextreen Reach*,
*Surveyable maps*, and *What a creature leaves behind*. Then read
`HANDOFF-M14.md` §2 entire — it is the record of a plan written against the
wrong `Requirement` type and three blind counts that were guesses, and this
block is the same shape of work. Then read `PLAN-M16.md`, which is the plan
you are executing.

The block is one ask from the human, in their own words:

> here's an almost finished run that is very powerful; use this run to produce
> a new dungeon spec. add a dungeon in the wextreen sands. the wextreen sands
> should be surveyable i.e needs the mapping to be done in order to open the
> area, and then has a dungeon inside with 3 floors, more complicated puzzles
> on each floor than the existing ones, and a new enemy as a boss with my
> current run's loadout as its gear slots.

and a second, folded in:

> also come up with two new base class and corresponding expert classes
> derived from it. the class should be based on consuming pool stats to gain
> mana empowerment, and the second class should be based around doing enough
> mind damage to kill any enemy.

The Sands is already surveyable — `the-edge-of-the-sands` has `needs_survey`
— and the plan keeps that and adds a second survey gate under it. The run is
`testing/saves/the-run-20260910.json`, level 45, and it is both the block's
yardstick and its boss.

Execute in the order `PLAN-M16.md` §6 gives: **M16.0 through M16.6.** The
dungeon is M16.0–M16.3, the two base classes are M16.4, the eleven experts are
M16.5, and the gate is M16.6. Two deploy points, after M16.3 and after M16.5.
The classes come after the dungeon on purpose: a dungeon the human can play is
worth more than a fork with two more cards on it, and `PLAN-M16.md` §13 says
so.

Work milestone by milestone. For each one:

1. **Do the recon the milestone asks for before writing content**, and put
   what you found in the commit message. Four things in this plan are guesses
   the recon replaces with counts: the boss's health and strength (by bracket
   against the save, never by adding to the last boss's numbers); her three
   drops (catalogue only, at least one `EVENT_ONLY`); which pieces move on
   the two M14 bosses to bring them under the curse cap; and every blind
   ceiling — 9, 3, 8 — which were counted on paper and are asserted with
   `==`, so if a floor measures differently **the floor is wrong, not the
   number**, unless you can say in the commit why the paper count was.
2. **Load the save before you draw anything.** `common::from_save` is the
   first thing M16.0 builds, and the first test is that the save opens at
   forty-five with thirty-eight seated pieces and six enchs. Every fixture in
   this block that says *the run* means that character, not `geared_from`.
3. Build it in core wherever it is a rule. `crates/wasm` decides nothing;
   the shim draws a `q` cell the way it draws a `t` cell and asks core about
   everything else.
4. Write the tests the plan's acceptance column names, **break each one and
   watch it fail before you keep it**, then restore. `no_creature_is_past_the
   _curse_cap` must be red on two creatures before anything is re-dressed —
   commit it red, then commit the re-dressing, so the history shows what it
   found.
5. Run `make test`, then `make web` and `make test-ui` (three engines), then
   `GM2D_FROM=testing/saves/the-run-20260910.json make play` and **read the
   transcript**. From M16.1 on it has to show the walker solving a floor, and
   the fatigue it arrives at each stair with goes in the commit beside the
   plan's number for that stair.
6. Commit in the house style — read `git log -10` first. Say *why*, name what
   was rejected, and say out loud when something cost a day.
7. **Stop at the deploy point and ask.** You do not `git push` or
   `make publish` on your own judgement. Say what `git log origin/main..HEAD`
   would send — and note that M15's seven commits are already waiting there
   undeployed, so the answer is not only this block's.

Seven standing constraints for this block:

- **No new components.** Every drop, every door's price and every piece on
  the boss is in the catalogue. §8 says zero and the recon keeps it there.
- **Every puzzle is monotone.** `quick` drains to `silt` and never the other
  way; every sinkhole `repeats`; no flag on any floor makes a stair
  unreachable. `puzzle::solvable_blind` is the proof and it runs on all three.
- **The compass never sets a flag.** It is offered on every floor because the
  Sands' prose says it lies here, and a lie that pays would be a hint. `the_
  compass_sets_nothing_on_any_floor` is the test.
- **She wears the run.** `the_tenth_surveyor_wears_the_run` compares her
  `outfit()` to the save's `reports()` by item name and count per slot. If a
  placement does not seat, the gear block is wrong, not the board; find out
  why before moving a piece.
- **Every player-visible sentence goes through `log()`, with `TONE.md`
  open.** Nine stakes with three choices each is twenty-seven sentences that
  have to be identical in kind and different in fact.
- **A refusal names the thing in the way.** The way under says it needs the
  sheet or an instrument, by name; a door says which rarity; the plate says
  which levers are still up.
- **An expert tree moves its own promise and nothing else.** M13 §1.6 is
  still the rule and `expert_nodes_touch_only_the_expert` runs over all
  twenty-one. The eleven new experts grant exactly three new rules between
  them — `burn_keeps_bonus`, `burn_carries`, `mind_pierce` — and each is
  granted by more than one tree; a fourth is a smell.

`PLAN-M16.md` §9 lists six decisions that are the human's. Where the work
cannot start without an answer, take the plan's recommendation, say in the
commit that you took it, and flag it in the handoff. The boss's name is the
one to take as written; the re-dressing of the M14 bosses (§9.4) is the one
to do as its own commit so it can be reverted alone.

If you find something the plan got wrong — and you will, because the floors
were drawn before anybody walked them — say so, propose the change, and
record it as a divergence with its reason the way `HANDOFF-M14.md` §2 does.
In particular: the Needle Room's four sinkholes use `Outcome::Warp` as a
puzzle piece for the first time, and `Warp` was written for a shortcut, not
a lock. If it cannot land you in a walled alcove, that is the first thing to
report, and the plan's fallback is a `needs` gate on each alcove keyed to
the sinkhole's flag.

One thing to check before M16.4 and to report if it is not so: the plan
assumes `pools_worth_holding` returns rage, faith and nature and that a
`Combatant` can be asked for its largest with one call. If the pool set is
different, the Stoker burns whatever that function returns, and the plan's
sentence about *"the three pools"* is the thing to correct, not the class.

Start with M16.0. It ships one fixture, one terrain, one field on
`MonsterSpec`, one lint that is meant to fail — and nothing a player can see.

---

## Notes for whoever hands this over

- The tree is at `b3fdb4f` with M15 committed and **not deployed**. This
  block's deploy point sends M15 too; say so when asking.
- The save is the human's real run. Do not edit it, do not re-save it through
  the game, and do not commit a version the game has touched — the fixture's
  whole value is that it is the file the human uploaded. If the catalogue
  fingerprint ever moves, `from_save` should refuse it loudly rather than
  patch it.
- `make test-ui-setup` is the one-time venv + browser install.
- The riskiest milestone is **M16.3**, for two reasons. `MonsterSpec.enchs`
  is the first time a creature carries an ench and it has to read through
  `loadout_at` *after* the piece seats and *before* the lock, or the profile
  numbers will not match the save's own item. And the DPS bracket is a claim
  about a fight between two real boards; if the run beats her 95% of the time
  at Medium, the numbers move up, and if 20%, down — the bracket is the
  test, and the numbers are whatever makes it pass.
- **M16.4 is the second riskiest** for a screen reason, not an engine one:
  the fork has drawn five cards since M5 and its layout, its Escape refusal
  and its browser check all assume five. Seven is two rows. Look at
  `check_the_fork_refuses_escape` before touching the card grid.
- The two M14 bosses at 114 and 120 curse resist is a real finding from
  reading the data, not a plan invention. It is fixed here because the lint
  that catches it is one this block writes anyway.
