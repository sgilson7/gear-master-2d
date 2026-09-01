//! The themes survive being a file.
//!
//! `PLAN.md` §2 says content lives in `data/*.json`, and decision 1.9 says the
//! theme is content. The statics are still what the engine reads — flipping
//! that is M2's, when there is a UI reading a theme at all — so what M0 owes is
//! the file and the evidence that nothing was lost on the way into it.
//!
//! That evidence is here: every key of every table, read back off the JSON,
//! answers what the static answers. If it does, `data/theme.td.json` is the
//! theme, and M2 can delete the static without checking anything twice.
//!
//! Regenerate with `REBASELINE_THEME_DATA=1 cargo test -p gm2d-core`, and say
//! in the commit what changed in the words.

use gm2d_core::theme::{Theme, PLAIN, THEMES, TURTLE_DICK};
use gm2d_core::theme_data::{ThemeData, FORMAT, VERSION};

fn path(id: &str) -> String {
    format!("{}/../../data/theme.{id}.json", env!("CARGO_MANIFEST_DIR"))
}

fn write_out(t: &'static Theme) -> String {
    serde_json::to_string_pretty(&ThemeData::of(t)).expect("a theme serialises")
}

#[test]
fn every_theme_is_written_out_as_data() {
    let rebaseline = std::env::var("REBASELINE_THEME_DATA").as_deref() == Ok("1");
    if rebaseline {
        std::fs::create_dir_all(format!("{}/../../data", env!("CARGO_MANIFEST_DIR"))).unwrap();
    }
    for t in THEMES {
        let p = path(t.id);
        let want = write_out(t);
        if rebaseline {
            std::fs::write(&p, &want).unwrap();
            continue;
        }
        let got = std::fs::read_to_string(&p).unwrap_or_else(|e| {
            panic!("cannot read {p}: {e}\nRegenerate with REBASELINE_THEME_DATA=1")
        });
        assert_eq!(
            got, want,
            "{} has drifted from the static it was written from.\n\
             If the words changed on purpose, regenerate with REBASELINE_THEME_DATA=1.",
            p
        );
    }
}

/// The file answers what the static answers, for every key it holds.
///
/// The point of the whole exercise. A theme is a lookup, so "lossless" means
/// the lookups agree — not that the bytes match some other serialisation.
#[test]
fn the_file_answers_what_the_static_answers() {
    for t in THEMES {
        let d = ThemeData::parse(&write_out(t)).expect("it parses");
        assert_eq!(d.id, t.id);
        assert_eq!(d.label, t.label);
        assert_eq!(d.blurb, t.blurb);

        for (canonical, _) in t.pieces {
            assert_eq!(d.piece(canonical), t.piece(canonical), "piece {canonical}");
        }
        for (canonical, _) in t.monsters {
            assert_eq!(d.monster(canonical), t.monster(canonical), "monster {canonical}");
        }
        for (canonical, _) in t.classes {
            assert_eq!(d.class(canonical), t.class(canonical), "class {canonical}");
        }
        for (slug, _) in t.words {
            assert_eq!(d.word(slug, "x"), t.word(slug, "x"), "word {slug}");
        }
        for (monster, _) in t.notes {
            assert_eq!(d.note(monster).as_deref(), t.note(monster), "note {monster}");
        }
        for r in t.told {
            assert_eq!(d.place(r.id, "x"), t.place(r.id, "x"), "place {}", r.id);
        }
    }
}

/// A key nothing has an entry for falls through to the canonical name.
///
/// This is the property that makes tone iteration safe — a half-finished theme
/// is a game with some untranslated words, not a game that will not start — so
/// it is asserted rather than assumed.
#[test]
fn a_missing_entry_falls_through_rather_than_failing() {
    let d = ThemeData::parse(&write_out(&TURTLE_DICK)).unwrap();
    assert_eq!(d.piece("A Component Nobody Wrote"), "A Component Nobody Wrote");
    assert_eq!(d.monster("Something Unthemed"), "Something Unthemed");
    assert_eq!(d.word("no-such-slug", "the default"), "the default");
    assert_eq!(d.note("Something Unthemed"), None);
    assert_eq!(d.place("no-such-door", "THE CANONICAL TITLE"), "THE CANONICAL TITLE");
}

/// A file this build cannot read says so in a sentence, and does not panic.
///
/// The same contract `load_json` gets in M1, written here first because the
/// theme file is the first data file GM2D ships and the failure mode is
/// identical.
#[test]
fn an_unreadable_theme_file_explains_itself() {
    // A file that parses but is a different kind of file. This is the case
    // worth a good message: it is what happens when somebody loads a save into
    // the theme slot, and the shape is close enough that serde will not object.
    let mut wrong = ThemeData::of(&PLAIN);
    wrong.format = "gm2d-save".to_string();
    let err = ThemeData::parse(&serde_json::to_string(&wrong).unwrap()).unwrap_err();
    assert!(
        err.contains(FORMAT) && err.contains("gm2d-save"),
        "a wrong format should name both what was wanted and what arrived: {err}"
    );

    // A file missing the envelope entirely is caught a step earlier, by serde.
    // Its message is still a sentence rather than a panic, which is the whole
    // requirement.
    let err = ThemeData::parse("{}").unwrap_err();
    assert!(err.starts_with("this is not a theme file"), "{err}");

    let mut future = ThemeData::of(&PLAIN);
    future.version = VERSION + 1;
    let text = serde_json::to_string(&future).unwrap();
    let err = ThemeData::parse(&text).unwrap_err();
    assert!(
        err.contains(&(VERSION + 1).to_string()) && err.contains(&VERSION.to_string()),
        "a future version should name both versions: {err}"
    );

    let err = ThemeData::parse("this is not json at all").unwrap_err();
    assert!(!err.is_empty(), "garbage should still get a sentence");
}
