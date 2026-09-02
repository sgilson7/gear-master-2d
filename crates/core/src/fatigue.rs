//! What a fight costs you beyond the fight.
//!
//! Health resets at every bell — that has been true since M0 and is why there
//! was never anything for a rest to restore. **Fatigue is the thing a fight
//! actually spends.** Every battle takes a share of your *maximum* health for
//! good, so an expedition is a budget rather than a corridor: the fourth fight
//! in a row is fought by a weaker character than the first, and knowing when to
//! turn round is the decision the map was missing.
//!
//! Two things give it back and they are not the same thing. **A town takes all
//! of it off on arrival** — see `Game::arrive_in_town` — which is what makes
//! the walk home worth taking rather than a formality. **A restorative takes
//! some of it off wherever you are standing**, which is the decision this
//! exists to create: another fight, open the tin, or turn round.
//!
//! It is a percentage rather than a number of points because it has to mean
//! the same thing at level one and level twenty. Twelve points is a third of a
//! starting character and a rounding error later on; twelve percent is twelve
//! percent.

use serde::{Deserialize, Serialize};

/// What one battle takes, in percent of maximum health.
///
/// **Set against the pit, not by taste.** Four is enough that a fifth fight is
/// a decision and not enough that a bad first fight ends the trip:
/// `tests/fatigue.rs::a_full_expedition_is_a_budget_and_not_a_wall` walks the
/// starting character out and refuses a number that makes the second fight
/// unwinnable or the tenth free.
pub const PER_FIGHT: i32 = 4;

/// The most a body will carry. Past this a character is not tired, they are
/// finished — and a game that lets your maximum reach zero is a game with an
/// unloseable-and-unwinnable state in it.
pub const CAP: i32 = 60;

/// What `pct` fatigue does to a maximum.
///
/// Rounds towards the player: a character is never reduced below one point of
/// health by tiredness alone.
pub fn worn(max_health: i32, pct: i32) -> i32 {
    let pct = pct.clamp(0, CAP);
    ((max_health as i64 * (100 - pct) as i64) / 100).max(1) as i32
}

pub const FORMAT: &str = "gm2d-supplies";
pub const VERSION: u32 = 1;

/// Something you carry and drink.
///
/// Not a component. It has no shape, it goes on no grid, and it is spent
/// rather than worn — three good reasons not to force it into `PieceDef`,
/// where every one of those would have had to be a special case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supply {
    pub id: String,
    pub name: String,
    /// One line, in the world's words.
    pub blurb: String,
    /// Percentage points of fatigue it takes off.
    pub restores: i32,
    pub price: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppliesData {
    pub format: String,
    pub version: u32,
    pub supplies: Vec<Supply>,
}

impl SuppliesData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: SuppliesData = serde_json::from_str(text)
            .map_err(|e| format!("supplies.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these supplies are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        for s in &d.supplies {
            if s.restores <= 0 {
                return Err(format!("{}: a restorative that restores nothing", s.id));
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&Supply> {
        self.supplies.iter().find(|s| s.id == id)
    }
}
