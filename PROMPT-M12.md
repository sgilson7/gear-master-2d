# The prompt

Copy everything between the rules into a fresh session, in the repository root.

---

You are picking up an existing, deployed game project to execute its next
block of work. Everything you need is written down; nothing has been started.

**Read `HANDOFF-M12.md` first, entire.** It tells you what to read next and in
what order, where the code for each milestone is, and which rules will bite
you. Then read `PLAN-M12-EXEC.md`, which is the plan you are executing.
`PLAN-M12.md` is the earlier frame and the exec plan wins where they disagree.

Execute the block in the order `PLAN-M12-EXEC.md` §3 gives: **M12.0, M12.1,
M12.2, M12.5, M12.3, M12.4.** M12.5 is third on purpose — the argument is in
§2 and §7 row 3, and reordering it undoes the reason M12.3's scope can be
decided on evidence.

Work milestone by milestone. For each one:

1. Do the recon the milestone asks for **before** writing code, and put what
   you found in the commit message. Several milestones exist because recon
   changed the answer.
2. Build it, in core wherever it is a rule. `crates/wasm` decides nothing.
3. Write the tests the plan's acceptance section names, **break each one and
   watch it fail before you keep it**, then restore. This project has shipped
   three vacuous checks and the most recent shipped vacuous twice in one
   sitting.
4. Run `make test`, then `make web` and `make test-ui` (three engines), then
   `make play` and actually read the transcript.
5. Commit with a message in the house style — read the last ten with
   `git log -10` first. They explain *why*, name what was rejected and why,
   and say out loud when something cost a day.
6. **Stop at each deploy point and ask.** You do not `git push` or
   `make publish` on your own judgement, ever, even when the work is green and
   obviously wanted. Say what `git log origin/main..HEAD` would send.

Four standing constraints for this block, all in `HANDOFF-M12.md` §3:
**no new components** (the catalogue must not move — the recon proving you
don't need any is already done), **no reroll in any costume**, **every
player-visible sentence goes through `log()`**, and **never write a game
string without `TONE.md` open**.

`PLAN-M12-EXEC.md` §8 lists twelve decisions that are the human's. Where a row
records a recommendation and the work cannot start without an answer, take the
recommendation, say in the commit that you took it, and flag it. Where it does
not, ask.

If you find something the plan got wrong — and you will, because plans are
written before recon — say so, propose the change, and record it as a
divergence with its reason the way `CLAUDE.md`'s divergence table does. A plan
that survives contact unchanged usually means nobody checked.

Start with M12.0. It ships nothing a player can see and does not deploy; its
whole job is to make board pressure a number before anything tries to move it.

---

## Notes for whoever hands this over

- The prompt assumes the working directory is the repo root and the tree is
  clean at `7686004` or later.
- `make test-ui-setup` is a one-time venv + browser install. If `make test-ui`
  fails with a missing Playwright, that is the fix.
- If a long-running playtest is on `dist/web`, everything that builds must use
  `GM2D_WEB` — see `HANDOFF-M12.md` §2. A rebuild moves the save fingerprint
  and ends the other run.
- The riskiest milestone is **M12.5**, not because it is hard but because an
  outcomes box is a promise printed on a screen, and this project has shipped
  four promises that reached nothing. `PLAN-M12-EXEC.md` §6 entry 7 says to
  treat any disagreement between the box, the receipt and the character's
  actual state as a fifth instance rather than a rendering bug. That is the
  sentence to hold them to.
