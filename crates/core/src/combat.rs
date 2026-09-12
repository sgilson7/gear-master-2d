//! Combat: a fixed-timestep simulation where every assembled item runs its own
//! cooldown.
//!
//! There are no turns. The fight is stepped in [`TICK_MS`] slices and each item
//! fills its own bar independently, so a fast weapon really does swing more
//! often than a slow one. Nothing is random — the same loadout against the same
//! monster always produces the same log, which is what lets the tests assert on
//! exact numbers and lets the GUI replay a fight it did not simulate.

use crate::curse::{mind_damage_after_resist, CurseKind, Curses, STUN_CAP_MS, TICK_MS};
use crate::loadout::ItemProfile;
use crate::piece::{Action, Resource, SlotKind, Target, Trigger, Watched};
use crate::stats::Stats;

/// How often damage-over-time is summarised into the log.
pub const BURN_REPORT_MS: u32 = 1000;

/// A fight this long is called a draw, so a build that cannot finish the job
/// doesn't hang the simulation.
/// How long slow time spreads a hit over.
pub const SLOW_TIME_MS: u32 = 5000;

pub const MAX_DURATION_MS: u32 = 60_000;

/// When a fight that will not end starts ending itself.
///
/// Nothing happens for the first thirty seconds - a long fight is allowed to
/// be a long fight. Past that, both fighters take a share of their own maximum
/// health every second, and the share grows: one percent, then two, then
/// three. The total passes a hundred percent after fourteen seconds, so no
/// fight runs beyond about forty-four however much health or armour is in it,
/// which is the point.
///
/// It replaces a sixty-second cap that scored a draw as a loss. That rule made
/// every defensive option unplayable: armour buys survival, survival was not
/// victory, and a build that could out-last anything but out-damage nothing
/// lost anyway. Nothing here is dodgeable - the damage ignores armour and
/// resistance both, because a wall you can hide behind for ever is the thing
/// being fixed.
pub const SUDDEN_DEATH_MS: u32 = 30_000;

/// How much surrendered damage buys one step of the wrong sense.
pub const WRONG_SENSE_PER: i32 = 60;
/// What one step is worth, as a percentage added to mind damage.
pub const WRONG_SENSE_STEP: i32 = 10;
/// How many steps it may reach.
///
/// Capped, because an uncapped conversion is a board that gets stronger for
/// every second it fails to kill anything - and `SUDDEN_DEATH_MS` already owns
/// everything past thirty seconds, so an uncapped one would make the clock the
/// only opponent. Twenty steps is triple mind damage and it is reached in a
/// fight a board was winning anyway.
pub const WRONG_SENSE_CAP: i32 = 20;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Side {
    Player,
    Enemy,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Player => Side::Enemy,
            Side::Enemy => Side::Player,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Side::Player => "You",
            Side::Enemy => "Enemy",
        }
    }
}

// ------------------------------------------------------------- monsters

/// One repeating attack belonging to a monster. Monsters use the same cooldown
/// machinery as the player's gear rather than a special case.
#[derive(Copy, Clone, Debug)]
pub struct MonsterAttack {
    pub name: &'static str,
    pub cooldown_ms: u32,
    pub damage: i32,
    pub mind: i32,
    pub armor: i32,
    /// Landed on the player each time this attack resolves.
    pub curse: Option<CurseKind>,
}

impl MonsterAttack {
    pub const fn hit(name: &'static str, cooldown_ms: u32, damage: i32) -> Self {
        MonsterAttack { name, cooldown_ms, damage, mind: 0, armor: 0, curse: None }
    }
    pub const fn cursing(
        name: &'static str,
        cooldown_ms: u32,
        damage: i32,
        curse: CurseKind,
    ) -> Self {
        MonsterAttack { name, cooldown_ms, damage, mind: 0, armor: 0, curse: Some(curse) }
    }
    pub const fn mind(name: &'static str, cooldown_ms: u32, mind: i32) -> Self {
        MonsterAttack { name, cooldown_ms, damage: 0, mind, armor: 0, curse: None }
    }
    pub const fn shielding(name: &'static str, cooldown_ms: u32, armor: i32) -> Self {
        MonsterAttack { name, cooldown_ms, damage: 0, mind: 0, armor, curse: None }
    }
}

/// Which silhouette to draw for a monster. Named rather than matched on the
/// monster's name, so a rename can't silently change what it looks like.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MonsterSprite {
    Rat,
    Warden,
    Gearwright,
    Toad,
    Archer,
    Golem,
    Wisp,
    Hound,
    Sentinel,
    Wraith,
    Idol,
    Fiend,
    King,
    // Added when the ladder grew to forty-nine and thirteen silhouettes were
    // being shared five ways. A creature you cannot tell from the last one is
    // a creature you have not really met.
    Francis,
    Marshal,
    Null,
    Lantern,
    Choir,
    Silence,
    Hourglass,
    Tallow,
    Weeping,
    Wedding,
    Twin,
    Mirror,
    Sootmother,
    Ashes,
    Crown,
    Drowned,
    Anvil,
    Parliament,
    Abbot,
    Gilt,
    Vermin,
    Behemoth,
    Cantor,
    Ember,
    Curator,
    Idiot,
    Rimefather,
    Slag,
    Obsidian,
    Gallows,
    CogPriest,
    RuinHound,
    Salt,
    Verdigris,
    March,
    Bells,
    Colossus,
}

/// One entry in a monster's loadout: `(component, slot, x, y, quarter turns)`.
pub type GearPlacement = (&'static str, SlotKind, u8, u8, u8);

/// What kind of fight this is.
///
/// Not decoration: rank decides how densely a creature is allowed to pack its
/// board (see `Rank::min_items_per_slot`), and whether beating it drops
/// something a shop will never sell you.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum Rank {
    #[default]
    Ordinary,
    Mini,
    Boss,
}

impl Rank {
    /// How many assembled items each slot this creature *uses* must hold.
    ///
    /// An ordinary creature is allowed a loose board. The named ones are not:
    /// a boss whose helmet holds one item is a boss you out-gear, and the
    /// whole point of locking items is that a board can hold more than the
    /// authoring tool used to be able to find.
    ///
    /// "Uses" rather than "has", which it used to be. A themed creature wears
    /// two slots and a themed hybrid three or four - that is what a theme is -
    /// so demanding density in all five demands that no creature have a theme.
    /// The density rule is about the slots a creature actually turns up
    /// wearing; how many of those there must be is `min_slots`.
    pub fn min_items_per_slot(self) -> usize {
        match self {
            Rank::Ordinary => 0,
            Rank::Mini => 2,
            Rank::Boss => 3,
        }
    }

    /// The same, for one slot.
    ///
    /// The weapon is one item whatever the rank. A creature carrying three
    /// swings three times a cooldown and no board can answer that - which the
    /// packer has enforced since it was written, while this rule asked a boss
    /// for three items in every slot it wears. Six of the ten named fights are
    /// given a weapon slot by their theme, so for six of ten the two rules
    /// could not both be satisfied; The Dreaming Idiot was already in that
    /// state, unnoticed, because the test that asks walks the ladder and it is
    /// an alternate.
    pub fn min_items_in(self, slot: SlotKind) -> usize {
        if slot == SlotKind::Weapon {
            self.min_items_per_slot().min(1)
        } else {
            self.min_items_per_slot()
        }
    }

    /// How many slots a named creature has to turn up wearing.
    ///
    /// Set by the themes rather than by taste. A mini-boss is a hybrid of its
    /// own cluster and the next, and the thinnest such pairing shares no slot
    /// and has none to spare: the two drainers at rungs 39 and 43 find nothing
    /// past rung 44 to hybridise with and wear their own two. A boss's two
    /// clusters always overlap by one, which is three. Anything below these is
    /// a named fight hiding in a corner of its board.
    pub fn min_slots(self) -> usize {
        match self {
            Rank::Ordinary => 0,
            Rank::Mini => 2,
            Rank::Boss => 3,
        }
    }

    pub fn is_named(self) -> bool {
        !matches!(self, Rank::Ordinary)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MonsterSpec {
    pub name: &'static str,
    /// Innate stats before gear: mostly just how much health it has.
    pub health: i32,
    /// Innate strength, which its weapons then scale.
    pub strength: i32,
    pub regen: i32,
    pub mind_resist: i32,
    pub curse_resist: i32,
    /// The two resistances most attacks answer to. Without these on the
    /// ladder, piercing and hardening would be inert: you would always be
    /// piercing nothing.
    pub physical_resist: i32,
    pub magic_resist: i32,
    /// Innate attacks — a rat's teeth, not equipment. Most of the ladder
    /// leaves this empty and fights with gear instead.
    pub attacks: &'static [MonsterAttack],
    /// Real components in real slots, assembled by the same rules the player
    /// plays by. This is what actually sets a monster's difficulty: to make one
    /// harder, give it better gear.
    pub gear: &'static [GearPlacement],
    /// Steps this monster's gear up or down its own kinds, on top of whatever
    /// the difficulty does. Negative means it fights in worse equipment than
    /// written at every setting.
    ///
    /// This is the dial for a monster that is out of step with its rung -
    /// preferable to rewriting its loadout, because the harder settings still
    /// climb from wherever it is put.
    ///
    /// Eight mid-ladder ones were once stepped down to soften a wall at rung
    /// 9, but the wall turned out to be the balance harness packing its builds
    /// too loosely, not the monsters. **Move one off zero only with evidence
    /// from a densely packed profile.**
    ///
    /// **Six are off zero, and this is the evidence.** The Kettleworks field
    /// pool sits at -2, reported from play as unbeatable on arrival and
    /// measured against a level-ten board that has bought both shelves, done
    /// the errands and packed one item to a grid — a densely packed profile at
    /// the level the map is met at. At 0 that board beat *nothing* in the
    /// pool: The Curator, Pale Twin and the Kettle Wight all ran to the
    /// sudden-death clock and both hounds killed it. At -2 the two commonest
    /// draws are ordinary fights — The Curator dies in seventeen seconds
    /// rather than at the buzzer — and the pool's ratings fall 7 to 17%.
    ///
    /// Body numbers were tried first and are not the lever: health, strength,
    /// regen and both resistances scaled to *seventy* percent changed not one
    /// outcome. Almost all of a Kettleworks creature's rating is the gear it
    /// wears, which is what this dial moves and what `creature_rating` is
    /// mostly counting.
    ///
    /// `tests/kettleworks.rs` is the measurement, kept so the next person to
    /// touch this pool finds out what it cost rather than what it is.
    pub gear_offset: i32,
    /// Gold awarded for beating it.
    pub bounty: i32,
    pub sprite: MonsterSprite,
    /// Ordinary, mini-boss or boss. Defaults to ordinary, which is what the
    /// forty creatures that are neither stay at.
    pub rank: Rank,
    /// Components only this creature carries, and only it can drop. Empty for
    /// everything that is not named.
    pub drops: &'static [&'static str],
    /// How many pieces of `gear`, in order, make up each item.
    ///
    /// Needed only where a board is packed tightly enough that the pieces
    /// would otherwise negotiate: two items sitting flush merge into one
    /// over-full item unless each is locked before the next goes down. Empty
    /// means "work it out", which is right for the loose boards.
    pub items: &'static [usize],
    /// Enchs bolted to this creature's gear: the ench's id, and the index into
    /// `gear` of the component it is on.
    ///
    /// **The first creature in the game that carries one**, and it is a
    /// creature wearing a real player's board, so the alternative was a boss
    /// whose numbers quietly did not match the save they were copied from.
    ///
    /// An index into `gear` rather than a component name, for the reason the
    /// save uses a registry index: this board holds three Quicksilver Inks and
    /// a name cannot say which of them the Band is on.
    ///
    /// **Applied to the profiles, never to the board**, which is the division
    /// `Character::combat_items` already makes: a profile is the board's answer
    /// to what these cells made and an ench is the *wearer's*. A loadout that
    /// knew about enchs would be a loadout that knew about a licence — and a
    /// creature has no licence, which is exactly why this is a field on the
    /// spec and not a rule anywhere.
    pub enchs: &'static [(&'static str, usize)],
}

/// The component this one becomes `step` rungs up its own kind.
///
/// Same kind and the same footprint, so the monster's layout still packs
/// exactly as authored - no re-solving, and a boss cannot end up with a hole
/// in its board because a swap was one cell too wide. Where a kind has no
/// better piece of that shape, the original stands.
pub fn stepped_component(name: &str, step: i32) -> &'static str {
    use crate::piece::CATALOG;
    let Some(here) = CATALOG.iter().find(|d| d.name == name) else { return "" };
    if step == 0 {
        return here.name;
    }
    let mut family: Vec<&'static crate::piece::PieceDef> = CATALOG
        .iter()
        .filter(|d| d.kind == here.kind && d.slot == here.slot && d.cells == here.cells)
        // Never step into gear that belongs to somebody.
        //
        // A trophy is off the scale for its slot by design, and stepping does
        // not know that: it sorts a footprint family by rating and takes the
        // next one up. On Hard that handed the Padded Base's family Francis's
        // coat - 2100 health - so the fourth creature on the ladder fought
        // with 2400 health instead of 475, and forty-five others were doing
        // the same thing. It was one piece until ten trophies were added, and
        // then it was everywhere.
        .filter(|d| !crate::piece::is_boss_only(d.name))
        // Quest rewards are earned, not stepped into, for the same reason
        // they are kept off the shelves.
        .filter(|d| !crate::piece::is_quest_reward(d.name))
        // And so is everything else the road hands over.
        //
        // This list was two entries long and should always have been four.
        // Event gear was already reaching monster boards before anybody
        // noticed - `Gold Chip` and `Crownwright's Measure` both turn up in
        // Nine of Ashes's Easy step - and it is the same fault the trophies
        // had: a footprint family sorted by worth does not know that some of
        // its members are things you are *given*. Thirty-one new components,
        // most of them one-cell rewards, turned a quiet wrongness into a loud
        // one: a creature was being handed the Mainspring's shape and the
        // astronomer's lens.
        .filter(|d| !crate::piece::is_event_only(d.name))
        // The mind lane's gear is worse than wrong on a creature: it banks a
        // pool the fight has no other use for, and a player cannot even meet
        // the piece until THE THRESHOLD is cleared. A creature wearing gear
        // nobody can buy is a creature wearing a stat line.
        .filter(|d| !crate::piece::touches_insight(d))
        // And the threshold's shelf, for the same reason one line up and the
        // same reason as event gear two lines above it: a footprint family
        // sorted by worth does not know that some of its members are things
        // you have to go and *buy at the bottom of a stair*. A5 appended five
        // helmet pieces and a creature stepped straight into one.
        .filter(|d| !crate::piece::is_threshold_stock(d.name))
        .collect();
    // Ordered by what a piece is worth to a *creature*, not to a shop.
    //
    // This sorted on `piece_rating`, which prices gear for a player who can
    // build a run around it. A drain rates well on that reasoning and does
    // nothing at all against a board banking no pools - so Francis's Insane
    // step traded a damage crest for Tithe Collector and came out easier than
    // his Hard step. A difficulty setting that lowers the difficulty is worse
    // than any mispricing it was correcting.
    family.sort_by(|a, b| {
        crate::rating::monster_value(a)
            .partial_cmp(&crate::rating::monster_value(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let Some(at) = family.iter().position(|d| d.name == here.name) else { return here.name };
    let want = (at as i32 + step).clamp(0, family.len() as i32 - 1) as usize;
    family[want].name
}

/// How many times the man at the top may double before the numbers stop
/// meaning anything.
///
/// `2^n` on an `i32` leaves the rails at 31 and leaves *sense* long before
/// that. Twelve is the ceiling because four thousand times Francis's strength
/// is already a one-tick kill against any board that can exist, and a run that
/// gets there has proved whatever it was proving. Past it the multiplier stops
/// rising rather than wrapping, which is the difference between a hard fight
/// and a negative one.
pub const MOST_DOUBLINGS: u32 = 12;

/// How many times a full board may grow looking for room for the setting's
/// item, two rows a time.
///
/// Bounded because a creature whose items are wider than the grid would never
/// fit one, and a loop that grew until it did would hang the fight rather than
/// lose it. Three is enough for every board in the game today; when it is not,
/// the count is the thing to look at rather than the ceiling to raise.
const GROW_ATTEMPTS: usize = 3;

/// Where each authored item starts and ends in a creature's gear list.
///
/// The `items` field is a partition by length; this is the same partition as
/// index pairs, which is what anything wanting to copy a whole item needs.
fn chunk_bounds(gear: &[GearPlacement], items: &'static [usize]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut at = 0usize;
    // An empty `items` means "work it out", which is right for the loose
    // boards - but one chunk spanning the whole gear list is not an item, it
    // is five slots in a trenchcoat, and anything copying a whole item would
    // reject it for straddling. Consecutive placements in the same slot is the
    // nearest honest reading.
    let chunks: Vec<usize> = if items.is_empty() {
        let mut runs: Vec<usize> = Vec::new();
        for (i, p) in gear.iter().enumerate() {
            if i > 0 && gear[i - 1].1 == p.1 {
                *runs.last_mut().expect("started above") += 1;
            } else {
                runs.push(1);
            }
        }
        runs
    } else {
        items.to_vec()
    };
    for take in chunks {
        let end = (at + take).min(gear.len());
        if end > at {
            out.push((at, end));
        }
        at = end;
    }
    out
}

impl MonsterSpec {
    /// This creature, `n` doublings harder.
    ///
    /// `n = 0` returns it exactly as written, which is the gate: nothing any
    /// player currently fights may move because this exists.
    ///
    /// **Health and strength double. The resistances do not.** The resists are
    /// percentages that piercing answers; taking 78 to 156 does not make a
    /// fighter twice as hard to hurt, it makes a number the rest of the engine
    /// would have to be defended against. Health and strength are plain
    /// quantities and multiply cleanly.
    ///
    /// The two dials pull opposite ways on the clock, and that is the point.
    /// More health is a longer fight; more strength is a shorter one. Doubling
    /// both leaves the fight roughly the length it was and moves who wins it -
    /// which is what "harder" has to mean in a game where `SUDDEN_DEATH_MS`
    /// owns everything past thirty seconds. Health alone would hand every
    /// fight past `n = 1` to the clock rather than to the boards.
    pub fn doubled(&self, n: u32) -> MonsterSpec {
        let mut out = *self;
        if n == 0 {
            return out;
        }
        let m = 1i64 << n.min(MOST_DOUBLINGS);
        let scale = |v: i32| (v as i64).saturating_mul(m).min(i32::MAX as i64) as i32;
        out.health = scale(self.health);
        out.strength = scale(self.strength);
        // Worth more every time it is worth more to beat.
        out.bounty = scale(self.bounty);
        out
    }

    /// Lay this monster's gear out in real slots. Returned so the interface can
    /// draw an enemy's board exactly the way it draws yours.
    /// This monster's gear, stepped for a difficulty.
    /// Where this creature stands on the ladder, if it stands on it at all.
    ///
    /// By name, because `LADDER` is spliced (`RUST_GOLEM` goes in by name) and
    /// a creature does not otherwise know its own rung. `ALTERNATES` answers
    /// `None`: a dungeon floor or an event's creature is not on the road, so
    /// the run-in rule below does not reach it.
    pub fn ladder_index(&self) -> Option<usize> {
        LADDER.iter().position(|m| m.name == self.name)
    }

    /// The step this creature's components take at a setting.
    ///
    /// Not simply `difficulty.gear_step()`: through The Hollow King, Hard and
    /// Insane are Medium exactly. The run-in is the same road whichever
    /// setting you picked, and the difficulty starts arguing after him.
    pub fn step_at(&self, difficulty: Difficulty) -> i32 {
        let softened = difficulty.gear_step() > 0
            && self.ladder_index().is_some_and(|i| i <= Difficulty::SAME_AS_MEDIUM_THROUGH);
        if softened {
            0
        } else {
            difficulty.gear_step()
        }
    }

    pub fn gear_at(&self, difficulty: Difficulty) -> Vec<GearPlacement> {
        let step = self.step_at(difficulty);
        self.gear
            .iter()
            .map(|&(name, slot, x, y, rot)| {
                (stepped_component(name, step + self.gear_offset), slot, x, y, rot)
            })
            .collect()
    }

    pub fn loadout(&self) -> (crate::piece::PieceRegistry, crate::loadout::Loadout) {
        self.loadout_at(Difficulty::Medium)
    }

    pub fn loadout_at(
        &self,
        difficulty: Difficulty,
    ) -> (crate::piece::PieceRegistry, crate::loadout::Loadout) {
        let gear = self.gear_at(difficulty);
        let mut reg = crate::piece::PieceRegistry::new();
        let mut loadout = crate::loadout::Loadout::new();
        // Seed names off the monster's own name so its gear is named too, and
        // named the same way every run.
        loadout.name_seed = self.name.bytes().fold(0xA5A5_u64, |a, b| {
            a.rotate_left(7) ^ b as u64
        });

        // Placed in item order, locking each one before the next goes down.
        //
        // The order matters and it is not cosmetic. An unlocked board
        // negotiates with itself: two items packed flush merge, or trade their
        // optional pieces to whichever core is nearest, and what comes out is
        // an over-full item that assembles into nothing. Locking each item as
        // it lands is what makes a tightly packed board hold - the same button
        // the player has, which the creatures use now too.
        let mut at = 0usize;
        let mut chunks: Vec<usize> = self.items.to_vec();
        if chunks.is_empty() {
            chunks = vec![gear.len()];
        }
        for take in chunks {
            let end = (at + take).min(gear.len());
            let mut touched: Vec<SlotKind> = Vec::new();
            for &(name, slot, x, y, rot) in &gear[at..end] {
                let Some(def) = crate::piece::CATALOG.iter().position(|d| d.name == name) else {
                    continue;
                };
                let id = reg.alloc(def);
                reg.set_rotation(id, rot);
                if loadout.can_place(&reg, id, slot, x, y).is_ok() {
                    loadout.slot_mut(slot).place(&reg, id, x, y);
                    if !touched.contains(&slot) {
                        touched.push(slot);
                    }
                }
            }
            for kind in touched {
                crate::loadout::lock_assembled_in(&mut loadout, &reg, kind);
            }
            at = end;
        }

        // And then the setting's own items, on top of the authored board.
        //
        // This is what M15 replaced the multipliers with. A harder setting
        // used to hand the creature better components and then multiply its
        // health and damage; now it hands it *another item*, which is the
        // same thing the player would do with the same grid.
        for _ in 0..self.extra_items_at(difficulty) {
            self.pack_one_more(&mut reg, &mut loadout, &gear, &chunk_bounds(&gear, self.items));
        }
        (reg, loadout)
    }

    /// How many items a setting adds to this creature's authored board.
    ///
    /// Mirrors `step_at`: nothing through The Hollow King, because the run-in
    /// is the same road whichever setting you picked. Hard adds one after him
    /// and Insane two, so Insane is Hard plus one rather than a separate
    /// authoring of the same board.
    pub fn extra_items_at(&self, difficulty: Difficulty) -> usize {
        if self.step_at(difficulty) <= 0 {
            return 0;
        }
        match difficulty {
            Difficulty::Hard => 1,
            Difficulty::Insane => 2,
            _ => 0,
        }
    }

    /// Copy one of this creature's own items into the free cells of its slot.
    ///
    /// Its own, rather than something drawn from the catalogue, for two
    /// reasons. A creature stays in character - a Burner gets more burning,
    /// not a random helmet - and the copy is known to assemble, because the
    /// original did. Nothing here can invent an item the author did not.
    ///
    /// Largest item first, then anchors in reading order, first fit wins. All
    /// of that is deterministic, which it has to be: `simulate_party` consults
    /// no RNG and this runs inside it.
    fn pack_one_more(
        &self,
        reg: &mut crate::piece::PieceRegistry,
        loadout: &mut crate::loadout::Loadout,
        gear: &[GearPlacement],
        bounds: &[(usize, usize)],
    ) {
        let mut order: Vec<&(usize, usize)> = bounds.iter().collect();
        // Biggest first, and ties broken by position so the order is total.
        order.sort_by_key(|&&(a, b)| (std::cmp::Reverse(b - a), a));
        let mut first: Option<(usize, usize, SlotKind)> = None;
        for &&(from, to) in &order {
            let pieces = &gear[from..to];
            let Some(&(_, slot, ..)) = pieces.first() else { continue };
            // One slot only: an item that straddles two grids is not an item.
            if pieces.iter().any(|&(_, s, ..)| s != slot) {
                continue;
            }
            if first.is_none() {
                first = Some((from, to, slot));
            }
            if self.seat_copy(reg, loadout, pieces, slot) {
                return;
            }
        }

        // Nothing fitted anywhere. Rather than drop the setting's item - which
        // would read as a difficulty that stops meaning anything against
        // exactly the creatures that are already hardest - give the densest
        // board a row and try the biggest item once more.
        //
        // Bounded to one retry on purpose. A creature that cannot take a copy
        // after two extra rows has items wider than the grid, and growing for
        // ever would hang the fight.
        if let Some((from, to, slot)) = first {
            for _ in 0..GROW_ATTEMPTS {
                let before = loadout.slot(slot).rows();
                loadout.grow_one(slot, 2);
                if loadout.slot(slot).rows() == before {
                    return;
                }
                if self.seat_copy(reg, loadout, &gear[from..to], slot) {
                    return;
                }
            }
        }
    }

    /// One attempt at seating a copy of `pieces` in `slot`, anchors in reading
    /// order. Split out so the grow-a-row retry is the same code as the first
    /// pass rather than a second copy of it.
    fn seat_copy(
        &self,
        reg: &mut crate::piece::PieceRegistry,
        loadout: &mut crate::loadout::Loadout,
        pieces: &[GearPlacement],
        slot: SlotKind,
    ) -> bool {
        let (ox, oy) = pieces.iter().fold((u8::MAX, u8::MAX), |(mx, my), &(_, _, x, y, _)| {
            (mx.min(x), my.min(y))
        });
        let rows = loadout.slot(slot).rows();
        for ay in 0..rows {
            for ax in 0..crate::slot::SLOT_W {
                let mut seated: Vec<crate::piece::PieceId> = Vec::new();
                let mut ok = true;
                for &(name, _, x, y, rot) in pieces {
                    let Some(def) = crate::piece::CATALOG.iter().position(|d| d.name == name)
                    else {
                        ok = false;
                        break;
                    };
                    let id = reg.alloc(def);
                    reg.set_rotation(id, rot);
                    let (nx, ny) = (ax + (x - ox), ay + (y - oy));
                    if nx >= crate::slot::SLOT_W
                        || ny >= rows
                        || loadout.can_place(reg, id, slot, nx, ny).is_err()
                    {
                        ok = false;
                        break;
                    }
                    loadout.slot_mut(slot).place(reg, id, nx, ny);
                    seated.push(id);
                }
                if ok && !seated.is_empty() {
                    crate::loadout::lock_assembled_in(loadout, reg, slot);
                    return true;
                }
                for id in seated {
                    loadout.slot_mut(slot).remove(id);
                }
            }
        }
        false
    }

    /// Build this monster's loadout and reduce it to stats plus activation
    /// profiles — the exact pipeline the player's gear goes through.
    pub fn outfit(&self) -> (Stats, Vec<ItemProfile>) {
        self.outfit_at(Difficulty::Medium)
    }

    pub fn outfit_at(&self, difficulty: Difficulty) -> (Stats, Vec<ItemProfile>) {
        let (reg, loadout) = self.loadout_at(difficulty);
        let mut stats = loadout.total_stats(&reg);
        // `total_stats` starts from the player's baseline; swap in the
        // monster's own.
        stats.health = stats.health - crate::stats::BASE_HEALTH + self.health;
        // Swap the player's baseline strength for the monster's own.
        stats.strength = stats.strength - crate::stats::BASE_STRENGTH + self.strength;
        stats.regen += self.regen;
        stats.mind_resist += self.mind_resist;
        stats.curse_resist += self.curse_resist;
        stats.physical_resist += self.physical_resist;
        stats.magic_resist += self.magic_resist;

        // Past a point on the road, everything knows how to get through
        // armour, and past a further point it knows how to shrug off somebody
        // else's piercing.
        //
        // A rule rather than fifty hand-set numbers. Half the deep ladder was
        // swinging for two hundred physical with no piercing at all, so a
        // player who stacked one resistance simply stopped being hit - and the
        // defence triangle, which is most of what the late catalogue is about,
        // did nothing from either side. Written here so it stays true when the
        // ladder is renumbered, which has happened three times.
        let depth = LADDER.iter().position(|m| m.name == self.name).map(|i| i + 1);
        if let Some(rung) = depth {
            if rung > PIERCE_FROM {
                // Enough to matter against a build that has committed to one
                // resistance, and never enough to make committing pointless.
                let p = (15 + (rung - PIERCE_FROM) as i32 * 2).min(55);
                // Relevant to what it actually deals: there is no sense
                // piercing magic resistance with a club.
                let phys: i32 =
                    stats.physical_damage + stats.strength + stats.rage;
                let magic: i32 = stats.magic_damage;
                if phys > 0 {
                    stats.physical_pierce += p;
                }
                if magic > 0 {
                    stats.magic_pierce += p;
                }
            }
            if rung > HARDEN_FROM {
                let h = (10 + (rung - HARDEN_FROM) as i32 * 2).min(45);
                stats.physical_harden += h;
                stats.magic_harden += h;
            }
        }
        let mut profiles = loadout.combat_items(&reg);
        // **The enchs, on the profiles and nowhere else.** Same door the
        // player's go through — `ench::apply` over the profiles, reading the
        // same `data/enchs.json` — because two answers to *what an ench does*
        // is exactly the thing this project has paid for six times. A creature
        // gets no `enched` flag and no beacon: those are read by rules a
        // character holds, and a creature holds none.
        if !self.enchs.is_empty() {
            let data = crate::data::enchs();
            crate::ench::apply(&mut profiles, &self.enchs_at(difficulty), &data);
        }
        (stats, profiles)
    }

    /// This creature's enchs, resolved to the piece ids `loadout_at` allocated.
    ///
    /// **An index into `gear`, turned into a `PieceId` the same way
    /// `loadout_at` allocates them** — in order, skipping any name the
    /// catalogue has not got, which is the one case where the two would
    /// otherwise drift. Nothing ships with such a name (`unassembled` refuses
    /// it and `tests/enemies.rs` runs over the whole ladder), and counting it
    /// honestly here costs one line and removes a way for a boss to wear
    /// somebody else's ench.
    pub fn enchs_at(&self, difficulty: Difficulty) -> Vec<crate::ench::Ench> {
        let gear = self.gear_at(difficulty);
        let mut id_of = Vec::with_capacity(gear.len());
        let mut next = 0u32;
        for &(name, _, _, _, _) in gear.iter() {
            if crate::piece::CATALOG.iter().any(|d| d.name == name) {
                id_of.push(Some(crate::piece::PieceId(next)));
                next += 1;
            } else {
                id_of.push(None);
            }
        }
        self.enchs
            .iter()
            .filter_map(|&(id, at)| {
                id_of.get(at).copied().flatten().map(|on| crate::ench::Ench {
                    on,
                    id: id.to_string(),
                    active: true,
                })
            })
            .collect()
    }

    /// Which of its gear failed to assemble, if any. A monster whose loadout
    /// silently falls apart is a monster that does nothing.
    pub fn unassembled(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for &(name, _, _, _, _) in self.gear {
            if !crate::piece::CATALOG.iter().any(|d| d.name == name) {
                missing.push(format!("{}: no such component", name));
            }
        }
        // Built through `loadout_at`, not by re-placing the gear here. The two
        // are not the same board: `loadout_at` locks each item as it lands,
        // and on a tightly packed board that is the difference between three
        // items and one over-full one that assembles into nothing. Checking a
        // board the creature never fights in is worse than not checking.
        let (reg, loadout) = self.loadout();
        for kind in SlotKind::ALL {
            for item in loadout.report(&reg, kind).items {
                if item.assembled {
                    continue;
                }
                // Some gear is better left in bits. A piece whose effect is
                // gated on `When::NotAssembled` - the Vast Tapestry's +550
                // health while it stays loose - is doing its whole job sitting
                // there unfinished, so calling it a broken loadout is calling a
                // deliberate build a typo. An enchantment is loose for the same
                // reason: no recipe names its kind.
                let on_purpose = item.pieces.iter().all(|&p| {
                    let def = reg.def(p);
                    def.kind.is_enchantment()
                        || def
                            .effect
                            .as_ref()
                            .is_some_and(|e| matches!(e.when, crate::piece::When::NotAssembled))
                });
                if !on_purpose {
                    missing.push(format!("{} item: {}", kind.name(), item.status));
                }
            }
        }
        missing
    }
}

/// How much harder than a baseline run this is.
///
/// The scale is what the player picks - 1x, 3x, 9x, 27x - and it is the
/// monster's total effectiveness that gets multiplied, not any one stat.
/// Splitting it evenly between staying alive and hitting back means each side
/// takes the square root, so their product is the factor you chose: Insane is
/// a monster about 5.2 times tougher and 5.2 times deadlier, which is 27 times
/// the fight.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Insane,
}

impl Difficulty {
    /// The line under the heading on the screen where a setting is picked.
    ///
    /// Beside `Mode::WHAT_THE_CHOICE_IS` and for the same reason. The one that
    /// was here said "Bigger numbers mean tougher, meaner monsters. Medium is
    /// the fight the game was built around" - which names an option standing
    /// directly below it, in a card that already says "the intended fight" on
    /// its own face, and which is wrong about the mechanism besides: most of a
    /// setting is `gear_step`, and the numbers are what is left over.
    pub const WHAT_THE_CHOICE_IS: &'static str =
        "Set once, for the whole run. It steps the gear the opposition wears \
         before it touches any of its numbers.";

    pub const ALL: &'static [Difficulty] =
        &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Insane];

    /// The advertised multiple: how many times as effective the opposition is.
    pub fn factor(self) -> f32 {
        match self {
            Difficulty::Easy => 0.5,
            Difficulty::Medium => 1.0,
            Difficulty::Hard => 3.0,
            Difficulty::Insane => 9.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Medium => "MEDIUM",
            Difficulty::Hard => "HARD",
            Difficulty::Insane => "INSANE",
        }
    }

    pub fn label(self) -> String {
        let f = self.factor();
        if (f - f.round()).abs() < 0.01 {
            format!("{}x", f as i32)
        } else {
            format!("{}x", f)
        }
    }

    /// Medium is the way the game is meant to be played; the others are set
    /// against it.
    pub fn is_default(self) -> bool {
        matches!(self, Difficulty::Medium)
    }

    /// How far up its own kind each of a monster's components is swapped.
    ///
    /// This is where most of a difficulty setting now lives. Medium is the
    /// gear as written; Hard and Insane trade each component for a better one
    /// of the same kind, and Easy trades down. A Bog Toad on Insane is not the
    /// Medium toad with bigger numbers - it is a toad in better armour.
    pub fn gear_step(self) -> i32 {
        match self {
            Difficulty::Easy => -1,
            Difficulty::Medium => 0,
            Difficulty::Hard => 1,
            Difficulty::Insane => 2,
        }
    }

    /// What is left for raw stats to carry, once gear has done its part.
    ///
    /// **Nothing, above Medium.** This was `factor().powf(0.25)` for every
    /// setting, kept as a floor because a component at the top of its kind has
    /// no better version to swap to. M15 answers that differently: a creature
    /// that cannot be given a better component is given *another item*, which
    /// is a board decision rather than a multiplier, and the whole point of
    /// the milestone is that a harder setting is a harder board.
    ///
    /// Easy keeps its half, because softening the run-in is not the same
    /// question and nothing asked for it to change.
    pub fn each_way(self) -> f32 {
        match self {
            Difficulty::Easy => self.factor().powf(0.25),
            _ => 1.0,
        }
    }

    /// Standing bonuses the opposition gets on top of the raw scaling. These
    /// are the prototype for class passives: a named rule that edits a
    /// combatant's stats once, at the start of the fight.
    /// Standing bonuses the opposition gets on top of the raw scaling.
    ///
    /// Hard and Insane carried `Warded` and `Relentless` on top of `Hardened`.
    /// Both are gone at M15: they are the same crude lever `each_way` was, a
    /// rule handed to the creature rather than a board it is standing in.
    /// Medium keeps `Hardened` because Medium is the game as written and this
    /// milestone is about what the other settings do differently.
    pub fn passives(self) -> &'static [Passive] {
        match self {
            Difficulty::Easy => &[],
            _ => &[Passive::Hardened],
        }
    }

    /// The last rung that fights the same at every setting above Easy.
    ///
    /// `LADDER[14]` is The Hollow King, rung 15 spoken. Up to and including
    /// him, Hard and Insane are Medium exactly - same components, same items,
    /// same stats. A player who picks a harder setting should meet the game
    /// before it starts arguing, and fifteen rungs is where the road's own
    /// shape says that stops.
    pub const SAME_AS_MEDIUM_THROUGH: usize = 14;
}

/// What kind of harm an attack is, so the matching defences apply.
///
/// There is no untyped option on purpose. Every number a piece of gear deals
/// is physical, magic, or mind, which is what makes resistance worth buying:
/// a defence that half the game ignored would be a coin flip at the shop.
/// Curse burn is the one thing that still bypasses all of it, and it answers
/// to curse resistance instead.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum DamageType {
    #[default]
    Physical,
    Magic,
}

impl DamageType {
    pub fn name(self) -> &'static str {
        match self {
            DamageType::Physical => "physical damage",
            DamageType::Magic => "magic damage",
        }
    }
}

/// A standing rule that edits a combatant before the fight starts.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Passive {
    /// Knits itself back together: regeneration every second.
    Hardened,
    /// Turns aside the mind and the curse alike.
    Warded,
    /// Never stops coming: everything it does lands sooner.
    Relentless,
}

impl Passive {
    pub fn name(self) -> &'static str {
        match self {
            Passive::Hardened => "Hardened",
            Passive::Warded => "Warded",
            Passive::Relentless => "Relentless",
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Passive::Hardened => "heals 4 a second",
            Passive::Warded => "shrugs off 40% of mind and curses, 20% of blows and spells",
            Passive::Relentless => "all its gear comes round a quarter sooner",
        }
    }
}

/// The original opponent, named because several tests predate the ladder.
pub const RUST_GOLEM: MonsterSpec = MonsterSpec {
    name: "Rust Golem",
    health: 300,
    strength: 13,
    regen: 0,
    mind_resist: 0,
    physical_resist: 7,
        magic_resist: 7,
        curse_resist: 0,
    attacks: &[],
    gear: &[
        ("Chained Codex", SlotKind::Weapon, 0, 0, 1),
        ("Kingsblood Ink", SlotKind::Weapon, 3, 0, 0),
        ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
        ("Bloodstone Bead", SlotKind::Weapon, 4, 1, 1),
    ],
    gear_offset: 0,
    bounty: 10,
    sprite: MonsterSprite::Golem,
    rank: Rank::Ordinary,
    drops: &[],
    items: &[4],
    enchs: &[],
};

/// The monster ladder, easiest first.
///
/// Difficulty is set by what each one is *wearing*, not by hand-tuned numbers:
/// they buy from the same catalogue and assemble by the same rules. Making a
/// monster harder means giving it better gear.
/// What a spell costs to cast at full strength, and what it lands for when
/// there is nothing to pay with.
///
/// An unpaid spell is not cancelled - it still fires, which matters, because a
/// build that runs dry should get weaker rather than stop.
pub const SPELL_MANA_COST: i32 = 3;

/// What one turn of the spin banks, in hundredths of power.
///
/// Ten, which is the `+0.1x` the plan asks for expressed in the unit `power`
/// already uses — 100 is a plain multiple of one, so ten is a tenth.
///
/// **Uncapped, because the spend is the cap.** A slow item stacks more and
/// fires less and a fast one the other way round, which is a trade rather than
/// a ceiling, and a ceiling would have had to be tuned against every cadence
/// in the game.
pub const SPIN_PCT_PER_TURN: i32 = 10;

/// How long one turn takes.
pub const SPIN_EVERY_MS: u32 = 1_000;
pub const WEAK_CAST_PCT: i32 = 45;

/// What a paid cast lands for.
///
/// Playtesters found spells universally weak and crystal balls not worth the
/// room they take. The reason was that paying for a spell bought you nothing
/// except not being weakened - the ceiling was the number printed on the
/// piece, and that number had to compete with a blade that swings for it
/// every time and never asks for mana. So paying now doubles the cast. The
/// shop price is unchanged on purpose: the point is to make casters worth
/// their slot, not to make them cost more.
pub const EMPOWERED_CAST_PCT: i32 = 200;

/// How many of its spells a crystal ball casts each time it comes round.
///
/// Two, always. A class can raise it; nothing lowers it.
pub const BALL_VOICES: u32 = 2;

/// What one stack of Spellblade adds to a physical hit, in power-hundredths.
///
/// Half a multiplier. Flat, unconditional, and the physical lane's answer to
/// empowerment - which buys 0.05x a stack per point of mana, so it passes this
/// at ten mana and keeps going. The twin is the better opening and the worse
/// ceiling, which is the trade the two lanes are meant to have.
pub const SPELLBLADE_POWER: i32 = 50;

/// What one stack of Deflection turns off an incoming physical hit.
///
/// Flat ten, ahead of armour, on the same terms: the mana shield takes one
/// point per point of mana, so it passes this at ten mana as well. The two
/// numbers are deliberately the same crossing point.
pub const DEFLECTION_FLAT: i32 = 10;

/// What a stack of Dread divides the Insight it stands on by.
///
/// Mind damage gains `dread x insight / DREAD_DIVISOR` per hit. Two, so a
/// stack against twenty Insight is worth ten a hit - which is empowerment's
/// arithmetic seen from the other end, and the number A3 leaves to be tuned.
pub const DREAD_DIVISOR: i32 = 2;

/// The rung past which everything on the road pierces, and past which it also
/// hardens. Both are exclusive: rung 30 does not, rung 31 does.
pub const PIERCE_FROM: usize = 30;
pub const HARDEN_FROM: usize = 40;

pub const LADDER: &[MonsterSpec] = &[
    MonsterSpec {
        name: "Cave Rat",
        health: 55,
        strength: 2,
        regen: 0,
        mind_resist: 0,
        physical_resist: 1,
        magic_resist: 1,
        curse_resist: 0,
        // No gear at all — it just has teeth.
        attacks: &[MonsterAttack::hit("bite", 900, 4)],
        gear: &[],
        gear_offset: 0,
        bounty: 6,
        sprite: MonsterSprite::Rat,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    MonsterSpec {
        name: "Bog Toad",
        health: 110,
        strength: 5,
        regen: 1,
        mind_resist: 0,
        physical_resist: 2,
        magic_resist: 2,
        curse_resist: 0,
        attacks: &[],
        // Fast hands, and something sharp.
        gear: &[
            ("Godsteel Haft", SlotKind::Weapon, 0, 0, 1),
            ("Iron Blade", SlotKind::Weapon, 0, 1, 1),
            ("Worldweave Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 8,
    sprite: MonsterSprite::Toad,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Bone Archer",
        health: 120,
        strength: 5,
        regen: 0,
        mind_resist: 0,
        physical_resist: 3,
        magic_resist: 3,
        curse_resist: 0,
        attacks: &[],
        // Fast hands, and something sharp.
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Cometfall", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 4, 1, 1),
            ("Mana Loom", SlotKind::Chest, 0, 0, 0),
            ("Wrathbreaker", SlotKind::Chest, 1, 2, 2),
        ],
        gear_offset: 0,
        bounty: 9,
    sprite: MonsterSprite::Archer,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 2],
        enchs: &[],
    },
    RUST_GOLEM,
    MonsterSpec {
        name: "Frost Wisp",
        health: 150,
        strength: 6,
        regen: 0,
        mind_resist: 0,
        physical_resist: 3,
        magic_resist: 3,
        curse_resist: 25,
        attacks: &[],
        // Fast hands, and something sharp.
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Unmaking", SlotKind::Weapon, 3, 0, 1),
            ("Hollow Lance", SlotKind::Weapon, 5, 0, 0),
            ("Worldweave Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 12,
    sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Plague Hound",
        health: 190,
        strength: 8,
        regen: 0,
        mind_resist: 0,
        physical_resist: 4,
        magic_resist: 4,
        curse_resist: 0,
        attacks: &[],
        // Fast hands, and something sharp.
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Sanctified Material", SlotKind::Gloves, 0, 0, 1),
            ("Sovereign Mold", SlotKind::Gloves, 2, 0, 0),
        ],
        gear_offset: 0,
        bounty: 14,
    sprite: MonsterSprite::Hound,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Iron Warden",
        health: 340,
        strength: 14,
        regen: 2,
        mind_resist: 20,
        physical_resist: 8,
        magic_resist: 8,
        curse_resist: 20,
        attacks: &[],
        // Halfway up the ladder, and the first opponent whose armour is the
        // point: every one of the 48 chest cells is covered, by three separate
        // chestpieces, so it soaks far more than anything before it. The rest
        // of its gear is deliberately ordinary - one weapon, one glove, one
        // Made to be hit, and it hits back harder than it hits.
        gear: &[
            ("Zealot's Haft", SlotKind::Weapon, 0, 0, 1),
            ("Gluttonous Fang", SlotKind::Weapon, 3, 0, 1),
            ("Bonesaw", SlotKind::Weapon, 1, 1, 0),
            ("Ossuary Frame", SlotKind::Helmet, 0, 0, 1),
            ("Lonely Plating", SlotKind::Helmet, 2, 0, 0),
            ("Grove Base", SlotKind::Chest, 1, 6, 0),
            ("Rag Layer", SlotKind::Chest, 3, 7, 0),
        ],
        gear_offset: 0,
        bounty: 22,
        sprite: MonsterSprite::Warden,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Iron Sentinel",
        health: 240,
        strength: 10,
        regen: 0,
        mind_resist: 0,
        physical_resist: 6,
        magic_resist: 6,
        curse_resist: 0,
        attacks: &[],
        // Made to be hit, and it hits back harder than it hits.
        gear: &[
            ("Zealot's Haft", SlotKind::Weapon, 0, 0, 1),
            ("Gluttonous Fang", SlotKind::Weapon, 3, 0, 1),
            ("Ossuary Frame", SlotKind::Helmet, 0, 0, 1),
            ("Lonely Plating", SlotKind::Helmet, 2, 0, 0),
            ("Heartwood Base", SlotKind::Chest, 0, 0, 0),
            ("Vast Tapestry", SlotKind::Chest, 0, 2, 0),
        ],
        gear_offset: 0,
        bounty: 24,
    sprite: MonsterSprite::Sentinel,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Whisperling",
        health: 160,
        strength: 7,
        regen: 0,
        mind_resist: 0,
        physical_resist: 4,
        magic_resist: 4,
        curse_resist: 0,
        attacks: &[],
        // Made to be hit, and it hits back harder than it hits.
        gear: &[
            ("Scrying Orb", SlotKind::Weapon, 0, 0, 0),
            ("Echo Sigil", SlotKind::Weapon, 3, 0, 0),
            ("Mirror Ward", SlotKind::Weapon, 4, 1, 0),
            ("Rime Nova", SlotKind::Weapon, 0, 3, 0),
            ("Bone Frame", SlotKind::Helmet, 0, 0, 0),
            ("Braced Plating", SlotKind::Helmet, 3, 0, 0),
            ("Bone Frame", SlotKind::Helmet, 0, 1, 2),
            ("Braced Plating", SlotKind::Helmet, 3, 2, 0),
            ("Hexweave Shroud", SlotKind::Chest, 0, 0, 0),
            ("Ironbark Layer", SlotKind::Chest, 3, 0, 0),
            ("Hexweave Shroud", SlotKind::Chest, 3, 2, 0),
            ("Scale Layer", SlotKind::Chest, 0, 3, 0),
            ("Steel Material", SlotKind::Greaves, 0, 0, 1),
            ("Warded Sabatons", SlotKind::Greaves, 3, 0, 3),
            ("Steel Material", SlotKind::Greaves, 4, 1, 0),
            ("Warded Sabatons", SlotKind::Greaves, 1, 2, 3),
        ],
        gear_offset: 0,
        bounty: 26,
    sprite: MonsterSprite::Wraith,
        rank: Rank::Mini,
        drops: &["Asker's Monocle"],
        items: &[4, 2, 2, 2, 2, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Warded Idol",
        health: 280,
        strength: 12,
        regen: 2,
        mind_resist: 0,
        physical_resist: 7,
        magic_resist: 7,
        curse_resist: 55,
        attacks: &[],
        // Made to be hit, and it hits back harder than it hits.
        gear: &[
            ("Executioner's Haft", SlotKind::Weapon, 0, 0, 1),
            ("Iron Blade", SlotKind::Weapon, 0, 1, 1),
            ("Reliquary Frame of Nine", SlotKind::Helmet, 0, 0, 0),
            ("Warded Plating", SlotKind::Helmet, 3, 0, 0),
            ("Voidsilk Base", SlotKind::Chest, 0, 0, 0),
            ("Vast Tapestry", SlotKind::Chest, 0, 3, 0),
        ],
        gear_offset: 0,
        bounty: 30,
    sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Mirror Fiend",
        health: 250,
        strength: 11,
        regen: 0,
        mind_resist: 45,
        physical_resist: 6,
        magic_resist: 6,
        curse_resist: 20,
        attacks: &[],
        gear: &[
            ("Grand Grimoire", SlotKind::Weapon, 0, 0, 0),
            ("Starlit Ink", SlotKind::Weapon, 3, 0, 0),
            ("Unmaking", SlotKind::Weapon, 2, 1, 3),
            ("Ossuary Frame", SlotKind::Helmet, 0, 0, 1),
            ("Scarred Plating", SlotKind::Helmet, 2, 0, 0),
            ("Heartwood Base", SlotKind::Chest, 0, 0, 0),
            ("Verdant Weave", SlotKind::Chest, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 34,
    sprite: MonsterSprite::Fiend,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Rust Colossus",
        // gear rating 47
        health: 800,
        strength: 28,
        regen: 2,
        mind_resist: 20,
        physical_resist: 20,
        magic_resist: 15,
        curse_resist: 20,
        attacks: &[],
        gear: &[
            ("Mage's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Bloomcap", SlotKind::Helmet, 3, 0, 0),
            ("Hide Base", SlotKind::Chest, 0, 0, 0),
            ("Berserker's Plate", SlotKind::Chest, 3, 0, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 0, 0, 0),
            ("Featherweight Mold", SlotKind::Gloves, 2, 0, 0),
            ("Mage's Sandals", SlotKind::Greaves, 0, 0, 0),
            ("Striding Mold", SlotKind::Greaves, 2, 0, 0),
            ("Apprentice's Primer", SlotKind::Weapon, 0, 0, 0),
            ("Bloodletter's Ink", SlotKind::Weapon, 2, 0, 0),
            ("Warding Sigil", SlotKind::Weapon, 4, 0, 0),
            ("Duskweave Material", SlotKind::Gloves, 2, 1, 0),
            ("Empowering Mold", SlotKind::Gloves, 2, 2, 0),
        ],
        gear_offset: 0,
        bounty: 44,
        sprite: MonsterSprite::Colossus,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    MonsterSpec {
        name: "Ashen Marshal",
        // gear rating 99
        health: 930,
        strength: 31,
        regen: 2,
        mind_resist: 23,
        physical_resist: 23,
        magic_resist: 18,
        curse_resist: 23,
        attacks: &[],
        gear: &[
            ("Apprentice's Primer", SlotKind::Weapon, 0, 0, 0),
            ("Starlit Ink", SlotKind::Weapon, 2, 0, 0),
            ("Unmaking", SlotKind::Weapon, 4, 0, 2),
            ("Ossuary Frame", SlotKind::Helmet, 0, 0, 1),
            ("Scarred Plating", SlotKind::Helmet, 2, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 4, 0, 0),
            ("Heartwood Base", SlotKind::Chest, 0, 0, 0),
            ("Berserker's Plate", SlotKind::Chest, 3, 0, 0),
            ("Polished Orb", SlotKind::Weapon, 0, 6, 0),
            ("Crimson Alignment", SlotKind::Weapon, 2, 4, 0),
            ("Last Rite", SlotKind::Weapon, 0, 4, 1),
            ("Sympathetic Bloom", SlotKind::Weapon, 3, 5, 0),
            ("Wildgrowth", SlotKind::Weapon, 2, 5, 0),
            ("Fumbler's Mold", SlotKind::Greaves, 2, 3, 0),
            ("Ashwoven Material", SlotKind::Greaves, 2, 2, 0),
        ],
        gear_offset: 0,
        bounty: 75,
        sprite: MonsterSprite::Marshal,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 3, 2, 5, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Grave Chorus",
        // gear rating 154
        health: 950,
        strength: 31,
        regen: 2,
        mind_resist: 26,
        physical_resist: 26,
        magic_resist: 21,
        curse_resist: 26,
        attacks: &[],
        gear: &[
            ("Chained Codex", SlotKind::Weapon, 0, 0, 1),
            ("Kingsblood Ink", SlotKind::Weapon, 3, 0, 0),
            ("Cometfall", SlotKind::Weapon, 3, 1, 2),
            ("Bloodstone Bead", SlotKind::Weapon, 0, 2, 0),
            ("Rootwoven Material", SlotKind::Greaves, 0, 0, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 1, 0),
            ("Plaguewalkers", SlotKind::Greaves, 2, 2, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 4, 2, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 3, 0),
        ],
        gear_offset: -2,
        bounty: 80,
        sprite: MonsterSprite::Choir,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Hollow King",
        health: 400,
        strength: 18,
        regen: 3,
        mind_resist: 30,
        physical_resist: 10,
        magic_resist: 5,
        curse_resist: 30,
        attacks: &[],
        // It does not need to reach you.
        gear: &[
            ("Timeworn Orb", SlotKind::Weapon, 0, 0, 0),
            ("Unmaking", SlotKind::Weapon, 3, 0, 1),
            ("Last Rite", SlotKind::Weapon, 4, 0, 1),
            ("Emberburst", SlotKind::Weapon, 1, 1, 0),
            ("Ember Alignment", SlotKind::Weapon, 0, 2, 3),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 0, 0),
            ("Witherroot", SlotKind::Greaves, 2, 0, 3),
            ("Warmed Material", SlotKind::Greaves, 4, 0, 0),
            ("Witherroot", SlotKind::Greaves, 2, 1, 1),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 2, 0),
            ("Witherroot", SlotKind::Greaves, 2, 3, 3),
            ("Thornweald Grip", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 4, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 1, 2),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 2, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 3, 0),
        ],
        gear_offset: 0,
        bounty: 89,
    sprite: MonsterSprite::King,
        rank: Rank::Boss,
        drops: &["Henpeck's Cell Keys"],
        items: &[5, 2, 2, 2, 2, 2, 2],
        enchs: &[],
    },
    // The buyer Henpeck names as he goes down. The player has been buying
    // gear off this one since rung one without ever asking where a shop that
    // size gets its stock.
    MonsterSpec {
        name: "The Curator",
        health: 640,
        strength: 24,
        regen: 4,
        mind_resist: 34,
        physical_resist: 16,
        magic_resist: 18,
        curse_resist: 32,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Ember Alignment", SlotKind::Weapon, 3, 1, 1),
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Gravewalker Mold", SlotKind::Greaves, 2, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 1, 1, 0),
            ("Rootwoven Material", SlotKind::Greaves, 3, 1, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Greaves, 1, 3, 0),
            ("Rootwoven Material", SlotKind::Greaves, 0, 2, 1),
            ("Gravewalker Mold", SlotKind::Greaves, 0, 5, 0),
        ],
        gear_offset: -2,
        bounty: 93,
        sprite: MonsterSprite::Curator,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 3, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Salt Idol",
        // gear rating 208
        health: 1190,
        strength: 37,
        regen: 3,
        mind_resist: 29,
        physical_resist: 29,
        magic_resist: 24,
        curse_resist: 29,
        attacks: &[],
        gear: &[
            ("Zealot's Haft", SlotKind::Weapon, 0, 0, 1),
            ("Gluttonous Fang", SlotKind::Weapon, 3, 0, 1),
            ("Sunderer", SlotKind::Weapon, 0, 1, 1),
            ("Bloodstone Bead", SlotKind::Weapon, 5, 0, 1),
            ("Oathstone Bead", SlotKind::Weapon, 4, 1, 1),
            ("Rootwoven Material", SlotKind::Greaves, 0, 0, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 1, 0),
            ("Ashwoven Material", SlotKind::Greaves, 5, 0, 1),
            ("Witherroot", SlotKind::Greaves, 3, 2, 3),
            ("Consecrated Plating", SlotKind::Greaves, 1, 3, 0),
            ("Rootwoven Material", SlotKind::Greaves, 0, 3, 1),
            ("Pilgrim Sole", SlotKind::Greaves, 1, 5, 0),
        ],
        gear_offset: 0,
        bounty: 98,
        sprite: MonsterSprite::Salt,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Pale Twin",
        // gear rating 259
        health: 1190,
        strength: 36,
        regen: 3,
        mind_resist: 32,
        physical_resist: 33,
        magic_resist: 28,
        curse_resist: 32,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Ember Alignment", SlotKind::Weapon, 4, 2, 2),
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Pilgrim Sole", SlotKind::Greaves, 2, 0, 0),
            ("Plaguewalkers", SlotKind::Greaves, 4, 0, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Greaves, 1, 2, 0),
            ("Sanctified Material", SlotKind::Greaves, 0, 3, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 2, 4, 0),
            ("Consecrated Plating", SlotKind::Greaves, 4, 4, 0),
        ],
        gear_offset: -2,
        bounty: 107,
        sprite: MonsterSprite::Twin,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 2, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Ruin Hound",
        // gear rating 311
        health: 1450,
        strength: 43,
        regen: 3,
        mind_resist: 35,
        physical_resist: 36,
        magic_resist: 31,
        curse_resist: 35,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Ember Alignment", SlotKind::Weapon, 3, 1, 1),
            ("Rootwoven Material", SlotKind::Greaves, 0, 0, 0),
            ("Witherroot", SlotKind::Greaves, 3, 0, 3),
            ("Rootwoven Material", SlotKind::Greaves, 0, 1, 0),
            ("Gravewalker Mold", SlotKind::Greaves, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Greaves, 3, 2, 0),
            ("Rootwoven Material", SlotKind::Greaves, 5, 0, 1),
            ("Anchored Sole", SlotKind::Greaves, 4, 3, 3),
            ("Consecrated Plating", SlotKind::Greaves, 2, 4, 0),
            ("Rootwoven Material", SlotKind::Greaves, 0, 3, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 6, 0),
        ],
        gear_offset: 0,
        bounty: 116,
        sprite: MonsterSprite::RuinHound,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 2, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Bone Cantor",
        // gear rating 368
        health: 1580,
        strength: 46,
        regen: 4,
        mind_resist: 38,
        physical_resist: 39,
        magic_resist: 34,
        curse_resist: 38,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 4, 1, 1),
            ("Ember Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 2, 0),
            ("Rootwoven Material", SlotKind::Greaves, 2, 2, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 2, 3, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Sovereign Mold", SlotKind::Gloves, 2, 0, 0),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Ring of Embers", SlotKind::Gloves, 2, 1, 0),
            ("Ashwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Sovereign Mold", SlotKind::Gloves, 3, 1, 2),
        ],
        gear_offset: 0,
        bounty: 125,
        sprite: MonsterSprite::Cantor,
        rank: Rank::Mini,
        drops: &["Toolwright's Grip"],
        items: &[5, 3, 2, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Ember Wisp",
        // gear rating 420
        health: 1710,
        strength: 49,
        regen: 4,
        mind_resist: 41,
        physical_resist: 42,
        magic_resist: 37,
        curse_resist: 41,
        attacks: &[],
        gear: &[
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Sapling Mold", SlotKind::Greaves, 2, 0, 1),
            ("Sanctified Material", SlotKind::Greaves, 4, 0, 1),
            ("Sapling Mold", SlotKind::Greaves, 2, 1, 1),
            ("Braced Plating", SlotKind::Greaves, 1, 2, 0),
            ("Sanctified Material", SlotKind::Greaves, 4, 1, 3),
            ("Grave-Iron Mold", SlotKind::Greaves, 3, 2, 2),
            ("Sanctified Material", SlotKind::Greaves, 0, 3, 0),
            ("Sapling Mold", SlotKind::Greaves, 2, 4, 1),
            ("Mage's Wrapping", SlotKind::Gloves, 0, 0, 0),
            ("Deft Mold", SlotKind::Gloves, 2, 0, 0),
            ("Sanctified Material", SlotKind::Gloves, 4, 0, 1),
            ("Deft Mold", SlotKind::Gloves, 2, 1, 0),
            ("Iron Band", SlotKind::Gloves, 5, 1, 0),
            ("Oathring", SlotKind::Gloves, 2, 2, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 0, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 2, 3, 0),
        ],
        gear_offset: 0,
        bounty: 134,
        sprite: MonsterSprite::Ember,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 3, 2, 2, 2, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Slag Warden",
        // gear rating 480
        health: 1840,
        strength: 52,
        regen: 4,
        mind_resist: 44,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 44,
        attacks: &[],
        gear: &[
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Sapling Mold", SlotKind::Greaves, 2, 0, 1),
            ("Witch's Claw", SlotKind::Greaves, 4, 0, 0),
            ("Runner's Mold", SlotKind::Greaves, 2, 1, 0),
            ("Braced Plating", SlotKind::Greaves, 0, 2, 0),
            ("Sanctified Material", SlotKind::Greaves, 4, 2, 3),
            ("Sapling Mold", SlotKind::Greaves, 2, 3, 1),
            ("Braced Plating", SlotKind::Greaves, 1, 4, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 0, 1),
            ("Deft Mold", SlotKind::Gloves, 3, 0, 0),
            ("Sanctified Material", SlotKind::Gloves, 4, 0, 3),
            ("Deft Mold", SlotKind::Gloves, 2, 1, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 1, 3),
            ("Deft Mold", SlotKind::Gloves, 2, 2, 0),
            ("Tin Band", SlotKind::Gloves, 4, 2, 0),
            ("Oathring", SlotKind::Gloves, 5, 2, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 3, 1),
            ("Deft Mold", SlotKind::Gloves, 2, 3, 0),
        ],
        gear_offset: 0,
        bounty: 143,
        sprite: MonsterSprite::Slag,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 3, 3, 2, 2, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Gearwright",
        health: 540,
        strength: 19,
        regen: 3,
        mind_resist: 40,
        physical_resist: 18,
        magic_resist: 13,
        curse_resist: 40,
        attacks: &[],
        // The end of the ladder: every slot filled with the best-rated legal
        // item the catalogue allows, found by the packing search in
        // It takes your time before it takes anything else.
        gear: &[
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 0, 0),
            ("Sunderer", SlotKind::Weapon, 3, 0, 1),
            ("Oathstone Bead", SlotKind::Weapon, 0, 1, 1),
            ("Fury Sigil", SlotKind::Weapon, 2, 1, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 0, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 0, 1),
            ("Sevenleague Boots", SlotKind::Gloves, 4, 1, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 2, 1),
            ("Titan's Grip", SlotKind::Greaves, 0, 0, 0),
            ("Worldstrider Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 2, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 2, 2, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 0, 4, 0),
            ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
            ("Scrying Lens", SlotKind::Helmet, 3, 0, 0),
            ("Bloomcap", SlotKind::Helmet, 3, 1, 3),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 3, 0),
        ],
        gear_offset: -2,
        bounty: 152,
        sprite: MonsterSprite::Gearwright,
        rank: Rank::Mini,
        drops: &["Kaklon's Patent"],
        items: &[4, 2, 2, 3, 2, 3, 2],
        enchs: &[],
    },
    // ---- past the Gearwright ----
    //
    // Twenty more, climbing steadily. Each wears a loadout built from layouts
    // already verified to assemble, so the ladder can grow without every new
    // rung needing the packing search run over it again.
    MonsterSpec {
        name: "Crowned Hollow",
        // gear rating 532
        health: 1970,
        strength: 55,
        regen: 5,
        mind_resist: 47,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 47,
        attacks: &[],
        gear: &[
            ("Leaden Tome", SlotKind::Weapon, 0, 0, 0),
            ("Deepwater Ink", SlotKind::Weapon, 3, 0, 1),
            ("Slash and Burn", SlotKind::Weapon, 3, 1, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 0, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 0, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Ring of Roots", SlotKind::Gloves, 4, 1, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 2, 1),
            ("Gripping Mold", SlotKind::Gloves, 3, 2, 0),
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Trailworn Sole", SlotKind::Greaves, 2, 0, 1),
            ("Consecrated Plating", SlotKind::Greaves, 4, 0, 0),
            ("Rootwoven Material", SlotKind::Greaves, 1, 1, 1),
            ("Sapling Mold", SlotKind::Greaves, 2, 2, 1),
            ("Consecrated Plating", SlotKind::Greaves, 4, 2, 0),
            ("Witch's Hat", SlotKind::Helmet, 0, 0, 2),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Warding Plate", SlotKind::Helmet, 2, 2, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 4, 2, 1),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
        ],
        gear_offset: 0,
        bounty: 161,
        sprite: MonsterSprite::Crown,
        rank: Rank::Mini,
        drops: &["Eighth Ray Crown"],
        items: &[3, 4, 2, 3, 3, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Cog Priest",
        // gear rating 588
        health: 2100,
        strength: 58,
        regen: 5,
        mind_resist: 50,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 50,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Greaves, 0, 0, 0),
            ("Worldstrider Sole", SlotKind::Greaves, 3, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 1, 0),
            ("Worldstrider Sole", SlotKind::Greaves, 2, 1, 2),
            ("Consecrated Plating", SlotKind::Greaves, 0, 3, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 2, 3, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 0, 5, 0),
            ("Consecrated Plating", SlotKind::Greaves, 3, 5, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 0, 1),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 0, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Piercer's Band", SlotKind::Gloves, 1, 1, 0),
            ("Sanctified Material", SlotKind::Gloves, 3, 1, 1),
            ("Deft Mold", SlotKind::Gloves, 5, 1, 1),
            ("Unshod Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Roots", SlotKind::Gloves, 1, 2, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 2, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 3, 1),
            ("Witch's Stilts", SlotKind::Gloves, 4, 2, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 4, 3),
        ],
        gear_offset: 0,
        bounty: 170,
        sprite: MonsterSprite::CogPriest,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 3, 3, 4, 4, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Mire Behemoth",
        // gear rating 642
        health: 2230,
        strength: 61,
        regen: 5,
        mind_resist: 53,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 53,
        attacks: &[],
        gear: &[
            ("Sanctified Material", SlotKind::Greaves, 0, 0, 1),
            ("Treadmill Sole", SlotKind::Greaves, 2, 0, 0),
            ("Witch's Stilts", SlotKind::Greaves, 0, 1, 3),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 3, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 2, 4, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 0, 5, 2),
            ("Consecrated Plating", SlotKind::Greaves, 3, 6, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 0, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 0, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Piercer's Band", SlotKind::Gloves, 4, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 2, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 2, 0),
            ("Unshod Signet", SlotKind::Gloves, 5, 1, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 3, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 3, 3),
            ("Unshod Signet", SlotKind::Gloves, 5, 3, 0),
            ("Oathring", SlotKind::Gloves, 5, 4, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 5, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 5, 1),
        ],
        gear_offset: 0,
        bounty: 179,
        sprite: MonsterSprite::Behemoth,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 3, 3, 4, 4, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Vermin Sovereign",
        // gear rating 695
        health: 2360,
        strength: 64,
        regen: 6,
        mind_resist: 56,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 56,
        attacks: &[],
        gear: &[
            ("Witch's Stilts", SlotKind::Greaves, 0, 0, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 1, 1, 0),
            ("Sanctified Material", SlotKind::Greaves, 3, 1, 0),
            ("Worldstrider Sole", SlotKind::Greaves, 1, 3, 0),
            ("Consecrated Plating", SlotKind::Greaves, 4, 3, 0),
            ("Sanctified Material", SlotKind::Greaves, 0, 3, 0),
            ("Sapling Mold", SlotKind::Greaves, 0, 5, 1),
            ("Rootwoven Material", SlotKind::Greaves, 3, 4, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 1, 5, 1),
            ("Consecrated Plating", SlotKind::Greaves, 4, 5, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 0, 0, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 0, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Piercer's Band", SlotKind::Gloves, 4, 1, 0),
            ("Warmed Material", SlotKind::Gloves, 0, 2, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 2, 1),
            ("Warmed Material", SlotKind::Gloves, 4, 2, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 2, 3, 3),
            ("Unshod Signet", SlotKind::Gloves, 5, 1, 0),
            ("Iron Band", SlotKind::Gloves, 1, 4, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 4, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 4, 3),
        ],
        gear_offset: 0,
        bounty: 188,
        sprite: MonsterSprite::Vermin,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 3, 2, 3, 4, 2, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Obsidian Colossus",
        // gear rating 739
        health: 2490,
        strength: 67,
        regen: 6,
        mind_resist: 59,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 59,
        attacks: &[],
        gear: &[
            ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 1),
            ("Worldstrider Sole", SlotKind::Greaves, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Greaves, 0, 2, 0),
            ("Warmed Material", SlotKind::Greaves, 2, 2, 0),
            ("Trailworn Sole", SlotKind::Greaves, 4, 1, 3),
            ("Consecrated Plating", SlotKind::Greaves, 4, 3, 0),
            ("Sanctified Material", SlotKind::Greaves, 0, 4, 1),
            ("Trailworn Sole", SlotKind::Greaves, 2, 4, 1),
            ("Consecrated Plating", SlotKind::Greaves, 3, 5, 0),
            ("Sanctified Material", SlotKind::Greaves, 0, 5, 3),
            ("Zealot's Sole", SlotKind::Greaves, 0, 7, 1),
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 0, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 0, 0),
            ("Oathring", SlotKind::Gloves, 0, 1, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 1, 1, 0),
            ("Quickfinger Mold", SlotKind::Gloves, 3, 1, 3),
            ("Tin Band", SlotKind::Gloves, 5, 1, 0),
            ("Piercer's Band", SlotKind::Gloves, 0, 2, 0),
            ("Sevenleague Boots", SlotKind::Gloves, 3, 3, 1),
            ("Quickfinger Mold", SlotKind::Gloves, 1, 4, 1),
            ("Unshod Signet", SlotKind::Gloves, 5, 2, 0),
            ("Ring of Roots", SlotKind::Gloves, 0, 4, 0),
        ],
        gear_offset: 0,
        bounty: 197,
        sprite: MonsterSprite::Obsidian,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 3, 3, 2, 4, 4, 4],
        enchs: &[],
    },
    MonsterSpec {
        name: "Null Sentinel",
        // gear rating 809
        health: 2620,
        strength: 70,
        regen: 6,
        mind_resist: 62,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 62,
        attacks: &[],
        gear: &[
            ("Leaden Tome", SlotKind::Weapon, 0, 0, 0),
            ("Kingsblood Ink", SlotKind::Weapon, 3, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 4, 1, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Scrying Lens", SlotKind::Helmet, 0, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 3, 4, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 5, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 5, 1),
            ("Forked Crest", SlotKind::Helmet, 0, 7, 0),
        ],
        gear_offset: 0,
        bounty: 206,
        sprite: MonsterSprite::Null,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 3, 4, 4],
        enchs: &[],
    },
    MonsterSpec {
        name: "Silence",
        // gear rating 861
        health: 2750,
        strength: 73,
        regen: 7,
        mind_resist: 65,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 65,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Rootwork Alignment", SlotKind::Weapon, 3, 1, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Reckoning Plate", SlotKind::Helmet, 3, 2, 0),
            ("Scrying Lens", SlotKind::Helmet, 0, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 4, 3, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 5, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 6, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 4, 1),
        ],
        gear_offset: 0,
        bounty: 215,
        sprite: MonsterSprite::Silence,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 3, 4, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Weeping Idol",
        // gear rating 907
        health: 2880,
        strength: 76,
        regen: 7,
        mind_resist: 68,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 68,
        attacks: &[],
        gear: &[
            ("Orb of the Nine", SlotKind::Weapon, 0, 0, 0),
            ("Unmaking", SlotKind::Weapon, 3, 0, 1),
            ("Kingsbane", SlotKind::Weapon, 4, 0, 1),
            ("Cometfall", SlotKind::Weapon, 0, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Reckoning Plate", SlotKind::Helmet, 4, 1, 2),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 3, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 6, 0),
            ("Sanctified Material", SlotKind::Gloves, 0, 0, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Grasping Ring", SlotKind::Gloves, 4, 0, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 5, 0, 0),
            ("Titan's Grip", SlotKind::Gloves, 3, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 1, 1, 3),
            ("Titan's Grip", SlotKind::Gloves, 0, 3, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Unshod Signet", SlotKind::Gloves, 0, 2, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 5, 3, 0),
            ("Titan's Grip", SlotKind::Gloves, 4, 4, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 5, 0),
            ("Mage's Sandals", SlotKind::Gloves, 0, 5, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 6, 2),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Weeping,
        rank: Rank::Boss,
        drops: &["The Seeker's Tears"],
        items: &[4, 3, 2, 3, 4, 2, 4, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Long Mirror",
        // gear rating 933
        health: 3010,
        strength: 79,
        regen: 7,
        mind_resist: 70,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 70,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Bronze Frame", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 4, 4, 1),
            ("Bronze Frame", SlotKind::Helmet, 0, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 6, 0),
            ("Bloomcap", SlotKind::Helmet, 4, 6, 1),
        ],
        gear_offset: 0,
        bounty: 233,
        sprite: MonsterSprite::Mirror,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 3, 2, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Iron Abbot",
        // gear rating 949
        health: 3140,
        strength: 82,
        regen: 8,
        mind_resist: 70,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 70,
        attacks: &[],
        gear: &[
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 0, 0),
            ("Arcane Splinter", SlotKind::Weapon, 3, 0, 1),
            ("Arcane Splinter", SlotKind::Weapon, 4, 0, 3),
            ("Oathstone Bead", SlotKind::Weapon, 0, 1, 1),
            ("Fury Sigil", SlotKind::Weapon, 2, 1, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Scrying Lens", SlotKind::Helmet, 0, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 3, 4, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 5, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 5, 1),
            ("Martyr's Crest", SlotKind::Helmet, 0, 7, 0),
        ],
        gear_offset: 0,
        bounty: 242,
        sprite: MonsterSprite::Abbot,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 4, 4],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Last Gearwright",
        // gear rating 956
        health: 3270,
        strength: 85,
        regen: 8,
        mind_resist: 70,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 70,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Cometfall", SlotKind::Weapon, 0, 2, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 3, 2, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 0, 1),
            ("Martyr's Crest", SlotKind::Helmet, 0, 2, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 3, 2, 1),
            ("Consecrated Plating", SlotKind::Helmet, 1, 3, 0),
            ("Martyr's Crest", SlotKind::Helmet, 0, 3, 1),
            ("Bone Frame", SlotKind::Helmet, 3, 4, 2),
            ("Consecrated Plating", SlotKind::Helmet, 1, 5, 0),
            ("Reckoning Plate", SlotKind::Helmet, 0, 6, 3),
            ("Martyr's Crest", SlotKind::Helmet, 3, 6, 0),
            ("Scale Layer", SlotKind::Chest, 2, 3, 0),
            ("Seedbed Layer", SlotKind::Chest, 2, 4, 0),
            ("Emberplate", SlotKind::Chest, 2, 5, 0),
            ("Heartwood Base", SlotKind::Chest, 2, 1, 0),
            ("Studded Sole", SlotKind::Greaves, 3, 2, 0),
            ("Scaled Plating", SlotKind::Greaves, 4, 2, 0),
            ("Anchor Material", SlotKind::Greaves, 1, 2, 0),
            ("Zealot's Sole", SlotKind::Greaves, 0, 5, 0),
            ("Reliquary Sole", SlotKind::Greaves, 1, 6, 0),
            ("Reckoning Plate", SlotKind::Greaves, 3, 6, 0),
        ],
        gear_offset: 0,
        bounty: 251,
        sprite: MonsterSprite::Gearwright,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 3, 4, 4, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Rimefather",
        health: 3480,
        strength: 92,
        regen: 8,
        mind_resist: 70,
        physical_resist: 48,
        magic_resist: 44,
        curse_resist: 70,
        attacks: &[],
        gear: &[
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 0, 0),
            ("Arcane Splinter", SlotKind::Weapon, 3, 0, 1),
            ("Arcane Splinter", SlotKind::Weapon, 4, 0, 3),
            ("Oathstone Bead", SlotKind::Weapon, 0, 1, 1),
            ("Fury Sigil", SlotKind::Weapon, 2, 1, 0),
            ("Steel Frame", SlotKind::Helmet, 0, 0, 0),
            ("Bloomcap", SlotKind::Helmet, 3, 0, 3),
            ("Martyr's Crest", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Reckoning Plate", SlotKind::Helmet, 4, 3, 2),
            ("Martyr's Crest", SlotKind::Helmet, 0, 4, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 5, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 5, 1),
            ("Martyr's Crest", SlotKind::Helmet, 0, 7, 0),
        ],
        gear_offset: 0,
        bounty: 262,
        sprite: MonsterSprite::Rimefather,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 4, 4],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Tallow Saint",
        health: 3690,
        strength: 96,
        regen: 9,
        mind_resist: 72,
        physical_resist: 50,
        magic_resist: 46,
        curse_resist: 72,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Sunder", SlotKind::Weapon, 3, 1, 0),
            ("Starfall", SlotKind::Weapon, 0, 2, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 3, 2, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Bronze Frame", SlotKind::Helmet, 0, 2, 0),
            ("Reckoning Plate", SlotKind::Helmet, 2, 2, 0),
            ("Scrying Lens", SlotKind::Helmet, 4, 2, 1),
            ("Martyr's Crest", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 3, 1),
            ("Bone Frame", SlotKind::Helmet, 0, 6, 0),
            ("Scrying Lens", SlotKind::Helmet, 1, 7, 0),
        ],
        gear_offset: 0,
        bounty: 273,
        sprite: MonsterSprite::Tallow,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 2, 4, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Hollowmarch",
        health: 3910,
        strength: 101,
        regen: 9,
        mind_resist: 74,
        physical_resist: 52,
        magic_resist: 48,
        curse_resist: 74,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 1, 2),
            ("Seal of the Deep", SlotKind::Gloves, 5, 1, 1),
            ("Blightfinger", SlotKind::Gloves, 2, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 3, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 5, 0),
            ("Rootwoven Material", SlotKind::Gloves, 5, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 3, 4, 2),
            ("Mage's Wrapping", SlotKind::Gloves, 1, 6, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 6, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Reckoning Plate", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 4, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 6, 0),
        ],
        gear_offset: 0,
        bounty: 284,
        sprite: MonsterSprite::March,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 4, 2, 2, 2, 2, 3, 2, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Iron Choir",
        health: 4140,
        strength: 106,
        regen: 10,
        mind_resist: 76,
        physical_resist: 54,
        magic_resist: 50,
        curse_resist: 76,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 2, 2, 0),
            ("Grasping Ring", SlotKind::Gloves, 4, 2, 0),
            ("Rootwoven Material", SlotKind::Gloves, 5, 0, 1),
            ("Flaying Mold", SlotKind::Gloves, 4, 3, 0),
            ("Grasping Ring", SlotKind::Gloves, 4, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 3, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 1, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 3, 3),
            ("Seal of the Deep", SlotKind::Gloves, 0, 4, 1),
            ("Grasping Ring", SlotKind::Gloves, 2, 5, 0),
            ("Rootwoven Material", SlotKind::Gloves, 3, 5, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 6, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Crown of the Deep", SlotKind::Helmet, 4, 1, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 3, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 3, 3),
            ("Mirrored Visor", SlotKind::Helmet, 1, 5, 0),
            ("Warlord's Crest", SlotKind::Helmet, 4, 4, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 6, 2),
        ],
        gear_offset: 0,
        bounty: 295,
        sprite: MonsterSprite::Bells,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[2, 4, 4, 4, 2, 4, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Gallowglass",
        health: 4380,
        strength: 112,
        regen: 10,
        mind_resist: 78,
        physical_resist: 56,
        magic_resist: 52,
        curse_resist: 78,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Hexer's Tally", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 1, 2),
            ("Rootwoven Material", SlotKind::Gloves, 5, 1, 1),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 1, 3, 0),
            ("Grasping Ring", SlotKind::Gloves, 0, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 5, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 5, 0),
            ("Grasping Ring", SlotKind::Gloves, 3, 5, 0),
            ("Rootwoven Material", SlotKind::Gloves, 4, 4, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 4, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 5, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Tithe Collector", SlotKind::Helmet, 5, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 1),
            ("Mirrored Visor", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 4, 4, 1),
        ],
        gear_offset: 0,
        bounty: 306,
        sprite: MonsterSprite::Gallows,
        rank: Rank::Mini,
        drops: &["Assassin's Hemline"],
        items: &[4, 2, 4, 4, 4, 4, 2, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Rust Parliament",
        health: 4640,
        strength: 118,
        regen: 11,
        mind_resist: 80,
        physical_resist: 58,
        magic_resist: 54,
        curse_resist: 80,
        attacks: &[],
        gear: &[
            ("Mage's Wrapping", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 4, 0, 0),
            ("Grasping Ring", SlotKind::Gloves, 3, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 1, 2),
            ("Rootwoven Material", SlotKind::Gloves, 5, 1, 1),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 1, 3, 0),
            ("Grasping Ring", SlotKind::Gloves, 0, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 5, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 5, 0),
            ("Seal of the Deep", SlotKind::Gloves, 3, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 1, 6, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 7, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 3, 6, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Watchful Crest", SlotKind::Helmet, 5, 0, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Bloomcap", SlotKind::Helmet, 4, 3, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 6, 0),
            ("Scrying Lens", SlotKind::Helmet, 3, 7, 0),
        ],
        gear_offset: 0,
        bounty: 317,
        sprite: MonsterSprite::Parliament,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 2, 4, 2, 4, 2, 3, 3, 2, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "Sootmother",
        health: 4910,
        strength: 124,
        regen: 11,
        mind_resist: 82,
        physical_resist: 60,
        magic_resist: 56,
        curse_resist: 82,
        attacks: &[],
        gear: &[
            ("Sanctified Material", SlotKind::Gloves, 0, 0, 1),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 0, 1),
            ("Seal of the Deep", SlotKind::Gloves, 5, 0, 1),
            ("Grasping Ring", SlotKind::Gloves, 1, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 3, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 1, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 2, 1),
            ("Grasping Ring", SlotKind::Gloves, 5, 2, 0),
            ("Sanctified Material", SlotKind::Gloves, 2, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 4, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 4, 1),
            ("Flaying Mold", SlotKind::Gloves, 1, 4, 3),
            ("Rootwoven Material", SlotKind::Gloves, 3, 4, 1),
            ("Flaying Mold", SlotKind::Gloves, 4, 4, 2),
            ("Grasping Ring", SlotKind::Gloves, 2, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 1, 6, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 7, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 6, 2),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Reckoning Plate", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 4, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Bloomcap", SlotKind::Helmet, 3, 2, 3),
            ("Bloomcap", SlotKind::Helmet, 4, 3, 3),
            ("Crown of the Deep", SlotKind::Helmet, 0, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 5, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 6, 0),
            ("Warlord's Crest", SlotKind::Helmet, 4, 6, 3),
        ],
        gear_offset: 0,
        bounty: 328,
        sprite: MonsterSprite::Sootmother,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 4, 2, 2, 4, 2, 3, 4, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Quiet Hour",
        health: 5190,
        strength: 131,
        regen: 12,
        mind_resist: 84,
        physical_resist: 62,
        magic_resist: 58,
        curse_resist: 84,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 1, 0),
            ("Blightfinger", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 1, 2),
            ("Seal of the Deep", SlotKind::Gloves, 5, 1, 1),
            ("Grasping Ring", SlotKind::Gloves, 2, 1, 0),
            ("Spun Material", SlotKind::Gloves, 0, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 4, 3, 0),
            ("Blightfinger", SlotKind::Gloves, 1, 4, 0),
            ("Rootwoven Material", SlotKind::Gloves, 3, 4, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 1, 5, 1),
            ("Rootwoven Material", SlotKind::Gloves, 0, 5, 1),
            ("Flaying Mold", SlotKind::Gloves, 1, 6, 2),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 5, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 6, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Warlord's Crest", SlotKind::Helmet, 4, 1, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 3, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 4, 0),
            ("Mirrored Visor", SlotKind::Helmet, 4, 4, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 5, 1),
            ("Consecrated Plating", SlotKind::Helmet, 0, 6, 0),
            ("Warlord's Crest", SlotKind::Helmet, 4, 6, 0),
        ],
        gear_offset: 0,
        bounty: 339,
        sprite: MonsterSprite::Hourglass,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 4, 4, 2, 2, 2, 4, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Verdigris",
        health: 5490,
        strength: 138,
        regen: 12,
        mind_resist: 86,
        physical_resist: 63,
        magic_resist: 60,
        curse_resist: 86,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Throttling Mold", SlotKind::Gloves, 3, 2, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 4, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 5, 1, 1),
            ("Throttling Mold", SlotKind::Gloves, 4, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 2, 4, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 1, 4, 3),
            ("Seal of the Deep", SlotKind::Gloves, 3, 5, 1),
            ("Grasping Ring", SlotKind::Gloves, 1, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 6, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 6, 3),
            ("Seal of the Deep", SlotKind::Gloves, 0, 7, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 6, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 3, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 4, 4, 1),
            ("Consecrated Plating", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 5, 1),
        ],
        gear_offset: 0,
        bounty: 350,
        sprite: MonsterSprite::Verdigris,
        rank: Rank::Mini,
        drops: &["Handman's Peel"],
        items: &[4, 4, 4, 4, 4, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Drowned Court",
        health: 5810,
        strength: 146,
        regen: 13,
        mind_resist: 88,
        physical_resist: 64,
        magic_resist: 62,
        curse_resist: 88,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 1, 3),
            ("Siphon Ring", SlotKind::Gloves, 1, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 0, 2, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 3, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 3, 1),
            ("Rootwoven Material", SlotKind::Gloves, 0, 5, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 4, 2),
            ("Seal of the Deep", SlotKind::Gloves, 0, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 4, 0),
            ("Rootwoven Material", SlotKind::Gloves, 2, 6, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 3, 6, 3),
            ("Grasping Ring", SlotKind::Gloves, 5, 5, 0),
            ("Siphon Ring", SlotKind::Gloves, 2, 7, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Crown of the Deep", SlotKind::Helmet, 0, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 3, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 4, 4, 1),
            ("Consecrated Plating", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 5, 1),
        ],
        gear_offset: 0,
        bounty: 361,
        sprite: MonsterSprite::Drowned,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[4, 4, 4, 4, 4, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "Anvilheart",
        health: 6150,
        strength: 154,
        regen: 14,
        mind_resist: 90,
        physical_resist: 66,
        magic_resist: 64,
        curse_resist: 90,
        attacks: &[],
        gear: &[
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Godsteel Plating", SlotKind::Helmet, 0, 2, 0),
            ("Crown of the Deep", SlotKind::Helmet, 3, 2, 0),
            ("Bulwark Base", SlotKind::Chest, 0, 0, 0),
            ("Godsheet Layer", SlotKind::Chest, 0, 2, 0),
            ("Godsheet Layer", SlotKind::Chest, 3, 2, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 4, 0, 0),
            ("Titan's Grip", SlotKind::Gloves, 0, 0, 0),
            ("Sovereign Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 2, 0),
            ("Warding Ring", SlotKind::Gloves, 3, 1, 0),
            ("Worldweave Material", SlotKind::Greaves, 0, 0, 0),
            ("Godsteel Plating", SlotKind::Greaves, 3, 0, 0),
            ("Sevenleague Sole", SlotKind::Greaves, 3, 1, 0),
            ("Sunderer", SlotKind::Weapon, 0, 0, 0),
            ("Sunderer", SlotKind::Weapon, 2, 0, 0),
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 3, 0),
            ("Grimoire Rack", SlotKind::Weapon, 4, 0, 0),
            ("Grimoire Rack", SlotKind::Weapon, 5, 0, 0),
            ("Ossuary Frame", SlotKind::Helmet, 0, 3, 0),
            ("Lonely Plating", SlotKind::Helmet, 2, 4, 0),
        ],
        gear_offset: 0,
        bounty: 372,
        sprite: MonsterSprite::Anvil,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Salt Wedding",
        health: 6510,
        strength: 163,
        regen: 14,
        mind_resist: 92,
        physical_resist: 68,
        magic_resist: 66,
        curse_resist: 92,
        attacks: &[],
        gear: &[
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Godsteel Plating", SlotKind::Helmet, 3, 0, 0),
            ("Godsteel Plating", SlotKind::Helmet, 0, 2, 0),
            ("Warlord's Crest", SlotKind::Helmet, 3, 1, 0),
            ("Bulwark Base", SlotKind::Chest, 0, 0, 0),
            ("Godsheet Layer", SlotKind::Chest, 0, 2, 0),
            ("Godsheet Layer", SlotKind::Chest, 3, 2, 0),
            ("Godsheet Layer", SlotKind::Chest, 0, 4, 0),
            ("Titan's Grip", SlotKind::Gloves, 0, 0, 0),
            ("Sovereign Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 2, 2, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
            ("Godsteel Plating", SlotKind::Greaves, 2, 0, 0),
            ("Sevenleague Sole", SlotKind::Greaves, 5, 0, 0),
            ("Sunderer", SlotKind::Weapon, 0, 0, 0),
            ("Sunderer", SlotKind::Weapon, 2, 0, 0),
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 3, 0),
            ("Grimoire Rack", SlotKind::Weapon, 4, 0, 0),
            ("Bileglass Vial", SlotKind::Weapon, 4, 2, 0),
        ],
        gear_offset: 0,
        bounty: 383,
        sprite: MonsterSprite::Wedding,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    MonsterSpec {
        name: "Nine of Ashes",
        health: 6890,
        strength: 172,
        regen: 15,
        mind_resist: 93,
        physical_resist: 70,
        magic_resist: 68,
        curse_resist: 93,
        attacks: &[],
        gear: &[
            ("Anvil Frame", SlotKind::Helmet, 0, 0, 0),
            ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
            ("Lonely Plating", SlotKind::Helmet, 3, 1, 0),
            ("Buttressed Frame", SlotKind::Helmet, 0, 2, 0),
            ("Lonely Plating", SlotKind::Helmet, 3, 2, 0),
            ("Lonely Plating", SlotKind::Helmet, 2, 3, 0),
            ("Coven Crest", SlotKind::Helmet, 5, 1, 0),
            ("Ossuary Frame", SlotKind::Helmet, 0, 3, 0),
            ("Scarred Plating", SlotKind::Helmet, 2, 4, 0),
            ("Scarred Plating", SlotKind::Helmet, 4, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 4, 3, 0),
            ("Cinder Base", SlotKind::Chest, 0, 0, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 3, 0, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 2, 1, 0),
            ("Ungloved Layer", SlotKind::Chest, 0, 2, 0),
            ("Grove Base", SlotKind::Chest, 1, 3, 0),
            ("Riveted Layer", SlotKind::Chest, 3, 4, 0),
            ("The Growing Weight", SlotKind::Chest, 4, 2, 0),
            ("Grove Base", SlotKind::Chest, 0, 5, 0),
            ("Wildfire Layer", SlotKind::Chest, 3, 5, 0),
            ("The Growing Weight", SlotKind::Chest, 2, 6, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 3, 0, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 5, 0, 0),
            ("Unshod Signet", SlotKind::Gloves, 0, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 1, 1, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 4, 1, 0),
            ("Unshod Signet", SlotKind::Gloves, 1, 2, 0),
            ("Rootwoven Material", SlotKind::Gloves, 2, 2, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 1, 3, 0),
            ("Ring of Embers", SlotKind::Gloves, 5, 2, 0),
            ("Unshod Signet", SlotKind::Gloves, 0, 3, 0),
            ("Ashwoven Material", SlotKind::Greaves, 0, 0, 0),
            ("Deeprooted Sole", SlotKind::Greaves, 3, 0, 0),
            ("Overflow Plate", SlotKind::Greaves, 0, 1, 0),
            ("Reliquary Sole", SlotKind::Greaves, 2, 1, 0),
            ("Widow's Sole", SlotKind::Greaves, 4, 1, 0),
            ("Warded Plating", SlotKind::Greaves, 1, 3, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 3, 3, 0),
            ("Widow's Sole", SlotKind::Greaves, 5, 2, 0),
            ("Mana Ward", SlotKind::Greaves, 0, 5, 0),
            ("Zealot's Haft", SlotKind::Weapon, 0, 0, 0),
            ("Gluttonous Fang", SlotKind::Weapon, 1, 0, 0),
            ("Fury Sigil", SlotKind::Weapon, 2, 0, 0),
            ("Bloodstone Bead", SlotKind::Weapon, 3, 0, 0),
            ("Zealot's Haft", SlotKind::Weapon, 5, 0, 0),
            ("Gluttonous Fang", SlotKind::Weapon, 3, 1, 0),
            ("Grudge Bead", SlotKind::Weapon, 4, 1, 0),
            ("Zealot's Haft", SlotKind::Weapon, 1, 2, 0),
            ("Gluttonous Fang", SlotKind::Weapon, 2, 2, 0),
            ("Oathstone Bead", SlotKind::Weapon, 4, 3, 0),
        ],
        gear_offset: 0,
        bounty: 394,
        sprite: MonsterSprite::Ashes,
        rank: Rank::Boss,
        drops: &["Tetrahedron Shard"],
        items: &[3, 4, 4, 4, 3, 3, 4, 3, 4, 3, 3, 3, 4, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "The Last Light",
        health: 7290,
        strength: 182,
        regen: 16,
        mind_resist: 94,
        physical_resist: 72,
        magic_resist: 70,
        curse_resist: 94,
        attacks: &[],
        gear: &[
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Godsteel Plating", SlotKind::Helmet, 3, 0, 0),
            ("Godsteel Plating", SlotKind::Helmet, 0, 2, 0),
            ("Martyr's Crest", SlotKind::Helmet, 3, 2, 0),
            ("Adamant Carapace", SlotKind::Chest, 0, 0, 0),
            ("Godsheet Layer", SlotKind::Chest, 0, 3, 0),
            ("Godsheet Layer", SlotKind::Chest, 3, 3, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 4, 0, 0),
            ("Titan's Grip", SlotKind::Gloves, 0, 0, 0),
            ("Sovereign Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 2, 0),
            ("Warding Ring", SlotKind::Gloves, 3, 1, 0),
            ("Titan's Grip", SlotKind::Greaves, 0, 0, 0),
            ("Godsteel Plating", SlotKind::Greaves, 3, 0, 0),
            ("Sevenleague Sole", SlotKind::Greaves, 3, 1, 0),
            ("Sunderer", SlotKind::Weapon, 0, 0, 0),
            ("Sunderer", SlotKind::Weapon, 2, 0, 0),
            ("Kingmaker Hilt", SlotKind::Weapon, 0, 3, 0),
            ("Bileglass Vial", SlotKind::Weapon, 4, 0, 0),
            ("Duelist's Fob", SlotKind::Weapon, 4, 1, 0),
        ],
        gear_offset: 0,
        bounty: 405,
        sprite: MonsterSprite::Lantern,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    MonsterSpec {
        name: "Gilt",
        health: 7720,
        strength: 192,
        regen: 17,
        mind_resist: 95,
        physical_resist: 74,
        magic_resist: 72,
        curse_resist: 95,
        attacks: &[],
        gear: &[
            ("Aegis Crown", SlotKind::Helmet, 0, 0, 0),
            ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
            ("Heartwood Crest", SlotKind::Helmet, 3, 1, 0),
            ("Aegis Crown", SlotKind::Helmet, 3, 2, 0),
            ("Lonely Plating", SlotKind::Helmet, 1, 3, 0),
            ("Lonely Plating", SlotKind::Helmet, 0, 4, 0),
            ("Bastion Base", SlotKind::Chest, 0, 0, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 3, 0, 0),
            ("The Growing Weight", SlotKind::Chest, 0, 2, 0),
            ("Grove Base", SlotKind::Chest, 2, 2, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 4, 3, 0),
            ("Warlord's Pauldron", SlotKind::Chest, 2, 4, 0),
            ("Ungloved Layer", SlotKind::Chest, 0, 4, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 3, 0, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Unshod Signet", SlotKind::Gloves, 0, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 1, 1, 0),
            ("Wrathful Mold", SlotKind::Gloves, 0, 2, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 4, 1, 0),
            ("Unshod Signet", SlotKind::Gloves, 5, 1, 0),
            ("Worldweave Material", SlotKind::Greaves, 0, 0, 0),
            ("Deeprooted Sole", SlotKind::Greaves, 3, 0, 0),
            ("Warded Plating", SlotKind::Greaves, 3, 1, 0),
            ("Sevenleague Boots", SlotKind::Greaves, 0, 2, 0),
            ("Sevenleague Sole", SlotKind::Greaves, 2, 2, 0),
            ("Lonely Plating", SlotKind::Greaves, 3, 3, 0),
            ("Zealot's Haft", SlotKind::Weapon, 0, 0, 0),
            ("Gluttonous Fang", SlotKind::Weapon, 1, 0, 0),
            ("Fury Sigil", SlotKind::Weapon, 2, 0, 0),
            ("Fury Sigil", SlotKind::Weapon, 3, 0, 0),
            ("Zealot's Haft", SlotKind::Weapon, 4, 0, 0),
            ("Gluttonous Fang", SlotKind::Weapon, 3, 2, 0),
            ("Grudge Bead", SlotKind::Weapon, 5, 0, 0),
            ("Bloodstone Bead", SlotKind::Weapon, 1, 2, 0),
        ],
        gear_offset: 0,
        bounty: 416,
        sprite: MonsterSprite::Gilt,
        rank: Rank::Mini,
        drops: &["Gilded Offcuts"],
        items: &[3, 3, 3, 4, 4, 4, 3, 3, 4, 4],
        enchs: &[],
    },
    // The top of the ladder. Everything above the Gearwright wears the best
    // the shop can sell; Francis wears something it never could.
    MonsterSpec {
        name: "Francis",
        health: 9400,
        strength: 215,
        regen: 22,
        mind_resist: 96,
        physical_resist: 78,
        magic_resist: 76,
        curse_resist: 96,
        attacks: &[],
        // Ninety-five percent of his cells, in nineteen items. He was on
        // thirty-six percent with one item a slot, which is not a hard fight,
        // it is four fifths of an empty board - the two finished human boards
        // in `share` pack ninety-seven and ninety-eight. Laid out by
        // `tests/pack_francis.rs` rather than by hand.
        gear: &[
            ("Buttressed Frame", SlotKind::Helmet, 0, 0, 0),
            ("Deadweight Plating", SlotKind::Helmet, 3, 0, 1),
            ("Warded Plating", SlotKind::Helmet, 2, 1, 0),
            ("Buttressed Frame", SlotKind::Helmet, 0, 1, 3),
            ("Broken Crown", SlotKind::Helmet, 1, 3, 0),
            ("Warded Plating", SlotKind::Helmet, 4, 1, 0),
            ("Zealot's Crest", SlotKind::Helmet, 0, 4, 1),
            ("Buttressed Frame", SlotKind::Helmet, 2, 5, 0),
            ("Deadweight Plating", SlotKind::Helmet, 0, 6, 1),
            ("Warded Plating", SlotKind::Helmet, 4, 6, 0),
            ("Zealot's Crest", SlotKind::Helmet, 0, 7, 0),
            ("The Money Jacket", SlotKind::Chest, 0, 0, 0),
            ("Runic Weave", SlotKind::Chest, 4, 0, 1),
            ("Bastion Base", SlotKind::Chest, 0, 3, 0),
            ("Bulwark Layer", SlotKind::Chest, 3, 3, 0),
            ("Verdant Weave", SlotKind::Chest, 5, 1, 1),
            ("Verdant Weave", SlotKind::Chest, 0, 5, 0),
            ("Bastion Base", SlotKind::Chest, 3, 5, 0),
            ("Runic Weave", SlotKind::Chest, 0, 6, 0),
            ("Breaker's Fist", SlotKind::Gloves, 0, 0, 1),
            ("Vicegrip Mold", SlotKind::Gloves, 3, 0, 0),
            ("Breaker's Fist", SlotKind::Gloves, 3, 1, 1),
            ("Vicegrip Mold", SlotKind::Gloves, 1, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 2, 1),
            ("Warding Ring", SlotKind::Gloves, 5, 0, 0),
            ("Breaker's Fist", SlotKind::Gloves, 1, 3, 1),
            ("Vicegrip Mold", SlotKind::Gloves, 4, 3, 0),
            ("Breaker's Fist", SlotKind::Gloves, 4, 4, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 2, 5, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 5, 0),
            ("Warding Ring", SlotKind::Gloves, 0, 4, 0),
            ("Breaker's Fist", SlotKind::Gloves, 0, 6, 1),
            ("Vicegrip Mold", SlotKind::Gloves, 3, 6, 1),
            ("Breaker's Fist", SlotKind::Greaves, 0, 0, 1),
            ("Witherroot", SlotKind::Greaves, 3, 0, 3),
            ("Deadweight Plating", SlotKind::Greaves, 5, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 2, 0),
            ("Witherroot", SlotKind::Greaves, 2, 2, 3),
            ("Broken Crown", SlotKind::Greaves, 0, 4, 0),
            ("Breaker's Fist", SlotKind::Greaves, 1, 6, 1),
            ("Witherroot", SlotKind::Greaves, 4, 6, 1),
            ("Deadweight Plating", SlotKind::Greaves, 5, 3, 0),
            ("Ironbound Haft", SlotKind::Weapon, 0, 0, 1),
            ("Bonesaw", SlotKind::Weapon, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 500,
        sprite: MonsterSprite::Francis,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 4, 4, 2, 4, 2, 2, 4, 2, 4, 2, 3, 3, 3, 2],
        enchs: &[],
    },
    // ---- M11.9's eight, and the new maps stop borrowing --------------------
    //
    // Dressed on the bench (`crates/lab`, `make dress`) and then moved by hand,
    // which is the order the bench itself insists on: it is a starting point
    // and not an answer, because a creature is a character before it is a
    // number.
    //
    // **Appended, never inserted.** `LADDER` order is a rung, and a rung is
    // read by `PIERCE_FROM`, `HARDEN_FROM` and the theme's own tables. Six of
    // these live on the new maps' pools and two stand on tower floors; none of
    // them changes what anything already on the ladder wears, which
    // `no_creature_changed_what_it_wears` says out loud.
    // **The Thing In The Fortieth Kettle** — rates 521. Thirty-nine kettles in the cooling yard are upside down and the fortieth is not.
    // There is rainwater in it. Something lives in the rainwater.
    MonsterSpec {
        name: "Kettle Wight",
        health: 865,
        strength: 26,
        regen: 2,
        mind_resist: 26,
        curse_resist: 26,
        physical_resist: 23,
        magic_resist: 20,
        attacks: &[],
        gear: &[
        ("Worldeye Orb", SlotKind::Weapon, 0, 0, 0),
        ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
        ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
        ("Four Hundred and Second Step", SlotKind::Helmet, 3, 1, 0),
        ("Countingstair Plating", SlotKind::Helmet, 5, 0, 0),
        ("Foreman's Harness", SlotKind::Chest, 0, 0, 0),
        ("Wickstub", SlotKind::Chest, 2, 0, 0),
        ("The Growing Weight", SlotKind::Chest, 3, 0, 0),
        ("Adamant Carapace", SlotKind::Chest, 2, 2, 0),
        ("Ungloved Layer", SlotKind::Chest, 0, 3, 0),
        ("Split Weave", SlotKind::Chest, 0, 5, 0),
        ],
        gear_offset: -2,
        bounty: 43,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Hooper's Dog** — rates 643. It has been round the hoop store nine times this morning and it is not looking
    // for anything. The hooper has stopped calling it.
    MonsterSpec {
        name: "Hoop Hound",
        health: 1290,
        strength: 32,
        regen: 2,
        mind_resist: 33,
        curse_resist: 33,
        physical_resist: 30,
        magic_resist: 27,
        attacks: &[],
        gear: &[
        ("Godsteel Haft", SlotKind::Weapon, 0, 0, 0),
        ("Sunderer", SlotKind::Weapon, 1, 0, 0),
        ("Worldsplitter", SlotKind::Weapon, 3, 0, 0),
        ("Ambusher's Grip", SlotKind::Weapon, 3, 1, 0),
        ("Blade of Helms", SlotKind::Weapon, 4, 2, 0),
        ("Manaflay", SlotKind::Weapon, 1, 3, 0),
        ("Bulwark Bead", SlotKind::Weapon, 5, 1, 0),
        ("Grudge Bead", SlotKind::Weapon, 1, 4, 0),
        ("Adamant Fang", SlotKind::Weapon, 2, 4, 0),
        ("Zealot's Haft", SlotKind::Weapon, 4, 4, 0),
        ("Aegis Crown", SlotKind::Helmet, 0, 0, 0),
        ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
        ("Four Hundred and Second Step", SlotKind::Helmet, 3, 1, 0),
        ("Countingstair Plating", SlotKind::Helmet, 5, 0, 0),
        ("Listening Frame", SlotKind::Helmet, 3, 2, 0),
        ("Bulwark Plating", SlotKind::Helmet, 0, 3, 0),
        ("Doubter's Crest", SlotKind::Helmet, 4, 4, 0),
        ("Antechamber Crown", SlotKind::Helmet, 0, 5, 0),
        ("Scarred Plating", SlotKind::Helmet, 3, 5, 0),
        ("Chapel Frame", SlotKind::Helmet, 2, 6, 0),
        ("Warmed Material", SlotKind::Gloves, 0, 0, 0),
        ("Throttling Mold", SlotKind::Gloves, 2, 0, 0),
        ("Seal of the Deep", SlotKind::Gloves, 4, 0, 0),
        ("Opening Grudge", SlotKind::Gloves, 4, 1, 0),
        ],
        gear_offset: -2,
        bounty: 53,
        sprite: MonsterSprite::Hound,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **What Comes Off At Dusk** — rates 731. The Stack sheds at dusk and not all of what comes off it lands. Four of the six
    // surveys of the smell line mention this and none of the six writes it down as a
    // creature.
    MonsterSpec {
        name: "Curd Wraith",
        health: 1700,
        strength: 43,
        regen: 2,
        mind_resist: 37,
        curse_resist: 37,
        physical_resist: 33,
        magic_resist: 29,
        attacks: &[],
        gear: &[
        ("Worldeye Orb", SlotKind::Weapon, 0, 0, 0),
        ("Adamant Carapace", SlotKind::Chest, 0, 0, 0),
        ("Ungloved Layer", SlotKind::Chest, 4, 0, 0),
        ("Split Weave", SlotKind::Chest, 0, 3, 0),
        ("Wrathbreaker", SlotKind::Chest, 4, 2, 0),
        ("Adamant Base", SlotKind::Chest, 0, 4, 0),
        ("Deep Roots Base", SlotKind::Chest, 3, 4, 0),
        ("Godsheet Layer", SlotKind::Chest, 0, 6, 0),
        ("Warlord's Pauldron", SlotKind::Chest, 3, 6, 0),
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Deeprooted Sole", SlotKind::Greaves, 2, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 2, 1, 0),
        ],
        gear_offset: 0,
        bounty: 60,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Wall Somebody Built** — rates 856. Four feet high, ninety feet long, running from nothing to nothing. It is a very
    // good wall, which is the part people come to see, and it is nine feet further
    // east than it was.
    MonsterSpec {
        name: "The Good Wall",
        health: 2020,
        strength: 51,
        regen: 3,
        mind_resist: 44,
        curse_resist: 44,
        physical_resist: 40,
        magic_resist: 36,
        attacks: &[],
        gear: &[
        ("Archmage's Primer", SlotKind::Weapon, 0, 0, 0),
        ("Kingsbane", SlotKind::Weapon, 2, 0, 0),
        ("Kingsblood Ink", SlotKind::Weapon, 0, 2, 0),
        ("Manaflay", SlotKind::Weapon, 4, 1, 0),
        ("Void Alignment", SlotKind::Weapon, 3, 2, 0),
        ("Voidwritten Ink", SlotKind::Weapon, 0, 4, 0),
        ("Aegis Crown", SlotKind::Helmet, 0, 0, 0),
        ("Godsteel Plating", SlotKind::Helmet, 3, 0, 0),
        ("Braced Plating", SlotKind::Helmet, 3, 2, 0),
        ("Crown of the Deep", SlotKind::Helmet, 0, 3, 0),
        ("Stonewall Frame", SlotKind::Helmet, 2, 4, 0),
        ("The Eyeless Stare", SlotKind::Helmet, 0, 5, 0),
        ("Bastion Base", SlotKind::Chest, 0, 0, 0),
        ("Warlord's Pauldron", SlotKind::Chest, 3, 0, 0),
        ("Seedbed Layer", SlotKind::Chest, 0, 2, 0),
        ("Becalming Layer", SlotKind::Chest, 3, 1, 0),
        ("Heartwood Base", SlotKind::Chest, 0, 3, 0),
        ("Leyline Cuirass", SlotKind::Chest, 3, 3, 0),
        ("Worldweave Material", SlotKind::Greaves, 0, 0, 0),
        ("Cadence Mold", SlotKind::Greaves, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 71,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Tenth Surveyor** — rates 970. Nine surveys of the reach and nine cairns where nine people gave up. The tenth
    // did not give up.
    MonsterSpec {
        name: "Trig Sentinel",
        health: 2440,
        strength: 62,
        regen: 4,
        mind_resist: 53,
        curse_resist: 53,
        physical_resist: 48,
        magic_resist: 43,
        attacks: &[],
        gear: &[
        ("Worldeye Orb", SlotKind::Weapon, 0, 0, 0),
        ("Lamplighter's Cage", SlotKind::Helmet, 0, 0, 0),
        ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
        ("Four Hundred and Second Step", SlotKind::Helmet, 3, 1, 0),
        ("Countingstair Plating", SlotKind::Helmet, 5, 0, 0),
        ("Listening Frame", SlotKind::Helmet, 3, 2, 0),
        ("Bulwark Plating", SlotKind::Helmet, 0, 3, 0),
        ("Doubter's Crest", SlotKind::Helmet, 4, 4, 0),
        ("Antechamber Crown", SlotKind::Helmet, 0, 5, 0),
        ("Scarred Plating", SlotKind::Helmet, 3, 5, 0),
        ("Chapel Frame", SlotKind::Helmet, 2, 6, 0),
        ("Deadweight Plating", SlotKind::Helmet, 5, 5, 0),
        ("Warlord's Crest", SlotKind::Helmet, 0, 6, 0),
        ("Titan's Grip", SlotKind::Gloves, 0, 0, 0),
        ("Twinning Mold", SlotKind::Gloves, 3, 0, 0),
        ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
        ("Seal of the Deep", SlotKind::Gloves, 4, 1, 0),
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Widow's Sole", SlotKind::Greaves, 2, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 3, 0, 0),
        ],
        gear_offset: 0,
        bounty: 80,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Nine Who Stopped** — rates 1141. One for each survey, each one built on the spot the surveyor was standing. They
    // have got up.
    MonsterSpec {
        name: "Cairn Chorus",
        health: 2670,
        strength: 68,
        regen: 4,
        mind_resist: 58,
        curse_resist: 58,
        physical_resist: 52,
        magic_resist: 46,
        attacks: &[],
        gear: &[
        ("Godsteel Haft", SlotKind::Weapon, 0, 0, 0),
        ("Sunderer", SlotKind::Weapon, 1, 0, 0),
        ("Worldsplitter", SlotKind::Weapon, 3, 0, 0),
        ("Ambusher's Grip", SlotKind::Weapon, 3, 1, 0),
        ("Blade of Helms", SlotKind::Weapon, 4, 2, 0),
        ("Manaflay", SlotKind::Weapon, 1, 3, 0),
        ("Bulwark Bead", SlotKind::Weapon, 5, 1, 0),
        ("Grudge Bead", SlotKind::Weapon, 1, 4, 0),
        ("Adamant Fang", SlotKind::Weapon, 2, 4, 0),
        ("Zealot's Haft", SlotKind::Weapon, 4, 4, 0),
        ("Leech Bead", SlotKind::Weapon, 3, 4, 0),
        ("Sunder Haft", SlotKind::Weapon, 5, 4, 0),
        ("Gravebound Haft", SlotKind::Weapon, 1, 5, 0),
        ("Bloodstone Bead", SlotKind::Weapon, 0, 7, 0),
        ("Oathstone Bead", SlotKind::Weapon, 2, 7, 0),
        ("Adamant Base", SlotKind::Chest, 0, 0, 0),
        ("Split Weave", SlotKind::Chest, 3, 0, 0),
        ("Wrathbreaker", SlotKind::Chest, 3, 1, 0),
        ("Deep Roots Base", SlotKind::Chest, 0, 2, 0),
        ("Godsheet Layer", SlotKind::Chest, 3, 3, 0),
        ("Warlord's Pauldron", SlotKind::Chest, 0, 4, 0),
        ("Bastion Base", SlotKind::Chest, 2, 5, 0),
        ("Seedbed Layer", SlotKind::Chest, 0, 7, 0),
        ("Becalming Layer", SlotKind::Chest, 0, 5, 0),
        ("Wildfire Layer", SlotKind::Chest, 3, 7, 0),
        ("Mail Layer", SlotKind::Chest, 4, 2, 0),
        ("Warmed Material", SlotKind::Gloves, 0, 0, 0),
        ("Twinning Mold", SlotKind::Gloves, 2, 0, 0),
        ("Grasping Ring", SlotKind::Gloves, 4, 0, 0),
        ("Seal of the Deep", SlotKind::Gloves, 3, 1, 0),
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Witherroot", SlotKind::Greaves, 2, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 4, 0, 0),
        ],
        gear_offset: 0,
        bounty: 95,
        sprite: MonsterSprite::Wraith,
        rank: Rank::Mini,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **What Was Left On Five** — rates 892. The fifth floor's, and the reason the fifth floor is two hundred feet across and
    // four feet high: something has been walking about up there.
    MonsterSpec {
        name: "Curd Sentinel",
        health: 2320,
        strength: 59,
        regen: 4,
        mind_resist: 50,
        curse_resist: 50,
        physical_resist: 45,
        magic_resist: 40,
        attacks: &[],
        gear: &[
        ("Ambusher's Grip", SlotKind::Weapon, 0, 0, 0),
        ("Blade of Helms", SlotKind::Weapon, 1, 0, 0),
        ("Manaflay", SlotKind::Weapon, 3, 0, 0),
        ("Bulwark Bead", SlotKind::Weapon, 5, 0, 0),
        ("Voidglass Shard", SlotKind::Weapon, 3, 1, 0),
        ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
        ("Godsteel Plating", SlotKind::Helmet, 3, 0, 0),
        ("Braced Plating", SlotKind::Helmet, 0, 2, 0),
        ("Crown of the Deep", SlotKind::Helmet, 2, 2, 0),
        ("Sightless Crown", SlotKind::Helmet, 0, 4, 0),
        ("Bulwark Plating", SlotKind::Helmet, 4, 3, 0),
        ("Doubter's Crest", SlotKind::Helmet, 3, 5, 0),
        ("Rimeguard Base", SlotKind::Chest, 0, 0, 0),
        ("Seedbed Layer", SlotKind::Chest, 3, 0, 0),
        ("Becalming Layer", SlotKind::Chest, 3, 1, 0),
        ("Heartwood Base", SlotKind::Chest, 0, 2, 0),
        ("Wildfire Layer", SlotKind::Chest, 3, 3, 0),
        ("Aether Layer", SlotKind::Chest, 0, 4, 0),
        ("Layered Core", SlotKind::Chest, 2, 4, 0),
        ("Bulwark Layer", SlotKind::Chest, 4, 4, 0),
        ("Oathplate", SlotKind::Chest, 0, 6, 0),
        ("Starlit Mantle", SlotKind::Chest, 2, 6, 0),
        ("Mail Layer", SlotKind::Chest, 4, 1, 0),
        ("Felt Layer", SlotKind::Chest, 4, 7, 0),
        ("Anchor Material", SlotKind::Gloves, 0, 0, 0),
        ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
        ("Seal of Power", SlotKind::Gloves, 4, 0, 0),
        ("Deepdraught Ring", SlotKind::Gloves, 3, 1, 0),
        ],
        gear_offset: 0,
        bounty: 74,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **What The Stack Was Standing On** — rates 1507. Two hundred and ten feet of the Drambus Stack stood on this for as long as
    // anybody has been counting, and the tally shed has six years of nothing to say
    // about it.
    MonsterSpec {
        name: "The Ground Floor",
        health: 3590,
        strength: 91,
        regen: 6,
        mind_resist: 75,
        curse_resist: 75,
        physical_resist: 70,
        magic_resist: 63,
        attacks: &[],
        gear: &[
        ("Worldeye Orb", SlotKind::Weapon, 0, 0, 0),
        ("Lamplighter's Cage", SlotKind::Helmet, 0, 0, 0),
        ("Lonely Plating", SlotKind::Helmet, 3, 0, 0),
        ("Four Hundred and Second Step", SlotKind::Helmet, 3, 1, 0),
        ("Countingstair Plating", SlotKind::Helmet, 5, 0, 0),
        ("Listening Frame", SlotKind::Helmet, 3, 2, 0),
        ("Bulwark Plating", SlotKind::Helmet, 0, 3, 0),
        ("Doubter's Crest", SlotKind::Helmet, 4, 4, 0),
        ("Antechamber Crown", SlotKind::Helmet, 0, 5, 0),
        ("Scarred Plating", SlotKind::Helmet, 3, 5, 0),
        ("Chapel Frame", SlotKind::Helmet, 2, 6, 0),
        ("Deadweight Plating", SlotKind::Helmet, 5, 5, 0),
        ("Warlord's Crest", SlotKind::Helmet, 0, 6, 0),
        ("Adamant Carapace", SlotKind::Chest, 0, 0, 0),
        ("Wickstub", SlotKind::Chest, 4, 0, 0),
        ("The Growing Weight", SlotKind::Chest, 4, 1, 0),
        ("Ungloved Layer", SlotKind::Chest, 1, 2, 0),
        ("Adamant Base", SlotKind::Chest, 2, 3, 0),
        ("Godsheet Layer", SlotKind::Chest, 0, 5, 0),
        ("Warlord's Pauldron", SlotKind::Chest, 3, 5, 0),
        ("Seedbed Layer", SlotKind::Chest, 0, 7, 0),
        ("Wellspring Base", SlotKind::Chest, 3, 7, 0),
        ("Mail Layer", SlotKind::Chest, 0, 4, 0),
        ("Titan's Grip", SlotKind::Gloves, 0, 0, 0),
        ("Twinning Mold", SlotKind::Gloves, 3, 0, 0),
        ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
        ("Seal of the Deep", SlotKind::Gloves, 4, 1, 0),
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Ridge Runner", SlotKind::Greaves, 2, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 2, 1, 0),
        ],
        gear_offset: 0,
        bounty: 125,
        sprite: MonsterSprite::Idol,
        rank: Rank::Boss,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Ninth Surveyor** — rates above Sootmother, and the number was set by
    // measurement rather than by adding to hers. `PLAN-M14.md` §4.4 says *by DPS
    // bracket against the walker's board, never by copying 4910/124 and adding*,
    // and the recon behind it is the reason: measured against
    // `common::geared_from`, this board beats Francis at 2958 and loses to Cairn
    // Chorus at 1141, so a **rating** predicts nothing about whether a fight is
    // winnable. What it loses to is damage per second. So this is dressed to be
    // beaten slowly rather than to hit hard, and `tests/sump.rs` measures the
    // margin against Sootmother's rather than against a number.
    //
    // She went down instead of up, and she has been standing on the island at
    // the bottom of her own hole for eleven years with the weather still on her.
    MonsterSpec {
        name: "The Ninth Surveyor",
        health: 6900,
        strength: 196,
        regen: 13,
        mind_resist: 88,
        curse_resist: 88,
        physical_resist: 66,
        magic_resist: 62,
        attacks: &[],
        gear: &[
        // **The weapon grid is where a fight is decided, and the first draft
        // did not have one.** A hilt and two accessories assemble nothing, so
        // the creature turned up to the fight carrying a decoration: measured,
        // it dealt **7.8 damage a second** and the same at strength 152 and at
        // 320, because strength pays a swing and there was no swing to pay.
        // Laid out the way The Last Light's is, which is the one board in the
        // ladder measured at the bracket this creature is aimed at.
        ("Sunderer", SlotKind::Weapon, 0, 0, 0),
        ("Sunderer", SlotKind::Weapon, 2, 0, 0),
        ("Kingmaker Hilt", SlotKind::Weapon, 0, 3, 0),
        ("Clockwork Key", SlotKind::Weapon, 4, 0, 0),
        ("Chain Coil", SlotKind::Weapon, 4, 1, 0),
        // Eleven years of weather on a helmet.
        ("Anvil Frame", SlotKind::Helmet, 0, 0, 0),
        ("Mirrored Visor", SlotKind::Helmet, 3, 0, 0),
        ("Bulwark Plating", SlotKind::Helmet, 0, 2, 0),
        ("Archon's Crest", SlotKind::Helmet, 3, 1, 0),
        ("Stonewall Frame", SlotKind::Helmet, 2, 3, 0),
        ("Mana Ward", SlotKind::Helmet, 0, 4, 0),
        ("Consecrated Plating", SlotKind::Helmet, 4, 4, 0),
        ("Warlord's Crest", SlotKind::Helmet, 0, 6, 0),
        // The coat.
        ("Bulwark Base", SlotKind::Chest, 0, 0, 0),
        ("Aegis Weave", SlotKind::Chest, 0, 2, 0),
        ("Aegis Weave", SlotKind::Chest, 3, 2, 0),
        ("Adamant Base", SlotKind::Chest, 0, 4, 0),
        ("Godsheet Layer", SlotKind::Chest, 3, 4, 0),
        ("Wellspring Base", SlotKind::Chest, 0, 6, 0),
        ("Mail Layer", SlotKind::Chest, 3, 6, 0),
        ("Bulwark Material", SlotKind::Gloves, 0, 0, 0),
        ("Sovereign Mold", SlotKind::Gloves, 3, 0, 0),
        ("Seal of the Deep", SlotKind::Gloves, 0, 2, 0),
        ("Storm Signet", SlotKind::Gloves, 2, 2, 0),
        // Boots that have been down and up nine times.
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Treadmill Sole", SlotKind::Greaves, 2, 0, 0),
        ("Sevenleague Sole", SlotKind::Greaves, 4, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 2, 1, 0),
        ],
        gear_offset: 0,
        bounty: 402,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Boss,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **What Marbulon Faced Away From** — the deeper of the two by one map's
    // worth of walking, and the last thing between a player and the country
    // under the country. Dressed the same way the Ninth Surveyor was and
    // against the same bracket: measured against `common::geared_from`, the
    // thing that decides a fight here is damage a second, and the ceiling is
    // whatever Gilt deals — because Gilt beats that board and this must not.
    //
    // She did not need to look at it. She needed you to.
    MonsterSpec {
        name: "What Marbulon Faced Away From",
        health: 7600,
        strength: 214,
        regen: 15,
        mind_resist: 90,
        curse_resist: 90,
        physical_resist: 68,
        magic_resist: 64,
        attacks: &[],
        gear: &[
        // **Laid out the way The Last Light's is**, which is the one board in
        // the ladder measured at this bracket. The first draft put a hilt, a
        // key and a charm in one row at y=3: they touch, so they merged, and
        // her whole board came to **three** items against the Ninth Surveyor's
        // eight — 114 damage a second where she is meant to be the deeper of
        // the two. What a creature *rates* is mostly what its gear rates, and
        // what its gear rates is mostly how many items come out of it.
        ("Sunderer", SlotKind::Weapon, 0, 0, 0),
        ("Sunderer", SlotKind::Weapon, 2, 0, 0),
        ("Kingmaker Hilt", SlotKind::Weapon, 0, 3, 0),
        ("Clockwork Key", SlotKind::Weapon, 4, 0, 0),
        ("Silver Charm", SlotKind::Weapon, 4, 1, 0),
        // **The coordinates are the Ninth Surveyor's and the pieces are not.**
        // What decides how many items a board makes is where the pieces sit,
        // not what they are called, and her first arrangement made three where
        // that one makes eight. Two creatures at the bottom of two dungeons cut
        // by the same people wearing the same *shape* of kit is a fact about
        // the reach; wearing the same components would be a fact about nobody
        // having looked.
        ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
        ("Mirrored Visor", SlotKind::Helmet, 3, 0, 0),
        ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
        ("Martyr's Crest", SlotKind::Helmet, 3, 1, 0),
        ("Anvil Frame", SlotKind::Helmet, 2, 3, 0),
        ("Mana Ward", SlotKind::Helmet, 0, 4, 0),
        ("Bulwark Plating", SlotKind::Helmet, 4, 4, 0),
        ("Archon's Crest", SlotKind::Helmet, 0, 6, 0),
        ("Adamant Carapace", SlotKind::Chest, 0, 0, 0),
        ("Aegis Weave", SlotKind::Chest, 0, 2, 0),
        ("Aegis Weave", SlotKind::Chest, 3, 2, 0),
        ("Bulwark Base", SlotKind::Chest, 0, 4, 0),
        ("Warlord's Pauldron", SlotKind::Chest, 3, 4, 0),
        ("Adamant Base", SlotKind::Chest, 0, 6, 0),
        ("Godsheet Layer", SlotKind::Chest, 3, 6, 0),
        ("Bulwark Material", SlotKind::Gloves, 0, 0, 0),
        ("Twinning Mold", SlotKind::Gloves, 3, 0, 0),
        ("Storm Signet", SlotKind::Gloves, 0, 2, 0),
        ("Seal of the Deep", SlotKind::Gloves, 2, 2, 0),
        ("Sevenleague Boots", SlotKind::Greaves, 0, 0, 0),
        ("Sevenleague Sole", SlotKind::Greaves, 2, 0, 0),
        ("Ridge Runner", SlotKind::Greaves, 4, 0, 0),
        ("Overflow Plate", SlotKind::Greaves, 2, 1, 0),
        ],
        gear_offset: 0,
        bounty: 448,
        sprite: MonsterSprite::Sootmother,
        rank: Rank::Boss,
        drops: &[],
        items: &[],
        enchs: &[],
    },
    // **The Tenth Surveyor** — on the plate in the middle of the Needle Room,
    // under three feet of sand and eleven years of a compass turning over her.
    //
    // **She is wearing the run.** Thirty-eight components in the cells the
    // human seated them in, and all six of that character's enchs — the first
    // creature in the game to carry one. `the_tenth_surveyor_wears_the_run`
    // compares her `outfit()` against `common::from_save`'s `reports()` by name
    // and count per slot, so if a placement stops seating, the gear block is
    // wrong and not the board.
    //
    // **The gear is in item order and not board order**, which is
    // `Character::item_partition`'s whole reason to exist: `items` is a chunk
    // list, and the run's first helmet item is board entries 0, 3 and 5 while
    // its second is 1, 2 and 4. Reordering costs nothing, because a placement
    // carries its own absolute cell.
    //
    // **No instrument frame.** A creature has no survey — and §5.1 does not
    // transcribe one either, which is the same decision arrived at from two
    // directions.
    //
    // The body numbers are found rather than typed: see
    // `the_run_beats_her_and_the_shopper_does_not`. `PLAN-M16.md` §5.2 asks for
    // a win rate between 55% and 70% "over a loop of seeds", and **combat has
    // no RNG** — a loop over seeds counts the same fight every time. What
    // varies between two players meeting her is the *board*, so the bracket is
    // over boards, which is M11.7's `common::geared_from` rule stated twice.
    MonsterSpec {
        name: "The Tenth Surveyor",
        // **Found, not typed.** Health barely moves this fight and strength is
        // the whole dial, which is the Kettleworks finding a third time: what
        // decides a fight at this depth is damage a second. Measured against
        // `common::geared_from` — the board a player actually has when they get
        // here — the deep ladder deals
        //
        //   The Ninth Surveyor          122.1/s   Victory
        //   What Marbulon Faced Away    138.7/s   Victory
        //   Gilt                        428.3/s   Defeat
        //   Nine of Ashes               531.0/s   Defeat
        //
        // so the band where a fight is a fight rather than a wall is between a
        // hundred and forty and four hundred. She deals **221.2/s** and that
        // board beats her in forty-three seconds, past the sudden-death clock.
        // Her first draft was 236 strength and dealt **807.9/s**, which killed
        // it in three.
        //
        // Fifteen thousand health is the largest number in the game and it is
        // doing work: everything at this depth is settled after `SUDDEN_DEATH
        // _MS`, and what health buys is how long she stands in it.
        health: 15_000,
        strength: 64,
        regen: 15,
        mind_resist: 74,
        curse_resist: 70,
        physical_resist: 66,
        magic_resist: 70,
        attacks: &[],
        gear: &[
        ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
        ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
        ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
        ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
        ("Bronze Frame", SlotKind::Helmet, 4, 1, 0),
        ("Bronze Plating", SlotKind::Helmet, 4, 0, 0),
        ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
        ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
        ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
        ("Chain Layer", SlotKind::Chest, 0, 3, 0),
        ("Brigandine Base", SlotKind::Chest, 4, 1, 0),
        ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
        ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
        ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
        ("Oathring", SlotKind::Gloves, 1, 2, 0),
        ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
        ("Tin Band", SlotKind::Gloves, 3, 1, 0),
        ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
        ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
        ("Padded Mold", SlotKind::Gloves, 4, 2, 0),
        ("Plain Sole", SlotKind::Greaves, 0, 0, 0),
        ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
        ("Spun Material", SlotKind::Greaves, 2, 0, 1),
        ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
        ("Spun Material", SlotKind::Greaves, 2, 1, 3),
        ("Sapling Mold", SlotKind::Greaves, 4, 1, 0),
        ("Herbal", SlotKind::Weapon, 0, 0, 0),
        ("Chain Coil", SlotKind::Weapon, 1, 0, 2),
        ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
        ("Emberburst", SlotKind::Weapon, 2, 2, 0),
        ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
        ("Quicksilver Ink", SlotKind::Weapon, 2, 1, 1),
        ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
        ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
        ("Quicksilver Ink", SlotKind::Weapon, 5, 0, 1),
        ("Ratchet Cog", SlotKind::Weapon, 0, 4, 1),
        ("Quicksilver Ink", SlotKind::Weapon, 2, 4, 0),
        ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
        ],
        items: &[3, 3, 4, 2, 4, 4, 2, 2, 2, 4, 5, 3],
        // The six, by index into `gear` above — an index and not a name,
        // because this board holds three Quicksilver Inks and a name cannot
        // say which of them the Band is on.
        enchs: &[
        ("the-yodregar-index", 28),  // Runewash Ink
        ("the-wextreen-correction", 29),  // Emberburst
        ("the-chonga-swing", 10),  // Brigandine Base
        ("sneel-bearing", 26),  // Herbal
        ("plug-energy-tap", 27),  // Chain Coil
        ("grungo-elastic-band", 31),  // Quicksilver Ink
        ],
        gear_offset: 0,
        bounty: 486,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Boss,
        drops: &[],
    },

    // ---------------------------------------------------------------- the deep
    //
    // **Ten for the Wextreen Sands and the Eleven Reefs**, asked for as
    // *extremely unbelievelably dangerous*, with boards that are variations on
    // the Tenth Surveyor's — which is a real player's board, seated by hand.
    //
    // So that is literally what they are: her slots, worn in part, with the
    // per-slot item chunking carried across. Nothing invents a component,
    // because a creature whose gear does not assemble turns up to a fight in an
    // empty frame and nothing in the suite would call that a bug.
    //
    // **Strength is the dial and health is not**, which this file has now found
    // three times — the Kettleworks, the Ninth Surveyor, and the Tenth. So the
    // ladder here climbs on strength and item count, and the health numbers
    // only keep the fights from ending in three seconds.
    // **Chainman** — the Sands. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, gloves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "Chainman",
        health: 9000,
        strength: 70,
        regen: 12,
        mind_resist: 78,
        curse_resist: 74,
        physical_resist: 62,
        magic_resist: 66,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
        ],
        items: &[4, 5, 3, 4, 4],
        gear_offset: 0,
        bounty: 520,
        sprite: MonsterSprite::Marshal,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **Backsight** — the Sands. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, helmet. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "Backsight",
        health: 10500,
        strength: 76,
        regen: 12,
        mind_resist: 80,
        curse_resist: 76,
        physical_resist: 64,
        magic_resist: 68,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 560,
        sprite: MonsterSprite::Wraith,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Eleventh Notch** — the Sands. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, greaves, gloves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "The Eleventh Notch",
        health: 12000,
        strength: 82,
        regen: 14,
        mind_resist: 82,
        curse_resist: 78,
        physical_resist: 68,
        magic_resist: 70,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Racing Sole", SlotKind::Greaves, 0, 0, 0),
            ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
            ("Spun Material", SlotKind::Greaves, 2, 0, 1),
            ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
            ("Spun Material", SlotKind::Greaves, 2, 1, 3),
            ("Pilgrim's Sole", SlotKind::Greaves, 4, 1, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
        ],
        items: &[4, 5, 3, 2, 2, 2, 4, 4],
        gear_offset: 0,
        bounty: 610,
        sprite: MonsterSprite::Null,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **Iron Under It** — the Sands. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, chest, helmet. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "Iron Under It",
        health: 13500,
        strength: 88,
        regen: 14,
        mind_resist: 84,
        curse_resist: 80,
        physical_resist: 72,
        magic_resist: 72,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 4, 2, 3, 3],
        gear_offset: 0,
        bounty: 660,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Levelling Staff** — the Flat Below. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, chest, gloves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "The Levelling Staff",
        health: 16000,
        strength: 95,
        regen: 16,
        mind_resist: 86,
        curse_resist: 82,
        physical_resist: 74,
        magic_resist: 74,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
        ],
        items: &[4, 5, 3, 4, 2, 4, 4],
        gear_offset: 0,
        bounty: 720,
        sprite: MonsterSprite::Sentinel,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **What the Flat Kept** — the Flat Below. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, helmet, greaves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "What the Flat Kept",
        health: 18000,
        strength: 102,
        regen: 16,
        mind_resist: 88,
        curse_resist: 84,
        physical_resist: 76,
        magic_resist: 76,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
            ("Racing Sole", SlotKind::Greaves, 0, 0, 0),
            ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
            ("Spun Material", SlotKind::Greaves, 2, 0, 1),
            ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
            ("Spun Material", SlotKind::Greaves, 2, 1, 3),
            ("Pilgrim's Sole", SlotKind::Greaves, 4, 1, 0),
        ],
        items: &[4, 5, 3, 3, 3, 2, 2, 2],
        gear_offset: 0,
        bounty: 780,
        sprite: MonsterSprite::Silence,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Closing Error** — the Assay. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, chest, gloves, helmet. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "The Closing Error",
        health: 20000,
        strength: 110,
        regen: 18,
        mind_resist: 88,
        curse_resist: 86,
        physical_resist: 78,
        magic_resist: 78,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 4, 2, 4, 4, 3, 3],
        gear_offset: 0,
        bounty: 850,
        sprite: MonsterSprite::Choir,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Benchmark** — the Assay. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, chest, greaves, gloves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "The Benchmark",
        health: 22500,
        strength: 118,
        regen: 18,
        mind_resist: 90,
        curse_resist: 88,
        physical_resist: 80,
        magic_resist: 80,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Racing Sole", SlotKind::Greaves, 0, 0, 0),
            ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
            ("Spun Material", SlotKind::Greaves, 2, 0, 1),
            ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
            ("Spun Material", SlotKind::Greaves, 2, 1, 3),
            ("Pilgrim's Sole", SlotKind::Greaves, 4, 1, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
        ],
        items: &[4, 5, 3, 4, 2, 2, 2, 2, 4, 4],
        gear_offset: 0,
        bounty: 920,
        sprite: MonsterSprite::Lantern,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **Datum** — the Needle Room. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, helmet, chest, gloves, greaves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "Datum",
        health: 25000,
        strength: 126,
        regen: 20,
        mind_resist: 92,
        curse_resist: 90,
        physical_resist: 82,
        magic_resist: 82,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
            ("Racing Sole", SlotKind::Greaves, 0, 0, 0),
            ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
            ("Spun Material", SlotKind::Greaves, 2, 0, 1),
            ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
            ("Spun Material", SlotKind::Greaves, 2, 1, 3),
            ("Pilgrim's Sole", SlotKind::Greaves, 4, 1, 0),
        ],
        items: &[4, 5, 3, 3, 3, 4, 2, 4, 4, 2, 2, 2],
        gear_offset: 0,
        bounty: 1000,
        sprite: MonsterSprite::King,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Traverse** — the Needle Room. Its board is the Tenth Surveyor's own, worn in
    // part: weapon, helmet, chest, gloves, greaves. **Nothing here is a new component**, which is why
    // ten of these could be written at all — her blocks are known to assemble,
    // and a creature whose gear does not assemble turns up to the fight in an
    // empty frame, alive and harmless and indistinguishable from a balance
    // decision. The chunking comes over with them, per slot.
    MonsterSpec {
        name: "The Traverse",
        health: 28000,
        strength: 136,
        regen: 22,
        mind_resist: 94,
        curse_resist: 92,
        physical_resist: 84,
        magic_resist: 84,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
            ("Chorister's Base", SlotKind::Chest, 0, 0, 1),
            ("Chorister's Weave", SlotKind::Chest, 2, 0, 0),
            ("Chorister's Layer", SlotKind::Chest, 2, 1, 0),
            ("Runed Lining", SlotKind::Chest, 0, 3, 0),
            ("Layered Core", SlotKind::Chest, 4, 1, 0),
            ("Sprocketman's Gratitude", SlotKind::Chest, 4, 3, 0),
            ("Rimeglove Material", SlotKind::Gloves, 0, 0, 0),
            ("Bramble Mold", SlotKind::Gloves, 2, 0, 0),
            ("Iron Band", SlotKind::Gloves, 1, 2, 0),
            ("Rat Signet", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 3, 1, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 0, 0),
            ("Witch's Thimble", SlotKind::Gloves, 3, 2, 0),
            ("Featherweight Mold", SlotKind::Gloves, 4, 2, 0),
            ("Racing Sole", SlotKind::Greaves, 0, 0, 0),
            ("Ratskin Material", SlotKind::Greaves, 0, 1, 0),
            ("Spun Material", SlotKind::Greaves, 2, 0, 1),
            ("Greave Mold", SlotKind::Greaves, 4, 0, 0),
            ("Spun Material", SlotKind::Greaves, 2, 1, 3),
            ("Pilgrim's Sole", SlotKind::Greaves, 4, 1, 0),
        ],
        items: &[4, 5, 3, 3, 3, 4, 2, 4, 4, 2, 2, 2],
        gear_offset: 0,
        bounty: 1100,
        sprite: MonsterSprite::Fiend,
        rank: Rank::Ordinary,
        drops: &[],
        enchs: &[],
    },
    // **The Unwritten** — the Undercountry, and the hardest fight in the game.
    //
    // The third town is empty and says so: the writing stops there, and
    // `common::UNWRITTEN` is where that emptiness is declared. This is what is
    // standing in the country the writing stopped in — a post with nothing cut
    // into it, which is also what the town's own sign is.
    //
    // **Her whole board, and then some strength.** It wears the Tenth
    // Surveyor's five slots entire rather than a part of them, which is what
    // makes it the top of the ladder without inventing a component. *Strength
    // is the dial and health is not*, so that is where the danger is put.
    MonsterSpec {
        name: "The Unwritten",
        health: 12_000,
        strength: 80,
        regen: 18,
        mind_resist: 62,
        curse_resist: 62,
        physical_resist: 42,
        magic_resist: 46,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        // **Two slots, and that is what makes it beatable at all.** Its first
        // draft wore her whole board — twelve items — and killed the best board
        // the game hands out in **8.9 seconds** whatever else was tuned:
        // strength down to 50 and health down to 10,000 changed the outcome not
        // once, because *what a creature deals is mostly how many items its
        // board makes*, which this file has now found for the fourth time.
        //
        // `every_region_has_a_fight_you_can_win_and_every_boss_can_be_beaten`
        // is what said so, and it is right to: **a boss nobody can beat is a
        // wall with a sentence on it**, and this project shipped a whole block
        // unfinishable once because a range said *some of these are winnable*.
        // So the item count comes down to the one shape in the set a player
        // survives, and the danger goes back into strength and health where it
        // can be afforded.
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 1400,
        sprite: MonsterSprite::Null,
        rank: Rank::Boss,
        drops: &[],
        enchs: &[],
    },
    // ---- the Cairnworks, four floors under the Wextreen Reach -------------
    //
    // **Each one shuts every lane but one, and "100%" is 95.** `stats::LANE_CAP`
    // has been 95 since M16 for a reason this block must not undo — *a lane you
    // can commit to is never one you can be shut out of* — so a boss written at
    // a literal hundred would be a wall with a sentence on it. Ninety-five
    // means the wrong lane does a twentieth of its damage, which against these
    // healths is a loss at the buzzer. That *is* "you have to use the other
    // lane", in a game with a clock.
    //
    // **The open lane is written as a large negative, not as a zero.** A board
    // grants resistances of its own, so a boss with `curse_resist: 0` and forty
    // pieces on it is a boss with about forty curse resist — the design would
    // have been undone by the costume. The negative is what makes the sum land
    // at nothing whatever the gear adds, and
    // `the_cairnworks_shuts_every_lane_but_one` is what measures it rather than
    // trusting the arithmetic.
    //
    // **All four wear The Unwritten's two slots**, which is the one shape in
    // the set a player survives — see its own note. Nothing here invents a
    // component, so the catalogue and the save fingerprint are untouched.
    // **Floor one: sear it.** Swings and spells alike come off it and it has never
    // been on fire, which is a thing it does not have a word for.
    MonsterSpec {
        name: "The Unslaked Kiln",
        health: 26_000,
        strength: 96,
        regen: 20,
        mind_resist: 130,
        curse_resist: -400,
        physical_resist: 130,
        magic_resist: 130,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 900,
        sprite: MonsterSprite::Fiend,
        rank: Rank::Boss,
        drops: &[],
        enchs: &[],
    },
    // **Floor two: hit it.** A wall built by somebody who was counting. Nothing
    // subtle gets through it and a hammer does.
    MonsterSpec {
        name: "Nine Courses of Brick",
        health: 11_000,
        strength: 74,
        regen: 20,
        mind_resist: 130,
        curse_resist: 130,
        physical_resist: -400,
        magic_resist: 130,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 950,
        sprite: MonsterSprite::Colossus,
        rank: Rank::Boss,
        drops: &[],
        enchs: &[],
    },
    // **Floor three: burn it down with something that is not a blade.** Draught,
    // and a draught you cannot put a blade in.
    MonsterSpec {
        name: "The Cold Flue",
        health: 6_000,
        strength: 46,
        regen: 20,
        mind_resist: 130,
        curse_resist: 130,
        physical_resist: 130,
        magic_resist: -400,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 1000,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Boss,
        drops: &[],
        enchs: &[],
    },
    // **Floor four: pierce it.** Every lane at the cap and health enough to sit
    // out a twentieth of anything. Piercing cuts a resistance rather than
    // beating it, which is the one thing that reaches this.
    MonsterSpec {
        name: "What Was Left Banked",
        health: 7_500,
        strength: 52,
        regen: 20,
        mind_resist: 130,
        curse_resist: 130,
        physical_resist: 130,
        magic_resist: 130,
        attacks: &[],
        gear: &[
            ("Hymnal", SlotKind::Weapon, 0, 0, 0),
            ("Bulwark Vial", SlotKind::Weapon, 1, 0, 2),
            ("Runewash Ink", SlotKind::Weapon, 0, 2, 2),
            ("Emberburst", SlotKind::Weapon, 2, 2, 0),
            ("Cosmic Alignment", SlotKind::Weapon, 2, 0, 0),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 1, 1),
            ("The Bog Census", SlotKind::Weapon, 3, 0, 0),
            ("Census Bolt", SlotKind::Weapon, 4, 2, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 5, 0, 1),
            ("Flywheel Cog", SlotKind::Weapon, 0, 4, 1),
            ("Cinderscript Ink", SlotKind::Weapon, 2, 4, 0),
            ("Azure Alignment", SlotKind::Weapon, 4, 4, 0),
            ("Bone Crown", SlotKind::Helmet, 0, 0, 3),
            ("Bone Scale", SlotKind::Helmet, 1, 1, 2),
            ("Bone Fletch", SlotKind::Helmet, 2, 2, 3),
            ("Idol's Crest", SlotKind::Helmet, 3, 0, 0),
            ("Tin Frame", SlotKind::Helmet, 4, 1, 0),
            ("Tin Plating", SlotKind::Helmet, 4, 0, 0),
        ],
        items: &[4, 5, 3, 3, 3],
        gear_offset: 0,
        bounty: 1500,
        sprite: MonsterSprite::Null,
        rank: Rank::Boss,
        drops: &[],
        enchs: &[],
    },
];

// ----------------------------------------------------------- combatants

/// An item mid-fight: its profile plus how far its cooldown has filled.
///
/// `Default` is here for tests that care about one field - which item a stun
/// picks depends on `rating` and `stun_ms` and nothing else, and spelling out
/// thirty irrelevant fields to say so buries the point.
#[derive(Clone, Debug, Default)]
pub struct RunningItem {
    pub name: String,
    /// Effectiveness on the shared scale, so the interface can badge it.
    pub rating: i32,
    pub slot: Option<SlotKind>,
    pub cooldown_ms: u32,
    pub progress_ms: u32,
    /// How much longer this one item is stopped for. A stun holds a single
    /// item rather than the whole fighter, so it lives here.
    pub stun_ms: u32,
    /// Bar-fill this item owes, because it gave the time away.
    ///
    /// A `Shunt` hands `ms` to a neighbour and takes on `ms` of debt; the debt
    /// is paid down out of the step before any of the step reaches
    /// `progress_ms`, so the same millisecond that left one bar arrives on the
    /// other and time is conserved. Kept as a debt rather than subtracted from
    /// `progress_ms` on the spot because a bar cannot go below zero, and an
    /// item that had just fired would otherwise swallow the cost.
    pub owed_ms: u32,
    /// Run gold this item has spent so far this fight, and how many times it
    /// has paid. The budget belongs to the item, so the tally does too.
    pub gold_spent: i32,
    pub gold_paid: u32,
    /// Standing on a Lightning Rod, so anything that picks a target on this
    /// board picks this.
    pub attracts_curses: bool,
    /// A misfire does not eat this one's activation.
    pub steady: bool,
    /// **Overtake**: the first firing of the fight runs twice.
    pub overtakes: bool,
    /// Whether this item has fired yet, which is the whole of Overtake's
    /// condition. Per item rather than per fighter, because a board with two
    /// overtaking gloves gets two opening double-swings and that is what
    /// building two of them is for.
    pub has_fired: bool,
    /// This item turns once a second where it stands, banking power.
    ///
    /// Set from the profile, which took it from an ench **and** from whether
    /// the board leaves it anywhere to turn. An item boxed in does not spin,
    /// which is the trade rather than an oversight.
    pub spins: bool,
    /// This item fires once and is finished.
    pub fragile: bool,
    /// Something is bolted to one of this item's components.
    ///
    /// Off the profile, which took it from the character. The two rules that
    /// read it — `Productivity` and `Beacon` — are about the ench rather than
    /// about the gear, and a fight has no way to ask which is which otherwise.
    pub enched: bool,
    /// How many times this item has activated this fight.
    ///
    /// **Per item, not per fighter.** `Combatant::activations` counts the whole
    /// side and is what `Echo` reads; `Productivity` is a property of the
    /// *item* — a board with two enched items gets two schedules, which is what
    /// enching two of them is for.
    pub fires: u32,
    /// The cooldown this item started the fight with.
    ///
    /// Kept so `Productivity`'s slowdown is applied to the original rather than
    /// compounded onto the running value: scaling a cooldown that has already
    /// been scaled is the same fault as a replay subtracting damage from a
    /// health total it keeps itself, and ten procs would stop the item.
    pub base_cooldown_ms: u32,
    /// How much slower this item has become, in percent, for the rest of the
    /// fight.
    ///
    /// Accumulated by `Productivity`, which is the only thing that writes it.
    /// **For the fight and not for good**, like `broken`: a `RunningItem` is
    /// rebuilt at every bell.
    pub slowed_pct: u32,
    /// And it has. Its bar does not advance, it does not turn, and it does not
    /// fire again this fight.
    ///
    /// **Its own field rather than `stun_ms = u32::MAX`.** A stun is a curse
    /// somebody put on you — it is aimed, it is resisted, it is counted by
    /// `StunAim`, and it ends. This is a property of the gear, and folding the
    /// two together would have made every question about stuns answer wrongly
    /// about a broken item.
    ///
    /// **And not `has_fired`**, which already exists and is Overtake's: that
    /// one asks *was this the first?* and this asks *is it finished?* One flag
    /// answering two questions is how the next person gets it wrong.
    pub broken: bool,
    /// Turns banked and not yet spent. Cleared the moment the item activates.
    pub spin_stacks: u32,
    /// Where in its turn cycle it currently stands.
    ///
    /// The index rather than the cells: the cycle is a board fact and lives on
    /// the profile, and the fight only has to say which of its entries is
    /// showing. Two copies of the geometry would be two answers to "what shape
    /// is it right now".
    pub turn_index: u32,
    /// How long its cycle is, so the index can wrap without asking the board.
    pub turn_cycle_len: u32,
    /// Milliseconds since the last turn, so a turn is a whole second of this
    /// item's fight rather than a count of ticks somebody has to divide.
    pub spin_ms: u32,
    /// Neither stunned nor misfiring, for the rest of the fight.
    ///
    /// `steady` is the first half and predates this. The second is the answer
    /// to `StunStrongest`, which aims at the best item a fighter owns - so
    /// what this protects is exactly what that picks.
    pub unshakable: bool,
    /// What this item multiplies its own damage by, in hundredths.
    pub power: i32,
    pub physical_damage: i32,
    pub magic_damage: i32,
    pub mind: i32,
    pub armor: i32,
    pub mana: i32,
    pub rage: i32,
    pub faith: i32,
    pub nature: i32,
    pub triggers: Vec<Trigger>,
    pub adjacent_assembled_same_slot: usize,
    /// Empty cells touching this item on the board it was built on.
    pub open_cells: usize,
    /// Indices, in the owner's item list, of items this one reacts to.
    pub adjacent_items: Vec<usize>,
    pub aligned_items: Vec<usize>,
    pub diagonal_items: Vec<usize>,
    /// One tally per entry in `triggers`, so a `Watch` remembers what it has
    /// seen. Parallel to `triggers` rather than keyed by anything, because two
    /// identical watchers on one item are two separate counts.
    pub watched: Vec<u32>,
    /// Which watchers have already paid out. Only read by the ones that do not
    /// repeat.
    pub watch_paid: Vec<bool>,
    /// Monster attacks can carry a curse; player items use triggers instead.
    pub curse: Option<CurseKind>,
    /// Fingerprint used to draw this item's emblem.
    pub sigil_seed: u64,
    /// Weapon power that applies to this item alone - a spell's ink.
    pub power_bonus: i32,
    /// The payloads a spell cycles through. Empty for ordinary gear.
    pub casts: Vec<crate::loadout::Cast>,
    /// Which payload the next cast will use.
    pub cast_index: usize,
}

impl RunningItem {
    fn from_profile(p: &ItemProfile) -> Self {
        RunningItem {
            name: p.name.clone(),
            slot: Some(p.slot),
            attracts_curses: p.attracts_curses,
            steady: p.steady,
            overtakes: p.overtakes,
            has_fired: false,
            fragile: p.fragile,
            broken: false,
            unshakable: false,
            cooldown_ms: p.cooldown_ms,
            progress_ms: 0,
            stun_ms: 0,
            // **Both halves.** The ench says it would like to turn and the
            // board says whether there is anywhere to turn to. One entry in
            // the cycle is an item that lands on itself or is boxed in.
            spins: p.spins && p.turn_cycle.len() > 1,
            enched: p.enched,
            fires: 0,
            slowed_pct: 0,
            base_cooldown_ms: p.cooldown_ms,
            spin_stacks: 0,
            spin_ms: 0,
            turn_index: 0,
            turn_cycle_len: p.turn_cycle.len().max(1) as u32,
            owed_ms: 0,
            gold_spent: 0,
            gold_paid: 0,
            physical_damage: p.stats.physical_damage,
            magic_damage: p.stats.magic_damage,
            rage: p.stats.rage,
            faith: p.stats.faith,
            nature: p.stats.nature,
            mind: p.stats.mind,
            armor: p.stats.armor,
            mana: p.stats.mana,
            triggers: p.triggers.clone(),
            adjacent_assembled_same_slot: p.adjacent_assembled_same_slot,
            open_cells: p.open_cells,
            power: p.power,
            adjacent_items: p.adjacent_items.clone(),
            aligned_items: p.aligned_items.clone(),
            diagonal_items: p.diagonal_items.clone(),
            watched: vec![0; p.triggers.len()],
            watch_paid: vec![false; p.triggers.len()],
            curse: None,
            sigil_seed: p.sigil_seed,
            rating: p.rating,
            power_bonus: p.power_bonus,
            casts: p.casts.clone(),
            cast_index: 0,
        }
    }

    fn from_attack(a: &MonsterAttack) -> Self {
        RunningItem {
            name: a.name.to_string(),
            slot: None,
            // A monster's own teeth stand on nothing.
            attracts_curses: false,
            steady: false,
            // Overtake is a glove's, and a creature wears no gloves.
            overtakes: false,
            has_fired: false,
            // A creature's own teeth do not break. An ench is bolted to a
            // component and a bite stands on none.
            fragile: false,
            broken: false,
            unshakable: false,
            // A bite stands on no component, so nothing can be bolted to it.
            enched: false,
            fires: 0,
            slowed_pct: 0,
            base_cooldown_ms: a.cooldown_ms.max(TICK_MS),
            spins: false,
            spin_stacks: 0,
            spin_ms: 0,
            turn_index: 0,
            turn_cycle_len: 1,
            cooldown_ms: a.cooldown_ms.max(TICK_MS),
            progress_ms: 0,
            stun_ms: 0,
            owed_ms: 0,
            gold_spent: 0,
            gold_paid: 0,
            physical_damage: a.damage,
            magic_damage: 0,
            rage: 0,
            faith: 0,
            nature: 0,
            mind: a.mind,
            armor: a.armor,
            mana: 0,
            triggers: Vec::new(),
            adjacent_assembled_same_slot: 0,
            open_cells: 0,
            power: 100,
            adjacent_items: Vec::new(),
            aligned_items: Vec::new(),
            diagonal_items: Vec::new(),
            watched: Vec::new(),
            watch_paid: Vec::new(),
            curse: a.curse,
            // Innate attacks have no gear behind them, so seed off the name.
            rating: 0,
            power_bonus: 0,
            casts: Vec::new(),
            cast_index: 0,
            sigil_seed: a.name.bytes().fold(0x1234_5678_u64, |h, b| {
                h.rotate_left(5) ^ b as u64
            }),
        }
    }

    /// Fraction of the way to the next activation, for cooldown bars.
    pub fn progress(&self) -> f32 {
        if self.cooldown_ms == 0 {
            return 0.0;
        }
        (self.progress_ms as f32 / self.cooldown_ms as f32).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct Combatant {
    pub name: String,
    pub max_health: i32,
    pub health: i32,
    /// Temporary hit points. Always starts a fight at zero — gear has to build
    /// it up — and soaks damage before health does.
    pub armor: i32,
    pub mana: i32,
    pub strength: i32,
    pub power: i32,
    pub regen: i32,
    pub mind_resist: i32,
    pub curse_resist: i32,
    // The defence triangle, per damage type. See `stats::after_defences`.
    pub physical_resist: i32,
    pub physical_pierce: i32,
    pub physical_harden: i32,
    pub magic_resist: i32,
    pub magic_pierce: i32,
    pub magic_harden: i32,
    /// Percent of absorbed damage turned back on whoever swung.
    pub reflect: i32,
    /// Banked resources. Each is spent by triggers and worth something merely
    /// by being held - see `held_bonus`.
    pub rage: i32,
    pub faith: i32,
    pub nature: i32,
    /// The fused pools. Made by `Action::Fuse`, worth both parents at double
    /// rate, and spendable by nothing - the only way one leaves is a `Drain`.
    pub druidic_might: i32,
    pub communion: i32,
    pub zealotry: i32,
    /// Run gold carried into the fight. Only the player has one, and only
    /// `SpendGold` touches it; what it spends is gone at the shop afterwards.
    pub purse: i32,
    /// Which foe the next single-target attack is aimed at. Only the player
    /// has one, and it moves along every time an attack lands - see `aim_of`.
    pub aim: usize,
    /// Set by Immense Guilt: regeneration does nothing at all.
    pub no_regen: bool,
    /// Set by Trundle: everything runs this much slower, and every point of
    /// armour counts this much. Percentages; 0 and 100 mean "as written".
    pub slower_pct: i32,
    pub armour_pct: i32,
    /// Set by Longhauler: everything runs this much faster for every second
    /// the fight has been going, capped at twice speed.
    pub haste_per_s: i32,
    /// Set by Ticket to Ride: every `n`th attack made against this fighter
    /// misses entirely. Zero means nothing misses.
    pub warded_every: u32,
    /// Attacks this fighter has made that were counted against a ward. Kept on
    /// the attacker, so two creatures each miss every other swing rather than
    /// sharing one tally between them.
    pub warded_count: u32,
    pub curses: Curses,
    /// Stacks of mana empowerment and mana shield. Both scale off *current*
    /// mana, and both are bought with mana — so stacking them hard drains the
    /// very pool they multiply. That tension is the point.
    ///
    /// Both are the **magic** lane's and only the magic lane's. Empowerment
    /// multiplies magic-typed hits and the shield reduces magic-typed damage;
    /// a physical swing is computed as though neither stack were there.
    pub empowerment: u32,
    /// How many of those stacks the **furnace** bought.
    ///
    /// **Reported from play: *"mana empowerment ... seemingly does nothing for
    /// my attacks"*, and it was exactly right.** Empowerment is upstream's
    /// caster mechanic: `magic_empower` scales a **magic** hit and nothing
    /// else, so a Kettle-Stoker swinging a blade got stacks that could never
    /// be read. Measured against `common::geared_from` — the board a player
    /// actually has — a Stoker dealt 746 and a classless character dealt 746.
    ///
    /// So the furnace's own stacks are counted apart and pay **both lanes**.
    /// That is a change to what a *Stoker* gets and not to what empowerment
    /// means: a Chronomancer's stacks are still the caster's, because nothing
    /// but `stoke` ever puts a number here. The class is GM2D's own and
    /// nothing obliged it to inherit a restriction its promise does not
    /// mention — *shovel a pool into the firebox and swing harder* is what a
    /// firebox is.
    pub burn_stacks: u32,
    pub shield: u32,
    /// The mind lane's pool and its stack. Insight is fuel like mana - it pays
    /// nothing at all while held - and Dread is what turns it into damage.
    pub insight: i32,
    pub dread: u32,
    /// Whether this fighter still owes itself one blow that cannot be stopped.
    ///
    /// Set by Wumpus Hunter and spent by the first hit that lands. Two things
    /// in this game can eat a swing outright - a ward and a deflection - and
    /// this is the only answer to either.
    pub first_blood: bool,
    /// Percentage of the target's mind resistance the mind damage this fighter
    /// deals goes straight through.
    ///
    /// The third lane had an amplifier, a pool and an answer, and no way at
    /// all through the answer - which the other two have had since typed
    /// damage landed. Only one thing in the game sets it.
    pub mind_pierce: i32,
    /// Stacks of Spellblade and Deflection: the same pair in the physical
    /// lane, and **not** scaled by mana.
    ///
    /// That is the whole difference between the two pairs. Mana scaling is
    /// what makes the mana pair conditional - a ceiling to build towards and a
    /// pool to keep full - so the twins have neither, and are worth the same
    /// to every board that manages to gain one.
    pub spellblade: u32,
    /// **The wrong sense.** Set by `Action::SeeWithTheWrongSense`, and after it
    /// every point of physical and magic this fighter would deal is not dealt -
    /// the mind lane is paid instead, multiplied by what was given up.
    pub wrong_sense: bool,
    /// Damage surrendered to the wrong sense so far, in points.
    ///
    /// Held as the surrendered swing rather than as a factor, because the
    /// factor *is* the board's own damage and a number here would be a second
    /// copy of it. `wrong_sense_multiplied` turns it into one.
    pub surrendered: i64,
    pub deflection: u32,
    /// Stacks of spell forking: every cast lands once more per stack.
    pub forking: u32,
    pub items: Vec<RunningItem>,
    /// Sub-point accumulators, so 10 damage a second spread over 50ms ticks
    /// loses nothing to rounding.
    /// Chronomancer's slow time: damage waiting to arrive, and how long each
    /// portion has left. Empty for everyone else.
    pending: Vec<(i32, u32)>,
    /// Whether incoming damage is queued rather than taken at once.
    /// Seconds damage is spread over. Zero means it lands at once.
    pub slow_time: u32,
    /// Held resources count double.
    /// How many times a held pool counts. One is ordinary.
    pub overflowing: i32,
    /// Percent of damage dealt that comes back as health.
    pub leech: i32,
    /// Every nth activation fires twice. Zero means never.
    pub echo_every: u32,
    /// Percent of absorbed damage handed back as armour.
    pub bastion: i32,
    /// Curses landed bring the other kind with them.
    /// Extra curses dragged in alongside each one landed.
    pub contagion: u32,
    /// Faith banked whenever a hit lands on you.
    pub reprisal: i32,
    /// Milliseconds every enemy activation gives back to your cooldowns.
    pub riposte: u32,
    /// Strength gained per second the fight has run.
    pub momentum: i32,
    /// Reactions fire twice.
    /// How many times a reaction pays out. One is ordinary.
    pub resonance: u32,
    /// Percent of physical damage that lands again as magic.
    pub transmute: i32,
    /// Every activation banks one of each pool.
    /// Of each pool banked per activation. Zero is ordinary.
    pub adaptable: i32,
    /// Oracle: every this-many-th activation lands the two curses that work on
    /// time - a stun and a misfire.
    pub untimely: u32,
    /// Stormcaller: every activation pushes every OTHER item's cooldown
    /// forward by this many ms, so a fast build compounds on itself.
    pub cascade: u32,
    /// Warpriest: armour gained is this much stronger, in percent, while any
    /// faith is held.
    pub consecrate: i32,
    /// Activations counted for the misfire curse. Counting rather than rolling
    /// keeps the fight deterministic.
    pub misfire_count: u32,
    /// How many stuns this fighter has taken. Mixed into the choice of which
    /// item the next one lands on, so a chain of stuns walks across the kit
    /// instead of hammering one slot.
    pub stun_count: u32,
    /// The same, for an Oracle's periodic reach at the clock.
    pub untimely_count: u32,
    /// Bloodletter: landing a curse banks this much rage.
    pub bloodscent: i32,
    /// Wellspring: spending a pool refunds this percent of it to each of the
    /// other three.
    pub confluence: i32,
    /// The expert class in play, with its knobs already tuned by the tree.
    ///
    /// **One `Option` rather than a field a knob**, and the difference is that
    /// there is no sentinel. Ten experts carrying thirty-one knobs between
    /// them would be thirty-one fields on this struct, every one of which has
    /// to mean *off* at some value — and `rate: 0` meaning "not a Loud
    /// Calculation" is exactly the kind of number a screen cannot tell from a
    /// bug. A character holds at most one expert, so `None` is the whole of
    /// what "not one" means.
    ///
    /// It arrives through [`Held`] like every other fight input, and is
    /// translated at the bell the way a `ClassPower` is — so combat stays a
    /// pure function of what it was handed.
    pub expert: Option<crate::expert::ExpertPower>,
    /// Every this-many-th activation of an **enched** item runs twice, and
    /// that item is this much slower for the rest of the fight.
    ///
    /// `None` for everybody who has not granted it. An `Option` rather than a
    /// zero, for the reason `expert` is one: `every: 0` would have to mean
    /// *never*, and a zero standing in for absence is a number a screen cannot
    /// tell from a bug.
    pub productivity: Option<(u32, u32)>,
    /// Funny bought with strength this fight, and what is still free.
    ///
    /// Loud Calculation's ledger. Two numbers because the class has two
    /// budgets: `cap` is how much may be bought at all, and `floor` is how
    /// much of it costs no strength — *you may buy this many past empty*.
    pub loud_bought: i32,
    pub loud_free_left: i32,
    /// Strength borrowed so far, for the rebate a kill pays back.
    pub loud_owed: i32,
    /// Curses standing on the other side that will never expire.
    ///
    /// Counted on the **lander**, not the sufferer: `carry` is how many this
    /// fighter may keep standing, and a brawl must not let three foes share
    /// one allowance or divide it.
    pub facts_standing: u32,
    /// Requisitions left this fight, and how many have been made.
    ///
    /// `reqs_done` is what `relist` counts against, and it is separate from
    /// `reqs_left` because *how many are gone* and *how many were made* stop
    /// being the same number the first time one is refunded.
    pub reqs_left: i32,
    pub reqs_done: u32,
    /// When the Opening Number's free window closes, and how many restarts of
    /// it are left.
    pub opening_until_ms: u32,
    pub opening_encores: i32,
    /// Whether the window's leftover Funny has already been banked as armour.
    /// Once a fight, at the moment it closes — a second payout would make the
    /// window worth reopening for its own sake.
    pub opening_banked: bool,
    /// The empty frames' spin: how far round they are, and what they hold.
    pub overwound_ms: u32,
    pub overwound_stacks: i32,
    /// The Stoker's furnace: how long since the last shovelful, and what it is
    /// set to. Zeroed for everybody who is not one — **a creature is never a
    /// Stoker**, the same rule an expert follows.
    pub burn_ms: u32,
    pub burn_every_ms: u32,
    pub burn_per_stack: i32,
    /// What a burned pool still pays, in percent of its standing bonus.
    ///
    /// `Rule::BurnKeepsBonus`, and it is the Stoker's only rule: the trade made
    /// gentler rather than removed. Zero for everybody, which is the trade as
    /// written.
    pub burn_keeps_pct: i32,
    /// **M16's eleven experts, in two counters and nothing else.** Each is a
    /// budget a fight starts with and spends, the way `reqs_left` and
    /// `loud_free_left` are — everything else the eleven do is read off
    /// `Combatant::expert` where its own mechanic lives, which is what keeps
    /// eleven new powers from being thirty new fields.
    pub fired_left: i32,
    pub silent_left: i32,
    /// Mind damage a silence has banked and the next activation will say.
    pub silent_owed: i32,
    /// **Told Once**, copied onto the sufferer at the bell: points a curse
    /// standing on it adds to whatever threshold is being measured against it,
    /// and the ceiling on that.
    ///
    /// On the *sufferer* rather than the lander, which is the one place this
    /// block departs from `facts_standing`: the threshold is read inside
    /// `take_mind_pierced`, which has one fighter and no view of the room.
    pub told_per_curse: i32,
    pub told_cap: i32,
    /// Mind damage the character adds to every mind hit, off the tree.
    ///
    /// Named for what it is rather than for the stat it came from: `mind` on a
    /// `Combatant` would read as *this fighter's mind damage*, which is the
    /// board's and is already in the profiles.
    pub said: i32,
    /// Tenths of a stack the boiler has taken off the spin and not yet paid.
    ///
    /// Tenths in and whole stacks out, which is `overwound_stacks`'s own rule:
    /// the pile is kept fine and only the crossings are paid, so nothing is
    /// lost to rounding and nothing is paid twice.
    pub boiler_tenths: i32,
    /// What the furnace has taken this fight, by pool, so the bonus it keeps
    /// can be paid without the pool being there.
    pub burned: [i32; 3],
    /// The Whisperer's threshold: the maximum health at or below which this
    /// fighter is unmade, set once at the bell. `None` for everybody else.
    pub unmade_at: Option<i32>,
    /// How many of the five worn frames have nothing seated in them.
    ///
    /// **A board fact the fight has to be told**, because a fight has never
    /// had a board — `ItemProfile` is a flat snapshot, which is why a
    /// mid-fight save carries a creature name and a tile and nothing else.
    /// Two experts read it and neither could work it out from the profiles: an
    /// empty frame produces no profile at all, so *nothing* and *five of
    /// nothing* look identical from in here.
    pub empty_frames: u32,
    /// Extra stacks a turning item banks each turn, on top of the one.
    pub spin_extra: u32,
    /// Stacks a turning item keeps through an activation.
    pub spin_keep: u32,
    /// How long one turn takes for this fighter's gear.
    pub spin_every_ms: u32,
    /// Slots whose every activation lands a curse, because the skill tree
    /// said so.
    ///
    /// **A fight input, not a global.** It arrives on `Held` beside the armour
    /// and the mana the tree grants, the same way a `ClassPower` arrives on
    /// `classes` — combat stays a pure function of what it was handed, which
    /// is the property a mid-fight save rests on.
    pub curse_on_activate: Vec<(SlotKind, crate::curse::CurseKind)>,
    /// How many times this side has activated anything, for `echo_every`.
    activations: u32,
    dot_milli: i32,
    regen_milli: i32,
    /// Burn damage already taken but not yet written to the log, and how long
    /// since the last entry. Damage-over-time lands every tick; logging it
    /// every tick buries everything else under a wall of "burns for 1".
    burn_acc: i32,
    burn_timer: u32,
    /// Non-zero while curse watchers are being told about a curse.
    ///
    /// A watcher that counts curses and answers with a curse would count its
    /// own answer and answer that, and so on until the stack ran out - which
    /// is exactly what one accessory did, and the crash arrived as a fatal
    /// runtime error in a test three files away rather than as anything the
    /// catalogue tests could see. Nothing an author writes should be able to
    /// do that, so the notification does not re-enter.
    curse_watch_depth: u32,
}

impl Combatant {
    pub fn player(stats: Stats, profiles: &[ItemProfile]) -> Self {
        Combatant {
            name: "You".to_string(),
            // Both are set at the bell by `apply_held`, off what the character
            // handed in. Nothing is one here, which is what a fighter with no
            // expert and no board is.
            expert: None,
            productivity: None,
            loud_bought: 0,
            loud_free_left: 0,
            loud_owed: 0,
            facts_standing: 0,
            reqs_left: 0,
            reqs_done: 0,
            opening_until_ms: 0,
            opening_encores: 0,
            opening_banked: false,
            overwound_ms: 0,
            overwound_stacks: 0,
            burn_ms: 0,
            burn_every_ms: 0,
            burn_per_stack: 0,
            burn_keeps_pct: 0,
            fired_left: 0,
            silent_left: 0,
            silent_owed: 0,
            said: 0,
            told_per_curse: 0,
            told_cap: 0,
            boiler_tenths: 0,
            burned: [0; 3],
            unmade_at: None,
            empty_frames: 0,
            max_health: stats.health,
            health: stats.health,
            armor: 0,
            mana: 0,
            druidic_might: 0,
            communion: 0,
            zealotry: 0,
            strength: stats.strength,
            power: stats.power,
            regen: stats.regen,
            mind_resist: stats.mind_resist,
            physical_resist: stats.physical_resist,
            physical_pierce: stats.physical_pierce,
            physical_harden: stats.physical_harden,
            magic_resist: stats.magic_resist,
            magic_pierce: stats.magic_pierce,
            magic_harden: stats.magic_harden,
            reflect: stats.reflect,
            rage: 0,
            faith: 0,
            nature: 0,
            pending: Vec::new(),
            slow_time: 0,
            overflowing: 1,
            leech: 0,
            echo_every: 0,
            bastion: 0,
            contagion: 0,
            reprisal: 0,
            riposte: 0,
            momentum: 0,
            resonance: 1,
            transmute: 0,
            adaptable: 0,
            untimely: 0,
            cascade: 0,
            consecrate: 0,
            misfire_count: 0,
            stun_count: 0,
            untimely_count: 0,
            bloodscent: 0,
            confluence: 0,
            activations: 0,
            curse_resist: stats.curse_resist,
            purse: 0,
            aim: 0,
            no_regen: false,
            slower_pct: 0,
            armour_pct: 100,
            haste_per_s: 0,
            warded_every: 0,
            warded_count: 0,
            curses: Curses::new(),
            empowerment: 0,
            burn_stacks: 0,
            shield: 0,
            insight: 0,
            dread: 0,
            first_blood: false,
            mind_pierce: 0,
            spellblade: 0,
            // Read off the board, once, at the bell. A standing state and not
            // a trigger: "you do not deal damage any more" is true from the
            // first tick, and anything that set it later would let the opening
            // blows land - a free multiplier for the start of the fight and a
            // trade for the rest of it.
            wrong_sense: profiles.iter().any(|p| p.wrong_sense),
            surrendered: 0,
            deflection: 0,
            forking: 0,
            items: profiles.iter().map(RunningItem::from_profile).collect(),
            dot_milli: 0,
            regen_milli: 0,
            burn_acc: 0,
            burn_timer: 0,
            curse_watch_depth: 0,
            curse_on_activate: Vec::new(),
            spin_extra: 0,
            spin_keep: 0,
            spin_every_ms: SPIN_EVERY_MS,
        }
    }

    pub fn monster(spec: &MonsterSpec) -> Self {
        Combatant::monster_at(spec, Difficulty::Easy)
    }

    pub fn monster_at(spec: &MonsterSpec, difficulty: Difficulty) -> Self {
        // Most of the setting is in what it is wearing; the multiplier below
        // is only what is left over.
        let (mut stats, profiles) = spec.outfit_at(difficulty);

        // Half the difficulty goes into staying alive and half into hitting
        // back, so the two multiply out to the factor on the tin.
        let each = difficulty.each_way();
        stats.health = ((stats.health as f32) * each).round() as i32;
        stats.strength = ((stats.strength as f32) * each).round() as i32;

        let mut haste = 100;
        for passive in difficulty.passives() {
            match passive {
                Passive::Hardened => stats.regen += 4,
                Passive::Warded => {
                    stats.mind_resist += 40;
                    stats.curse_resist += 40;
                    stats.physical_resist += 20;
                    stats.magic_resist += 20;
                }
                Passive::Relentless => haste = 125,
            }
        }
        // Innate attacks first, then anything its gear assembles.
        let mut items: Vec<RunningItem> =
            spec.attacks.iter().map(RunningItem::from_attack).collect();
        items.extend(profiles.iter().map(RunningItem::from_profile));
        if haste != 100 {
            for it in &mut items {
                it.cooldown_ms = ((it.cooldown_ms as i64 * 100 / haste as i64) as u32).max(TICK_MS);
            }
        }
        Combatant {
            name: spec.name.to_string(),
            // **A creature is never an expert.** Classes are applied to the
            // player only — `simulate_with_class` says so — and an empty frame
            // is a thing a *player* left empty; a creature's board is what the
            // dresser wrote and has no bearing on either.
            expert: None,
            productivity: None,
            loud_bought: 0,
            loud_free_left: 0,
            loud_owed: 0,
            facts_standing: 0,
            reqs_left: 0,
            reqs_done: 0,
            opening_until_ms: 0,
            opening_encores: 0,
            opening_banked: false,
            overwound_ms: 0,
            overwound_stacks: 0,
            burn_ms: 0,
            burn_every_ms: 0,
            burn_per_stack: 0,
            burn_keeps_pct: 0,
            fired_left: 0,
            silent_left: 0,
            silent_owed: 0,
            said: 0,
            told_per_curse: 0,
            told_cap: 0,
            boiler_tenths: 0,
            burned: [0; 3],
            unmade_at: None,
            empty_frames: 0,
            max_health: stats.health,
            health: stats.health,
            armor: 0,
            mana: 0,
            druidic_might: 0,
            communion: 0,
            zealotry: 0,
            strength: stats.strength,
            power: stats.power,
            regen: stats.regen,
            mind_resist: stats.mind_resist,
            physical_resist: stats.physical_resist,
            physical_pierce: stats.physical_pierce,
            physical_harden: stats.physical_harden,
            magic_resist: stats.magic_resist,
            magic_pierce: stats.magic_pierce,
            magic_harden: stats.magic_harden,
            reflect: stats.reflect,
            rage: 0,
            faith: 0,
            nature: 0,
            pending: Vec::new(),
            slow_time: 0,
            overflowing: 1,
            leech: 0,
            echo_every: 0,
            bastion: 0,
            contagion: 0,
            reprisal: 0,
            riposte: 0,
            momentum: 0,
            resonance: 1,
            transmute: 0,
            adaptable: 0,
            untimely: 0,
            cascade: 0,
            consecrate: 0,
            misfire_count: 0,
            stun_count: 0,
            untimely_count: 0,
            bloodscent: 0,
            confluence: 0,
            activations: 0,
            curse_resist: stats.curse_resist,
            purse: 0,
            aim: 0,
            no_regen: false,
            slower_pct: 0,
            armour_pct: 100,
            haste_per_s: 0,
            warded_every: 0,
            warded_count: 0,
            curses: Curses::new(),
            empowerment: 0,
            burn_stacks: 0,
            shield: 0,
            insight: 0,
            dread: 0,
            first_blood: false,
            mind_pierce: 0,
            spellblade: 0,
            wrong_sense: false,
            surrendered: 0,
            deflection: 0,
            forking: 0,
            items,
            dot_milli: 0,
            regen_milli: 0,
            burn_acc: 0,
            burn_timer: 0,
            curse_watch_depth: 0,
            curse_on_activate: Vec::new(),
            spin_extra: 0,
            spin_keep: 0,
            spin_every_ms: SPIN_EVERY_MS,
        }
    }

    pub fn is_down(&self) -> bool {
        self.health <= 0 || self.max_health <= 0
    }

    /// Weapon power after mana empowerment: 0.05x per stack per point of mana.
    /// What the resources you are holding are worth right now. Spending them
    /// gives it up, which is the whole tension: a hoarded pool is a standing
    /// bonus, and a spent one is a burst.
    /// One of the four banked pools, by name.
    /// Take on armour, counting whatever Trundle makes a point worth.
    ///
    /// Every route to armour goes through here. It used to be four separate
    /// `armor +=` sites, which is three chances to add a multiplier and one
    /// chance to forget.
    pub fn gain_armor(&mut self, n: i32) -> i32 {
        let got = (n as i64 * self.armour_pct as i64 / 100) as i32;
        self.armor += got;
        got
    }

    pub fn pool(&self, what: crate::piece::Resource) -> i32 {
        use crate::piece::Resource::*;
        match what {
            Mana => self.mana,
            Rage => self.rage,
            Faith => self.faith,
            Nature => self.nature,
            DruidicMight => self.druidic_might,
            Communion => self.communion,
            Zealotry => self.zealotry,
            Insight => self.insight,
        }
    }

    pub fn set_pool(&mut self, what: crate::piece::Resource, v: i32) {
        use crate::piece::Resource::*;
        match what {
            Mana => self.mana = v,
            Rage => self.rage = v,
            Faith => self.faith = v,
            Nature => self.nature = v,
            DruidicMight => self.druidic_might = v,
            Communion => self.communion = v,
            Zealotry => self.zealotry = v,
            Insight => self.insight = v,
        }
    }

    /// What one point of a pool pays, while it is held.
    ///
    /// `held_bonus` is the rulebook for what a banked pool is worth and it has
    /// never been shown to anybody - the glossary describes it in sentences
    /// like "every point adds resistance of both types while held", which is a
    /// sentence about an arrow.
    ///
    /// So the interface draws the arrow, and this is where it gets the numbers
    /// from. Derived by asking `held_bonus` rather than by writing the rates
    /// down a second time: a diagram that disagrees with the function is worse
    /// than no diagram, and the only way to be sure it cannot is to not know
    /// the numbers.
    pub fn pool_pays(what: crate::piece::Resource) -> Stats {
        // An empty player, holding one point of the pool and nothing else,
        // so what comes back is that point and no other term.
        let mut probe = Combatant::player(Stats::ZERO, &[]);
        probe.set_pool(what, 1);
        probe.held_bonus()
    }

    /// The pools a board can actually bank **and** that pay something for
    /// being held.
    ///
    /// Two questions, and neither of them is a screen's. Something in the
    /// catalogue has to be able to *give* you the pool, and holding it has to
    /// *do* something — [`Combatant::pool_pays`] is the second answer and the
    /// catalogue is the first. Mana and insight fail the second: they are
    /// spent and they empower, and neither pays a wearer for sitting on a pile
    /// of it. The three fusions fail the first, because nothing in this
    /// catalogue makes one.
    ///
    /// Derived rather than listed, so a component that starts granting a pool
    /// puts it on the panel without anybody remembering to.
    pub fn pools_worth_holding() -> Vec<crate::piece::Resource> {
        crate::piece::Resource::ALL
            .into_iter()
            .filter(|&r| Combatant::pool_pays(r) != Stats::ZERO)
            .filter(|&r| {
                crate::piece::CATALOG.iter().any(|d| {
                    d.triggers.iter().any(|t| {
                        let mut found = false;
                        crate::piece::walk_actions(t, &mut |a| {
                            if let crate::piece::Action::Gain { what, .. } = a {
                                found |= *what == r;
                            }
                        });
                        found
                    })
                })
            })
            .collect()
    }

    pub fn held_bonus(&self) -> Stats {
        let m = self.overflowing.max(1);
        // **What the furnace took still pays, if the rule says so.**
        // `Rule::BurnKeepsBonus` is the Stoker's only rule and it is the trade
        // made gentler rather than removed: a burned pool pays a share of the
        // standing bonus it would have paid, for the rest of the fight. Without
        // it the trade is what the class is — the bonus you were holding is
        // exactly what you gave up — so this is zero for everybody who has not
        // bought the node.
        let kept = |i: usize| self.burned[i] * self.burn_keeps_pct.clamp(0, 100) / 100;
        let (rage, faith, nature) = (
            (self.rage + kept(0)) * m,
            (self.faith + kept(1)) * m,
            (self.nature + kept(2)) * m,
        );
        // A fusion pays both its parents, each at double the parent's rate.
        // Written out rather than derived from `parents()` because the rates
        // are the design and reading them off a table hides what they are.
        let (might, comm, zeal) =
            (self.druidic_might * m, self.communion * m, self.zealotry * m);
        Stats {
            // Fury sharpens the blade.
            physical_damage: rage + might * 2 + zeal * 2,
            // Conviction turns aside both kinds of harm, and no longer stops
            // at forty percent. The cap meant a faith build hit a ceiling it
            // could not see and everything banked past it was dead weight -
            // which is the opposite of what a pool is for.
            physical_resist: faith * 2 + comm * 4 + zeal * 4,
            magic_resist: faith * 2 + comm * 4 + zeal * 4,
            // Growth knits you back together.
            regen: nature + might * 2 + comm * 2,
            ..Stats::ZERO
        }
    }

    /// Regeneration a second, pools included.
    ///
    /// `held_bonus` has always computed this correctly and nothing has ever
    /// read it. The one call site takes `.physical_damage` and throws the rest
    /// away, so rage reached a fight and **nature and faith did not**: the
    /// regen tick read the flat `regen` field, `take_typed` read the flat
    /// resists, and a hundred banked nature was worth exactly nothing. The
    /// unit test that was supposed to cover it asserts `held_bonus()` directly
    /// and so has been green throughout, testing arithmetic nobody consulted.
    ///
    /// Named `effective_*`, beside `effective_power`, because that is the shape
    /// this file already uses for "the number after everything the wearer
    /// brings".
    pub fn effective_regen(&self) -> i32 {
        self.regen + self.held_bonus().regen
    }

    pub fn effective_physical_resist(&self) -> i32 {
        self.physical_resist + self.held_bonus().physical_resist
    }

    pub fn effective_magic_resist(&self) -> i32 {
        self.magic_resist + self.held_bonus().magic_resist
    }

    /// Weapon power on a **magic** hit: 0.05x per stack per point of mana.
    pub fn effective_power(&self) -> i32 {
        self.power + self.magic_empower()
    }

    /// Weapon power on a **physical** hit: 0.50x flat per Spellblade stack.
    pub fn effective_physical_power(&self) -> i32 {
        self.power + self.physical_empower()
    }

    /// What empowerment adds to a magic hit, in power-hundredths.
    pub fn magic_empower(&self) -> i32 {
        self.empowerment as i32 * 5 * self.mana.max(0)
    }

    /// What Spellblade and the furnace add to a physical hit, in
    /// power-hundredths.
    ///
    /// Spellblade's half is flat, and that is its design: half a multiplier a
    /// stack, whatever the board is holding.
    ///
    /// **The furnace's half is the same sum `magic_empower` does**, and it is
    /// here so that a Kettle-Stoker with a blade gets what its promise says.
    /// See [`Combatant::burn_stacks`]: only `stoke` ever writes that, so this
    /// changes nothing for any other source of empowerment.
    pub fn physical_empower(&self) -> i32 {
        self.spellblade as i32 * SPELLBLADE_POWER
            + self.burn_stacks as i32 * 5 * self.mana.max(0)
    }

    /// Flat reduction the mana shield applies to an incoming **magic** hit.
    pub fn damage_reduction(&self) -> i32 {
        self.shield as i32 * self.mana.max(0)
    }

    /// Flat reduction Deflection applies to an incoming **physical** hit.
    pub fn physical_reduction(&self) -> i32 {
        self.deflection as i32 * DEFLECTION_FLAT
    }

    /// What Dread adds to every point of mind damage this fighter deals.
    ///
    /// Zero without the pool and zero without the stacks, which is the whole
    /// of the third lane's bargain and the same one the first lane has.
    /// What the wrong sense makes of one point of mind damage.
    ///
    /// The multiplier is the damage this board has already given up, over
    /// `WRONG_SENSE_PER`, and it is capped: an uncapped conversion is a board
    /// that gets stronger for every second it fails to kill anything, which is
    /// a fight decided by the clock rather than by either board.
    ///
    /// Without the crest it is the identity, so nothing that does not wear one
    /// pays a tick for it.
    pub fn wrong_sense_multiplied(&self, mind: i32) -> i32 {
        if !self.wrong_sense || mind <= 0 {
            return mind;
        }
        let steps = (self.surrendered / WRONG_SENSE_PER as i64).min(WRONG_SENSE_CAP as i64);
        ((mind as i64) * (100 + steps * WRONG_SENSE_STEP as i64) / 100).max(0) as i32
    }

    pub fn mind_bonus(&self) -> i32 {
        self.dread as i32 * self.insight.max(0) / DREAD_DIVISOR
    }

    /// Mana shield first, then armour, then health. Returns (absorbed by
    /// armour, through to health).
    /// Take `amount` of `kind`, from an attacker with `pierce` percent
    /// piercing of that type.
    /// Public because the lanes are a rule rather than an implementation
    /// detail: `typed_lanes.rs` asks this directly, which is the only way to
    /// put one number in and read what each lane did to it.
    pub fn take_typed(&mut self, amount: i32, kind: DamageType, pierce: i32) -> (i32, i32) {
        self.take_typed_with(amount, kind, pierce, false)
    }

    /// The same, with the option of walking past the flat answer entirely.
    ///
    /// `unstoppable` is Wumpus Hunter's first blow, and it is the only thing
    /// in the game that skips a shield or a deflection rather than reducing
    /// what is left after one.
    pub fn take_typed_with(
        &mut self,
        amount: i32,
        kind: DamageType,
        pierce: i32,
        unstoppable: bool,
    ) -> (i32, i32) {
        let amount = match kind {
            DamageType::Physical => crate::stats::after_defences(
                amount,
                self.effective_physical_resist(),
                pierce,
                self.physical_harden,
            ),
            DamageType::Magic => crate::stats::after_defences(
                amount,
                self.effective_magic_resist(),
                pierce,
                self.magic_harden,
            ),
        };
        // Each lane has its own flat answer, and neither answers the other:
        // the mana shield takes magic, Deflection takes physical. Before this
        // the shield took everything, which is what made the mana pair the
        // only defensive stack worth owning.
        let amount = if unstoppable {
            amount
        } else {
            match kind {
                DamageType::Physical => (amount - self.physical_reduction()).max(0),
                DamageType::Magic => (amount - self.damage_reduction()).max(0),
            }
        };
        if amount <= 0 {
            return (0, 0);
        }
        if self.slow_time > 0 {
            // Nothing lands now. It arrives in slices over the next few
            // seconds, which is time for armour and regeneration to answer.
            self.pending.push((amount, self.slow_time * 1000));
            return (0, 0);
        }
        let absorbed = amount.min(self.armor.max(0));
        self.armor -= absorbed;
        let through = amount - absorbed;
        self.health -= through;
        // A wall that rebuilds itself under fire.
        if self.bastion > 0 && absorbed > 0 {
            self.gain_armor(absorbed * self.bastion / 100);
        }
        // Being ground down is itself a resource.
        if self.reprisal > 0 {
            self.faith += self.reprisal;
        }
        (absorbed, through)
    }

    /// Mind damage eats maximum health, so it can never be healed back off.
    pub fn take_mind(&mut self, raw: i32) -> i32 {
        self.take_mind_pierced(raw, 0)
    }

    /// The same, with a share of the resistance walked straight through.
    pub fn take_mind_pierced(&mut self, raw: i32, pierce: i32) -> i32 {
        // The mind lane's only answer is `mind_resist`, which is the helmet's,
        // and that is deliberate. The mana shield used to blunt this too -
        // "whatever the damage type" - which made mana the answer to two lanes
        // out of three. Three lanes, three answers: the shield takes magic,
        // Deflection takes physical, and mind resistance takes this.
        let left = self.mind_resist - (self.mind_resist * pierce.clamp(0, 100)) / 100;
        let dealt = mind_damage_after_resist(raw, left);
        if dealt <= 0 {
            return 0;
        }
        self.max_health = (self.max_health - dealt).max(0);
        if self.health > self.max_health {
            self.health = self.max_health;
        }
        // **The unmaking, and it is here rather than at the tick** because this
        // is the one place a maximum falls. `is_down` has fired at
        // `max_health <= 0` since the fork, so the class does not invent a way
        // to die — it moves where the line is.
        if let Some(floor) = self.unmade_at {
            // **Told Once: every curse standing on them raises it**, and it is
            // read here rather than set at the bell because what it reads is a
            // fact about *now*. The two knobs are copied onto the sufferer at
            // the bell, which is the only way a fighter can know what has been
            // said about it: this function has one combatant and no view of
            // the room.
            // **Percentage points of the maximum, not points of health.**
            // `PLAN-M16.md` §12.8 says *raises the unmaking third by three
            // points*, and a "point" of a threshold is a percentage point —
            // eight raw points against a maximum in the thousands is noise, and
            // a knob that moves noise is a knob nobody can watch move.
            let standing: i32 = self.curses.iter().map(|c| c.stacks as i32).sum();
            let pts = (self.told_per_curse * standing).min(self.told_cap);
            let raised = floor + self.max_health * pts / 100;
            if self.max_health <= raised {
                self.health = 0;
            }
        }
        dealt
    }
}

// ----------------------------------------------------------------- log

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// An item finished its cooldown. Always precedes that item's effects.
    /// `index` is the item's position in its owner's list, so two items with
    /// the same name stay distinguishable.
    Activate { side: Side, item: String, index: usize },
    /// Rage, faith or nature banked.
    GainResource { side: Side, what: &'static str, amount: i32, total: i32, accrued: bool },
    /// A pool taken off someone. `amount` is what was actually there to take.
    Drained { on: Side, what: &'static str, amount: i32, total: i32 },
    /// Run gold spent mid-fight. `remaining` is what is left in the purse,
    /// which is what you will arrive at the shop with.
    Spent { side: Side, amount: i32, remaining: i32 },
    /// The fight has gone on long enough and is now ending itself. `pct` is
    /// the share of maximum health both sides are losing this second.
    SuddenDeath { pct: i32 },
    /// A blow, and **which of the swinger's items threw it**.
    ///
    /// `damage` is the swing before any of the defender's answers, which is
    /// deliberate - see the note where it is pushed. `by_item` indexes the
    /// swinger's own item list, the same index `Activate` reports, so a screen
    /// can put the number beside the row that earned it. `None` is a blow no
    /// item owns.
    ///
    /// **What an item hits for moves during a fight, and carrying the index is
    /// what lets a screen say so.** Held fury is added to every swing and a
    /// spin lifts the item's own power, so the tenth second is not the
    /// first - and a row printing one figure for the whole fight was printing
    /// the opening estimate for ever.
    Hit {
        by: Side,
        by_item: Option<usize>,
        damage: i32,
        absorbed: i32,
        target_health: i32,
        target_armor: i32,
    },
    /// An item came round and nothing happened - a misfire ate it.
    Misfired { side: Side, item: String },
    /// An attack was warded off before it landed. Ticket to Ride.
    Warded { side: Side, item: String },
    /// A spell went off. `paid` says whether it was cast in full or weakly.
    Cast { side: Side, paid: bool, cost: i32, remaining: i32 },
    /// Maximum health grew mid-fight.
    /// `paid_armor` is what was spent to buy the growth, which is nonzero only
    /// for `Ballast`. A field rather than a second event, so every reader of
    /// `Grew` - `settle`'s growth banking among them - keeps working and the
    /// one that wants to know reads the field.
    Grew { side: Side, amount: i32, total: i32, paid_armor: i32 },
    MindHit { by: Side, amount: i32, target_max_health: i32 },
    GainArmor { side: Side, amount: i32, total: i32 },
    GainMana { side: Side, amount: i32, total: i32, accrued: bool },
    /// `paid` says which branch of a mana trigger ran.
    ManaCheck { side: Side, cost: i32, paid: bool, remaining: i32 },
    /// A spend against rage, faith or nature.
    ResourceCheck { side: Side, what: &'static str, cost: i32, paid: bool, remaining: i32 },
    /// `stacks` is the count *after* this one landed, so the interface can
    /// say "curse of searing x3" without keeping its own tally.
    Cursed { on: Side, kind: CurseKind, duration_ms: u32, stacks: u32 },
    /// An item fired for the last time and stopped.
    ///
    /// Its own event for the reason `Stunned` is one: the interface needs to
    /// know *which* item, because a bar that stops with nothing said about it
    /// reads as a bug in the playback rather than as the thing the player
    /// bought. `index` is that item's position in its owner's list.
    Broke { side: Side, index: usize, item: String },
    /// A stun stopped one item. Its own event rather than a `Cursed`, because
    /// a stun rides on an item and the interface needs to know which one:
    /// `index` is that item's position in its owner's list, and `duration_ms`
    /// is the whole time it is now stopped for.
    Stunned { on: Side, index: usize, item: String, duration_ms: u32, aimed: bool },
    /// Damage-over-time landing this tick.
    Burn { side: Side, damage: i32, health: i32 },
    Regen { side: Side, amount: i32, health: i32 },
    /// A reaction pushed an item's cooldown forward.
    Hastened { side: Side, item: String, by_ms: u32 },
    /// A spinning item turned where it stands. `to` indexes its turn cycle and
    /// `stacks` is the count *after* this turn, so the interface can draw the
    /// orientation and say what it is worth without keeping a tally.
    ///
    /// **Logged rather than left to the clock.** A frosted item turns slower
    /// for the same reason it fires slower, so a screen dividing the playback
    /// head by a second would draw an orientation the fight never had — the
    /// same class of mistake as subtracting damage from a health total.
    Turned { side: Side, index: usize, item: String, to: u32, stacks: u32 },
    /// A spinning item spent everything it had banked. `pct` is what it was
    /// worth on this one activation, in hundredths of power.
    Spun { side: Side, index: usize, item: String, stacks: u32, pct: i32 },
    /// Time moved from one bar to another on the same board.
    ///
    /// Both names, because the whole of what a shunt does is take from one and
    /// give to the other, and a log that says only where it landed reads as
    /// free haste.
    Shunted { side: Side, from: String, to: String, ms: u32 },
    /// The enemy's best item, caught at the top of its swing and set back.
    Derailed { side: Side, item: String, by_ms: u32 },
    /// Armour turned a blow back on whoever threw it.
    Reflected { side: Side, damage: i32 },
    /// Two pools became one of a fused pool. `total` is what is now held of
    /// it, and `from`/`and` are the parents with what each has left.
    ///
    /// The parents are named here rather than left implicit because two pools
    /// going down with no line to explain them is exactly the sort of thing a
    /// player notices and cannot account for. A fusion is the only action in
    /// the game that spends something it was not asked for.
    Fused {
        side: Side,
        what: &'static str,
        total: i32,
        from: (&'static str, i32),
        and: (&'static str, i32),
    },
    /// A watcher counted something. `seen` is where its counter stands
    /// afterwards, out of `count`, and `paid` is whether that sighting was the
    /// one that came round.
    ///
    /// Logged on **every** sighting rather than only on the payout, and that is
    /// the whole reason it carries numbers. A watcher runs on the board's clock
    /// rather than its own, so its counter is the only thing on the row that
    /// says when it will pay - and the interface replays a log rather than the
    /// fight, so a count the log does not record is a count the interface
    /// cannot draw. It read zero for the whole fight, because the combatant a
    /// log stores is the one from *before* it.
    /// `what` is the relation itself rather than a sentence about it. A log
    /// entry storing prose is a log entry that cannot be re-worded, themed, or
    /// pluralised by whoever is drawing it - and the wording is exactly what
    /// was wrong here.
    Watched { side: Side, item: String, what: Watched, seen: u32, count: u32, paid: bool },
    /// A mana buff gained stacks. `total` is the new stack count.
    Empowered { side: Side, total: u32, power_bonus: i32 },
    Shielded { side: Side, total: u32, reduction: i32 },
    /// The physical twins. Same shape as the pair above, because they are the
    /// same pair in the other lane.
    Whetted { side: Side, total: u32, power_bonus: i32 },
    /// The mind lane's stack. `mind_bonus` is what it currently works out to
    /// against the Insight held, which is nothing until there is some.
    Dreading { side: Side, total: u32, mind_bonus: i32 },
    Deflecting { side: Side, total: u32, reduction: i32 },
    /// Spell forking gained. Every cast lands once more per stack.
    Forking { side: Side, total: u32 },
    /// A Stoker's furnace took a shovelful. **The pool falling and the stacks
    /// rising on the same line**, so the replay can draw both on the tick they
    /// happened rather than inferring one from the other — which is the *page
    /// draws numbers core sent it* rule, stated for a number that moves twice.
    Burned { side: Side, what: &'static str, points: i32, left: i32, stacks: u32 },
    /// A creature whose maximum health went under a Whisperer's threshold.
    ///
    /// Its own event rather than a `Fell`, because *how* it ended is the whole
    /// of what the class bought — and `Fell` is still logged after it, since it
    /// is still down.
    ///
    /// **The maximum it is at and the line it went under**, which are the two
    /// numbers in the room. Neither is the maximum it *started* with, and that
    /// is deliberate: storing a starting maximum on every combatant to spell
    /// out a log line is a field the fight would not otherwise have.
    Unmade { side: Side, at: i32, under: i32 },
    Fell { side: Side },
    End { outcome: Outcome },
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub at_ms: u32,
    pub event: Event,
    /// Which foe this entry is about, when the fight has more than one of
    /// them. Zero in a duel, and zero for anything the player did to himself.
    ///
    /// One field here rather than a `who` on each of the twenty-odd `Event`
    /// variants that name a side. It is unambiguous because the player is
    /// always singular: when a foe acts this is the actor, when the player
    /// acts on a foe this is the victim, and there is never a third party.
    pub who: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Outcome {
    Victory,
    Defeat,
    Stalemate,
}

impl Outcome {
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Victory => "VICTORY",
            Outcome::Defeat => "DEFEAT",
            Outcome::Stalemate => "STALEMATE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CombatLog {
    pub player: Combatant,
    /// Everything on the other side, in the order they were written. Almost
    /// every fight has exactly one; `enemy()` is the shorthand for that case.
    pub enemies: Vec<Combatant>,
    /// The monsters fought, so the interface can lay their gear out beside
    /// yours without having to guess which rung the run has moved on to.
    pub specs: Vec<MonsterSpec>,
    pub entries: Vec<LogEntry>,
    pub outcome: Outcome,
    pub duration_ms: u32,
    /// Run gold the player spent during the fight. The run deducts it when
    /// the fight settles - the simulation never touches `Run::gold` itself.
    pub gold_spent: i32,
    /// What was standing on the other side when it went down.
    ///
    /// **Written where the truth is.** `player` and `enemies` are the fighters
    /// *as the bell went* — that is what makes a replay replayable — so the
    /// live curse state at the end is nowhere in this log unless it is put
    /// here. Settlement needs it (Eleventh Season bills by the curse), and the
    /// alternative was `fight::settle` walking `entries` and re-deriving which
    /// curses had expired by the last timestamp, which is a second copy of
    /// what the simulation already knew and would have gone stale the first
    /// time a curse changed how it ends.
    pub curse_bill: CurseBill,
}

/// Curses landed on the other side, and how they stood at the end.
///
/// One struct rather than three fields on the log, because they are one fact
/// asked from one place, and the next settlement rule that wants a fourth
/// should find them together.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CurseBill {
    /// Stacks still running on the fallen when the fight ended.
    pub standing: u32,
    /// **Which** kinds were still running, each once.
    ///
    /// The list rather than the count, because two things read it and they
    /// want different halves: Eleventh Season wants *how many different*, and
    /// Standing Fact's `told` wants *which*. `log.enemies` cannot answer either
    /// — those are the fighters as the bell went, which is what makes a replay
    /// replayable — so this is the only place the ending state exists.
    pub kinds: Vec<crate::curse::CurseKind>,
    /// Curses that landed on it and had burned off before the bell.
    ///
    /// Landings minus what is still standing, floored at zero — so a long
    /// fight where everything expired reads high and a short one reads zero,
    /// which is the shape the rule that reads it is rescuing.
    pub expired: u32,
}

impl CombatLog {
    /// The creature you were fighting, when there was only the one - which is
    /// every fight except the handful an event sets up.
    pub fn enemy(&self) -> &Combatant {
        &self.enemies[0]
    }

    pub fn spec(&self) -> &MonsterSpec {
        &self.specs[0]
    }

    /// Is this a fight with more than one thing in it?
    pub fn is_brawl(&self) -> bool {
        self.enemies.len() > 1
    }

    /// A win with no fight in it. For the ladder picker and for tests, where
    /// what is under test is the settlement rather than the simulation.
    pub fn won_by_default(spec: &MonsterSpec) -> CombatLog {
        CombatLog {
            // Nothing was fought, so nothing was cursed. A rout pays what a
            // win pays and this is the honest bill for a fight that did not
            // happen — not a zero standing in for one nobody counted.
            curse_bill: CurseBill::default(),
            player: Combatant::player(Stats::base_character(), &[]),
            enemies: vec![Combatant::monster_at(spec, Difficulty::Medium)],
            specs: vec![*spec],
            entries: Vec::new(),
            outcome: Outcome::Victory,
            duration_ms: 0,
            gold_spent: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn who(&self, s: Side) -> &str {
        match s {
            Side::Player => &self.player.name,
            Side::Enemy => &self.enemy().name,
        }
    }

    /// One line of plain text, for the CLI and the on-screen log.
    pub fn describe(&self, e: &LogEntry) -> String {
        let t = format!("{:>5.1}s", e.at_ms as f32 / 1000.0);
        match &e.event {
            Event::Activate { side, item, .. } => {
                format!("{} {} activates {}", t, self.who(*side), item)
            }
            // **Counted, never dramatised.** What went in, what is left, and
            // what came out of it — the three numbers a person reading this
            // line came for.
            Event::Burned { side, what, points, left, stacks } => format!(
                "{} {} shovels {} {} into the furnace ({} left, {} stacks)",
                t,
                self.who(*side),
                points,
                what,
                left,
                stacks
            ),
            Event::Unmade { side, at, under } => format!(
                "{} {} is unmade: {} maximum health left, against the {} it takes",
                t,
                self.who(*side),
                at,
                under
            ),
            Event::Broke { side, item, .. } => {
                format!("{} {}'s {} comes apart and stops", t, self.who(*side), item)
            }
            Event::Grew { side, amount, total, paid_armor } if *paid_armor > 0 => format!(
                "{} {} beds {} of its armour down into {} more to lose ({} max health)",
                t,
                self.who(*side),
                paid_armor,
                amount,
                total
            ),
            Event::Grew { side, amount, total, .. } => format!(
                "{} {} grows {} tougher ({} max health)",
                t,
                self.who(*side),
                amount,
                total
            ),
            Event::Misfired { side, item } => {
                format!("{} {}'s {} misfires and does nothing", t, self.who(*side), item)
            }
            Event::Warded { side, item } => {
                format!("{} {}'s {} misses entirely", t, self.who(*side), item)
            }
            Event::Cast { side, paid, cost, remaining } => {
                if *paid {
                    format!(
                        "{} {} spends {} mana and casts in full ({} left)",
                        t,
                        self.who(*side),
                        cost,
                        remaining
                    )
                } else {
                    format!(
                        "{} {} has no mana - the spell lands weakly",
                        t,
                        self.who(*side)
                    )
                }
            }
            Event::ResourceCheck { side, what, cost, paid, remaining } => format!(
                "{} {} {} {} {} ({} left)",
                t,
                self.who(*side),
                if *paid { "spends" } else { "cannot pay" },
                cost,
                what,
                remaining
            ),
            Event::GainResource { side, what, amount, total, accrued } => {
                let how = if *accrued { " on what it was holding" } else { "" };
                format!("{} {} gains {} {}{} ({})", t, self.who(*side), amount, what, how, total)
            }
            Event::Reflected { side, damage } => {
                format!("{} {} turns back {}", t, self.who(*side), damage)
            }
            Event::Fused { side, what, total, from, and } => format!(
                "{} {} fuses 1 {} ({}) - {} {} and {} {} left",
                t,
                self.who(*side),
                what,
                total,
                from.1,
                from.0,
                and.1,
                and.0
            ),
            Event::Watched { side, item, what, seen, count, paid } => {
                if *paid {
                    format!(
                        "{} {}'s {} has counted its {}",
                        t,
                        self.who(*side),
                        item,
                        what.counted(*count)
                    )
                } else {
                    // "3 of 8" and then what they are, said once. The phrase
                    // carries its own number, so the count goes in front of it
                    // rather than being bolted to a noun that cannot take one.
                    format!(
                        "{} {}'s {} counts {} of {}",
                        t,
                        self.who(*side),
                        item,
                        seen % count.max(&1),
                        what.counted(*count)
                    )
                }
            }
            Event::Hit { by, by_item: _, damage, absorbed, target_health, target_armor } => {
                let soak = if *absorbed > 0 {
                    format!(" ({} soaked, {} armor left)", absorbed, target_armor)
                } else {
                    String::new()
                };
                format!(
                    "{} {} hits {} for {}{} -> {} hp",
                    t,
                    self.who(*by),
                    self.who(by.other()),
                    damage,
                    soak,
                    (*target_health).max(0)
                )
            }
            Event::MindHit { by, amount, target_max_health } => format!(
                "{} {} deals {} MIND damage -> max hp now {}",
                t,
                self.who(*by),
                amount,
                target_max_health
            ),
            Event::GainArmor { side, amount, total } => {
                format!("{} {} gains {} armor ({})", t, self.who(*side), amount, total)
            }
            Event::GainMana { side, amount, total, accrued } => {
                let how = if *accrued { " on what it was holding" } else { "" };
                format!("{} {} gains {} mana{} ({})", t, self.who(*side), amount, how, total)
            }
            Event::ManaCheck { side, cost, paid, remaining } => {
                if *paid {
                    format!("{} {} spends {} mana ({} left)", t, self.who(*side), cost, remaining)
                } else {
                    format!(
                        "{} {} cannot pay {} mana (has {})",
                        t,
                        self.who(*side),
                        cost,
                        remaining
                    )
                }
            }
            Event::Stunned { on, item, duration_ms, aimed, .. } => format!(
                "{} {}{}'s {} is stunned for {:.1}s",
                t,
                if *aimed { "picks out " } else { "" },
                self.who(*on),
                item,
                *duration_ms as f32 / 1000.0
            ),
            Event::SuddenDeath { pct } => {
                format!("{} the fight turns - {}% of everyone, and rising", t, pct)
            }
            Event::Spent { side, amount, remaining } => format!(
                "{} {} spends {} gold ({} left)",
                t,
                self.who(*side),
                amount,
                remaining
            ),
            Event::Drained { on, what, amount, total } => format!(
                "{} {} loses {} {} ({} left)",
                t,
                self.who(*on),
                amount,
                what,
                total
            ),
            Event::Cursed { on, kind, duration_ms, stacks } => format!(
                "{} curse of {}{} on {} for {:.1}s",
                t,
                kind.name(),
                if *stacks > 1 { format!(" x{}", stacks) } else { String::new() },
                self.who(*on),
                *duration_ms as f32 / 1000.0
            ),
            Event::Burn { side, damage, health } => format!(
                "{} {} burns for {} -> {} hp",
                t,
                self.who(*side),
                damage,
                (*health).max(0)
            ),
            Event::Regen { side, amount, health } => {
                format!("{} {} regenerates {} -> {} hp", t, self.who(*side), amount, health)
            }
            Event::Shunted { side, from, to, ms } => format!(
                "{} {}'s {} hands {:.1}s to {}",
                t,
                self.who(*side),
                from,
                *ms as f32 / 1000.0,
                to
            ),
            Event::Derailed { side, item, by_ms } => format!(
                "{} {} catches {} at the top of its swing and sets it back {:.1}s",
                t,
                self.who(*side),
                item,
                *by_ms as f32 / 1000.0
            ),
            Event::Turned { side, item, to, stacks, .. } => format!(
                "{} {}'s {} turns (position {}, {} banked)",
                t,
                self.who(*side),
                item,
                to,
                stacks
            ),
            Event::Spun { side, item, stacks, pct, .. } => format!(
                "{} {}'s {} spends {} turns (+{}.{:02}x power on this one)",
                t,
                self.who(*side),
                item,
                stacks,
                pct / 100,
                pct % 100
            ),
            Event::Hastened { side, item, by_ms } => format!(
                "{} {}'s {} hastened by {:.1}s",
                t,
                self.who(*side),
                item,
                *by_ms as f32 / 1000.0
            ),
            Event::Empowered { side, total, power_bonus } => format!(
                "{} {} empowered x{} (+{}.{:02}x power on magic)",
                t,
                self.who(*side),
                total,
                power_bonus / 100,
                power_bonus % 100
            ),
            Event::Dreading { side, total, mind_bonus } => format!(
                "{} {} dread x{} (+{} per point of mind)",
                t,
                self.who(*side),
                total,
                mind_bonus
            ),
            Event::Whetted { side, total, power_bonus } => format!(
                "{} {} spellblade x{} (+{}.{:02}x power on iron)",
                t,
                self.who(*side),
                total,
                power_bonus / 100,
                power_bonus % 100
            ),
            Event::Forking { side, total } => format!(
                "{} {} spell forking x{} (every cast lands {} times)",
                t,
                self.who(*side),
                total,
                total + 1
            ),
            Event::Shielded { side, total, reduction } => format!(
                "{} {} mana shield x{} (-{} per magic hit)",
                t,
                self.who(*side),
                total,
                reduction
            ),
            Event::Deflecting { side, total, reduction } => format!(
                "{} {} deflection x{} (-{} per physical hit)",
                t,
                self.who(*side),
                total,
                reduction
            ),
            Event::Fell { side } => format!("{} {} falls!", t, self.who(*side)),
            Event::End { outcome } => format!("-- {} --", outcome.label()),
        }
    }
}

// ------------------------------------------------------------ simulate

/// Run the whole fight to completion.
///
/// Each [`TICK_MS`] slice, in strict order:
///   1. curses burn, then regeneration heals, on both sides
///   2. curse timers advance and expired curses drop
///   3. every item advances its cooldown — slowed if its owner is frosted —
///      and activates if full. The player's items resolve before the enemy's,
///      and within a side they resolve in loadout order.
///   4. deaths are checked
///
/// Nothing here consults a random number generator.
/// What the player brings to the bell that is not in their stats.
///
/// Armour and mana are grants an *item* makes on its own tick everywhere else
/// in the engine, so the character-level totals of them describe nothing and
/// [`Combatant::player`] has always started both at zero. The skill tree is the
/// one thing that hands them out standing rather than per activation, so it
/// passes them here — beside `Stats` rather than inside it, because folding
/// them in would pay every item's armour a second time as a starting balance.
///
/// `rules` arrived with the tree's fifth effect kind and goes through the same
/// door for the same reason: **a granted rule is a fight input, like a class
/// power, and not a mutable global.** Combat stays a pure function of what it
/// was handed. It costs this type its `Copy`, which is the whole of the price.
/// How slow `Rule::Productivity` may ever make one item.
///
/// **A cap, because the rule stacks with itself.** Every proc adds, and an
/// item a hundred percent slower is a stopped item — which is a stun the player
/// bought for themselves and cannot take off. Seventy-five is three procs of
/// the shipped twenty-five and five of the shipped fifteen, which is a long
/// fight, and past it the trade stops being one.
pub const MAX_PRODUCTIVITY_SLOW_PCT: u32 = 75;

/// What a funnel will hold before it spills, in Funny.
///
/// **Ten casts' worth**, because the pool it fills is the one a cast spends
/// and the number a player can hold in their head about mana is *how many
/// casts is that*. Only `PatentedFunnel`'s `overflow` reads it — mana has no
/// cap anywhere else in this engine and must not grow one, because every
/// casting item in the game is priced against a pool that only goes up.
pub const FUNNEL_HOLD: i32 = SPELL_MANA_COST * 10;

/// Turn a fighter's empty frames, once a tick's worth of time.
///
/// **Overwound Arm, and it is the only thing in the game that reads
/// `empty_frames`.** A frame with nothing in it produces no `ItemProfile`, so
/// there is no item to hang a spin on and no cooldown for it to ride; it turns
/// on the fighter's own clock instead, at the rate the fighter's spin runs at,
/// and what it banks goes straight into bare strength.
///
/// `half` is tenths of a stack a turn, so the arithmetic is integer the whole
/// way down — the same reason every roll in this game is per-mille.
/// The Stoker's furnace: shovel the largest held pool into empowerment.
///
/// **On the fighter's own clock, beside the spin's**, and here rather than
/// inside the item loop for the reason `overwind` is: a fighter with four items
/// must not stoke four times a tick. There is no item; there is a furnace.
///
/// **The largest, and only ever one of them.** `pools_worth_holding` returns
/// rage, faith and nature — checked at the call site rather than assumed — and
/// never mana or insight, which are spent already and pay nothing for being
/// held. Ties go to the earliest, so two equal pools burn in a fixed order and
/// a fight replays identically.
///
/// What it takes is recorded in `burned`, so `Rule::BurnKeepsBonus` can pay a
/// share of a standing bonus the pool is no longer there to pay.
fn stoke(c: &mut Combatant, who: u8, side: Side, t: u32, log: &mut Vec<LogEntry>) {
    use crate::piece::Resource;
    if c.burn_every_ms == 0 || c.burn_per_stack <= 0 {
        return;
    }
    c.burn_ms += TICK_MS;
    while c.burn_ms >= c.burn_every_ms {
        c.burn_ms -= c.burn_every_ms;
        const POOLS: [(Resource, &str); 3] =
            [(Resource::Rage, "rage"), (Resource::Faith, "faith"), (Resource::Nature, "nature")];
        let Some((at, &(what, name))) = POOLS
            .iter()
            .enumerate()
            .max_by_key(|(i, (r, _))| (c.pool(*r), std::cmp::Reverse(*i)))
        else {
            return;
        };
        let have = c.pool(what);
        if have <= 0 {
            // **An empty hopper buys nothing.** A furnace with nothing in it is
            // a furnace, not a free stack — which is the whole reason the class
            // asks you to decide which pool to bank.
            continue;
        }
        let took = have.min(c.burn_per_stack);
        c.set_pool(what, have - took);
        c.burned[at] += took;
        c.empowerment += 1;
        c.burn_stacks += 1;
        // **The furnace does not bank mana, and that is Fired Funnel's.**
        // A first draft had it bank what it shovelled, which made the class
        // work and quietly took the expert's whole promise — *every stack the
        // furnace buys is also mana* is not a promise if the furnace already
        // does it. `every_point_in_an_expert_tree_buys_something` said so on
        // the next run, naming `ff-twice-through` as a point the tree sells
        // and the engine never reads. **An expert's power is its own**, and
        // the lint is the thing that keeps saying so.
        // **Fired Funnel: the furnace pays the funnel.** Every stack it buys
        // is also mana, up to a budget the fight starts with — here rather
        // than at the cast, because what is being exchanged is the *stack*
        // and this is where a stack is bought.
        if let Some(crate::expert::ExpertPower::FiredFunnel { worth, .. }) = c.expert {
            if c.fired_left > 0 && worth > 0 {
                c.fired_left -= 1;
                c.mana += worth;
            }
        }
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::Burned {
                side,
                what: name,
                points: took,
                left: have - took,
                stacks: c.empowerment,
            },
        });
    }
}

fn overwind(c: &mut Combatant, every_ms: u32) {
    use crate::expert::ExpertPower::OverwoundArm;
    let Some(OverwoundArm { half, ceiling, carry }) = c.expert else { return };
    if c.empty_frames == 0 || half <= 0 {
        return;
    }
    c.overwound_ms += TICK_MS;
    while c.overwound_ms >= every_ms {
        c.overwound_ms -= every_ms;
        // Every bare frame turns, so five of them wind five times as fast —
        // which is the joke the class is making and the reason it is priced
        // against a board you did not fill.
        let gained = half * c.empty_frames as i32;
        let before = c.overwound_stacks;
        c.overwound_stacks = (c.overwound_stacks + gained).min(ceiling.max(0) * 10);
        // Tenths in, whole points of strength out: the pile is kept in tenths
        // and only the crossings are paid, so nothing is lost to rounding and
        // nothing is paid twice.
        let paid = c.overwound_stacks / 10 - before / 10;
        c.strength += paid;
        // `carry` is what survives an activation, and an empty frame has no
        // activation — so it is what survives the *ceiling*: stacks past the
        // top are kept rather than thrown away, which is what keeps a long
        // fight paying.
        if c.overwound_stacks >= ceiling.max(0) * 10 && carry > 0 {
            c.overwound_stacks -= carry.min(ceiling.max(0)) * 10;
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Held {
    pub armor: i32,
    pub mana: i32,
    /// Rules the skill tree granted. Translated into combatant fields below,
    /// the same way a `ClassPower` is.
    pub rules: Vec<crate::skills::Rule>,
    /// Curses that outlived the last fight and land on whatever you meet next.
    ///
    /// **The one thing in this block that reaches past the bell**, and it is
    /// Standing Fact's capstone doing it — the pair in one sentence: the
    /// Keeper's permanence and the Gorillathon's opening. It rides in `Held`
    /// like every other fight input, so combat stays a pure function of what it
    /// was handed; what fills it is `Character::told_curses`, which is the only
    /// new save field this block needed.
    pub told: Vec<crate::curse::CurseKind>,
    /// How many worn frames the player left empty.
    ///
    /// **Counted where the board is and carried in, never derived here.** See
    /// [`Combatant::empty_frames`] for why the fight cannot answer it itself.
    /// `Character::start_with` fills it; everything that builds a `Held` by
    /// hand gets zero, which is the honest answer for a fixture with no board.
    pub empty_frames: u32,
    /// Empowerment stacks already on you at the bell.
    ///
    /// **`Rule::BurnCarries` is the only thing that fills it**, and it is in
    /// `Held` rather than in `Stats` for the reason `armor` and `mana` are: a
    /// stack is a thing you are already holding, not a rate anything pays.
    pub empowerment: u32,
    /// Pools already banked at the bell, and the two things a mind build spends.
    ///
    /// **`Held` and not `Stats`**, which is the division this struct exists to
    /// make: a `Stats` figure is a rate something pays and a `Held` figure is a
    /// quantity you already have. A pool is the second.
    pub rage: i32,
    pub faith: i32,
    pub nature: i32,
    pub insight: i32,
    pub dread: i32,
    /// Mind damage the **character** adds to every mind hit it lands.
    ///
    /// **Through `Held` and not through `Stats`, which is the whole finding.**
    /// `Effect::Stat { mind }` lands in the character's own `Stats`, and
    /// `ItemProfile::stats.mind` is what the fight reads — so a tree that
    /// granted mind damage granted nothing at all, which is the *eight skill
    /// nodes* failure exactly and was caught by
    /// `every_point_in_an_expert_tree_buys_something` before it shipped.
    ///
    /// It cannot be read off `player_stats` either: that sums every item's
    /// stats, so adding it to every item's hit would pay the board's own mind
    /// damage once per item. `Held` is the one door for *what the character
    /// contributes, once, beside the item* — the same door the tree's armour,
    /// its mana and its granted rules go through.
    pub mind: i32,
    /// Rates the character has **for this fight only**.
    ///
    /// **The one field in `Held` that is a rate rather than a quantity**, and
    /// it is here rather than in the character's own `Stats` because that is
    /// what *for this fight only* means: a potion that added to `player_stats`
    /// would be a permanent stat with a click in front of it, and there would
    /// be no moment at which it wore off. Added to the player's stats at the
    /// bell and nowhere else, so combat stays a pure function of what it was
    /// handed.
    ///
    /// Nothing but a brew fills it today. It is not named `brew` for the same
    /// reason `armor` is not named `tree`: the next thing that wants a rate for
    /// one fight should use this rather than inventing a second one.
    pub stats: crate::stats::Stats,
}

/// Put an expert's power onto the fighter at the bell.
///
/// **Six of the ten are the fight's and four are not**, and every one of them
/// says which here rather than being left out — the same posture every arm of
/// the rule loop above takes, and the reason adding an expert is a decision
/// about combat rather than a silence.
///
/// The six that are combat's all land in the one `expert` field: what each
/// does with its knobs is read where the mechanic is, because a knob copied
/// into a second field here would be a second answer to what the tree tuned.
/// Every furnace expert has a furnace, whether or not a Stoker lit one.
///
/// **An expert's power is self-contained**, which is the rule M13 set and this
/// is the first block where it has cost anything to keep. Six of the
/// twenty-one are about what a furnace does, and a furnace is the Stoker's —
/// so a Fired Funnel whose promise is *every stack the furnace buys is also
/// mana* would promise nothing if it had to wait for somebody else's power to
/// be in the list beside it.
///
/// It is belt-and-braces rather than a behaviour change: you cannot hold one of
/// these without being a Stoker. What it buys is that the promise is true of
/// the power rather than of the pair, which is what lets
/// `every_point_in_an_expert_tree_buys_something` ask its question of the
/// expert alone — and asking it of the pair buries five of the original ten
/// under two parent powers' worth of noise.
fn light_the_furnace(c: &mut Combatant) {
    if c.burn_every_ms == 0 {
        c.burn_every_ms = 4_000;
    }
    c.burn_per_stack = c.burn_per_stack.max(10);
}

fn apply_expert(c: &mut Combatant, e: crate::expert::ExpertPower) {
    use crate::expert::ExpertPower::*;
    match e {
        // The fight's. Read at the tick, off `Combatant::expert`.
        LoudCalculation { .. }
        | StandingFact { .. }
        | OverwoundArm { .. }
        | CurseRequisition { .. }
        | PatentedFunnel { .. }
        | OpeningNumber { .. }
        | CursedLicence { .. }
        // **Nine of M16's eleven are the fight's too**, and each is read where
        // its own mechanic is rather than copied into a field here — which is
        // the rule this function's own doc states and the reason there is one
        // `expert` field rather than thirty.
        | BareFurnace { .. }
        | FiredFunnel { .. }
        | ColdStoke { .. }
        | PonkeyBoiler { .. }
        | FlashPowder { .. }
        | LoudDoubt { .. }
        | RequisitionedSilence { .. }
        | LicensedRumour { .. }
        | AshAndWhisper { .. } => {
            c.expert = Some(e);
            // **The budgets a fight starts with, seeded once at the bell.**
            // Read off the tuned power rather than kept beside it, so a knob
            // the tree moved is the number the fight actually runs on — which
            // is the same reason the promise is printed from the tuned value.
            match e {
                LoudCalculation { floor, .. } => c.loud_free_left = floor.max(0),
                CurseRequisition { per_fight, .. } => c.reqs_left = per_fight.max(0),
                OpeningNumber { window_ms, encore, .. } => {
                    c.opening_until_ms = window_ms.max(0) as u32;
                    c.opening_encores = encore.max(0);
                }
                // **A bare frame is a hopper that never empties**, and it is
                // paid once at the bell rather than at the tick: an empty frame
                // does not fill up and a stack bought from one is a stack you
                // walked in with.
                // **Ash and Whisper's furnace runs on what it burns**, so it
                // needs a shovel even on a board with no Stoker in it.
                BareFurnace { per_frame, every_ms, cap } => {
                    let got = per_frame.max(0) * c.empty_frames as i32 / 10;
                    c.empowerment += got.clamp(0, cap.max(0)) as u32;
                    light_the_furnace(c);
                    c.burn_every_ms = every_ms.max(TICK_MS as i32) as u32;
                }
                ColdStoke { every_ms, .. } | AshAndWhisper { every_ms, .. } => {
                    light_the_furnace(c);
                    c.burn_every_ms = every_ms.max(TICK_MS as i32) as u32;
                }
                // The Funnel's exchange window, spent like a requisition.
                FiredFunnel { per_fight, .. } => {
                    c.fired_left = per_fight.max(0);
                    light_the_furnace(c);
                }
                // **The boiler's two other knobs land on things the fight
                // already reads**: a turning item keeps more of its turns, and
                // a component holds more enchs — which is the board's and is
                // answered in `Character::ench_racks`, one of the thirty-one
                // places that walk a board.
                PonkeyBoiler { keep, .. } => {
                    c.spin_keep += keep.max(0) as u32;
                    light_the_furnace(c);
                }
                RequisitionedSilence { per_fight, .. } => c.silent_left = per_fight.max(0),
                // **The flash is before the first tick**, which is what makes
                // it the Showstopper's half of the pairing: everything you were
                // holding, at once, and the fight is decided early or not at
                // all.
                FlashPowder { all_at_once, .. } => {
                    light_the_furnace(c);
                    let pct = all_at_once.clamp(0, 100);
                    for (i, what) in [
                        crate::piece::Resource::Rage,
                        crate::piece::Resource::Faith,
                        crate::piece::Resource::Nature,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let have = c.pool(what);
                        let took = have * pct / 100;
                        if took > 0 {
                            c.set_pool(what, have - took);
                            c.burned[i] += took;
                            c.empowerment += (took / 10).max(1) as u32;
                        }
                    }
                }
                _ => {}
            }
        }
        // **The purse's**, settled by `reward::bounty_with_class` where the
        // argument about what a fight pays already lives. `Showstopper` is the
        // precedent and the reason that file exists.
        ShortProgramme { .. } | EleventhSeason { .. } | CurtainLine { .. } => {}
        // **The board's.** How many enchs fit on a component, and what an
        // enched component lends its neighbours, are packing-screen facts —
        // already in the profiles this fight was handed, the same way
        // `Recycler`'s assembly bonus is.
        FullBill { .. } | ToldOnce { .. } => {}
    }
}

pub fn simulate(player_stats: Stats, profiles: &[ItemProfile], spec: &MonsterSpec) -> CombatLog {
    simulate_at(player_stats, profiles, spec, Difficulty::Easy)
}

pub fn simulate_at(
    player_stats: Stats,
    profiles: &[ItemProfile],
    spec: &MonsterSpec,
    difficulty: Difficulty,
) -> CombatLog {
    simulate_with_class(player_stats, profiles, spec, difficulty, &[])
}

/// The same, with the player's class applied. `Standing` powers are already
/// folded into `player_stats` by the run; the rest are rules the fight has to
/// know about.
pub fn simulate_with_class(
    player_stats: Stats,
    profiles: &[ItemProfile],
    spec: &MonsterSpec,
    difficulty: Difficulty,
    classes: &[crate::class::ClassDef],
) -> CombatLog {
    simulate_with_purse(player_stats, profiles, spec, difficulty, classes, 0)
}

/// The same, with a purse for `SpendGold` to reach into.
///
/// Split from `simulate_with_class` rather than added to it because only the
/// run has a purse: every test and every analysis tool fights without one, and
/// none of them should have to say so.
pub fn simulate_with_purse(
    player_stats: Stats,
    profiles: &[ItemProfile],
    spec: &MonsterSpec,
    difficulty: Difficulty,
    classes: &[crate::class::ClassDef],
    purse: i32,
) -> CombatLog {
    simulate_holding(player_stats, profiles, spec, difficulty, classes, purse, Held::default())
}

/// The same, starting with armour or mana already banked.
///
/// The last rung, and the one the run uses. Split off rather than added to
/// `simulate_with_purse` for the reason every rung here is split off: only the
/// run has a skill tree, and no test or analysis tool should have to say it
/// holds nothing.
pub fn simulate_holding(
    player_stats: Stats,
    profiles: &[ItemProfile],
    spec: &MonsterSpec,
    difficulty: Difficulty,
    classes: &[crate::class::ClassDef],
    purse: i32,
    held: Held,
) -> CombatLog {
    simulate_party_holding(
        player_stats,
        profiles,
        std::slice::from_ref(spec),
        difficulty,
        classes,
        purse,
        held,
    )
}

/// Fight everything in `specs` at once.
///
/// The player's single-target attacks land on whoever is at the front - the
/// first of them still standing - and every one of them acts against you
/// independently. It is over when all of them are down, or you are.
pub fn simulate_party(
    player_stats: Stats,
    profiles: &[ItemProfile],
    specs: &[MonsterSpec],
    difficulty: Difficulty,
    classes: &[crate::class::ClassDef],
    purse: i32,
) -> CombatLog {
    simulate_party_holding(player_stats, profiles, specs, difficulty, classes, purse, Held::default())
}

/// The same, starting with armour or mana already banked.
pub fn simulate_party_holding(
    player_stats: Stats,
    profiles: &[ItemProfile],
    specs: &[MonsterSpec],
    difficulty: Difficulty,
    classes: &[crate::class::ClassDef],
    purse: i32,
    held: Held,
) -> CombatLog {
    assert!(!specs.is_empty(), "a fight needs something to fight");
    // **Rates for this fight only, folded in before the fighter is built.**
    // A brew is the only thing that fills this today, and it goes on here
    // rather than on the character so that there is a moment at which it wears
    // off — which is the whole of what *temporary* means.
    let mut player_stats = player_stats;
    player_stats += held.stats;
    let mut start_player = Combatant::player(player_stats, profiles);
    start_player.purse = purse;
    // Before the class powers, so `Tired`'s debt and `Unionized`'s plate still
    // add to and subtract from what the tree granted rather than replacing it.
    start_player.armor += held.armor;
    start_player.mana += held.mana;
    start_player.empowerment += held.empowerment;
    start_player.said += held.mind;
    start_player.rage += held.rage;
    start_player.faith += held.faith;
    start_player.nature += held.nature;
    start_player.insight += held.insight;
    start_player.dread += held.dread.max(0) as u32;
    // A board fact, carried in rather than derived, because an empty frame
    // produces no `ItemProfile` and so is invisible from in here.
    start_player.empty_frames = held.empty_frames;
    // **What followed you here.** Landed before the first tick, on whatever is
    // in front of you, which is what *before it acts* means when a fight is
    // fifty-millisecond slices: there is no earlier moment than this one.
    let told = held.told.clone();
    // **Rules the tree granted.** Translated here rather than read as `Rule`s
    // in the tick, so combat goes on speaking its own vocabulary and a new
    // rule is one arm in one place. Exhaustive, so a rule nobody wires up is a
    // compile error rather than a node that costs a point and does nothing.
    for r in &held.rules {
        match r {
            crate::skills::Rule::CurseOnActivate { slot, curse } => {
                if let (Some(s), Some(k)) = (
                    crate::skills::slot_of(slot),
                    crate::curse::CurseKind::by_name(curse),
                ) {
                    start_player.curse_on_activate.push((s, k));
                }
            }
            // Not a combat rule at all: it decides what the map screen is
            // allowed to print. Same shape as `Prospector` and `Showstopper`,
            // which are settlement rules and are ignored here too.
            // The three that tune the spin. They add and take the fastest,
            // because two nodes granting the same rule is a tree that stacks
            // rather than a tree with a last-one-wins in it.
            // **All three of M16's, and each adds rather than replaces**, the
            // way the spin's three do: two nodes granting the same rule is a
            // tree that stacks, not a tree with a last-one-wins in it. Clamped
            // where they are read rather than here.
            crate::skills::Rule::BurnKeepsBonus { pct } => {
                start_player.burn_keeps_pct += *pct as i32
            }
            // **Not combat's.** What survives a bell is settled after one, the
            // same way `StandingFact::told` is — see `Character::carry_out_of`.
            // The arm exists so that adding a rule is a decision rather than a
            // silence.
            crate::skills::Rule::BurnCarries { .. } => {}
            crate::skills::Rule::MindPierce { pct } => start_player.mind_pierce += *pct as i32,
            crate::skills::Rule::SpinExtra { per_turn } => start_player.spin_extra += per_turn,
            crate::skills::Rule::SpinKeep { stacks } => start_player.spin_keep += stacks,
            crate::skills::Rule::SpinEvery { ms } => {
                start_player.spin_every_ms = start_player.spin_every_ms.min(*ms).max(TICK_MS)
            }
            crate::skills::Rule::Scout => {}
            // **Neither of M9's two is a combat rule**, and that is the whole
            // reason `Rule` moved out of the tree rather than growing a
            // combat-only sibling. A rout is settled where the encounter is,
            // before there is a fight to put it in; a wade is answered by
            // `world::step`, where a wall is refused. Ignored here for the same
            // reason `Scout` is — this arm exists so that adding a rule is a
            // decision about combat rather than a silence.
            // And M11.5's is a *map* rule, which is a third kind again: it
            // decides how a surveyable map reads, and a fight has never known
            // which map it is on.
            crate::skills::Rule::Rout { .. }
            | crate::skills::Rule::Wade
            | crate::skills::Rule::Survey { .. }
            // And a travel rule, which is a fourth kind again: it is a gesture
            // the player makes on the map screen and a fight has never had one.
            | crate::skills::Rule::Homeward
            // **Two board rules, and a fifth kind.** A filled row is paid into
            // `Held::mana` before this runs, and an ench lent to a neighbour is
            // already in the profiles this fight was handed — both are answered
            // where the board is, which is the same division `Recycler`'s
            // assembly bonus has always made. `Spread` is the board's too and
            // is settled when the fight ends, because a fight that wrote to the
            // character is a fight a mid-fight save could not carry.
            | crate::skills::Rule::RowHarvest { .. }
            | crate::skills::Rule::Beacon { .. }
            | crate::skills::Rule::Spread { .. } => {}
            // The fight's, and the first new thing in a fight since the Chonga
            // Swing. Held on the fighter rather than folded into a profile: a
            // profile is the *board's* answer and this is the *character's*, so
            // two players with the identical board do not have the identical
            // fight — the same reason `CurseOnActivate` fires beside an item's
            // own triggers rather than inside them.
            crate::skills::Rule::Productivity { every, slower_pct } => {
                start_player.productivity = Some((*every, *slower_pct));
            }
        }
    }
    // Every class you hold applies at once. The fountains hand out different
    // classes, never the same one twice, so two powers never fight over the
    // same field.
    for c in classes {
        match c.power {
            // **The one class in the game the fight never reads.** An
            // apothecary's power is honoured where a brew is made and where a
            // brew is worth something — `Game::retort` and `Character::boon` —
            // and the potion it produces arrives here through `Held` like every
            // other thing you walk in already holding. The arm exists so that
            // adding a specialization is a decision about combat rather than a
            // silence.
            crate::class::ClassPower::Apothecary { .. } => {}
            crate::class::ClassPower::SlowTime(n) => start_player.slow_time = n,
            crate::class::ClassPower::Overflowing(n) => start_player.overflowing = n,
            crate::class::ClassPower::Leeching(pct) => start_player.leech = pct,
            // **The furnace is lit at the bell**, like everything else here.
            // `burn_ms` counts up on the fighter's own clock beside the spin's,
            // for the same reason the spin does: there is no item, so there is
            // no item's clock.
            crate::class::ClassPower::Stoker { every_ms, per_stack } => {
                start_player.burn_every_ms = every_ms.max(TICK_MS);
                start_player.burn_per_stack = per_stack.max(0);
            }
            // **The threshold is the *foe's* and is set from the power the
            // player holds**, which is why it is done below rather than here:
            // `start_player` is the wrong combatant for it.
            crate::class::ClassPower::Whisperer { .. } => {}
            crate::class::ClassPower::WrongSense(pct) => start_player.mind_pierce = pct,
            crate::class::ClassPower::FirstBlood => start_player.first_blood = true,
            // Not a combat rule at all: it changes what a corpse leaves
            // behind, which is `Run::settle`'s business.
            crate::class::ClassPower::Prospector(_) => {}
            // Armour before the first blow, and it stacks - so this one adds
            // where nearly every other arm here assigns.
            crate::class::ClassPower::Unionized { armor } => {
                start_player.armor += armor;
            }
            // Not a combat rule either: it changes what a win is worth, which
            // is `Run::settle`'s business.
            crate::class::ClassPower::Showstopper { .. } => {}
            crate::class::ClassPower::Standing(_) => {}
            crate::class::ClassPower::Echo(n) => start_player.echo_every = n,
            crate::class::ClassPower::Bastion(pct) => start_player.bastion = pct,
            crate::class::ClassPower::Contagion(n) => start_player.contagion = n,
            crate::class::ClassPower::Guilt => start_player.no_regen = true,
            // The two that stack. Every other arm here assigns, because the
            // fountains never hand out the same class twice; a town hands out
            // the same one over and over on purpose, so these add.
            // Recycler is a board rule, not a fight rule: it scales assembly
            // bonuses, which are already in the stats and the item profiles
            // this fight was handed. See `Loadout::assembly_pct`.
            crate::class::ClassPower::Recycler { .. } => {}
            // **The ten experts.** Six of them are the fight's and land in
            // `Combatant` fields below; four are the purse's or the board's
            // and say so here, so that adding an expert is a decision about
            // combat rather than a silence — the same posture every arm above
            // takes. M13.6 is where each of them was wired, and
            // `every_offered_class_reaches_something` over all fifteen is what
            // proves none of them is a promise reaching nothing.
            crate::class::ClassPower::Expert(e) => apply_expert(&mut start_player, e),
            crate::class::ClassPower::Piety { faith } => start_player.faith += faith,
            crate::class::ClassPower::Tired { mana } => start_player.mana -= mana,
            crate::class::ClassPower::Ticket { nth } => start_player.warded_every = nth,
            crate::class::ClassPower::Trundle { slower, armour } => {
                start_player.slower_pct = slower;
                start_player.armour_pct = armour;
            }
            crate::class::ClassPower::Longhaul { per_second } => {
                start_player.haste_per_s = per_second;
            }
            crate::class::ClassPower::Reprisal(n) => start_player.reprisal = n,
            crate::class::ClassPower::Riposte(ms) => start_player.riposte = ms,
            crate::class::ClassPower::Momentum(n) => start_player.momentum = n,
            crate::class::ClassPower::Resonance(n) => start_player.resonance = n,
            crate::class::ClassPower::Transmute(pct) => start_player.transmute = pct,
            crate::class::ClassPower::Adaptable(n) => start_player.adaptable = n,
            crate::class::ClassPower::Untimely(n) => start_player.untimely = n,
            crate::class::ClassPower::Cascade(ms) => start_player.cascade = ms,
            crate::class::ClassPower::Consecrate(pct) => start_player.consecrate = pct,
            crate::class::ClassPower::Bloodscent(n) => start_player.bloodscent = n,
            crate::class::ClassPower::Confluence(pct) => start_player.confluence = pct,
            // Split the wisdom: every item takes a share of the best
            // multiplier on the board on top of its own. Done here rather than
            // in the profile because it is a property of the whole board, and
            // the profile only knows about one item.
            crate::class::ClassPower::Avenged(n) => start_player.rage += n,
            crate::class::ClassPower::Splintered(pct) => {
                let best = start_player.items.iter().map(|i| i.power).max().unwrap_or(100);
                let share = (best - 100).max(0) * pct / 100;
                for it in &mut start_player.items {
                    it.power += share;
                }
            }
        }
    }
    let start_player = start_player;
    let mut start_enemies: Vec<Combatant> =
        specs.iter().map(|m| Combatant::monster_at(m, difficulty)).collect();
    // **The Whisperer's threshold is the foe's, set from the power the player
    // holds**, which is why it is not in the loop above: `start_player` is the
    // wrong combatant for it. Set once, off the maximum as the bell went, so
    // eating a maximum down does not move the line it is being measured
    // against — a threshold that chased the number it reads would never be
    // crossed.
    //
    // **The tuning is read off `class_defs`**, which is the classes you *are*
    // with the expert's knobs turned, not `CLASSES` — the thirty-eight dead
    // nodes of M13.6 are what happens when a promise is printed from the
    // roster.
    // **Five of the twenty-one experts move the line as well**, and they are
    // read here rather than each finding its own way to the foe for the reason
    // the base class is: the threshold is set once, off the maximum as the bell
    // went, and a line that chased the number it reads would never be crossed.
    //
    // The **largest** of them, not the sum. Two experts that each say *forty
    // percent* say the same thing, and a character holding both would otherwise
    // unmake at eighty — which is a threshold that has stopped describing
    // anything.
    let third = classes
        .iter()
        .filter_map(|c| match c.power {
            crate::class::ClassPower::Whisperer { third } => Some(third),
            crate::class::ClassPower::Expert(e) => match e {
                crate::expert::ExpertPower::LoudDoubt { third, .. }
                | crate::expert::ExpertPower::RequisitionedSilence { third, .. }
                | crate::expert::ExpertPower::LicensedRumour { third, .. }
                | crate::expert::ExpertPower::CurtainLine { third, .. }
                | crate::expert::ExpertPower::AshAndWhisper { third, .. } => Some(third),
                _ => None,
            },
            _ => None,
        })
        .max();
    if let Some(third) = third {
        for f in start_enemies.iter_mut() {
            f.unmade_at = Some(f.max_health * third.clamp(1, 99) / 100);
        }
    }
    // **Told Once's two knobs, copied onto whoever is going to suffer them.**
    // `take_mind_pierced` has one combatant and no view of the room, so the
    // only way a fighter can know what has been said about it is to be told at
    // the bell — the same shape `unmade_at` above it takes, and for the same
    // reason.
    if let Some((per, cap)) = classes.iter().find_map(|c| match c.power {
        crate::class::ClassPower::Expert(crate::expert::ExpertPower::ToldOnce {
            per_curse,
            cap,
            standing,
        }) => Some((per_curse.max(0) * (1 + standing.max(0)), cap.max(0))),
        _ => None,
    }) {
        for f in start_enemies.iter_mut() {
            f.told_per_curse = per;
            f.told_cap = cap;
            // **A threshold that is not there is not one anything can be
            // pushed over**, so this seeds the Whisperer's own floor when
            // nothing else has — the same self-containment `light_the_furnace`
            // gives the six furnace experts, and for the same reason: you
            // cannot hold Told Once without being a Whisperer, so a promise
            // that waited for another power to be in the list beside it would
            // be a promise about the pair rather than about the power.
            let floor = f.max_health * 33 / 100;
            f.unmade_at.get_or_insert(floor);
        }
    }
    let start_enemies = start_enemies;
    let mut p = start_player.clone();
    let mut foes: Vec<Combatant> = start_enemies.clone();
    let mut log: Vec<LogEntry> = Vec::new();
    // Reported once each, as they go down.
    let mut fallen: Vec<usize> = Vec::new();
    // How many quarters each foe has already been reported as having lost.
    // See `the_fight_turned`, which is the only milestone a fight in this game
    // has that is not its own last tick.
    let mut turned: Vec<u32> = vec![0; foes.len()];

    // **What followed you out of the last fight lands before this one starts.**
    // At `t = 0`, on the first thing in front of you, and logged like any other
    // curse so the replay draws it — a chip that appeared with no entry behind
    // it would be the page inventing a number, which is the one thing it must
    // never do. Landed on `foes` and not on `start_enemies`, because the
    // starting fighters are the ones a replay is rebuilt from.
    for kind in &told {
        if let Some(f) = foes.first_mut() {
            land_curse(f, Ref::foe(0), *kind, StunAim::Unaimed, 0, &mut log);
        }
    }

    // Everyone in the fight, player first, so the loops below read the same
    // whether there is one thing across the table or three.
    let everyone = |foes: &[Combatant]| -> Vec<Ref> {
        std::iter::once(Ref::PLAYER).chain((0..foes.len()).map(Ref::foe)).collect()
    };

    // What each side walks in already holding. Everything else starts a fight
    // at zero and earns its way up, which makes the opening of every fight
    // look the same whatever you are wearing; this is the gear that does not.
    for me in everyone(&foes) {
        let opening: Vec<(usize, Action)> = pick(&mut p, &mut foes, me)
            .items
            .iter()
            .enumerate()
            .flat_map(|(i, it)| {
                let open = it.open_cells;
                it.triggers
                    .iter()
                    .flat_map(move |t| match t {
                        Trigger::OnBattleStart(a) => vec![(i, *a)],
                        // `PerAdjacentEmpty` wraps a trigger, and until now it
                        // was only ever unwrapped on the *activation* path - so
                        // "for each empty cell, at the bell" matched nothing
                        // here and did nothing at all. It composes with the
                        // spending triggers by design; it has to compose with
                        // this one too.
                        Trigger::PerAdjacentEmpty(inner) => match **inner {
                            Trigger::OnBattleStart(a) => vec![(i, a); open],
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        for (idx, action) in opening {
            apply(&mut p, &mut foes, me, action, 0, &mut log, Some(idx));
        }
    }
    let mut outcome = Outcome::Stalemate;
    let mut t: u32 = 0;

    'fight: while t < MAX_DURATION_MS {
        t += TICK_MS;

        // 0. Slow time: whatever was queued arrives a slice at a time.
        for c in std::iter::once(&mut p).chain(foes.iter_mut()) {
            if c.pending.is_empty() {
                continue;
            }
            let mut still = Vec::new();
            let mut arriving = 0;
            for (amount, left) in std::mem::take(&mut c.pending) {
                let slice = (amount * TICK_MS as i32 / SLOW_TIME_MS as i32).max(1).min(amount);
                arriving += slice;
                let rest = amount - slice;
                let left = left.saturating_sub(TICK_MS);
                if rest > 0 && left > 0 {
                    still.push((rest, left));
                } else if rest > 0 {
                    arriving += rest;
                }
            }
            c.pending = still;
            if arriving > 0 {
                let absorbed = arriving.min(c.armor.max(0));
                c.armor -= absorbed;
                c.health -= arriving - absorbed;
            }
        }

        // 1. Damage over time, then healing.
        for me in everyone(&foes) {
            let side = me.side;
            let who = me.who as u8;
            let c = pick(&mut p, &mut foes, me);
            c.dot_milli += c.curses.dot_millidamage_per_tick();
            let whole = c.dot_milli / 1000;
            if whole > 0 {
                c.dot_milli %= 1000;
                c.health -= whole;
                c.burn_acc += whole;
            }
            // Report burn once a second, or immediately if it just killed
            // them, rather than a line per tick.
            c.burn_timer += TICK_MS;
            if c.burn_acc > 0 && (c.burn_timer >= BURN_REPORT_MS || c.health <= 0) {
                let (dmg, hp) = (c.burn_acc, c.health);
                c.burn_acc = 0;
                c.burn_timer = 0;
                log.push(LogEntry { who, at_ms: t, event: Event::Burn { side, damage: dmg, health: hp } });
            }
            let regen = c.effective_regen();
            if regen > 0 && c.health < c.max_health && !c.no_regen {
                c.regen_milli += regen * TICK_MS as i32;
                let heal = (c.regen_milli / 1000).min(c.max_health - c.health);
                if heal > 0 {
                    c.regen_milli %= 1000;
                    c.health += heal;
                    let hp = c.health;
                    log.push(LogEntry {
                        who,
                        at_ms: t,
                        event: Event::Regen { side, amount: heal, health: hp },
                    });
                }
            }
        }
        // 1b. Sudden death. A fight that has gone on this long ends itself,
        // and it ends itself for everybody at once - straight off health,
        // past armour and resistance, because a wall you can hide behind for
        // ever is exactly what this exists to stop.
        if t >= SUDDEN_DEATH_MS && t % 1000 == 0 {
            let second = ((t - SUDDEN_DEATH_MS) / 1000 + 1) as i32;
            log.push(LogEntry { who: 0, at_ms: t, event: Event::SuddenDeath { pct: second } });
            for c in std::iter::once(&mut p).chain(foes.iter_mut()) {
                let bite = (c.max_health * second / 100).max(1);
                c.health -= bite;
            }
        }

        // **The two experts that pay when the fight turns.** Beside the check
        // that reports a corpse rather than inside it, because `check_down`
        // takes the fighters by reference on purpose — it reports and decides
        // and changes nothing — and a settlement that wrote through it would
        // be the one exception somebody has to remember.
        let turn = the_fight_turned(&foes, &mut turned);
        rebate_when_it_turns(&mut p, turn);
        encore_when_it_turns(&mut p, t, turn);
        if check_down(&p, &foes, t, &mut log, &mut outcome, &mut fallen) {
            break 'fight;
        }

        // 2. Curse timers.
        p.curses.tick();
        for f in foes.iter_mut() {
            f.curses.tick();
        }

        // 3. Cooldowns and activations.
        //
        // A foe that is already down does not get a turn, but the loop still
        // walks past it: the living ones keep their own item order, which is
        // what makes a fight replay identically.
        for me in everyone(&foes) {
            let side = me.side;
            if pick(&mut p, &mut foes, me).is_down() {
                continue;
            }
            // **The empty frames turn too, once a tick.** Overwound Arm is
            // the one thing in the game that gets anything out of a frame with
            // nothing in it, and it rides the *fighter's* clock rather than an
            // item's for the obvious reason: there is no item. Here rather
            // than inside the item loop below, because a fighter with four
            // items must not wind its bare frames four times.
            {
                let c = pick(&mut p, &mut foes, me);
                let every = c.spin_every_ms.max(TICK_MS);
                overwind(c, every);
                stoke(c, me.who as u8, side, t, &mut log);
            }
            let count = pick(&mut p, &mut foes, me).items.len();
            for idx in 0..count {
                let (ready, turned, banked) = {
                    let c = pick(&mut p, &mut foes, me);
                    // Frost stretches the cooldown by slowing how fast the
                    // bar fills, rather than by rewriting the cooldown. It is
                    // a property of the fighter, so it is read before the item.
                    let slow = c.curses.slow_pct();
                    let slower = c.slower_pct;
                    // The long haul: everything winds up as the fight drags,
                    // to twice speed and no further.
                    let haste = (c.haste_per_s * (t / 1000) as i32).clamp(0, 100);
                    // The spin's tuning is the fighter's, like the slow above
                    // it: a node is taken by a person, not by a blade.
                    let (spin_extra, spin_every) = (c.spin_extra, c.spin_every_ms.max(TICK_MS));
                    let per_stack = match c.expert {
                        Some(crate::expert::ExpertPower::PatentedFunnel { per_stack, .. }) => {
                            per_stack.max(0)
                        }
                        _ => 0,
                    };
                    let item = &mut c.items[idx];
                    // **Broken is finished.** Beside the stun rather than
                    // folded into it: a stun is a curse somebody put on you and
                    // ends, and this is the gear. Nothing else about the item
                    // moves either — a broken bar does not turn, for the same
                    // reason a stopped one does not.
                    if item.broken {
                        (false, None, 0)
                    } else if item.stun_ms > 0 {
                        item.stun_ms = item.stun_ms.saturating_sub(TICK_MS);
                        // A stopped bar does not turn either. The spin rides
                        // the item's own clock, and a stunned item's clock is
                        // exactly what a stun stops.
                        (false, None, 0)
                    } else {
                        let step = (TICK_MS as i32 * (100 - slow) / 100 * (100 - slower) / 100
                            * (100 + haste)
                            / 100)
                            .max(1) as u32;
                        // A debt is paid out of the bar before the bar moves.
                        // Frost and stun reach it the way they reach everything
                        // else, which is correct twice over: a slowed item pays
                        // slower because it is slower, and a stopped one does
                        // not pay at all because a stopped bar does not move.
                        let step = if item.owed_ms > 0 {
                            let paid = step.min(item.owed_ms);
                            item.owed_ms -= paid;
                            step - paid
                        } else {
                            step
                        };
                        // **One turn a second, banked.** Off the same slice
                        // the bar moves in, so a frosted item turns slower for
                        // the same reason it fires slower and a stunned one
                        // does not turn at all — the spin is a property of the
                        // item's own clock rather than of the wall.
                        let mut turns = 0u32;
                        let mut banked = 0i32;
                        if item.spins {
                            item.spin_ms += step;
                            while item.spin_ms >= spin_every {
                                item.spin_ms -= spin_every;
                                item.spin_stacks += 1 + spin_extra;
                                item.turn_index = (item.turn_index + 1) % item.turn_cycle_len;
                                turns += 1;
                                // **The Patented Funnel: the spin banks Funny.**
                                // Per stack per turn, so a slow item that has
                                // been turning a while pays more than a fast
                                // one that just started — which is what makes
                                // it worth leaving room to turn.
                                banked += per_stack * item.spin_stacks as i32;
                            }
                        }
                        let turned = (turns > 0).then(|| {
                            (item.name.clone(), item.spin_stacks, item.turn_index)
                        });
                        item.progress_ms += step;
                        let ready = if item.progress_ms >= item.cooldown_ms {
                            item.progress_ms -= item.cooldown_ms;
                            true
                        } else {
                            false
                        };
                        (ready, turned, banked)
                    }
                };
                // **What the funnel banked, paid once the item's borrow is
                // done.** Into `mana`, which is the pool the theme calls the
                // Funny — the same pool a cast spends, which is what makes a
                // spinning board and a casting board the same build.
                if banked > 0 {
                    let me_c = pick(&mut p, &mut foes, me);
                    me_c.mana += banked;
                    // Funny past what a funnel will hold spills as armour,
                    // 1 for 1 — the only sink for a class that will otherwise
                    // bank more than it can ever spend.
                    if let Some(crate::expert::ExpertPower::PatentedFunnel { overflow, .. }) =
                        me_c.expert
                    {
                        if overflow > 0 && me_c.mana > FUNNEL_HOLD {
                            let spill = me_c.mana - FUNNEL_HOLD;
                            me_c.mana = FUNNEL_HOLD;
                            me_c.armor += spill;
                        }
                    }
                }
                // Logged outside the borrow above, in the slice it happened
                // in. Every turn gets an entry, so the replay reads the
                // orientation rather than dividing the playback head by a
                // second — which would draw a shape the fight never had the
                // moment anything slowed the item down.
                if let Some((name, stacks, to)) = turned {
                    // **Ponkey Boiler: the spin feeds the furnace.** Every turn
                    // a spinning item banks is tenths of a stack, booked here
                    // because this is where a turn is banked — and there is no
                    // second place a turn happens.
                    {
                        let c = pick(&mut p, &mut foes, me);
                        if let Some(crate::expert::ExpertPower::PonkeyBoiler {
                            per_spin, ..
                        }) = c.expert
                        {
                            c.boiler_tenths += per_spin.max(0);
                            let whole = c.boiler_tenths / 10;
                            if whole > 0 {
                                c.boiler_tenths -= whole * 10;
                                c.empowerment += whole as u32;
                            }
                        }
                    }
                    let front = aim_of(&foes, p.aim);
                    log.push(LogEntry {
                        who: me.logged_as(front),
                        at_ms: t,
                        event: Event::Turned { side, index: idx, item: name, to, stacks },
                    });
                }
                if ready {
                    // A misfire eats the activation itself: the cooldown has
                    // already come round, and nothing comes of it.
                    let fizzled = {
                        let c = pick(&mut p, &mut foes, me);
                        c.misfire_count = c.misfire_count.wrapping_add(1);
                        // Counted whatever happens, because the curse is on
                        // the fighter and eats every nth activation *they*
                        // have. A steady item does not stop the count, it
                        // simply is not the one that goes quiet - so building
                        // one buys reliability for that item and hands the
                        // fizzle to the next one round.
                        //
                        // And the hunter's first blow cannot miss, which is
                        // the other half of "cannot miss and cannot be
                        // deflected" - a fizzle is the only thing in this game
                        // that eats a swing of yours outright.
                        c.curses.misfires(c.misfire_count)
                            && !c.items[idx].steady
                            && !c.first_blood
                    };
                    if fizzled {
                        let name = pick(&mut p, &mut foes, me).items[idx].name.clone();
                        let front = aim_of(&foes, p.aim);
                        log.push(LogEntry {
                            who: me.logged_as(front),
                            at_ms: t,
                            event: Event::Misfired { side, item: name },
                        });
                        continue;
                    }
                    // Ticket to Ride: every nth thing they swing at you comes
                    // to nothing. Eaten here rather than at each damage site
                    // because a warded attack lands nothing at all - no hit,
                    // no curse, no drain - and there are a dozen ways for one
                    // activation to reach you.
                    if side == Side::Enemy && p.warded_every > 0 {
                        let c = pick(&mut p, &mut foes, me);
                        c.warded_count = c.warded_count.wrapping_add(1);
                        let warded = c.warded_count % p.warded_every == 0;
                        if warded {
                            let name = pick(&mut p, &mut foes, me).items[idx].name.clone();
                            let front = aim_of(&foes, p.aim);
                            log.push(LogEntry {
                                who: me.logged_as(front),
                                at_ms: t,
                                event: Event::Warded { side, item: name },
                            });
                            continue;
                        }
                    }
                    let again = activate(&mut p, &mut foes, me, idx, t, &mut log);
                    if check_down(&p, &foes, t, &mut log, &mut outcome, &mut fallen) {
                        break 'fight;
                    }
                    if again {
                        activate(&mut p, &mut foes, me, idx, t, &mut log);
                        if check_down(&p, &foes, t, &mut log, &mut outcome, &mut fallen) {
                            break 'fight;
                        }
                    }
                }
            }
        }
    }

    log.push(LogEntry { who: 0, at_ms: t, event: Event::End { outcome } });
    // What the purse lost over the fight, for the run to charge afterwards.
    let spent_from_purse = purse - p.purse;
    // **Counted off the live foes, not off the entries.** `foes` is what the
    // fight actually ended holding; walking the log for it would be re-deriving
    // a fact three lines from the thing that knows it.
    let standing: u32 = foes.iter().flat_map(|f| f.curses.iter()).map(|c| c.stacks).sum();
    let mut kinds: Vec<crate::curse::CurseKind> = Vec::new();
    for f in &foes {
        for c in f.curses.iter() {
            if c.stacks > 0 && !kinds.contains(&c.kind) {
                kinds.push(c.kind);
            }
        }
    }
    let landed: u32 = log
        .iter()
        .filter(|e| matches!(e.event, Event::Cursed { on, .. } if on != Side::Player))
        .count() as u32;
    CombatLog {
        player: start_player,
        enemies: start_enemies,
        specs: specs.to_vec(),
        curse_bill: CurseBill { standing, kinds, expired: landed.saturating_sub(standing) },
        entries: log,
        outcome,
        duration_ms: t,
        gold_spent: spent_from_purse,
    }
}

/// One combatant in a fight: the player, or the nth foe.
///
/// `Side` stays two-valued because the *rules* are two-sided - you and them -
/// and only the far side can have more than one body in it. This pairs the
/// side with which body, and is what every helper threads instead of a bare
/// `Side`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Ref {
    side: Side,
    who: usize,
}

impl Ref {
    const PLAYER: Ref = Ref { side: Side::Player, who: 0 };

    fn foe(who: usize) -> Ref {
        Ref { side: Side::Enemy, who }
    }

    /// The far side of this exchange. For a foe that is always the player;
    /// for the player it is whoever is at the front of the queue.
    fn other(self, front: usize) -> Ref {
        match self.side {
            Side::Player => Ref::foe(front),
            Side::Enemy => Ref::PLAYER,
        }
    }

    /// What the log records this entry as being about. The player is always
    /// singular, so an entry is about a foe either way: the one acting, or the
    /// one being acted upon.
    fn logged_as(self, front: usize) -> u8 {
        match self.side {
            Side::Player => front as u8,
            Side::Enemy => self.who as u8,
        }
    }
}

/// Whoever the player's next single-target attack lands on.
///
/// Every attack moves the aim along, so a brawl is whittled down at roughly
/// one rate rather than one at a time. That matters for what a two-creature
/// fight *is*: focusing the front one down would make a brawl a queue, where
/// killing the first thing halves the incoming damage and the second half of
/// the fight is easier than the first. Spreading it means both of them are
/// hitting you until nearly the end, which is what makes two of something
/// worse than one of something twice the size.
///
/// Skips anything already down, and never gets stuck: if they are all down
/// the fight is over before this is asked again.
fn aim_of(foes: &[Combatant], cursor: usize) -> usize {
    let n = foes.len();
    (0..n)
        .map(|k| (cursor + k) % n)
        .find(|&i| !foes[i].is_down())
        .unwrap_or(0)
}

fn pick<'a>(p: &'a mut Combatant, foes: &'a mut [Combatant], r: Ref) -> &'a mut Combatant {
    match r.side {
        Side::Player => p,
        Side::Enemy => &mut foes[r.who.min(foes.len().saturating_sub(1))],
    }
}

fn check_down(
    p: &Combatant,
    foes: &[Combatant],
    t: u32,
    log: &mut Vec<LogEntry>,
    outcome: &mut Outcome,
    fallen: &mut Vec<usize>,
) -> bool {
    // Each foe is reported the once, as it goes down, so a brawl reads like a
    // brawl rather than announcing the same corpse every tick.
    for (i, f) in foes.iter().enumerate() {
        if f.is_down() && !fallen.contains(&i) {
            fallen.push(i);
            // **How it ended, before the fact that it did.** An unmaking is
            // reported here rather than in `take_mind_pierced` because that is
            // where the maximum falls and this is where a fall is *told*, and a
            // second telling site is the thing this file has paid for six
            // times. It is still a `Fell` underneath, because it is still down.
            if let Some(floor) = f.unmade_at {
                if f.max_health <= floor && f.max_health > 0 {
                    log.push(LogEntry {
                        who: i as u8,
                        at_ms: t,
                        event: Event::Unmade {
                            side: Side::Enemy,
                            at: f.max_health,
                            under: floor,
                        },
                    });
                }
            }
            log.push(LogEntry { who: i as u8, at_ms: t, event: Event::Fell { side: Side::Enemy } });
        }
    }
    let cleared = foes.iter().all(|f| f.is_down());
    let fell = p.is_down();
    if fell {
        log.push(LogEntry { who: 0, at_ms: t, event: Event::Fell { side: Side::Player } });
    }

    match (cleared, fell) {
        (false, false) => false,
        (true, false) => {
            *outcome = Outcome::Victory;
            true
        }
        (false, true) => {
            *outcome = Outcome::Defeat;
            true
        }
        // Everyone went down on the same tick, which sudden death makes a
        // real possibility rather than a curiosity. Whoever is less far past
        // zero takes it, and a dead heat goes to the player: the fight was
        // even, and an even fight should not cost a life.
        (true, true) => {
            let best = foes.iter().map(|f| f.health).max().unwrap_or(i32::MIN);
            *outcome = if p.health >= best { Outcome::Victory } else { Outcome::Defeat };
            true
        }
    }
}

/// Resolve one item firing: its flat effects, then its triggers in order.
/// Which item a stun takes out.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum StunAim {
    /// Whichever one it happens to catch. This is what a plain curse of stun
    /// does, and not knowing which is most of what keeps it fair.
    Unaimed,
    /// The best thing they own, by the same effectiveness rating the shop
    /// prices gear with. Costs more, and it should - picking the target is
    /// worth more than the stun.
    Strongest,
}

/// Land a stun on one of `victim`'s items and return which, with how long for.
///
/// The choice is deterministic, because the whole engine is: every test in the
/// suite replays a fight and expects the same answer, and a real roll would
/// end that. It is still unpredictable from the far side of the screen, which
/// is the property that actually matters - the same trade `Misfire` makes by
/// counting activations rather than rolling for them.
///
/// Nothing lands on an item that is already stopped for longer than this stun
/// would stop it, when there is a live one to hit instead: a chain of stuns
/// should spread across the kit, not bury one item.
/// Hand back a share of the strength Loud Calculation borrowed, once a kill.
///
/// Has anything opposite just lost another quarter of itself?
///
/// **The only milestone a fight in this game has that is not its own last
/// tick**, and the reason it had to be invented. Two knobs — Loud Calculation's
/// `rebate` and Opening Number's `encore` — were written against *a kill inside
/// the fight*, which is what `PLAN-M13-2.md` §3.1 D and §3.7 C both say. A
/// brawl has those; **GM2D does not deal one.** `fight::run` builds a single
/// `MonsterSpec` from a single `Encounter`, and `check_down` breaks the loop on
/// the same tick the last foe falls — so strength refunded when a foe goes down
/// is refunded onto the final tick of the fight, and a free-cast window
/// reopened after the only foe is dead is a window nobody casts in. Both knobs
/// parsed, cost two points each, and paid nothing, which is the *eight skill
/// nodes* failure wearing a plan's own words.
///
/// So both are re-aimed at the fight **turning**, and each keeps what it was
/// for: the rebate still rewards spending the whole standing order on a fight
/// you are winning, and the encore still buys *time* rather than money.
///
/// **Quarters, not halves, and that is `encore` deciding it.** An encore is a
/// *count* — the tree sells two of them — and a milestone that can happen once
/// is a count that can only ever be one, which is the same dead knob one step
/// along. Three quarters, a half and a quarter left is three turns a fight can
/// have, and the fourth is the corpse, which is the end and pays nothing.
///
/// `turned` is how many each foe has already been reported for, so a threshold
/// fires on the tick it is crossed and never again — the same guard `fallen`
/// gives `check_down`.
fn the_fight_turned(foes: &[Combatant], turned: &mut [u32]) -> bool {
    let mut any = false;
    for (i, f) in foes.iter().enumerate() {
        if f.max_health <= 0 {
            continue;
        }
        let lost = (f.max_health - f.health).max(0);
        // Capped at three: a foe that has lost the fourth quarter is a foe that
        // is down, and the fight ends on that tick.
        let quarters = ((lost as i64 * 4) / f.max_health as i64).min(3) as u32;
        while turned[i] < quarters {
            turned[i] += 1;
            any = true;
        }
    }
    any
}

/// **Every turn refunds a share of the bill**, which rewards spending the whole
/// standing order on a fight you are winning — the Gorillathon half of the pair
/// talking. Paid while there is still a fight to spend it in; see
/// [`the_fight_turned`] for why it is not paid on a corpse.
fn rebate_when_it_turns(p: &mut Combatant, turned: bool) {
    use crate::expert::ExpertPower::LoudCalculation;
    let Some(LoudCalculation { rebate, .. }) = p.expert else { return };
    if !turned || rebate <= 0 || p.loud_owed <= 0 {
        return;
    }
    let back = p.loud_owed * rebate.clamp(0, 100) / 100;
    p.strength += back;
    // The bill is settled, so a second turn does not refund the same borrowing
    // twice. What is borrowed after this is a new bill.
    p.loud_owed -= back;
}

/// **The overture is played again**, once for every encore bought.
///
/// The window reopens for its full length from the moment the fight turns —
/// which is deliberately *not* "and again immediately", because a window that
/// reopened the instant it shut would be indistinguishable from a longer
/// `window_ms`, and a knob that duplicates the knob beside it is a knob nobody
/// can spend a point on knowingly.
fn encore_when_it_turns(p: &mut Combatant, t: u32, turned: bool) {
    use crate::expert::ExpertPower::OpeningNumber;
    let Some(OpeningNumber { window_ms, .. }) = p.expert else { return };
    if !turned || p.opening_encores <= 0 {
        return;
    }
    p.opening_encores -= 1;
    p.opening_until_ms = t + window_ms.max(0) as u32;
}

/// Has this fighter room to make another curse permanent, and how hard does
/// it bite?
///
/// **The condition is the lander's board**, which is the whole of Standing
/// Fact: *how few finished items are you carrying*. `items` is the finished
/// items a fight was handed, so a bare build reads low and a packed one does
/// not — and there is nothing to look up, because that is already the list a
/// fight runs on.
fn standing_fact_room(who: &Combatant) -> Permanence {
    match who.expert {
        Some(crate::expert::ExpertPower::StandingFact { worn, carry, bite, .. })
            if who.items.len() as i32 <= worn && (who.facts_standing as i32) < carry =>
        {
            Some(bite)
        }
        _ => None,
    }
}

/// Spend a curse standing on the other side to pay for a cast.
///
/// **The cheapest goes first, or the ripest once `pick` is bought** — the one
/// knob in the block that changes a decision rather than a number, and worth a
/// point precisely because it is free value afterwards: a requisition that
/// spends something about to be lost anyway costs nothing at all.
///
/// A relisted curse comes back **still running**, which the engine can do
/// because a curse is a duration and re-landing one is a thing `land_curse`
/// already does.
fn requisition(
    p: &mut Combatant,
    foes: &mut [Combatant],
    me: Ref,
    t: u32,
    log: &mut Vec<LogEntry>,
) -> bool {
    use crate::expert::ExpertPower::CurseRequisition;
    let Some(CurseRequisition { worth, relist, pick: ripest, .. }) = pick(p, foes, me).expert else {
        return false;
    };
    if pick(p, foes, me).reqs_left <= 0 {
        return false;
    }
    // Whoever is standing opposite. In a brawl the first one carrying anything
    // — a requisition is a form, not a search.
    let victim = if me.side == Side::Player { Side::Enemy } else { Side::Player };
    let found = if victim == Side::Enemy {
        foes.iter().position(|f| !f.curses.is_empty())
    } else {
        (!p.curses.is_empty()).then_some(0)
    };
    let Some(vi) = found else { return false };
    let target: &mut Combatant = if victim == Side::Enemy { &mut foes[vi] } else { p };
    let Some(kind) = target.curses.spend_one(ripest > 0) else { return false };

    let me_c = pick(p, foes, me);
    me_c.reqs_left -= 1;
    me_c.reqs_done += 1;
    // A spent curse covers `worth` percent of a cast; the excess banks as Funny
    // rather than evaporating, which is the Sergeant's ledger and the Keeper's
    // feeding each other.
    let covered = SPELL_MANA_COST * worth.max(0) / 100;
    me_c.mana += (covered - SPELL_MANA_COST).max(0);
    let done = me_c.reqs_done;
    let back = crate::expert::ExpertPower::relist_every(relist).is_some_and(|n| done % n == 0);
    if back {
        let on = if victim == Side::Enemy { Ref { side: Side::Enemy, who: vi } } else { Ref { side: Side::Player, who: 0 } };
        // Unaimed: a curse coming *back* was never aimed in the first place,
        // and paying for the pick a second time would be paying twice for one
        // landing.
        let target: &mut Combatant = if victim == Side::Enemy { &mut foes[vi] } else { p };
        land_curse(target, on, kind, StunAim::Unaimed, t, log);
    }
    covered >= SPELL_MANA_COST
}

/// Can this fighter afford the cast it is about to make?
///
/// **One door, and three of the ten experts argue at it.** Mana is what a
/// casting item spends and everybody starts a fight with none, so *what
/// happens when you cannot pay* is the one question the Funnel Sergeant's half
/// of the roster is all about — and putting the three of them in three places
/// would have been three answers to it.
///
/// The order is the argument. A free window costs nothing, so it is asked
/// first; a discount only matters once you are paying; and buying the shortfall
/// is the last resort, because it is the only one that costs you something
/// other than Funny.
fn pay_for_a_cast(me: &mut Combatant, t: u32) -> bool {
    use crate::expert::ExpertPower::*;
    // **Opening Number: the first seconds are free**, and what is left when
    // the window shuts may land as armour.
    if let Some(OpeningNumber { after, bank, .. }) = me.expert {
        if t < me.opening_until_ms {
            return true;
        }
        // The window has closed. Once, and only once, the unspent Funny is
        // banked — a second payout would make reopening the window worth
        // something for its own sake, which is not what an encore is for.
        if bank > 0 && !me.opening_banked {
            me.opening_banked = true;
            me.armor += me.mana.max(0);
            me.mana = 0;
        }
        // **The price is the power's**, worked out in one place and printed
        // from the same one — see `ExpertPower::cast_price`, and why a discount
        // that rounds down is a node that sells nothing.
        let cost = crate::expert::ExpertPower::cast_price(after);
        if me.mana >= cost {
            me.mana -= cost;
            return true;
        }
        return false;
    }
    // **Curse Requisition: a curse you landed pays for a cast.** The curse is
    // consumed off whoever is holding it, which is why the caller passes the
    // whole fighter: the pool being spent is not on this side of the fight.
    if me.mana >= SPELL_MANA_COST {
        me.mana -= SPELL_MANA_COST;
        // **Fired Funnel: a cast refunds part of what it cost.** After the
        // spend rather than instead of it, so a board with no mana still
        // cannot cast — a refund is change, not credit.
        if let Some(FiredFunnel { refund, .. }) = me.expert {
            me.mana += SPELL_MANA_COST * refund.clamp(0, 100) / 100;
        }
        return true;
    }
    // **Loud Calculation: pay the shortfall in strength.** Everything above
    // this line spends Funny; this is the only one that spends you.
    if let Some(LoudCalculation { rate, cap, .. }) = me.expert {
        let short = SPELL_MANA_COST - me.mana.max(0);
        let room = (cap - me.loud_bought).max(0);
        let mut want = short.min(room);
        if want > 0 {
            // The free points first — that is what buying "past empty" means —
            // then whatever strength can still afford, never below 1.
            let free = want.min(me.loud_free_left);
            let mut charged = want - free;
            let rate = rate.max(1);
            let affordable = ((me.strength - 1).max(0) * 10) / rate;
            if charged > affordable {
                charged = affordable;
                want = free + charged;
            }
            if want > 0 {
                let cost = charged * rate / 10;
                me.loud_free_left -= free;
                me.loud_bought += want;
                me.loud_owed += cost;
                me.strength -= cost;
                me.mana += want;
            }
        }
        if me.mana >= SPELL_MANA_COST {
            me.mana -= SPELL_MANA_COST;
            return true;
        }
    }
    // **Requisitioned Silence: a cast you cannot pay for is said anyway.**
    // Last, after everything that could still afford it has tried, because
    // this is what happens when nothing can. It does not return `true` — the
    // cast still does not go off; what it does is bank mind damage for the
    // activation to land, which is the whole joke: the form was not signed and
    // the word was said.
    if let Some(RequisitionedSilence { rate, .. }) = me.expert {
        if me.silent_left > 0 && rate > 0 {
            me.silent_left -= 1;
            me.silent_owed += SPELL_MANA_COST * rate / 100;
        }
    }
    false
}

/// **Cold Stoke: a curse you land is fuel.**
///
/// Called by whoever landed one, off the *lander* rather than the sufferer,
/// which is the division `Curses::bite` already makes — a curse's tuning is set
/// by whoever put it there.
fn stoke_on_curse(c: &mut Combatant, deep: bool) {
    if let Some(crate::expert::ExpertPower::ColdStoke { per_stack, standing, .. }) = c.expert {
        let n = per_stack.max(0) * if deep { 1 + standing.max(0) } else { 1 };
        c.empowerment += n.max(0) as u32;
    }
}

fn land_curse(
    victim: &mut Combatant,
    on: Ref,
    kind: CurseKind,
    aim: StunAim,
    t: u32,
    log: &mut Vec<LogEntry>,
) {
    land_curse_for(victim, on, kind, aim, t, log, None);
}

/// The same, told who landed it.
///
/// **Standing Fact is the only rule in the game about the *lander* of a
/// curse**, and every other curse in the engine is a fact about the sufferer —
/// which is why `Curses` holds no author and this takes one instead. `None` is
/// every existing caller: a curse whose author nobody asked about behaves
/// exactly as it always has, which is what makes this safe to add to a path
/// four other things already use.
/// `Some(bite)` when the fighter landing this curse is a Standing Fact with
/// room to hold another. A plain value rather than the author's own
/// `&mut Combatant`, because the author and the victim are two fighters in one
/// vector and the borrow checker is right to refuse both at once — the caller
/// decides, this lands it, and the caller books it.
type Permanence = Option<i32>;

fn land_curse_for(
    victim: &mut Combatant,
    on: Ref,
    kind: CurseKind,
    aim: StunAim,
    t: u32,
    log: &mut Vec<LogEntry>,
    by: Permanence,
) -> bool {
    let who = on.who as u8;
    let on = on.side;
    if kind == CurseKind::Stun {
        if let Some((index, ms)) = land_stun(victim, aim, t) {
            let item = victim.items[index].name.clone();
            let aimed = aim == StunAim::Strongest;
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Stunned { on, index, item, duration_ms: ms, aimed },
            });
        }
        // **A stun is never a standing fact.** It rides on one item and ends;
        // making one permanent would stop a piece of gear for the whole fight
        // with no way to answer it, which is the outcome `STUN_CAP_MS` exists
        // to prevent one tenth as badly.
        return false;
    }
    // **Standing Fact: a curse landed by somebody wearing little enough does
    // not expire.** The condition is on the lander's board — *how few finished
    // items are you carrying* — and the allowance is counted on the lander too,
    // so a brawl does not let three foes share it or divide it.
    let standing = by.is_some();
    if let Some(bite) = by.filter(|b| *b > 0) {
        victim.curses.bite_harder(bite);
    }
    let ms = if standing {
        victim.curses.apply_standing(kind)
    } else {
        victim.curses.apply(kind, victim.curse_resist)
    };
    if ms > 0 {
        let stacks = victim.curses.stacks_of(kind);
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::Cursed { on, kind, duration_ms: ms, stacks },
        });
    }
    standing && ms > 0
}

/// `land_stun`, for a test that wants to put two items in front of it and see
/// which one it picks.
///
/// The choice is the whole of the Lightning Rod and most of what keeps an
/// aimed stun fair, and it is not reachable through `simulate` without
/// building a board that happens to be cursed.
pub fn land_stun_for_test(
    victim: &mut Combatant,
    aim: StunAim,
    at_ms: u32,
) -> Option<(usize, u32)> {
    land_stun(victim, aim, at_ms)
}

fn land_stun(victim: &mut Combatant, aim: StunAim, at_ms: u32) -> Option<(usize, u32)> {
    let duration = CurseKind::Stun.landing_ms(victim.curse_resist);
    if duration == 0 || victim.items.is_empty() {
        return None;
    }
    victim.stun_count = victim.stun_count.wrapping_add(1);

    // The rod first, whatever the aim was.
    //
    // "Every curse applied to your board lands on whatever covers it", and a
    // stun is the only curse in this game that has a target on the board at
    // all - the other three land on the fighter and always have. So this is
    // the whole of the rule, and it is a decision rather than a reward: lay
    // the rod under something you do not mind losing the use of, and the thing
    // you do mind stops being picked.
    // An unshakable item is not a candidate at all - not for the rod's pull,
    // not for the aimed pick, not for the ordinary one.
    if let Some(i) =
        victim.items.iter().position(|it| it.attracts_curses && !it.unshakable)
    {
        let item = &mut victim.items[i];
        item.stun_ms = (item.stun_ms + duration).min(STUN_CAP_MS);
        return Some((i, item.stun_ms));
    }
    // Everything that can be stopped. An unshakable item is not a candidate
    // for any of the three picks, so a board of nothing but those takes no
    // stun at all rather than taking one somewhere odd.
    let takers: Vec<usize> =
        victim.items.iter().enumerate().filter(|(_, it)| !it.unshakable).map(|(i, _)| i).collect();
    if takers.is_empty() {
        return None;
    }
    let idx = match aim {
        StunAim::Strongest => takers
            .iter()
            .copied()
            // **Never a broken one, whatever it is rated.** Being stopped
            // breaks a tie between equals — a stun on a stopped item is a
            // curse wasted for a second — but a stun on a *broken* item is a
            // curse wasted for the whole fight, which is not a tie-break, it is
            // a waste. So it sorts ahead of the rating rather than behind it,
            // and a board of nothing but broken items still takes one
            // somewhere.
            //
            // This cannot move a fight that existed before M10.1: nothing was
            // ever `broken`, so the first key was constant and the order is the
            // one it always was.
            .max_by_key(|&i| {
                let it = &victim.items[i];
                (!it.broken, it.rating, it.stun_ms == 0)
            })?,
        StunAim::Unaimed => {
            let n = takers.len();
            // A cheap integer hash of the fight's own state. Time alone
            // clusters, because stuns arrive on cooldown boundaries.
            let mix = (at_ms as u64)
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add((victim.stun_count as u64).wrapping_mul(0x2545_F491_4F6C_DD1D));
            let start = (mix >> 33) as usize % n;
            // Walk from there to the first item that is not already stopped,
            // falling back to the original pick if every one of them is.
            takers[(0..n)
                .map(|k| (start + k) % n)
                .find(|&i| {
                    let it = &victim.items[takers[i]];
                    it.stun_ms == 0 && !it.broken
                })
                .unwrap_or(start)]
        }
    };

    let item = &mut victim.items[idx];
    // Stacks pile onto that item's clock rather than refreshing it, so a
    // second stun landing on the same item is worth something.
    item.stun_ms = (item.stun_ms + duration).min(STUN_CAP_MS);
    Some((idx, item.stun_ms))
}

/// Run one item.
///
/// Returns whether it should be run once more straight away - Overtake, and
/// only on an item's first firing of the fight. The caller re-runs it rather
/// than this function recursing, so that `check_down` sits between the two:
/// an opening blow that kills does not get a second one.
fn activate(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    me: Ref,
    idx: usize,
    t: u32,
    log: &mut Vec<LogEntry>,
) -> bool {
    let front = aim_of(foes, p.aim);
    let side = me.side;
    // Taken before the local rebindings below shadow `me` with a combatant.
    let who = me.logged_as(front);
    let mut item = pick(p, foes, me).items[idx].clone();

    // **The spin is spent here, on the tick it fires.**
    //
    // Taken off the combatant's own copy rather than the clone above, because
    // the clone is this activation's working copy and the tally belongs to the
    // item. An overtake runs the whole activation twice and the second run
    // finds nothing banked, which is right: one spend a spin.
    let spun = {
        // **`bleed` is the Patented Funnel's half of `spin_keep`.** The rule
        // and the knob add rather than one replacing the other, which is the
        // division this whole block makes: the rule is the switch and the knob
        // is the tuning, and there are not two answers to how much a turning
        // item keeps.
        let keep = {
            let me = pick(p, foes, me);
            me.spin_keep
                + match me.expert {
                    Some(crate::expert::ExpertPower::PatentedFunnel { bleed, .. }) => {
                        bleed.max(0) as u32
                    }
                    _ => 0,
                }
        };
        let it = &mut pick(p, foes, me).items[idx];
        let n = it.spin_stacks;
        // Everything banked is spent; what `keep` buys is starting again from
        // there rather than from nothing. Capped at what was actually there,
        // so a node cannot hand an item stacks it never earned.
        it.spin_stacks = keep.min(n);
        n
    };
    if spun > 0 {
        let pct = spun as i32 * SPIN_PCT_PER_TURN;
        item.power += pct;
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::Spun { side, index: idx, item: item.name.clone(), stacks: spun, pct },
        });
    }

    // A spell swaps in the payload whose turn it is. A book has bound one and
    // casts it every time; a crystal ball cycles through the two or three it
    // holds, so the same item does something different each time it comes
    // round. The index lives on the combatant's copy, not this clone.
    // Echo: every nth activation runs its payload a second time.
    let echoes = {
        let me = pick(p, foes, me);
        me.activations += 1;
        me.echo_every > 0 && me.activations % me.echo_every == 0
    };
    // **Productivity: every nth act of an enched item runs twice, and the item
    // is slower afterwards for the rest of the fight.** Counted on the item,
    // like `has_fired` below and unlike `Echo` above, because two enched items
    // deserve two schedules — which is what bolting an ench onto both is for.
    //
    // The slowdown is written *after* the double is decided and lands on the
    // item's own cooldown, so the swing that earned it lands in full and
    // everything after it is dearer. Same bargain as `Fragile`, and the same
    // ordering argument.
    let doubles = {
        let sched = pick(p, foes, me).productivity;
        let it = &mut pick(p, foes, me).items[idx];
        it.fires += 1;
        match sched {
            Some((every, slower)) if it.enched && every > 0 && it.fires % every == 0 => {
                it.slowed_pct = (it.slowed_pct + slower).min(MAX_PRODUCTIVITY_SLOW_PCT);
                // Recomputed off the slowdown total rather than multiplied in
                // place, so ten procs do not compound into a stopped item — a
                // running cooldown scaled repeatedly is the same fault as a
                // replay subtracting damage from its own total.
                it.cooldown_ms = (it.base_cooldown_ms as i64 * 100
                    / (100 - it.slowed_pct as i64).max(1))
                    as u32;
                true
            }
            _ => false,
        }
    };
    // Overtake: the first firing of the fight runs a second time.
    //
    // Returned to the caller rather than repeated here, because what runs
    // again is the **whole activation** - triggers, pools, spells and all -
    // and not the blow. `reps` would have been the cheap place to put it and
    // would have been wrong for exactly the slot the effect is for: only
    // weapons swing, gloves act entirely through triggers, and a gloves
    // effect that doubled a swing would do nothing at all.
    //
    // `has_fired` is set here, at the top, so the second run cannot qualify
    // on its own - one repeat, not a loop.
    let overtakes = {
        let it = &mut pick(p, foes, me).items[idx];
        let first = it.overtakes && !it.has_fired;
        it.has_fired = true;
        first
    };
    let mut cast_name = None;
    if !item.casts.is_empty() {
        let n = item.casts.len();
        let which = item.cast_index % n;
        let cast = item.casts[which].clone();
        item.physical_damage = cast.stats.physical_damage;
        item.magic_damage = cast.stats.magic_damage;
        item.rage = cast.stats.rage;
        item.faith = cast.stats.faith;
        item.nature = cast.stats.nature;
        item.mind = cast.stats.mind;
        item.armor = cast.stats.armor;
        item.mana = cast.stats.mana;
        item.triggers = cast.triggers;
        // The spells that did not come up this turn still answer the one that
        // did. This is what makes a ball worth more than its spells apart:
        // only a crystal ball holds several, so only a ball can pay this out.
        for (i, other) in item.casts.iter().enumerate() {
            if i == which {
                continue;
            }
            for trig in &other.triggers {
                if let Trigger::OnOtherCast(a) = trig {
                    item.triggers.push(Trigger::OnActivate(*a));
                }
            }
        }
        if n > 1 {
            cast_name = Some(cast.name);
        }
        // A ball speaks with two voices. This is what a ball IS - a book binds
        // one spell and casts it every time, and if a ball only ever cast one
        // too then holding three of them bought nothing but variety. The
        // second is whichever is next in the cycle, so which pair you get
        // still changes each time it comes round.
        let extra = (BALL_VOICES - 1) as usize;
        for k in 0..extra.min(n.saturating_sub(1)) {
            let also = &item.casts[(which + 1 + k) % n];
            item.physical_damage += also.stats.physical_damage;
            item.magic_damage += also.stats.magic_damage;
            item.rage += also.stats.rage;
            item.faith += also.stats.faith;
            item.nature += also.stats.nature;
            item.mind += also.stats.mind;
            item.armor += also.stats.armor;
            item.mana += also.stats.mana;
            item.triggers.extend(also.triggers.iter().copied());
        }
        pick(p, foes, me).items[idx].cast_index = (which + 1) % n;

        // A spell has two intensities. Paid for, it lands in full; unpaid, it
        // still goes off but weakly. Mana stops being a thing some gear
        // happens to grant and becomes the difference between a spell that
        // works and a spell that merely happens.
        //
        // One price per activation, covering every voice: a ball is meant to
        // be the committed choice, and charging it twice for being one would
        // undo that.
        let paid = {
            if pay_for_a_cast(pick(p, foes, me), t) {
                true
            } else {
                // **Curse Requisition, and it is the one payment that cannot
                // be made from inside one fighter.** The pool being spent is a
                // curse standing on somebody *else*, so it is asked here,
                // where both sides are in hand, rather than in `pay_for_a_cast`
                // — which is the same reason `land_curse` takes the victim.
                requisition(p, foes, me, t, log)
            }
        };
        let scale = if paid { EMPOWERED_CAST_PCT } else { WEAK_CAST_PCT };
        for v in [
            &mut item.physical_damage,
            &mut item.magic_damage,
            &mut item.mind,
            &mut item.armor,
        ] {
            *v = *v * scale / 100;
        }
        let remaining = pick(p, foes, me).mana;
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::Cast { side, paid, cost: SPELL_MANA_COST, remaining },
        });
    }

    log.push(LogEntry {
        who,
        at_ms: t,
        event: Event::Activate {
            side,
            item: match cast_name {
                Some(spell) => format!("{} ({})", item.name, spell),
                None => item.name.clone(),
            },
            index: idx,
        },
    });

    // Weapons swing; everything else just does its job. A monster's attacks
    // have no slot and always count as weapons.
    let is_weapon = item.slot.map(|s| s == SlotKind::Weapon).unwrap_or(true);
    if is_weapon {
        // Strength reaches every weapon; power does not reach past the one
        // carrying it. The two amplifiers are the exception and are meant to
        // be - they apply to whatever is swinging - but each one applies to
        // its own lane only. Empowerment is bought with mana and sharpens
        // magic; Spellblade is bought flat and sharpens iron.
        let (strength, empower, whetted) = {
            let me = pick(p, foes, me);
            (me.strength, me.magic_empower(), me.physical_empower())
        };
        // The wearer's power, plus whatever ink is bound into this item alone.
        // Rage held sharpens the physical half.
        let (rage, phys_pierce, magic_pierce) = {
            let me = pick(p, foes, me);
            (me.held_bonus().physical_damage, me.physical_pierce, me.magic_pierce)
        };
        // The item's own numbers already carry its power - it was applied
        // when the profile was built, so the card and the fight agree. What
        // the wearer brings does not, so it picks the multiplier up here -
        // and which multiplier depends on which lane the number is landing
        // in. A board holding twenty empowerment stacks swings iron exactly
        // as hard as a board holding none.
        let mult_magic =
            |flat: i32| -> i32 { ((flat as i64) * (100 + empower) as i64 / 100).max(0) as i32 };
        let mult_phys =
            |flat: i32| -> i32 { ((flat as i64) * (100 + whetted) as i64 / 100).max(0) as i32 };
        let from_wearer =
            (((rage + strength) as i64 * item.power as i64) / 100).max(0) as i32;
        let physical = mult_phys(item.physical_damage + from_wearer);
        // Transmute: part of the iron lands again as magic. Taken off the
        // physical number after it is settled, so what crosses is a blow that
        // was already whetted rather than one that is about to be empowered -
        // a conversion, not a second amplifier.
        let transmute = pick(p, foes, me).transmute;
        let magic = mult_magic(item.magic_damage) + physical * transmute / 100;
        // Momentum: the longer the fight runs, the harder you swing. Iron, so
        // it is Spellblade's.
        let momentum = pick(p, foes, me).momentum * (t / 1000) as i32;
        let physical =
            physical + mult_phys((((momentum as i64) * item.power as i64) / 100) as i32);
        // A fork copies the cast, and only a cast: a blade swings once
        // however many stacks are up.
        let forks = if item.casts.is_empty() { 0 } else { pick(p, foes, me).forking };
        let reps: u32 = if echoes { 2 } else { 1 } * if doubles { 2 } else { 1 } * (1 + forks);

        // **The wrong sense.** Everything the blow was about to be is
        // surrendered here, before a single point of it crosses - which is
        // what makes it a trade rather than a bonus. A version that let the
        // damage land and added mind on top would be a free multiplier, and
        // every board in the game would wear this crest.
        let wrong = pick(p, foes, me).wrong_sense;
        let (physical, magic) = if wrong {
            let given = (physical + magic) as i64 * reps as i64;
            pick(p, foes, me).surrendered += given;
            (0, 0)
        } else {
            (physical, magic)
        };
        // The log reports the swing, not what survived the defences: a hit
        // that is turned aside completely still has to show up, or a player
        // stacking resistance sees nothing happening at all.
        let swing = physical + magic;
        // One blow per repetition, each aimed afresh. An echo or a fork is
        // another attack, so it takes the next one along - and a line of its
        // own in the log, which is also more honest about what happened than
        // folding two blows into one number was.
        for _ in 0..reps {
            let aim = aim_of(foes, p.aim);
            let at = me.other(aim);
            let mut absorbed_total = 0;
            // Wumpus Hunter: the first blow of a fight goes through whatever
            // they have flat in front of it. Read and spent here rather than
            // at each damage site, because "the first hit" has to mean one
            // hit however many ways an activation can reach somebody.
            let unstoppable = {
                let me = pick(p, foes, me);
                let owed = me.first_blood;
                me.first_blood = false;
                owed
            };
            for (amount, kind, pierce) in [
                (physical, DamageType::Physical, phys_pierce),
                (magic, DamageType::Magic, magic_pierce),
            ] {
                if amount <= 0 {
                    continue;
                }
                let target = pick(p, foes, at);
                let (absorbed, _) = target.take_typed_with(amount, kind, pierce, unstoppable);
                absorbed_total += absorbed;
            }
            // Reflection. What the armour ate is turned back on whoever swung
            // it, which is why this is the body's attack and nothing else's: it
            // needs the blow to land and be absorbed first, so it pays nothing
            // to a board that dies quickly and everything to one built to be
            // hit. Taken as physical, and it cannot itself be reflected - the
            // return is dealt directly rather than back through this path, so
            // two reflecting boards cannot bounce a hit between them for ever.
            let pct = pick(p, foes, at).reflect;
            if pct > 0 && absorbed_total > 0 {
                let back = absorbed_total * pct / 100;
                if back > 0 {
                    let swinger = pick(p, foes, me);
                    swinger.health -= back;
                    log.push(LogEntry {
                        who: me.logged_as(aim),
                        at_ms: t,
                        event: Event::Reflected { side: at.side, damage: back },
                    });
                }
            }
            if swing > 0 {
                let target = pick(p, foes, at);
                let (hp, ar) = (target.health, target.armor);
                log.push(LogEntry {
                    who: me.logged_as(aim),
                    at_ms: t,
                    event: Event::Hit {
                        by: side,
                        by_item: Some(idx),
                        damage: swing,
                        absorbed: absorbed_total,
                        target_health: hp,
                        target_armor: ar,
                    },
                });
            }
            // Next blow goes to the next one along.
            if me.side == Side::Player && foes.len() > 1 {
                p.aim = aim + 1;
            }
        }
        // Leeching: a share of what you dealt comes back.
        let leech = pick(p, foes, me).leech;
        if leech > 0 && swing > 0 {
            let me = pick(p, foes, me);
            let back = (swing * reps as i32) * leech / 100;
            me.health = (me.health + back).min(me.max_health);
        }
    }

    if let Some(kind) = item.curse {
        apply(p, foes, me, Action::Curse { kind, target: Target::Enemy }, t, log, Some(idx));
    }

    // **Four of M16's eleven say something on an activation**, and all four
    // arrive here rather than each finding its own place to hit from: a mind
    // hit is one thing, dealt once, and four ways of adding to it must not
    // become four places that deal one.
    let extra = {
        let who_i_am = me;
        let me = pick(p, foes, who_i_am);
        let mut extra = 0;
        match me.expert {
            // **Said plainly.** Wearing little enough, strength counts.
            Some(crate::expert::ExpertPower::LoudDoubt { worn, rate, .. })
                if me.items.len() as i32 <= worn =>
            {
                extra += me.strength * rate.max(0) * 25 / 100;
            }
            // **Every point the furnace burned is also said**, at halves.
            Some(crate::expert::ExpertPower::AshAndWhisper { rate, .. }) => {
                extra += me.burned.iter().sum::<i32>() * rate.max(0) / 2;
            }
            _ => {}
        }
        // A silence that was banked is said on the next activation and once.
        extra += std::mem::take(&mut pick(p, foes, who_i_am).silent_owed);
        // And what the tree said, which is the character's rather than the
        // item's — see `Held::mind`.
        extra += pick(p, foes, who_i_am).said;
        extra
    };
    // **The rumour, which is a share of what is left rather than a number.**
    // Its own block because it is measured off the *target* and everything
    // above is measured off the swinger.
    let rumour = {
        let pct = match pick(p, foes, me).expert {
            Some(crate::expert::ExpertPower::LicensedRumour { pct, .. }) if item.enched => {
                pct.max(0)
            }
            _ => 0,
        };
        if pct == 0 { 0 } else { pick(p, foes, me.other(front)).max_health * pct / 1000 }
    };

    if item.mind + extra + rumour > 0 {
        // Dread is the wearer's, so it is read off the swinger before the
        // blow leaves - the same shape as empowerment, which is picked up on
        // the way out rather than applied on arrival.
        let (raw, pierce) = {
            let me = pick(p, foes, me);
            (
                me.wrong_sense_multiplied(item.mind + extra + me.mind_bonus()) + rumour,
                me.mind_pierce,
            )
        };
        let target = pick(p, foes, me.other(front));
        let dealt = target.take_mind_pierced(raw, pierce);
        let mh = target.max_health;
        if dealt > 0 {
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::MindHit { by: side, amount: dealt, target_max_health: mh },
            });
        }
    }

    if item.armor > 0 {
        let me = pick(p, foes, me);
        let got = me.gain_armor(item.armor);
        let total = me.armor;
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::GainArmor { side, amount: got, total },
        });
    }

    if item.mana > 0 {
        let me = pick(p, foes, me);
        me.mana += item.mana;
        let total = me.mana;
        log.push(LogEntry { who, at_ms: t, event: Event::GainMana { side, amount: item.mana, total, accrued: false } });
    }

    let banked = pick(p, foes, me).adaptable;
    if banked > 0 {
        let me = pick(p, foes, me);
        me.mana += banked;
        me.rage += banked;
        me.faith += banked;
        me.nature += banked;
    }
    // Riposte: watching them act gives your own gear a nudge.
    {
        let ms = pick(p, foes, me.other(front)).riposte;
        if ms > 0 {
            for it in &mut pick(p, foes, me.other(front)).items {
                it.progress_ms += ms;
            }
        }
    }

    for (amount, label) in [(item.rage, "rage"), (item.faith, "faith"), (item.nature, "nature")] {
        if amount > 0 {
            let me = pick(p, foes, me);
            match label {
                "rage" => me.rage += amount,
                "faith" => me.faith += amount,
                _ => me.nature += amount,
            }
            let total = match label {
                "rage" => me.rage,
                "faith" => me.faith,
                _ => me.nature,
            };
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::GainResource { side, what: label, amount, total, accrued: false },
            });
        }
    }

    // A repeat is expanded here rather than in the match below, so the thing
    // being repeated stays an ordinary trigger and every arm keeps working.
    let mut firing: Vec<Trigger> = Vec::with_capacity(item.triggers.len());
    for trigger in &item.triggers {
        match *trigger {
            Trigger::PerAdjacentEmpty(inner) => {
                for _ in 0..item.open_cells {
                    firing.push(*inner);
                }
            }
            other => firing.push(other),
        }
    }

    // **A rule the tree granted, not a trigger the item carries.**
    //
    // Here rather than folded into `item.triggers` at profile time, because
    // the profile is the board's answer and this is the character's: two
    // players with the identical board do not have the identical fight, and
    // an item's card must not start claiming a curse the item does not own.
    let granted: Vec<crate::curse::CurseKind> = {
        let me = pick(p, foes, me);
        match item.slot {
            Some(s) => me
                .curse_on_activate
                .iter()
                .filter(|(want, _)| *want == s)
                .map(|(_, k)| *k)
                .collect(),
            None => Vec::new(),
        }
    };
    // **Cursed Licence: an enched item lands its frame's curse `stack` times.**
    // The rule says *which* curse a frame lands and the class says *how many*,
    // which is the same division `SpinExtra` makes over a spin an ench granted
    // — and the reason the licence is that node's licence rather than a new
    // mechanic. An item with nothing bolted to it lands one, like everybody
    // else's.
    let times = {
        let me = pick(p, foes, me);
        match me.expert {
            Some(crate::expert::ExpertPower::CursedLicence { stack })
                if me.items[idx].enched =>
            {
                stack.max(1) as u32
            }
            _ => 1,
        }
    };
    for kind in granted {
        for _ in 0..times {
            apply(p, foes, me, Action::Curse { kind, target: Target::Enemy }, t, log, Some(idx));
        }
    }

    for trigger in &firing {
        match *trigger {
            Trigger::OnActivate(action) => apply(p, foes, me, action, t, log, Some(idx)),
            Trigger::SpendGold { cost, budget, on_success } => {
                let paid = {
                    let me = pick(p, foes, me);
                    let it = &mut me.items[idx];
                    // Two ways to come up short, and they are not the same:
                    // the budget is the promise the piece made, the purse is
                    // the money you actually have.
                    if it.gold_spent + cost <= budget && me.purse >= cost {
                        it.gold_spent += cost;
                        it.gold_paid += 1;
                        let times = it.gold_paid;
                        me.purse -= cost;
                        Some((times, me.purse))
                    } else {
                        None
                    }
                };
                if let Some((times, left)) = paid {
                    log.push(LogEntry {
                        who,
                        at_ms: t,
                        event: Event::Spent { side, amount: cost, remaining: left },
                    });
                    // Harder every time it pays. `scaled` touches outcomes and
                    // never costs, so the price stays flat while the payout
                    // climbs - which is the whole shape of the thing.
                    let grown = on_success.scaled(100 * times as i32);
                    apply(p, foes, me, grown, t, log, Some(idx));
                }
            }
            Trigger::SpendMana { cost, on_success, on_failure } => {
                let paid = {
                    let me = pick(p, foes, me);
                    if me.mana >= cost {
                        me.mana -= cost;
                        true
                    } else {
                        false
                    }
                };
                let remaining = pick(p, foes, me).mana;
                log.push(LogEntry {
                    who,
                    at_ms: t,
                    event: Event::ManaCheck { side, cost, paid, remaining },
                });
                apply(p, foes, me, if paid { on_success } else { on_failure }, t, log, Some(idx));
            }
            Trigger::Consume { what, each, per } => {
                // Takes the whole pool and pays out by the handful. The
                // remainder below one handful is spent too - the trigger is
                // "empty your reserve", not "spend a multiple of `each`".
                let (held, times) = {
                    let me = pick(p, foes, me);
                    let held = me.pool(what).max(0);
                    let times = held / each.max(1);
                    if times > 0 {
                        me.set_pool(what, 0);
                        // Confluence pays on this too: what one pool spends,
                        // the others drink.
                        let back = me.confluence * held / 100;
                        if back > 0 {
                            for other in
                                [Resource::Mana, Resource::Rage, Resource::Faith, Resource::Nature]
                            {
                                if other != what {
                                    let total = me.pool(other) + back;
                                    me.set_pool(other, total);
                                }
                            }
                        }
                    }
                    (held, times)
                };
                if times > 0 {
                    log.push(LogEntry {
                        who,
                        at_ms: t,
                        event: Event::ResourceCheck {
                            side,
                            what: what.name(),
                            cost: held,
                            paid: true,
                            remaining: 0,
                        },
                    });
                    for _ in 0..times {
                        apply(p, foes, me, per, t, log, Some(idx));
                    }
                }
            }
            Trigger::Spend { what, cost, on_success, on_failure } => {
                let paid = {
                    let me = pick(p, foes, me);
                    let held = me.pool(what);
                    if held >= cost {
                        me.set_pool(what, held - cost);
                        // Confluence: what one pool spends, the others drink.
                        let back = me.confluence * cost / 100;
                        if back > 0 {
                            for other in
                                [Resource::Mana, Resource::Rage, Resource::Faith, Resource::Nature]
                            {
                                if other != what {
                                    let total = me.pool(other) + back;
                                    me.set_pool(other, total);
                                }
                            }
                        }
                        true
                    } else {
                        false
                    }
                };
                let remaining = pick(p, foes, me).pool(what);
                log.push(LogEntry {
                    who,
                    at_ms: t,
                    event: Event::ResourceCheck { side, what: what.name(), cost, paid, remaining },
                });
                apply(p, foes, me, if paid { on_success } else { on_failure }, t, log, Some(idx));
            }
            Trigger::PerAdjacentItem { action, same_slot_only: _ } => {
                for _ in 0..item.adjacent_assembled_same_slot {
                    apply(p, foes, me, action, t, log, Some(idx));
                }
            }
            // Already expanded above; a nested one is not authored.
            Trigger::PerAdjacentEmpty(_) => {}
            // Fired before the first tick, not on the cooldown.
            Trigger::OnBattleStart(_) => {}
            // Waits for the *other side* to act, which `notify_opponents`
            // answers. Here it does nothing, exactly like the three board-side
            // reactions below it.
            Trigger::OnEnemyActivate(_) => {}
            // These wait for someone else to act.
            Trigger::OnAdjacentActivate(_)
            | Trigger::OnAlignedActivate(_)
            | Trigger::OnDiagonalActivate(_)
            | Trigger::OnOtherCast(_) => {}
            // A watcher does not act on its own cadence either. It is fed by
            // `notify_watchers` as the events it counts go past.
            Trigger::Watch { .. } => {}
        }
    }

    // Untimely: an Oracle reaches past the gear and at the clock behind it.
    let untimely = pick(p, foes, me).untimely;
    if untimely > 0 {
        let due = {
            let me = pick(p, foes, me);
            me.untimely_count = me.untimely_count.wrapping_add(1);
            me.untimely_count % untimely == 0
        };
        if due {
            for kind in [CurseKind::Stun, CurseKind::Misfire] {
                let victim = pick(p, foes, me.other(front));
                land_curse(victim, me.other(front), kind, StunAim::Unaimed, t, log);
            }
        }
    }

    // Cascade: everything else moves a little closer to firing. Never the item
    // that just went off, or a single fast item would wind itself up forever.
    let cascade = pick(p, foes, me).cascade;
    if cascade > 0 {
        let me = pick(p, foes, me);
        for (i, it) in me.items.iter_mut().enumerate() {
            if i != idx {
                it.progress_ms =
                    (it.progress_ms + cascade).min(it.cooldown_ms.saturating_sub(1));
            }
        }
    }

    // Finally, let the neighbours react. A reaction never emits an activation
    // of its own, so two items that react to each other cannot loop.
    notify_reactors(p, foes, me, idx, t, log);
    // And the other side, which nothing answered until the feet learned to.
    notify_opponents(p, foes, me, t, log);

    // **And it breaks here, at the end.** After everything the activation pays,
    // which is the whole bargain: the swing that finishes the item lands in
    // full, and nothing after it does. Before the payout it would be an ench
    // that triples an item's power and never lets it use any of it.
    //
    // Overtake runs the whole activation twice and sets `has_fired` at the top,
    // so a fragile item that also overtakes gets both runs and then stops —
    // which is two items' worth of trade bought with two enchs, and correct.
    {
        let it = &mut pick(p, foes, me).items[idx];
        if it.fragile && !it.broken {
            it.broken = true;
            let name = it.name.clone();
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Broke { side, index: idx, item: name },
            });
        }
    }

    overtakes
}

/// Run every reaction the **other side** owes to an activation.
///
/// `notify_reactors` answers your own board - what is touching you, what shares
/// your rows, what shares a corner. Nothing in the game answered the
/// opposition until this, and the feet are what it is for: moving when they
/// move is what a stride ahead means.
///
/// It cannot loop for the same reason a board-side reaction cannot: a reaction
/// never emits an activation, so nothing it does can come back round as the
/// event it was answering. `ReduceCooldown` in particular is clamped below the
/// cooldown and so cannot fire the item it hastens.
fn notify_opponents(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    actor: Ref,
    t: u32,
    log: &mut Vec<LogEntry>,
) {
    // Every combatant but the one that acted. `Ref::Player` and one per foe,
    // the same enumeration the opening scan walks.
    let sides: Vec<Ref> =
        std::iter::once(Ref::PLAYER).chain((0..foes.len()).map(Ref::foe)).collect();
    for other in sides {
        if other == actor {
            continue;
        }
        let triggers: Vec<(usize, Trigger)> = pick(p, foes, other)
            .items
            .iter()
            .enumerate()
            .flat_map(|(j, it)| it.triggers.iter().map(move |t| (j, *t)).collect::<Vec<_>>())
            .filter(|(_, tr)| matches!(tr, Trigger::OnEnemyActivate(_)))
            .collect();
        for (j, tr) in triggers {
            if let Trigger::OnEnemyActivate(a) = tr {
                apply(p, foes, other, a, t, log, Some(j));
            }
        }
    }
}

/// Run every reaction owed to `actor_idx` firing.
fn notify_reactors(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    me: Ref,
    actor_idx: usize,
    t: u32,
    log: &mut Vec<LogEntry>,
) {
    let count = pick(p, foes, me).items.len();
    for j in 0..count {
        if j == actor_idx {
            continue;
        }
        let (touches, lines_up, corners, triggers) = {
            let c = pick(p, foes, me);
            let it = &c.items[j];
            (
                it.adjacent_items.contains(&actor_idx),
                it.aligned_items.contains(&actor_idx),
                it.diagonal_items.contains(&actor_idx),
                it.triggers.clone(),
            )
        };
        // Resonance doubles the answer, not the question: a reaction still
        // never emits an activation, so two items answering each other cannot
        // loop however loud it gets.
        let times = pick(p, foes, me).resonance.max(1);
        for tr in &triggers {
            for _ in 0..times {
                match *tr {
                    Trigger::OnAdjacentActivate(a) if touches => {
                        apply(p, foes, me, a, t, log, Some(j))
                    }
                    Trigger::OnAlignedActivate(a) if lines_up => {
                        apply(p, foes, me, a, t, log, Some(j))
                    }
                    Trigger::OnDiagonalActivate(a) if corners => {
                        apply(p, foes, me, a, t, log, Some(j))
                    }
                    _ => {}
                }
            }
        }
    }

    // And let the watchers count it. Separate pass so a reaction that fires
    // this tick is not itself counted as an activation - a watcher counts
    // items coming round, not the answers they provoke.
    for j in 0..count {
        if j == actor_idx {
            continue;
        }
        let (touches, lines_up, corners) = {
            let it = &pick(p, foes, me).items[j];
            (
                it.adjacent_items.contains(&actor_idx),
                it.aligned_items.contains(&actor_idx),
                it.diagonal_items.contains(&actor_idx),
            )
        };
        for what in [
            Watched::AnyActivation,
            Watched::AdjacentActivation,
            Watched::AlignedActivation,
            Watched::DiagonalActivation,
        ] {
            let saw = match what {
                Watched::AnyActivation => true,
                Watched::AdjacentActivation => touches,
                Watched::AlignedActivation => lines_up,
                Watched::DiagonalActivation => corners,
                Watched::CurseApplied => false,
            };
            if saw {
                tick_watchers(p, foes, me, j, what, t, log);
            }
        }
    }
}

/// Advance every `Watch` on item `j` that counts `what`, and run the payload of
/// any that came round.
///
/// A watcher never observes itself. Reactions already work that way and one
/// rule is easier to hold than two - and a fast item watching its own
/// activations would be counting its cadence, which is what `OnActivate` is
/// for.
fn tick_watchers(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    me: Ref,
    j: usize,
    what: Watched,
    t: u32,
    log: &mut Vec<LogEntry>,
) {
    // Every sighting, and whether it was the one that came round. `due` holds
    // only the ones that pay; `seen` holds all of them, because the interface
    // needs the count between payouts and has nowhere else to get it.
    let mut seen: Vec<(String, u32, u32, bool)> = Vec::new();
    let due: Vec<(Action, String)> = {
        let it = &mut pick(p, foes, me).items[j];
        let mut due = Vec::new();
        for k in 0..it.triggers.len() {
            let Trigger::Watch { what: w, count, then, repeats } = it.triggers[k] else { continue };
            if w != what || count == 0 || (!repeats && it.watch_paid[k]) {
                continue;
            }
            // The counter ticks after the event it watched has resolved, and
            // the payload runs immediately after that.
            it.watched[k] += 1;
            let paid = it.watched[k] % count == 0;
            seen.push((it.name.clone(), it.watched[k], count, paid));
            if paid {
                it.watch_paid[k] = true;
                due.push((then, it.name.clone()));
            }
        }
        due
    };
    let front = aim_of(foes, p.aim);
    let (side, who) = (me.side, me.logged_as(front));
    for (item, count_so_far, count, paid) in seen {
        log.push(LogEntry {
            who,
            at_ms: t,
            event: Event::Watched {
                side,
                item,
                // The whole phrase, not the bare noun: the log line reads
                // "counts 3 of 8 activations by your other items".
                what,
                seen: count_so_far,
                count,
                paid,
            },
        });
    }
    for (action, item) in due {
        let _ = item;
        apply(p, foes, me, action, t, log, Some(j));
    }
}

/// Let every watcher on `side` count a curse landing.
///
/// Curses are the one thing a watcher counts that is not an activation, and
/// they land on both sides, so this is called from where the curse lands rather
/// than from where an item fires.
fn notify_curse_watchers(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    me: Ref,
    t: u32,
    log: &mut Vec<LogEntry>,
) {
    if p.curse_watch_depth > 0 {
        return;
    }
    p.curse_watch_depth += 1;
    let count = pick(p, foes, me).items.len();
    for j in 0..count {
        tick_watchers(p, foes, me, j, Watched::CurseApplied, t, log);
    }
    p.curse_watch_depth -= 1;
}

/// `owner` is the item the action belongs to, needed by effects that act on
/// the item itself rather than on a combatant.
fn apply(
    p: &mut Combatant,
    foes: &mut Vec<Combatant>,
    me: Ref,
    action: Action,
    t: u32,
    log: &mut Vec<LogEntry>,
    owner: Option<usize>,
) {
    let front = aim_of(foes, p.aim);
    let side = me.side;
    // Taken before any local rebinding shadows `me` with a combatant.
    let who = me.logged_as(front);
    // `Target::Yourself` means the side that owns the item, not the item's
    // victim — several strong items pay for themselves this way.
    let resolve = |target: Target| match target {
        Target::Enemy => me.other(front),
        Target::Yourself => me,
    };

    match action {
        // ---- the cadence three ----
        Action::Prime { pct } => {
            let Some(idx) = owner else { return };
            let c = pick(p, foes, me);
            let Some(it) = c.items.get_mut(idx) else { return };
            // The same clamp `ReduceCooldown` uses, and for the same reason: a
            // bar filled to the top is a free activation, and a head start is
            // not one.
            let to = (it.cooldown_ms as i64 * pct.clamp(0, 100) as i64 / 100) as u32;
            it.progress_ms = to.min(it.cooldown_ms.saturating_sub(1));
            let (name, by) = (it.name.clone(), it.progress_ms);
            log.push(LogEntry {
                who: me.logged_as(front),
                at_ms: t,
                event: Event::Hastened { side, item: name, by_ms: by },
            });
        }
        Action::PrimeBoard { pct } => {
            let c = pick(p, foes, me);
            let mut primed: Vec<(String, u32)> = Vec::new();
            for it in c.items.iter_mut() {
                let to = (it.cooldown_ms as i64 * pct.clamp(0, 100) as i64 / 100) as u32;
                it.progress_ms = to.min(it.cooldown_ms.saturating_sub(1));
                primed.push((it.name.clone(), it.progress_ms));
            }
            for (name, by) in primed {
                log.push(LogEntry {
                    who: me.logged_as(front),
                    at_ms: t,
                    event: Event::Hastened { side, item: name, by_ms: by },
                });
            }
        }
        Action::Drift { ms } => {
            let Some(idx) = owner else { return };
            let c = pick(p, foes, me);
            let Some(it) = c.items.get_mut(idx) else { return };
            // Permanently. Nothing else in the game does this: frost lasts a
            // while and haste is a standing percentage, and both are answers
            // to what is happening. This is what the item is.
            it.cooldown_ms = it.cooldown_ms.saturating_add(ms);
        }
        Action::Unshakable => {
            let Some(idx) = owner else { return };
            let c = pick(p, foes, me);
            let Some(it) = c.items.get_mut(idx) else { return };
            it.unshakable = true;
            it.steady = true;
            it.stun_ms = 0;
        }
        Action::Fuse { a, b, into } => {
            // Both parents have to have something in them, and neither may
            // itself be a fusion - a product is not fuel. Anything else is a
            // no-op rather than a partial trade, so a board that fuses on a
            // fast cadence simply does nothing until it can afford to.
            let ok = !a.is_fused() && !b.is_fused() && into.is_fused() && a != b;
            let me_c = pick(p, foes, me);
            if ok && me_c.pool(a) > 0 && me_c.pool(b) > 0 {
                let (pa, pb) = (me_c.pool(a) - 1, me_c.pool(b) - 1);
                me_c.set_pool(a, pa);
                me_c.set_pool(b, pb);
                let total = me_c.pool(into) + 1;
                me_c.set_pool(into, total);
                log.push(LogEntry {
                    who,
                    at_ms: t,
                    event: Event::Fused {
                        side,
                        what: into.name(),
                        total,
                        from: (a.name(), pa),
                        and: (b.name(), pb),
                    },
                });
            }
        }
        Action::Curse { kind, target } => {
            // Bloodscent: what you rot, you feed on.
            if matches!(target, Target::Enemy) {
                let gain = pick(p, foes, me).bloodscent;
                if gain > 0 {
                    let me = pick(p, foes, me);
                    let total = me.pool(Resource::Rage) + gain;
                    me.set_pool(Resource::Rage, total);
                    log.push(LogEntry {
                        who,
                        at_ms: t,
                        event: Event::GainResource {
                            side,
                            what: Resource::Rage.name(),
                            amount: gain,
                            total,
                            accrued: false,
                        },
                    });
                }
            }
            // Contagion: landing one brings the other along.
            let spread = if matches!(target, Target::Enemy) {
                pick(p, foes, me).contagion
            } else {
                0
            };
            for _ in 0..spread {
                // Contagion pairs a curse with its opposite number: heat and
                // cold, stopped and unreliable.
                let other = match kind {
                    CurseKind::Searing => CurseKind::Frost,
                    CurseKind::Frost => CurseKind::Searing,
                    CurseKind::Stun => CurseKind::Misfire,
                    CurseKind::Misfire => CurseKind::Stun,
                };
                let victim = pick(p, foes, me.other(front));
                land_curse(victim, me.other(front), other, StunAim::Unaimed, t, log);
            }
            let on = resolve(target);
            // **Decided before the victim is borrowed, booked after.** The
            // author and the victim are two entries in one vector.
            let permanent = standing_fact_room(pick(p, foes, me));
            let c = pick(p, foes, on);
            let stood = land_curse_for(c, on, kind, StunAim::Unaimed, t, log, permanent);
            if stood {
                pick(p, foes, me).facts_standing += 1;
            }
            // **Cold Stoke: a curse you land is fuel**, and it is booked on
            // the *lander* — the same side `facts_standing` is, for the same
            // reason: a curse's tuning belongs to whoever put it there.
            //
            // `standing` reads a curse that has **stacked** rather than one
            // that cannot expire. `PLAN-M16.md` §12.3 says *permanent*, and a
            // permanent curse is Standing Fact's — which is a different expert,
            // so a knob that only paid beside it would be a point you can only
            // spend as somebody else. A second stack is the same idea and is
            // reachable from this class's own chair.
            let deep = pick(p, foes, on).curses.stacks_of(kind) > 1;
            stoke_on_curse(pick(p, foes, me), deep);
            // A curse is the one thing a watcher counts that nobody activated,
            // and it is watched from both sides: the gear that landed it and
            // the gear wearing it both saw the same event.
            notify_curse_watchers(p, foes, me, t, log);
            notify_curse_watchers(p, foes, me.other(front), t, log);
        }
        Action::StunStrongest { target } => {
            let on = resolve(target);
            let c = pick(p, foes, on);
            land_curse(c, on, CurseKind::Stun, StunAim::Strongest, t, log);
            notify_curse_watchers(p, foes, me, t, log);
            notify_curse_watchers(p, foes, me.other(front), t, log);
        }
        Action::Damage { amount, kind, target } => {
            let on = resolve(target);
            // Next swing goes to the next one along. Done before the hit
            // resolves so a payload that lands twice still spreads.
            if me.side == Side::Player && on.side == Side::Enemy && foes.len() > 1 {
                p.aim = front + 1;
            }
            let pierce = match kind {
                DamageType::Physical => pick(p, foes, on.other(front)).physical_pierce,
                DamageType::Magic => pick(p, foes, on.other(front)).magic_pierce,
            };
            let c = pick(p, foes, on);
            let (absorbed, _) = c.take_typed(amount, kind, pierce);
            let (hp, ar) = (c.health, c.armor);
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Hit {
                    by: on.other(front).side,
                    by_item: owner,
                    damage: amount,
                    absorbed,
                    target_health: hp,
                    target_armor: ar,
                },
            });
        }
        Action::MindDamage { amount, target } => {
            let on = resolve(target);
            let (raw, pierce) = {
                let me = pick(p, foes, me);
                (amount + me.mind_bonus(), me.mind_pierce)
            };
            let c = pick(p, foes, on);
            let dealt = c.take_mind_pierced(raw, pierce);
            let mh = c.max_health;
            if dealt > 0 {
                log.push(LogEntry {
                    who,
                    at_ms: t,
                    event: Event::MindHit { by: on.other(front).side, amount: dealt, target_max_health: mh },
                });
            }
        }
        Action::Gain { what, amount } => {
            let me = pick(p, foes, me);
            let now = me.pool(what) + amount;
            me.set_pool(what, now);
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::GainResource { side, what: what.name(), amount, total: now, accrued: false },
            });
        }
        Action::Drain { what, amount, hurt, target } => {
            let on = resolve(target);
            let c = pick(p, foes, on);
            let have = c.pool(what).max(0);
            // Zero means the lot. Taking more than they hold is not a debt -
            // an empty pool is simply empty.
            let taken = if amount == 0 { have } else { amount.min(have) };
            if taken > 0 {
                let left = have - taken;
                c.set_pool(what, left);
                log.push(LogEntry {
                    who,
                    at_ms: t,
                    event: Event::Drained {
                            on: on.side,
                            what: what.name(),
                            amount: taken,
                            total: left,
                        },
                });
                if hurt > 0 {
                    // Priced off what was actually taken, so a dry pool costs
                    // them nothing and a deep one costs them dearly.
                    let raw = taken * hurt;
                    let pierce = pick(p, foes, on.other(front)).magic_pierce;
                    let c = pick(p, foes, on);
                    let (absorbed, _) = c.take_typed(raw, DamageType::Magic, pierce);
                    let (hp, ar) = (c.health, c.armor);
                    log.push(LogEntry {
                        who,
                        at_ms: t,
                        event: Event::Hit {
                            by: on.other(front).side,
                            by_item: owner,
                            damage: raw,
                            absorbed,
                            target_health: hp,
                            target_armor: ar,
                        },
                    });
                }
            }
        }
        Action::GainMana(n) => {
            let c = pick(p, foes, me);
            c.mana += n;
            let total = c.mana;
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::GainMana { side, amount: n, total, accrued: false } });
        }
        Action::Grow(n) => {
            // Maximum health up, and the new room filled - growing into a gap
            // you then have to heal would make it useless in the fight that is
            // actually happening.
            let c = pick(p, foes, me);
            c.max_health += n;
            c.health += n;
            let total = c.max_health;
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::Grew { side, amount: n, total, paid_armor: 0 } });
        }
        Action::GainArmor(n) => {
            let c = pick(p, foes, me);
            // Consecrate: faith held makes the wall worth more. Gated on
            // actually holding some, so it rewards banking rather than being a
            // flat bonus wearing a name.
            let n = if c.consecrate > 0 && c.pool(Resource::Faith) > 0 {
                n + n * c.consecrate / 100
            } else {
                n
            };
            let n = c.gain_armor(n);
            let total = c.armor;
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::GainArmor { side, amount: n, total } });
        }
        Action::GainEmpowerment(n) => {
            let c = pick(p, foes, me);
            c.empowerment += n;
            let (total, bonus) = (c.empowerment, c.effective_power() - c.power);
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Empowered { side, total, power_bonus: bonus },
            });
        }
        Action::GainShield(n) => {
            let c = pick(p, foes, me);
            c.shield += n;
            let (total, reduction) = (c.shield, c.damage_reduction());
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::Shielded { side, total, reduction } });
        }
        Action::SeeWithTheWrongSense => {
            // Kept as an arm so the enum stays exhaustive, and it does nothing:
            // the trade is `EffectKind::WrongSense`, read off the board at the
            // bell, because it is a standing state rather than something that
            // happens when an item comes round. No piece carries this action.
        }
        Action::GainDread(n) => {
            let c = pick(p, foes, me);
            c.dread += n;
            let (total, bonus) = (c.dread, c.mind_bonus());
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Dreading { side, total, mind_bonus: bonus },
            });
        }
        Action::GainSpellblade(n) => {
            let c = pick(p, foes, me);
            c.spellblade += n;
            let (total, bonus) = (c.spellblade, c.physical_empower());
            log.push(LogEntry {
                who,
                at_ms: t,
                event: Event::Whetted { side, total, power_bonus: bonus },
            });
        }
        Action::GainDeflection(n) => {
            let c = pick(p, foes, me);
            c.deflection += n;
            let (total, reduction) = (c.deflection, c.physical_reduction());
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::Deflecting { side, total, reduction } });
        }
        Action::GainForking(n) => {
            let c = pick(p, foes, me);
            c.forking += n;
            let total = c.forking;
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::Forking { side, total } });
        }
        // ---- the yard's four verbs ------------------------------------
        Action::Shunt { ms } => {
            let Some(idx) = owner else { return };
            let c = pick(p, foes, me);
            let Some(from_name) = c.items.get(idx).map(|i| i.name.clone()) else { return };
            // The slowest neighbour, ties to the lowest index: a second is
            // worth most on the bar that fills slowest, and "slowest" is the
            // whole reason to hand it over rather than keep it.
            let Some(&to) = c.items[idx]
                .adjacent_items
                .iter()
                .filter(|&&j| j != idx && j < c.items.len())
                .max_by_key(|&&j| (c.items[j].cooldown_ms, std::cmp::Reverse(j)))
            else {
                return;
            };
            let (name, cap) = {
                let it = &c.items[to];
                (it.name.clone(), it.cooldown_ms.saturating_sub(TICK_MS))
            };
            let before = c.items[to].progress_ms;
            c.items[to].progress_ms = (before + ms).min(cap);
            // Only what actually landed is owed. The cap means a bar already
            // near the top takes less than was offered, and charging for time
            // that went nowhere would make a shunt a net loss.
            let moved = c.items[to].progress_ms.saturating_sub(before);
            if moved == 0 {
                return;
            }
            c.items[idx].owed_ms += moved;
            log.push(LogEntry {
                who: me.logged_as(front),
                at_ms: t,
                event: Event::Shunted { side, from: from_name, to: name, ms: moved },
            });
        }
        Action::Ballast(n) => {
            let c = pick(p, foes, me);
            let paid = n.min(c.armor.max(0));
            if paid <= 0 {
                return;
            }
            c.armor -= paid;
            c.max_health += paid;
            c.health += paid;
            let total = c.max_health;
            log.push(LogEntry {
                who: me.logged_as(front),
                at_ms: t,
                event: Event::Grew { side, amount: paid, total, paid_armor: paid },
            });
        }
        Action::Derail { window_ms, back_ms } => {
            // The front foe's, always. A `Yourself` derail is refused by
            // `assembly::every_action_is_well_formed`, because there is no
            // reading of it that is not a stun on your own bar.
            let on = me.other(front);
            let c = pick(p, foes, on);
            let Some(i) = c
                .items
                .iter()
                .enumerate()
                .filter(|(_, it)| it.cooldown_ms.saturating_sub(it.progress_ms) <= window_ms)
                .max_by_key(|(i, it)| (it.rating, std::cmp::Reverse(*i)))
                .map(|(i, _)| i)
            else {
                return;
            };
            c.items[i].progress_ms = c.items[i].progress_ms.saturating_sub(back_ms);
            let name = c.items[i].name.clone();
            log.push(LogEntry {
                who: me.logged_as(front),
                at_ms: t,
                event: Event::Derailed { side: on.side, item: name, by_ms: back_ms },
            });
        }
        Action::Accrue { what, pct } => {
            // A fused pool is deliberately fuel for nothing (`piece.rs`), so a
            // proportional income on one would be a second currency at better
            // rates. `assembly::every_action_is_well_formed` keeps it out of
            // the catalogue; this keeps it out of the fight, because a rule
            // that only a lint enforces is a rule a hand-built profile can
            // walk straight through.
            if what.is_fused() {
                return;
            }
            let c = pick(p, foes, me);
            let held = c.pool(what).max(0);
            let gain = held * pct / 100;
            if gain <= 0 {
                return;
            }
            let total = c.pool(what) + gain;
            c.set_pool(what, total);
            log.push(LogEntry {
                who: me.logged_as(front),
                at_ms: t,
                event: if what == Resource::Mana {
                    // Mana is counted through its own event by `settle`
                    // (`run.rs`), so an accrual has to arrive on that one or
                    // the run's books would miss it.
                    Event::GainMana { side, amount: gain, total, accrued: true }
                } else {
                    Event::GainResource { side, what: what.name(), amount: gain, total, accrued: true }
                },
            });
        }
        Action::ReduceCooldown(ms) => {
            let Some(idx) = owner else { return };
            let c = pick(p, foes, me);
            let Some(it) = c.items.get_mut(idx) else { return };
            // Push the bar forward rather than shortening the cooldown, so the
            // effect is "fires sooner once" and cannot stack into a free item.
            it.progress_ms = (it.progress_ms + ms).min(it.cooldown_ms.saturating_sub(1));
            let name = it.name.clone();
            log.push(LogEntry { who: me.logged_as(front), at_ms: t, event: Event::Hastened { side, item: name, by_ms: ms } });
        }
    }
}

// ---------------------------------------------------------------------------
// Alternates: creatures that are not on the ladder.
//
// An alternate stands in for a rung rather than adding one, so choosing to
// fight it does not lengthen the road. The ladder stays fifty long whichever
// way you go.

/// Creatures an event can put in front of you instead of the rung's own.
pub const ALTERNATES: &[MonsterSpec] = &[
    // The thing Nibbalonius will one day swallow, met early and still whole.
    // Armoured to start, regrows what it loses, and does no harm you can heal:
    // every point it takes off you it takes off your maximum.
    MonsterSpec {
        name: "The Dreaming Idiot",
        health: 520,
        strength: 0,
        regen: 2,
        mind_resist: 40,
        physical_resist: 22,
        magic_resist: 22,
        curse_resist: 45,
        attacks: &[],
        gear: &[
            ("Covenant Frame", SlotKind::Helmet, 0, 0, 0),
            ("Warded Plating", SlotKind::Helmet, 3, 0, 0),
            ("Covenant Frame", SlotKind::Helmet, 0, 2, 0),
            ("Bulwark Plating", SlotKind::Helmet, 3, 2, 0),
            ("Covenant Frame", SlotKind::Helmet, 0, 4, 0),
            ("Braced Plating", SlotKind::Helmet, 3, 4, 0),
            ("Hexweave Shroud", SlotKind::Chest, 0, 0, 0),
            ("Seedbed Layer", SlotKind::Chest, 3, 0, 0),
            ("Seedbed Layer", SlotKind::Chest, 3, 1, 0),
            ("Deep Roots Base", SlotKind::Chest, 3, 2, 0),
            ("Seedbed Layer", SlotKind::Chest, 0, 3, 0),
            ("Hexweave Shroud", SlotKind::Chest, 0, 4, 0),
            ("Seedbed Layer", SlotKind::Chest, 3, 4, 0),
            ("Duskweave Material", SlotKind::Gloves, 0, 0, 0),
            ("Empowering Mold", SlotKind::Gloves, 3, 0, 0),
            ("Tithe Ring", SlotKind::Gloves, 4, 0, 0),
            ("Duskweave Material", SlotKind::Gloves, 0, 2, 0),
            ("Empowering Mold", SlotKind::Gloves, 3, 2, 0),
            ("Duskweave Material", SlotKind::Gloves, 0, 4, 0),
            ("Channeling Mold", SlotKind::Gloves, 3, 4, 0),
            ("Ring of Tides", SlotKind::Gloves, 0, 3, 0),
            ("Tithe Ring", SlotKind::Gloves, 2, 3, 0),
            ("Duskweave Material", SlotKind::Greaves, 0, 0, 0),
            ("Standing Start", SlotKind::Greaves, 3, 0, 0),
            ("Mana Ward", SlotKind::Greaves, 2, 1, 0),
            ("Duskweave Material", SlotKind::Greaves, 0, 2, 0),
            ("Striding Mold", SlotKind::Greaves, 2, 3, 0),
            ("Braced Plating", SlotKind::Greaves, 4, 2, 0),
            ("Duskweave Material", SlotKind::Greaves, 0, 4, 0),
            ("Striding Mold", SlotKind::Greaves, 2, 5, 0),
            // One voice. A creature that deals nothing but mind damage has
            // exactly one weapon in it: the orb-and-Unmaking build is the whole
            // of what the catalogue offers that does no other kind of harm.
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Siphon", SlotKind::Weapon, 3, 0, 0),
            ("Siphon", SlotKind::Weapon, 4, 0, 0),
            ("Siphon", SlotKind::Weapon, 5, 0, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 2, 0),
        ],
        gear_offset: 0,
        bounty: 140,
        sprite: MonsterSprite::Idiot,
        rank: Rank::Boss,
        drops: &["The Idiot's Gift"],
        items: &[2, 2, 2, 3, 2, 2, 3, 2, 4, 3, 3, 2, 5],
        enchs: &[],
    },
    // ---- Bunko's Cavern, pp. 84-85 ------------------------------------------
    //
    // Floor one: the Head Cork Priest of Corrqk's Cavern, reciting the '62
    // Anticipations to a room of workers kneeling on a floor that cuts.
    MonsterSpec {
        name: "The Reciter",
        health: 430,
        strength: 14,
        regen: 2,
        mind_resist: 25,
        physical_resist: 14,
        magic_resist: 20,
        curse_resist: 30,
        attacks: &[],
        gear: &[
            ("Covenant Frame", SlotKind::Helmet, 0, 0, 0),
            ("Braced Plating", SlotKind::Helmet, 3, 0, 0),
            ("Covenant Frame", SlotKind::Helmet, 0, 2, 0),
            ("Mana Ward", SlotKind::Helmet, 3, 2, 0),
            ("Vigil Crest", SlotKind::Helmet, 5, 0, 0),
            ("Bloodbank Base", SlotKind::Chest, 0, 0, 0),
            ("Split Weave", SlotKind::Chest, 2, 0, 0),
            ("Hexweave Shroud", SlotKind::Chest, 2, 1, 0),
            ("Seedbed Layer", SlotKind::Chest, 0, 4, 0),
            ("Duskweave Material", SlotKind::Gloves, 0, 0, 0),
            ("Empowering Mold", SlotKind::Gloves, 3, 0, 0),
            ("Ring of Tides", SlotKind::Gloves, 4, 0, 0),
            ("Ring of Tides", SlotKind::Gloves, 5, 0, 0),
            ("Warmed Material", SlotKind::Gloves, 0, 2, 0),
            ("Empowering Mold", SlotKind::Gloves, 2, 1, 0),
            ("Ring of Tides", SlotKind::Gloves, 0, 1, 0),
            ("Ring of Tides", SlotKind::Gloves, 4, 2, 0),
            ("Duskweave Material", SlotKind::Greaves, 0, 0, 0),
            ("Standing Start", SlotKind::Greaves, 3, 0, 0),
            ("Warmed Material", SlotKind::Greaves, 2, 1, 0),
            ("Striding Mold", SlotKind::Greaves, 4, 1, 0),
            ("Braced Plating", SlotKind::Greaves, 0, 2, 0),
            ("Ambusher's Grip", SlotKind::Weapon, 0, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 1, 0, 0),
            ("Empowering Focus", SlotKind::Weapon, 3, 0, 0),
            ("Empowering Focus", SlotKind::Weapon, 4, 1, 0),
            ("Ambusher's Grip", SlotKind::Weapon, 2, 2, 0),
            ("Cursed Blade", SlotKind::Weapon, 3, 2, 0),
            ("Cursed Blade", SlotKind::Weapon, 0, 3, 0),
            ("Bulwark Bead", SlotKind::Weapon, 1, 3, 0),
            ("Bulwark Bead", SlotKind::Weapon, 5, 3, 0),
        ],
        gear_offset: 0,
        bounty: 96,
        sprite: MonsterSprite::Abbot,
        rank: Rank::Mini,
        drops: &["Bulwark Bead"],
        items: &[2, 3, 2, 2, 4, 4, 2, 3, 4, 5],
        enchs: &[],
    },
    // Floor two: the train the dissenters were loaded onto, still running.
    MonsterSpec {
        name: "The Long Haul",
        health: 620,
        strength: 22,
        regen: 0,
        mind_resist: 10,
        physical_resist: 26,
        magic_resist: 10,
        curse_resist: 20,
        attacks: &[],
        gear: &[
            ("Covenant Frame", SlotKind::Helmet, 0, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 3, 0, 0),
            ("Covenant Frame", SlotKind::Helmet, 0, 2, 0),
            ("Mana Ward", SlotKind::Helmet, 3, 2, 0),
            ("Third Eye", SlotKind::Helmet, 0, 1, 0),
            ("Hexweave Shroud", SlotKind::Chest, 0, 0, 0),
            ("Aether Layer", SlotKind::Chest, 3, 0, 0),
            ("Bloodbank Base", SlotKind::Chest, 3, 2, 0),
            ("Split Weave", SlotKind::Chest, 0, 3, 0),
            ("Duskweave Material", SlotKind::Gloves, 0, 0, 0),
            ("Empowering Mold", SlotKind::Gloves, 3, 0, 0),
            ("Ring of Tides", SlotKind::Gloves, 4, 0, 0),
            ("Ring of Tides", SlotKind::Gloves, 5, 0, 0),
            ("Warmed Material", SlotKind::Gloves, 0, 2, 0),
            ("Rending Mold", SlotKind::Gloves, 2, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 0, 1, 0),
            ("Ironthread Material", SlotKind::Greaves, 0, 0, 0),
            ("Standing Start", SlotKind::Greaves, 3, 0, 0),
            ("Duskweave Material", SlotKind::Greaves, 2, 1, 0),
            ("Standing Start", SlotKind::Greaves, 0, 2, 0),
            ("Ambusher's Grip", SlotKind::Weapon, 0, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 1, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 3, 0, 0),
            ("Bulwark Bead", SlotKind::Weapon, 2, 0, 0),
            ("Ambusher's Grip", SlotKind::Weapon, 5, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 4, 2, 0),
            ("Grimoire Rack", SlotKind::Weapon, 3, 3, 0),
        ],
        gear_offset: 0,
        bounty: 104,
        sprite: MonsterSprite::Parliament,
        rank: Rank::Mini,
        drops: &["Grimoire Rack"],
        items: &[2, 3, 2, 2, 4, 3, 2, 2, 4, 3],
        enchs: &[],
    },
    // Floor three: the old gods, watching in horror as he ascends.
    MonsterSpec {
        name: "The Watchers",
        health: 880,
        strength: 20,
        regen: 6,
        mind_resist: 45,
        physical_resist: 24,
        magic_resist: 34,
        curse_resist: 45,
        attacks: &[],
        gear: &[
            ("Covenant Frame", SlotKind::Helmet, 0, 0, 0),
            ("Broken Crown", SlotKind::Helmet, 0, 1, 0),
            ("Third Eye", SlotKind::Helmet, 3, 0, 0),
            ("Covenant Frame", SlotKind::Helmet, 1, 3, 0),
            ("Braced Plating", SlotKind::Helmet, 0, 4, 0),
            ("Third Eye", SlotKind::Helmet, 3, 4, 0),
            ("Covenant Frame", SlotKind::Helmet, 2, 5, 0),
            ("Mana Ward", SlotKind::Helmet, 0, 6, 0),
            ("Mana Ward", SlotKind::Helmet, 4, 3, 1),
            ("Bloodbank Base", SlotKind::Chest, 0, 0, 0),
            ("Seedbed Layer", SlotKind::Chest, 2, 0, 0),
            ("Bloodbank Base", SlotKind::Chest, 2, 1, 0),
            ("Seedbed Layer", SlotKind::Chest, 0, 3, 0),
            ("Bloodbank Base", SlotKind::Chest, 4, 1, 0),
            ("Seedbed Layer", SlotKind::Chest, 3, 3, 0),
            ("Aether Layer", SlotKind::Chest, 2, 4, 0),
            ("Warmed Material", SlotKind::Gloves, 0, 0, 0),
            ("Empowering Mold", SlotKind::Gloves, 2, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 3, 0, 0),
            ("Warmed Material", SlotKind::Gloves, 4, 1, 0),
            ("Hexer's Reckoning", SlotKind::Gloves, 2, 2, 0),
            ("Warmed Material", SlotKind::Gloves, 0, 2, 0),
            ("Empowering Mold", SlotKind::Gloves, 0, 4, 0),
            ("Ring of Tides", SlotKind::Gloves, 1, 4, 0),
            ("Warding Ring", SlotKind::Gloves, 2, 4, 0),
            ("Ironthread Material", SlotKind::Greaves, 0, 0, 0),
            ("Striding Mold", SlotKind::Greaves, 3, 0, 0),
            ("Ironthread Material", SlotKind::Greaves, 0, 2, 0),
            ("Striding Mold", SlotKind::Greaves, 2, 1, 0),
            ("Warmed Material", SlotKind::Greaves, 4, 2, 0),
            ("Standing Start", SlotKind::Greaves, 2, 4, 0),
            ("Broken Crown", SlotKind::Greaves, 0, 5, 0),
            ("Balanced Grip", SlotKind::Weapon, 0, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 1, 0, 0),
            ("Whetstone", SlotKind::Weapon, 2, 0, 0),
            ("Gravebound Haft", SlotKind::Weapon, 3, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 3, 2, 0),
            ("Balanced Grip", SlotKind::Weapon, 5, 0, 0),
            ("Cursed Blade", SlotKind::Weapon, 4, 4, 2),
            ("Balance Weight", SlotKind::Weapon, 2, 5, 0),
        ],
        gear_offset: 0,
        bounty: 150,
        sprite: MonsterSprite::Choir,
        rank: Rank::Boss,
        drops: &["The Split Wisdom"],
        items: &[3, 3, 3, 2, 2, 3, 3, 2, 4, 2, 2, 3, 3, 2, 3],
        enchs: &[],
    },

    // --------------------------------------------------- the Unwinding
    //
    // Frames. Name, health, band and nothing on. A creature that exists
    // before its board does is not a placeholder, it is the order the mission
    // is built in: content lands as frames, all of it, and then every board is
    // authored by hand in one pass against a settled rating curve - because a
    // board authored before the curve under it stops moving is a board that
    // will be authored twice.
    //
    // `CREVICE` was an empty list of specs and the four above stood beside the
    // road for a long time without anybody saying how hard they were meant to
    // be, so this is the pattern the repo already had rather than a new one.
    // `bestiary::FRAMES` says what each is for and what band it packs to, and
    // `no_frame_ships_without_a_board` is red until every one of them is
    // dressed.
    MonsterSpec {
        name: "DOORKEEP",
        health: 900,
        strength: 10,
        regen: 2,
        mind_resist: 30,
        physical_resist: 10,
        magic_resist: 10,
        curse_resist: 40,
        attacks: &[],
        gear: &[
            ("Apprentice's Primer", SlotKind::Weapon, 3, 5, 0),
            ("Hollow Lance", SlotKind::Weapon, 3, 1, 0),
            ("Deepwater Ink", SlotKind::Weapon, 0, 6, 1),
            ("Forking Bead", SlotKind::Weapon, 5, 6, 0),
            ("Plate Layer", SlotKind::Chest, 1, 3, 0),
            ("Hollow Weave", SlotKind::Chest, 1, 4, 0),
            ("Ribbed Base", SlotKind::Chest, 1, 1, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 4, 0),
            ("Iron Plating", SlotKind::Helmet, 2, 2, 0),
        ],
        gear_offset: 0,
        bounty: 170,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &["Iron Plating"],
        items: &[4, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE STAIR THAT LISTENS",
        health: 1_000,
        strength: 10,
        regen: 2,
        mind_resist: 35,
        physical_resist: 12,
        magic_resist: 12,
        curse_resist: 45,
        attacks: &[],
        gear: &[
            ("Stormcaught Frame", SlotKind::Helmet, 2, 3, 0),
            ("Lonely Plating", SlotKind::Helmet, 3, 5, 0),
            ("Forked Crest", SlotKind::Helmet, 3, 6, 0),
            ("Hexbolt", SlotKind::Weapon, 0, 2, 0),
            ("Manaflay", SlotKind::Weapon, 1, 7, 0),
            ("Zealot's Haft", SlotKind::Weapon, 0, 5, 0),
            ("Wildgrowth", SlotKind::Weapon, 3, 1, 0),
            ("Stray Orb", SlotKind::Weapon, 2, 4, 0),
            ("Pilgrim Alignment", SlotKind::Weapon, 4, 3, 0),
            ("Shatterbolt", SlotKind::Weapon, 1, 0, 0),
            ("Waxed Material", SlotKind::Greaves, 1, 4, 0),
            ("Ambush Mold", SlotKind::Greaves, 2, 3, 0),
            ("Braced Plating", SlotKind::Greaves, 2, 1, 0),
            ("Storm Signet", SlotKind::Gloves, 4, 3, 0),
            ("Ironhide Wrap", SlotKind::Gloves, 2, 2, 0),
            ("Deft Mold", SlotKind::Gloves, 2, 1, 0),
            ("Rootbound Material", SlotKind::Gloves, 2, 5, 0),
            ("Vicegrip Mold", SlotKind::Gloves, 2, 4, 0),
        ],
        gear_offset: 0,
        bounty: 180,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &["Vicegrip Mold"],
        items: &[3, 3, 4, 3, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE LAST LANDING",
        health: 2007,
        strength: 54,
        regen: 4,
        mind_resist: 40,
        physical_resist: 14,
        magic_resist: 14,
        curse_resist: 50,
        attacks: &[],
        gear: &[
            ("Leaden Tome", SlotKind::Weapon, 0, 0, 0),
            ("Kingsblood Ink", SlotKind::Weapon, 3, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 2, 0),
            ("Oathstone Bead", SlotKind::Weapon, 0, 3, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Deft Mold", SlotKind::Gloves, 2, 0, 0),
            ("Unshod Signet", SlotKind::Gloves, 4, 0, 0),
            ("Warding Ring", SlotKind::Gloves, 5, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 2, 1, 0),
            ("Deft Mold", SlotKind::Gloves, 4, 1, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 0, 0),
            ("Worldstrider Sole", SlotKind::Greaves, 2, 0, 0),
            ("Overflow Plate", SlotKind::Greaves, 4, 1, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 2, 0),
            ("Widow's Sole", SlotKind::Greaves, 2, 1, 0),
            ("Broken Crown", SlotKind::Greaves, 0, 4, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Witch's Hat", SlotKind::Helmet, 0, 2, 2),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("Mana Ward", SlotKind::Helmet, 4, 3, 1),
            ("Coven Crest", SlotKind::Helmet, 5, 1, 0),
        ],
        gear_offset: 0,
        bounty: 200,
        sprite: MonsterSprite::Idol,
        rank: Rank::Mini,
        drops: &["Coven Crest"],
        items: &[4, 4, 2, 3, 3, 2, 4],
        enchs: &[],
    },
    // The Herald is two of them at once, which is the first party fight in the
    // game outside the casino - your shadow, and what your shadow carries.
    MonsterSpec {
        name: "THE SHADOW",
        health: 3568,
        strength: 89,
        regen: 7,
        mind_resist: 45,
        physical_resist: 18,
        magic_resist: 18,
        curse_resist: 55,
        attacks: &[],
        gear: &[
            ("Reliquary Sole", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Blightfinger", SlotKind::Gloves, 4, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 0, 1),
            ("Reliquary Sole", SlotKind::Gloves, 3, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 1, 2, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 0, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 2, 1),
            ("Witch's Stilts", SlotKind::Gloves, 2, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 0, 3, 3),
            ("Reliquary Sole", SlotKind::Gloves, 3, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 1, 5, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 5, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 5, 1),
            ("Reliquary Sole", SlotKind::Gloves, 2, 6, 0),
            ("Flaying Mold", SlotKind::Gloves, 4, 5, 2),
            ("Deepdraught Ring", SlotKind::Gloves, 1, 7, 0),
            ("Seal of the Deep", SlotKind::Gloves, 4, 7, 0),
            ("Antechamber Crown", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 0, 2, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 4, 0),
            ("Third Eye", SlotKind::Helmet, 5, 3, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
        ],
        gear_offset: 0,
        // A fight an event arranges pays nothing - the reward is what it
        // hands over - but a creature still says what it would be worth, the
        // way everything else on and beside this road does.
        bounty: 361,
        sprite: MonsterSprite::Idol,
        rank: Rank::Mini,
        drops: &["Overflow Plate"],
        items: &[4, 4, 2, 4, 4, 4, 4, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE LANTERN",
        health: 2470,
        strength: 62,
        regen: 5,
        mind_resist: 0,
        physical_resist: 8,
        magic_resist: 8,
        curse_resist: 10,
        attacks: &[],
        gear: &[
            ("Reliquary Sole", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 4, 0, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 0, 1),
            ("Mage's Wrapping", SlotKind::Gloves, 3, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 1, 2, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 0, 2, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 2, 1),
            ("Reliquary Sole", SlotKind::Gloves, 2, 3, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 3, 3),
            ("Reliquary Sole", SlotKind::Gloves, 4, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 5, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 4, 3, 0),
            ("Blightfinger", SlotKind::Gloves, 1, 5, 0),
            ("Reliquary Sole", SlotKind::Gloves, 0, 6, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 6, 2),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 4, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 6, 0),
            ("Scrying Lens", SlotKind::Helmet, 3, 6, 0),
            ("Martyr's Crest", SlotKind::Helmet, 3, 7, 0),
        ],
        gear_offset: 0,
        bounty: 180,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &["Martyr's Crest"],
        items: &[4, 4, 2, 4, 2, 3, 3, 3, 3],
        enchs: &[],
    },
    // THE UNDER-MINE, two floors of Wardens who dug in and stayed.
    MonsterSpec {
        name: "THE DIGGERS",
        health: 2512,
        strength: 65,
        regen: 6,
        mind_resist: 10,
        physical_resist: 26,
        magic_resist: 20,
        curse_resist: 30,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Ember Alignment", SlotKind::Weapon, 4, 2, 2),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 4, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 6, 1),
        ],
        gear_offset: 0,
        bounty: 251,
        sprite: MonsterSprite::Golem,
        rank: Rank::Ordinary,
        drops: &["The Empty Crown"],
        items: &[5, 3, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "WHAT THE SEAM HID",
        health: 3106,
        strength: 80,
        regen: 7,
        mind_resist: 15,
        physical_resist: 30,
        magic_resist: 24,
        curse_resist: 40,
        attacks: &[],
        gear: &[
            ("Fateglass Orb", SlotKind::Weapon, 0, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 2, 0, 0),
            ("Resonant Chord", SlotKind::Weapon, 4, 0, 3),
            ("Emberburst", SlotKind::Weapon, 1, 1, 0),
            ("Pilgrim Alignment", SlotKind::Weapon, 0, 2, 0),
            ("Buttressed Frame", SlotKind::Helmet, 0, 0, 0),
            ("Visor of Focus", SlotKind::Helmet, 3, 0, 0),
            ("Buttressed Frame", SlotKind::Helmet, 2, 1, 0),
            ("Visor of Focus", SlotKind::Helmet, 0, 2, 0),
            ("Crown of the Deep", SlotKind::Helmet, 4, 1, 1),
            ("Reliquary Frame of Nine", SlotKind::Helmet, 0, 3, 0),
            ("Visor of Focus", SlotKind::Helmet, 3, 3, 1),
            ("Bloomcap", SlotKind::Helmet, 4, 3, 2),
            ("Buttressed Frame", SlotKind::Helmet, 0, 4, 3),
            ("Visor of Focus", SlotKind::Helmet, 2, 4, 1),
            ("Bloomcap", SlotKind::Helmet, 3, 5, 1),
            ("Buttressed Frame", SlotKind::Helmet, 0, 6, 2),
            ("Visor of Focus", SlotKind::Helmet, 3, 7, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 0, 1),
            ("Channeling Mold", SlotKind::Gloves, 3, 0, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 1, 3),
            ("Channeling Mold", SlotKind::Gloves, 3, 1, 2),
            ("Witch's Stilts", SlotKind::Gloves, 3, 2, 3),
            ("Channeling Mold", SlotKind::Gloves, 1, 3, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 3, 0),
            ("Channeling Mold", SlotKind::Gloves, 2, 4, 0),
            ("Witch's Stilts", SlotKind::Gloves, 3, 4, 3),
            ("Empowering Mold", SlotKind::Gloves, 2, 6, 1),
            ("Siphon Ring", SlotKind::Gloves, 4, 4, 0),
            ("Ring of Tides", SlotKind::Gloves, 1, 6, 0),
        ],
        gear_offset: 0,
        bounty: 262,
        sprite: MonsterSprite::Golem,
        rank: Rank::Mini,
        drops: &["Ring of Tides"],
        items: &[5, 2, 3, 3, 3, 2, 2, 2, 2, 2, 4],
        enchs: &[],
    },
    // THE UNDERTOW, where the water sets the pace.
    MonsterSpec {
        name: "THE CURRENT",
        health: 2512,
        strength: 65,
        regen: 6,
        mind_resist: 8,
        physical_resist: 18,
        magic_resist: 22,
        curse_resist: 45,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 4, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
        ],
        gear_offset: 0,
        bounty: 251,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &["Overflow Plate"],
        items: &[5, 3, 3, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE THING ON THE HOOK",
        health: 3306,
        strength: 87,
        regen: 7,
        mind_resist: 12,
        physical_resist: 20,
        magic_resist: 26,
        curse_resist: 55,
        attacks: &[],
        gear: &[
            ("Fateglass Orb", SlotKind::Weapon, 0, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 2, 0, 0),
            ("Shatterbolt", SlotKind::Weapon, 5, 0, 1),
            ("Emberburst", SlotKind::Weapon, 1, 1, 0),
            ("Pilgrim Alignment", SlotKind::Weapon, 0, 2, 0),
            ("Buttressed Frame", SlotKind::Helmet, 0, 0, 0),
            ("Deadweight Plating", SlotKind::Helmet, 3, 0, 1),
            ("Bloomcap", SlotKind::Helmet, 2, 1, 3),
            ("Crown of the Deep", SlotKind::Helmet, 0, 1, 3),
            ("Reliquary Frame of Nine", SlotKind::Helmet, 3, 1, 2),
            ("Visor of Focus", SlotKind::Helmet, 1, 3, 0),
            ("Bloomcap", SlotKind::Helmet, 4, 3, 3),
            ("Crown of the Deep", SlotKind::Helmet, 0, 4, 0),
            ("Buttressed Frame", SlotKind::Helmet, 2, 4, 2),
            ("Visor of Focus", SlotKind::Helmet, 5, 4, 1),
            ("Deadweight Plating", SlotKind::Helmet, 0, 6, 1),
            ("Crown of the Deep", SlotKind::Helmet, 2, 6, 2),
            ("Witch's Stilts", SlotKind::Gloves, 0, 0, 1),
            ("Hexer's Mold", SlotKind::Gloves, 3, 0, 3),
            ("Blightfinger", SlotKind::Gloves, 5, 0, 0),
            ("Blightfinger", SlotKind::Gloves, 1, 1, 0),
            ("Spun Material", SlotKind::Gloves, 4, 1, 1),
            ("Channeling Mold", SlotKind::Gloves, 2, 1, 3),
            ("Witch's Stilts", SlotKind::Gloves, 0, 2, 2),
            ("Channeling Mold", SlotKind::Gloves, 2, 3, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 3, 0),
            ("Channeling Mold", SlotKind::Gloves, 2, 4, 2),
            ("Witch's Stilts", SlotKind::Gloves, 4, 3, 2),
            ("Channeling Mold", SlotKind::Gloves, 3, 5, 2),
            ("Mage's Sandals", SlotKind::Gloves, 0, 6, 0),
            ("Channeling Mold", SlotKind::Gloves, 1, 6, 2),
        ],
        gear_offset: 0,
        bounty: 273,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Mini,
        drops: &["Channeling Mold"],
        items: &[5, 4, 4, 4, 4, 2, 2, 2, 2, 2],
        enchs: &[],
    },
    // DEN RIVALS, which is exactly what the exhibit promised.
    MonsterSpec {
        name: "THE DEN MOUTH",
        health: 3245,
        strength: 85,
        regen: 5,
        mind_resist: 0,
        physical_resist: 16,
        magic_resist: 10,
        curse_resist: 10,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Kingsbane", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 2, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 4, 0),
            ("Watchful Crest", SlotKind::Helmet, 5, 2, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 4, 1),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Rat,
        rank: Rank::Ordinary,
        drops: &["Martyr's Crest"],
        items: &[5, 4, 4, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE THOUSANDTH BEAR",
        health: 2140,
        strength: 56,
        regen: 7,
        mind_resist: 0,
        physical_resist: 22,
        magic_resist: 12,
        curse_resist: 15,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Anvil Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Reliquary Sole", SlotKind::Gloves, 4, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 1, 2),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 2, 0),
            ("Twinning Mold", SlotKind::Gloves, 2, 3, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 4, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 4, 1),
            ("Deepdraught Ring", SlotKind::Gloves, 2, 5, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 6, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 2, 6, 0),
            ("Flaying Mold", SlotKind::Gloves, 4, 5, 3),
        ],
        gear_offset: 0,
        bounty: 242,
        sprite: MonsterSprite::Rat,
        rank: Rank::Mini,
        drops: &["Flaying Mold"],
        items: &[4, 4, 2, 4, 2, 2, 2, 4, 2, 2],
        enchs: &[],
    },
    // WUMPUS WORLD. Something in the dark already knows your footsteps.
    MonsterSpec {
        name: "DARK FLOOR",
        health: 3244,
        strength: 84,
        regen: 5,
        mind_resist: 0,
        physical_resist: 6,
        magic_resist: 6,
        curse_resist: 5,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Witch's Hat", SlotKind::Helmet, 0, 0, 2),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 4, 2, 1),
            ("Overflow Plate", SlotKind::Helmet, 2, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 3, 0),
            ("The Empty Crown", SlotKind::Helmet, 0, 5, 0),
            ("Stonewall Frame", SlotKind::Helmet, 4, 5, 1),
            ("Warding Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 6, 0),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &["Consecrated Plating"],
        items: &[5, 4, 4, 3],
        enchs: &[],
    },
    MonsterSpec {
        name: "THE WUMPUS",
        health: 748,
        strength: 19,
        regen: 7,
        mind_resist: 20,
        physical_resist: 20,
        magic_resist: 18,
        curse_resist: 35,
        attacks: &[],
        gear: &[
            ("Orb of the Nine", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Starfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 2, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 3, 1),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 0, 0),
            ("Reliquary Sole", SlotKind::Gloves, 4, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 1, 2),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 3, 0),
            ("Unshod Signet", SlotKind::Gloves, 4, 3, 0),
            ("Deepdraught Ring", SlotKind::Gloves, 4, 2, 0),
            ("Titan's Grip", SlotKind::Gloves, 3, 4, 0),
            ("Gripping Mold", SlotKind::Gloves, 1, 4, 3),
            ("Unshod Signet", SlotKind::Gloves, 5, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 4, 1),
            ("Reliquary Sole", SlotKind::Gloves, 0, 6, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 6, 0),
        ],
        gear_offset: 0,
        bounty: 242,
        sprite: MonsterSprite::Rat,
        rank: Rank::Mini,
        drops: &["Flaying Mold"],
        items: &[5, 2, 3, 4, 2, 2, 4, 4, 2],
        enchs: &[],
    },
    // The birds. Annoying before deadly, which is the whole of a swarm: no
    // one of them is the problem and the aim moving along is.
    MonsterSpec {
        name: "THE FLOCK",
        health: 1298,
        strength: 35,
        regen: 3,
        mind_resist: 0,
        physical_resist: 4,
        magic_resist: 4,
        curse_resist: 5,
        attacks: &[],
        gear: &[
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 0, 0),
            ("Zealot's Sole", SlotKind::Greaves, 2, 0, 1),
            ("Overflow Plate", SlotKind::Greaves, 2, 1, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 4, 1, 0),
            ("Pilgrim Sole", SlotKind::Greaves, 3, 3, 0),
            ("Overflow Plate", SlotKind::Greaves, 1, 3, 0),
            ("Witch's Claw", SlotKind::Greaves, 0, 2, 0),
            ("Widow's Sole", SlotKind::Greaves, 0, 5, 1),
            ("Tallykeeper's Weave", SlotKind::Greaves, 2, 5, 0),
            ("Trailworn Sole", SlotKind::Greaves, 4, 4, 3),
            ("Overflow Plate", SlotKind::Greaves, 0, 6, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 4, 0, 0),
            ("Deft Mold", SlotKind::Gloves, 3, 1, 1),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 2, 0),
            ("Deft Mold", SlotKind::Gloves, 2, 2, 1),
            ("Tallykeeper's Weave", SlotKind::Gloves, 4, 2, 0),
            ("Deft Mold", SlotKind::Gloves, 3, 3, 1),
            ("Witch's Stilts", SlotKind::Gloves, 0, 4, 1),
            ("Deft Mold", SlotKind::Gloves, 1, 5, 0),
            ("Unshod Signet", SlotKind::Gloves, 3, 5, 0),
            ("Warding Ring", SlotKind::Gloves, 4, 5, 0),
        ],
        gear_offset: 0,
        bounty: 188,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &["Warding Ring"],
        items: &[3, 3, 2, 3, 2, 2, 2, 2, 4],
        enchs: &[],
    },
    // Rung fifty-one, and the only creature in the game that is not on the
    // road until a run has earned the road twice: the chain finished and the
    // man at the top put down.
    //
    // A frame like the rest of them, and the last one to be packed. Its band
    // is 51, which is off the end of a curve that stops at fifty, and its
    // target is 16-29 seconds at Medium - the band with its top edge clipped
    // clear of sudden death, because a boss decided by the clock is not a boss
    // decided by the board. See RECONCILIATION II #17.
    MonsterSpec {
        name: "THE UNWOUND",
        health: 15_000,
        strength: 345,
        regen: 50,
        mind_resist: 40,
        physical_resist: 30,
        magic_resist: 30,
        curse_resist: 60,
        attacks: &[],
        gear: &[
            ("Ash Haft", SlotKind::Weapon, 0, 0, 1),
            ("Bronze Fang", SlotKind::Weapon, 3, 0, 1),
            ("Cursed Blade", SlotKind::Weapon, 0, 1, 1),
            ("Ratchet Cog", SlotKind::Weapon, 5, 0, 0),
            ("Flywheel Cog", SlotKind::Weapon, 4, 1, 0),
            ("Wellspring Base", SlotKind::Chest, 0, 0, 0),
            ("Sigil Layer", SlotKind::Chest, 3, 0, 0),
            ("Woven Underlayer", SlotKind::Chest, 0, 1, 0),
            ("Wildfire Layer", SlotKind::Chest, 0, 2, 0),
            ("Bloodbank Base", SlotKind::Chest, 4, 1, 0),
            ("Sigil Layer", SlotKind::Chest, 2, 3, 0),
            ("Woven Underlayer", SlotKind::Chest, 0, 4, 0),
            ("Wildfire Layer", SlotKind::Chest, 5, 3, 1),
            ("Bloodbank Base", SlotKind::Chest, 0, 5, 0),
            ("Sigil Layer", SlotKind::Chest, 2, 5, 0),
            ("Runed Material", SlotKind::Gloves, 0, 0, 0),
            ("Wrathful Talons", SlotKind::Gloves, 2, 0, 3),
            ("Runed Material", SlotKind::Gloves, 4, 0, 0),
            ("Wrathful Talons", SlotKind::Gloves, 2, 1, 1),
            ("Siphon Ring", SlotKind::Gloves, 1, 2, 0),
            ("Emberloop", SlotKind::Gloves, 0, 2, 0),
            ("Runed Material", SlotKind::Gloves, 4, 2, 0),
            ("Wrathful Talons", SlotKind::Gloves, 2, 3, 3),
            ("Oathring", SlotKind::Gloves, 1, 3, 0),
            ("Emberloop", SlotKind::Gloves, 0, 3, 0),
            ("Runed Material", SlotKind::Greaves, 0, 0, 0),
            ("Echo Sole", SlotKind::Greaves, 2, 0, 0),
            ("Sprawling Handwrap", SlotKind::Greaves, 0, 1, 0),
            ("Echo Sole", SlotKind::Greaves, 3, 1, 0),
            ("Iron Plating", SlotKind::Greaves, 1, 5, 0),
            ("Runed Material", SlotKind::Greaves, 4, 3, 0),
            ("Echo Sole", SlotKind::Greaves, 5, 5, 1),
            ("Tin Plating", SlotKind::Greaves, 4, 6, 1),
            ("Helm of Blades", SlotKind::Helmet, 0, 0, 0),
            ("Iron Plating", SlotKind::Helmet, 3, 0, 0),
            ("Harvest Crest", SlotKind::Helmet, 1, 1, 0),
            ("Bronze Frame", SlotKind::Helmet, 2, 2, 0),
            ("Iron Plating", SlotKind::Helmet, 4, 2, 1),
            ("Layered Plating", SlotKind::Helmet, 0, 3, 0),
            ("Doorward Frame", SlotKind::Helmet, 0, 4, 0),
            ("Iron Plating", SlotKind::Helmet, 1, 5, 0),
            ("Layered Plating", SlotKind::Helmet, 4, 5, 0),
            ("Harvest Crest", SlotKind::Helmet, 0, 6, 0),
            ("Archmage's Primer", SlotKind::Weapon, 0, 4, 0),
            ("Voidwritten Ink", SlotKind::Weapon, 2, 4, 0),
            ("Last Rite", SlotKind::Weapon, 4, 4, 2),
            ("Balance Weight", SlotKind::Weapon, 0, 6, 0),
            ("Rending Mold", SlotKind::Gloves, 4, 6, 0),
            ("Bulwark Material", SlotKind::Gloves, 3, 4, 0),
            ("Hermit's Band", SlotKind::Gloves, 3, 6, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 5, 0),
            ("Steel Material", SlotKind::Gloves, 0, 5, 0),
        ],
        gear_offset: 0,
        bounty: 600,
        sprite: MonsterSprite::Idol,
        rank: Rank::Boss,
        drops: &["Harvest Crest"],
        items: &[5, 4, 4, 2, 2, 4, 4, 2, 3, 3, 3, 3, 4, 4, 3, 2],
        enchs: &[],
    },

    // ---- THE SWITCHYARD, nine floors ------------------------------------
    //
    // Undressed on purpose. Phase 2 lands creatures as *frames* - a name, a
    // band, a theme and the stats of the ladder creature standing at that
    // band - and Phase 4 packs their boards by hand. `bestiary::unpacked()`
    // is the count of what is left and the frame lint is red until it is
    // zero, which is what the lint is for.
    //
    // Stats are the ladder's at each floor's entry band, per
    // `post-unwinding.md` §3.11: THE SHUNTER takes Obsidian Colossus's
    // (band 27), floors 1 and 5 Null Sentinel's (28), floors 2 and 6
    // Silence's (29), and the four buffer stops Weeping Idol's (30). Four
    // fights down the yard pay about 840 gold at a rung where a run has
    // earned roughly 2,100, which is a reason to go down and not a jackpot.
    //
    // `rank: Ordinary` and `drops: &[]` for all nine: the dungeon-victory arm
    // never reads `drops` (A0), and a drop list nobody can drop is dead
    // content. What the yard pays, its buffer stops pay through `Floor::also`.
    // The turntable's own engine, and it keeps the turntable's time. Warden at
    // band 27: it makes you pay for the yard being slow, which is the first
    // thing the yard has to teach.
    MonsterSpec {
        name: "THE SHUNTER",
        health: 2490,
        strength: 67,
        regen: 6,
        mind_resist: 59,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 59,
        attacks: &[],
        gear: &[
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 0, 0),
            ("Widow's Sole", SlotKind::Greaves, 2, 0, 1),
            ("Overflow Plate", SlotKind::Greaves, 4, 0, 0),
            ("Rootwoven Material", SlotKind::Greaves, 0, 2, 0),
            ("Widow's Sole", SlotKind::Greaves, 2, 1, 1),
            ("Overflow Plate", SlotKind::Greaves, 3, 2, 0),
            ("Tallykeeper's Weave", SlotKind::Greaves, 0, 3, 0),
            ("Widow's Sole", SlotKind::Greaves, 2, 3, 0),
            ("Consecrated Plating", SlotKind::Greaves, 3, 4, 0),
            ("Witch's Stilts", SlotKind::Greaves, 0, 5, 1),
            ("Sapling Mold", SlotKind::Greaves, 1, 6, 1),
            ("Overflow Plate", SlotKind::Greaves, 3, 6, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 0, 0, 0),
            ("Gripping Mold", SlotKind::Gloves, 2, 0, 0),
            ("Unshod Signet", SlotKind::Gloves, 4, 0, 0),
            ("Warding Ring", SlotKind::Gloves, 5, 0, 0),
            ("Tallykeeper's Weave", SlotKind::Gloves, 3, 1, 0),
            ("Deft Mold", SlotKind::Gloves, 5, 1, 1),
            ("Reliquary Sole", SlotKind::Gloves, 0, 2, 0),
            ("Deft Mold", SlotKind::Gloves, 2, 2, 1),
            ("Unshod Signet", SlotKind::Gloves, 3, 3, 0),
            ("Warding Ring", SlotKind::Gloves, 4, 3, 0),
            ("Witch's Claw", SlotKind::Gloves, 0, 4, 1),
            ("Featherweight Mold", SlotKind::Gloves, 3, 4, 0),
        ],
        gear_offset: 0,
        bounty: 197,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[3, 3, 3, 3, 4, 2, 4, 2],
        enchs: &[],
    },
    // Many small blows, the rail put back as fast as it is lifted.
    MonsterSpec {
        name: "THE PLATELAYERS",
        health: 2620,
        strength: 70,
        regen: 6,
        mind_resist: 62,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 62,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 3, 1, 0),
            ("Cometfall", SlotKind::Weapon, 0, 2, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 2, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 2, 0),
            ("Tithe Collector", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 4, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
        ],
        gear_offset: 0,
        bounty: 206,
        sprite: MonsterSprite::Choir,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 3, 3, 2],
        enchs: &[],
    },
    // What came up out of the pit with the ballast. A wall, and the one weapon a
    // wall carries.
    MonsterSpec {
        name: "THE BALLAST",
        health: 2750,
        strength: 73,
        regen: 7,
        mind_resist: 65,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 65,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Anvil Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 2, 2, 0),
            ("Mana Ward", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 4, 1),
        ],
        gear_offset: 0,
        bounty: 215,
        sprite: MonsterSprite::Golem,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 4, 3],
        enchs: &[],
    },
    // The heap is warm. Searing on the clock rather than on the swing.
    MonsterSpec {
        name: "THE COAL STAGE",
        health: 2880,
        strength: 76,
        regen: 7,
        mind_resist: 68,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 68,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Last Rite", SlotKind::Weapon, 4, 1, 1),
            ("Prism Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Mana Ward", SlotKind::Helmet, 3, 0, 0),
            ("Warding Plate", SlotKind::Helmet, 0, 2, 0),
            ("Martyr's Crest", SlotKind::Helmet, 3, 1, 1),
            ("Anvil Frame", SlotKind::Helmet, 4, 2, 1),
            ("Overflow Plate", SlotKind::Helmet, 2, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 4, 0),
            ("Stonewall Frame", SlotKind::Helmet, 4, 5, 1),
            ("Overflow Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 6, 0),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 3, 3],
        enchs: &[],
    },
    // The tank sets the pace and has nothing much of its own.
    MonsterSpec {
        name: "THE WATER TOWER",
        health: 2880,
        strength: 76,
        regen: 7,
        mind_resist: 68,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 68,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 2, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 4, 1),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Wisp,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 4, 4],
        enchs: &[],
    },
    // Eleven arms, eleven casts. Bursty and mana-gated.
    MonsterSpec {
        name: "THE GANTRY",
        health: 2620,
        strength: 70,
        regen: 6,
        mind_resist: 62,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 62,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 2, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 1, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 4, 0),
            ("Martyr's Crest", SlotKind::Helmet, 5, 2, 1),
            ("Buttressed Frame", SlotKind::Helmet, 0, 5, 3),
            ("Overflow Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 4, 6, 0),
        ],
        gear_offset: 0,
        bounty: 206,
        sprite: MonsterSprite::Archer,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 4, 3],
        enchs: &[],
    },
    // Every lamp lit and burning. Kills on the clock, not the swing.
    MonsterSpec {
        name: "THE LAMP ROOM",
        health: 2750,
        strength: 73,
        regen: 7,
        mind_resist: 65,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 65,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Stonewall Frame", SlotKind::Helmet, 2, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 1, 4, 0),
            ("Warding Plate", SlotKind::Helmet, 3, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Buttressed Frame", SlotKind::Helmet, 0, 5, 3),
            ("Overflow Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 4, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 4, 1),
        ],
        gear_offset: 0,
        bounty: 215,
        sprite: MonsterSprite::Idol,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 4, 4],
        enchs: &[],
    },
    // The clerk keeps the accounts, yours included.
    MonsterSpec {
        name: "THE GOODS SHED",
        health: 2880,
        strength: 76,
        regen: 7,
        mind_resist: 68,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 68,
        attacks: &[],
        gear: &[
            ("Emberheart Orb", SlotKind::Weapon, 0, 0, 0),
            ("Cometfall", SlotKind::Weapon, 3, 0, 0),
            ("Emberburst", SlotKind::Weapon, 2, 1, 0),
            ("Cometfall", SlotKind::Weapon, 0, 2, 2),
            ("Rootwork Alignment", SlotKind::Weapon, 5, 1, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 0, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 0, 1),
            ("Buttressed Frame", SlotKind::Helmet, 2, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 4, 3, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("The Empty Crown", SlotKind::Helmet, 0, 4, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 5, 1),
            ("Overflow Plate", SlotKind::Helmet, 2, 6, 0),
            ("Consecrated Plating", SlotKind::Helmet, 4, 5, 0),
            ("The Empty Crown", SlotKind::Helmet, 4, 7, 0),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Tallow,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 4, 4, 4],
        enchs: &[],
    },
    // It is in steam. Strength, health, and no trick at all.
    MonsterSpec {
        name: "THE ROUNDHOUSE",
        health: 2880,
        strength: 76,
        regen: 7,
        mind_resist: 68,
        physical_resist: 45,
        magic_resist: 40,
        curse_resist: 68,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Hollow Lance", SlotKind::Weapon, 0, 2, 1),
            ("Cometfall", SlotKind::Weapon, 3, 1, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 0, 3, 0),
            ("Anvil Frame", SlotKind::Helmet, 0, 0, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 0, 0),
            ("Watchful Crest", SlotKind::Helmet, 5, 0, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 2, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 2, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 2, 1),
            ("Stonewall Frame", SlotKind::Helmet, 0, 4, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 4, 0),
            ("Stonewall Frame", SlotKind::Helmet, 0, 6, 0),
            ("Overflow Plate", SlotKind::Helmet, 3, 6, 0),
            ("The Empty Crown", SlotKind::Helmet, 5, 5, 1),
        ],
        gear_offset: 0,
        bounty: 224,
        sprite: MonsterSprite::Golem,
        rank: Rank::Ordinary,
        drops: &[],
        items: &[5, 3, 3, 2, 3],
        enchs: &[],
    },

    // ---------------------------------------------------- THE HUNDRED's five
    //
    // Three chain endings, the herd one of them drives, and the thing at the
    // end of the perambulation. Landed undressed: stats at their band and
    // `gear: &[]`, which is what `bestiary::unpacked()` counts and what the
    // frame lint goes red on until F12 dresses them.
    //
    // **Appended, never inserted, and the fixture is why.** `gear_at.txt` keys
    // every line on `ALTERNATES[i]`, so five specs at the top of this table
    // moved 2,592 placements without one creature changing what it wears -
    // which reads exactly like a re-gearing and is not one. `ALTERNATES` is
    // append-only for the same reason `CATALOG` is, and until this milestone
    // nothing said so anywhere.
    //
    // Bands take the ladder's stats at band, the Switchyard precedent. The
    // curve is defined at Medium and F12 is what measures them against it.
    MonsterSpec {
        // THE ORDNANCE. On the hill three lines cross at, and nothing marks
        // the hill - the lines do.
        name: "THE SURVEYOR",
        health: 3690,
        strength: 96,
        regen: 9,
        mind_resist: 72,
        physical_resist: 50,
        magic_resist: 46,
        curse_resist: 72,
        attacks: &[],
        gear: &[
            ("Grovemind Orb", SlotKind::Weapon, 0, 0, 0),
            ("Slash and Burn", SlotKind::Weapon, 3, 0, 0),
            ("Sunder", SlotKind::Weapon, 3, 1, 0),
            ("Starfall", SlotKind::Weapon, 0, 2, 0),
            ("Rootwork Alignment", SlotKind::Weapon, 3, 2, 0),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Bronze Frame", SlotKind::Helmet, 0, 2, 0),
            ("Reckoning Plate", SlotKind::Helmet, 2, 2, 0),
            ("Scrying Lens", SlotKind::Helmet, 4, 2, 1),
            ("Martyr's Crest", SlotKind::Helmet, 5, 0, 1),
            ("Stormcaught Frame", SlotKind::Helmet, 0, 4, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 5, 0),
            ("Scrying Lens", SlotKind::Helmet, 5, 3, 1),
            ("Bone Frame", SlotKind::Helmet, 0, 6, 0),
            ("Scrying Lens", SlotKind::Helmet, 1, 7, 0),
        ],
        gear_offset: 0,
        bounty: 273,
        sprite: MonsterSprite::Warden,
        rank: Rank::Ordinary,
        drops: &["Trig Pillar"],
        items: &[5, 2, 4, 3, 2],
        enchs: &[],
    },
    MonsterSpec {
        // THE DROVE ROADS, and the half of it that is a man.
        name: "THE DROVER",
        health: 5490,
        strength: 138,
        regen: 12,
        mind_resist: 86,
        physical_resist: 63,
        magic_resist: 60,
        curse_resist: 86,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Throttling Mold", SlotKind::Gloves, 3, 2, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 4, 1, 0),
            ("Rootwoven Material", SlotKind::Gloves, 5, 1, 1),
            ("Throttling Mold", SlotKind::Gloves, 4, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 2, 4, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 3, 1),
            ("Flaying Mold", SlotKind::Gloves, 1, 4, 3),
            ("Seal of the Deep", SlotKind::Gloves, 3, 5, 1),
            ("Grasping Ring", SlotKind::Gloves, 1, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 6, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 2, 6, 3),
            ("Seal of the Deep", SlotKind::Gloves, 0, 7, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 6, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 3, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 4, 4, 1),
            ("Consecrated Plating", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 5, 1),
        ],
        gear_offset: 0,
        bounty: 350,
        sprite: MonsterSprite::Marshal,
        rank: Rank::Ordinary,
        drops: &["Drove Way"],
        items: &[4, 4, 4, 4, 4, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        // And the half that is not. A drover without a herd is a man on a
        // walk, which is why the interception is a brawl.
        name: "THE DRIVEN",
        health: 4390,
        strength: 110,
        regen: 12,
        mind_resist: 70,
        physical_resist: 55,
        magic_resist: 52,
        curse_resist: 70,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Hexer's Tally", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 2, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 1, 2),
            ("Rootwoven Material", SlotKind::Gloves, 5, 1, 1),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 1, 3, 0),
            ("Grasping Ring", SlotKind::Gloves, 0, 3, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 4, 0),
            ("Flaying Mold", SlotKind::Gloves, 0, 5, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 5, 0),
            ("Grasping Ring", SlotKind::Gloves, 3, 5, 0),
            ("Rootwoven Material", SlotKind::Gloves, 4, 4, 1),
            ("Flaying Mold", SlotKind::Gloves, 2, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 4, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 5, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 2, 0),
            ("Tithe Collector", SlotKind::Helmet, 5, 0, 1),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 4, 1),
            ("Mirrored Visor", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 4, 4, 1),
        ],
        gear_offset: 0,
        bounty: 200,
        sprite: MonsterSprite::March,
        rank: Rank::Ordinary,
        drops: &["Drover's Orb"],
        items: &[4, 2, 4, 4, 4, 4, 2, 3],
        enchs: &[],
    },
    MonsterSpec {
        // THE ENCLOSURE, standing at the end of the corner the pale opens.
        name: "THE COMMISSIONER",
        health: 7720,
        strength: 192,
        regen: 17,
        mind_resist: 95,
        physical_resist: 74,
        magic_resist: 72,
        curse_resist: 95,
        attacks: &[],
        gear: &[
            ("Rootwoven Material", SlotKind::Gloves, 0, 0, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 0, 0),
            ("Seal of Power", SlotKind::Gloves, 0, 1, 0),
            ("Grasping Ring", SlotKind::Gloves, 5, 0, 0),
            ("Mage's Wrapping", SlotKind::Gloves, 4, 1, 0),
            ("Flaying Mold", SlotKind::Gloves, 2, 1, 3),
            ("Siphon Ring", SlotKind::Gloves, 1, 2, 0),
            ("Ring of Tides", SlotKind::Gloves, 0, 2, 0),
            ("Rootwoven Material", SlotKind::Gloves, 0, 3, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 3, 0),
            ("Seal of the Deep", SlotKind::Gloves, 0, 4, 0),
            ("Seal of the Deep", SlotKind::Gloves, 5, 3, 1),
            ("Rootwoven Material", SlotKind::Gloves, 0, 5, 0),
            ("Flaying Mold", SlotKind::Gloves, 3, 4, 2),
            ("Seal of the Deep", SlotKind::Gloves, 0, 6, 0),
            ("Grasping Ring", SlotKind::Gloves, 2, 4, 0),
            ("Rootwoven Material", SlotKind::Gloves, 2, 6, 0),
            ("Gauntlet Mold", SlotKind::Gloves, 3, 6, 3),
            ("Grasping Ring", SlotKind::Gloves, 5, 5, 0),
            ("Siphon Ring", SlotKind::Gloves, 2, 7, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 0, 0, 0),
            ("Consecrated Plating", SlotKind::Helmet, 3, 0, 0),
            ("Crown of the Deep", SlotKind::Helmet, 0, 2, 0),
            ("Overseer's Circlet", SlotKind::Helmet, 3, 2, 0),
            ("Consecrated Plating", SlotKind::Helmet, 2, 4, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 3, 3),
            ("Overseer's Circlet", SlotKind::Helmet, 4, 4, 1),
            ("Consecrated Plating", SlotKind::Helmet, 2, 6, 0),
            ("Mirrored Visor", SlotKind::Helmet, 0, 5, 1),
        ],
        gear_offset: 0,
        bounty: 416,
        sprite: MonsterSprite::Crown,
        rank: Rank::Ordinary,
        drops: &["The Common Ground"],
        items: &[4, 4, 4, 4, 4, 3, 3, 3],
        enchs: &[],
    },
    MonsterSpec {
        // THE PERAMBULATION's end. Band fifty and over: the county has spent
        // thirty tiles proving whoever got here has all five basis vectors,
        // and this is the thing that asks for all five at once.
        name: "THE PARISH",
        health: 9900,
        strength: 228,
        regen: 22,
        mind_resist: 96,
        physical_resist: 80,
        magic_resist: 78,
        curse_resist: 96,
        attacks: &[],
        gear: &[
            ("Fateglass Orb", SlotKind::Weapon, 0, 0, 0),
            ("Kingsbane", SlotKind::Weapon, 2, 0, 0),
            ("Shatterbolt", SlotKind::Weapon, 5, 0, 1),
            ("Emberburst", SlotKind::Weapon, 1, 1, 0),
            ("Pilgrim Alignment", SlotKind::Weapon, 0, 2, 0),
            ("Buttressed Frame", SlotKind::Helmet, 0, 0, 0),
            ("Deadweight Plating", SlotKind::Helmet, 3, 0, 1),
            ("Bloomcap", SlotKind::Helmet, 2, 1, 3),
            ("Crown of the Deep", SlotKind::Helmet, 0, 1, 3),
            ("Reliquary Frame of Nine", SlotKind::Helmet, 3, 1, 2),
            ("Visor of Focus", SlotKind::Helmet, 1, 3, 0),
            ("Bloomcap", SlotKind::Helmet, 4, 3, 3),
            ("Crown of the Deep", SlotKind::Helmet, 0, 4, 0),
            ("Buttressed Frame", SlotKind::Helmet, 2, 4, 2),
            ("Visor of Focus", SlotKind::Helmet, 5, 4, 1),
            ("Deadweight Plating", SlotKind::Helmet, 0, 6, 1),
            ("Crown of the Deep", SlotKind::Helmet, 2, 6, 2),
            ("Witch's Stilts", SlotKind::Gloves, 0, 0, 1),
            ("Hexer's Mold", SlotKind::Gloves, 3, 0, 3),
            ("Blightfinger", SlotKind::Gloves, 5, 0, 0),
            ("Blightfinger", SlotKind::Gloves, 1, 1, 0),
            ("Spun Material", SlotKind::Gloves, 4, 1, 1),
            ("Channeling Mold", SlotKind::Gloves, 2, 1, 3),
            ("Witch's Stilts", SlotKind::Gloves, 0, 2, 2),
            ("Channeling Mold", SlotKind::Gloves, 2, 3, 0),
            ("Witch's Stilts", SlotKind::Gloves, 0, 3, 0),
            ("Channeling Mold", SlotKind::Gloves, 2, 4, 2),
            ("Witch's Stilts", SlotKind::Gloves, 4, 3, 2),
            ("Channeling Mold", SlotKind::Gloves, 3, 5, 2),
            ("Mage's Sandals", SlotKind::Gloves, 0, 6, 0),
            ("Channeling Mold", SlotKind::Gloves, 1, 6, 2),
        ],
        gear_offset: 0,
        bounty: 560,
        sprite: MonsterSprite::Bells,
        rank: Rank::Ordinary,
        drops: &["Surveyor's Orb"],
        items: &[5, 4, 4, 4, 4, 2, 2, 2, 2, 2],
        enchs: &[],
    },
];

/// The floors of Bunko's Cavern, pp. 84-85. Authored by the packing tool like
/// every other named board; the gear lists are pasted from its output.
pub const CREVICE: &[MonsterSpec] = &[];

/// An alternate by name.
pub fn alternate(name: &str) -> Option<&'static MonsterSpec> {
    ALTERNATES.iter().find(|m| m.name == name)
}

/// Any creature in the game by name, wherever it is written.
///
/// `alternate` only knows the ones written specially for events. An event that
/// wants two creatures off the ladder itself - a pair of gamblers who are also
/// rungs twelve and thirteen - needs to find those too.
pub fn creature(name: &str) -> Option<&'static MonsterSpec> {
    LADDER.iter().find(|m| m.name == name).or_else(|| alternate(name))
}


#[cfg(test)]
mod stun_aim_tests {
    use super::*;
    use crate::stats::Stats;

    /// A fighter carrying items that differ only in how good they are.
    fn victim(ratings: &[i32]) -> Combatant {
        let mut c = Combatant::player(Stats::ZERO, &[]);
        c.items = ratings
            .iter()
            .enumerate()
            .map(|(i, &rating)| RunningItem {
                name: format!("item {i}"),
                rating,
                cooldown_ms: 1000,
                ..Default::default()
            })
            .collect();
        c
    }

    #[test]
    fn an_aimed_stun_always_takes_the_best_item() {
        let mut c = victim(&[10, 90, 40, 5]);
        for t in [0, 700, 1500, 2600] {
            let (idx, _) = land_stun(&mut c, StunAim::Strongest, t).expect("a stun landed");
            assert_eq!(idx, 1, "aimed at t={t} and missed the 90-rated item");
        }
    }

    #[test]
    fn an_unaimed_stun_spreads_across_the_kit() {
        let mut c = victim(&[10, 90, 40, 5]);
        let mut seen: Vec<usize> = Vec::new();
        // Four stuns, and nothing is stopped for long enough to still be
        // stopped when the next one lands.
        for (n, t) in [0u32, 5_000, 10_000, 15_000].into_iter().enumerate() {
            for item in &mut c.items {
                item.stun_ms = 0;
            }
            let (idx, _) = land_stun(&mut c, StunAim::Unaimed, t).expect("a stun landed");
            assert!(idx < 4, "picked item {idx} of four on stun {n}");
            if !seen.contains(&idx) {
                seen.push(idx);
            }
        }
        assert!(
            seen.len() >= 2,
            "four unaimed stuns all landed on item {seen:?} - it is meant to pick without \
             warning, not to be predictable"
        );
    }

    #[test]
    fn an_unaimed_stun_prefers_an_item_that_is_still_running() {
        let mut c = victim(&[10, 20, 30]);
        c.items[0].stun_ms = 900;
        c.items[2].stun_ms = 900;
        // Only item 1 is live, so wherever the hash points it has to end there
        // - burying an already-stopped item is the one thing this must not do.
        for t in [0, 350, 900, 1250, 4000] {
            let (idx, _) = land_stun(&mut c, StunAim::Unaimed, t).expect("a stun landed");
            assert_eq!(idx, 1, "at t={t} it stunned something already stopped");
            c.items[1].stun_ms = 0;
        }
    }

    #[test]
    fn stacking_piles_onto_one_clock_and_stops_at_the_cap() {
        let mut c = victim(&[10, 90]);
        let base = CurseKind::Stun.landing_ms(0);
        let (_, first) = land_stun(&mut c, StunAim::Strongest, 0).unwrap();
        assert_eq!(first, base);
        let (_, second) = land_stun(&mut c, StunAim::Strongest, 100).unwrap();
        assert_eq!(second, base * 2, "a second stun on the same item has to add to the clock");
        for t in 0..20 {
            land_stun(&mut c, StunAim::Strongest, t * 100);
        }
        assert_eq!(c.items[1].stun_ms, STUN_CAP_MS, "a stun chain is not a lock");
        assert_eq!(c.items[0].stun_ms, 0, "the aimed stun never wandered off its target");
    }

    /// **The most resistant thing in the game is still stunned, briefly.**
    ///
    /// This asserted *never* until M16, and the reason it changed is in
    /// [`crate::stats::LANE_CAP`]: at a hundred the curse lane shut out
    /// completely, and twenty-three creatures were there. What is still pinned
    /// is the part that was the point — the aim does not wander, and a stun
    /// that lands on the resistant target lands on the item it was aimed at.
    #[test]
    fn a_fully_resistant_target_is_stunned_for_a_twentieth() {
        let mut c = victim(&[10, 90]);
        c.curse_resist = 400;
        let (item, ms) = land_stun(&mut c, StunAim::Strongest, 0).expect("a twentieth is not none");
        assert_eq!(item, 1, "the aimed stun lands on the strongest item");
        // One tick: a twentieth of 1200ms, floored to the 50ms grid.
        assert_eq!(ms, 50);
        assert_eq!(c.items[0].stun_ms, 0, "and not on the other one");
    }

    #[test]
    fn a_fighter_with_no_items_cannot_be_stunned() {
        let mut c = victim(&[]);
        assert!(land_stun(&mut c, StunAim::Unaimed, 0).is_none());
    }
}

// ------------------------------------------------- what one item actually did
//
// A `CombatLog` is a flat transcript and everything in it is true, which is
// not the same as being readable: a fight is forty lines of consequence and
// the question a player has is "what did *that* piece do". Answering it needs
// attribution, and attribution needs a rule.
//
// **The rule.** `Event::Activate` is documented to precede its own item's
// effects and carries the item's index, so a hit belongs to whichever item on
// that side last activated. That is exactly the rule `tests/baseline.rs` has
// attributed damage by since the slot rewrite, stated once here so the
// interface and the measurement cannot drift apart. Its known limit is stated
// with it: strength and power granted by other slots land under the item that
// swung, which is the intended reading of "the weapon deals the damage".
//
// Events belonging to nobody - sudden death, the ending - are attributed to
// nobody rather than to whatever fired last.

/// One line of an item's account: a series the graphs draw, and what this item
/// put into or took out of it.
///
/// The names match `build_series`' own, plus two the graphs do not draw as the
/// item's own line because they land on the other side: `damage` and `mind`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contribution {
    pub what: &'static str,
    pub amount: i32,
}

/// Everything one item did in one fight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemTally {
    /// Position in its owner's item list, which is how `Activate` names it.
    pub index: usize,
    pub name: String,
    pub activations: u32,
    pub misfires: u32,
    /// Total time this one item spent stopped.
    pub stunned_ms: u32,
    /// Non-zero lines only, in a fixed order so two tallies read the same way.
    pub contributed: Vec<Contribution>,
    /// Indices into `CombatLog::entries` that belong to this item, including
    /// its own activations. The interface shows the log filtered to these.
    pub entries: Vec<usize>,
}

impl ItemTally {
    /// One line's worth, or zero if this item never touched it.
    pub fn of(&self, what: &str) -> i32 {
        self.contributed.iter().find(|c| c.what == what).map(|c| c.amount).unwrap_or(0)
    }
}

/// The account for every item on one side of one fight.
///
/// `who` selects the foe in a party fight and is ignored for the player, who
/// is always singular - the same convention `LogEntry::who` uses.
pub fn tally_items(log: &CombatLog, side: Side, who: u8) -> Vec<ItemTally> {
    let owner = match side {
        Side::Player => &log.player,
        Side::Enemy => match log.enemies.get(who as usize) {
            Some(c) => c,
            None => return Vec::new(),
        },
    };
    let mine = |e: &LogEntry| side == Side::Player || e.who == who;

    let mut out: Vec<ItemTally> = owner
        .items
        .iter()
        .enumerate()
        .map(|(index, it)| ItemTally {
            index,
            name: it.name.clone(),
            activations: 0,
            misfires: 0,
            stunned_ms: 0,
            contributed: Vec::new(),
            entries: Vec::new(),
        })
        .collect();
    if out.is_empty() {
        return out;
    }

    // Six graph lines plus the two that land on the other side, per item.
    let mut books: Vec<Vec<(&'static str, i32)>> = vec![Vec::new(); out.len()];
    let add = |books: &mut Vec<Vec<(&'static str, i32)>>, at: Option<usize>, what, n: i32| {
        if n == 0 {
            return;
        }
        let Some(i) = at else { return };
        let book = &mut books[i];
        match book.iter_mut().find(|(w, _)| *w == what) {
            Some((_, sum)) => *sum += n,
            None => book.push((what, n)),
        }
    };

    let mut acting: Option<usize> = None;
    for (i, e) in log.entries.iter().enumerate() {
        match &e.event {
            // An activation opens the account and closes the last one.
            Event::Activate { side: s, index, .. } if *s == side && mine(e) => {
                acting = out.get(*index).map(|_| *index);
                if let Some(a) = acting {
                    out[a].activations += 1;
                    out[a].entries.push(i);
                }
                continue;
            }
            // Two events name their own item rather than relying on the last
            // activation, because both happen *instead* of one.
            Event::Misfired { side: s, item } if *s == side && mine(e) => {
                if let Some(a) = out.iter().position(|t| t.name == *item) {
                    out[a].misfires += 1;
                    out[a].entries.push(i);
                }
                continue;
            }
            Event::Stunned { on, index, duration_ms, .. } if *on == side && mine(e) => {
                if let Some(t) = out.get_mut(*index) {
                    t.stunned_ms += duration_ms;
                    t.entries.push(i);
                }
                continue;
            }
            // Belongs to nobody: the clock, and the ending.
            Event::SuddenDeath { .. } | Event::End { .. } => {
                acting = None;
                continue;
            }
            _ => {}
        }

        // Everything else is the standing item's, if it is on this side.
        let ours = match &e.event {
            Event::Hit { by, .. } | Event::MindHit { by, .. } => *by == side,
            Event::GainResource { side: s, .. }
            | Event::Spent { side: s, .. }
            | Event::Cast { side: s, .. }
            | Event::Grew { side: s, .. }
            | Event::GainArmor { side: s, .. }
            | Event::GainMana { side: s, .. }
            | Event::ManaCheck { side: s, .. }
            | Event::ResourceCheck { side: s, .. }
            | Event::Burn { side: s, .. }
            | Event::Regen { side: s, .. }
            | Event::Hastened { side: s, .. }
            | Event::Shunted { side: s, .. }
            | Event::Derailed { side: s, .. }
            | Event::Reflected { side: s, .. }
            | Event::Fused { side: s, .. }
            | Event::Watched { side: s, .. }
            | Event::Empowered { side: s, .. }
            | Event::Shielded { side: s, .. }
            | Event::Whetted { side: s, .. }
            | Event::Deflecting { side: s, .. }
            | Event::Dreading { side: s, .. }
            | Event::Forking { side: s, .. }
            | Event::Warded { side: s, .. }
            | Event::Fell { side: s } => *s == side,
            // A curse and a drain are named for whoever they landed *on*, so
            // this side owns them when the other side is the target.
            Event::Cursed { on, .. } | Event::Drained { on, .. } => *on != side,
            _ => false,
        };
        if !ours || !mine(e) {
            continue;
        }
        let Some(a) = acting else { continue };
        out[a].entries.push(i);

        match &e.event {
            Event::Hit { damage, .. } => add(&mut books, Some(a), "damage", *damage),
            Event::MindHit { amount, .. } => add(&mut books, Some(a), "mind", *amount),
            Event::Reflected { damage, .. } => add(&mut books, Some(a), "damage", *damage),
            Event::Burn { damage, .. } => add(&mut books, Some(a), "damage", *damage),
            Event::GainArmor { amount, .. } => add(&mut books, Some(a), "armour", *amount),
            Event::GainMana { amount, .. } => add(&mut books, Some(a), "mana", *amount),
            Event::ManaCheck { cost, paid, .. } => {
                if *paid {
                    add(&mut books, Some(a), "mana", -*cost)
                }
            }
            Event::Cast { paid, cost, .. } => {
                if *paid {
                    add(&mut books, Some(a), "mana", -*cost)
                }
            }
            Event::Regen { amount, .. } | Event::Grew { amount, .. } => {
                add(&mut books, Some(a), "health", *amount)
            }
            Event::GainResource { what, amount, .. } => add(&mut books, Some(a), pool_line(what), *amount),
            Event::ResourceCheck { what, cost, paid, .. } => {
                if *paid {
                    add(&mut books, Some(a), pool_line(what), -*cost)
                }
            }
            Event::Spent { amount, .. } => add(&mut books, Some(a), "gold", *amount),
            _ => {}
        }
    }

    // A fixed order, so two tallies are read the same way round.
    const ORDER: &[&str] =
        &["damage", "mind", "health", "armour", "mana", "rage", "faith", "nature", "gold"];
    for (t, book) in out.iter_mut().zip(books) {
        for what in ORDER {
            if let Some((_, n)) = book.iter().find(|(w, _)| w == what) {
                t.contributed.push(Contribution { what, amount: *n });
            }
        }
    }
    out
}

/// The graph a pool's events belong to. `build_series` names them this way.
fn pool_line(what: &str) -> &'static str {
    match what {
        "rage" => "rage",
        "faith" => "faith",
        _ => "nature",
    }
}
