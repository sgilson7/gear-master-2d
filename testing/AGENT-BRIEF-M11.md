# AGENT-BRIEF-M11.md — everything a player could know

You are playing Gear Master 2D. Play it the way somebody who bought it would:
read what the game says, decide, press keys, and write down what happened.

**You did not build this and you must not look at how it works.** The two
prohibitions below are the point of the exercise, and they are not negotiable.

---

## 1. The two prohibitions

1. **Do not read the source, the data or the tests.** Not `crates/`, not
   `data/`, not `testing/drive.py` or `testing/playthrough.py`, not
   `PLAN-M11.md`, not `CLAUDE.md`, not the transcripts, and not this repo's git
   log. The only file in the repository you may read is this one. If you find
   yourself wanting to know a number, that is the finding — write down that you
   wanted it and could not get it, and carry on.
2. **Play the build that is served, not the code.** `testing/agent_driver.py`
   is the only tool. Everything you learn comes off the screen.

The reason is floodline's and it holds here: a run that reads `balance.rs`
measures your reading of `balance.rs` and not the game.

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

This is the part you are here to play. Everything below is a thing a player
could be told; none of it is a number out of the source.

**A door in the western wall.** Once you have been to the bottom of the Great
Gear Cave and taken what is down there, a door appears in the wall on the west
side of West Bambulon, and the key from the Cave opens it. Behind it is the
Treyway, which is a country rather than a field: sixteen tiles across, where
West Bambulon is one of them.

**A road west, and a town at the end of it.** Kettleworks. It is a metalworking
town and it sells plate, gauntlets and edges. The field it stands in is thick
with things to read.

**The Drambus Stack.** Two hundred and ten feet of cheese standing in the middle
of that field. There is a door in the south face. Every time you clear a floor
the tower drops a level and you are put outside, and the next time you go in it
is a different floor, because there is one fewer. Five floors. **A floor is one
sitting** — there is no walking out of one and a save taken inside reopens
outside.

**The lake.** When the Stack has come all the way down, the lake in the middle
of West Bambulon empties, and there is a way down in the middle of it. There is
something at the bottom, and a door behind it.

**And the early way.** The Bog Toad in the pit drops pieces of a set. Assembled
whole, that set lets you walk on water — all of it, not just the edge — which
means you can walk out to the middle of the lake and go down before the tower
falls. It is harder that way.

**Map shards, and three instruments.** Floors of the Stack leave map shards
behind, and so does the thing under the lake. Three recipes build on the weapon
board:

- a **compass**: one map shard, one glass lens, one magnet;
- an **atlas**: two map shards, one glass lens, one cosmic orb, one cosmic
  alignment;
- a **survey golem**: three map shards, two living earth.

The lens, the magnet and the living earth come off things that live in the
Stack's shadow. **A weapon grid holds gear or an instrument and never both** —
surveying costs your sword arm.

**The Wextreen Reach.** At the north edge of the Treyway the plain stops. With
an instrument assembled you can go in; without one there is nothing to read it
with. It is the same map every time you go, and what changes is the instrument.

---

## 5. What the run is for

Play from a new game to the end of that. Roughly:

1. Get out of the pit, take a class, do the errands, get Marbulon's key.
2. Do the Cave. Take the door in the wall.
3. Cross the Treyway. Find Kettleworks.
4. Drop the Stack.
5. Drain the lake and go down.
6. Build at least one instrument and survey the reach.

**You do not have to finish.** A run that stops is a finding, and the reason it
stopped is the headline. Budget two sittings; save between them.

---

## 6. What to write down

`PLAYTEST-M11.md`, in your own words, in the order things happened. Four
headings and nothing else is required:

- **What was fun.** Say what and say when.
- **What was invisible.** Anything you had to work out that the game should have
  told you, or a number you wanted and could not get, or a thing you did that
  you could not tell had worked.
- **What was wrong.** Anything that behaved as though it were broken, with the
  moment it happened and what you had done just before.
- **What you never found.** Content you know exists from this brief and never
  reached, and why you think you did not.

Quote the game where you can. A finding with the sentence the game actually said
next to it is worth three without.

**Do not fix anything.** You are not editing this repository except to write
`PLAYTEST-M11.md`.
