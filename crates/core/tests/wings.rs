//! M22.1 — a wing is a shelf with a host.
//!
//! A wing is a counter standing inside somebody else's town. High Wick's
//! seventeen lines come down one country without High Wick becoming the town
//! they are sold in, and the clerk's desk is a counter with errands and no
//! stock at all.
//!
//! **Nothing here is authored yet.** M22.1 is the rule with no content behind
//! it, so every one of these builds the shelf it is asking about.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::shop::{self, ShopsData};
use gm2d_core::world::{self, WorldState};

mod common;

const D: Difficulty = Difficulty::Easy;

/// A shops file with one wing bolted onto it, built by editing the shipped one.
fn with_a_wing(host: &str, arrives: &str) -> ShopsData {
    let mut d = data::shops();
    d.towns.push(gm2d_core::shop::TownShelf {
        id: "a-wing".into(),
        wing_of: Some(host.into()),
        arrives: Some(arrives.into()),
        name: "A Wing".into(),
        commissions: Vec::new(),
        stock: vec!["Oak Handle".into(), "Iron Blade".into()],
    });
    d
}

/// **A wing is not on the counter until it has arrived, and then it is.**
///
/// The whole of what the field is for. `shop::wings` is the one answer and the
/// page draws its list — a screen that decided for itself whether a counter was
/// open would be the second rulebook this project has spent six blocks not
/// building.
#[test]
fn a_wing_is_not_on_the_counter_until_it_arrives() {
    let d = with_a_wing("the-end-of-all-gears", "a-mark");
    let mut state = WorldState::default();
    assert!(
        shop::wings(&d, "the-end-of-all-gears", &state).is_empty(),
        "a wing was on the counter before it arrived"
    );
    state.flags.push("a-mark".into());
    let open = shop::wings(&d, "the-end-of-all-gears", &state);
    assert_eq!(open.len(), 1, "the wing did not arrive");
    assert_eq!(open[0].id, "a-wing");
    // And it is the *host's* counter and nobody else's.
    assert!(
        shop::wings(&d, "kettleworks", &state).is_empty(),
        "a wing of one town turned up in another"
    );
}

/// **A sale is keyed by the wing, and the host selling the same thing does not
/// spend it.**
///
/// `WorldState::bought` is `(id, index)` and *the index is the identity* — the
/// reason a sold entry is greyed and left where it was rather than dropped. So
/// a wing keeps its own id and its own numbering, which is the whole argument
/// for `wing_of` instead of appending seventeen lines to the host's stock:
/// appending would move what somebody already bought.
#[test]
fn a_wing_sale_is_keyed_by_the_wing_and_survives_the_host_selling() {
    let d = with_a_wing("the-end-of-all-gears", "a-mark");
    // The host sells its own entry 0 and nothing else.
    let sold = vec![("the-end-of-all-gears".to_string(), 0u16)];
    let host = shop::shelf(&d, "the-end-of-all-gears", &sold);
    assert!(host[0].sold, "the host's own entry 0 is not marked sold");
    let wing = shop::shelf(&d, "a-wing", &sold);
    assert_eq!(wing.len(), 2);
    assert!(
        !wing[0].sold,
        "the host selling its entry 0 spent the wing's entry 0 as well"
    );
    // And the other way round.
    let sold = vec![("a-wing".to_string(), 1u16)];
    assert!(shop::shelf(&d, "a-wing", &sold)[1].sold);
    assert!(
        !shop::shelf(&d, "the-end-of-all-gears", &sold)[1].sold,
        "the wing selling its entry 1 spent the host's"
    );
}

/// **A finished errand is a thing that has happened, and until M22 nothing but
/// the quest log knew.**
///
/// `world::met` is the one predicate and `hidden_until`, `hidden_until_all`,
/// `needs_all`, a floor's `cleared`, a drain's `when`, a wing's `arrives` and a
/// post's `named.when` all ask it. `quests_done` joining it is one line and **no
/// new save field**: the list already round-trips.
#[test]
fn a_finished_errand_meets_a_hidden_until() {
    let mut state = WorldState::default();
    let key = format!("{}an-errand", world::DONE);
    assert!(!world::met(&state, &key));
    state.quests_done.push("an-errand".into());
    assert!(world::met(&state, &key), "a finished errand is not met");
    // **And a bare id never matches an errand**, because the namespaces
    // collide: an errand and an event are both called `the-tenth-survey`, and
    // an unprefixed read opened the way under the Wextreen flat for somebody
    // who had finished the errand and never found the sheet.
    assert!(
        !world::met(&state, "an-errand"),
        "a bare key matched an errand, and an errand id is also an event id"
    );

    // And a place hidden behind one is there once it is done. Built by
    // borrowing a shipped place and moving its condition, because `PlaceDef` is
    // a map file's shape and writing one out by hand here would be a second
    // copy of it.
    let w = data::map("west-bambulon", D);
    let mut p = w
        .places
        .iter()
        .find(|p| p.hidden_until.is_some())
        .expect("a hidden place on the first map")
        .clone();
    p.hidden_until = Some(key.clone());
    p.hidden_until_all.clear();
    let allowed = gm2d_core::world::Allowances::default();
    assert!(
        world::place_is_there(&p, &state, &allowed),
        "a place behind a finished errand is still not there"
    );
    state.quests_done.clear();
    assert!(!world::place_is_there(&p, &state, &allowed));
}

/// **`met` and `marks` are one question asked two ways, and they must agree.**
///
/// A predicate and a list that answer the same thing from two bodies of code is
/// the *rule with two homes* this project has paid for seven times — and it
/// would have happened here: `marks()` was `answered` and `flags`, and teaching
/// only the closure about `quests_done` would have left `sealed_because`,
/// `still_wanted`, `opens_onto`, `Floor::cleared`, the drains and
/// `Requirement::Flag` reading a shorter list.
#[test]
fn met_is_marks_with_one_key_in_it() {
    let mut state = WorldState::default();
    state.answered.push("a".into());
    state.flags.push("b".into());
    state.quests_done.push("c".into());
    let marks = state.marks();
    for k in ["a", "b", "c", "done:c", "done:a", "d", ""] {
        assert_eq!(
            world::met(&state, k),
            marks.iter().any(|m| m == k),
            "met and marks disagree about {k:?}"
        );
    }
}

/// **A wing arrives on something, and a town does not.**
///
/// `ShopsData::parse`'s half, proved through `parse` rather than restated: a
/// wing that is always there is the host's own stock wearing a second name, and
/// a town that arrives is a town that is not on the map yet — which is what
/// `hidden_until` is for.
///
/// This is `4a3ae9a`'s finding obeyed in advance — *a check whose negative test
/// cannot be made to fail through the check is a check nobody has proved.*
#[test]
fn a_wing_arrives_or_it_is_the_hosts_own() {
    let ok = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"a","stock":["Oak Handle"]},
        {"id":"b","wing_of":"a","arrives":"m","name":"B","stock":["Iron Blade"]}]}"#;
    ShopsData::parse(ok).expect("a wing with a host and an arrival loads");

    let no_arrival = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"a","stock":["Oak Handle"]},
        {"id":"b","wing_of":"a","name":"B","stock":["Iron Blade"]}]}"#;
    let why = ShopsData::parse(no_arrival).expect_err("a wing that is always there loaded");
    assert!(why.contains("arrives on nothing"), "{why}");

    let town_arrives = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"a","arrives":"m","stock":["Oak Handle"]}]}"#;
    let why = ShopsData::parse(town_arrives).expect_err("a town that arrives loaded");
    assert!(why.contains("arrives by being drawn on a map"), "{why}");

    let itself = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"a","wing_of":"a","arrives":"m","name":"A","stock":["Oak Handle"]}]}"#;
    let why = ShopsData::parse(itself).expect_err("a wing of itself loaded");
    assert!(why.contains("wing of itself"), "{why}");

    let nameless = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"a","stock":["Oak Handle"]},
        {"id":"b","wing_of":"a","arrives":"m","stock":["Iron Blade"]}]}"#;
    let why = ShopsData::parse(nameless).expect_err("a nameless wing loaded");
    assert!(why.contains("no name to draw"), "{why}");

    // And the shipped file still loads, which is the half a mutation cannot
    // prove.
    ShopsData::parse(gm2d_core::data::SHOPS_JSON).expect("the shipped shelves load");
}

/// **A wing's host is a town on a map — and that is the lint's, not the
/// parse's.**
///
/// The split is decision 5: `ShopsData::parse` owns what a shops file can be
/// asked about *itself*, and whether a host is drawn anywhere needs the map
/// files, which `parse` has no business reading. The negative test hands a
/// mutated `SHOPS_JSON` to `parse` and asserts what comes back is a file this
/// lint then refuses — exactly the shape `a_bargain_is_never_on_the_barrel`
/// took in `4a3ae9a`.
#[test]
fn a_wing_has_a_host_on_a_map() {
    let towns = data::towns_on_the_map();
    let shops = data::shops();
    let ids: Vec<&str> = shops.towns.iter().map(|t| t.id.as_str()).collect();
    for t in &shops.towns {
        let Some(host) = &t.wing_of else { continue };
        assert!(
            ids.contains(&host.as_str()),
            "{} is a wing of {host}, which has no shelf at all",
            t.id
        );
        assert!(
            towns.contains(host),
            "{} is a wing of {host}, which is on no map — a counter inside a town \
             that does not exist is a counter nobody stands at",
            t.id
        );
    }
    // The negative: a wing hung on a shelf that is on no map. Built through
    // `parse`, so the mutation is proved to be a file this build would load.
    let staged = r#"{"format":"gm2d-shops","version":1,"towns":[
        {"id":"high-wick","stock":["Oak Handle"]},
        {"id":"a-wing","wing_of":"high-wick","arrives":"m","name":"A Wing","stock":["Iron Blade"]}]}"#;
    let d = ShopsData::parse(staged).expect("the mutation is a file this build loads");
    let bad = d
        .towns
        .iter()
        .filter(|t| t.wing_of.is_some())
        .find(|t| !towns.contains(t.wing_of.as_ref().unwrap()));
    assert!(
        bad.is_some(),
        "the mutation did not produce a wing on a host that is on no map"
    );
}

/// **Every shelf is a town on a map, a wing of one, or named as staged.**
///
/// `avail.rs`'s orphan rule, widened for the one shape it could not see: a wing
/// is never a town, so before this every wing would have read as a shelf for
/// nowhere. *Content waiting for a map is fine and content waiting for nothing
/// is an orphan, and the only difference is somebody having written the name
/// down* — a wing's host is where it is written down.
#[test]
fn every_shelf_is_a_town_a_wing_or_staged() {
    // **Asked of a file that has wings in it, because the shipped one has
    // none.** `data::shelves_on_the_map` reads a compiled-in constant, so a
    // check written against it today would assert *the towns* whatever the rule
    // said — the `false &&` that breaks the rule passes it. `shop::shelves_among`
    // is the rule with the file handed to it, which is why it exists.
    let towns: Vec<String> =
        ["a", "b"].iter().map(|s| s.to_string()).collect();
    let d = ShopsData::parse(
        r#"{"format":"gm2d-shops","version":1,"towns":[
            {"id":"a","stock":["Oak Handle"]},
            {"id":"b","stock":["Oak Handle"]},
            {"id":"a-wing","wing_of":"a","arrives":"m","name":"A Wing","stock":["Iron Blade"]},
            {"id":"staged","stock":["Iron Blade"]},
            {"id":"staged-wing","wing_of":"staged","arrives":"m","name":"Staged Wing","stock":["Iron Blade"]}]}"#,
    )
    .expect("the fixture loads");
    let reachable = shop::shelves_among(&d, &towns);
    assert!(reachable.contains(&"a-wing".to_string()), "a wing of a drawn town is not reachable");
    assert!(
        !reachable.contains(&"staged-wing".to_string()),
        "a wing of a shelf that is on no map is reachable, and it undercuts nothing"
    );
    assert!(!reachable.contains(&"staged".to_string()));
    assert_eq!(reachable.len(), 3, "{reachable:?}");

    // And the shipped file: every shelf is a town on a map, a wing of one, or
    // staged, with nothing in between.
    let towns = data::towns_on_the_map();
    let shops = data::shops();
    for id in &data::shelves_on_the_map() {
        let is_town = towns.contains(id);
        let is_wing = shops
            .town(id)
            .and_then(|t| t.wing_of.clone())
            .is_some_and(|h| towns.contains(&h));
        assert!(is_town || is_wing, "{id} is reachable and is neither a town nor a wing");
    }
}

/// **A counter you are standing in front of is the town and its open wings.**
///
/// One question, three callers: which shelf you may buy off, which errands the
/// guild lists, and whether an errand may be taken here. The shim was about to
/// answer it three times — `take_quest` compares where you are standing against
/// an errand's `giver`, and a wing's giver is the wing.
#[test]
fn counters_at_is_the_town_and_what_has_arrived() {
    let d = with_a_wing("the-end-of-all-gears", "done:a-mark");
    let mut state = WorldState::default();
    assert_eq!(
        shop::counters_at(&d, "the-end-of-all-gears", &state),
        vec!["the-end-of-all-gears".to_string()]
    );
    state.quests_done.push("a-mark".into());
    assert_eq!(
        shop::counters_at(&d, "the-end-of-all-gears", &state),
        vec!["the-end-of-all-gears".to_string(), "a-wing".to_string()],
        "the host comes first, because the host is the town"
    );
}

/// **The wings this block authored, by name.**
///
/// **This replaced `no_wing_is_authored_yet` in the commit that made it
/// false**, which is what a test pinning the behaviour you are changing is
/// for. That one asserted the rule was unused, because the only way to tell a
/// rule that is not yet used from one that is quietly in use is to say so; M22.2
/// put the clerk's desk on the third town's counter and it went red, correctly.
///
/// Named rather than counted, the way `common::UNWRITTEN` is: a list that
/// quietly grew is a list that has gone stale, and a wing is a counter appearing
/// in somebody's town — which is not a thing that should appear without anybody
/// deciding to put it there.
#[test]
fn the_wings_are_the_ones_this_block_authored() {
    let shops = data::shops();
    let mut wings: Vec<&str> =
        shops.towns.iter().filter(|t| t.wing_of.is_some()).map(|t| t.id.as_str()).collect();
    wings.sort();
    assert_eq!(wings, vec!["the-clerks-desk"], "the wings are {wings:?}");
    for w in &wings {
        let t = shops.town(w).expect("a wing that exists");
        assert_eq!(t.wing_of.as_deref(), Some("the-third-town"), "{w} stands somewhere else");
    }
    let _ = common::geared_from(&["the-end-of-all-gears"]);
    let _ = D;
}

// ------------------------------------------------------------ M22.2, the desk

/// **The clerk is not there until she is sent for.**
///
/// The wing's whole point, asked of the shipped file rather than of a fixture:
/// before `send-for-the-clerk` is handed in the third town has one counter, and
/// after it there are two.
#[test]
fn the_clerk_is_not_there_until_sent_for() {
    let shops = data::shops();
    let mut state = WorldState::default();
    assert_eq!(
        shop::counters_at(&shops, "the-third-town", &state),
        vec!["the-third-town".to_string()],
        "the desk is on the counter before anybody sent for the clerk"
    );
    // **Taken and not finished is still not there**, which is the second visit
    // this project keeps forgetting to check: `quests_taken` is not
    // `quests_done`.
    state.quests_taken.push("send-for-the-clerk".into());
    assert_eq!(
        shop::counters_at(&shops, "the-third-town", &state).len(),
        1,
        "taking the errand brought the clerk down"
    );
    state.quests_done.push("send-for-the-clerk".into());
    assert_eq!(
        shop::counters_at(&shops, "the-third-town", &state),
        vec!["the-third-town".to_string(), "the-clerks-desk".to_string()]
    );
}

/// **The third town wants something once the clerk is down, and nothing until
/// then.**
///
/// The lint `avail.rs` holds over a town, asked of the **counters** standing in
/// it — which is what a player sees. `the-third-town` itself is still
/// `UNWRITTEN` in M22.2 and hands out nothing of its own; what changes is that
/// there is now somebody at a desk in it who does.
#[test]
fn the_third_town_wants_something_once_the_clerk_is_down() {
    let shops = data::shops();
    let quests = data::quests();
    let wants = |state: &WorldState| -> usize {
        shop::counters_at(&shops, "the-third-town", state)
            .iter()
            .map(|c| quests.at(c).len())
            .sum()
    };
    let mut state = WorldState::default();
    assert_eq!(wants(&state), 0, "the empty town wants something before the clerk");
    state.quests_done.push("send-for-the-clerk".into());
    assert!(wants(&state) > 0, "the clerk came down and wants nothing");
}

/// **The mirror inventory is given at the desk and nowhere else.**
///
/// Re-keyed from `high-wick`, which is on no map, to the counter that came down
/// off it. **Seam-free**: a save carries quest *ids* and not givers, so a
/// character who had taken it — nobody has, because High Wick has never been
/// stood in — keeps it.
#[test]
fn the_mirror_errand_is_given_at_the_desk_and_nowhere_else() {
    let quests = data::quests();
    let q = quests.get("the-long-mirror-inventory").expect("the errand");
    assert_eq!(q.giver, "the-clerks-desk");
    assert_eq!(
        gm2d_core::quest::QuestsData::turn_in_of(q),
        "the-clerks-desk",
        "it is handed back somewhere else"
    );
    assert_eq!(q.requires, vec!["send-for-the-clerk".to_string()]);
    // And nobody else offers it.
    let shops = data::shops();
    for t in &shops.towns {
        if t.id == "the-clerks-desk" {
            continue;
        }
        assert!(
            !quests.at(&t.id).iter().any(|x| x.id == q.id),
            "{} also offers the mirror inventory",
            t.id
        );
    }
}

/// **What the clerk is sent for is a fight down there and a word about it.**
///
/// A `Slay` rather than a `Bring`, and the plan says *of something the
/// Undercountry drops*: **nothing down there drops anything**, and a `Bring` is
/// answered by a component that already exists in the world. A `Slay`'s token is
/// created by the errand and cannot be got before it is asked for or after it is
/// finished, which is what *go down and come back with proof* wants.
///
/// The token is `A Word About the Cellar` — one of six rumour components the cut
/// campaign left behind, on no shelf and granted by nothing, which is the same
/// move M20 made with four dead casting pieces. **No new component**, so the
/// catalogue fingerprint does not move.
#[test]
fn the_clerk_is_sent_for_with_a_word_from_down_there() {
    use gm2d_core::quest::Goal;
    let quests = data::quests();
    let q = quests.get("send-for-the-clerk").expect("the errand");
    assert_eq!(q.giver, "kettleworks", "an empty town cannot ask for its own clerk");
    assert_eq!(q.requires, vec!["nobody-has-named-it".to_string()]);
    let Goal::Slay { creature, count, token } = &q.goal else {
        panic!("{:?} is not a slaying", q.goal)
    };
    assert_eq!(*count, 1);
    // **Something the Undercountry deals**, and the commonest of them, because
    // this errand is the arrival tale rather than the wall.
    let w = data::map("the-undercountry", D);
    let pool = &w.regions[0].enemies;
    assert!(
        pool.iter().any(|m| m.name == creature),
        "{creature} does not stand in the Undercountry's pool"
    );
    let commonest = pool
        .iter()
        .min_by_key(|m| gm2d_core::rating::creature_rating(m, D))
        .expect("a pool");
    assert_eq!(
        commonest.name, creature,
        "the errand asks for {creature} where the commonest draw is {}",
        commonest.name
    );
    // **A token nothing else wants**, which is the no-two-errands-share-a-tally
    // rule, and one the catalogue already had.
    assert!(
        gm2d_core::piece::CATALOG.iter().any(|d| d.name == token),
        "{token} is not in the catalogue"
    );
    let others = quests
        .quests
        .iter()
        .filter(|x| x.id != q.id)
        .filter(|x| x.goal.token() == Some(token.as_str()))
        .count();
    assert_eq!(others, 0, "{token} is somebody else's tally too");
    // And it pays money and no gear, so `common::geared_from` does not move.
    assert!(q.reward.is_empty() && q.enchs.is_empty() && q.rows.is_none(), "it pays gear");
    assert!(q.gold > 0);
}
