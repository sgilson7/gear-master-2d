//! The ten experts, their promises, and their knobs.
//!
//! # What an expert is
//!
//! A third class, taken free once **two class trees are finished**, decided by
//! **which pair you hold** rather than picked off a list. There are ten of
//! them because there are five offered classes and `C(5,2) = 10`, so every
//! pair a player can reach has one and no pair reaches two.
//!
//! **An expert replaces nothing.** Both parent powers stay on;
//! `Character::classes` is the list and the three places a power is honoured —
//! the purse, the fighter at the bell, the board — fold over it. That is why
//! no arbitration had to be invented: the five shipped powers touch five
//! different rules, the ten below are written to the same constraint, and
//! `no_pair_of_live_powers_disagrees` is what keeps it true.
//!
//! # What a knob is, and why the tree may only move one
//!
//! A **knob** is a named integer on a live power that a skill node may move,
//! through [`crate::skills::Effect::Tunes`]. Knobs are declared here, in Rust,
//! beside the power that owns them, and a tree naming one its own class has
//! not got **does not load** — the same guard `Rule::check` already applies to
//! a slot or a curse name.
//!
//! The constraint that gives this block its shape is that **every node of an
//! expert tree must reach that expert's power**: a `tunes` of a declared knob,
//! a `grants` of a rule the power is kin to, or a `gives_ench` inside the
//! power's licence. No flat stats, no starting balances, no rows. A `+12
//! strength` node would be a node you could take without noticing which class
//! you were in — the five base trees are allowed to be a mix because they are
//! the character's first shape, and an expert tree is the argument for its own
//! promise, six nodes long. `expert_nodes_touch_only_the_expert` is the lint.
//!
//! # Tenths
//!
//! Integer per-mille and per-ten are how this engine keeps two machines
//! agreeing about a number, so a fractional knob is stored in tenths and
//! printed with a decimal point: `rate: 20` is 2.0 strength a point. Everything
//! a player reads is printed by [`ExpertPower::describe`] **from the tuned
//! value**, so the promise re-reads itself after every point spent and cannot
//! go stale. That was already the rule for the base classes; it is now the
//! rule for twelve points of tuning.

/// One expert's power, carrying its own knobs as named integers.
///
/// **One variant per expert, and one `ClassPower::Expert` holding all ten.**
/// The alternative — ten variants on `ClassPower` — is ten new arms in every
/// exhaustive match in the engine, and there are three of those with thirty
/// arms already. A nested enum is one arm each and the exhaustiveness that
/// guard rests on is unchanged: matching `Expert(e)` and then matching `e` is
/// still two exhaustive matches.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExpertPower {
    /// Gorillathon × Funnel Sergeant. *Muscle is a mana pool with an exchange
    /// rate.* A cast short of Funny pays the difference in strength.
    ///
    /// `rate` is tenths of a point of strength per point of Funny.
    LoudCalculation { rate: i32, cap: i32, floor: i32, rebate: i32 },
    /// Gorillathon × Worm-Fact Keeper. Curses landed while wearing little
    /// enough do not expire.
    ///
    /// `told` is how many follow you into the next encounter — **the one knob
    /// in the block that crosses a fight boundary**, which is why it is the
    /// capstone of its tree and nothing else reaches past the bell.
    StandingFact { worn: i32, carry: i32, bite: i32, told: i32 },
    /// Gorillathon × Kaklon Patent. An empty frame spins anyway, into your
    /// bare strength. `half` is tenths of a stack a turn.
    OverwoundArm { half: i32, ceiling: i32, carry: i32 },
    /// Gorillathon × Top of the Bill. The Showstopper window widens with every
    /// frame you left empty.
    ShortProgramme { per_slot: i32, floor_ms: i32, pct: i32, streak: i32 },
    /// Funnel Sergeant × Worm-Fact Keeper. A cast may be paid for by consuming
    /// a curse you landed.
    ///
    /// `relist` is a **level, not a period**: 0 never, and every step up puts
    /// the curse back one requisition sooner — see [`ExpertPower::relist_every`],
    /// which is the one place that arithmetic is done.
    CurseRequisition { per_fight: i32, worth: i32, relist: i32, pick: i32 },
    /// Funnel Sergeant × Kaklon Patent. The spin banks Funny.
    ///
    /// `per_cell` **adds to** what `Rule::RowHarvest` says it pays rather than
    /// replacing it. The rule is the switch and the knob is the tuning, which
    /// is the same division `SpinExtra` makes over a spin an ench granted — and
    /// the reason there are not two answers to what a filled row is worth.
    ///
    /// **It was called `harvest` and could not stay.** Harvest is this game's
    /// word for the nature pool, so a node's line reading `+2 harvest` promised
    /// a pool it does not touch — caught by `no_mechanical_line_speaks_the_theme`
    /// the moment the trees landed. It shares its name with the rule's own
    /// field now, which is what it tunes.
    PatentedFunnel { per_stack: i32, bleed: i32, overflow: i32, per_cell: i32 },
    /// Funnel Sergeant × Top of the Bill. The first seconds are free.
    OpeningNumber { window_ms: i32, after: i32, encore: i32, bank: i32 },
    /// Worm-Fact Keeper × Kaklon Patent. An enched component curses on every
    /// activation of its frame.
    CursedLicence { stack: i32 },
    /// Worm-Fact Keeper × Top of the Bill. The fallen are billed by the curse.
    ///
    /// `posthumous` is **tenths of a curse**: 5 is a curse that expired before
    /// the bell counting half.
    EleventhSeason { pct: i32, count_cap: i32, distinct: i32, posthumous: i32 },
    /// Kaklon Patent × Top of the Bill. Two enchs a component, off either
    /// licence's list.
    ///
    /// `beacon_pct` adds to `Rule::Beacon`, for the reason `harvest` does.
    FullBill { racks: i32, beacon_pct: i32 },

    // ------------------------------------------------- M16's eleven
    //
    // **`C(7,2)` is twenty-one and the table held ten**, which is the whole of
    // why these exist: a roster that grows leaves every pair it makes without a
    // counter, and `every_pair_of_offered_classes_reaches_an_expert` said so on
    // the commit that made the roster seven.
    //
    // Each one's knobs land on a field the fight already reads wherever they
    // can, which is M13's rule and is what keeps eleven new powers from being
    // eleven new mechanics. Where a field is new it is one field, read in one
    // place, and the promise says what it does.

    /// Stoker × Gorillathon. *An empty frame is a hopper that never empties.*
    ///
    /// `per_frame` is stacks at the bell per bare frame, in tenths.
    BareFurnace { per_frame: i32, every_ms: i32, cap: i32 },
    /// Stoker × Funnel Sergeant. *The furnace pays the funnel.* Every stack the
    /// furnace buys is also `worth` mana.
    FiredFunnel { worth: i32, per_fight: i32, refund: i32 },
    /// Stoker × Worm-Fact Keeper. *A curse you land is fuel.* Every curse
    /// landed buys `per_stack` stacks, and `standing` doubles it for a curse
    /// that cannot expire.
    ColdStoke { per_stack: i32, every_ms: i32, standing: i32 },
    /// Stoker × Kaklon Patent. *The spin feeds the furnace.* Every turn a
    /// spinning item banks is `per_spin` tenths of a stack.
    PonkeyBoiler { per_spin: i32, keep: i32, licence: i32 },
    /// Stoker × Top of the Bill. *Everything at once, and early.* `all_at_once`
    /// percent of every pool is burned before the first tick.
    FlashPowder { all_at_once: i32, until_ms: i32, pct: i32 },
    /// Whisperer × Gorillathon. *Said plainly.* Wearing `worn` items or fewer,
    /// strength counts toward mind damage at `rate` quarters.
    LoudDoubt { worn: i32, rate: i32, third: i32 },
    /// Whisperer × Funnel Sergeant. *A cast you cannot pay for is said anyway*,
    /// as mind damage worth `rate` percent of what it would have cost.
    RequisitionedSilence { rate: i32, per_fight: i32, third: i32 },
    /// Whisperer × Worm-Fact Keeper. *Each curse standing on them raises the
    /// threshold*, by `per_curse` points, up to `cap`.
    ToldOnce { per_curse: i32, cap: i32, standing: i32 },
    /// Whisperer × Kaklon Patent. *An enched component says something.* Every
    /// activation of one eats `pct` tenths of a percent of their maximum.
    LicensedRumour { pct: i32, racks: i32, third: i32 },
    /// Whisperer × Top of the Bill. *A creature unmade inside the window pays
    /// the Showstopper's cut `mult` percent again.*
    CurtainLine { mult: i32, under_ms: i32, third: i32 },
    /// Stoker × Whisperer. *Every point burned is also said*, as mind damage at
    /// `rate` halves.
    AshAndWhisper { rate: i32, third: i32, every_ms: i32 },
}

/// Tenths, printed. `20` is `2.0`.
fn tenths(n: i32) -> String {
    format!("{}.{}", n / 10, (n % 10).abs())
}

impl ExpertPower {
    /// The knob names this power declares.
    ///
    /// **The vocabulary lives with the power**, for the reason a rule's slot
    /// name lives in `piece.rs`: a second enum listing rate-cap-floor-rebate
    /// would be two lists to keep in step, and this one is read by
    /// `SkillsData::parse` so a tree naming a knob that is not here does not
    /// load.
    pub fn knobs(self) -> &'static [&'static str] {
        use ExpertPower::*;
        match self {
            LoudCalculation { .. } => &["rate", "cap", "floor", "rebate"],
            StandingFact { .. } => &["worn", "carry", "bite", "told"],
            OverwoundArm { .. } => &["half", "ceiling", "carry"],
            ShortProgramme { .. } => &["per_slot", "floor_ms", "pct", "streak"],
            CurseRequisition { .. } => &["per_fight", "worth", "relist", "pick"],
            PatentedFunnel { .. } => &["per_stack", "bleed", "overflow", "per_cell"],
            OpeningNumber { .. } => &["window_ms", "after", "encore", "bank"],
            CursedLicence { .. } => &["stack"],
            EleventhSeason { .. } => &["pct", "count_cap", "distinct", "posthumous"],
            FullBill { .. } => &["racks", "beacon_pct"],
            BareFurnace { .. } => &["per_frame", "every_ms", "cap"],
            FiredFunnel { .. } => &["worth", "per_fight", "refund"],
            ColdStoke { .. } => &["per_stack", "every_ms", "standing"],
            PonkeyBoiler { .. } => &["per_spin", "keep", "licence"],
            FlashPowder { .. } => &["all_at_once", "until_ms", "pct"],
            LoudDoubt { .. } => &["worn", "rate", "third"],
            RequisitionedSilence { .. } => &["rate", "per_fight", "third"],
            ToldOnce { .. } => &["per_curse", "cap", "standing"],
            LicensedRumour { .. } => &["pct", "racks", "third"],
            CurtainLine { .. } => &["mult", "under_ms", "third"],
            AshAndWhisper { .. } => &["rate", "third", "every_ms"],
        }
    }

    /// The smallest move of this knob that a player can **see**.
    ///
    /// **Because two of the thirty-one are printed coarser than they are
    /// stored.** `floor_ms` and `window_ms` are milliseconds and
    /// [`Self::describe`] prints whole seconds, so a node moving one of them
    /// by 500 would cost two points and change no sentence anywhere — which is
    /// the *eight skill nodes that did nothing* failure with a decimal point
    /// in it, and this project has shipped that shape three times.
    ///
    /// Declared off the name rather than per variant: a knob whose name ends
    /// `_ms` is stored in milliseconds and shown in seconds, and everything
    /// else — tenths included, which *are* printed to a tenth — moves one at a
    /// time. `every_expert_node_moves_a_knob_it_can_be_seen_to_move` is what
    /// holds the trees to it.
    pub fn step(knob: &str) -> i32 {
        if knob.ends_with("_ms") {
            1000
        } else {
            1
        }
    }

    /// The same, asked of **one power** rather than of a knob name.
    ///
    /// **M16's, and it is the same finding `ClassPower::step` is.** The rule is
    /// *the smallest move a player can see*, and what a player can see depends
    /// on how the owning power prints the number. A knob ending `_ms` is
    /// printed in whole seconds by the two experts it was written for — so 500
    /// costs two points and changes no sentence — and to a **tenth** by the
    /// four furnace experts, where eight hundred is 4.0s becoming 3.2s.
    ///
    /// Read off `describe`'s own format string rather than off a list: the four
    /// that print a tenth are the four whose clock is a furnace's.
    pub fn step_of(self, knob: &str) -> i32 {
        use ExpertPower::*;
        match (self, knob) {
            (BareFurnace { .. } | ColdStoke { .. } | AshAndWhisper { .. }, "every_ms") => 100,
            _ => Self::step(knob),
        }
    }

    /// Read a knob. `None` for a name this power has not got.
    pub fn knob(self, name: &str) -> Option<i32> {
        use ExpertPower::*;
        Some(match (self, name) {
            (LoudCalculation { rate, .. }, "rate") => rate,
            (LoudCalculation { cap, .. }, "cap") => cap,
            (LoudCalculation { floor, .. }, "floor") => floor,
            (LoudCalculation { rebate, .. }, "rebate") => rebate,
            (StandingFact { worn, .. }, "worn") => worn,
            (StandingFact { carry, .. }, "carry") => carry,
            (StandingFact { bite, .. }, "bite") => bite,
            (StandingFact { told, .. }, "told") => told,
            (OverwoundArm { half, .. }, "half") => half,
            (OverwoundArm { ceiling, .. }, "ceiling") => ceiling,
            (OverwoundArm { carry, .. }, "carry") => carry,
            (ShortProgramme { per_slot, .. }, "per_slot") => per_slot,
            (ShortProgramme { floor_ms, .. }, "floor_ms") => floor_ms,
            (ShortProgramme { pct, .. }, "pct") => pct,
            (ShortProgramme { streak, .. }, "streak") => streak,
            (CurseRequisition { per_fight, .. }, "per_fight") => per_fight,
            (CurseRequisition { worth, .. }, "worth") => worth,
            (CurseRequisition { relist, .. }, "relist") => relist,
            (CurseRequisition { pick, .. }, "pick") => pick,
            (PatentedFunnel { per_stack, .. }, "per_stack") => per_stack,
            (PatentedFunnel { bleed, .. }, "bleed") => bleed,
            (PatentedFunnel { overflow, .. }, "overflow") => overflow,
            (PatentedFunnel { per_cell, .. }, "per_cell") => per_cell,
            (OpeningNumber { window_ms, .. }, "window_ms") => window_ms,
            (OpeningNumber { after, .. }, "after") => after,
            (OpeningNumber { encore, .. }, "encore") => encore,
            (OpeningNumber { bank, .. }, "bank") => bank,
            (CursedLicence { stack }, "stack") => stack,
            (EleventhSeason { pct, .. }, "pct") => pct,
            (EleventhSeason { count_cap, .. }, "count_cap") => count_cap,
            (EleventhSeason { distinct, .. }, "distinct") => distinct,
            (EleventhSeason { posthumous, .. }, "posthumous") => posthumous,
            (FullBill { racks, .. }, "racks") => racks,
            (FullBill { beacon_pct, .. }, "beacon_pct") => beacon_pct,
            (BareFurnace { per_frame, .. }, "per_frame") => per_frame,
            (BareFurnace { every_ms, .. }, "every_ms") => every_ms,
            (BareFurnace { cap, .. }, "cap") => cap,
            (FiredFunnel { worth, .. }, "worth") => worth,
            (FiredFunnel { per_fight, .. }, "per_fight") => per_fight,
            (FiredFunnel { refund, .. }, "refund") => refund,
            (ColdStoke { per_stack, .. }, "per_stack") => per_stack,
            (ColdStoke { every_ms, .. }, "every_ms") => every_ms,
            (ColdStoke { standing, .. }, "standing") => standing,
            (PonkeyBoiler { per_spin, .. }, "per_spin") => per_spin,
            (PonkeyBoiler { keep, .. }, "keep") => keep,
            (PonkeyBoiler { licence, .. }, "licence") => licence,
            (FlashPowder { all_at_once, .. }, "all_at_once") => all_at_once,
            (FlashPowder { until_ms, .. }, "until_ms") => until_ms,
            (FlashPowder { pct, .. }, "pct") => pct,
            (LoudDoubt { worn, .. }, "worn") => worn,
            (LoudDoubt { rate, .. }, "rate") => rate,
            (LoudDoubt { third, .. }, "third") => third,
            (RequisitionedSilence { rate, .. }, "rate") => rate,
            (RequisitionedSilence { per_fight, .. }, "per_fight") => per_fight,
            (RequisitionedSilence { third, .. }, "third") => third,
            (ToldOnce { per_curse, .. }, "per_curse") => per_curse,
            (ToldOnce { cap, .. }, "cap") => cap,
            (ToldOnce { standing, .. }, "standing") => standing,
            (LicensedRumour { pct, .. }, "pct") => pct,
            (LicensedRumour { racks, .. }, "racks") => racks,
            (LicensedRumour { third, .. }, "third") => third,
            (CurtainLine { mult, .. }, "mult") => mult,
            (CurtainLine { under_ms, .. }, "under_ms") => under_ms,
            (CurtainLine { third, .. }, "third") => third,
            (AshAndWhisper { rate, .. }, "rate") => rate,
            (AshAndWhisper { third, .. }, "third") => third,
            (AshAndWhisper { every_ms, .. }, "every_ms") => every_ms,
            _ => return None,
        })
    }

    /// Move a knob by `by`. A name this power has not got moves nothing.
    ///
    /// **Silently, and that is safe here only because of the parse guard.**
    /// `SkillsData::parse` refuses a tree naming an undeclared knob, so a
    /// no-op here means a node in *another* class's tree, which is a node this
    /// character cannot have taken. Without that guard this would be the
    /// serde-drops-a-key failure with a new coat of paint.
    pub fn tune(&mut self, name: &str, by: i32) {
        use ExpertPower::*;
        match (self, name) {
            (LoudCalculation { rate, .. }, "rate") => *rate += by,
            (LoudCalculation { cap, .. }, "cap") => *cap += by,
            (LoudCalculation { floor, .. }, "floor") => *floor += by,
            (LoudCalculation { rebate, .. }, "rebate") => *rebate += by,
            (StandingFact { worn, .. }, "worn") => *worn += by,
            (StandingFact { carry, .. }, "carry") => *carry += by,
            (StandingFact { bite, .. }, "bite") => *bite += by,
            (StandingFact { told, .. }, "told") => *told += by,
            (OverwoundArm { half, .. }, "half") => *half += by,
            (OverwoundArm { ceiling, .. }, "ceiling") => *ceiling += by,
            (OverwoundArm { carry, .. }, "carry") => *carry += by,
            (ShortProgramme { per_slot, .. }, "per_slot") => *per_slot += by,
            (ShortProgramme { floor_ms, .. }, "floor_ms") => *floor_ms += by,
            (ShortProgramme { pct, .. }, "pct") => *pct += by,
            (ShortProgramme { streak, .. }, "streak") => *streak += by,
            (CurseRequisition { per_fight, .. }, "per_fight") => *per_fight += by,
            (CurseRequisition { worth, .. }, "worth") => *worth += by,
            (CurseRequisition { relist, .. }, "relist") => *relist += by,
            (CurseRequisition { pick, .. }, "pick") => *pick += by,
            (PatentedFunnel { per_stack, .. }, "per_stack") => *per_stack += by,
            (PatentedFunnel { bleed, .. }, "bleed") => *bleed += by,
            (PatentedFunnel { overflow, .. }, "overflow") => *overflow += by,
            (PatentedFunnel { per_cell, .. }, "per_cell") => *per_cell += by,
            (OpeningNumber { window_ms, .. }, "window_ms") => *window_ms += by,
            (OpeningNumber { after, .. }, "after") => *after += by,
            (OpeningNumber { encore, .. }, "encore") => *encore += by,
            (OpeningNumber { bank, .. }, "bank") => *bank += by,
            (CursedLicence { stack }, "stack") => *stack += by,
            (EleventhSeason { pct, .. }, "pct") => *pct += by,
            (EleventhSeason { count_cap, .. }, "count_cap") => *count_cap += by,
            (EleventhSeason { distinct, .. }, "distinct") => *distinct += by,
            (EleventhSeason { posthumous, .. }, "posthumous") => *posthumous += by,
            (FullBill { racks, .. }, "racks") => *racks += by,
            (BareFurnace { per_frame, .. }, "per_frame") => *per_frame += by,
            (BareFurnace { every_ms, .. }, "every_ms") => *every_ms += by,
            (BareFurnace { cap, .. }, "cap") => *cap += by,
            (FiredFunnel { worth, .. }, "worth") => *worth += by,
            (FiredFunnel { per_fight, .. }, "per_fight") => *per_fight += by,
            (FiredFunnel { refund, .. }, "refund") => *refund += by,
            (ColdStoke { per_stack, .. }, "per_stack") => *per_stack += by,
            (ColdStoke { every_ms, .. }, "every_ms") => *every_ms += by,
            (ColdStoke { standing, .. }, "standing") => *standing += by,
            (PonkeyBoiler { per_spin, .. }, "per_spin") => *per_spin += by,
            (PonkeyBoiler { keep, .. }, "keep") => *keep += by,
            (PonkeyBoiler { licence, .. }, "licence") => *licence += by,
            (FlashPowder { all_at_once, .. }, "all_at_once") => *all_at_once += by,
            (FlashPowder { until_ms, .. }, "until_ms") => *until_ms += by,
            (FlashPowder { pct, .. }, "pct") => *pct += by,
            (LoudDoubt { worn, .. }, "worn") => *worn += by,
            (LoudDoubt { rate, .. }, "rate") => *rate += by,
            (LoudDoubt { third, .. }, "third") => *third += by,
            (RequisitionedSilence { rate, .. }, "rate") => *rate += by,
            (RequisitionedSilence { per_fight, .. }, "per_fight") => *per_fight += by,
            (RequisitionedSilence { third, .. }, "third") => *third += by,
            (ToldOnce { per_curse, .. }, "per_curse") => *per_curse += by,
            (ToldOnce { cap, .. }, "cap") => *cap += by,
            (ToldOnce { standing, .. }, "standing") => *standing += by,
            (LicensedRumour { pct, .. }, "pct") => *pct += by,
            (LicensedRumour { racks, .. }, "racks") => *racks += by,
            (LicensedRumour { third, .. }, "third") => *third += by,
            (CurtainLine { mult, .. }, "mult") => *mult += by,
            (CurtainLine { under_ms, .. }, "under_ms") => *under_ms += by,
            (CurtainLine { third, .. }, "third") => *third += by,
            (AshAndWhisper { rate, .. }, "rate") => *rate += by,
            (AshAndWhisper { third, .. }, "third") => *third += by,
            (AshAndWhisper { every_ms, .. }, "every_ms") => *every_ms += by,
            (FullBill { beacon_pct, .. }, "beacon_pct") => *beacon_pct += by,
            _ => {}
        }
    }

    /// What a cast costs once the Opening Number's free window has shut.
    ///
    /// **Rounded the payer's way, and it has to be.** A cast costs three, so a
    /// fifth off three is nought point six — and integer division of a discount
    /// is a node that sells *twenty percent less* and takes exactly nothing
    /// off, which is the eight-dead-skill-nodes failure with a decimal point in
    /// it. `on-standing-discount` was that node until M13.6.
    ///
    /// **One place**, because `pay_for_a_cast` charges it and `describe` prints
    /// it, and a price worked out twice is a promise that goes stale — the same
    /// reason [`ExpertPower::relist_every`] exists.
    pub fn cast_price(after: i32) -> i32 {
        let full = crate::combat::SPELL_MANA_COST;
        full - (full * after.clamp(0, 100) + 99) / 100
    }

    /// Which requisition puts the curse back: never, or every nth.
    ///
    /// **One place, because `relist` is a level and not a period.** The tree
    /// spends two points raising it from nothing to every third to every
    /// second, and a screen that did `4 - relist` for itself would be a second
    /// answer to a question the power owns.
    pub fn relist_every(relist: i32) -> Option<u32> {
        (relist > 0).then(|| (4 - relist).max(2) as u32)
    }

    /// The canonical class name this power belongs to.
    ///
    /// Matches the `class` key of the tree in `data/skills.json`, which is
    /// what makes `tree_for_class` find it.
    pub fn id(self) -> &'static str {
        use ExpertPower::*;
        match self {
            BareFurnace { .. } => "BareFurnace",
            FiredFunnel { .. } => "FiredFunnel",
            ColdStoke { .. } => "ColdStoke",
            PonkeyBoiler { .. } => "PonkeyBoiler",
            FlashPowder { .. } => "FlashPowder",
            LoudDoubt { .. } => "LoudDoubt",
            RequisitionedSilence { .. } => "RequisitionedSilence",
            ToldOnce { .. } => "ToldOnce",
            LicensedRumour { .. } => "LicensedRumour",
            CurtainLine { .. } => "CurtainLine",
            AshAndWhisper { .. } => "AshAndWhisper",
            LoudCalculation { .. } => "LoudCalculation",
            StandingFact { .. } => "StandingFact",
            OverwoundArm { .. } => "OverwoundArm",
            ShortProgramme { .. } => "ShortProgramme",
            CurseRequisition { .. } => "CurseRequisition",
            PatentedFunnel { .. } => "PatentedFunnel",
            OpeningNumber { .. } => "OpeningNumber",
            CursedLicence { .. } => "CursedLicence",
            EleventhSeason { .. } => "EleventhSeason",
            FullBill { .. } => "FullBill",
        }
    }

    /// A few words for a panel line. The whole sentence is [`Self::describe`].
    pub fn short(self) -> String {
        use ExpertPower::*;
        match self {
            BareFurnace { per_frame, .. } => format!("a bare frame is worth {} stacks at the bell", tenths(per_frame)),
            FiredFunnel { worth, .. } => format!("every stack the furnace buys is {worth} mana"),
            ColdStoke { per_stack, .. } => format!("a curse you land is {per_stack} stacks"),
            PonkeyBoiler { per_spin, .. } => format!("every spin turn is {} of a stack", tenths(per_spin)),
            FlashPowder { all_at_once, .. } => format!("{all_at_once}% of every pool burns before the first tick"),
            LoudDoubt { rate, .. } => format!("bare-handed, {}% of strength counts as mind damage", rate * 25),
            RequisitionedSilence { .. } => format!("a cast you cannot pay for is said anyway"),
            ToldOnce { per_curse, .. } => format!("every curse on them raises the unmaking by {per_curse}"),
            LicensedRumour { pct, .. } => format!("an enched activation eats {}% of their maximum", tenths(pct)),
            CurtainLine { mult, .. } => format!("an unmaking inside the window pays {mult}% more"),
            AshAndWhisper { rate, .. } => format!("{}% of what the furnace burns is also said", rate * 50),
            LoudCalculation { rate, .. } => {
                format!("strength buys mana at {} the point", tenths(rate))
            }
            StandingFact { worn, .. } => {
                format!("curses landed wearing {worn} items or fewer do not expire")
            }
            OverwoundArm { half, .. } => {
                format!("an empty frame spins anyway, {} a stack a turn", tenths(half))
            }
            ShortProgramme { pct, per_slot, .. } => {
                format!("+{pct}% for a fast win, {per_slot}s wider per empty frame")
            }
            CurseRequisition { per_fight, .. } => {
                format!("a curse pays for a cast, {per_fight} a fight")
            }
            PatentedFunnel { per_stack, .. } => {
                format!("the spin banks {per_stack} mana a stack a turn")
            }
            OpeningNumber { window_ms, .. } => {
                format!("casts cost nothing for {}s", window_ms / 1000)
            }
            CursedLicence { stack } => {
                format!("an enched component curses its frame, {stack} a hit")
            }
            EleventhSeason { pct, count_cap, .. } => {
                format!("+{pct}% a curse on the fallen, up to {count_cap}")
            }
            FullBill { racks, .. } => format!("{racks} enchs a component, off either list"),
        }
    }

    /// The whole promise, **read off the tuned numbers**.
    ///
    /// Unthemed, like every other mechanical line in this game: somebody
    /// choosing between two classes is comparing numbers, and a number wearing
    /// a joke has to be translated first. TONE rule 13a.
    ///
    /// Every clause a knob turns on is *absent* until the knob is moved, so a
    /// fresh expert's promise is short and a finished one's is long — which is
    /// the twelve points, stated.
    pub fn describe(self) -> String {
        use ExpertPower::*;
        match self {
            // ------------------------------------------------- M16's eleven
            BareFurnace { per_frame, every_ms, cap } => format!(
                "Every frame you left bare is a hopper that never empties: you walk into every \
                 fight with {} stacks of mana empowerment a bare frame, up to {cap}. The furnace \
                 runs every {:.1} seconds.",
                tenths(per_frame),
                every_ms as f32 / 1000.0
            ),
            FiredFunnel { worth, per_fight, refund } => format!(
                "Every stack the furnace buys is also {worth} mana, up to {per_fight} times a \
                 fight, and a cast refunds {refund}% of what it cost.",
            ),
            ColdStoke { per_stack, every_ms, standing } => format!(
                "Every curse you land is fuel: {per_stack} stacks of mana empowerment, and {} for \
                 one that cannot expire. The furnace runs every {:.1} seconds.",
                per_stack * (1 + standing.max(0)),
                every_ms as f32 / 1000.0
            ),
            PonkeyBoiler { per_spin, keep, licence } => format!(
                "Every turn a spinning item banks is {} of a stack of mana empowerment. A turning \
                 item keeps {keep} of its turns when it goes off, and you may hold {licence} \
                 enchs a component.",
                tenths(per_spin)
            ),
            FlashPowder { all_at_once, until_ms, pct } => format!(
                "{all_at_once}% of every pool you are holding is burned before the first tick. A \
                 fight won inside {:.0} seconds pays {pct}% more.",
                until_ms as f32 / 1000.0
            ),
            LoudDoubt { worn, rate, third } => format!(
                "Wearing {worn} items or fewer, {}% of your strength is added to every mind hit \
                 you land. Anything unmade at {third}% of its maximum health.",
                rate * 25
            ),
            RequisitionedSilence { rate, per_fight, third } => format!(
                "A cast you cannot pay for is said anyway, as mind damage worth {rate}% of what \
                 it would have cost, up to {per_fight} times a fight. Anything unmade at {third}% \
                 of its maximum health.",
            ),
            ToldOnce { per_curse, cap, standing } => format!(
                "Every curse standing on them raises the unmaking by {per_curse} points, up to \
                 {cap}, and one that cannot expire counts {}.",
                1 + standing.max(0)
            ),
            LicensedRumour { pct, racks, third } => format!(
                "Every activation of an enched component eats {}% of their maximum health. You \
                 may hold {racks} enchs a component, and anything is unmade at {third}% of its \
                 maximum.",
                tenths(pct)
            ),
            CurtainLine { mult, under_ms, third } => format!(
                "A creature unmade inside {:.0} seconds pays {mult}% more on top of whatever the \
                 bill already was. Anything unmade at {third}% of its maximum health.",
                under_ms as f32 / 1000.0
            ),
            AshAndWhisper { rate, third, every_ms } => format!(
                "Every point the furnace burns is also said: {}% of it lands as mind damage. The \
                 furnace runs every {:.1} seconds and anything is unmade at {third}% of its \
                 maximum health.",
                rate * 50,
                every_ms as f32 / 1000.0
            ),
            LoudCalculation { rate, cap, floor, rebate } => {
                let mut s = format!(
                    "A cast you cannot afford is paid for in strength, at {} strength a point \
                     of mana, up to {cap} points a fight. Borrowed strength comes back at \
                     the bell.",
                    tenths(rate)
                );
                if floor > 0 {
                    s.push_str(&format!(
                        " You may buy {floor} points past empty; strength never falls below 1."
                    ));
                }
                if rebate > 0 {
                    s.push_str(&format!(
                        " Every quarter of the enemy down refunds {rebate}% of the bill."
                    ));
                }
                s
            }
            StandingFact { worn, carry, bite, told } => {
                let mut s = format!(
                    "A curse you land while wearing {worn} finished items or fewer does not \
                     expire. {carry} may be standing at once."
                );
                if bite > 0 {
                    s.push_str(&format!(" A curse that cannot expire bites {bite}% harder."));
                }
                if told > 0 {
                    s.push_str(&format!(
                        " {told} of them follow you into the next fight and land before it acts."
                    ));
                }
                s
            }
            OverwoundArm { half, ceiling, carry } => {
                let mut s = format!(
                    "A frame with nothing in it turns anyway, banking {} of a stack a turn into \
                     your bare strength, up to {ceiling} stacks.",
                    tenths(half)
                );
                if carry > 0 {
                    s.push_str(&format!(" {carry} survive an activation."));
                }
                s
            }
            ShortProgramme { per_slot, floor_ms, pct, streak } => {
                let mut s = format!(
                    "A fight won inside {}s pays {pct}% more, and the window is {per_slot}s wider \
                     for every frame you left empty.",
                    floor_ms / 1000
                );
                if streak > 0 {
                    s.push_str(&format!(
                        " Each straight fast win adds {streak}s to the next window, up to {}; a \
                         slow one puts it back to nothing.",
                        Self::STREAK_CAP
                    ));
                }
                s
            }
            CurseRequisition { per_fight, worth, relist, pick } => {
                let mut s = format!(
                    "A cast may be paid for by consuming a curse you landed, {per_fight} a \
                     fight, each covering {worth}% of one."
                );
                s.push_str(if pick > 0 {
                    " The one with the least time left goes first."
                } else {
                    " The cheapest goes first."
                });
                if let Some(n) = Self::relist_every(relist) {
                    s.push_str(&format!(" Every {n}th comes back, still running."));
                }
                s
            }
            PatentedFunnel { per_stack, bleed, overflow, per_cell } => {
                let mut s =
                    format!("A turning item banks {per_stack} mana a stack a turn.");
                if bleed > 0 {
                    s.push_str(&format!(" {bleed} stacks survive an activation."));
                }
                if overflow > 0 {
                    s.push_str(" Mana past what you can hold lands as armour, 1 for 1.");
                }
                if per_cell > 0 {
                    s.push_str(&format!(
                        " A filled row pays {per_cell} more a cell at the bell."
                    ));
                }
                s
            }
            OpeningNumber { window_ms, after, encore, bank } => {
                let mut s =
                    format!("Casts cost nothing for the first {}s of a fight.", window_ms / 1000);
                if after > 0 {
                    // **The price, not the percentage.** A fifth off three is
                    // one, which is a third; printing the percentage would be
                    // a promise arrived at by a different sum from the one the
                    // fight does. TONE 13a — the number is the engine's.
                    s.push_str(&format!(
                        " After that a cast costs {} instead of {}.",
                        Self::cast_price(after),
                        crate::combat::SPELL_MANA_COST
                    ));
                }
                if encore > 0 {
                    s.push_str(&format!(
                        " Every quarter of the enemy down starts it again, {encore} time{}.",
                        if encore == 1 { "" } else { "s" }
                    ));
                }
                if bank > 0 {
                    s.push_str(" Mana unspent when it closes lands as armour.");
                }
                s
            }
            CursedLicence { stack } => format!(
                "A component with an ench on it lands that ench's curse on every activation of \
                 the item it is part of, {stack} at a time."
            ),
            EleventhSeason { pct, count_cap, distinct, posthumous } => {
                let mut s = format!(
                    "Each curse standing on what you beat adds {pct}% to the purse, counting up \
                     to {count_cap} of them."
                );
                if distinct > 0 {
                    s.push_str(" Four different kinds on one of them pays double.");
                }
                if posthumous > 0 {
                    s.push_str(&format!(
                        " A curse that expired before the bell counts {}.",
                        tenths(posthumous)
                    ));
                }
                s
            }
            FullBill { racks, beacon_pct } => {
                let mut s = format!(
                    "{racks} enchs may be bolted to one component, off either licence's list."
                );
                if beacon_pct > 0 {
                    s.push_str(&format!(
                        " An enched component lends {beacon_pct}% of each ench it carries to \
                         every finished item touching it in the same grid."
                    ));
                }
                s
            }
        }
    }

    /// How far a Short Programme streak may run.
    ///
    /// **A constant and not a knob**, because nothing sells it: a runaway is
    /// what a cap is for, and a tree that could lift its own cap would be a
    /// tree selling the absence of a limit. `PLAN-M13.md` §3.4 names 5.
    pub const STREAK_CAP: i32 = 5;
}

/// One expert: the pair that reaches it, and what it is.
#[derive(Copy, Clone, Debug)]
pub struct ExpertDef {
    /// The two canonical class names, in the order the roster is written.
    /// Looked up **order-insensitively** — see [`for_pair`].
    pub pair: (&'static str, &'static str),
    /// Canonical name, matching the `class` key of its tree.
    pub name: &'static str,
    pub blurb: &'static str,
    pub power: ExpertPower,
}

/// The twenty-one, one per pair of the seven offered classes.
///
/// **`C(7,2)` and no more.** `every_pair_of_offered_classes_reaches_an_expert`
/// is what says the table is complete and holds no pair twice — a list of ten
/// written by hand is a list that can be nine.
pub static EXPERTS: &[ExpertDef] = &[
    ExpertDef {
        pair: ("Berserker", "Hexweaver"),
        name: "LoudCalculation",
        blurb: "The Sergeant kept shouting figures and the Gorillathon kept paying them.",
        power: ExpertPower::LoudCalculation { rate: 20, cap: 40, floor: 0, rebate: 0 },
    },
    ExpertDef {
        pair: ("Berserker", "Bloodletter"),
        name: "StandingFact",
        blurb: "You wore almost nothing and said something true, and it stayed said.",
        power: ExpertPower::StandingFact { worn: 2, carry: 1, bite: 0, told: 0 },
    },
    ExpertDef {
        pair: ("Berserker", "Recycler"),
        name: "OverwoundArm",
        blurb: "An empty frame, wound past where it stops, and nobody willing to stand near it.",
        power: ExpertPower::OverwoundArm { half: 5, ceiling: 8, carry: 0 },
    },
    ExpertDef {
        pair: ("Berserker", "Showstopper"),
        name: "ShortProgramme",
        blurb: "Billed at eight, finished by nine, and the hall still filling up.",
        power: ExpertPower::ShortProgramme { per_slot: 1, floor_ms: 10_000, pct: 50, streak: 0 },
    },
    ExpertDef {
        pair: ("Hexweaver", "Bloodletter"),
        name: "CurseRequisition",
        blurb: "A form, in triplicate, for spending something that was already going bad.",
        power: ExpertPower::CurseRequisition { per_fight: 3, worth: 100, relist: 0, pick: 0 },
    },
    ExpertDef {
        pair: ("Hexweaver", "Recycler"),
        name: "PatentedFunnel",
        blurb: "The funnel turns, and Kaklon has the paperwork saying it may.",
        power: ExpertPower::PatentedFunnel { per_stack: 5, bleed: 0, overflow: 0, per_cell: 0 },
    },
    ExpertDef {
        pair: ("Hexweaver", "Showstopper"),
        name: "OpeningNumber",
        blurb: "Nobody is charged for the overture. Nobody has ever asked why.",
        power: ExpertPower::OpeningNumber { window_ms: 10_000, after: 0, encore: 0, bank: 0 },
    },
    ExpertDef {
        pair: ("Bloodletter", "Recycler"),
        name: "CursedLicence",
        blurb: "Page nine of the licence, which nobody reads, permits this.",
        power: ExpertPower::CursedLicence { stack: 1 },
    },
    ExpertDef {
        pair: ("Bloodletter", "Showstopper"),
        name: "EleventhSeason",
        blurb: "Eleven seasons of it, and the billing department has stopped asking questions.",
        power: ExpertPower::EleventhSeason { pct: 10, count_cap: 4, distinct: 0, posthumous: 0 },
    },
    // ---- M16's eleven, which are what `C(7, 2)` costs ----------------------
    //
    // **Twenty-one, and the eleven are not a wish list.** Two classes on the
    // fork is eleven new pairs, and a pair without a counter is a player who
    // finishes two trees and is handed nothing —
    // `every_pair_of_offered_classes_reaches_an_expert` went red on the commit
    // that made the roster seven and is what says the table is complete.
    ExpertDef {
        pair: ("Berserker", "Stoker"),
        name: "BareFurnace",
        blurb: "An empty frame is a hopper, and nobody has ever explained why that works.",
        power: ExpertPower::BareFurnace { per_frame: 10, every_ms: 4_000, cap: 1 },
    },
    ExpertDef {
        pair: ("Hexweaver", "Stoker"),
        name: "FiredFunnel",
        blurb: "The funnel turns over the firebox and the paperwork says this is a kettle.",
        power: ExpertPower::FiredFunnel { worth: 4, per_fight: 1, refund: 0 },
    },
    ExpertDef {
        pair: ("Bloodletter", "Stoker"),
        name: "ColdStoke",
        blurb: "Nine facts, all of them true, all of them on fire.",
        power: ExpertPower::ColdStoke { per_stack: 1, every_ms: 4_000, standing: 0 },
    },
    ExpertDef {
        pair: ("Recycler", "Stoker"),
        name: "PonkeyBoiler",
        blurb: "The Ponkey Turn, geared down eleven to one, into a firebox.",
        power: ExpertPower::PonkeyBoiler { per_spin: 2, keep: 0, licence: 1 },
    },
    ExpertDef {
        pair: ("Showstopper", "Stoker"),
        name: "FlashPowder",
        blurb: "Everything you had, at once, in front of four thousand people.",
        power: ExpertPower::FlashPowder { all_at_once: 25, until_ms: 10_000, pct: 0 },
    },
    ExpertDef {
        pair: ("Berserker", "Whisperer"),
        name: "LoudDoubt",
        blurb: "A gorilla wearing nothing, saying one thing, very quietly, four times.",
        power: ExpertPower::LoudDoubt { worn: 2, rate: 1, third: 33 },
    },
    ExpertDef {
        pair: ("Hexweaver", "Whisperer"),
        name: "RequisitionedSilence",
        blurb: "Requisition denied. The Sergeant said it anyway and filed the form after.",
        power: ExpertPower::RequisitionedSilence { rate: 50, per_fight: 2, third: 33 },
    },
    ExpertDef {
        pair: ("Bloodletter", "Whisperer"),
        name: "ToldOnce",
        blurb: "Every fact you put on it is a thing it now has to carry.",
        power: ExpertPower::ToldOnce { per_curse: 1, cap: 3, standing: 0 },
    },
    ExpertDef {
        pair: ("Recycler", "Whisperer"),
        name: "LicensedRumour",
        blurb: "Page eleven of the licence permits saying things about people.",
        power: ExpertPower::LicensedRumour { pct: 5, racks: 1, third: 33 },
    },
    ExpertDef {
        pair: ("Showstopper", "Whisperer"),
        name: "CurtainLine",
        blurb: "The last line of the last act, and the hall is already standing.",
        power: ExpertPower::CurtainLine { mult: 50, under_ms: 10_000, third: 33 },
    },
    ExpertDef {
        pair: ("Stoker", "Whisperer"),
        name: "AshAndWhisper",
        blurb: "Ash in one ear and a word in the other, and nobody can tell which did it.",
        power: ExpertPower::AshAndWhisper { rate: 1, third: 33, every_ms: 4_000 },
    },
    ExpertDef {
        pair: ("Recycler", "Showstopper"),
        name: "FullBill",
        blurb: "Both licences on one counter, and a component with no room left on it.",
        power: ExpertPower::FullBill { racks: 2, beacon_pct: 0 },
    },
];

/// The expert a pair of classes reaches, in either order.
///
/// **Order-insensitive**, because which class you took first is a fact about
/// your afternoon and not about what the pair is. A player who forked into the
/// Gorillathon and bought the Funnel Sergeant reaches the same counter as one
/// who did it the other way round, and a table keyed on order would be twenty
/// rows of which ten were duplicates.
pub fn for_pair(a: &str, b: &str) -> Option<&'static ExpertDef> {
    if a == b {
        return None;
    }
    EXPERTS.iter().find(|e| {
        (e.pair.0 == a && e.pair.1 == b) || (e.pair.0 == b && e.pair.1 == a)
    })
}

/// Is this canonical name one of the ten?
///
/// Read by `class::is_earned`, which is what keeps an expert out of every
/// ranking: nothing you build points at one, you go and finish two trees.
pub fn is_expert(name: &str) -> bool {
    EXPERTS.iter().any(|e| e.name == name)
}

/// The definition by canonical name.
pub fn by_name(name: &str) -> Option<&'static ExpertDef> {
    EXPERTS.iter().find(|e| e.name == name)
}
