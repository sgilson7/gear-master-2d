//! A theme, as data.
//!
//! # Why this exists
//!
//! `PLAN.md` §2 requires that content live in `data/*.json` and never in Rust
//! literals, so a tone edit is a data edit. `theme.rs` already gets the hard
//! half of that right — it treats every name the engine works with as a *key*
//! rather than a label, and a missing entry falls through to the canonical
//! name, so a half-finished theme is a game with some untranslated words in it
//! rather than a game that does not start. What it gets wrong for GM2D's
//! purposes is only *where the tables live*.
//!
//! This module is the bridge. [`ThemeData`] is the same shape, owned and
//! serialisable, with [`ThemeData::of`] to write one out and the same lookups
//! to read one back.
//!
//! # What is not done yet
//!
//! `Theme`'s methods all take `&'static self`, so the statics are still the
//! ones the engine reads. Flipping that — loading `data/theme.td.json` at
//! startup and deleting the statics — is M2's, when there is a UI reading a
//! theme at all. Doing it now would be a two-thousand-line refactor with no
//! consumer to prove it against.
//!
//! What M0 owes is the data file and the evidence that it is lossless, and
//! `theme_data.rs`'s tests are that evidence: every key of every table, read
//! back off the JSON, answers what the static answers.

use serde::{Deserialize, Serialize};

/// One place on the road, retold. The owned twin of `theme::Retold`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetoldData {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prose: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub landings: Vec<String>,
}

/// The words items are named out of. The owned twin of `naming::Naming`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingData {
    pub weapon_bases: Vec<String>,
    pub helmet_bases: Vec<String>,
    pub chest_bases: Vec<String>,
    pub glove_bases: Vec<String>,
    pub greave_bases: Vec<String>,
    pub attributives: Vec<String>,
    pub suffixes: Vec<String>,
    pub epithets: Vec<String>,
}

/// One complete set of words for the game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeData {
    /// Bumped when this file's shape changes incompatibly, like every other
    /// data file GM2D reads.
    pub format: String,
    pub version: u32,

    pub id: String,
    pub label: String,
    pub blurb: String,
    pub story: Vec<String>,
    pub naming: NamingData,

    /// Canonical name -> the name to show. Written as objects rather than
    /// pair-arrays so a human editing the file can find a key by searching for
    /// it, which is the whole reason it is a file.
    pub pieces: Vec<[String; 2]>,
    pub monsters: Vec<[String; 2]>,
    pub classes: Vec<[String; 2]>,
    pub words: Vec<[String; 2]>,
    /// Whole words swapped inside prose the engine wrote. Matched
    /// case-insensitively on whole words, so "mana" becomes "Funny" and
    /// "manacle" is left alone.
    pub vocabulary: Vec<[String; 2]>,
    pub notes: Vec<[String; 2]>,
    /// `(term replaced, new term, new definition)`. An empty first field adds
    /// an entry the plain game has not got.
    pub glossary: Vec<[String; 3]>,
    /// Keyed by canonical monster name.
    pub cutscenes: Vec<(String, Vec<String>)>,
    pub told: Vec<RetoldData>,
}

pub const FORMAT: &str = "gm2d-theme";
pub const VERSION: u32 = 1;

fn pairs(src: &[(&str, &str)]) -> Vec<[String; 2]> {
    src.iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect()
}

fn strs(src: &[&str]) -> Vec<String> {
    src.iter().map(|s| s.to_string()).collect()
}

impl ThemeData {
    /// Read a shipped theme out as data.
    pub fn of(t: &'static crate::theme::Theme) -> Self {
        ThemeData {
            format: FORMAT.to_string(),
            version: VERSION,
            id: t.id.to_string(),
            label: t.label.to_string(),
            blurb: t.blurb.to_string(),
            story: strs(t.story),
            naming: NamingData {
                weapon_bases: strs(t.naming.weapon_bases),
                helmet_bases: strs(t.naming.helmet_bases),
                chest_bases: strs(t.naming.chest_bases),
                glove_bases: strs(t.naming.glove_bases),
                greave_bases: strs(t.naming.greave_bases),
                attributives: strs(t.naming.attributives),
                suffixes: strs(t.naming.suffixes),
                epithets: strs(t.naming.epithets),
            },
            pieces: pairs(t.pieces),
            monsters: pairs(t.monsters),
            classes: pairs(t.classes),
            words: pairs(t.words),
            vocabulary: pairs(t.vocabulary),
            notes: pairs(t.notes),
            glossary: t
                .glossary
                .iter()
                .map(|(a, b, c)| [a.to_string(), b.to_string(), c.to_string()])
                .collect(),
            cutscenes: t
                .cutscenes
                .iter()
                .map(|(m, lines)| (m.to_string(), strs(lines)))
                .collect(),
            told: t
                .told
                .iter()
                .map(|r| RetoldData {
                    id: r.id.to_string(),
                    title: r.title.to_string(),
                    prose: strs(r.prose),
                    entry: strs(r.entry),
                    landings: strs(r.landings),
                })
                .collect(),
        }
    }

    /// Parse, refusing a file this build cannot read — in a sentence, not a
    /// panic. The same contract `load_json` will have in M1.
    pub fn parse(text: &str) -> Result<Self, String> {
        let d: ThemeData =
            serde_json::from_str(text).map_err(|e| format!("this is not a theme file: {e}"))?;
        if d.format != FORMAT {
            return Err(format!(
                "expected a {FORMAT} file and this says {:?}",
                d.format
            ));
        }
        if d.version > VERSION {
            return Err(format!(
                "this theme is version {} and this build reads up to {VERSION}",
                d.version
            ));
        }
        Ok(d)
    }

    fn look(table: &[[String; 2]], key: &str) -> Option<String> {
        table.iter().find(|p| p[0] == key).map(|p| p[1].clone())
    }

    /// The name to show for a component, or the canonical one.
    ///
    /// Falls through, exactly as the static does. **A missing entry must never
    /// be an error**: that fall-through is what makes a half-finished theme
    /// safe to ship and tone iteration cheap.
    pub fn piece(&self, canonical: &str) -> String {
        Self::look(&self.pieces, canonical).unwrap_or_else(|| canonical.to_string())
    }

    pub fn monster(&self, canonical: &str) -> String {
        Self::look(&self.monsters, canonical).unwrap_or_else(|| canonical.to_string())
    }

    pub fn class(&self, canonical: &str) -> String {
        Self::look(&self.classes, canonical).unwrap_or_else(|| canonical.to_string())
    }

    pub fn word(&self, slug: &str, default: &str) -> String {
        Self::look(&self.words, slug).unwrap_or_else(|| default.to_string())
    }

    pub fn note(&self, monster: &str) -> Option<String> {
        Self::look(&self.notes, monster)
    }

    pub fn place(&self, id: &str, canonical: &str) -> String {
        self.told
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.title.clone())
            .unwrap_or_else(|| canonical.to_string())
    }
}
