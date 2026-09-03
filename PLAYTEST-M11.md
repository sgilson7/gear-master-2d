# PLAYTEST-M11.md

Played from a fresh game (the browser crashed once mid-session and lost an
un-saved run entirely — see below — so this is really the second attempt,
carried to level 12, 1500+ Fnorp, class Gorillathon). Two sittings' worth of
budget. Reached the Great Gear Cave, crossed into the Treyway, and ran out of
runway partway across it. Everything below is off the screen; nothing here
comes from the source.

## What was fun

**The narrative event cards are the best writing I hit.** THE COUNTED HEAP —
someone's stacked 11,200 gear-teeth and keeps correcting a sign on the pile
downward — let me pick "Take four," which paid out with the line "The number
on the board becomes wrong by four, which is what the crossings-out are," and
+25 Fnorp landed immediately. THE CORK BOUNDARY had a man cutting a doorway
through a four-foot cork fence with a bread knife, "four strokes down, four
across, and a pause to let the cut close a little before he widens it again" —
"Ninth one," he says, without looking up. "They close." Choosing to help him
cut the tenth paid off the errand in one motion. Best of all was **Marbulon's
Door**: an old woman on a kitchen chair facing away from a shut door with no
house behind it, who has been counting how many people walk past without
asking. "Forty-one people have walked past me on their way to the Cave this
year, and Henpeck has a gate on it, and I have the key, and forty-one of them
did not ask." Her two errands (kill three Whisperlings to find out if there
are three left; go stand in front of the gate and confirm it's still
standing) paid out Marbulon's Key and a piece of glass, and every line of her
dialogue earned it.

**The level-5 class fork was a real moment.** Five options (Gorillathon,
Funnel Sergeant, Worm-Fact Keeper, Kaklon Licensee, Top of the Bill), each
with its own one-line mechanical promise, no undo. I took Gorillathon for the
lifesteal and never looked back — the screen was clearly signalling this was
permanent.

**Crossing from level 1 to level 5 and having the game visibly tell you no.**
Walking into the first crossing at level 1 produced: *"A Sprocketman with a
barrow has stopped where the slag ends. 'Everything past here is water and
what lives in it,' he says. 'You are the fourth this month. The other three
went anyway.' It wants level 5, and you are 1."* The second crossing, later,
worded the refusal completely differently — a plank nailed across the road
with "a list of eleven names, and a twelfth with nothing written after it. It
wants level 9, and you are 5." Two gates, two different pieces of writing, and
both told me the exact number I needed. That's a good way to gate content.

**The numbers get big and it feels earned.** Watching health climb from 100 to
530 and Fnorp from 28 to 1500+ over the course of the run, purely from
Auto-pack plus buying every component a shelf had, felt like real progress
rather than a treadmill.

**Finding the Kaklon van exactly at level 10.** The brief mentions an ench
vendor who "is not there until level 10," and I hit the van on the Verge road
the very next time I passed that spot after banking into level 10. It told me
flatly: "He does not ask what you are. Bolting one onto a component is the
Kaklon Patent's, and you are not a licensee — what you buy here goes in the
rack and waits." Exactly the rule the brief described, delivered in character.

## What was invisible

**A defeat that resets everything is invisible until it happens to you.** My
browser crashed partway through the first attempt (a Playwright/Chrome
`Browser.setDownloadBehavior` error I couldn't do anything about), and because
I hadn't downloaded a save recently, an entire session — level 3, ~110 Fnorp,
two toad-eye errand progress, several errands taken — was gone the instant I
reconnected, back to a brand-new level-1 character. The brief's "save early"
warning is not decorative. I have no complaint about the game here — this is
exactly what it warned me about — but it's worth recording as a real cost a
player could hit with no warning on screen at the moment it happens.

**Whether I am winning or losing a fight is not written down anywhere until
the fight is over.** `history` never logs a fight's outcome — no "beat the
A. Rat" or "lost to Frosty Kev" line ever appears in the running log, only in
the transient result screen. I have to catch the screen at the right moment or
infer it from whether Fnorp/carried-xp changed. Combined with the fight
screen's two buttons — "Fight" and "Walk away" — sitting at the same visual
position screen-to-screen but *not* always at the same selector I expected, I
spent my first six "fights" accidentally walking away every single time
without realizing it, because I assumed the second button matched the
"Done"/proceed button from the packing screen. Nothing on screen told me I'd
walked away instead of fought until I finally checked `buttons` out of
suspicion and saw "Fight" / "Walk away" spelled out. That was my error, not
the game lying to me, but the game also never confirmed *which* action I'd
taken — no "You turn back" line, nothing. It just silently returned me to the
map.

**A card's own stated reward does not always show up.** Three separate times,
choosing a dialogue option in a narrative card printed a specific promise —
"+14 toward the next level," "+16 toward the next level," "+60 toward the
next level" — and in every one of the three cases the *carrying* field on the
standing panel was unchanged immediately afterward (verified against a known
baseline each time). Compare that with combat wins, which reliably print
"+3 experience, carried" and then show the exact new total in the panel a
line below. The Fnorp side of the same choices (e.g. "-200 Fnorp" for the
cave toll) landed exactly as printed every time. I never found a case where a
card's *quoted* xp reward actually landed. I could not get a number for how
much xp these cards are actually supposed to grant, if any, because the
panel disagreed with the sentence I'd just read.

**Region danger and encounter chance are permanently "you could not say," and
I never found the node that changes that.** Every single panel check, all
run, all region, printed `you could not say` for both fields. The brief
implies a knowable number exists somewhere (a Scout-style rule), but I never
found a shelf, errand, or skill that offered it in the roughly 60 skill
points I ended up spending, and by level 12 I'd taken every top-row base-tree
node without seeing the option appear.

**My own power level relative to a monster is not a number I can compare to
anything.** The fight screen shows "IT RATES" for the enemy and "YOUR BOARD"
for me, and I assumed these were the same unit for a long time — they are
not. "YOUR BOARD" tracks assembled-item count (it went 1 → 3 → 4 → 6 as I
bought gear), not a power score, so there is no way to eyeball "will I win
this" from the numbers on the pre-fight screen. I only learned this by
watching the actual health bars during a live replay.

**The Great Gear Cave's boss was never clearly distinguished from a regular
fight.** The cave is a small dead-end room, and the one tile at the far end
that I could never step past kept giving me the exact same named creature —
"Velothi High Guard," rated 161 — every time I approached, including several
times in a row with no other encounter mixed in. On one of those repeats it
dropped an item named "The Key to the Deep Chocolate." Nothing on that fight
screen said BOSS, said this was special, or confirmed the cave was "done."
I still don't know for certain whether that was the cave's intended boss
fight, a strong stationary guard with a rare drop, or something else — the
screen gave me no signal either way, and the header just said "AN ENCOUNTER"
like every other fight. I only inferred I'd made progress because I later
found myself able to reach an entirely new map.

**"West Bambulon" never opened as a place I could stand in and use.** I
crossed the region labeled West Bambulon repeatedly (the toll gate, the
Kaklon van, the way into the Cave), but never once got a town screen, a shop,
or any sign of the lake the brief describes as being in the middle of it. I
don't know if I walked past the actual town tile without stepping on it, or
whether it simply isn't reachable from the path I was taking.

## What was wrong

**Buying a supply item and handing in an errand item with the same name can
click the wrong thing.** In the pit town, `THE ONES STILL DOWN THERE` asks for
4 × Long Shift Tin, and the shelf sells a restorative also called Long Shift
Tin. Clicking the shop's "Long Shift Tin" button by its visible text landed
on the *errand card's* hand-in action instead, four times in a row, and
printed **"0 of 4, and nobody writes down what they are not handed"** as a
rejection each time — no Fnorp spent, no tin bought. I only got the actual
purchase to register by clicking on the button's price text ("11 Fnorp")
instead of its name. A player using a mouse rather than automation would
likely never hit this, since the two buttons look different on screen, but
the fact that the game itself produced a clean, in-character rejection
message for a click that landed on the wrong control (rather than, say,
silently doing nothing) suggests the two controls really do share a name
somewhere underneath.

**A card's stated experience reward not landing** (see "What was invisible"
above) is the one I'd call an actual bug rather than a thing I misunderstood:
the same mechanism (a plain "+N experience, carried" line) works flawlessly
for every combat win, and the cards use different wording ("+N toward the
next level") for what reads like the identical mechanic, and it just doesn't
show up in `carrying` afterward. Three for three.

**Losing a fight anywhere on the Treyway walks you *the entire way home* to
the original pit town**, not to the nearest place you've been. After crossing
the Great Gear Cave and reaching the region called The Wextreen Heights, a
single lost fight (to something called "Warden of Sneel," rated 903 against
my board of 7 — wildly out of my league) sent me all the way back to tile
(1, 18) in The End of All Gears, several screens and two map transitions
away. I don't know if this is intended ("the walk after a defeat is a
placement" per the game's own framing) or a gap in how far the "nearest known
town" search reaches once you're two maps deep, but it made losing in the
Treyway cost several minutes of pure walking on top of the fatigue and
carried-xp loss, for no in-fiction reason I could find (nothing in West
Bambulon or the Cave ever presented itself as a town I could rest at).

## What you never found

**Kettleworks.** I crossed into the Treyway (confirmed by two new region
names, "The Kolok Downs" and "The Wextreen Heights," and terrain — everything
was suddenly ringed by open water where the pit map never had any) and spent
a long stretch pushing through both, but never found a town, a road
distinctly marked as going toward one, or any shop. Every route further into
the interior ran into monsters far past what I could survive — "Warden of
Sneel" rated 903 and "Warden of the Centrifuge" rated 686 against a board
that rated 7 — and losing to them reset my position all the way to the start.
I suspect I was heading in the wrong direction (north, toward what the brief
calls the Wextreen Reach, which explicitly wants an instrument I never
built) rather than the "road west" the brief says leads to Kettleworks, but I
ran out of budget before I could test that theory properly.

**The Drambus Stack, the lake in West Bambulon, and the way down under it.**
Never seen. I don't know if I was simply in the wrong part of the map or
whether West Bambulon's own tile (which I never located as a stand-alone
town — see above) is the gateway to it.

**Any of the three instruments (compass, atlas, survey golem), and the
Wextreen Reach itself.** Never got close. Never picked up a map shard, a
glass lens, a magnet, a cosmic orb, cosmic alignment, or living earth —
nothing on that ingredient list ever turned up in a shop, an errand reward, or
a drop in my time in the Treyway.

**The Bog Toad's water-walking set, used deliberately.** I killed plenty of
Bengulon Jungle Toads for the pit errand and never noticed a set piece drop
or a "you can wade now" message; I never tried to walk out into the lake at
the pit either, so I can't say whether I simply didn't get the drop or didn't
think to test it.

**Two of the pit town's own errands** — `THE FRAME THAT STANDS` and
`A DOOR WHERE THERE WAS NOT ONE` — stayed marked "something else first" for
the entire run and I never worked out what that something was; the second
one explicitly wants whoever is "carrying the Cave's key," and I never went
back to check whether obtaining Marbulon's Key or the cave drop unlocked it.

**Drinking a restorative.** I bought four Long Shift Tins and spent all four
on an errand hand-in before ever testing the "drink it from the standing
panel" flow the brief describes. Fatigue only ever went away by walking into
a town.
