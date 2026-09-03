//! Everything one save file holds.
//!
//! `Character` is who you are and what is on your frames. `Game` is that plus
//! the things a *session* owns: the random stream every encounter is drawn
//! from, and which theme is doing the talking.
//!
//! It is deliberately small right now — the world arrives in M2, levels and
//! skills in M4, a class in M5 — and every one of those lands here rather than
//! on `Character`, because the question this type answers is "what has to
//! survive being written to a file and read back".
//!
//! # Adding a field
//!
//! Add it here, and `save.rs` stops compiling until you have said what happens
//! to it. That is on purpose: see `save::SaveState`. A field added to `Game`
//! and forgotten in the save is a save that loads a subtly different game, and
//! nothing about it looks wrong until a player loses a run to it.

use crate::character::Character;
use crate::fight::Encounter;
use crate::rng::Rng;
use crate::world::WorldState;

/// The whole of a session's state.
#[derive(Clone, Debug)]
pub struct Game {
    /// The one random stream. Encounter rolls, shop stock, drops — all of it,
    /// so that a seeded walk replays and a save can put the stream back
    /// exactly where it was.
    pub rng: Rng,
    /// Which theme is talking, by id. The theme's *contents* are content and
    /// live in `data/`; a save carries the id and nothing else.
    pub theme: String,
    pub character: Character,
    /// Where you are standing and what you have answered. **Not the map** —
    /// the map is `data/tiles.json` and is derived, never stored.
    pub world: WorldState,
    /// The fight the player is standing in, if any.
    ///
    /// No log and no seed: combat does not draw, so the situation is enough to
    /// reproduce the fight exactly. See `fight.rs`.
    pub encounter: Option<Encounter>,
}
// **There is no shelf here any more.** What a town sells is
// `data/shops.json` and never changes, so it is content and is derived where
// it is drawn; what a save carries is `WorldState::bought`, which is the short
// list of things already taken off a shelf. Same discipline as the map.

impl Game {
    /// A new session from a seed.
    ///
    /// The seed does double duty: it starts the random stream and it seeds the
    /// item-name hash, so the same seed names the same arrangement the same
    /// way for the life of the save.
    pub fn new(seed: u64, theme: &str) -> Self {
        let rng = Rng::new(seed);
        Game {
            rng,
            theme: theme.to_string(),
            character: Character::starting_seeded(seed),
            // Left at the default rather than reading the map: `Game` must not
            // depend on content loading, or a broken data file becomes a
            // panic in every constructor. Whoever has a `World` places the
            // player with `WorldState::at_start`.
            world: WorldState::default(),
            encounter: None,
        }
    }
}

impl Game {
    /// A creature's name in the theme that is talking.
    ///
    /// Falls through to the canonical name, like every other theme lookup, so
    /// an unthemed creature is one untranslated word rather than a crash.
    pub fn theme_name(&self, canonical: &'static str) -> String {
        crate::theme::by_id(&self.theme).monster(canonical).to_string()
    }

    /// A component's name in the theme that is talking.
    ///
    /// Takes a `&str` where `theme_name` takes a `&'static str`, because the
    /// callers have one out of a data file rather than out of `CATALOG` — so
    /// the catalogue is what turns it back into a literal, and a name the
    /// catalogue does not know comes back unchanged.
    ///
    /// A receipt naming a component in the engine's words is a receipt about a
    /// thing no other screen in the game calls that. The three lines that do it
    /// are here: an errand's tally, a boss's drop, and a set piece.
    pub fn theme_piece(&self, canonical: &str) -> String {
        match crate::piece::CATALOG.iter().find(|d| d.name == canonical) {
            Some(d) => crate::theme::by_id(&self.theme).piece(d.name).to_string(),
            None => canonical.to_string(),
        }
    }

    /// Walking into a town takes the tiredness off. Returns how much came off.
    ///
    /// **This is not a rest, and there still is not one.** Health has reset at
    /// every bell since M0, so a rest would restore something that was never
    /// spent — the note at the top of `CLAUDE.md` has said so for five
    /// milestones. What a town undoes is the one thing a fight *does* spend.
    ///
    /// It does not make the tins decoration: a tin is what you drink four
    /// tiles into the Verge with a Rust Colossus on the next square, and the
    /// decision fatigue exists to create is that one. The town is what makes
    /// the walk home worth taking rather than a formality.
    ///
    /// In core rather than in the shim, because "a town mends you" is a rule
    /// and the shim decides nothing.
    pub fn arrive_in_town(&mut self, id: &str) -> i32 {
        self.world.last_town = id.to_string();
        let took = self.character.fatigue;
        self.character.fatigue = 0;
        took
    }

    /// Ask the gear to take you home, and pay it a tin.
    ///
    /// **The block's one piece of new travel, and it is a rule.** Here rather
    /// than in the shim because every clause of it is a rule: whether you may,
    /// what it costs, where you land, and what it says when it refuses. A shim
    /// that decided any of those would be a second rulebook.
    ///
    /// Four refusals, and each names the thing that is in the way (`TONE.md`
    /// rule 12):
    ///
    /// - you are not wearing it;
    /// - you have not been to a town yet, so there is nowhere to go back to;
    /// - you are under the lake, which `PLAN-M11.md` §8 row 9 says no (*a
    ///   dungeon you can post yourself out of is not under a lake*), while the
    ///   Drambus Stack says yes — it is five entries by design and the kick
    ///   already moves you;
    /// - you have no tin, and the fare is the whole of what makes it a
    ///   decision.
    ///
    /// The tin goes **on departure**, which is the same bargain the Chonga
    /// Swing makes: the thing that is spent is spent whether or not you like
    /// where you end up.
    pub fn go_home(&mut self, difficulty: crate::combat::Difficulty) -> Result<Homeward, String> {
        if !self.character.rules().contains(&crate::rule::Rule::Homeward) {
            return Err("nothing you are wearing knows the way home".into());
        }
        let town = self.world.last_town.clone();
        if town.is_empty() {
            return Err("you have not been to a town yet, so there is nowhere to go back to"
                .into());
        }
        let here = crate::data::map_now(&self.world.map_id(), difficulty, &self.world);
        if here.no_homeward {
            return Err(
                "not from down here. Whatever it is doing, it is doing it upwards, and                  there are two hundred and six steps of rock in the way"
                    .into(),
            );
        }
        // The cheapest tin you are carrying, because a player who is spending a
        // fare pays it out of small change — and because choosing which tin to
        // burn is a decision nobody wants to make twice a session.
        let supplies = crate::data::supplies();
        let mut carried: Vec<(String, i32)> = self
            .character
            .supplies
            .iter()
            .filter(|(_, n)| *n > 0)
            .filter_map(|(id, _)| supplies.get(id).map(|d| (id.clone(), d.price)))
            .collect();
        carried.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        let Some((fare, _)) = carried.first().cloned() else {
            return Err("the gear knows the way home and it does not know it sober.                         One restorative, and you have not got one."
                .into());
        };
        let name = supplies.get(&fare).map(|d| d.name.clone()).unwrap_or(fare.clone());
        self.character.take_supply(&fare, 1);

        // Where the town is, across every map — the same walk a defeat takes.
        let mut moved = None;
        for (id, _) in crate::data::MAPS {
            let w = crate::data::map_now(id, difficulty, &self.world);
            if let Some(p) = w.places.iter().find(|p| p.id == town) {
                self.world.remember();
                self.world.map = w.id.clone();
                self.world.at = p.at;
                moved = Some(w.id.clone());
                break;
            }
        }
        if moved.is_none() {
            return Err("the town you came from is not on any map this build has".into());
        }
        let mended = self.arrive_in_town(&town);
        Ok(Homeward { town, fare: name, mended })
    }
}

/// What happened when you walked onto something that wanted a key.
///
/// **A key is spent opening its lock, and the lock stays open.** Both halves
/// are load-bearing and the second is the one that stops a soft-lock: the door
/// in the wall is the only way to the back half of the game, and a defeat in
/// the Treyway walks you home to West Bambulon. A key that were spent *and*
/// re-locked would end the run there, and there is no second key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unlocked {
    /// Nothing was ever locked here.
    Open,
    /// It was opened before now. The key is long gone and is not wanted again.
    Already,
    /// The key turned just now, and left the bag doing it.
    Spent { key: String },
    /// Still shut. What it wants is the place's own `shut` line.
    Shut,
}

impl Game {
    /// Turn the key, once.
    ///
    /// **The rule is core's and the bag question is core's with it.** Whether a
    /// gate opens used to be decided in the shim, on the grounds that a `World`
    /// does not know about bags — which is true, and is an argument for not
    /// putting it in `World`. It was never an argument for putting it in a
    /// shim: a `Game` is exactly the thing that holds both a bag and a world,
    /// and *a key is spent* is a rule the fast suite has to be able to reach.
    ///
    /// **An instrument is asked for every time and is never spent.** It is not
    /// in the bag, it is assembled on the board, and the Wextreen Reach's whole
    /// design is that what you carry changes what you read — so a survey gate
    /// is never written into `answered`. Only a key is.
    pub fn unlock(&mut self, place: &crate::world::PlaceDef) -> Unlocked {
        if place.needs_survey && self.survey_kind().is_none() {
            return Unlocked::Shut;
        }
        let Some(key) = place.needs.clone() else {
            return Unlocked::Open;
        };
        if self.world.answered.iter().any(|a| *a == place.id) {
            return Unlocked::Already;
        }
        if crate::quest::holding(self, &key) == 0 {
            return Unlocked::Shut;
        }
        // Off the character the same way an errand's tally goes over a counter.
        self.character.spend_one(&key);
        self.world.answered.push(place.id.clone());
        Unlocked::Spent { key }
    }

    /// Which instrument is assembled, if any.
    ///
    /// Derived from the rules an assembled item grants, the same as everything
    /// else about surveying — there is no field saying which one you carry.
    pub fn survey_kind(&self) -> Option<String> {
        self.character.rules().into_iter().find_map(|r| match r {
            crate::rule::Rule::Survey { kind } => Some(kind.into_owned()),
            _ => None,
        })
    }
}

/// What the way home cost and what it gave back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Homeward {
    /// Where you are now, by place id.
    pub town: String,
    /// The restorative it drank, by its themed name.
    pub fare: String,
    /// How much tiredness arriving took off, as a percentage.
    pub mended: i32,
}

impl Default for Game {
    fn default() -> Self {
        Game::new(0x5EED_1234_ABCD_0001, "td")
    }
}

/// Two games are equal when a player could not tell them apart.
///
/// Hand-written rather than derived because `Character` holds an undo stack
/// that a save deliberately does not carry, and `Loadout` holds a pointer into
/// a theme's word tables. Neither is part of the game; both would make a
/// derived `PartialEq` answer "different" about two identical positions.
///
/// This is the equality M1's round-trip property is stated in, so what it
/// counts is exactly what a save is required to preserve.
impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        let a = &self.character;
        let b = &other.character;
        self.rng.state() == other.rng.state()
            && self.theme == other.theme
            && a.gold == b.gold
            && a.grown_health == b.grown_health
            && a.owned == b.owned
            && a.loadout.name_seed == b.loadout.name_seed
            && a.loadout.locks == b.loadout.locks
            && a.loadout.assembly_pct == b.loadout.assembly_pct
            && a.loadout.slots == b.loadout.slots
            && a.registry == b.registry
            // **Who the character has become**, which this equality left out
            // until an ench needed adding to it. A level-one and a level-nine
            // character are extremely tellable apart, and the round-trip
            // property is stated in this operator — so anything a save has to
            // preserve and this did not compare was being asserted one field
            // at a time in one test and nowhere else.
            && a.xp == b.xp
            && a.carried == b.carried
            && a.fatigue == b.fatigue
            && a.supplies == b.supplies
            && a.skill_points == b.skill_points
            && a.skills_taken == b.skills_taken
            && a.class == b.class
            && a.enchs_owned == b.enchs_owned
            && a.enchanted == b.enchanted
            && self.world == other.world
            && self.encounter == other.encounter
    }
}

impl Eq for Game {}
