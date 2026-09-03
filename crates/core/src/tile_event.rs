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
