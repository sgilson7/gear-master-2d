//! TONE.md, as a lint.
//!
//! A style guide nobody can run is a style guide that decays. These are the
//! rules from `TONE.md` that a machine can actually check — not the ones about
//! register, which need a reader, but the ones that are facts about a string.
//!
//! Every one of them caught something on its first run.

use gm2d_core::data;
use gm2d_core::tile_event::{Choice, EventsData, Requirement};

/// Every player-facing string in the shipped content.
///
/// Deliberately not the schema keys. Rule 13's whole point is that the engine
/// still says "mana" everywhere, because everything it decides depends on that
/// word meaning one thing — it is the *content* that speaks the game's
/// language, and this is the content.
fn prose() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let events = data::events();
    for e in &events.events {
        out.push((format!("{} title", e.id), e.title.clone()));
        for (i, p) in e.prose.iter().enumerate() {
            out.push((format!("{} prose[{i}]", e.id), p.clone()));
        }
        for c in &e.choices {
            out.push((format!("{} label {:?}", e.id, c.label), c.label.clone()));
            out.push((format!("{} blurb {:?}", e.id, c.label), c.blurb.clone()));
            if !c.unmet.is_empty() {
                out.push((format!("{} unmet {:?}", e.id, c.label), c.unmet.clone()));
            }
        }
    }
    let skills = data::skills();
    for t in &skills.trees {
        for n in &t.nodes {
            out.push((format!("{} name", n.id), n.name.clone()));
            out.push((format!("{} blurb", n.id), n.blurb.clone()));
        }
    }
    out
}

/// **Rule 13.** The economy speaks the book's language.
///
/// A canonical stat name in a player-facing string is a string that will read
/// as the engine's rather than the game's. `skills.json` had one: a blurb about
/// a plaid suit that said "armour" twice where the game says Cork.
#[test]
fn no_player_facing_string_uses_the_engine_vocabulary() {
    // Canonical -> what the game calls it. Whole words only, case-insensitive.
    const SWAP: &[(&str, &str)] = &[
        ("gold", "Fnorp"),
        ("mana", "the Funny"),
        ("armour", "Cork"),
        ("armor", "Cork"),
        ("rage", "Fury"),
        ("faith", "Devotion"),
        ("nature", "Harvest"),
    ];
    let mut bad = Vec::new();
    for (where_, text) in prose() {
        let lower = text.to_lowercase();
        for (canonical, themed) in SWAP {
            let hit = lower
                .split(|c: char| !c.is_alphanumeric())
                .any(|w| w == *canonical);
            if hit {
                bad.push(format!("{where_}: says {canonical:?}, and the game says {themed:?}\n    {text}"));
            }
        }
    }
    assert!(bad.is_empty(), "TONE.md rule 13:\n  {}", bad.join("\n  "));
}

/// **Rule 4.** Scale is a number, never an intensifier.
#[test]
fn nothing_reaches_for_an_intensifier_instead_of_a_number() {
    const VAGUE: &[&str] = &[
        "countless", "untold", "innumerable", "myriad", "endless", "infinite",
        "immeasurable", "unimaginable", "legions", "untellable",
    ];
    let mut bad = Vec::new();
    for (where_, text) in prose() {
        let lower = text.to_lowercase();
        for w in VAGUE {
            if lower.split(|c: char| !c.is_alphanumeric()).any(|t| t == *w) {
                bad.push(format!("{where_}: {w:?} is doing a number's job\n    {text}"));
            }
        }
    }
    assert!(bad.is_empty(), "TONE.md rule 4:\n  {}", bad.join("\n  "));
}

/// **Rule 10.** A speech tag carries an action or nothing — never an adverb of
/// manner, which is rule 5 (never explain a joke) wearing a different hat.
#[test]
fn no_speech_tag_explains_its_own_tone() {
    let mut bad = Vec::new();
    for (where_, text) in prose() {
        let words: Vec<&str> = text.split_whitespace().collect();
        for pair in words.windows(2) {
            let tag = pair[0].trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            let next = pair[1].trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            let is_tag = matches!(tag.as_str(), "says" | "said" | "asks" | "asked" | "replies" | "replied");
            if is_tag && next.ends_with("ly") && next.len() > 4 {
                bad.push(format!("{where_}: {tag:?} {next:?} tells the reader the tone\n    {text}"));
            }
        }
    }
    assert!(bad.is_empty(), "TONE.md rule 10:\n  {}", bad.join("\n  "));
}

/// **Rule 3.** Characters count things and report the count.
///
/// Checked per *event* rather than per line: the tic is a property of a scene,
/// and demanding a number in every sentence would produce a worse one.
#[test]
fn every_event_counts_something() {
    let events = data::events();
    let mut bare = Vec::new();
    for e in &events.events {
        let scene = e.prose.join(" ").to_lowercase();
        let has_digit = scene.chars().any(|c| c.is_ascii_digit());
        let words = [
            "one", "two", "three", "four", "five", "six", "seven", "eight",
            "nine", "ten", "eleven", "twelve", "hundred", "thousand", "ninth",
            "sixth", "fifth", "fourth", "third", "second", "dozen", "forty",
        ];
        let has_word = scene
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| words.contains(&w));
        if !has_digit && !has_word {
            bare.push(e.id.clone());
        }
    }
    assert!(
        bare.is_empty(),
        "TONE.md rule 3 — these scenes count nothing: {bare:?}"
    );
}

/// A number as a player would write it. Enough for the costs the game has.
fn spell(n: i32) -> String {
    const ONES: [&str; 20] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
        "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
        "sixteen", "seventeen", "eighteen", "nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy",
        "eighty", "ninety",
    ];
    let n = n.unsigned_abs() as usize;
    match n {
        0..=19 => ONES[n].to_string(),
        20..=99 => {
            let (t, o) = (n / 10, n % 10);
            if o == 0 { TENS[t].to_string() } else { format!("{}-{}", TENS[t], ONES[o]) }
        }
        100..=999 => {
            let (h, r) = (n / 100, n % 100);
            if r == 0 {
                format!("{} hundred", ONES[h])
            } else {
                format!("{} hundred and {}", ONES[h], spell(r as i32))
            }
        }
        _ => n.to_string(),
    }
}

/// **Rule 12.** A refusal names the thing that is missing.
///
/// Not "requirement not met". The `unmet` line is what a greyed-out choice
/// says, and a greyed-out choice that does not say why is a choice the player
/// argues with.
#[test]
fn every_refusal_names_what_is_missing() {
    let events = data::events();
    let mut bad = Vec::new();
    for e in &events.events {
        for c in &e.choices {
            // A cost may be named in digits or in words — "Forty Fnorp" names
            // forty, and spelling small numbers out is the house style. The
            // first version of this test only looked for digits and failed on
            // two lines that were perfectly clear.
            let subjects: Vec<String> = match &c.requires {
                Requirement::None => vec![],
                Requirement::Gold(n) => vec![n.to_string(), spell(*n)],
                Requirement::Flag(f) => vec![f.replace('-', " ")],
                Requirement::Holding(name) => vec![name.to_lowercase()],
            };
            if subjects.is_empty() {
                continue;
            }
            let unmet = c.unmet.to_lowercase();
            let named = subjects.iter().any(|subject| {
                unmet.contains(subject)
                    || subject.split_whitespace().any(|w| w.len() > 2 && unmet.contains(w))
            });
            let subject = subjects.join(" or ");
            if !named {
                bad.push(format!(
                    "{}: {:?} refuses with {:?}, which never mentions {subject:?}",
                    e.id, c.label, c.unmet
                ));
            }
        }
    }
    assert!(bad.is_empty(), "TONE.md rule 12:\n  {}", bad.join("\n  "));
}

/// A skill that grants rows names the frame it grants them on.
///
/// Not a TONE.md rule but the same failure it guards against: a blurb that
/// overstates its own effect is the worst kind, because the player finds out by
/// not getting it. One node promised a row on the weapon *and* the chest and
/// granted the weapon.
#[test]
fn a_row_granting_skill_names_its_frame() {
    use gm2d_core::skills::Effect;
    let tree = data::skills();
    let mut bad = Vec::new();
    for t in &tree.trees {
        for n in &t.nodes {
            if let Effect::GrowSlotRows { slot, .. } = &n.effect {
                if !n.blurb.to_lowercase().contains(&slot.to_lowercase()) {
                    bad.push(format!("{}: grants {slot:?} and says {:?}", n.id, n.blurb));
                }
            }
        }
    }
    assert!(bad.is_empty(), "a blurb that overstates its effect:\n  {}", bad.join("\n  "));
}

/// Nothing in the content is empty, and nothing is a placeholder.
#[test]
fn nothing_shipped_is_a_stub() {
    let mut bad = Vec::new();
    for (where_, text) in prose() {
        let t = text.trim();
        if t.is_empty() {
            bad.push(format!("{where_} is empty"));
        }
        for stub in ["TODO", "TBD", "FIXME", "lorem", "placeholder", "XXX"] {
            if t.to_lowercase().contains(&stub.to_lowercase()) {
                bad.push(format!("{where_} is a stub: {t:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n  "));
}

/// Choice labels are what you *do*; blurbs are what it costs or what you are in
/// for. A label that is a whole sentence is a blurb in the wrong field.
#[test]
fn choice_labels_are_actions_not_sentences() {
    let events = data::events();
    let mut bad = Vec::new();
    for e in &events.events {
        for Choice { label, blurb, .. } in &e.choices {
            if label.len() > 34 || label.ends_with('.') {
                bad.push(format!("{}: label {label:?} is a sentence", e.id));
            }
            if blurb.len() < 12 {
                bad.push(format!("{}: blurb {blurb:?} says nothing", e.id));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n  "));
}
