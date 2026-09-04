# TRIAGE-M12.md — everything the block's own sweep found

Written at M12.4, before anybody outside has been asked to play it. Same format
as `TRIAGE-M11.md`: every line has a **severity**, a **cost** and a
**disposition**, and blockers are fixed here while the rest is carried
*openly* — listed in `testing/AGENT-BRIEF-M12.md`'s known section, so the
playtest spends its attention on the unknown.

**Severity**, about the player rather than the code: **blocks** — cannot get
past it, or gets past it and the game is wrong afterwards. **wrong but
survivable** — not what was meant, and you can play on. **cosmetic** — reads
badly, costs nothing.

**Cost** is content, a number, a function, or a design decision that is the
human's.

---

## The measure, closed

`testing/transcripts/m12.0.txt` against `testing/transcripts/m12.4.txt`, same
probe, same walker, at matched levels.

    level   baseline   closing      what changed
        3       35%       30%       the shelf costs 5x, so a beginner buys less
        5       43%       47%
        6       41%       48%
        7       39%       51%
        8       37%       54%
        9       40%       54%
       10       48%       54%

    bench:  0 until level 15   ->   1 from level 8
    events paying gear: 0      ->   nonzero from level 7

**The headline is not the level of the curve, it is its slope.** The baseline
*fell* as you levelled — 43% at five down to 37% at eight — because rows
arrived on a clock and components did not. It rises now: 47% at five up to 54%
at eight. That is M12.3's whole argument, and it was a hypothesis in the frame
and a measurement in `PLAN-M12-EXEC.md` §10 before it was a change.

**The targets are still missed.** 30% at level three against 70%, and nothing
reaches 80%. The block moved pressure in the right direction and did not reach
the number written down for it, and row 8 below is why.

---

## Fixed in the block

| # | severity | what | cost | disposition |
|---|---|---|---|---|
| 1 | **blocks** | **A player's save could not be played.** After clearing a floor of the Drambus Stack the character stopped moving; every reload bought one action and then nothing. `quest::guide` asked all eleven maps whether a crossing stood between the player and an errand, handing each of them a position that belongs to one map — at (4, 16) on a 20×20 field, the 16×16 Treyway was asked for index 260 of 256 and the wasm trapped. | a function | **Fixed** in M12.B, three layers deep, and their own save is a test fixture. Reproduced against the deployed page first, so what was fixed is what they have. |
| 2 | **wrong but survivable** | **`idx` wrapped instead of failing.** `y * width + x` with no bounds check: past the bottom it trapped, past the right-hand edge it returned a real tile *from the next row* and said nothing. The silent half is the worse half and had been there since there were two maps. | a function | **Fixed.** Every grid accessor answers *nothing is there* off the map. |
| 3 | **wrong but survivable** | **The shim decided the event rules.** `event_json` and `answer` each carried their own copy of the requirement match, and outcomes were applied in the shim — a rule the fast suite could not reach. | a function | **Fixed** in M12.5: `Game::can_take` and `Game::answer_event`. |
| 4 | **wrong but survivable** | **`Guide::shut` had no test at all.** M9.4's "the log says when a road is shut" could have been deleted outright and 638 tests would have stayed green; the `shut` this suite checked elsewhere is a different field on a different type. | a test | **Fixed.** Pinned, and the pin fails when the guide stops asking. |
| 5 | **wrong but survivable** | **Seventeen flags were set by an event and read by nothing.** A chain has been possible in the data format since M2 and had never been used once. | content | **Fixed** in M12.5. All seventeen are read, three chains run end to end, and a lint refuses the next orphan. |
| 6 | **wrong but survivable** | **The greaves grid was 0% for fourteen levels** of a whole playthrough — a fifth of the canvas, never used. Not a content shortage: the pit sells two greaves-capable pieces and six of nine sets drop greaves. | content | **Fixed** by the barrel, which stocks a greaves material and a mould. It is nonzero from level five now. |
| 7 | **cosmetic** | **The page's opening blurb had gone stale** — "every level adds a row to one frame" is what M12.3 retires. | prose | **Fixed.** Removed, which is what paid for the recipe box. |

## Carried, openly

| # | severity | what | cost | disposition |
|---|---|---|---|---|
| 8 | **wrong but survivable** | **The barrel fills the bag and not the board, and it is the block's biggest miss.** 182 barrel components bought in the closing run and the final reading has **203 owned components that fit nowhere**. Auto-pack refuses any placement that does not strictly improve `(items assembled, what they rate)`, so cheap filler is bought and then declined. It is why the fill target is missed. | a design decision, and the human's | **Carried as `PLAN-M12-EXEC.md` §8 row 13.** `PLAN.md` already says the button must "leave nothing obvious in the bag" and it is leaving two hundred, so a final pass that seats what fits is arguably that rule honoured rather than the button becoming an optimiser — but it raises the floor of player power across the whole early game, which is not mine to decide. **Nothing else in this block can reach the fill number without it.** |
| 9 | **wrong but survivable** | **The loss rate is up by a fifth.** Baseline 447 wins / 236 losses (34.5% lost); closing 831 / 607 (42.2%). The likeliest cause is M12.1a: the shelf costs five times what it did, so a character meets the same regions with less bought gear. It is a consequence of a change that was asked for, and it may be exactly right — a shop you have to save for is a shop with stakes — but it was not measured before and it is measured now. | a number | **Carried.** The knob is `SHELF_PCT`, and the honest test is a person playing it rather than a walker that buys everything it can afford. |
| 10 | **wrong but survivable** | **`common::geared_from` never pays for anything.** It gives a character every shelf's stock free, so M12.1a's price rise moved no reachability test at all. That was convenient this block and it means the fixture is now further from what a player can actually afford than it has ever been — the gap between *reachable content* and *affordable content* is new, and the fixture only models the first. | a test fixture | **Carried.** Worth a second fixture that shops with a purse before the next content block leans on this one. |
| 11 | **wrong but survivable** | **The fill target is missed and the curve numbers may be wrong rather than the game.** 70% by level three was written before any of it was measured. The closing run reaches 30% there and 54% by eight, and the slope is right. Whether 70/80 was ever the correct number is a question the block never asked. | a design decision, and the human's | **Carried.** `pressure::target` is one edit; what it should say is a judgement about how tense the game should feel, which is the human's and wants a person playing it. |
| 12 | **cosmetic** | **`character.rs`'s `STARTER` constant and its `seat` method are dead code.** The starting kit is *given* into the bag and the board starts empty; Auto-pack is what seats the blade turned. `CLAUDE.md` quoted that constant's comment as live fact. | a deletion | **Carried.** Behaviour is correct; the trap is the next person who greps `STARTER` and believes the kit is seated. |
| 13 | — | **The agent spot-run has not been done, and the friend has not played it.** `testing/AGENT-BRIEF-M12.md` is written and its five errands include the one only a stranger can do — read three events and say, before choosing, what each half will give you. | two people | **Outstanding, and named as outstanding.** Neither is something the builder can do: the first needs an agent forbidden this repo's source, the second needs the friend whose sentence started the block. The block is not closed until both are in. |

---

## What the block did not do

`PLAN-M12.md` §7 held: **no reroll in any costume**, no harvesting off beaten
creatures, no world-clock restock, no new maps, creatures, sets or rules. And
`PLAN-M12-EXEC.md` §7 row 1 held: **no new components anywhere**, so the
catalogue is still 568 and **no save written before this block is refused by
it.** Every save that opened on M11 opens on M12.
