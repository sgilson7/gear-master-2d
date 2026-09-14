//! The larder and the bench.
//!
//! Asked for: *"add a potion making stand in cities, which lets you brew
//! together ingredients that enemies drop to make bonuses that can be consumed
//! before battles for a temporary benefit during that battle ... ingredients do
//! not go in the inventory but instead into an ingredient inventory that can
//! only be accessed in towns. the potion brewing window is a single slot with a
//! bizzarre shape."*
//!
//! # What an ingredient is, and what it is deliberately not
//!
//! **It is not a component.** No `PieceKind`, no entry in `CATALOG`, no home in
//! any of the five worn grids — because adding to the catalogue moves the save
//! fingerprint, and there is a player mid-run. It is also not a restorative:
//! a tin is bought and drunk on the road, and this is picked up off a creature
//! and can only be handled at a counter.
//!
//! **It does have a shape**, because the ask says so and because the retort is
//! the whole of the puzzle: nine cells, no symmetry, and two ingredients have
//! to be *arranged* in it rather than merely owned. The shape is a
//! [`crate::shape::Shape`], which is the same polyomino every component uses,
//! so a cell in the bench is the same kind of thing as a cell on a board.
//!
//! # Why a brew adds to `Held`
//!
//! [`crate::combat::Held`] is the one door for *what the character is holding
//! when the bell goes* — the tree's armour, its mana, its granted rules and the
//! furnace's carried stacks all arrive through it. A potion is exactly that
//! shape, so it is **zero new combat code**, and it expires when the fight does
//! for free: `Held` is translated into a `Combatant` at the bell and nothing
//! persists.

use serde::{Deserialize, Serialize};

use crate::shape::Shape;
use crate::stats::Stats;

/// The mask the bench is brewed in, as `(x, y)` cells.
///
/// **Seven cells and not a rectangle**, which is the ask — *a single slot with a
/// bizarre shape* — and is also the only thing that makes brewing a decision. A
/// tidy box is an inventory slot with a different label: every pair fits, so the
/// arrangement carries nothing. This is a bent neck into a bulb, so a
/// three-cell ingredient beside another three-cell ingredient does not always
/// go, and which pair you can actually make is a fact about the glass.
/// Per-mille chance a win leaves an ingredient.
///
/// **It was a certainty, and the Plot was a second source for something you
/// already had enough of.** Asked for: *ingredients should be much rarer so
/// that you have to use the growing system.* Measured rather than chosen — a
/// crop pays `plot::HARVEST_YIELD` every `plot::STAGES - 1` wins, which is two
/// a win from one crop and four from two, against a certain one from fighting.
/// At 250 a win pays a quarter and **two crops growing is sixteen times what
/// fighting pays**, which is the ask stated as a number.
///
/// Not lower, because a brew is a *pair* and the first one has to be reachable
/// before there is anything in the ground: at a quarter a win, two different
/// buckets is about eight fights, and the bed is in the first town from the
/// first afternoon.
///
/// **And this is what makes the Kennel's feed cost something.** A creature out
/// eats one a fight from the same larder, and it used to be fed by the corpse
/// of whatever it helped kill.
pub const INGREDIENT_PER_MILLE: u32 = 250;

pub const RETORT: &[(i8, i8)] = &[
    (0, 0),
    (0, 1),
    (1, 1),
    (2, 1),
    (1, 2),
    (2, 2),
    (3, 2),
];

/// What the Cairnworks adds to it: the one cell that makes room for an ink.
///
/// **One and not two, and the number is measured rather than chosen.** At nine
/// cells every one of the fifty-six triples tiles the glass, so the shape
/// carries nothing and the ink is a free slot — which is the *compares zero
/// with zero* failure with a polyomino in it. At eight, three three-cell
/// ingredients come to nine and can never go in together, so **a big ink wants
/// a small pair**, which is a decision. `the_retort_is_a_shape_and_not_a_box`
/// is what says so, and it was red at nine.
///
/// Separate from `RETORT` rather than a second constant listing eleven cells,
/// for the reason every "derived, never banked" note in this project gives — a
/// second copy of the seven would go stale the first time the glass is reblown.
pub const REBLOWN: &[(i8, i8)] = &[(3, 1)];

/// The mark that says the glass has been reblown, and the ink slot is open.
///
/// **A boss's own tile id, which is what beating it writes into `answered`.**
/// Derived, never banked — there is no `glass_reblown` field, the same way
/// there is no counter for how many floors of the Drambus Stack are gone. The
/// thing at the bottom of the Cairnworks is what reblows it, and the errand
/// Kettleworks hands out is the pointer rather than the trigger.
pub const REBLOWN_MARK: &str = "what-was-left-banked";

/// The most ingredients a brew takes once the glass is reblown.
pub const WITH_INK: usize = 3;

/// An ingredient, as `data/brews.json` writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngredientDef {
    pub id: String,
    pub name: String,
    pub blurb: String,
    /// Its footprint in the retort, in the same `(x, y)` offsets a component's
    /// cells are written in.
    pub cells: Vec<(i8, i8)>,
    /// How much this one multiplies a brew by when it is the **third**
    /// ingredient, in percentage points. An ink in a book, which is what the
    /// ask asks for in as many words.
    pub potency: i32,
    /// The art families whose creatures drop it.
    ///
    /// **Keyed by family and not by creature**, so a creature added to
    /// `enemies.json` cannot arrive without a drop: it already fails
    /// `every_creature_has_a_figure_and_every_figure_has_a_file` without a
    /// family, and a family without an ingredient fails
    /// `every_family_drops_something`.
    pub from: Vec<String>,
    /// Only ever the **third** thing in the glass, and never half of a pair.
    ///
    /// **What a trainer sells.** Asked for as *unlocks unique effects* and
    /// *unlocks special ingredients they can buy from the trainer*, and this is
    /// the shape that costs the table nothing: a pair ingredient added to the
    /// eight would want `C(9,2) - C(8,2)` new brews written for it, and two of
    /// them thirty-four — so a special ingredient that *combines* is a special
    /// ingredient nobody can afford to add a third of.
    ///
    /// An ink multiplies instead, which is exactly what the third slot has done
    /// since M20 and what the ask says an ink is. `every_pair_of_ingredients
    /// _brews_to_something` skips these, and says why where it skips them.
    #[serde(default)]
    pub ink_only: bool,
}

impl IngredientDef {
    pub fn shape(&self) -> Shape {
        Shape::new(&self.cells)
    }
}

/// What a brew is worth at the bell.
///
/// **`deny_unknown_fields` on a struct rather than a free map.** serde drops a
/// key it does not know without a word, which is the *eight skill nodes*
/// failure exactly — eight nodes that parsed, cost points and changed nothing.
/// A container attribute is the one place that can be said, and this is a
/// container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Gives {
    /// Armour already on you when the bell goes. A quantity, so it is `Held`'s.
    pub armor: i32,
    /// Mana already banked. Also a quantity.
    pub mana: i32,
    pub rage: i32,
    pub faith: i32,
    pub nature: i32,
    pub insight: i32,
    pub dread: i32,
    /// Mind damage the character adds to every mind hit, through `Held::mind`.
    pub mind: i32,
    // --- and the rates, which are the player's `Stats` for this fight only ---
    pub health: i32,
    pub strength: i32,
    pub regen: i32,
    pub power: i32,
    pub physical_damage: i32,
    pub magic_damage: i32,
    pub physical_resist: i32,
    pub magic_resist: i32,
    pub physical_pierce: i32,
    pub magic_pierce: i32,
    pub curse_resist: i32,
    pub mind_resist: i32,
}

impl Gives {
    pub fn is_nothing(&self) -> bool {
        *self == Gives::default()
    }

    /// The same, scaled by an ink's potency in percentage points.
    ///
    /// Rounded **down**, and toward the player only where it already is: a
    /// twelve percent ink on a fourteen-strength draught is one more point of
    /// strength rather than one and a bit, which is the same integer discipline
    /// every roll in this game keeps.
    pub fn scaled(&self, pct: i32) -> Gives {
        let s = |v: i32| v + v * pct / 100;
        Gives {
            armor: s(self.armor),
            mana: s(self.mana),
            rage: s(self.rage),
            faith: s(self.faith),
            nature: s(self.nature),
            insight: s(self.insight),
            dread: s(self.dread),
            mind: s(self.mind),
            health: s(self.health),
            strength: s(self.strength),
            regen: s(self.regen),
            power: s(self.power),
            physical_damage: s(self.physical_damage),
            magic_damage: s(self.magic_damage),
            physical_resist: s(self.physical_resist),
            magic_resist: s(self.magic_resist),
            physical_pierce: s(self.physical_pierce),
            magic_pierce: s(self.magic_pierce),
            curse_resist: s(self.curse_resist),
            mind_resist: s(self.mind_resist),
        }
    }

    /// Two draughts in one fighter, added field by field.
    ///
    /// **Written for the Chef**, whose promise is *multiple potions per
    /// fight*. It needed no new combat code and this is why: everything a brew
    /// gives goes through `Held`, which is the one door *what you are already
    /// holding when the bell goes* comes through, and two lots of the same
    /// addition is what two potions are.
    ///
    /// **Adding and not maxing.** Two brews that both give strength give the
    /// sum, because that is what drinking two of them means — and a `max`
    /// would quietly make the second one free, which is the sort of number
    /// nobody could check against anything the game tells them afterwards.
    pub fn and(&self, o: &Gives) -> Gives {
        Gives {
            armor: self.armor + o.armor,
            mana: self.mana + o.mana,
            rage: self.rage + o.rage,
            faith: self.faith + o.faith,
            nature: self.nature + o.nature,
            insight: self.insight + o.insight,
            dread: self.dread + o.dread,
            mind: self.mind + o.mind,
            health: self.health + o.health,
            strength: self.strength + o.strength,
            regen: self.regen + o.regen,
            power: self.power + o.power,
            physical_damage: self.physical_damage + o.physical_damage,
            magic_damage: self.magic_damage + o.magic_damage,
            physical_resist: self.physical_resist + o.physical_resist,
            magic_resist: self.magic_resist + o.magic_resist,
            physical_pierce: self.physical_pierce + o.physical_pierce,
            magic_pierce: self.magic_pierce + o.magic_pierce,
            curse_resist: self.curse_resist + o.curse_resist,
            mind_resist: self.mind_resist + o.mind_resist,
        }
    }

    /// The half of it that is a rate rather than a quantity.
    ///
    /// **The division `Held` exists to make**: a `Stats` figure is what
    /// something pays and a `Held` figure is what you already have. Strength is
    /// the first; armour at the bell is the second.
    pub fn rates(&self) -> Stats {
        Stats {
            health: self.health,
            strength: self.strength,
            regen: self.regen,
            power: self.power,
            physical_damage: self.physical_damage,
            magic_damage: self.magic_damage,
            physical_resist: self.physical_resist,
            magic_resist: self.magic_resist,
            physical_pierce: self.physical_pierce,
            magic_pierce: self.magic_pierce,
            curse_resist: self.curse_resist,
            mind_resist: self.mind_resist,
            ..Stats::ZERO
        }
    }

    /// A line a player can read, in the engine's register.
    ///
    /// **Derived, never typed** — the same rule `Node::line` follows, and for
    /// the same reason: a sentence written into the data file goes stale the
    /// first time the numbers are tuned, and nothing would say so. Unthemed,
    /// TONE 13a: somebody choosing between two brews is comparing numbers.
    pub fn line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        let add = |n: i32, what: &str, parts: &mut Vec<String>| {
            if n != 0 {
                parts.push(format!("{n:+} {what}"));
            }
        };
        add(self.health, "max health", &mut parts);
        add(self.strength, "strength", &mut parts);
        add(self.magic_damage, "magic damage", &mut parts);
        add(self.physical_damage, "physical damage", &mut parts);
        add(self.power, "power", &mut parts);
        add(self.regen, "regen", &mut parts);
        add(self.physical_pierce, "physical piercing", &mut parts);
        add(self.magic_pierce, "magic piercing", &mut parts);
        add(self.physical_resist, "physical resist", &mut parts);
        add(self.magic_resist, "magic resist", &mut parts);
        add(self.curse_resist, "curse resist", &mut parts);
        add(self.mind_resist, "mind resist", &mut parts);
        add(self.mind, "mind damage", &mut parts);
        add(self.armor, "armour at the bell", &mut parts);
        add(self.mana, "mana at the bell", &mut parts);
        add(self.rage, "fury at the bell", &mut parts);
        add(self.faith, "devotion at the bell", &mut parts);
        add(self.nature, "harvest at the bell", &mut parts);
        add(self.insight, "insight at the bell", &mut parts);
        add(self.dread, "dread at the bell", &mut parts);
        if parts.is_empty() {
            return "nothing at all".to_string();
        }
        parts.join(", ")
    }
}

/// A pair, and what the pair is worth. The pair **is** the recipe.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrewDef {
    /// The two ingredient ids, in either order — `BrewsData::pair` sorts before
    /// it compares, because which one you dropped in first is a fact about your
    /// hand and not about the brew.
    pub of: Vec<String>,
    pub name: String,
    pub blurb: String,
    pub gives: Gives,
    /// What an ink added to it, in percentage points.
    ///
    /// **Not in the file** — it is a fact about the potion somebody brewed
    /// rather than about the pair, so it is filled in by
    /// `Character::what_is_brewing` and carried on the potion. `#[serde(skip)]`
    /// because a file that named it would be a second answer to how strong a
    /// brew is.
    #[serde(skip)]
    pub ink_pct: i32,
}

impl BrewDef {
    /// The brew's own id: the two ingredients, sorted and joined.
    ///
    /// **Derived rather than written**, so a pair cannot be given an id that
    /// disagrees with what is in it — and a potion in the pack is the pair it
    /// was made of, which is the only thing anything needs to look one up by.
    pub fn id(&self) -> String {
        let mut of: Vec<&str> = self.of.iter().map(|s| s.as_str()).collect();
        of.sort_unstable();
        of.join("+")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrewsData {
    pub format: String,
    pub version: u32,
    #[serde(default, rename = "_note")]
    pub note: String,
    pub ingredients: Vec<IngredientDef>,
    pub brews: Vec<BrewDef>,
}

const FORMAT: &str = "gm2d-brews";
const VERSION: u32 = 1;

impl BrewsData {
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: BrewsData =
            serde_json::from_str(text).map_err(|e| format!("brews.json will not parse: {e}"))?;
        if d.format != FORMAT {
            return Err(format!("expected a {FORMAT} file, got {:?}", d.format));
        }
        if d.version > VERSION {
            return Err(format!(
                "these brews are version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        // Everything a brew names has to exist, and every ingredient has to fit
        // in the glass at all — checked once here rather than discovered by a
        // player holding two things that will never go in together.
        let retort = Shape::new(RETORT);
        for i in &d.ingredients {
            if i.cells.is_empty() {
                return Err(format!("{}: an ingredient with no shape", i.id));
            }
            if i.shape().area() > retort.area() {
                return Err(format!("{}: does not fit in the retort on its own", i.id));
            }
            if i.potency <= 0 {
                return Err(format!("{}: an ink that multiplies by nothing", i.id));
            }
            // **An ingredient comes off a creature, or a trainer sells it.**
            // `from` empty used to mean an orphan and now means the second
            // kind — so the rule is *somebody has it*, and an ink-only
            // ingredient nothing drops is refused only if no counter stocks it
            // either. `every_special_ingredient_is_on_a_counter` is the other
            // half, over the maps, because a map is not in scope here.
            if i.from.is_empty() && !i.ink_only {
                return Err(format!("{}: nothing drops it", i.id));
            }
        }
        for b in &d.brews {
            if b.of.len() != 2 {
                return Err(format!("{}: a brew is a pair", b.name));
            }
            for id in &b.of {
                if !d.ingredients.iter().any(|i| i.id == *id) {
                    return Err(format!("{}: {id:?} is not an ingredient", b.name));
                }
            }
            if b.gives.is_nothing() {
                return Err(format!("{}: gives nothing at all", b.name));
            }
        }
        Ok(d)
    }

    pub fn get(&self, id: &str) -> Option<&IngredientDef> {
        self.ingredients.iter().find(|i| i.id == id)
    }

    /// What this pair brews to, in either order.
    pub fn pair(&self, a: &str, b: &str) -> Option<&BrewDef> {
        let mut want = [a, b];
        want.sort_unstable();
        self.brews.iter().find(|d| {
            let mut got: Vec<&str> = d.of.iter().map(|s| s.as_str()).collect();
            got.sort_unstable();
            got == want
        })
    }

    /// Which ingredient a creature leaves, by the family it is drawn from.
    ///
    /// `None` for a creature with no family, which nothing in the shipped game
    /// is — `every_creature_has_a_figure_and_every_figure_has_a_file` sees to
    /// that, and this is the second half of it.
    pub fn from_family(&self, family: &str) -> Option<&IngredientDef> {
        self.ingredients.iter().find(|i| i.from.iter().any(|f| f == family))
    }
}

/// The cells of the glass this character has, which is seven or eight.
pub fn retort_cells(marks: &[String]) -> Vec<(i8, i8)> {
    let mut out = RETORT.to_vec();
    if marks.iter().any(|m| m == REBLOWN_MARK) {
        out.extend_from_slice(REBLOWN);
    }
    out
}

/// Whether these ingredients can be arranged in the glass at once.
///
/// **This is the whole of the puzzle**, and it is core's for the reason the
/// board's green fit preview is: a page that worked out its own answer would be
/// a second rulebook, and the two would part the first time the glass was
/// reblown.
///
/// Every rotation of every ingredient, seated in turn — the same *turn it and
/// try it* the packing screen has always allowed, because a player turns a
/// piece and would rightly expect to be able to turn a root. Exhaustive rather
/// than clever: nine cells and at most three ingredients of at most three cells
/// is a search that finishes in microseconds, and the alternative is a
/// heuristic that says *no* to an arrangement somebody can see with their eyes.
pub fn fits(glass: &[(i8, i8)], brews: &BrewsData, ids: &[String]) -> bool {
    let shapes: Vec<Vec<Shape>> = match ids
        .iter()
        .map(|id| {
            brews.get(id).map(|d| {
                let base = d.shape();
                let mut turns: Vec<Shape> = Vec::new();
                for t in 0..4 {
                    let s = base.rotated(t);
                    if !turns.contains(&s) {
                        turns.push(s);
                    }
                }
                turns
            })
        })
        .collect::<Option<Vec<_>>>()
    {
        Some(v) => v,
        None => return false,
    };
    let mut taken: Vec<(i8, i8)> = Vec::new();
    seat(glass, &shapes, 0, &mut taken)
}

fn seat(glass: &[(i8, i8)], shapes: &[Vec<Shape>], n: usize, taken: &mut Vec<(i8, i8)>) -> bool {
    if n == shapes.len() {
        return true;
    }
    for turn in &shapes[n] {
        for &(ox, oy) in glass {
            let cells: Vec<(i8, i8)> =
                turn.cells().iter().map(|&(x, y)| (x + ox, y + oy)).collect();
            if cells.iter().any(|c| !glass.contains(c) || taken.contains(c)) {
                continue;
            }
            let before = taken.len();
            taken.extend_from_slice(&cells);
            if seat(glass, shapes, n + 1, taken) {
                return true;
            }
            taken.truncate(before);
        }
    }
    false
}

/// One ingredient seated in the glass.
///
/// **The seating is the puzzle, so the seating is what is saved.** A list of
/// ids would say what went in and not how, and the whole of what the ask wants
/// — *built using gear shaped items like the surveying table* — is that an
/// ingredient is a shape you put somewhere rather than a line you tick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seat {
    pub id: String,
    /// Quarter turns clockwise, the same unit a component's rotation is in.
    #[serde(default)]
    pub turn: u8,
    pub at: [u8; 2],
}

/// Where an ingredient's cells land, seated at `at` and turned `turn`.
pub fn cells_of(def: &IngredientDef, turn: u8, at: [u8; 2]) -> Vec<(i8, i8)> {
    def.shape()
        .rotated(turn)
        .cells()
        .iter()
        .map(|&(x, y)| (x + at[0] as i8, y + at[1] as i8))
        .collect()
}

/// Every cell the seated ingredients are standing on.
pub fn taken(brews: &BrewsData, seats: &[Seat]) -> Vec<(i8, i8)> {
    seats
        .iter()
        .filter_map(|s| brews.get(&s.id).map(|d| cells_of(d, s.turn, s.at)))
        .flatten()
        .collect()
}

/// Which seat, if any, is standing on a cell.
pub fn seat_at(brews: &BrewsData, seats: &[Seat], x: u8, y: u8) -> Option<usize> {
    seats.iter().position(|s| {
        brews
            .get(&s.id)
            .map(|d| cells_of(d, s.turn, s.at).contains(&(x as i8, y as i8)))
            .unwrap_or(false)
    })
}

/// Every anchor this ingredient could be seated at, turned this way.
///
/// **Core's, for the reason the board's green fit preview is core's.** A page
/// that worked out its own answer would be a second rulebook about what goes
/// in the glass, and the two would part the first time the glass was reblown.
pub fn legal_anchors(
    glass: &[(i8, i8)],
    brews: &BrewsData,
    seats: &[Seat],
    id: &str,
    turn: u8,
) -> Vec<[u8; 2]> {
    let Some(def) = brews.get(id) else { return Vec::new() };
    let full = taken(brews, seats);
    let mut out = Vec::new();
    for &(ox, oy) in glass {
        if ox < 0 || oy < 0 {
            continue;
        }
        let at = [ox as u8, oy as u8];
        let cells = cells_of(def, turn, at);
        if cells.iter().all(|c| glass.contains(c) && !full.contains(c)) {
            out.push(at);
        }
    }
    out
}
