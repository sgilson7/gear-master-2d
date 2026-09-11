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

/// What running away costs, in percent of maximum health.
///
/// **Twenty, the human's number**, and five times a fight's four: fleeing is
/// meant to be the expensive way out of a fight you do not want, not the cheap
/// one. It goes through `tire_hard`, so it does not stop at [`CAP`].
pub const RUNNING_AWAY: i32 = 20;

/// What walking off a cairn you cannot use costs.
///
/// **Ten, the human's number.** The Cairnfield's blind solution is forty-five
/// card reads and thirty-six of them are a cairn refusing you, so a floor that
/// charged nothing for a wrong guess was a floor where guessing was free —
/// which is what *"make the cairn room more dangerous"* means. The ninth
/// clipboard and the survey golem are what it is worth avoiding.
pub const WALKING_OFF: i32 = 10;

/// The most **a fight** will leave on a body.
///
/// Sixty, and it has been sixty since M8: past this a character is not tired,
/// they are finished, and a game that lets your maximum reach zero is a game
/// with an unloseable-and-unwinnable state in it. Every ordinary source of
/// wear — the four percent a battle takes — stops here.
pub const CAP: i32 = 60;

/// The most **anything** will leave on a body.
///
/// **A second cap, and the difference between the two is the whole of what a
/// penalty is.** Wear stops at [`CAP`] because a fight is a budget; the things
/// a player does to *dodge* a fight are not wear and do not stop there — the
/// twenty percent for running away and the ten for walking off a cairn push
/// past it, up to here.
///
/// **Ninety-nine and not a hundred**, and the one is load-bearing: `worn`
/// floors a maximum at one point of health, so a hundred would be a character
/// who is alive by a rounding rule rather than by a number. At ninety-nine you
/// have a hundredth of yourself and you are going home; at a hundred you are a
/// division nobody wrote down.
///
/// **It is never a dead end.** Walking costs nothing, no map in the game is
/// without a way up, and a town takes all of it off — so a character pinned
/// here can always walk out. That is the difference between a penalty and a
/// soft-lock, and it is why this number is allowed to be brutal.
pub const HARD_CAP: i32 = 99;

/// What `pct` fatigue does to a maximum.
///
/// Rounds towards the player: a character is never reduced below one point of
/// health by tiredness alone.
///
/// **Clamped at [`HARD_CAP`] rather than at [`CAP`]**, because the second cap
/// would have been decoration otherwise: a character at ninety-nine percent
/// tired who was worn as though they were at sixty is a character the penalty
/// never reached.
pub fn worn(max_health: i32, pct: i32) -> i32 {
    let pct = pct.clamp(0, HARD_CAP);
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
    ///
    /// Read only by [`SupplyDoes::Restore`]. The other two kinds leave it at
    /// zero, and `SuppliesData::parse` refuses a tin that restores nothing and
    /// a charm that restores something — a field that means nothing on two of
    /// three kinds is a field somebody will one day fill in.
    #[serde(default)]
    pub restores: i32,
    pub price: i32,
    /// What drinking it does.
    ///
    /// **Defaulted, so the three tins in `supplies.json` did not have to move.**
    /// `Restore` is what a supply has always been; the other two are the items
    /// the human asked for — *an item you can use to allow you to run away from
    /// the next battle for free*, and *an item that lets you warp back to town
    /// instantly*.
    #[serde(default)]
    pub does: SupplyDoes,
}

/// The three things a thing in your pack can be.
///
/// **An enum rather than three optional fields**, so a supply is exactly one of
/// them and `use_supply` matches exhaustively — a fourth kind is a compile
/// error at every reader rather than a silence at one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SupplyDoes {
    /// Takes tiredness off, wherever you are standing. A tin.
    #[default]
    Restore,
    /// **Carried rather than drunk.** Holding one is what makes the next
    /// running-away free, and fleeing spends it — so the item *is* the state
    /// and there is no new field on the character, and therefore no seam.
    Flight,
    /// Puts you in your last town, now.
    Home,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppliesData {
    pub format: String,
    pub version: u32,
    pub supplies: Vec<Supply>,
}

impl Supply {
    /// What it says it does, unthemed and with the number in it. TONE 13a.
    ///
    /// **Derived from the kind rather than typed into the blurb**, so retuning
    /// what a tin restores retunes the line that promises it — the same
    /// arrangement `Node::line` has with its effect, and the reason a blurb
    /// that overstates is the worst kind.
    pub fn line(&self) -> String {
        match self.does {
            SupplyDoes::Restore => format!("Takes {}% of the tiredness off.", self.restores),
            SupplyDoes::Flight => {
                "Running from the next fight costs you nothing. Spent when you run.".to_string()
            }
            SupplyDoes::Home => "Puts you in the last town you stood in.".to_string(),
        }
    }
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
            // **`restores` means something on exactly one kind**, and a field
            // that means nothing on the other two is a field somebody will one
            // day fill in and expect to work.
            match s.does {
                SupplyDoes::Restore if s.restores <= 0 => {
                    return Err(format!("{}: a restorative that restores nothing", s.id));
                }
                SupplyDoes::Restore => {}
                _ if s.restores != 0 => {
                    return Err(format!(
                        "{}: a {:?} that also restores {}, and only a tin restores",
                        s.id, s.does, s.restores
                    ));
                }
                _ => {}
            }
            if s.price <= 0 {
                return Err(format!("{}: free, and everything in a pack is bought", s.id));
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&Supply> {
        self.supplies.iter().find(|s| s.id == id)
    }
}
