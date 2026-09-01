# TONE.md

Every string a player reads is checked against this file. Events, creature
notes, town lines, NPCs, class blurbs, item flavour, error messages, the
glossary. If a rule here and a good sentence disagree, the sentence is wrong or
the rule is; both are worth arguing about, and neither gets skipped quietly.

## Where this comes from

Three sources, in this order of authority.

1. **`../TurtleRichard/gear-master-td-retheme.md` §2, the content charter.**
   Binding, not advisory. It was written for a publicly hosted build and GM2D
   is one. Reproduced as rules 1 and 2 below.
2. **The shipped `td` theme** (`crates/core/src/theme.rs`, `TURTLE_DICK`). This
   is what the register sounds like in production rather than in a proposal.
   Most of the rules below are extracted from it, and every example marked
   *(shipped)* is a real string from it.
3. **The engine's own event prose.** Deadpan, concrete, English-countryside
   ominous. The house style GM2D inherits and does not abandon.

## The register, in one line

**The frame is deadpan and the contents are absurd.**

The book is broad comedy — a gorilla figure-skating while deadlifting, a moon
dropped by a sandwich, a god made by accident out of a filing error. The
engine's prose is dry and exact — *"THREE LINES CROSS SOMEWHERE. THE CROSSING
IS NOT MARKED. MARKING IT WAS NEVER THE POINT."* GM2D is the second voice
narrating the first. The prose never winks; the facts inside it are ridiculous
and are reported flat, at length, with the numbers included.

Anything that winks is off-register, whichever source it came from.

---

## The rules

Each one is answerable yes or no about a specific sentence. That is the bar: if
a reviewer cannot point at a clause and rule on it, it is not a rule and does
not belong here.

### 1. Nothing on the excluded list appears, in any form

No sexual or anatomical humour. No drugs, alcohol or smoking — semuta stays
*music*; Corquewine, B-quila, shermsticks and Skunkweed do not exist. No
bathroom humour; the Seeker is **the Weeping Seeker** and his other name is
never written. No slur-adjacent coinages. No real public figures.

*Check:* would this line need a content warning if it were read aloud to a
classroom? Then it is out.

### 2. Violence is Saturday-morning grade

The Crimper crunches. Nobody is described being crunched.

*Check:* does any sentence describe damage to a body? Rewrite toward the
machine, the noise, or the aftermath.

### 3. Characters count things, and report the count

> "You try them twice more, which is twice more than you need to." *(shipped)*

> "Ninth one," he says. "They close."

Counting is the house tic. It is how a scene shows somebody has been here a
while without saying so.

*Check:* does at least one line in a scene contain a number that a person in it
would actually have kept?

### 4. Scale is a number, never an intensifier

1.79 trillion residents. 7,583 HP. The 45th annual race. 603 flawless
frame-jobs. Not "countless", not "untold", not "innumerable".

*Check:* search the line for a quantity word with no digits in it. If one is
carrying weight, replace it with the digits.

### 5. The narrator never explains a joke

Henpeck's last word on the subject is *"I am not a **retailer**"* and the scene
ends. No one reacts to it, glosses it, or tells you it was funny.

*Check:* does any sentence exist to make sure the reader got the previous one?
Delete it.

### 6. A reversal lands flat, in the shortest sentence available

> "Then a gambler in a coat made of money fell through the roof of it."
> *(shipped)*

The largest thing in a scene gets the smallest sentence. Build-up is done with
plain declaratives; the turn is not announced.

*Check:* is the most surprising sentence in the scene also one of the longest?
Cut it down.

### 7. No adjective a monster could not itself use

The vocabulary belongs to the world, not to a narrator standing outside it.
"Corked", "sneelclad", "unmovable", "boring" — all fine, all things a thing in
the world would say. "Eldritch", "unspeakable", "otherworldly" — no: they are a
narrator telling you how to feel.

*Check:* could the creature being described have said this word about itself?

### 8. Every proper noun is sourced

To the book, the 161-row title CSV, or an existing GM2D name. A reviewer can
ask for the page and get one.

*Check:* `M2`'s lint fails the build on an unsourced proper noun. Until the
open question in `PLAN.md` §6.4 is settled, invention is out.

### 9. Common nouns carry a scene; proper nouns are spent sparingly

The shipped theme says this about itself:

> Every one of these scenes is written in common nouns — a ditch, a barn, a
> milestone, a fence — which `vocabulary` swaps in place, so there is nothing
> here for a paragraph to rescue. *(shipped)*

A scene built out of a ditch and a fence survives being retold. A scene built
out of four invented names is a scene only its author can read.

*Check:* count the proper nouns in a scene. More than two and it wants a
reason.

### 10. Dialogue is unattributed where it can be

> "Oh," he says, with some effort, and a little delight. *(shipped)*

Speech tags carry an action or nothing. No adverbs of manner on "says" —
"grimly", "wryly", "sarcastically" are rule 5 wearing a different hat.

*Check:* does any speech tag tell the reader the tone instead of showing it?

### 11. A choice's blurb says the cost, not the flavour

> "It comes away like bark and starts back while you are holding it."

The label is what you do. The blurb is what it costs you or what you are in
for. A blurb that only sets a mood is a blurb a player learns to skip.

*Check:* after reading the blurb, does the player know something they did not
know from the label?

### 12. A refusal says why, in one sentence, in the world's words

> "Forty Fnorp, and you have not got it."

Not "requirement not met". Not an apology. The `unmet` field on a choice and
every error string the player can reach are prose and are checked here.

*Check:* does the message name the thing that is missing?

### 13. The economy speaks the book's language

Gold is **Fnorp**. Mana is **the Funny**. Armour is **Cork**. Mind damage is
**Idiot Mode**. Mind resist is **Thick Skull**. Curse of Searing is **the
Roast**, of Frost is **Nut Freeze**, of Stun is **Semuta Trance**, of Misfire is
**Goof Gone Wrong**. Rage is **Fury**, Faith is **Devotion**, Nature is
**Harvest**.

These are theme lookups, not rewrites: the engine still says "mana" everywhere,
because everything it decides depends on that word meaning one thing. Content
authors write the themed word; code never does.

*Check:* does any string in `data/` use a canonical stat name where a themed one
exists?

### 13a. A spec is not prose, and does not get themed

The exception to 13, and the only one. A **mechanical description** — what a
skill node does, what a class power does, what an item contributes — goes out
in the engine's own words with the number in it: *"+12% mind resist"*, not
*"+12% Thick Skull"*. Two registers, kept apart on purpose:

| | written by | speaks | example |
|---|---|---|---|
| **name / blurb** | a person, in `data/` | the book | *Corked* — "A strip of it, wedged where a blow lands." |
| **spec** | derived in code from the effect | the engine | `start every fight with 12 armor` |

The reason is not consistency, it is arithmetic. Somebody choosing between two
nodes is comparing numbers, and a number wearing a joke has to be translated
before it can be compared. It is also the only defence against a description
that is *wrong*: a spec nobody writes by hand is a spec that cannot disagree
with the effect, and the tree shipped eight nodes whose blurbs described armour
and mana they did not grant.

*Check:* is the mechanical half derived from the effect rather than typed? And
does it contain a themed word? Both are lints — `tests/skills_read.rs`.

### 14. The title gag is handled once and then left alone

The book's best story does it as a boy and his turtle. The loading line may say
*"it's about a turtle, we promise."* Nothing else in the game touches it.

*Check:* is this the second place the joke appears? Then it is one too many.

### 15. A rule here beats a good sentence, and a good sentence beats a habit

If a line breaks a rule and is clearly right anyway, change the rule in this
file in the same commit, with the sentence as the reason. What is not allowed is
leaving both and hoping.

---

## Working notes

**The names are already generated.** `naming.rs` builds
`[Qualifier] [Base] of the [Suffix]` and the corpora are the book's. Item names
are not authored and are not checked against this file — the corpora are, once,
and after that the generator is trusted. Rarity is audible: three words common,
four rare, five epic, six legendary.

**The theme cannot break the game.** A missing entry falls through to the
canonical name, so a half-finished pass is a game with some untranslated words
rather than a game that will not start. This is why tone iteration is safe and
why it is data.

**Where the strings live.** `data/*.json` and `data/theme.td.json`. If you are
editing a `.rs` file to change what a player reads, stop — that is the wrong
file, and `PLAN.md` §2 says so.
