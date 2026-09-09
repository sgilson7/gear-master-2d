//! M13.6 — every expert reaches something, and so does every knob.
//!
//! **Called, not declared.** The first version of
//! `every_offered_class_reaches_something` matched the variant and named where
//! the power was honoured, which a stubbed payout passed cleanly — *a lint that
//! reads a list rather than the behaviour is the failure it exists to catch,
//! one level up*. So everything here runs a fight, or settles one, or builds a
//! board, and compares.
//!
//! And it goes one level further than that lint had to. An expert carries up to
//! four **knobs**, and a knob nothing reads is a promise the tree sells twelve
//! points of. `every_declared_knob_changes_a_fight` is the guard, and it is the
//! strongest one in the block: thirty-one numbers, each moved on its own, each
//! having to change something a player could see.
//!
//! # The fixture is most of the work, and it has to be
//!
//! The first draft asked all ten against one packed Auto-packed board and
//! reported that seven of them reached nothing. They did — *on that board*: it
//! has no empty frame for an Overwound Arm to turn, fifty-eight finished items
//! where a Standing Fact wants two, nothing that spins, and it wins in two and
//! a half seconds. **A check that needs something to happen has to make sure it
//! can**, which this project wrote down after a broken-item check walked into
//! whatever the ground rolled. So there are three boards here and each poses
//! the question its experts are about.

mod common;

use std::collections::HashMap;

use gm2d_core::character::Character;
use gm2d_core::class::{ClassDef, ClassPower, CLASSES};
use gm2d_core::combat::{self, Difficulty, MonsterSpec, Outcome};
use gm2d_core::expert::{self, ExpertPower, EXPERTS};
use gm2d_core::piece::SlotKind;
use gm2d_core::reward;

const D: Difficulty = Difficulty::Medium;

/// **Three fights and not one**, for the reason the purse is swept rather than
/// settled once: half the knobs in this block are *thresholds*, and a threshold
/// measured at one point on its axis reads dead everywhere it is not sitting.
///
/// | | what it is for |
/// |---|---|
/// | Cave Rat | over in three seconds — a fight won inside every window there is |
/// | Iron Sentinel | beaten, and slowly: the only one of the three that **turns** while there is still a fight left to spend it in |
/// | Kettle Wight | a wall that runs to the buzzer, which is the only way a board casts thirty times and a `cap` on borrowing ever binds |
///
/// And they are walked **in order, carrying out of each into the next**,
/// through the same `Character::carry_out_of` a settlement calls. `told` is the
/// one knob in the block that crosses a fight boundary, so a measurement that
/// never crosses one cannot see it.
const FOES: &[&str] = &["Cave Rat", "Iron Sentinel", "Kettle Wight"];

fn foe(name: &str) -> &'static MonsterSpec {
    combat::creature(name).expect("a creature on the ladder")
}

/// A power dressed as a class, so a fight can be handed it.
fn as_class(name: &'static str, power: ExpertPower) -> ClassDef {
    ClassDef { name, blurb: "", requires: &[], power: ClassPower::Expert(power) }
}

/// Everything about a fight a player could point at, as one string.
///
/// **The whole log, not one field.** Setting `Combatant::expert` alone makes
/// the fighter at the bell differ, so a comparison of fighters would pass on a
/// power wired to nothing — which is the stubbed-payout failure this file
/// exists to refuse.
fn shape(log: &combat::CombatLog) -> String {
    let mut out = format!("{:?}/{}/", log.outcome, log.duration_ms);
    for e in &log.entries {
        out.push_str(&format!("{}:{:?};", e.at_ms, e.event));
    }
    out
}

// ------------------------------------------------------------------ boards
//
// **Three boards and two of them purpose-built**, because the first draft used
// one and reported that seven experts reached nothing. They did, on that board:
// it had no empty frame for an Overwound Arm to turn, fifty-eight finished
// items where a Standing Fact wants four, nothing that spins, and every one of
// its twelve casts was the *enemy's* — so the three experts about paying for a
// cast were never asked a question. **A check that needs something to happen
// has to make sure it can.**

/// Seat a component by name, and say whether it went.
fn put(c: &mut Character, name: &str, k: SlotKind, x: u8, y: u8) -> bool {
    let Some(id) = c.find_by_name(name) else { return false };
    c.loadout.remove_anywhere(id);
    if c.can_equip(id, k, x, y).is_ok() {
        c.equip(id, k, x, y).is_ok()
    } else {
        false
    }
}

/// Eight finished items, forty-three seconds, thirteen curses landed.
fn busy() -> Character {
    let mut c = common::bench();
    common::build_full_loadout(&mut c);
    c
}

/// **Four finished items, two bare frames, a blade and a curse.** The board
/// Standing Fact, the Overwound Arm, the Short Programme and the Eleventh
/// Season are all priced against.
///
/// Every part of it is load-bearing and each part was arrived at by a knob
/// reading dead:
///
/// - **Four items exactly**, because `standing_fact_room` asks *are you
///   carrying this few*, and that tree walks `worn` from 2 to 4. A board with
///   three items is under the line at 3 and at 4 alike, so the node that buys
///   the fourth buys nothing anybody can see.
/// - **A weapon**, because an Overwound Arm banks its bare frames into bare
///   *strength*, and strength on a board of nothing but armour is a number
///   nothing spends. The first version of this fixture swept the weapon grid
///   and reported all five of that tree's nodes dead.
/// - **The chest, for the length of the fight.** Four items that fall over in
///   six seconds cannot reach a ceiling, fill a `carry` or bank past a `cap`;
///   half the knobs in this block are thresholds and a fight that ends early is
///   a measurement taken below all of them.
/// - **A cursing sole in the greaves**, built rather than dropped in:
///   `build_full_loadout` seats `Greave Mold` on the cell the sole needs, so
///   the seating has to be asserted. **A `put` whose answer is ignored is a
///   fixture that silently does nothing.**
/// - **Two frames left bare**, which is what an Overwound Arm turns and what a
///   Short Programme is paid for.
fn bare() -> Character {
    let mut c = busy();
    for k in [SlotKind::Helmet, SlotKind::Gloves] {
        c.loadout.slot_mut(k).clear();
    }
    // A greave is a Material and a Mold that touch. The sole is the Mold, and
    // it is the one in the catalogue whose every activation lands a curse.
    c.loadout.slot_mut(SlotKind::Greaves).clear();
    assert!(put(&mut c, "Runed Material", SlotKind::Greaves, 0, 0), "no material");
    assert!(put(&mut c, "Wayfarer's Sole", SlotKind::Greaves, 2, 0), "no curse source");
    c
}

/// **A board that casts more than it can pay for, and curses what it hits.**
///
/// Two book-and-spell items where the blade was, the chest and a dry pair of
/// greaves underneath them, and everything that pays Funny taken off. Each part
/// of that is a knob that read dead without it:
///
/// - **Two casters, not one, and both books hold no Funny.** Loud
///   Calculation's `cap` is how much may be bought on credit in one fight and
///   its tree walks that from 40 to 100, so a board that only ever runs forty
///   short cannot tell the two apart. `Hymnal` and `War Ledger` are two of the
///   four books in the catalogue granting none; with a `Pocket Grimoire` and a
///   helmet on, twenty-two of twenty-four casts paid for themselves and the
///   whole subject of these experts — *what happens when you cannot pay* —
///   never came up.
/// - **The greaves are rebuilt dry.** `Runner's Mold` grants Funny and the
///   busy board's gloves grant more; what is wanted from the lower half is the
///   armour that makes the fight long, not the Funny that makes it solvent.
/// - **Two spells and two different curses.** Curse Requisition's `pick` is
///   *cheapest first or ripest first*, and `Curses::spend_one` chooses between
///   entries **by kind** — so a caster landing one kind offers one entry and
///   the two orders are the same order. A Searing and a Frost is the smallest
///   board that can tell them apart.
/// - **The chest, for the length of the fight**, for the reason `bare` keeps
///   its own.
fn caster() -> Character {
    let mut c = busy();
    c.loadout.slot_mut(SlotKind::Helmet).clear();
    dry_greaves(&mut c);
    deep_pockets(&mut c);
    two_voices(&mut c);
    c
}

/// The caster with the gloves left on, which is a **trickle of Funny**.
///
/// Opening Number's `after` is a discount on a cast, and a discount is only a
/// thing to somebody who is nearly paying: with no Funny at all every cast is
/// short by the whole price and the percentage never comes up, and with a
/// helmet on nothing is ever short. The busy board's gloves pay two every three
/// seconds, which is exactly the state a discount decides.
fn busker() -> Character {
    let mut c = busy();
    c.loadout.slot_mut(SlotKind::Helmet).clear();
    dry_greaves(&mut c);
    two_voices(&mut c);
    c
}

/// Greaves that hold armour and no Funny.
fn dry_greaves(c: &mut Character) {
    c.loadout.slot_mut(SlotKind::Greaves).clear();
    assert!(put(c, "Runed Material", SlotKind::Greaves, 0, 0), "no material");
    assert!(put(c, "Greave Mold", SlotKind::Greaves, 2, 0), "no mold");
}

/// A chest that is **strength and a very long fight**, and both are the point.
///
/// Loud Calculation buys Funny *with strength*, never below one — so a caster
/// with a caster's shoulders can buy nine points and then nothing, whatever the
/// `cap` says, and `cap`, `rate` and `rebate` all read dead behind that. The
/// Money Jacket and the Hemline are forty-eight strength between them and no
/// Funny at all, and the Jacket's health is what keeps a board with two books
/// and no blade on its feet long enough to spend it.
///
/// The Hemline lands a **third** kind of curse, which is not a bonus: Curse
/// Requisition's `pick` chooses *between* what is standing.
fn deep_pockets(c: &mut Character) {
    c.loadout.slot_mut(SlotKind::Chest).clear();
    assert!(put(c, "The Money Jacket", SlotKind::Chest, 0, 0), "no jacket");
    assert!(put(c, "Assassin's Hemline", SlotKind::Chest, 0, 3), "no hemline");
    c.loadout.slot_mut(SlotKind::Gloves).clear();
    assert!(put(c, "Scaled Material", SlotKind::Gloves, 0, 0), "no scaled material");
    assert!(put(c, "Sovereign Mold", SlotKind::Gloves, 2, 0), "no sovereign mold");
}

/// Two book-and-spell items in the weapon frame, four rows apart so they do not
/// touch and become one.
///
/// **Both spells curse on their own activation.** `Attendant Flame` was the
/// first choice and is `OnOtherCast` — it curses when something *else* casts —
/// so on a board of two books it lands nothing at all and the fixture reported
/// one kind where it had asked for two.
fn two_voices(c: &mut Character) {
    c.loadout.slot_mut(SlotKind::Weapon).clear();
    assert!(put(c, "Hymnal", SlotKind::Weapon, 0, 0), "no first book");
    assert!(put(c, "Slash and Burn", SlotKind::Weapon, 1, 0), "no searing");
    assert!(put(c, "War Ledger", SlotKind::Weapon, 0, 4), "no second book");
    assert!(put(c, "Rime Nova", SlotKind::Weapon, 1, 4), "no frost");
}

/// The busy board with a **spin** ench on it and one row of the gloves filled
/// edge to edge.
///
/// Nothing in the catalogue spins by itself — the spin is an ench and the board
/// decides whether there is room to turn — and nothing in the catalogue fills a
/// row by accident, so a Patented Funnel measured against an ordinary board is
/// measured against neither half of itself.
fn spinning() -> Character {
    let mut c = busy();
    c.bought_licence = true;
    c.loadout.slot_mut(SlotKind::Gloves).clear();
    let one = gm2d_core::piece::CATALOG
        .iter()
        .find(|d| {
            d.slot == SlotKind::Gloves
                && d.cells.len() == 1
                && !d.kind.is_enchantment()
                && d.kind != gm2d_core::piece::PieceKind::Quest
        })
        .expect("a one-cell glove component")
        .name;
    for x in 0..gm2d_core::slot::SLOT_W {
        let id = c.give(one).expect("a component");
        c.loadout.slot_mut(SlotKind::Gloves).place(&c.registry, id, x, 0);
    }
    let spin = an_ench(|e| matches!(e, gm2d_core::ench::Effect::Spin));
    for p in c.combat_items().iter().take(3) {
        c.enchs_owned.push(spin.clone());
        let _ = c.attach_ench(&spin, p.pieces[0]);
    }
    c
}

/// **The busy board with every ench in the rack**, for the expert whose subject
/// is what an enched item does every time it comes round.
///
/// Not the Auto-packed board below it: that one has nineteen items and beats
/// everything on the ladder in under two seconds, which is a fight with no room
/// in it for a curse to be landed twice. Cursed Licence wants a board that is
/// still swinging in half a minute.
fn bewitched() -> Character {
    let mut c = busy();
    c.bought_licence = true;
    for e in &gm2d_core::data::enchs().enchs {
        c.enchs_owned.push(e.id.clone());
    }
    c
}

/// **An Auto-packed board and a rack full of enchs**, for the expert that is
/// about what a *neighbour* gets.
///
/// `build_full_loadout` makes eight items and **no two of them touch**, which
/// is fine for seven of the ten and useless for the two whose whole subject is
/// what a neighbour gets: `ench::broadcast` lends along `adjacent_items`, and a
/// board with no adjacency lends nothing whatever the percentage says. What
/// the button packs has nineteen items and forty-four touches.
///
/// One of every ench goes in the rack, loose, because Full Bill's `racks` is
/// *how many a component holds* — a knob you can only see move when there are
/// more enchs than racks — while the two nodes that hand an ench over can only
/// be seen when there are more racks than enchs. [`rack_up`] resolves that by
/// filling one component's rack before starting the next, so **both** the
/// count and the supply change what is bolted where.
fn enched() -> Character {
    let mut c = common::bench();
    c.pack_what_you_own();
    c.bought_licence = true;
    for e in &gm2d_core::data::enchs().enchs {
        c.enchs_owned.push(e.id.clone());
    }
    c
}

/// Bolt on everything the racks will take, one component at a time.
///
/// **After the expert and their nodes, never while the board is being built.**
/// How many a component holds is `Character::ench_racks`, which is Full Bill's
/// `racks`; what there is to bolt on includes whatever that tree hands over. A
/// fixture that racked up before either was set would be asking both questions
/// of a character who is not yet the expert — which is what it did, and why
/// five of Full Bill's six nodes read dead.
fn rack_up(c: &mut Character) {
    if !c.licensed() {
        return;
    }
    let seated: Vec<gm2d_core::piece::PieceId> =
        c.combat_items().iter().map(|p| p.pieces[0]).collect();
    for piece in seated {
        for id in c.enchs() {
            if c.attach_ench(&id, piece).is_err() {
                continue;
            }
        }
    }
}

fn an_ench(want: fn(&gm2d_core::ench::Effect) -> bool) -> String {
    gm2d_core::data::enchs()
        .enchs
        .iter()
        .find(|e| want(&e.effect))
        .expect("an ench of that kind")
        .id
        .clone()
}

/// Which board poses this expert's question.
fn board_for(name: &str) -> Character {
    match name {
        "OverwoundArm" | "StandingFact" | "ShortProgramme" | "EleventhSeason" => bare(),
        "LoudCalculation" | "CurseRequisition" => caster(),
        "OpeningNumber" => busker(),
        "PatentedFunnel" => spinning(),
        "CursedLicence" => bewitched(),
        "FullBill" => enched(),
        other => panic!("no board poses {other}'s question"),
    }
}

// ------------------------------------------------------------------- bench

/// A character who **is** that expert, with `nodes` of their tree spent.
///
/// The tree matters: four of the ten reach the fight partly through rules their
/// own nodes grant, and a power handed to a fight without them is half a class.
///
/// `rack_up` runs last, because what the racks hold is a question about the
/// expert and their tree rather than about the board.
fn as_the_expert(name: &str, nodes: &[String]) -> Character {
    let mut c = board_for(name);
    c.expert = Some(name.to_string());
    c.skills_taken.extend(nodes.iter().cloned());
    rack_up(&mut c);
    c
}

/// Everything that has to be taken before this node can be.
///
/// The transitive prerequisites, which is what makes *the chain plus this node*
/// a build somebody can be standing in rather than an arrangement of the tree
/// that no sequence of purchases reaches.
fn chain_to(class: &str, node: &str) -> Vec<String> {
    let skills = gm2d_core::data::skills();
    let tree = skills.tree_for_class(class).expect("a tree");
    let mut want = vec![node.to_string()];
    let mut seen: Vec<String> = Vec::new();
    while let Some(id) = want.pop() {
        let Some(n) = tree.nodes.iter().find(|n| n.id == id) else { continue };
        for r in &n.requires {
            if !seen.contains(r) {
                seen.push(r.clone());
                want.push(r.clone());
            }
        }
    }
    seen
}

fn nodes_of(class: &str) -> Vec<String> {
    gm2d_core::data::skills()
        .tree_for_class(class)
        .expect("a tree")
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect()
}

/// The fight this character has with this creature, with every class they are.
fn fight_of(c: &Character, against: &str) -> combat::CombatLog {
    let worn: Vec<ClassDef> = c.class_defs();
    combat::simulate_party_holding(
        c.player_stats(),
        &c.combat_items(),
        std::slice::from_ref(foe(against)),
        D,
        &worn,
        0,
        c.start_with(),
    )
}

/// What a purse does with this power, swept across the thresholds it has.
///
/// **A sweep and not one settlement**, because two of the purse knobs are
/// thresholds: once a window is wide enough to pay, widening it further changes
/// nothing, and a single measurement would call a working knob dead. The sweep
/// is over the two axes those thresholds sit on — how long the fight was, and
/// how much was standing on the corpse.
fn purse_sweep(name: &'static str, p: ExpertPower) -> String {
    let mut out = String::new();
    for ms in [1_000u32, 9_000, 13_000, 17_000, 23_000, 31_000, 44_000] {
        for standing in [0u32, 3, 6, 9, 12] {
            for streak in [0u32, 4] {
                let worn = [as_class(name, p)];
                let at = reward::AtTheBell {
                    empty_frames: 3,
                    curses_standing: standing,
                    curse_kinds: 4,
                    curses_expired: 4,
                    streak,
                };
                out.push_str(&format!(
                    "{},",
                    reward::bounty_with_class(Outcome::Victory, 400, &worn, ms, at)
                ));
            }
        }
    }
    out
}

/// Everything about this character a player could point at.
///
/// Three fights, what they hold at the bell, what the board makes, and what a
/// purse would do — because the ten experts land in three different places and
/// a measurement that watched only one of them would call the other two dead.
///
/// **The three fights are walked in order and each carries out into the next**,
/// through the same `Character::carry_out_of` a real settlement calls. That is
/// not decoration: `told` is the one knob in the block that crosses a fight
/// boundary, so a measurement that never crosses one cannot see it, and a
/// streak is the same shape a level further down.
fn measure(name: &'static str, c: &Character) -> String {
    let mut run = c.clone();
    let mut s = String::new();
    for against in FOES {
        let log = fight_of(&run, against);
        s.push_str(&shape(&log));
        run.carry_out_of(&log);
    }
    s.push_str(&format!("|held={:?}", c.start_with()));
    s.push_str(&format!("|items={:?}", c.combat_items()));
    if let Some(p) = c.expert_power() {
        s.push_str(&purse_sweep(name, p));
    }
    s
}

// ------------------------------------------------------- all fifteen classes

/// **Every class the game can hand out reaches something** — the five on the
/// fork and the ten behind the papers.
///
/// The five are held to this by `the_bill::every_offered_class_reaches_something`,
/// which is older than this block. This is the ten, measured the same way and
/// with the same rule: *called, not declared*.
#[test]
fn every_offered_class_reaches_something_over_all_fifteen() {
    let mut unreached = Vec::new();
    for e in EXPERTS {
        let def = CLASSES.iter().find(|d| d.name == e.name).expect("in the roster");
        assert!(matches!(def.power, ClassPower::Expert(_)), "{} is not an expert", e.name);
        let all = nodes_of(e.name);
        let with = as_the_expert(e.name, &all);
        let mut without = board_for(e.name);
        without.expert = None;
        rack_up(&mut without);
        if measure(e.name, &with) == measure(e.name, &without) {
            unreached.push(e.name);
        }
    }
    assert!(
        unreached.is_empty(),
        "an expert costing two finished trees and reaching nothing: {unreached:?}"
    );
}

// ------------------------------------------------------------------ the tree

/// **Every point spent in an expert tree buys something.**
///
/// Sixty nodes, each bought on its own, each having to change a fight, a purse
/// or a board. This is the guard that would have caught eight skill nodes in
/// M8 — *they parsed, they cost points, they showed as taken, and they changed
/// nothing* — stated over the sixty that replaced them, and on its first honest
/// run it caught thirty-eight.
///
/// It is stronger than asking each knob in isolation, and simpler: a knob is
/// only reachable through a node, some knobs are switches that change nothing
/// when turned further up, and some are read on the board rather than in the
/// fight. *Does spending this point change anything* covers all three and is
/// the question a player is actually asking.
///
/// # Asked at the moment it is bought, and again at the top
///
/// **The first question is the reachable one.** A point is spent from
/// somewhere: you take a node's prerequisites and then you take the node, and
/// what you want to know is whether the fight is different afterwards. So the
/// primary comparison is *the chain up to this node* against *the chain plus
/// this node*, which is a build a player can actually be standing in.
///
/// It also asks the knob where it is most likely to answer. Half of these are
/// thresholds — a `cap` on what may be borrowed, a `worn` count, a `ceiling` —
/// and a threshold is legible at the bottom of its range and invisible at the
/// top: forty against seventy is a different fight, and seventy against a
/// hundred is the same fight twice when nothing in the game can spend seventy.
///
/// **The second question is the whole tree minus this node**, asked only of
/// what the first one could not see. It is not a reachable build — dropping a
/// root leaves its children taken — and that is exactly what makes it useful as
/// a second look: it isolates one node's effect at the top of the tree, where a
/// capstone's own numbers live.
#[test]
fn every_point_in_an_expert_tree_buys_something() {
    let mut dead = Vec::new();
    for e in EXPERTS {
        let all = nodes_of(e.name);
        // **Measured once per set of nodes, not once per question.** Six nodes
        // of a tree share four chains between them — both roots have the empty
        // one — and a measurement is three fights, one of which runs to the
        // sudden-death buzzer. Keyed on the set rather than the order, because
        // a tree is a set of points spent and the order they were spent in is a
        // fact about somebody's afternoon.
        let mut seen: HashMap<Vec<String>, String> = HashMap::new();
        let mut of = |nodes: &[String]| -> String {
            let mut key = nodes.to_vec();
            key.sort();
            if let Some(had) = seen.get(&key) {
                return had.clone();
            }
            let said = measure(e.name, &as_the_expert(e.name, nodes));
            seen.insert(key, said.clone());
            said
        };

        for node in &all {
            // Bought: the chain that had to come first, and then this.
            let chain = chain_to(e.name, node);
            let mut bought = chain.clone();
            bought.push(node.clone());
            if of(&chain) != of(&bought) {
                continue;
            }
            // And again from the top, which is where a capstone's numbers are.
            let minus: Vec<String> = all.iter().filter(|n| *n != node).cloned().collect();
            if of(&minus) == of(&all) {
                dead.push(format!("{}::{node}", e.name));
            }
        }
    }
    assert!(
        dead.is_empty(),
        "a point the tree sells and the engine never reads:\n  {}",
        dead.join("\n  ")
    );
}

// ------------------------------------------------- three classes, one fight

/// **No pair of live powers disagrees.** Resolution is order-independent
/// across every reachable triple, because each of the fifteen touches its own
/// field and none writes where another reads.
#[test]
fn no_pair_of_live_powers_disagrees() {
    let c = busy();
    for e in EXPERTS {
        let (a, b) = e.pair;
        let run = |first: &str, second: &str| {
            let mut ch = Character::new();
            ch.class = Some(first.to_string());
            ch.second_class = Some(second.to_string());
            ch.expert = Some(e.name.to_string());
            let worn: Vec<ClassDef> = ch.class_defs();
            shape(&combat::simulate_party_holding(
                c.player_stats(),
                &c.combat_items(),
                std::slice::from_ref(foe("Rust Colossus")),
                D,
                &worn,
                0,
                c.start_with(),
            ))
        };
        assert_eq!(run(a, b), run(b, a), "{} depends on which class came first", e.name);
    }
}

/// All three powers are live at once, and the third does not switch off the
/// first two.
///
/// **The pair has to be two classes the fight can see, and the board has to
/// pose their question.** The first version asked this of Bloodletter and
/// Recycler and read the answer out of a fight: `Recycler` is `assembly_pct`,
/// which is a *board* rule a fight never hears about, so the second class
/// reached nothing **through that measurement** and the test failed on a game
/// that was working. Berserker's Leeching and Bloodletter's Bloodscent are both
/// the fight's, their expert is the Standing Fact, and `bare()` is the board
/// that lands the curses all three of them are about.
#[test]
fn a_third_class_does_not_replace_the_first_two() {
    let c = bare();
    let pair = expert::for_pair("Berserker", "Bloodletter").expect("a pair");
    let of = |names: &[&str]| {
        let worn: Vec<ClassDef> = names
            .iter()
            .map(|n| *CLASSES.iter().find(|d| d.name == *n).expect("a class"))
            .collect();
        shape(&combat::simulate_party_holding(
            c.player_stats(),
            &c.combat_items(),
            std::slice::from_ref(foe("Rust Colossus")),
            D,
            &worn,
            0,
            c.start_with(),
        ))
    };
    let one = of(&["Berserker"]);
    let two = of(&["Berserker", "Bloodletter"]);
    assert_ne!(one, two, "the second class reached nothing");
    // The expert on top of both, with the tree that makes it what it is.
    let mut three = bare();
    three.class = Some("Berserker".into());
    three.second_class = Some("Bloodletter".into());
    three.expert = Some(pair.name.to_string());
    for n in nodes_of(pair.name) {
        three.skills_taken.push(n);
    }
    let worn: Vec<ClassDef> = three.class_defs();
    let with = shape(&combat::simulate_party_holding(
        three.player_stats(),
        &three.combat_items(),
        std::slice::from_ref(foe("Rust Colossus")),
        D,
        &worn,
        0,
        three.start_with(),
    ));
    assert_ne!(with, two, "the expert reached nothing on top of its parents");
}

/// Full Bill's `racks`: how many enchs may be bolted to one component.
///
/// **Called, not declared**, in both halves. The knob moves and the promise
/// re-reads itself — and then a component is actually handed two enchs, and a
/// character who is not a Full Bill is actually refused the second. `racks`
/// was a knob two nodes sold and `attach_ench` never asked about until M13.6:
/// *"one ench a component"* was written into the refusal, so the expert whose
/// whole promise is the second rack promised a rack the engine would not give.
#[test]
fn an_extra_rack_holds_an_extra_ench() {
    let power = ExpertPower::FullBill { racks: 2, beacon_pct: 0 };
    let mut wider = power;
    wider.tune("racks", 2);
    assert_eq!(power.knob("racks"), Some(2));
    assert_eq!(wider.knob("racks"), Some(4));
    // And the promise says so, which is the only place a player reads it.
    assert!(power.describe().contains("2 enchs"), "{}", power.describe());
    assert!(wider.describe().contains("4 enchs"), "{}", wider.describe());

    // Two enchs, one component, and the second one is the test.
    let two: Vec<String> = gm2d_core::data::enchs()
        .enchs
        .iter()
        .take(2)
        .map(|e| e.id.clone())
        .collect();
    let seat = |c: &mut Character| -> Vec<Result<(), gm2d_core::ench::Refusal>> {
        let piece = c.combat_items()[0].pieces[0];
        two.iter().map(|id| c.attach_ench(id, piece)).collect()
    };

    let mut bill = bewitched();
    bill.expert = Some("FullBill".to_string());
    assert_eq!(bill.ench_racks(), 2, "a Full Bill starts with two racks");
    let went = seat(&mut bill);
    assert!(went.iter().all(|r| r.is_ok()), "a Full Bill was refused a rack: {went:?}");

    // Anybody else holds one, and is told what is on it.
    let mut alone = bewitched();
    assert_eq!(alone.ench_racks(), 1, "one ench a component, for everybody else");
    let went = seat(&mut alone);
    assert!(went[0].is_ok(), "the first was refused: {:?}", went[0]);
    let Err(gm2d_core::ench::Refusal::AlreadyEnched(what, racks)) = &went[1] else {
        panic!("a second ench went on a single rack: {:?}", went[1]);
    };
    assert_eq!(*racks, 1);
    assert!(!what.is_empty(), "the refusal does not name what is on it");
}
