//! A skill has to say what it does.
//!
//! The tree shipped for a while describing itself only in the world's words —
//! "Nine hundred feet of Deep Chocolate mine, and you never once came up
//! early" is a good sentence about a character and tells nobody it is sixty
//! max health. The name carries the world and [`Node::line`] carries the
//! arithmetic; these are the rules that keep the second half honest.
//!
//! The lint that pays for itself is the last one. serde ignores a key it does
//! not recognise, so eight nodes granting `armor` or `mana` — the whole spine
//! of one class tree — parsed cleanly, cost points, and did nothing whatever.

use gm2d_core::combat::{simulate_holding, Difficulty, Held, MonsterSpec, Side};
use gm2d_core::data;
use gm2d_core::skills::Effect;
use gm2d_core::stats::Stats;

/// Every node states a number.
///
/// Vagueness is the bug this file exists about: a line without a figure in it
/// is a line somebody has to take on faith.
#[test]
fn every_node_says_what_it_does_and_says_it_in_numbers() {
    let mut bad = Vec::new();
    for t in &data::skills().trees {
        for n in &t.nodes {
            let line = n.line();
            if line.is_empty() {
                bad.push(format!("{}: no mechanical line at all", n.id));
            } else if !line.chars().any(|c| c.is_ascii_digit()) {
                bad.push(format!("{}: {line:?} names no number", n.id));
            }
            if n.detail().is_empty() {
                bad.push(format!("{}: nothing to say on hover", n.id));
            }
        }
    }
    assert!(bad.is_empty(), "a node that will not say what it does:\n  {}", bad.join("\n  "));
}

/// **The inverse of TONE.md rule 13, and deliberately so.**
///
/// Everywhere else the content speaks the game's language; here it must not.
/// A player comparing two nodes is reading a spec, and a spec that says "29
/// Cork" when the number is armour makes them look up a joke to find out
/// whether to spend a point.
#[test]
fn no_mechanical_line_speaks_the_theme() {
    const THEMED: &[&str] = &["fnorp", "the funny", "cork", "fury", "devotion", "harvest"];
    let mut bad = Vec::new();
    for t in &data::skills().trees {
        for n in &t.nodes {
            let said = format!("{} {}", n.line(), n.detail().join(" ")).to_lowercase();
            for w in THEMED {
                if said.contains(w) {
                    bad.push(format!("{}: mechanical text says {w:?}", n.id));
                }
            }
        }
    }
    assert!(bad.is_empty(), "themed words in a spec:\n  {}", bad.join("\n  "));
}

/// The line is short enough to sit under the name without wrapping twice.
#[test]
fn a_mechanical_line_stays_short_enough_to_read_at_a_glance() {
    let mut bad = Vec::new();
    for t in &data::skills().trees {
        for n in &t.nodes {
            let line = n.line();
            if line.len() > 90 {
                bad.push(format!("{}: {} chars — {line:?}", n.id, line.len()));
            }
        }
    }
    assert!(bad.is_empty(), "a line nobody will read:\n  {}", bad.join("\n  "));
}

/// **The one that caught eight dead nodes.**
///
/// `Effect` is deserialised field by field and serde drops a key it has never
/// heard of, so `"stat": { "armor": 12 }` survived the removal of `armor` from
/// `Effect::Stat` in perfect silence — a node that costs a point and changes
/// nothing. Reading the raw JSON is the only place that can be caught.
#[test]
fn every_effect_key_is_one_the_engine_actually_reads() {
    const KNOWN: &[(&str, &[&str])] = &[
        ("stat", &["health", "strength", "regen", "mind_resist", "curse_resist"]),
        ("start_with", &["armor", "mana"]),
        ("grow_slot_rows", &["slot", "rows"]),
        ("assembly_pct", &["pct"]),
    ];
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../../data/skills.json")).unwrap();
    let mut bad = Vec::new();
    for tree in raw["trees"].as_array().unwrap() {
        for node in tree["nodes"].as_array().unwrap() {
            let id = node["id"].as_str().unwrap();
            let one = node["effect"].clone();
            let each = one.as_array().cloned().unwrap_or_else(|| vec![one]);
            for e in each {
                for (kind, body) in e.as_object().unwrap() {
                    let Some((_, fields)) = KNOWN.iter().find(|(k, _)| k == kind) else {
                        bad.push(format!("{id}: no effect called {kind:?}"));
                        continue;
                    };
                    for key in body.as_object().unwrap().keys() {
                        if !fields.contains(&key.as_str()) {
                            bad.push(format!("{id}: {kind}.{key} is read by nothing"));
                        }
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "a node paying for nothing:\n  {}", bad.join("\n  "));
}

/// Armour off the tree is armour in the fight.
///
/// Stated against the fight rather than against `start_with`, because what
/// went wrong was never the arithmetic — it was that the number never arrived.
#[test]
fn armour_the_tree_grants_is_armour_the_fight_starts_with() {
    const BITER: MonsterSpec = MonsterSpec {
        name: "Biter",
        health: 100_000,
        strength: 0,
        regen: 0,
        mind_resist: 0,
        physical_resist: 0,
        magic_resist: 0,
        curse_resist: 0,
        attacks: &[gm2d_core::combat::MonsterAttack::hit("bite", 500, 12)],
        gear: &[],
        gear_offset: 0,
        bounty: 0,
        sprite: gm2d_core::combat::MonsterSprite::Rat,
        rank: gm2d_core::combat::Rank::Ordinary,
        drops: &[],
        items: &[],
    };
    let stats = Stats::new(200, 0, 0, 100);
    // What armour *soaks*, not what the enemy swings for. Armour makes the
    // fight last longer, so a plated player is hit more times and the total
    // damage aimed at them goes up — the first version of this test measured
    // that and read it as the armour doing nothing.
    let soaked = |held: Held| {
        let log = simulate_holding(stats, &[], &BITER, Difficulty::Easy, &[], 0, held);
        log.entries
            .iter()
            .filter_map(|e| match e.event {
                gm2d_core::combat::Event::Hit { by, absorbed, .. } if by == Side::Enemy => {
                    Some(absorbed)
                }
                _ => None,
            })
            .sum::<i32>()
    };
    assert_eq!(soaked(Held::default()), 0, "nobody starts a fight wearing armour");
    assert_eq!(
        soaked(Held { armor: 40, mana: 0 }),
        40,
        "all forty points should be spent soaking, and no more than forty"
    );
}

/// And the shipped tree really does grant some.
#[test]
fn the_shipped_tree_still_hands_out_what_it_promises() {
    let tree = data::skills();
    let held = tree.start_with(&["corked".into(), "funnel-drill".into()]);
    assert_eq!(held, Held { armor: 12, mana: 20 }, "the two base nodes that grant them");

    // And the mixed node keeps both halves: strength through `stats_from`,
    // armour through `start_with`.
    let five = tree.node("g-the-five").expect("The Five");
    assert_eq!(five.effects.len(), 2, "a stat and a starting balance");
    assert_eq!(tree.stats_from(&["g-the-five".into()]).strength, 16);
    assert_eq!(tree.start_with(&["g-the-five".into()]).armor, 10);
    assert!(
        five.line().contains("+16 strength") && five.line().contains("10 armor"),
        "both halves in the line: {:?}",
        five.line()
    );
}

/// Growing a grid is still growing a grid.
#[test]
fn a_row_node_names_the_grid_and_the_count() {
    let e = Effect::GrowSlotRows { slot: "weapon".into(), rows: 1 };
    assert_eq!(e.line(), "+1 row on the weapon grid");
    assert_eq!(Effect::AssemblyPct { pct: 10 }.line(), "+10% to every assembly bonus");
}

#[test]
#[ignore = "prints the whole tree for a human to read"]
fn show() {
    for t in &data::skills().trees {
        println!("\n== {} ==", t.name);
        for n in &t.nodes {
            println!("  {:<22} {}", n.name, n.line());
            for d in n.detail() {
                println!("  {:<22}   · {d}", "");
            }
        }
    }
}

#[test]
#[ignore = "lists the catalogue by price, for picking shelves"]
fn show_cheap() {
    use gm2d_core::piece::{is_boss_only, is_off_the_road, is_quest_reward, CATALOG};
    let mut v: Vec<_> = CATALOG
        .iter()
        .filter(|d| !is_boss_only(d.name) && !is_quest_reward(d.name) && !is_off_the_road(d))
        .collect();
    v.sort_by_key(|d| d.price);
    let lo: i32 = std::env::var("LO").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let hi: i32 = std::env::var("HI").ok().and_then(|v| v.parse().ok()).unwrap_or(26);
    for d in v.iter().filter(|d| d.price >= lo && d.price <= hi) {
        println!("{:>4}  {:<10?} {:<12?} {:<28} {:?}", d.price, d.slot, d.kind, d.name, d.cells);
    }
}

/// Every component in the catalogue explains itself.
///
/// The last thing in the game that could not: a card said what an *item* did,
/// and a piece on the board said its name. `piece_lines` is what the hover
/// reads, and a piece with nothing to say is either a bug or a component that
/// should not be for sale.
#[test]
fn every_component_says_something_about_itself() {
    use gm2d_core::piece::{is_event_only, PieceKind, CATALOG};
    let mut bad = Vec::new();
    for d in CATALOG {
        // A quest token is a tally and is meant to do nothing at all.
        if d.kind == PieceKind::Quest {
            continue;
        }
        // **The six orphaned relics.** Their own comment says it: "their stat
        // lines are empty on purpose: what they are worth is a function of the
        // run, and it lives in `relic.rs`" — and `relic.rs` went with the
        // campaign in `48203ee`. They are `EVENT_ONLY`, they are on no town
        // shelf, and the events that handed them out left with the module, so
        // nothing in GM2D can put one in a bag. Skipped rather than fixed:
        // giving six unreachable components stats would be inventing content
        // to satisfy a lint.
        if is_event_only(d.name) {
            continue;
        }
        if gm2d_core::explain::piece_lines(d).is_empty() {
            bad.push(d.name);
        }
    }
    assert!(bad.is_empty(), "components that explain nothing:\n  {}", bad.join("\n  "));
}

/// And what it says is the engine's words, not the book's.
#[test]
fn no_component_line_speaks_the_theme() {
    use gm2d_core::piece::CATALOG;
    const THEMED: &[&str] = &["fnorp", "the funny", "cork", "fury", "devotion", "harvest"];
    let mut bad = Vec::new();
    for d in CATALOG {
        for (_, line) in gm2d_core::explain::piece_lines(d) {
            let low = line.to_lowercase();
            for w in THEMED {
                // **Fnorp is the exception, and only Fnorp.** Every other word
                // on this list has an engine name a player also meets — armour
                // is armour on the card, mana is mana in the tree. The
                // currency does not: the panel says Fnorp, the shelf says
                // Fnorp, and a spec that said "5 gold" would be the only place
                // in the game using a word for money that appears nowhere
                // else.
                if *w == "fnorp" {
                    continue;
                }
                if low.contains(w) {
                    bad.push(format!("{}: {line:?}", d.name));
                }
            }
        }
    }
    assert!(bad.is_empty(), "themed words in a spec:\n  {}", bad.join("\n  "));
}

/// **The log reports what the fight opened with, and the replay reads it.**
///
/// Reported from a real session: a character holding `Corked` and `Funnel
/// Drill` watched a fight begin with an empty armour bar and no mana, and
/// concluded the two skills did nothing. They were working — twelve armour was
/// soaking blows the whole time. Nothing *announced* the opening balance,
/// because nothing had to gain it, and the only armour event is one that
/// reports what is left after a hit.
///
/// `CombatLog::player` is the fighter as the bell went. This is the assertion
/// that it carries the balance, which is what the fight screen now seeds from.
#[test]
fn the_log_opens_holding_what_the_tree_granted() {
    use gm2d_core::character::Character;
    use gm2d_core::combat::{simulate_holding, Difficulty, Event, MonsterSpec, Side};

    let mut c = Character::starting();
    c.apply_preset();
    for id in ["corked", "funnel-drill"] {
        c.skill_points += 4;
        c.take_skill(&data::skills(), id).expect("a base node with a point in hand");
    }
    let held = c.start_with();
    assert_eq!(held, Held { armor: 12, mana: 20 }, "the two nodes as shipped");

    let spec: &MonsterSpec =
        gm2d_core::combat::LADDER.iter().find(|s| s.name == "Bog Toad").expect("a toad");
    let log = simulate_holding(
        c.player_stats(),
        &c.combat_items(),
        spec,
        Difficulty::Easy,
        &[],
        0,
        held,
    );

    assert_eq!(log.player.armor, held.armor, "the opening frame's armour");
    assert_eq!(log.player.mana, held.mana, "the opening frame's mana");

    // And it is real armour, not a number on a bar: it soaks.
    let soaked: i32 = log
        .entries
        .iter()
        .filter_map(|e| match e.event {
            Event::Hit { by, absorbed, .. } if by == Side::Enemy => Some(absorbed),
            _ => None,
        })
        .sum();
    assert_eq!(soaked, held.armor, "all twelve points should be spent soaking, and no more");
}

/// The four nodes from that session, and what each is worth.
#[test]
fn the_four_nodes_a_player_took_all_do_something() {
    use gm2d_core::character::Character;
    let mut c = Character::starting();
    let before = c.player_stats();
    for id in ["corked", "funnel-drill", "cave-lungs", "handspan"] {
        c.skill_points += 4;
        c.take_skill(&data::skills(), id).expect("takeable");
    }
    let after = c.player_stats();
    assert_eq!(after.health - before.health, 60, "Cave Lungs");
    assert_eq!(after.strength - before.strength, 6, "Handspan");
    assert_eq!(c.start_with(), Held { armor: 12, mana: 20 }, "Corked and Funnel Drill");
}
