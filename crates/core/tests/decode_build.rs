//! Read a shared ch code and print the board it describes.
//!
//! `cargo test -p gm2d-core --test decode_build -- --ignored --nocapture`

use gm2d_core::piece::{SlotKind, CATALOG};









#[test]
fn probe_boss_prices() {
    use gm2d_core::piece::{BOSS_ONLY, CATALOG};
    use gm2d_core::rating::{resale_price, shop_price};
    let mut worst = 0;
    for name in BOSS_ONLY {
        let d = CATALOG.iter().find(|d| d.name == *name).unwrap();
        println!("{:>24}  shop {:>5}  resale {:>5}", name, shop_price(d), resale_price(d));
        worst = worst.max(resale_price(d));
    }
    println!("worst boss resale: {worst}");
    let ordinary: i32 = CATALOG.iter()
        .filter(|d| !gm2d_core::piece::is_off_the_scale(d.name))
        .map(resale_price).max().unwrap();
    println!("best ordinary resale: {ordinary}");
}











/// **Re-pinned when the book recipe caught up with §2.2.** The friend's weapon
/// grid went from two items to three, and the third is the point: Chained
/// Codex, Gravebloom Ink, Pilgrim Alignment and Forking Bead were **loose
/// pieces** on that board - the strict recipe wanted an ink *and* refused an
/// alignment, so they could not bind to anything - and they are a book weapon
/// now.
///
/// The friend's board is 17 items and became 18. It still clears 48 of 50,
/// still loses to THE UNWOUND, and its median time-to-kill moved 8.15s to
/// 8.65s. That is the design's own risk realised and inspected rather than
/// re-blessed: "relaxing a recipe cannot stop a board assembling, but it can
/// make a loose pile *start* assembling".
///
/// The three shared boards, item by item, by name.
///
/// The floors above say a dense board must not collapse. This says what it
/// comes back as. Counts and ladder results agreed while the reconstruction
/// was wrong - nineteen weapon pieces coming back as one item is still one
/// item, and one item still fights - so the only thing that could have caught
/// it was looking at *which pieces ended up in which item*, which nothing did.
///
/// Written out in full rather than derived, because a derived expectation
/// would be the same code twice and would agree with itself while both halves
/// were wrong. This is what the boards hold; a diff here is a board coming
/// back different, and the reason has to be found before the table is edited.
///
/// Regenerate with `probe_membership`.
#[allow(clippy::type_complexity)]
const MEMBERSHIP: &[(&str, &[(SlotKind, &str)])] = &[
    ("owner", &[
        (SlotKind::Helmet, "Aegis Crown + Warding Plate"),
        (SlotKind::Helmet, "Bone Frame + Crown of the Deep + Layered Plating"),
        (SlotKind::Helmet, "Eighth Ray Crown + Heartwood Crest + Reckoning Plate"),
        (SlotKind::Chest, "Adamant Base + Seedbed Layer"),
        (SlotKind::Chest, "Deep Roots Base + Emberplate + Runic Weave + Scale Layer"),
        (SlotKind::Chest, "Riveted Layer + Runed Lining + Wellspring Base"),
        (SlotKind::Gloves, "Bloomguard + Padded Mold"),
        (SlotKind::Gloves, "Breaker's Fist + Sovereign Mold"),
        (SlotKind::Gloves, "Channeling Mold + Henpeck's Cell Keys + Rootwoven Material"),
        (SlotKind::Gloves, "Gripping Mold + Plaguewalkers"),
        (SlotKind::Gloves, "Iron Band + Quickfinger Mold + Seal of Power + Spun Material"),
        (SlotKind::Gloves, "Ring of Embers + Thornweald Grip + Wrathful Talons"),
        (SlotKind::Greaves, "Anchor Material + Anchored Sole + Warded Plating"),
        (SlotKind::Greaves, "Anchor Material + Plain Sole + Scaled Plating"),
        (SlotKind::Greaves, "Mage's Sandals + Studded Sole"),
        (SlotKind::Greaves, "Scaled Material + Striding Mold + Tin Plating"),
        (SlotKind::Greaves, "Scrying Lens + Sevenleague Boots + Widow's Sole"),
        (SlotKind::Weapon, "Forking Bead + Gravebound Haft + Loaded Fob + Witchglass Shard + Worldsplitter"),
        (SlotKind::Weapon, "Iron Blade + Oak Handle + Sawtooth Edge"),
    ]),
    ("friend", &[
        (SlotKind::Helmet, "Asker's Monocle + Mage's Circlet + Runed Plating"),
        (SlotKind::Helmet, "Runed Plating + Scaled Plating + Tin Frame"),
        (SlotKind::Helmet, "Visor of Focus + Witch's Hat"),
        (SlotKind::Chest, "Adamant Base + Lightweave + Plate Layer + Wickstub"),
        (SlotKind::Chest, "Becalming Layer + Quilted Base + Sigil Layer"),
        (SlotKind::Chest, "Becalming Layer + Wellspring Base"),
        (SlotKind::Chest, "Rimeguard Base + Seedbed Layer"),
        (SlotKind::Gloves, "Boiled Leather + Bramble Mold"),
        (SlotKind::Gloves, "Braced Mold + Mage's Sandals + Seal of the Grove + Tithe Ring"),
        (SlotKind::Gloves, "Empowering Mold + Tallykeeper's Weave"),
        (SlotKind::Gloves, "Hexer's Reckoning + Scaled Material + Seal of the Grove"),
        (SlotKind::Greaves, "Anchor Material + Layered Plating + Tarpit Sole"),
        (SlotKind::Greaves, "Anchored Sole + Ironthread Material"),
        (SlotKind::Greaves, "Pilgrim's Sole + Spun Material"),
        (SlotKind::Greaves, "Thornweald Grip + Treadmill Sole"),
        (SlotKind::Weapon, "Blood Rite + Chained Codex + Forking Bead + Gravebloom Ink + Pilgrim Alignment"),
        (SlotKind::Weapon, "Blood Rite + Hollow Sphere + Mirror Ward"),
        (SlotKind::Weapon, "Last Rite + Mirrorcast + The Seeker's Tears"),
    ]),
    ("perfect", &[
        (SlotKind::Helmet, "Bronze Frame + Runed Plating + Warlord's Crest"),
        (SlotKind::Helmet, "Deadweight Plating + Warded Frame"),
        (SlotKind::Chest, "Rimeguard Base + Starlit Mantle + Wrathbreaker"),
        (SlotKind::Chest, "Sackcloth Base + Woven Underlayer"),
        (SlotKind::Gloves, "Bloodring + Bulwark Material + Flaying Mold + Henpeck's Cell Keys"),
        (SlotKind::Gloves, "Coven Mold + Tallykeeper's Weave"),
        (SlotKind::Gloves, "Deft Mold + Mage's Sandals + Signet of Iron"),
        (SlotKind::Gloves, "Plaguewalkers + Spiked Vambrace"),
        (SlotKind::Greaves, "Greave Mold + Rootbound Material"),
        (SlotKind::Greaves, "Leather Material + Reckoning Plate + Stormstep Mold"),
        (SlotKind::Greaves, "Pilgrim's Sole + Witch's Claw"),
        (SlotKind::Greaves, "Rootbound Material + Stumblefoot Mold"),
        (SlotKind::Weapon, "Balance Weight + Bulwark Vial + Cull + Iron Blade + Oak Handle"),
        (SlotKind::Weapon, "Codex Interminable + Shatterbolt + Tidewrack Ink"),
        (SlotKind::Weapon, "Sawtooth Edge + Toolwright's Grip"),
    ]),
];






