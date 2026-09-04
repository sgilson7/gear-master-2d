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
    // ------------------------------------------------------------ events

    /// Whether this choice can be taken.
    ///
    /// **One answer, and it used to be two.** The shim asked this question in
    /// `event_json`, to grey a button, and again in `answer`, to refuse a
    /// click — two copies of the same match in the layer that decides nothing.
    /// A `Game` is exactly the thing that holds a purse, a bag and a world, so
    /// it is the thing that can answer.
    pub fn can_take(&self, c: &crate::tile_event::Choice) -> bool {
        use crate::tile_event::Requirement;
        match &c.requires {
            Requirement::None => true,
            Requirement::Gold(n) => self.character.gold >= *n,
            Requirement::Flag(f) => self.world.flags.iter().any(|x| x == f),
            Requirement::Holding(name) => self.character.holds(name),
        }
    }

    /// Take choice `n` of an event, and say what it paid.
    ///
    /// **Core's, and M12.5 is why.** Applying an outcome was the shim's while
    /// every outcome was a number; `Warp` is a rule about where a player may
    /// stand, and a rule the fast suite cannot reach in seconds is a rule with
    /// two rulebooks.
    pub fn answer_event(
        &mut self,
        id: &str,
        n: usize,
        difficulty: crate::combat::Difficulty,
    ) -> Result<Vec<String>, String> {
        let events = crate::data::events();
        let Some(e) = events.get(id) else { return Err("no such event".into()) };
        if self.world.answered.iter().any(|a| a == id) {
            return Err("already answered".into());
        }
        let Some(c) = e.choices.get(n) else { return Err("no such choice".into()) };
        if !self.can_take(c) {
            return Err(c.unmet.clone());
        }
        let outcome = c.outcome.clone();
        let mut receipt = Vec::new();
        self.apply_outcome(&outcome, &mut receipt, difficulty);
        self.world.answered.push(id.to_string());
        Ok(receipt)
    }

    /// [`apply_outcome`](Self::apply_outcome), reachable from a test.
    ///
    /// The private one is private because taking a choice goes through
    /// `answer_event`, which is where the *rules* about taking one live —
    /// spent-ness, requirements, writing the id down. A test that wants to
    /// prove one outcome kind pays should not have to author an event to do
    /// it, so this exists and says why rather than the field being loosened.
    #[doc(hidden)]
    pub fn apply_outcome_for_test(
        &mut self,
        o: &crate::tile_event::Outcome,
        receipt: &mut Vec<String>,
        difficulty: crate::combat::Difficulty,
    ) {
        self.apply_outcome(o, receipt, difficulty)
    }

    fn apply_outcome(
        &mut self,
        o: &crate::tile_event::Outcome,
        receipt: &mut Vec<String>,
        difficulty: crate::combat::Difficulty,
    ) {
        use crate::tile_event::Outcome;
        match o {
            Outcome::All(list) => {
                for i in list {
                    self.apply_outcome(i, receipt, difficulty);
                }
            }
            Outcome::Gold(n) => {
                self.character.gold = (self.character.gold + n).max(0);
                receipt.push(if *n >= 0 { format!("+{n} Fnorp") } else { format!("{n} Fnorp") });
            }
            Outcome::Flag(f) => {
                if !self.world.flags.iter().any(|x| x == f) {
                    self.world.flags.push(f.clone());
                }
            }
            Outcome::Give(name) => match self.character.give(name) {
                Some(_) => receipt.push(format!("Gained: {}", self.theme_piece(name))),
                None => receipt.push(format!("{name} is not in the catalogue")),
            },
            Outcome::Xp(n) => {
                // **Carried, the same as a win pays.** Nothing on the road
                // spends; a town is the only thing that turns carried into a
                // level.
                self.character.carry(*n);
                receipt.push(format!("+{n} experience, carried"));
            }
            Outcome::Supply { id, n } => {
                let all = crate::data::supplies();
                match all.get(id) {
                    Some(def) => {
                        self.character.give_supply(id, *n);
                        receipt.push(format!("{n} × {}", def.name));
                    }
                    None => receipt.push(format!("{id} is not a supply")),
                }
            }
            Outcome::Tire(pct) => {
                let before = self.character.fatigue;
                self.character.tire(*pct as i32);
                let by = self.character.fatigue - before;
                receipt.push(format!("{by}% more tired"));
            }
            Outcome::Warp { map, at } => {
                self.warp_to(map, *at, difficulty);
                receipt.push("You are somewhere else. It is a long walk back.".into());
            }
            Outcome::Nothing => receipt.push("Nothing you could point to".into()),
        }
    }

    /// Put the player on another map, at a tile they can stand on.
    ///
    /// **The same three steps a gate takes, in the same order**, because a
    /// warp is a crossing that did not ask: remember where you were standing
    /// so the map you are leaving knows, move, then `repair` — which is what
    /// stops a warp ever landing somebody in scenery. `every_warp_lands_
    /// somewhere_you_can_stand` checks the data over every map; this is what
    /// makes a warp safe even where the data is wrong.
    pub fn warp_to(&mut self, map: &str, at: [u8; 2], difficulty: crate::combat::Difficulty) {
        self.world.remember_at(self.world.at);
        self.world.map = map.to_string();
        self.world.at = at;
        let allowed = self.character.allowances();
        let id = self.world.map_id();
        let w = crate::data::map_now(&id, difficulty, &self.world);
        w.repair(&mut self.world, &allowed);
    }

    // ------------------------------------------------------------ orders

    /// Place an order at this town's counter.
    ///
    /// **Every refusal names the thing in the way**, TONE 12, and each is a
    /// different rule: you are not here, they do not make that, you already
    /// have one on order, or you have not got the money. A button that greys
    /// out with no sentence is a button that reads as broken.
    ///
    /// **One open order per town** (`PLAN-M12.md` §8 row 4) — per town rather
    /// than globally, so each town's ledger is its own small promise and the
    /// walk back is the economy working.
    pub fn order(&mut self, town: &str, index: usize) -> Result<crate::world::Commission, String> {
        let shops = crate::data::shops();
        let book = crate::shop::commissions(&shops, town);
        let Some(o) = book.iter().find(|o| o.index == index) else {
            return Err("They do not make that here.".into());
        };
        if let Some(open) = self.world.commissions.iter().find(|c| c.town == town) {
            let def = crate::shop::def_named(&open.piece).map(|i| crate::piece::CATALOG[i].name);
            return Err(format!(
                "They are already making you {}. One at a time.",
                def.map(|n| self.theme_piece(n)).unwrap_or_else(|| open.piece.clone())
            ));
        }
        if self.character.gold < o.price {
            return Err(format!("{} Fnorp, and you have {}.", o.price, self.character.gold));
        }
        self.character.gold -= o.price;
        let c = crate::world::Commission {
            town: town.to_string(),
            piece: o.def.name.to_string(),
            fights_left: o.fights,
        };
        self.world.commissions.push(c.clone());
        Ok(c)
    }

    /// Take delivery, if it is ready and you are standing where you ordered it.
    ///
    /// **Collected in person.** The piece does not arrive in the bag when the
    /// last fight ends, because then the order would be a timer rather than an
    /// errand — walking back for it is what makes the ledger part of the
    /// travel economy rather than a second inventory.
    pub fn collect(&mut self, town: &str) -> Result<String, String> {
        let Some(i) = self.world.commissions.iter().position(|c| c.town == town) else {
            return Err("You have nothing on order here.".into());
        };
        if self.world.commissions[i].fights_left > 0 {
            let left = self.world.commissions[i].fights_left;
            return Err(format!(
                "Not yet. {left} more {}.",
                if left == 1 { "fight" } else { "fights" }
            ));
        }
        let c = self.world.commissions.remove(i);
        self.character.give(&c.piece);
        Ok(c.piece)
    }

    /// What is on order here, if anything.
    pub fn order_at(&self, town: &str) -> Option<&crate::world::Commission> {
        self.world.commissions.iter().find(|c| c.town == town)
    }

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
