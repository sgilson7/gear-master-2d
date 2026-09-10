//! Events that stand on a tile.
//!
//! The shape is upstream's `LadderEvent` — an id, a title, some paragraphs, and
//! a list of choices each carrying a requirement, an outcome and a line to show
//! when the requirement is not met. That last field is the one worth keeping
//! deliberately: a greyed-out button that does not say why is a button the
//! player argues with.
//!
//! What changed is where an event lives. Upstream's stood on a *rung*; these
//! stand on a *tile*, and the tile is named in `tiles.json` rather than here.
//! An event that wants moving is moved without touching its prose, which is the
//! separation `PLAN.md` §5 asks for.
//!
//! Every string in the shipped file is checked against `TONE.md`.

use serde::{Deserialize, Serialize};

/// What a choice needs before it can be taken.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Requirement {
    None,
    /// Fnorp in the purse.
    Gold(i32),
    /// Something that has happened, by its mark.
    ///
    /// **`answered` or `flags`, since M14.4**, which is what every other
    /// "has this happened" in the engine has always read: `place_is_there`,
    /// `PlaceDef::opens_onto` and a drain all ask `WorldState::marks`, and this
    /// asked half of it. So a choice could wait on a flag an event raises and
    /// not on a boss going down, which is the same kind of fact — and Marbulon
    /// asking what is behind her door *now* is a question about two bosses.
    ///
    /// Checked before it was widened: no flag any choice in the game requires
    /// shares a name with an event or a place, so nothing already shipped
    /// changed answer.
    Flag(String),
    /// A component held, worn or not.
    Holding(String),
    /// A **loose** component in the bag whose footprint is exactly `w` by `h`
    /// at some rotation, and which fills that box solid.
    ///
    /// **Ported from `event::Requirement`, which is the cut campaign's type.**
    /// `PLAN-M14.md` §1.1 names this as one of the locks *"a player opens by
    /// packing"* — and it, `AlignedItems` and `AssembledOfRarity` were all on
    /// the dead type, `Copy`, `&'static str`, and unreachable from a data file.
    /// `CLAUDE.md`'s own warning, arriving on schedule: **grep for it, and then
    /// check which of the two you found.**
    ///
    /// Its doc came with it and is the design: *"Handing something over has to
    /// cost you something you could have used."* Loose rather than worn for
    /// exactly that — a component on the board is doing a job, and taking one
    /// off a grid would break an item on a screen the player is not looking at,
    /// which is the rule `Character::deposit` already obeys.
    ///
    /// **Solid, not just a bounding box.** A slot cut three by two takes a
    /// three-by-two, not an L that fits inside one; the bounding box alone
    /// would let a five-cell corner piece answer a six-cell hole.
    LooseItemOfSize { w: u8, h: u8 },
    /// An **assembled** item on the board of at least this rarity, by name.
    ///
    /// Also ported, and it is the one that reads the live board rather than the
    /// bag — which is why a door wanting it is a door you answer by *packing*
    /// rather than by shopping. Rarity is an item's and not a component's:
    /// `RARE_AT` is 90 on a scale where a single component almost never clears
    /// it.
    ///
    /// A `String` because this is a file. `check` refuses one that is not a
    /// rarity, so a typo is a map that will not load rather than a door nobody
    /// can open.
    AssembledOfRarity(String),
    /// All of them, and it is the mirror of [`Outcome::All`].
    ///
    /// **A requirement is one condition and that was a limit rather than a
    /// design.** M14.4's one user is the third thing you may ask Marbulon,
    /// which is a question about *both* bottoms — and the two arms of a door
    /// that wants two things met were otherwise a flag raised by a third event
    /// that exists to raise it, which is bookkeeping wearing a scene.
    ///
    /// `describe` joins with *and*, so the plain statement before an attempt
    /// stays one sentence.
    All(Vec<Requirement>),
    /// The instrument assembled on the character's frame is this one.
    ///
    /// **Never the only way through a door.** `PLAN-M14.md` §1.2: an instrument
    /// makes a floor *short*, not *possible*, and
    /// `an_instrument_is_never_the_only_way_through` is what says so over every
    /// event in the game.
    ///
    /// It names a *kind* where a gate's `needs_survey` is a bool, and the
    /// difference is the point: a floor any instrument reads is a floor no
    /// instrument is for. `check` refuses a kind outside `rule::INSTRUMENTS`,
    /// the same guard `Rule::Survey` gets.
    Surveying(String),
}

/// What taking a choice does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Several, in order.
    All(Vec<Outcome>),
    /// Positive pays, negative charges.
    Gold(i32),
    Flag(String),
    /// A component, by canonical catalogue name.
    Give(String),
    /// Banked toward the next level. M4 spends it; M2 only records it.
    Xp(i32),
    /// Nothing happened, and the receipt says so rather than staying silent.
    Nothing,
    /// Restoratives, by supply id.
    ///
    /// **`Give` is components only and a tin is not a component** — no shape,
    /// no grid, spent rather than worn, which is the same three reasons
    /// `data/supplies.json` exists at all.
    Supply { id: String, n: u32 },
    /// Costs fatigue, in percentage points.
    ///
    /// **Positive only.** Fatigue is the only currency the road has, and
    /// without a cost an event is a vending machine. A *negative* tire is a
    /// tin, and tins are bought.
    Tire(u32),
    /// Starts an errand, by id, and puts it in the quest log.
    ///
    /// **This is what makes a chain visible, and without it a chain is not a
    /// decision.** At The Shallows Marker the two halves paid 12 experience
    /// and 20, and both opened a different chain — so from where the player
    /// sits there was no choice at all, only a smaller number and a larger
    /// one. Nobody takes the 12.
    ///
    /// An errand is the one thing this game already has that says *something
    /// has been opened and it is somewhere else*: it lands in the log, the log
    /// points at the map, and `quest::guide` walks a `Word` goal to the tile
    /// it wants. So a chain root hands over a chain rather than a number, and
    /// the outcomes box says so before the choice is taken.
    Errand(String),
    /// Puts you somewhere else, on a named map.
    ///
    /// **One way, and never a shortcut home.** It moves you *out*;
    /// `Rule::Homeward` is the thing that takes you back and it costs a tin.
    Warp { map: String, at: [u8; 2] },
    /// A loose component of this footprint stays where you put it.
    ///
    /// The other half of [`Requirement::LooseItemOfSize`], and it is a separate
    /// arm rather than something the requirement does on its own because **a
    /// requirement is a question and an outcome is what happened**: three of
    /// the six doors in this block want a footprint and only two of them keep
    /// it, and a requirement that always consumed would make the third
    /// impossible to write.
    ///
    /// Which one goes is the **lowest-rated** that fits, so the cost is a cost
    /// and not a mugging, and so that two players with the same bag pay the
    /// same piece. `every_giving_up_choice_asks_for_what_it_takes` refuses a
    /// choice that takes a footprint it did not ask for — otherwise a door
    /// could want a 3x2 and quietly eat a 1x4.
    GiveUp { w: u8, h: u8 },
}

impl Outcome {
    /// The concrete deltas this hands over, one line each.
    ///
    /// **Ported from `event::Outcome::describe`, which is the same function on
    /// the cut campaign's type**, and whose doc comment is this design
    /// verbatim:
    ///
    /// > Static: what this outcome *is*, for a tooltip before it is taken.
    /// > What it *did*, with the run's own numbers in it, is `Run::receipt`.
    ///
    /// A `Vec` where [`Requirement::describe`] is a `String`, and the
    /// difference is the point: **a requirement is one condition and an
    /// outcome is however many things happen.** Inherited distinction, kept.
    ///
    /// **Derived and unthemed**, TONE 13a. A spec nobody writes by hand cannot
    /// disagree with the thing it describes, so retuning an outcome retunes
    /// its box — and somebody choosing between two halves of an event is
    /// comparing numbers, which have to be translated first if they are
    /// wearing a joke.
    pub fn describe(&self) -> Vec<String> {
        match self {
            Outcome::All(list) => list.iter().flat_map(|o| o.describe()).collect(),
            Outcome::Gold(n) if *n >= 0 => vec![format!("+{n} Fnorp")],
            Outcome::Gold(n) => vec![format!("{n} Fnorp")],
            Outcome::Xp(n) => vec![format!("+{n} experience, carried")],
            Outcome::Give(name) => vec![format!("Gained: {name}")],
            Outcome::Supply { id, n } => {
                vec![format!("{n} × {}", id.replace('-', " "))]
            }
            Outcome::Tire(pct) => vec![format!("{pct}% more tired")],
            // **The one outcome that does not name its own delta**, and
            // `PLAN-M12-EXEC.md` §8 row 9 is why: naming the destination turns
            // a weird event into a fast-travel menu. What it must still be
            // honest about is the cost, which is the walk.
            Outcome::Warp { .. } => {
                vec!["You are put somewhere else. It is a long walk back.".into()]
            }
            // **The line that tells a player they have started something.**
            // It names the errand rather than the chain's length or its prize,
            // because the log is where those belong and this is the box: what
            // it must convey is *this is not the end of it*.
            Outcome::Errand(id) => {
                // **The errand's own name, which is the one exception in this
                // function to TONE 13a.** Everything else here is the engine's
                // words with a number in it, because somebody comparing two
                // halves of an event is comparing numbers. An errand's name is
                // a **proper noun**, and rule 13a's own carve-out is that a
                // proper noun is not translated — the same reason a set's name
                // is not themed. Printing the id with its hyphens taken out
                // would be neither: not the engine's words and not the book's.
                let all = crate::data::quests();
                let name = all.get(id).map(|q| q.name.clone()).unwrap_or_else(|| id.clone());
                vec![format!("Begins an errand: {name}")]
            }
            // **The one line in this box that is a bill rather than a
            // receipt**, and it has to be there for the reason every refusal in
            // this game names what it wants: a door that eats a component
            // without saying so beforehand is a door reported as a bug.
            Outcome::GiveUp { w, h } => {
                vec![format!("A loose {w} x {h} stays in it")]
            }
            // A flag is bookkeeping. It is what makes a chain possible and it
            // is not a thing a player receives, so the box does not claim it
            // is one.
            Outcome::Flag(_) => Vec::new(),
            Outcome::Nothing => vec!["Nothing you could point to".into()],
        }
    }
}

/// Every flag this outcome raises, however deep it is nested.
///
/// Public because `no_flag_is_waited_on_forever` asks it of every choice in the
/// game and a second walker would be a second answer to *what does this set* —
/// the sixth hand-written list this project would have paid for.
pub fn flags_raised(o: &Outcome, out: &mut Vec<String>) {
    match o {
        Outcome::Flag(f) => out.push(f.clone()),
        Outcome::All(list) => {
            for x in list {
                flags_raised(x, out);
            }
        }
        _ => {}
    }
}

/// Whether this outcome raises `flag`, however deep it is nested.
///
/// `Outcome::All` holds outcomes, so a flag can sit one level down from the
/// choice — which is where every chain in the game puts it, beside the errand
/// it hands over.
fn sets_flag(o: &Outcome, flag: &str) -> bool {
    match o {
        Outcome::Flag(f) => f == flag,
        Outcome::All(list) => list.iter().any(|x| sets_flag(x, flag)),
        _ => false,
    }
}

/// Where a flag is raised, as a sentence a refusal can end with.
///
/// **The same lookup [`Requirement::wants`] does, for ground rather than for a
/// choice.** That one turned *"Requires: corked the frame"* into *"Requires:
/// corked the frame — THE STANDING FRAME"*, on the argument that with only the
/// flavour a refusal is a wall and the statement is what makes it a target.
/// A drained tile is the same wall with more rock in front of it: the shore
/// waits on `built-the-tenth` and the tenth cairn is cut **two maps away**, so
/// the map is named as well as the event.
///
/// **Looked up, never listed.** Whichever choice raises the flag is the one
/// that opens the ground, so a chain that is re-authored cannot leave this
/// sentence pointing at the wrong place.
///
/// `None` when nothing raises it — a flag some other system owns, which is not
/// this function's business to guess about.
pub fn where_a_flag_is_raised(events: &EventsData, flag: &str) -> Option<(String, String)> {
    let e = events
        .events
        .iter()
        .find(|e| e.choices.iter().any(|c| sets_flag(&c.outcome, flag)))?;
    Some((e.id.clone(), e.title.clone()))
}

impl Requirement {
    /// What this asks for, in a plain sentence.
    ///
    /// **The second half of the port, and the more useful half.** The cut
    /// campaign's version carries the reason, and it is a distinction the live
    /// type has never had:
    ///
    /// > Not the same thing as `Choice::unmet`, and both are needed. `unmet`
    /// > is flavour written for the moment after you have tried; this is the
    /// > plain statement *before* an attempt.
    ///
    /// So a locked choice gets two lines doing different jobs: **what it
    /// wants**, derived and unthemed, before you try — and **what it says when
    /// you try**, which is the author's and in voice. Until now it had only
    /// the second, which is why a refusal read as a wall rather than as a
    /// target to come back to.
    pub fn describe(&self) -> String {
        match self {
            Requirement::None => String::new(),
            Requirement::Gold(n) => format!("Requires: {n} Fnorp"),
            Requirement::Flag(what) => format!("Requires: {}", what.replace('-', " ")),
            Requirement::All(list) => {
                let each: Vec<String> = list
                    .iter()
                    .map(|r| r.describe().trim_start_matches("Requires: ").to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if each.is_empty() { String::new() } else { format!("Requires: {}", each.join(" and ")) }
            }
            Requirement::Holding(name) => format!("Requires: {name}"),
            // **The shape, and both ways round.** A slot does not care which
            // way up a rectangle goes in, and a line that named one orientation
            // would send somebody looking for a piece they are already
            // carrying sideways.
            Requirement::LooseItemOfSize { w, h } => {
                format!("Requires: a loose component {w} x {h}, either way up")
            }
            Requirement::AssembledOfRarity(r) => {
                format!("Requires: an assembled item, {r} or better")
            }
            Requirement::Surveying(kind) => format!("Requires: reading it with a {kind}"),
        }
    }

    /// Refuse a requirement the engine could never answer.
    ///
    /// **The same guard `Rule::check` is, and it runs where the events are
    /// read.** A `Surveying("compasss")` parses perfectly and is a door nobody
    /// can ever open, which is indistinguishable from a design decision from
    /// every seat in the house.
    pub fn check(&self) -> Result<(), String> {
        match self {
            Requirement::None
            | Requirement::Gold(_)
            | Requirement::Flag(_)
            | Requirement::Holding(_) => Ok(()),
            // **Empty means nothing is asked**, which is `None` wearing a list
            // — and a requirement that is secretly `None` is a door somebody
            // will think is shut.
            Requirement::All(list) => {
                if list.is_empty() {
                    return Err("asks for all of nothing".into());
                }
                list.iter().try_for_each(|r| r.check())
            }
            // Zero of anything is a hole with no sides. One by one is a
            // legitimate ask — the catalogue is full of rings.
            Requirement::LooseItemOfSize { w, h } => (*w > 0 && *h > 0)
                .then_some(())
                .ok_or_else(|| format!("{w} by {h} is not a shape")),
            Requirement::AssembledOfRarity(r) => crate::rating::Rarity::by_name(r)
                .map(|_| ())
                .ok_or_else(|| format!("there is no rarity called {r:?}")),
            Requirement::Surveying(kind) => crate::rule::INSTRUMENTS
                .contains(&kind.as_str())
                .then_some(())
                .ok_or_else(|| format!("there is no instrument called {kind:?}")),
        }
    }

    /// The same statement, and **where the thing it wants comes from**.
    ///
    /// A gold requirement names a number you can go and earn and a held
    /// component names a thing you can go and get. A *flag* names a fact about
    /// something you did, and until now the line said only what the fact was
    /// called — *"Requires: corked the frame"* — which is a wall rather than a
    /// target. Reported from play by somebody standing at the wall the errand
    /// had sent them to, holding the cork the errand said to bring, being told
    /// they had not corked a frame, with nothing anywhere saying where a frame
    /// might be.
    ///
    /// **The source is looked up, never listed.** Whichever choice sets the
    /// flag is the one that opens this door, so the events are asked and the
    /// answer cannot go stale when a chain is re-authored. Sixteen events in
    /// the game have exactly one choice and it is gated on a flag; every one
    /// of them says where to go now.
    ///
    /// Takes the events rather than reaching for `data::events()`, because
    /// this is called while that data is being read and a lazy static that
    /// asks for itself is a deadlock.
    pub fn wants(&self, events: &EventsData) -> String {
        let plain = self.describe();
        let Requirement::Flag(what) = self else { return plain };
        let from = events.events.iter().find(|e| {
            e.choices.iter().any(|c| sets_flag(&c.outcome, what))
        });
        match from {
            Some(e) => format!("{plain} — {}", e.title),
            None => plain,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    /// One line under the label: what it costs, or what you are in for.
    pub blurb: String,
    #[serde(default = "no_requirement")]
    pub requires: Requirement,
    pub outcome: Outcome,
    /// Shown instead of the choice when the requirement is not met, so a
    /// refused button always says why. Empty only where `requires` is `None`.
    #[serde(default)]
    pub unmet: String,
}

fn no_requirement() -> Requirement {
    Requirement::None
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileEvent {
    /// Stable id. `tiles.json` places it; this file never says where it is.
    pub id: String,
    pub title: String,
    pub prose: Vec<String>,
    /// What you may do about it. **May be empty**, and an empty one is a
    /// different kind of thing — see [`TileEvent::is_examinable`].
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// This one is never spent, so it can be answered again.
    ///
    /// **The third kind of event**, after the card that is answered once and
    /// the note that is only read. `Game::answer_event` refuses a second choice
    /// on an event already in `answered` — *a card could sell one thing once* —
    /// and that is right for every event this game had before M14, because
    /// every one of them was a decision.
    ///
    /// A **puzzle** is not a decision, it is a sequence: the chair at the
    /// bottom of the Silt Stair is one object in one room that wants three
    /// moves in an order, and three moves is three answers.
    ///
    /// **What it must not do is pay.** An event that can be answered for ever
    /// and hands over gold, a component, experience, a tin or an errand is a
    /// faucet, and `a_repeating_event_never_pays` refuses one at load. What a
    /// repeating event may do is raise a flag, cost you fatigue, or nothing —
    /// which is exactly what a puzzle is made of.
    #[serde(default)]
    pub repeats: bool,
}

impl TileEvent {
    /// Something to read that does not ask you anything.
    ///
    /// **M11.2's, and it is a category rather than a degenerate case.** An
    /// event with choices is a *card*: it is answered once, `answer` writes its
    /// id into `answered`, and the choices are spent for good. An event with
    /// none is a thing standing in a field — a post, a pond, a wall somebody
    /// built out of rind — and there is nothing to spend, so it is never
    /// answered and it reads the same on the ninth crossing as on the first.
    ///
    /// The engine refused one of these outright until M11.2, which was right
    /// while every event was a card. The dense map is forty tiles that answer
    /// and most of them have nothing to ask.
    pub fn is_examinable(&self) -> bool {
        self.choices.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventsData {
    pub format: String,
    pub version: u32,
    pub events: Vec<TileEvent>,
}

/// Does this outcome hand the player anything, however deep it is nested?
///
/// **A flag, a tiring and a shrug are not payments**, and they are exactly what
/// a puzzle is made of. Everything else is, including experience — which is the
/// one somebody would forget, because it is a number rather than a thing.
fn pays(o: &Outcome) -> bool {
    match o {
        Outcome::All(list) => list.iter().any(pays),
        Outcome::Flag(_) | Outcome::Nothing | Outcome::Tire(_) | Outcome::GiveUp { .. } => false,
        // A negative `Gold` is a charge and would be safe; it is refused with
        // the rest because *which sign* is a thing a data edit changes and a
        // lint that reads a sign is a lint that goes quiet on a typo.
        Outcome::Gold(_)
        | Outcome::Give(_)
        | Outcome::Xp(_)
        | Outcome::Supply { .. }
        | Outcome::Errand(_)
        // A warp is a free ride, which is the one thing an event may never
        // hand out twice — the Drover's Stride costs a tin.
        | Outcome::Warp { .. } => true,
    }
}

/// The footprint this outcome keeps, if it keeps one.
fn takes(o: &Outcome) -> Option<(u8, u8)> {
    match o {
        Outcome::GiveUp { w, h } => Some((*w, *h)),
        Outcome::All(list) => list.iter().find_map(takes),
        _ => None,
    }
}

impl EventsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: EventsData = serde_json::from_str(text)
            .map_err(|e| format!("events.json will not parse: {e}"))?;
        if d.format != "gm2d-events" {
            return Err(format!("expected a gm2d-events file, got {:?}", d.format));
        }
        for e in &d.events {
            // An event with no choices is an examinable and is allowed; one
            // with no *prose* is nothing at all, and that is still refused —
            // whichever kind it is, the whole of it is what it says.
            if e.prose.is_empty() {
                return Err(format!("{:?} has no prose", e.id));
            }
            // **A note is never a puzzle.** An event with nothing to answer is
            // never spent either, so `repeats` on one is a field claiming
            // something the engine already does — and a field that means
            // nothing is a field somebody will one day rely on.
            if e.repeats && e.choices.is_empty() {
                return Err(format!(
                    "{:?} repeats and asks nothing, which is what an examinable already is",
                    e.id
                ));
            }
            for c in &e.choices {
                if c.requires != Requirement::None && c.unmet.is_empty() {
                    return Err(format!(
                        "{:?}: the choice {:?} can be refused and does not say why",
                        e.id, c.label
                    ));
                }
                c.requires
                    .check()
                    .map_err(|why| format!("{:?}: the choice {:?} {why}", e.id, c.label))?;
                // **A repeating event may not pay.** See `TileEvent::repeats`:
                // a faucet you can stand on and press for ever is not a puzzle,
                // and the only reason to give an event this field is that it is
                // one.
                if e.repeats && pays(&c.outcome) {
                    return Err(format!(
                        "{:?}: the choice {:?} pays something and the event repeats",
                        e.id, c.label
                    ));
                }
                // **A door that takes a shape has to ask for that shape.** The
                // requirement is the sentence a player reads before they choose
                // and the outcome is what happens; a door wanting a 3x2 and
                // quietly eating a 1x4 is the two halves disagreeing, which is
                // the failure this project keeps finding one milestone late.
                if let Some((w, h)) = takes(&c.outcome) {
                    if c.requires != (Requirement::LooseItemOfSize { w, h }) {
                        return Err(format!(
                            "{:?}: the choice {:?} keeps a {w} by {h} and does not ask for one",
                            e.id, c.label
                        ));
                    }
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&TileEvent> {
        self.events.iter().find(|e| e.id == id)
    }
}
