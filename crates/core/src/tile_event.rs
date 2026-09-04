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
    /// A flag set by an earlier event.
    Flag(String),
    /// A component held, worn or not.
    Holding(String),
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
            // A flag is bookkeeping. It is what makes a chain possible and it
            // is not a thing a player receives, so the box does not claim it
            // is one.
            Outcome::Flag(_) => Vec::new(),
            Outcome::Nothing => vec!["Nothing you could point to".into()],
        }
    }
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
            Requirement::Holding(name) => format!("Requires: {name}"),
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
            for c in &e.choices {
                if c.requires != Requirement::None && c.unmet.is_empty() {
                    return Err(format!(
                        "{:?}: the choice {:?} can be refused and does not say why",
                        e.id, c.label
                    ));
                }
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&TileEvent> {
        self.events.iter().find(|e| e.id == id)
    }
}
