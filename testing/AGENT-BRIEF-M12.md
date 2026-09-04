# AGENT-BRIEF-M12.md — everything a player could know

You are playing Gear Master 2D. Play it the way somebody who bought it would:
read what the game says, decide, press keys, and write down what happened.

**You did not build this and you must not look at how it works.** The two
prohibitions below are the point of the exercise, and they are not negotiable.

---

## 1. The two prohibitions

1. **Do not read the source, the data or the tests.** Not `crates/`, not
   `data/`, not `testing/drive.py` or `testing/playthrough.py`, not any
   `PLAN-*.md`, not `CLAUDE.md`, not `HANDOFF*.md`, not the transcripts, and
   not this repo's git log. The only file in the repository you may read is
   this one. **If you find yourself wanting to know a number, that is the
   finding** — write down that you wanted it and could not get it, and carry
   on.
2. **Play the build that is served, not the code.** `testing/agent_driver.py`
   is the only tool. Everything you learn comes off the screen.

A run that reads the source measures your reading of the source and not the
game. M11.8's run earned this rule on its first sitting: it found a three-block
-old bug in a line every reader of the source had skimmed past, because the
comment beside it explained why it was right.

---

## 2. How to play

One command per turn. The browser stays open between them.

```
python testing/agent_driver.py start          # once, at the beginning
python testing/agent_driver.py look           # a screenshot; the path is printed
python testing/agent_driver.py panel          # the standing panel, as text
python testing/agent_driver.py screens        # what is open, if anything
python testing/agent_driver.py buttons        # everything clickable right now
python testing/agent_driver.py log            # the last few lines of the log
python testing/agent_driver.py history        # everything the game has said
python testing/agent_driver.py text "#shelf"  # read any part of the page
python testing/agent_driver.py key ArrowUp    # one keypress
python testing/agent_driver.py click "#go"    # one click
python testing/agent_driver.py save           # download the save; path printed
python testing/agent_driver.py load <path>    # put a save back
python testing/agent_driver.py stop           # at the end
```

Use `.venv-test/bin/python` if plain `python` has no Playwright.

`look` is a real screenshot and you should use it — the map, the board and the
fight are drawn on canvases and `text` cannot see them.

**`buttons` before you click.** It lists every clickable thing on the topmost
screen with its selector, so you never have to guess one.

---

## 3. What the game is

You walk a map one tile a keypress. Arrow keys or WASD. Something stops you and
a fight starts, and **you do not fight** — you pack five grids with components
beforehand, and the arrangement is the whole of your input. Components that
touch form *items* if they satisfy that grid's recipe; items go off on cooldowns
and the fight runs itself.

- **Fnorp** is money. **Cork** is armour. **The Funny** is mana.
- A win pays experience into your pocket. **A town is the only place that turns
  it into a level**, and a defeat takes everything you are carrying and nothing
  you have spent.
- **Every fight costs 4% of your maximum health for good**, won or lost. A town
  takes all of it off when you walk in. A tin takes some off wherever you are.
- At level five you choose what you are, once, and it does not come off.
- **Save early.** `save` downloads a file and `load` puts it back. It is also
  the pause button, and the run is expected to be two sittings.

---

## 4. What is new, as a shop poster would print it

**A barrel under the counter.** Every town has one. It is the same barrel
everywhere, it never runs out, and what is in it is cheap and not very good. It
is where a beginner fills a frame.

**The shelf costs more than it used to.** What a town has on its shelf is the
good stuff and it is priced like it. One shelf line or a frame full of barrel
junk is roughly the decision you can afford on your first afternoon.

**Made to order.** A town will make you one thing at a time. You pay up front,
it takes a number of fights to arrive, and it waits at the counter that took
the order until you come back for it. It is the dearest way to get anything and
it is the only way to choose exactly what.

**A grid tells you what it takes.** The packing screen says what each of the
five frames needs to make an item out of. It says it for the empty ones too.

**Events pay, and say what they pay before you choose.** A card with choices on
it now prints what each half will give you, underneath the description. A
choice you cannot take says what would open it as well as what it would pay.
Some of them hand over gear. Some of them cost you. **Some of them are the
start of something that finishes somewhere else on the map.**

**A row is earned, not given.** Levelling used to add a row to one of your
frames every time. It does not any more: a level pays a skill point, and a row
is one of the things you can spend a point on at the tree — or the reward for
finishing a line of errands. Growing the board is now a choice you make instead
of power.

---

## 5. The five errands

Do these in order where you can. Write down what happened, in your own words,
including anything that surprised you or that you had to guess at.

1. **Fill a frame out of the barrel by level two.** Start a new game, get to a
   town, and see how much of a board you can cover for what you can afford.
   Say whether it felt like a bargain or like litter.
2. **Place a commission and collect it.** Order something, fight your way to
   it, and go back for it. Say whether you knew how long it would take, and
   whether it was worth what it cost.
3. **Earn a row.** Get a level, find where a row comes from, and buy one. Say
   whether the game told you where to look, or whether you had to hunt.
4. **Read what a grid takes, and use it.** Find a frame you have nothing for,
   read what it wants, and go and get it. Say whether what it told you was
   enough.
5. **The one only you can do: read three events and say, before you choose,
   what each half will give you.** Then choose, and say whether you were right.

   Do this for three separate cards. Write down, for each half of each choice:
   what you expected from reading it, and what actually happened. **An
   outcomes box that the person who wrote it can read is not a box that
   works** — you are the only instrument that can tell us whether it does.
   If a box and a receipt ever disagree, that is the most important thing you
   will find all run; write down both, word for word.

---

## 6. Known already, so spend your attention elsewhere

These are on the list and do not need reporting again. Anything *else* is worth
writing down, however small.

- **The barrel fills your bag faster than it fills your board.** The Pack Your
  Frames button only seats a component if it makes the board rate better, so
  cheap filler often stays in the bag. If you want a frame covered you may have
  to place pieces by hand.
- **There is no way to sell anything.** Money only goes one way.
- **A whole playthrough is long.** Two sittings is expected. Save often.
- The demo ends at a door under the lake, and what is past that door is not
  written. The ending screen says so.

---

## 7. What to write

A file of your own, in your own words. What you did, in order; what the game
said; where you were confused; what you expected and did not get. Numbers where
you have them and a note where you wanted one and could not get it.

**Do not soften anything.** The point of you is that you have not read the
source and cannot be talked out of what the screen actually said.
