# SECOND-ORDER-M14.md — the notebook

*Written when noticed, not at the end. `SECOND-ORDER-M13.md` is the precedent
and M13.9 is what made keeping one worth it: seven rows marked as candidates,
all seven executed, three of them turning up something that was actually
wrong.*

**A row marked `M14.6 candidate` is worklist.** The last milestone of this
block reads this file and executes them.

---

## Rows

### 1. The plan is written against the dead `Requirement`, not the live one

`PLAN-M14.md` §1.1 names `LooseItemOfSize`, `AlignedItems` and
`AssembledOfRarity` as *"locks a player opens by packing"*, and §4.1, §4.2 and
§5.1 build three doors on them. **All three are on `event::Requirement`, which
is the cut campaign's type**, `Copy`, `&'static str`, and not deserialisable.
The type the game uses is `tile_event::Requirement`, and it has four arms:
`None`, `Gold`, `Flag`, `Holding`.

This is `CLAUDE.md`'s own warning arriving on schedule — *"Grep for it, and
then check which of the two you found"* — and it is the single largest thing
the plan understates. M14.0 grew by three ported arms.

**Status: done in M14.0.**

### 2. `AlignedItems` has nothing to read in GM2D

The dead type's own doc says it is *"at least this many assembled items sharing
one alignment word"* — upstream's naming system, where an item's generated name
carried a word other items could share. The fork kept `naming.rs` but nothing
exposes an alignment word on an assembled item, and `PieceKind::Alignment` is a
*component kind* the crystal ball and the atlas use, which is a different thing
wearing the same noun.

So §4.2's second choice cannot be built as written. `AssembledOfRarity` is the
plan's own third named lock, reads the same live board, and is what the shelf
door takes instead.

**Status: divergence, recorded. Done in M14.2.**

### 3. `no_flag_is_waited_on_forever` does not exist

M14.0's acceptance says *"extended and negative-tested"*. It is a doc comment
on `event::Requirement::Flag` describing a lint the campaign had; nothing in
`crates/core/tests` is called that. It is written rather than extended, over
the live events, and it is the reason a chain root that hands over a flag
nobody reads is caught.

**Status: written in M14.0.**

### 4. A tile event is answered once, and the chair needs nine

`Game::answer_event` refuses a second choice on an event already in
`answered` — *"a card could sell one thing once"*. §5.2's chair is one event
that wants nine answers.

`TileEvent::repeats` is what M14.0 adds. What it cannot fix is the other half:
**a sequence of nine over three always-offered labels is not expressible in
monotone flags.** See the divergence in M14.3.

**Status: primitive done in M14.0; the chair diverged in M14.3.**

### 5. `Outcome::Counter` would be `Outcome::Xp` planted on purpose

§3 asks for a `Counter` arm on `the-low-water-mark`, *"nothing reads it this
block; it is the watcher pattern, planted."* `Outcome::Xp` wrote into
`world.counters["xp"]` for four blocks and nothing read it, and nine events
printed a receipt for experience that never existed. `CLAUDE.md` has the rule
in as many words: **a number that is shown needs somewhere it is read.**

Planting one deliberately is that bug with a note beside it.

**Status: cut. Divergence recorded in M14.1.**

### 6. The instrument set is already completable, and only off a roll

Counted rather than assumed (§9 decision 5). The golem is three `Map Shard` and
two `Living Earth`. Map Shards are certainties — five off the Drambus Stack and
one off Sootmother. `Living Earth` has exactly one source in the game: **Bone
Cantor at 300 per mille**, a roll you have to win twice.

So the plan's claim is true and thin. One `Living Earth` off each of the two new
bottoms turns the golem from two rolls into a certainty, which is the version of
"a player who did both dungeons can build the golem" that is worth writing down.

**Status: answered in M14.2 and M14.3.**

### 7. Rating does not predict whether the board wins

Measured against `common::geared_from`, which is this repository's answer to
*the board a player actually has*:

| it beats | it loses to |
|---|---|
| Sootmother 1670, Anvilheart 1803, The Salt Wedding 1897, The Last Light 2031, Francis 2958 | Cairn Chorus 1141, The Tallow Saint 1223, Weeping Idol 1240, The Last Gearwright 1353, The Ground Floor 1507, Gilt 2489, Nine of Ashes 2519 |

It beats a 2958 and loses to a 1141. What decides it is **damage per second**,
not rating: Cairn Chorus kills it in 13.3 seconds and Sootmother leaves it on
1297 of 1942 after forty-three. `PLAN-M14.md` §4.4's instruction — *by DPS
bracket, never by adding to Sootmother's* — is right, and this is the
measurement behind it.

**Status: the bracket the two bosses were dressed against.**

### 8. The walker reaches level 14, not 22

`CLAUDE.md`'s current transcript is *342 wins, 170 losses, level 14, 4,406
steps*. M14.2's acceptance names `the_ninth_surveyor_is_beatable_by_the_walker_
at_22`, and a level-22 board is not a board this game produces.

`common::geared_from` is what M11.7 established as the thing to measure
against, and it is what the test measures against — under a name that says what
it measures.

**Status: divergence, recorded in M14.2.**
