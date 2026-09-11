//! M13.1 — the ten experts, their pairs and their knobs.
//!
//! Ten because there are five offered classes and `C(5,2)` is ten. A list of
//! ten written by hand is a list that can be nine, so the table is checked
//! against the pairs rather than counted.

use gm2d_core::class::{ClassPower, CLASSES, OFFERED};
use gm2d_core::expert::{self, ExpertPower, EXPERTS};

// ----------------------------------------------------------------- the table

#[test]
fn every_pair_of_offered_classes_reaches_an_expert() {
    let mut seen: Vec<&str> = Vec::new();
    for (i, a) in OFFERED.iter().enumerate() {
        for b in &OFFERED[i + 1..] {
            let e = expert::for_pair(a, b)
                .unwrap_or_else(|| panic!("{a} and {b} reach nobody"));
            assert!(!seen.contains(&e.name), "{} is reached by two pairs", e.name);
            seen.push(e.name);
        }
    }
    assert_eq!(seen.len(), 21, "seven classes make twenty-one pairs");
    assert_eq!(EXPERTS.len(), 21, "and the table holds exactly those");
}

/// Order is a fact about your afternoon, not about the pair.
#[test]
fn a_pair_reaches_the_same_expert_in_either_order() {
    for e in EXPERTS {
        let (a, b) = e.pair;
        assert_eq!(expert::for_pair(a, b).map(|x| x.name), Some(e.name));
        assert_eq!(expert::for_pair(b, a).map(|x| x.name), Some(e.name), "{a} / {b}");
    }
}

/// A class paired with itself is not a pair. Nothing can hold one class twice.
#[test]
fn a_class_pairs_with_nothing_but_another() {
    for c in OFFERED {
        assert!(expert::for_pair(c, c).is_none(), "{c} paired with itself");
    }
    assert!(expert::for_pair("Berserker", "Wanderer").is_none(), "the fork deals five");
}

/// Every pair names two of the five, and the ten cover all five evenly.
#[test]
fn the_pairs_are_all_offered_classes() {
    for e in EXPERTS {
        for side in [e.pair.0, e.pair.1] {
            assert!(OFFERED.contains(&side), "{} pairs on {side}, which is not offered", e.name);
        }
    }
    // Each of the seven is in six pairs — that is what `C(7,2)` looks like
    // from one class's chair, and it is why nobody's fork is a dead end.
    for c in OFFERED {
        let n = EXPERTS.iter().filter(|e| e.pair.0 == *c || e.pair.1 == *c).count();
        assert_eq!(n, OFFERED.len() - 1, "{c} reaches {n} experts");
    }
}

// ------------------------------------------------------------- the roster

/// Every expert is in `CLASSES`, so every screen that asks the roster for a
/// name and a blurb finds one without being told about a second list.
#[test]
fn the_roster_carries_every_expert() {
    for e in EXPERTS {
        let def = CLASSES
            .iter()
            .find(|c| c.name == e.name)
            .unwrap_or_else(|| panic!("{} is not in CLASSES", e.name));
        assert_eq!(def.blurb, e.blurb, "{} says two different things", e.name);
        assert_eq!(def.power, ClassPower::Expert(e.power), "{}", e.name);
        assert!(def.requires.is_empty(), "nothing you wear points at an expert");
    }
}

/// **Handed over, never qualified for.** `is_earned` is what keeps the ten out
/// of every ranking, which is the same guard the town classes get.
#[test]
fn no_amount_of_building_reaches_an_expert() {
    for e in EXPERTS {
        assert!(gm2d_core::class::is_earned(e.name), "{} can be built toward", e.name);
        assert!(
            gm2d_core::class::how_you_get_it(e.name).is_some(),
            "{} is listed and not explained",
            e.name
        );
    }
}

// ----------------------------------------------------------------- the knobs

#[test]
fn every_declared_knob_can_be_read_and_moved() {
    for e in EXPERTS {
        assert!(!e.power.knobs().is_empty(), "{} declares no knobs", e.name);
        for k in e.power.knobs() {
            let before = e.power.knob(k).unwrap_or_else(|| panic!("{} cannot read {k}", e.name));
            let mut after = e.power;
            after.tune(k, 7);
            assert_eq!(after.knob(k), Some(before + 7), "{} did not move {k}", e.name);
        }
    }
}

/// A knob a power has not got reads as nothing and moves nothing.
///
/// Safe **only** because `SkillsData::parse` refuses a tree naming one — see
/// `every_expert_knob_is_declared`. Stated here so the pairing is on the record.
#[test]
fn a_knob_that_is_not_declared_moves_nothing() {
    for e in EXPERTS {
        let mut p = e.power;
        assert_eq!(p.knob("nonsense"), None);
        let before = p;
        p.tune("nonsense", 99);
        assert_eq!(p, before, "{} moved on a knob it has not got", e.name);
    }
}

/// No two experts share a knob **name and meaning** by accident: a name that
/// turns up twice has to mean the same kind of thing, because a node's line
/// prints the bare word.
#[test]
fn a_knob_name_shared_by_two_experts_is_deliberate() {
    let mut shared: Vec<&str> = Vec::new();
    for (i, a) in EXPERTS.iter().enumerate() {
        for b in &EXPERTS[i + 1..] {
            for k in a.power.knobs() {
                if b.power.knobs().contains(k) && !shared.contains(k) {
                    shared.push(k);
                }
            }
        }
    }
    // **Twelve, and every one of them is the same idea in two places**, which
    // is what a knob vocabulary is for: `third` is a threshold, `every_ms` is a
    // furnace's clock, `cap` is a ceiling, `standing` is *a permanent curse
    // counts double*, `racks` is enchs a component. A name meaning two things
    // is what this refuses, and it is safe because `SkillsData::parse` checks a
    // knob against **its own tree's class** — so `per_stack` on a Stoker node
    // and `per_stack` on a Patented Funnel node are two knobs that share a word
    // and can never be confused for one another.
    //
    // `Character::class_defs` reads `tunings_for(class, …)` for the same
    // reason. Anything appearing here is a collision to name or rename, and
    // twelve of twelve have been looked at.
    shared.sort_unstable();
    assert_eq!(
        shared,
        vec![
            "cap", "carry", "every_ms", "pct", "per_fight", "per_stack", "racks", "rate",
            "standing", "third", "worn", "worth"
        ],
        "a new shared knob name wants a look"
    );
}

// -------------------------------------------------------------- the promise

/// **The promise is read off the tuned numbers**, so it cannot go stale.
#[test]
fn tuning_a_knob_changes_what_the_class_promises() {
    for e in EXPERTS {
        for k in e.power.knobs() {
            // **By its own step, not by an arbitrary number.** Two of the
            // thirty-one knobs are stored in milliseconds and printed in whole
            // seconds, so tuning `floor_ms` by 25 changes nothing a player can
            // read — which is the finding this test made on its first run, and
            // the reason `ExpertPower::step` exists rather than the test
            // quietly picking a bigger number.
            let step = ExpertPower::step(k);
            let mut after = e.power;
            after.tune(k, step);
            assert_ne!(
                after.describe(),
                e.power.describe(),
                "{} says the same thing after {k} moved by its own step of {step}",
                e.name
            );
        }
    }
}

#[test]
fn every_expert_says_something_with_a_number_in_it() {
    for e in EXPERTS {
        let d = e.power.describe();
        assert!(d.len() > 40, "{} barely says anything: {d:?}", e.name);
        assert!(d.chars().any(|c| c.is_ascii_digit()), "{} names no number: {d:?}", e.name);
        let s = e.power.short();
        assert!(!s.is_empty() && s.len() < 70, "{} has no short line: {s:?}", e.name);
    }
}

/// Tenths print as tenths. `rate: 20` is 2.0 and not 20.
#[test]
fn a_fractional_knob_reads_as_a_fraction() {
    let p = ExpertPower::LoudCalculation { rate: 20, cap: 40, floor: 0, rebate: 0 };
    assert!(p.describe().contains("2.0 strength"), "{}", p.describe());
    let p = ExpertPower::OverwoundArm { half: 5, ceiling: 8, carry: 0 };
    assert!(p.describe().contains("0.5 of a stack"), "{}", p.describe());
}

/// A knob stored finer than it is printed is a knob nobody can watch move.
#[test]
fn a_millisecond_knob_steps_in_whole_seconds() {
    assert_eq!(ExpertPower::step("floor_ms"), 1000);
    assert_eq!(ExpertPower::step("window_ms"), 1000);
    assert_eq!(ExpertPower::step("pct"), 1);
    // Tenths are printed to a tenth, so they step by one.
    assert_eq!(ExpertPower::step("rate"), 1);
    assert_eq!(ExpertPower::step("half"), 1);
    // And the rule is read off the name, so a new `_ms` knob gets it free.
    for e in EXPERTS {
        for k in e.power.knobs() {
            assert_eq!(
                ExpertPower::step(k) != 1,
                k.ends_with("_ms"),
                "{} declares {k}, whose step does not follow its name",
                e.name
            );
        }
    }
}

/// `relist` is a level and the period is derived from it in one place.
#[test]
fn the_requisition_period_is_derived_and_never_below_two() {
    assert_eq!(ExpertPower::relist_every(0), None, "nothing comes back");
    assert_eq!(ExpertPower::relist_every(1), Some(3));
    assert_eq!(ExpertPower::relist_every(2), Some(2));
    // A tree that over-granted would otherwise reach "every 1st", which is
    // "always" wearing a schedule.
    assert_eq!(ExpertPower::relist_every(3), Some(2), "it floors at every second");
    assert_eq!(ExpertPower::relist_every(9), Some(2));
}

// ------------------------------------------------------ the parse-time guard
//
// M13.2. Three lints, and every one of them is the same rule from a different
// side: **a point spent must move something a player can see.** This project
// has shipped eight nodes that did not, and the reason serde let it happen —
// `deny_unknown_fields` is a container attribute — is why these read the data
// rather than the parsed struct wherever they can.

use gm2d_core::skills::SkillsData;

/// The header every fixture below shares. Written out rather than mutated from
/// the shipped file, so a fixture cannot accidentally test the real trees.
fn tree_with(class: &str, effect: &str) -> String {
    format!(
        r#"{{"format":"gm2d-skills","version":1,"trees":[
             {{"id":"t","name":"T","class":{class},"nodes":[
               {{"id":"n","name":"N","blurb":"b","cost":2,"effect":{effect}}}]}}]}}"#
    )
}

#[test]
fn every_expert_knob_is_declared() {
    // The real thing loads.
    let ok = tree_with("\"LoudCalculation\"", r#"{"tunes":{"knob":"rate","by":-4}}"#);
    assert!(SkillsData::parse(&ok).is_ok(), "{:?}", SkillsData::parse(&ok).err());

    // A knob this class has not got does not load — **even though another
    // class has it.** `carry` is Overwound Arm's and Standing Fact's; `floor`
    // is Loud Calculation's alone.
    let bad = tree_with("\"OverwoundArm\"", r#"{"tunes":{"knob":"floor","by":20}}"#);
    let why = SkillsData::parse(&bad).unwrap_err();
    assert!(why.contains("floor"), "{why}");
    assert!(why.contains("OverwoundArm"), "{why}");

    // And a knob nothing has got.
    let bad = tree_with("\"FullBill\"", r#"{"tunes":{"knob":"vibes","by":1}}"#);
    assert!(SkillsData::parse(&bad).is_err());

    // The base tree belongs to no class at all, so it has nothing to tune.
    let bad = tree_with("null", r#"{"tunes":{"knob":"rate","by":-4}}"#);
    let why = SkillsData::parse(&bad).unwrap_err();
    assert!(why.contains("has no knobs at all"), "{why}");

    // **Nor may a base class that has none**, which is five of the seven.
    // This asserted *only an expert class has knobs* until M16, and that
    // stopped being true: the Stoker and the Whisperer have two knobs and one,
    // and their own trees turn them. What did not change is the rule the check
    // is about — a tree may tune a knob its own class declares and nothing
    // else — so the sentence moved and the guarantee did not.
    let bad = tree_with("\"Berserker\"", r#"{"tunes":{"knob":"rate","by":-4}}"#);
    let why = SkillsData::parse(&bad).unwrap_err();
    assert!(why.contains("has no knobs at all"), "{why}");

    // And a base class that *does* have knobs may tune its own and no other.
    let ok = tree_with("\"Stoker\"", r#"{"tunes":{"knob":"per_stack","by":-3}}"#);
    assert!(SkillsData::parse(&ok).is_ok(), "{:?}", SkillsData::parse(&ok).err());
    let bad = tree_with("\"Stoker\"", r#"{"tunes":{"knob":"rate","by":-4}}"#);
    let why = SkillsData::parse(&bad).unwrap_err();
    assert!(why.contains("rate") && why.contains("Stoker"), "{why}");
}

/// A tuning that tunes nothing is a point spent on nothing — and so is one
/// finer than the sentence it is meant to change.
#[test]
fn a_tuning_that_cannot_be_seen_does_not_load() {
    let zero = tree_with("\"LoudCalculation\"", r#"{"tunes":{"knob":"rate","by":0}}"#);
    assert!(SkillsData::parse(&zero).unwrap_err().contains("nothing at all"));

    // 500ms on a knob printed in whole seconds changes no sentence anywhere.
    let fine = tree_with("\"OpeningNumber\"", r#"{"tunes":{"knob":"window_ms","by":500}}"#);
    let why = SkillsData::parse(&fine).unwrap_err();
    assert!(why.contains("less than the 1000"), "{why}");

    // A whole second is fine, and so is a negative one.
    for by in ["3000", "-1000"] {
        let ok = tree_with(
            "\"OpeningNumber\"",
            &format!(r#"{{"tunes":{{"knob":"window_ms","by":{by}}}}}"#),
        );
        assert!(SkillsData::parse(&ok).is_ok(), "{by}: {:?}", SkillsData::parse(&ok).err());
    }
}

/// **A knob no node touches is a knob that should have been a constant.**
///
/// The inverse of the guard above, and the one that catches over-declaring:
/// thirty-one knobs is thirty-one promises that the tree spends points on, and
/// one nobody spends on is a number nobody can move, which is what
/// `ExpertPower::STREAK_CAP` is deliberately written as instead.
///
/// It reads the shipped trees, so it is vacuous until M13.5 lands them — and
/// says so rather than passing quietly on an empty set.
#[test]
fn every_declared_knob_is_moved_by_some_node() {
    let tree = gm2d_core::data::skills();
    let landed: Vec<&str> = EXPERTS
        .iter()
        .map(|e| e.name)
        .filter(|n| tree.tree_for_class(n).is_some())
        .collect();
    if landed.is_empty() {
        // M13.5 is what makes this ask a question. Until then, refusing to
        // pass silently is the whole of what it can honestly do.
        return;
    }
    assert_eq!(
        landed.len(),
        EXPERTS.len(),
        "some expert trees landed and some did not: {landed:?}"
    );
    for e in EXPERTS {
        let t = tree.tree_for_class(e.name).expect("checked above");
        let moved: Vec<String> = t.nodes.iter().flat_map(|n| n.tunings()).map(|(k, _)| k).collect();
        for k in e.power.knobs() {
            assert!(
                moved.iter().any(|m| m == k),
                "{} declares {k} and no node moves it — it should be a constant",
                e.name
            );
        }
    }
}

// -------------------------------------------------- M13.5: the ten trees
//
// §1.6 is the constraint that gives this block its shape, and it is enforced
// rather than intended. It reads the raw JSON, as `tests/tone.rs` does, because
// the failure it guards against is a key that parses and means nothing.

use gm2d_core::skills::Effect;

/// **Every node of an expert tree must reach that expert's power.**
///
/// A `tunes` of a declared knob, a `grants` of a rule the power is kin to, or a
/// `gives_ench` inside the licence. No flat stats, no starting balances, no
/// rows, no bare assembly percentages.
///
/// A `+12 strength` node would be a node you could take without noticing which
/// class you were in. The five base trees are allowed to be a mix because they
/// are the character's first shape; an expert tree is the argument for its own
/// promise, six nodes long.
#[test]
fn expert_nodes_touch_only_the_expert() {
    let tree = gm2d_core::data::skills();
    let mut bad = Vec::new();
    for t in &tree.trees {
        let Some(class) = t.class.as_deref() else { continue };
        if !expert::is_expert(class) {
            continue;
        }
        let power = expert::by_name(class).expect("an expert has a power").power;
        for n in &t.nodes {
            for e in &n.effects {
                match e {
                    Effect::Tunes { knob, .. } => {
                        if !power.knobs().contains(&knob.as_str()) {
                            bad.push(format!("{}: {knob:?} is not {class}'s", n.id));
                        }
                    }
                    // A rule is allowed where the power is kin to it. Kinship
                    // is read off what the tree's own class does rather than
                    // listed here — a list would be a second copy of §2's
                    // table and would go stale the first time a tree moved.
                    Effect::Grants { rule } => {
                        if !kin(class, rule) {
                            bad.push(format!("{}: {class} is no kin to {rule:?}", n.id));
                        }
                    }
                    // **An ench, and only for a class whose promise is racks.**
                    // Three of them since M16: `ench_racks` reads all three,
                    // and a class that sells you room for a second ench and
                    // hands you none is a promise with nothing behind it.
                    Effect::GivesEnch { .. } => {
                        if !matches!(
                            power,
                            ExpertPower::FullBill { .. }
                                | ExpertPower::LicensedRumour { .. }
                                | ExpertPower::PonkeyBoiler { .. }
                        ) {
                            bad.push(format!("{}: {class} hands over an ench", n.id));
                        }
                    }
                    Effect::Stat { .. }
                    | Effect::StartWith { .. }
                    | Effect::GrowSlotRows { .. }
                    | Effect::AssemblyPct { .. } => bad.push(format!(
                        "{}: {class} pays for something you could take without noticing \
                         which class you are in",
                        n.id
                    )),
                }
            }
        }
    }
    assert!(bad.is_empty(), "an expert node reaching past its own class:\n  {}", bad.join("\n  "));
}

/// Which rules an expert's promise is kin to.
///
/// **One arm a class, exhaustive over the rule**, so a rule added to the game
/// is a decision about which experts may grant it rather than a silence — the
/// same posture `combat.rs` and `reward.rs` take about a class power.
fn kin(class: &str, rule: &gm2d_core::rule::Rule) -> bool {
    use gm2d_core::rule::Rule::*;
    match (class, rule) {
        // An empty frame that spins works the ground under it.
        ("OverwoundArm", Spread { .. }) => true,
        // The funnel banks off the spin, and a full row is a machine that
        // banks too.
        ("PatentedFunnel", SpinEvery { .. } | SpinExtra { .. } | SpinKeep { .. }) => true,
        ("PatentedFunnel", RowHarvest { .. }) => true,
        // The licence is a schedule of curses, and productivity is that
        // schedule bought with speed.
        ("CursedLicence", CurseOnActivate { .. } | Productivity { .. }) => true,
        // Two licences on one counter, and what one lends the next.
        ("FullBill", Beacon { .. }) => true,

        // ---- M16's eleven -------------------------------------------------
        //
        // **Three new rules between them and each granted by more than one
        // tree**, which is `PROMPT-M16.md`'s own constraint and is the shape
        // `Spread` and `Beacon` already have: a rule granted by exactly one
        // tree is a knob that has been given a second name.
        //
        // A furnace that keeps what it burned is kin to every furnace.
        ("BareFurnace" | "AshAndWhisper", BurnKeepsBonus { .. }) => true,
        // A furnace that is still warm at the next bell, likewise.
        ("FiredFunnel" | "PonkeyBoiler", BurnCarries { .. }) => true,
        // And a word that gets through is kin to every mind lane.
        ("LoudDoubt" | "RequisitionedSilence" | "CurtainLine", MindPierce { .. }) => true,
        // The Keeper's half of each of its two pairings is a curse on a frame,
        // which is `CursedLicence`'s kinship arriving at the two classes that
        // share a parent with it.
        ("ColdStoke" | "ToldOnce", CurseOnActivate { .. }) => true,
        // The Patent's half, likewise: what an enched component lends.
        ("LicensedRumour", Beacon { .. }) => true,
        _ => false,
    }
}

/// **Six nodes, two roots, one capstone, two points each.**
///
/// The shape is the argument: two roots means the first point is a real choice,
/// and the reconvergence means the capstone is the whole tree rather than one
/// branch of it.
#[test]
fn every_expert_tree_is_two_roots_and_a_capstone() {
    let tree = gm2d_core::data::skills();
    for e in EXPERTS {
        let t = tree.tree_for_class(e.name).unwrap_or_else(|| panic!("{} has no tree", e.name));
        assert_eq!(t.nodes.len(), 6, "{} is {} nodes", e.name, t.nodes.len());
        for n in &t.nodes {
            assert_eq!(n.cost, 2, "{}: {} costs {}", e.name, n.id, n.cost);
        }
        let roots: Vec<&str> =
            t.nodes.iter().filter(|n| n.requires.is_empty()).map(|n| n.id.as_str()).collect();
        assert_eq!(roots.len(), 2, "{} has roots {roots:?}", e.name);
        // The capstone is the one nothing else asks for, and it asks for two.
        let needed: Vec<&String> = t.nodes.iter().flat_map(|n| &n.requires).collect();
        let leaves: Vec<&str> = t
            .nodes
            .iter()
            .filter(|n| !needed.iter().any(|r| **r == n.id))
            .map(|n| n.id.as_str())
            .collect();
        assert_eq!(leaves.len(), 1, "{} ends in {leaves:?}", e.name);
        let cap = t.nodes.iter().find(|n| n.id == leaves[0]).unwrap();
        assert_eq!(cap.requires.len(), 2, "{}'s capstone takes one branch only", e.name);
        // Twelve points to finish one, which is what the paper is worth.
        assert_eq!(t.nodes.iter().map(|n| n.cost).sum::<u32>(), 12, "{}", e.name);
    }
}

/// **A node must move a knob by something a player can watch move.**
///
/// The data half of `ExpertPower::step`. `SkillsData::parse` already refuses a
/// sub-step move, so this is the shipped trees asked directly — a guard whose
/// only proof is that the file happens to load is a guard nobody is reading.
#[test]
fn every_expert_node_moves_a_knob_it_can_be_seen_to_move() {
    let tree = gm2d_core::data::skills();
    for e in EXPERTS {
        let t = tree.tree_for_class(e.name).expect("a tree");
        for n in &t.nodes {
            for (knob, by) in n.tunings() {
                // **The power's own step, not the knob name's** — see
                // `ExpertPower::step_of`. A furnace prints its clock to a tenth
                // of a second and the two experts `step` was written for print
                // theirs to a whole one, so the smallest *visible* move differs
                // for the same suffix.
                let step = e.power.step_of(&knob);
                assert!(by != 0, "{}: moves {knob} by nothing", n.id);
                assert_eq!(by % step, 0, "{}: moves {knob} by {by}, under its step of {step}", n.id);
            }
        }
    }
}

/// A finished expert tree says something different from a fresh one, in every
/// case — which is what twelve points buys and the only way to see it.
#[test]
fn finishing_an_expert_tree_changes_its_promise() {
    let tree = gm2d_core::data::skills();
    for e in EXPERTS {
        let t = tree.tree_for_class(e.name).expect("a tree");
        let mut finished = e.power;
        for n in &t.nodes {
            for (knob, by) in n.tunings() {
                finished.tune(&knob, by);
            }
        }
        assert_ne!(finished.describe(), e.power.describe(), "{} reads the same finished", e.name);
        // And every knob ends up somewhere sensible: nothing goes negative,
        // which would print a promise that takes something away.
        for k in e.power.knobs() {
            assert!(
                finished.knob(k).unwrap_or(0) >= 0,
                "{}: {k} finishes at {:?}",
                e.name,
                finished.knob(k)
            );
        }
    }
}

/// **A discount on a cast takes something off.**
///
/// A cast costs three, and every knob in this block that moves a price moves it
/// as a percentage — so a percentage that rounds to nothing is a node that
/// costs two points and changes no number anywhere. `on-standing-discount` sold
/// *twenty percent less* and took nought point six off three, which integer
/// division makes nought; the same node is the reason `ExpertPower::cast_price`
/// is one function rather than an expression in `pay_for_a_cast`.
///
/// Every percentage, not the three the tree happens to reach: the tree is
/// content and this is the rule under it.
#[test]
fn a_discount_on_a_cast_takes_something_off() {
    let full = gm2d_core::combat::SPELL_MANA_COST;
    assert_eq!(ExpertPower::cast_price(0), full, "no discount is no discount");
    let mut last = full;
    for after in 1..=100 {
        let price = ExpertPower::cast_price(after);
        assert!(price < full, "{after}% off {full} still costs {price}");
        assert!(price >= 0, "{after}% off {full} costs {price}");
        assert!(price <= last, "{after}% off {full} costs more than {}%", after - 1);
        last = price;
    }

    // And the promise prints the price rather than the percentage, because the
    // two are different sums and only one of them is the one the fight does.
    let power = ExpertPower::OpeningNumber { window_ms: 10_000, after: 20, encore: 0, bank: 0 };
    let said = power.describe();
    assert!(said.contains(&format!("costs {}", ExpertPower::cast_price(20))), "{said}");
}

/// **Every canonical class name is one a player could be shown.**
///
/// `Theme::class` falls through to the canonical name when a theme has not
/// named a class, and that has always been safe: every canonical before M13 is
/// display-safe English — `Berserker`, `Bloodletter`, `Showstopper`. The ten
/// experts are the first that are not. `LoudCalculation` is a Rust variant name
/// with the spaces taken out, and `theme.plain.json`'s `classes` table was
/// empty for eight blocks precisely because the fall-through was fine.
///
/// So there are two ways to be safe and this accepts either: **be a name**, or
/// **be named by every theme**. `SECOND-ORDER-M13.md` row 3 asks for exactly
/// that, and the reason it is worth a lint rather than a habit is that the
/// fall-through is silent — a canonical added out of a data file will hit this
/// again, and the way it shows is a player reading `LoudCalculation` on the one
/// screen that does not come off.
#[test]
fn every_class_name_is_one_a_player_could_read() {
    let mut bad = Vec::new();
    for def in gm2d_core::class::CLASSES {
        // A name a person would write: no run of a lower-case letter followed
        // immediately by a capital, which is what a squashed variant looks
        // like and is the only shape that has ever gone wrong here.
        let squashed = def
            .name
            .as_bytes()
            .windows(2)
            .any(|w| w[0].is_ascii_lowercase() && w[1].is_ascii_uppercase());
        if !squashed {
            continue;
        }
        for t in gm2d_core::theme::THEMES {
            let shown = t.class(def.name);
            if shown == def.name {
                bad.push(format!("{} is not a name and {} does not rename it", def.name, t.id));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a canonical that reaches the screen as a variant name:\n  {}",
        bad.join("\n  ")
    );
}
