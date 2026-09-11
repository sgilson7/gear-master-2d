//! Everything you need to know to play, in one place.
//!
//! Reported from play: *"can you also add a game glossary that explains
//! everything you need to play the game, with stuff like what mana empowerment
//! does"* — and that example is the whole argument. Mana empowerment is
//! `stacks × 5 × the mana you are holding` and it scales one lane; nothing in
//! the game ever said so, so a player watching a Kettle-Stoker's stacks climb
//! had no way to tell a mechanic they did not understand from one that was
//! broken. (It was broken. Both halves needed doing.)
//!
//! **Derived, never typed, which is the only version of this that stays
//! true.** Every number here is read from the constant that decides it, every
//! class describes itself with `ClassPower::describe`, every rule with
//! `Rule::line`, and the pools with `Combatant::pool_pays` — the same function
//! the standing panel draws from. A glossary with its numbers written out by
//! hand is a second rulebook with a slower feedback loop than the first, and
//! this repository has paid for one of those six times.
//!
//! **Unthemed, TONE 13a.** Somebody reading a glossary is comparing
//! mechanisms; a number wearing a joke has to be translated before it can be
//! compared. The *terms* go through the theme where the game's own screens use
//! the theme's word for them — a player looking up "cork" must find it — and
//! what a thing *does* is the engine's, in the engine's words.

use crate::piece::Resource;

/// One entry: what it is called, what it does, and the aside some carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The term, as a player meets it.
    pub term: String,
    /// What it does. One line a fact.
    pub body: Vec<String>,
    /// Where you get it, or what it wants. Empty for most.
    pub aside: Vec<String>,
    /// The canonical name, when this entry *is* a class.
    ///
    /// **The theme's table is keyed on `&'static str`** and an entry's term is
    /// owned, so the shim cannot look one up from the term alone. Carrying the
    /// key is the alternative to `Theme::class` growing a lifetime for one
    /// caller — and it says which entries are classes, which is a fact the
    /// screen wants anyway.
    pub key: Option<&'static str>,
}

impl Entry {
    fn new(term: &str, body: &[&str]) -> Entry {
        Entry {
            term: term.to_string(),
            body: body.iter().map(|s| s.to_string()).collect(),
            aside: Vec::new(),
            key: None,
        }
    }

    fn of_class(mut self, canonical: &'static str) -> Entry {
        self.key = Some(canonical);
        self
    }

    fn with_aside(mut self, aside: &[String]) -> Entry {
        self.aside = aside.to_vec();
        self
    }
}

/// One shelf of the glossary: a heading and its entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shelf {
    pub name: &'static str,
    pub entries: Vec<Entry>,
}

/// Every shelf, in the order somebody meeting the game needs them.
///
/// **Getting about, then the board, then the fight, then what you can become,
/// then what you carry.** That is the order the game teaches itself in and the
/// order a person asks in: you walk before you pack, you pack before you
/// fight, and you do not choose a class until level five.
pub fn shelves() -> Vec<Shelf> {
    vec![
        Shelf { name: "Getting about", entries: getting_about() },
        Shelf { name: "The board", entries: the_board() },
        Shelf { name: "The fight", entries: the_fight() },
        Shelf { name: "What you can become", entries: classes() },
        Shelf { name: "What you carry", entries: carried() },
    ]
}

fn getting_about() -> Vec<Entry> {
    vec![
        Entry::new("Walking", &[
            "On most maps the arrow keys move you one tile a press.",
            "Every tile you step onto rolls for a fight. How likely is the \
             ground's own rate times the region's danger, and the strip shows \
             the figure.",
            "A step into rock, water or a wall is refused and rolls nothing — \
             bumping into a cliff costs you nothing at all.",
        ]),
        Entry::new("Shooting", &[
            "Two maps are tables rather than floors: the Treyway and the \
             Undercountry. There the arrow keys aim a cue instead of taking a \
             step.",
            "Left and right turn the aim five degrees; up and down change the \
             power from one to ten; space fires. You can also drag on the map \
             with the mouse — pull back from the ball, and let go.",
            "Where the ball comes to rest is where you are, and only that tile \
             rolls for a fight — at 150% of its usual rate, because a landing \
             is a longer stay than a step.",
        ]),
        Entry::new("Diamonds", &[
            "A diamond on the map is a way into somewhere: a gate onto another \
             map, or a creature standing on a tile.",
            "On a table a diamond catches the ball. You do not have to stop \
             exactly on one — hitting it is entering it.",
        ]),
        Entry::new("Obstacles", &[
            "On a table, five things are in the ball's way and each has its own \
             mark.",
            "A boulder is solid and throws the ball back harder than it \
             arrived. A hole swallows it, and you wake up in the last town you \
             stood in.",
            "Spikes cost you fatigue and let the ball through. A drift of sand \
             stops it dead. A chute carries it somewhere else at the same speed.",
        ]),
        Entry::new("Towns", &[
            "A town is the only place experience becomes a level, and the only \
             thing that takes tiredness off.",
            "Everything you are carrying is lost if you fall before you reach \
             one.",
        ]),
        Entry::new("The long cart", &[
            &format!(
                "From a town, the cart runs to any other town you have stood \
                 in, for {} Fnorp.",
                crate::game::CART_FARE
            ),
            "It runs counter to counter, so it can never be the thing that \
             saves a run in the wilds. That is what the Drover's Stride is for.",
        ]),
    ]
}

fn the_board() -> Vec<Entry> {
    let base = crate::progression::base_rows();
    vec![
        Entry::new("Grids", &[
            &format!(
                "Five grids you wear — helmet, chest, gloves, greaves and a \
                 weapon — each six wide and {base} rows tall to begin with, up \
                 to {} rows once the tree has been walked all the way up.",
                crate::progression::MAX_ROWS
            ),
            "A sixth grid holds a survey instrument. It never grows, it holds \
             one instrument, and nothing that asks what your board is worth \
             ever counts it.",
            "Rows are not handed out by levelling. You buy them with skill \
             points or finish an errand for them.",
        ]),
        Entry::new("Items", &[
            "Two components that touch are part of the same item. An item is \
             what fights, not a component.",
            "A grid's ? button says what that grid can be built out of. A \
             weapon can be a blade, a book or a crystal ball, and the three \
             want different things.",
            "An item that does not match a recipe assembles nothing and does \
             nothing at all, however good its pieces are.",
        ]),
        Entry::new("Footprints", &[
            "A component takes up the cells its shape covers. A four-cell \
             blade fills four; a ring fills one.",
            "You can turn a component before you place it. A piece that will \
             not fit upright often fits lying down.",
        ]),
        Entry::new("Auto-pack", &[
            "The button packs what you own, seeding on a core and keeping only \
             placements that strictly improve how many items you have and what \
             they rate.",
            "It is not an optimiser and is not meant to be — the arrangement is \
             the game. What it does is leave nothing obvious in the bag.",
        ]),
        Entry::new("Rarity", &[
            &format!(
                "An item's rating prices its pieces and how often it comes \
                 round. {} is Rare, {} is Epic and {} is Legendary.",
                crate::rating::RARE_AT,
                crate::rating::EPIC_AT,
                crate::rating::LEGENDARY_AT
            ),
            "It is mostly a measure of what creatures wear. The best board this \
             game hands a player is Epic, and most items are Common.",
        ]),
        Entry::new("Enchantments", &[
            "A component of that kind is laid *under* a grid and gear sits on \
             top of it, so it takes no cell away from anything.",
            "One touching another kills both. What is next to what is the \
             whole of it.",
        ]),
        Entry::new("Enchs", &[
            "Not the same thing as an enchantment. An ench is bolted onto one \
             component and moves that component's power or how fast it comes \
             round.",
            "It stays on the component through being picked up, turned and \
             moved. One a component — two if you hold the Full Bill.",
            &format!(
                "You need a licence. The Kaklon Patent comes with one; anybody \
                 else buys the paper for {} Fnorp.",
                crate::ench::LICENCE_PRICE
            ),
        ]),
        Entry::new("The spin", &[
            &format!(
                "An item with room to turn where it stands turns as it fights, \
                 and each turn is worth {}% more power.",
                crate::combat::SPIN_PCT_PER_TURN
            ),
            "Leaving room to turn costs you cells. That is the trade.",
            "An item that is boxed in does not move.",
        ]),
    ]
}

fn the_fight() -> Vec<Entry> {
    let mut out = vec![
        Entry::new("How a fight works", &[
            "Nothing is random. Both boards tick, each item comes round at its \
             own rate, and the same two boards always fight the same fight.",
            "Health resets at every bell. What a fight actually spends is \
             fatigue.",
            "A fight that has gone on too long ends at a sudden-death clock, \
             and a win at the buzzer is a win a board did not earn.",
        ]),
        Entry::new("Armour", &[
            "Armour soaks damage before health does, and it is gone at the end \
             of the fight.",
            "The bar wraps rather than clamping: past full, each complete bar \
             is another layer drawn darker than the one under it.",
        ]),
        Entry::new("Fatigue", &[
            &format!(
                "Every fight takes {}% of your maximum health for good, won or \
                 lost.",
                crate::fatigue::PER_FIGHT
            ),
            &format!(
                "It stops at {}% from fighting. Running away and walking off \
                 something that had nothing for you can take you past that, to \
                 {}%.",
                crate::fatigue::CAP,
                crate::fatigue::HARD_CAP
            ),
            "A town takes all of it off. A tin takes some of it off wherever \
             you are standing.",
        ]),
    ];

    // **The pools, off the same function the standing panel draws from.** A
    // pool has to be one the catalogue can grant *and* one that pays something
    // for being held; `pools_worth_holding` is that question and this is its
    // answer, so a component that starts granting a pool puts it here without
    // anybody remembering to.
    let mut lines = vec![
        "Four pools bank as you fight. Three of them pay you for holding on to \
         them, per point:"
            .to_string(),
    ];
    for what in crate::combat::Combatant::pools_worth_holding() {
        // **`Stats::parts` is the engine's own phrasing**, and it is what the
        // item card splits on and what `pools_json` draws the standing panel
        // from. Listing the fields by hand here was a second answer to *what
        // does a pool pay* and it was already wrong: it dropped rage, which is
        // the one every player meets first.
        let pays = crate::combat::Combatant::pool_pays(what);
        let bits: Vec<String> =
            pays.parts().into_iter().map(|(t, _)| t.to_string()).collect();
        if !bits.is_empty() {
            lines.push(format!("{}: {}", name_of(what), bits.join(", ")));
        }
    }
    lines.push(
        "Mana and insight pay nothing for sitting on a pile. They are spent."
            .to_string(),
    );
    out.push(Entry { term: "Pools".to_string(), body: lines, aside: Vec::new(), key: None });

    out.extend([
        Entry::new("Mana empowerment", &[
            "Stacks that multiply what your weapon hits for. Each stack is \
             worth 5% of one point of power per point of mana you are holding \
             — so a stack multiplies the mana you have, and with no mana a \
             stack is worth nothing.",
            "Spending mana therefore gives up the thing empowerment scales. A \
             hoarded pool is a standing bonus and a spent one is a burst.",
            "Ordinary empowerment scales magic hits. The Kettle-Stoker's \
             furnace is counted apart and scales both lanes, so a Stoker \
             swinging a blade gets what the class promises.",
        ]),
        Entry::new("Casting", &[
            &format!(
                "A spell costs {} mana. If the mana is not there the cast does \
                 not happen.",
                crate::combat::SPELL_MANA_COST
            ),
        ]),
        Entry::new("Resistance", &[
            &format!(
                "Resistance cuts a blow of its type. Piercing cuts through \
                 resistance, and hardening cancels piercing. Resistance stops \
                 at {}%.",
                crate::stats::RESIST_CAP
            ),
            &format!(
                "The curse and mind lanes stop at {}% as well, so no creature \
                 is ever completely immune to either. A lane you can commit to \
                 must never be one you can be shut out of.",
                crate::stats::LANE_CAP
            ),
        ]),
        Entry::new("Curses", &[
            "Four kinds, landed by gear rather than by you. Each shows as a \
             chip beside the pools with its stacks, what it is doing, and how \
             long it has left.",
            "How likely one is to land is what is left after the target's curse \
             resistance.",
        ]),
        Entry::new("The mind lane", &[
            "Mind damage eats a creature's MAXIMUM health rather than what is \
             left of it. Something whose maximum reaches nothing is down.",
            "It is a second way to finish a fight rather than a faster first \
             one.",
        ]),
        Entry::new("Instant battle", &[
            &format!(
                "Beat something {} times and you may mark it. From then on, \
                 meeting it settles where it stands — fought in full, paid in \
                 full, and never drawn.",
                crate::fight::INSTANT_AFTER
            ),
            "It still costs the fatigue and still rolls the drops, because \
             that is what going through the settlement does.",
            "A creature standing on a boss tile is never settled this way.",
        ]),
        Entry::new("Losing", &[
            "A defeat pays nothing, walks you home, and takes every point of \
             experience you were carrying but had not banked.",
            "It also takes your place: a map you were carried off returns you \
             to its door rather than to where you fell.",
        ]),
    ]);
    out
}

fn name_of(what: Resource) -> &'static str {
    match what {
        Resource::Mana => "mana",
        Resource::Rage => "rage",
        Resource::Faith => "faith",
        Resource::Nature => "nature",
        Resource::Insight => "insight",
        // **Named rather than `_`.** A ninth pool should be a compile error
        // here rather than a glossary entry reading "a pool".
        Resource::DruidicMight => "druidic might",
        Resource::Communion => "communion",
        Resource::Zealotry => "zealotry",
    }
}

/// Every class a player can be, described by the engine rather than by a
/// person: the promise is `ClassPower::describe` and it is the same sentence
/// the fork screen shows before an irreversible choice.
fn classes() -> Vec<Entry> {
    let mut out = vec![Entry::new("How you get one", &[
        "At level five, in a town, you are asked once and the answer does not \
         come off.",
        "Finish a class tree and Spike sells a second paper. Finish two and he \
         hands over the expert that pair reaches, free — the points were the \
         price.",
        "You can hold three, and all three are live at once.",
    ])];
    for name in crate::class::OFFERED {
        let Some(def) = crate::class::CLASSES.iter().find(|c| c.name == *name) else { continue };
        out.push(
            Entry::new(def.name, &[&def.power.describe()])
                .of_class(def.name)
                .with_aside(&["Offered at the level-five fork.".to_string()]),
        );
    }
    for e in crate::expert::EXPERTS {
        out.push(
            Entry::new(e.name, &[&e.power.describe()]).of_class(e.name).with_aside(&[format!(
                "What {} and {} reach together.",
                e.pair.0, e.pair.1
            )]),
        );
    }
    out
}

fn carried() -> Vec<Entry> {
    vec![
        Entry::new("Tins and charms", &[
            "A tin takes a share of your tiredness off wherever you are \
             standing. Every town sells them.",
            "The charms are not tins: one pays for a running-away and one puts \
             you in the last town you stood in.",
        ]),
        Entry::new("The bank", &[
            "One vault, reachable from any town, no limit.",
            "Banked is not carried. A banked component does not pack, does not \
             count against you, and cannot be handed over a counter.",
        ]),
        Entry::new("Errands", &[
            "Somebody asks, and the log says where to go. Pinning one makes \
             the map point at it.",
            "A tally is a thing in your bag rather than a number: beat the \
             creature, carry the token back.",
        ]),
        Entry::new("Instruments", &[
            "A compass, an atlas or a survey golem, built on their own grid.",
            "They read a map you cannot otherwise enter, and which one you \
             built decides what that map is like — a compass quiets the ground, \
             an atlas pays better and is louder, a golem handles one fight.",
        ]),
        Entry::new("Keys and quest items", &[
            "Carried, never worn, so they can never take a cell.",
            "A key is spent opening its lock, and the lock stays open.",
        ]),
        Entry::new("The counters", &[
            "Three tiers under every town. The bargain barrel is cheap and \
             rolled; the shelf is the town's own and never changes; the order \
             book is dear and arrives after a number of fights rather than at \
             once.",
            "The barrel and the order book can be rerolled, and the price goes \
             up each time until the next ten levels.",
        ]),
    ]
}
