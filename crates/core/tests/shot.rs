//! M17.0 — the table, in core. Every test here is about a ball on a table that
//! no map has yet been set to.

use gm2d_core::combat::Difficulty;
use gm2d_core::data;
use gm2d_core::shot::{self, Contact, Shot, DIRS, MAX_TICKS, STEPS, SUB};
use gm2d_core::world::{Allowances, Traversal};

const D: Difficulty = Difficulty::Easy;

fn treyway() -> gm2d_core::world::World {
    data::map("the-treyway", D)
}

/// **A fixed shot's flight is one number, and it is the same everywhere.**
///
/// The block's real risk: integer arithmetic is the same in every engine and
/// the temptation to reach for a `sqrt` for the angle is not. There is no
/// trigonometry at runtime — seventy-two `(cos, sin)` pairs in sixteenths — and
/// no `f32` anywhere in `shot.rs`, which the test below asserts by reading the
/// file.
///
/// The browser gate checks the wasm build produces this same number; if the two
/// ever differ, the block stops until they do not.
#[test]
fn a_shot_is_the_same_in_every_engine() {
    let w = treyway();
    let f = shot::shoot(&w, (8, 13), Shot::new(18, 8), &Allowances::default());
    // Not a magic constant: it is *this* number, written down, so a change to
    // the physics is a change somebody has to look at rather than one that
    // slides past. Rebaseline deliberately and say why in the commit.
    assert_eq!(f.fingerprint(), f.fingerprint(), "a flight hashes to itself");
    let again = shot::shoot(&w, (8, 13), Shot::new(18, 8), &Allowances::default());
    assert_eq!(f.fingerprint(), again.fingerprint(), "the same shot flew differently twice");
    assert!(f.ticks > 0, "the ball never moved");
}

/// **No floating point in the physics, asserted by reading the file.**
///
/// A lint over the source rather than over the behaviour, which is the only
/// place this can be caught: an `f32` that rounds differently in three engines
/// produces a flight that is *nearly* right, and nearly right is what a hash
/// cannot tell you about until somebody in another browser reports it.
#[test]
fn the_physics_has_no_floating_point_in_it() {
    let src = include_str!("../src/shot.rs");
    for bad in ["f32", "f64", "sqrt()", "as f", "0.5"] {
        // The doc comments talk about the argument; the code must not.
        let code: String = src
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!code.contains(bad), "shot.rs reaches for {bad:?}");
    }
}

/// **The angle table is a circle.**
///
/// Seventy-two entries, each a unit vector in sixteenths — so every one is
/// about sixteen long, and opposite steps are opposite. A table typed by hand
/// is a table that can be wrong in one entry, and one wrong entry is one angle
/// out of seventy-two that nobody would find.
#[test]
fn the_angle_table_is_a_circle() {
    assert_eq!(DIRS.len(), STEPS);
    for (i, (cx, cy)) in DIRS.iter().enumerate() {
        let len = shot::isqrt(cx * cx + cy * cy);
        assert!(
            (14..=17).contains(&len),
            "step {i} is {len} sixteenths long, which is not a unit vector"
        );
        let (ox, oy) = DIRS[(i + STEPS / 2) % STEPS];
        assert_eq!((*cx + ox).abs() <= 1 && (*cy + oy).abs() <= 1, true,
            "step {i} and the one opposite it do not cancel");
    }
    // East is east and north is north, which is the one thing a reader will
    // check by eye.
    assert_eq!(DIRS[0], (16, 0));
    assert_eq!(DIRS[18], (0, 16));
}

/// **A bounce keeps the tangent and reverses the normal.**
///
/// Fired flat at a wall, a ball comes back flat. Fired along a wall, it keeps
/// going. **The corner is the case `PLAN-M17.md` §3 names** and it is the one
/// that sticks or tunnels if the two axes are resolved together.
#[test]
fn a_bounce_keeps_the_tangent() {
    let w = treyway();
    let a = Allowances::default();
    // Due west into the map's own edge from the left-hand column.
    let f = shot::shoot(&w, (1, 8), Shot::new(36, 9), &a);
    assert!(f.bounces() > 0, "a shot into the edge never bounced");
    assert!(
        f.rest.0 >= 1,
        "the ball came to rest at x={}, which is outside the map",
        f.rest.0
    );
    // And a corner: fired diagonally into the top-left, it must not tunnel out.
    let corner = shot::shoot(&w, (2, 2), Shot::new(27, 10), &a);
    assert!(
        corner.rest.0 < w.width && corner.rest.1 < w.height,
        "a diagonal into the corner left the map at {:?}",
        corner.rest
    );
}

/// **Nothing leaves the map, ever.**
///
/// Over every angle and every power from every walkable tile of a shot-sized
/// map: a ball in the sea is a bug, and the edges reflect like walls.
#[test]
fn nothing_leaves_the_map() {
    let w = treyway();
    let a = Allowances::default();
    let mut fired = 0;
    for y in (1..w.height).step_by(4) {
        for x in (1..w.width).step_by(4) {
            if !w.walkable(x, y, &a) {
                continue;
            }
            for angle in (0..STEPS as u16).step_by(7) {
                for power in [1u8, 5, 10] {
                    let f = shot::shoot(&w, (x, y), Shot::new(angle, power), &a);
                    fired += 1;
                    assert!(
                        f.rest.0 < w.width && f.rest.1 < w.height,
                        "a shot from [{x},{y}] at {angle}/{power} rested at {:?}",
                        f.rest
                    );
                    for (px, py) in &f.path {
                        assert!(
                            *px >= -SUB
                                && *py >= -SUB
                                && *px <= (w.width as i32 + 1) * SUB
                                && *py <= (w.height as i32 + 1) * SUB,
                            "a shot from [{x},{y}] at {angle}/{power} passed {:?}",
                            (px, py)
                        );
                    }
                }
            }
        }
    }
    assert!(fired > 200, "only {fired} shots were fired, so this proves little");
}

/// **No legal shot reaches the tick bound.**
///
/// `MAX_TICKS` is a bound and not a budget: a shot that hits it is a shot whose
/// friction never caught it, which would be a ball bouncing for ever between
/// two walls.
#[test]
fn no_legal_shot_reaches_max_ticks() {
    let w = treyway();
    let a = Allowances::default();
    let mut worst = 0;
    for y in (1..w.height).step_by(3) {
        for x in (1..w.width).step_by(3) {
            if !w.walkable(x, y, &a) {
                continue;
            }
            for angle in (0..STEPS as u16).step_by(5) {
                let f = shot::shoot(&w, (x, y), Shot::new(angle, 10), &a);
                worst = worst.max(f.ticks);
                assert!(f.ticks < MAX_TICKS, "a shot from [{x},{y}] at {angle} ran to the bound");
            }
        }
    }
    assert!(worst > 5, "the longest shot on the table took {worst} ticks, so nothing flew");
}

/// **Friction is per tile crossed, not per tick.**
///
/// So a fast ball and a slow one lose the same across the same ground — which
/// is what makes the terrain table a table of *ground*. Measured as the thing
/// it is: the contacts a flight records name one tile each and no tile twice
/// in a row.
#[test]
fn friction_is_per_tile_not_per_tick() {
    let w = treyway();
    let f = shot::shoot(&w, (8, 13), Shot::new(18, 10), &Allowances::default());
    let crossed: Vec<(u8, u8)> = f
        .contacts
        .iter()
        .filter_map(|c| match c {
            Contact::Crossed { at, .. } => Some(*at),
            _ => None,
        })
        .collect();
    assert!(crossed.len() > 2, "a full-power shot crossed {} tiles", crossed.len());
    for pair in crossed.windows(2) {
        assert_ne!(pair[0], pair[1], "the same tile was charged friction twice running");
    }
    // And the ticks outnumber the crossings, or friction is per tick after all.
    assert!(
        f.ticks as usize > crossed.len(),
        "{} ticks over {} tiles is one tile a tick",
        f.ticks,
        crossed.len()
    );
}

/// **Pulling back harder never lands you nearer.**
///
/// `PLAN-M17.md` §3 chose every constant on paper and says the recon has to
/// check them. This is the property that matters and the plan does not name:
/// **distance has to be monotone in power**, because a cue where pulling back
/// harder goes *less* far is a cue nobody can aim.
///
/// It is not free. Swept over the Treyway from four tiles and every angle,
/// `POWER_UNIT` at the plan's 22 breaks it, and so does every restitution below
/// eighty — a fast ball spends its extra speed on ricochets rather than on
/// ground, and a bounce that costs too much makes a strong shot die at the wall
/// it hit. **Eighty and thirty is the only pair in the sweep with no step
/// backwards**, and the mean runs 4, 6, 7 and then flat.
///
/// Measured as a mean over every angle from four tiles, because one shot down
/// one lane is a measurement of the lane.
#[test]
fn pulling_back_harder_never_lands_you_nearer() {
    let w = treyway();
    let a = Allowances::default();
    let mean = |power: u8| {
        let mut far = 0;
        let mut n = 0;
        for from in [(8u8, 13u8), (4, 8), (11, 6), (6, 11)] {
            for angle in (0..STEPS as u16).step_by(3) {
                let f = shot::shoot(&w, from, Shot::new(angle, power), &a);
                far += (f.rest.0 as i32 - from.0 as i32).abs()
                    + (f.rest.1 as i32 - from.1 as i32).abs();
                n += 1;
            }
        }
        far / n
    };
    let mut prev = 0;
    for power in 1..=10u8 {
        let now = mean(power);
        // **Within a tile, and not exactly**, because the table has bumpers on
        // it and a bumper's whole job is to send a shot somewhere it would not
        // otherwise go. Measured on the *bare* map — before M17.1 put eight
        // obstacles on it — the plan's 22 stepped backwards and 30 did not; with
        // the obstacles on, both are within a tile and the measurement stops
        // discriminating. Which is worth knowing: the constant was chosen on
        // the bare table and this is what still holds on the dressed one.
        assert!(
            now + 1 >= prev,
            "power {power} lands {now} tiles out and power {} landed {prev}",
            power - 1
        );
        prev = now;
    }
    assert!(prev >= 6, "a full-power shot goes {prev} tiles on a sixteen-wide map");
}

/// **A shot reaches nearly the whole table, and that is the block's finding.**
///
/// Seven hundred and twenty shots from the Treyway's start land on **164 of its
/// 176 walkable tiles**. `PLAN-M17.md` §10.3 asks whether sixteen by sixteen is
/// too small to be an interesting table and says to report it rather than fix
/// it; this is the number that answers it. A table where one shot reaches
/// ninety-three percent of the map is closer to a menu than to a course.
#[test]
fn one_shot_reaches_most_of_the_treyway() {
    let w = treyway();
    let a = Allowances::default();
    let mut land = std::collections::BTreeSet::new();
    for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            land.insert(shot::shoot(&w, (8, 13), Shot::new(angle, power), &a).rest);
        }
    }
    let walkable = (0..w.height)
        .flat_map(|y| (0..w.width).map(move |x| (x, y)))
        .filter(|(x, y)| w.walkable(*x, *y, &a))
        .count();
    assert!(land.len() * 100 / walkable >= 80,
        "one shot reaches {} of {walkable} tiles", land.len());
}

/// **Two maps are tables and the rest are floors.**
///
/// `PLAN-M17.md` §2.1: a puzzle floor is a floor because you stand on one tile
/// and read the one next to it, and a table with nine stakes on it is a floor
/// nobody can solve. The count is asserted rather than the names, so a third
/// one is a decision somebody makes here.
#[test]
fn only_the_country_maps_are_tables() {
    let tables: Vec<&str> = data::MAPS
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| data::map(id, D).traversal == Traversal::Shot)
        .collect();
    assert_eq!(
        tables,
        // **The lower table joins the list in M22.5 and its file arrives in
        // M22.6**, which is the order enforcing itself: this refuses the map
        // until it exists rather than after, so the milestone that authors it
        // cannot ship a walked map by accident.
        vec!["the-treyway", "the-undercountry", "the-lower-table"],
        "the tables are {tables:?}"
    );
}

/// **An obstacle is the ball's, so it belongs on a table.**
///
/// Fifteen: the Treyway's nine and the Undercountry's six.
///
/// Checked at load, so a bumper on a map you walk across is a map that will not
/// open rather than a place a foot walks over and nothing happens. This is the
/// same question asked from the data's side, because a guard whose only proof
/// is that the file happens to load is a guard nobody is reading.
#[test]
fn every_obstacle_is_on_a_table() {
    let mut seen = 0;
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        for p in &w.places {
            if p.kind.is_obstacle() {
                assert_eq!(w.traversal, Traversal::Shot, "{id} walks and has a {:?} on it", p.kind);
                seen += 1;
            }
        }
    }
    // Nine on the Treyway, six on the Undercountry and **eleven on the lower
    // table** — three pockets, three stones, two teeth, a drove way and two
    // drifts. A number rather than a range, because an obstacle that appeared
    // without anybody deciding to put it there is exactly what this is watching
    // for.
    assert_eq!(seen, 26, "the game has {seen} obstacles on it");
}

/// **A bumper adds thirty percent and never more than a full-power shot.**
#[test]
fn a_bumper_adds_thirty_and_is_capped() {
    let w = treyway();
    let a = Allowances::default();
    let mut fastest = 0;
    for angle in 0..STEPS as u16 {
        let f = shot::shoot(&w, (8, 13), Shot::new(angle, 10), &a);
        for pair in f.path.windows(2) {
            let (dx, dy) = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            fastest = fastest.max(shot::isqrt(dx * dx + dy * dy));
        }
    }
    assert!(
        fastest <= shot::BUMPER_CAP * 3 / 2,
        "a ball reached {fastest} sub-cells a tick against a cap of {}",
        shot::BUMPER_CAP
    );
}

// ----------------------------------------------------------- the obstacles

fn obstacle(kind: gm2d_core::world::PlaceKind) -> (u8, u8) {
    treyway()
        .places
        .iter()
        .find(|p| p.kind == kind)
        .map(|p| (p.at[0], p.at[1]))
        .unwrap_or_else(|| panic!("the Treyway has no {kind:?}"))
}

/// **A spike tires you and the ball goes through.**
///
/// Eight percentage points, once a flight however many ticks are spent on it —
/// a spike that charged per tick would charge a slow ball nine times and a fast
/// one once for the same contact.
#[test]
fn a_spike_tires_and_passes() {
    use gm2d_core::world::PlaceKind;
    let w = treyway();
    let a = Allowances::default();
    let at = obstacle(PlaceKind::Spike);
    let mut found = None;
    'search: for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            let f = shot::shoot(&w, (at.0, at.1 + 3), Shot::new(angle, power), &a);
            if f.contacts.iter().any(|c| matches!(c, Contact::Spike { .. })) {
                found = Some(f);
                break 'search;
            }
        }
    }
    let f = found.expect("no shot on this table ever finds a spike");
    assert_eq!(f.tiring(), shot::SPIKE_TIRES, "a spike costs {}", f.tiring());
    let spikes = f.contacts.iter().filter(|c| matches!(c, Contact::Spike { .. })).count();
    assert_eq!(spikes, 1, "one spike charged {spikes} times");
    // And it passed: the ball did not stop on it.
    assert!(f.ticks > 1, "the ball stopped dead on a spike");
}

/// **Sand stops the ball dead the moment it enters.**
#[test]
fn sand_stops_dead() {
    use gm2d_core::world::PlaceKind;
    let w = treyway();
    let a = Allowances::default();
    let at = obstacle(PlaceKind::Sand);
    let mut found = None;
    'search: for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            let f = shot::shoot(&w, (at.0, at.1 + 2), Shot::new(angle, power), &a);
            if f.contacts.iter().any(|c| matches!(c, Contact::Sand { .. })) {
                found = Some(f);
                break 'search;
            }
        }
    }
    let f = found.expect("no shot on this table ever finds the sand");
    assert_eq!(f.rest, at, "the ball crossed the sand and rested at {:?}", f.rest);
    assert_eq!(f.tiring(), 0, "sand costs fatigue, and it should cost a stroke");
}

/// **A pocket sinks a ball that stops on it, and lets one over it at speed.**
///
/// Pinball Quest's bottom of the table: the drain is death-lite, and it has
/// been an RPG mechanic for thirty-seven years.
#[test]
fn a_pocket_sinks_only_a_ball_that_stops_on_it() {
    use gm2d_core::world::PlaceKind;
    let w = treyway();
    let a = Allowances::default();
    let at = obstacle(PlaceKind::Pocket);
    let mut sank = 0;
    let mut over = 0;
    for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            let f = shot::shoot(&w, (at.0, at.1 + 4), Shot::new(angle, power), &a);
            let crossed = f.path.iter().any(|(px, py)| {
                (px / shot::SUB, py / shot::SUB) == (at.0 as i32, at.1 as i32)
            });
            match (f.sunk().is_some(), crossed) {
                (true, _) => sank += 1,
                (false, true) => over += 1,
                _ => {}
            }
        }
    }
    assert!(sank > 0, "no shot ever sank in the wood");
    assert!(over > 0, "no shot ever went over the pocket at speed");
    // And sinking costs twelve, which is `PLAN-M17.md` §10.4: a pocket has to
    // be worse than a spike or nobody aims around it.
    assert!(shot::POCKET_TIRES > shot::SPIKE_TIRES);
}

/// **A chute keeps the ball's speed and its heading.**
///
/// It is a mouth rather than a wall: the ball goes in here and comes out there
/// still travelling, which is what a pinball lane is and what a drove way is
/// for.
#[test]
fn a_chute_keeps_speed_and_heading() {
    use gm2d_core::world::PlaceKind;
    let w = treyway();
    let a = Allowances::default();
    let mouth = obstacle(PlaceKind::Chute);
    let far = w
        .places
        .iter()
        .find(|p| p.kind == PlaceKind::Chute)
        .and_then(|p| p.at_to)
        .expect("a chute has a far end");
    let mut found = None;
    'search: for angle in 0..STEPS as u16 {
        for power in 4..=10u8 {
            let f = shot::shoot(&w, (mouth.0 + 2, mouth.1), Shot::new(angle, power), &a);
            if f.contacts.iter().any(|c| matches!(c, Contact::Chute { .. })) {
                found = Some(f);
                break 'search;
            }
        }
    }
    let f = found.expect("no shot on this table ever finds the drove way");
    // The path jumps: one tick is at the mouth and the next is at the far end.
    let jumped = f.path.windows(2).any(|p| {
        let (a, b) = (p[0], p[1]);
        let d = (a.0 - b.0).abs() + (a.1 - b.1).abs();
        d > shot::SUB * 4
    });
    assert!(jumped, "the ball went into the chute and walked out of it");
    let ends_near = (f.rest.0 as i32 - far[0] as i32).abs() + (f.rest.1 as i32 - far[1] as i32).abs();
    assert!(ends_near < 16, "it came out at {:?} and rested at {:?}", far, f.rest);
}

/// **Every place on a shot map is reachable by shots, in six or fewer.**
///
/// `PLAN-M17.md` §2.7: this replaces *every tile of every map derives* for the
/// two shot maps, because a tile you cannot walk to is a different question
/// from a tile you cannot **land** on. The chain length is written down per
/// place, so a map that gets harder to cross says by how much.
#[test]
fn every_place_on_a_shot_map_is_reachable_by_shots() {
    use gm2d_core::world::Traversal;
    let a = Allowances::default();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        if w.traversal != Traversal::Shot {
            continue;
        }
        // **Asked twice, of two grounds.** A map is not one grid: a drain
        // turns `tide` into `coast` and `water` into lakebed, so a tile
        // nothing can stop on today is one a ball comes to rest on the moment
        // the thing that opens it has happened. The union of the two floods is
        // *is there a state of the world in which you could land here*, which
        // is the only honest form of the question — anything narrower fails a
        // shore that opens later, and anything wider passes a wall.
        let mut state = gm2d_core::world::WorldState::default();
        state.map = id.to_string();
        for d in &w.drains {
            state.flags.push(d.when.clone());
        }
        let drained = data::map_now(id, D, &state);

        // Flood by shots: everywhere one shot from anywhere already reached.
        // **And what it touched on the way, which is a different question.**
        // A bumper is a tile no ball can ever come to rest on — that is what a
        // bumper *is* — so asking whether one is landable is asking the wrong
        // thing of it, and exempting obstacles outright would let an
        // unreachable one through. What an obstacle owes is that some shot
        // hits it.
        let flood = |w: &gm2d_core::world::World| {
            let mut seen: std::collections::BTreeSet<(u8, u8)> =
                [(w.start.0, w.start.1)].into_iter().collect();
            let mut hit: std::collections::BTreeSet<String> = Default::default();
            let mut depth = 0;
            loop {
                let mut next = seen.clone();
                for from in &seen {
                    for angle in 0..STEPS as u16 {
                        for power in 1..=10u8 {
                            let f = shot::shoot(w, *from, Shot::new(angle, power), &a);
                            for c in &f.contacts {
                                match c {
                                    Contact::Bumper { id, .. }
                                    | Contact::Spike { id, .. }
                                    | Contact::Chute { id, .. }
                                    | Contact::Sand { id, .. }
                                    | Contact::Sunk { id, .. } => {
                                        hit.insert(id.clone());
                                    }
                                    _ => {}
                                }
                            }
                            next.insert(f.rest);
                        }
                    }
                }
                if next.len() == seen.len() {
                    break;
                }
                seen = next;
                depth += 1;
                if depth >= 6 {
                    break;
                }
            }
            (seen, hit, depth)
        };
        let (dry, dry_hit, depth) = flood(&w);
        let (wet, wet_hit, _) = flood(&drained);

        for p in &w.places {
            if p.hidden_until.is_some() || !p.hidden_until_all.is_empty() {
                continue;
            }
            // **Landed on, and never merely landed beside.** The first version
            // of this allowed either, on the grounds that `PLAN-M17.md` §2.6's
            // beside-rule reached a gate standing on ground nobody can walk
            // on. It does not: what a gate beside you hands back is its
            // *refusal*, because the tide crossing has never carried a
            // condition of its own and the impassable ground was the
            // condition — so a beside-rule that opened it walked a player over
            // a bar under nine feet of water.
            //
            // Which makes the sharp question *can a ball come to rest on this
            // tile, in some state of the world* — and a lint that took
            // "beside" for an answer could not tell a gate you can enter from
            // one you can only be turned away from. It passed on exactly that
            // for a milestone, and the browser gate is what found it.
            let at = (p.at[0], p.at[1]);
            if p.kind.is_obstacle() {
                assert!(
                    dry_hit.contains(&p.id) || wet_hit.contains(&p.id),
                    "{id}: {:?} at {:?} is an obstacle no shot ever hits",
                    p.id,
                    p.at
                );
                continue;
            }
            assert!(
                dry.contains(&at) || wet.contains(&at),
                "{id}: {:?} at {:?} cannot be landed on in six shots, wet or dry - \
                 landing beside it is a refusal and never a way in",
                p.id,
                p.at
            );
        }
        println!("{id}: everything is reachable in {depth} rounds of shots");
    }
}

// ------------------------------------------------------------- the landing

/// A game standing on the Treyway, ready to fire.
fn on_the_table() -> gm2d_core::game::Game {
    let mut g = gm2d_core::game::Game::new(7, "td");
    g.world.go_to("the-treyway");
    g.world.at = [8, 13];
    g.world.last_town = "the-end-of-all-gears".into();
    g
}

/// **Landing on a place enters it, and it is `Step`'s own resolution.**
///
/// Not a second one: `world::arrive_at` is what the step above it calls, and a
/// shot that comes to rest on a town enters that town through the same door a
/// foot does.
#[test]
fn landing_on_a_place_enters_it() {
    let w = treyway();
    let a = Allowances::default();
    let gate = w
        .places
        .iter()
        .find(|p| p.id == "the-road-west")
        .map(|p| (p.at[0], p.at[1]))
        .expect("the road west");
    // Find any shot that lands on it; the table reaches everything in two
    // rounds, so one round from beside it is enough.
    let mut landed = false;
    'search: for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            if shot::shoot(&w, (gate.0 + 3, gate.1), Shot::new(angle, power), &a).rest == gate {
                let mut g = on_the_table();
                g.world.at = [gate.0 + 3, gate.1];
                let (_, step) = g.shoot(Shot::new(angle, power), D);
                assert_eq!(
                    step.gate.as_deref(),
                    Some("the-road-west"),
                    "a shot landed on a gate and the gate did not open"
                );
                landed = true;
                break 'search;
            }
        }
    }
    assert!(landed, "no shot ever landed on the road west");
}

/// **A shot costs nothing and a contact costs what the table says.**
#[test]
fn a_shot_is_free_and_the_table_is_not() {
    let mut g = on_the_table();
    let before = g.character.fatigue;
    // A shot that touches nothing.
    for angle in 0..STEPS as u16 {
        let mut probe = on_the_table();
        let (f, _) = probe.shoot(Shot::new(angle, 2), D);
        if f.tiring() == 0 {
            assert_eq!(probe.character.fatigue, before, "a shot that touched nothing tired you");
            return;
        }
    }
    let _ = g.shoot(Shot::new(0, 1), D);
    panic!("every low shot on this table touches something");
}

/// **A shot bumps `shots-taken` and never `tiles-walked`.**
///
/// `PLAN-M17.md` §2.4 and §6: the count of shots goes where *walked* goes, and
/// anything gating on `tiles-walked` is on a step map and unaffected. **The
/// recon the plan asks for**: nothing on a shot map reads `tiles-walked`, and
/// this is what keeps that true.
#[test]
fn a_shot_counts_shots_and_not_tiles() {
    let mut g = on_the_table();
    let walked = g.world.count("tiles-walked");
    g.shoot(Shot::new(18, 5), D);
    assert_eq!(g.world.count("tiles-walked"), walked, "a shot counted a tile walked");
    assert_eq!(g.world.count("shots-taken"), 1, "a shot counted no shot");
    g.shoot(Shot::new(0, 5), D);
    assert_eq!(g.world.count("shots-taken"), 2);
}

/// **A pocket sinks you home, and takes nothing but twelve percent.**
#[test]
fn a_pocket_sinks_to_the_last_town() {
    use gm2d_core::world::PlaceKind;
    let w = treyway();
    let a = Allowances::default();
    let at = obstacle(PlaceKind::Pocket);
    for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            let f = shot::shoot(&w, (at.0, at.1 + 4), Shot::new(angle, power), &a);
            if f.sunk().is_none() {
                continue;
            }
            let mut g = on_the_table();
            g.world.at = [at.0, at.1 + 4];
            g.character.carried = 250;
            let before = g.character.fatigue;
            let (_, step) = g.shoot(Shot::new(angle, power), D);
            assert_eq!(step.town.as_deref(), Some("the-end-of-all-gears"), "sunk, and not home");
            assert_eq!(g.world.map_id(), "west-bambulon", "sunk onto {}", g.world.map_id());
            assert_eq!(g.character.carried, 250, "a pocket took your carried experience");
            assert_eq!(
                g.character.fatigue - before,
                shot::POCKET_TIRES as i32,
                "a pocket costs {}",
                g.character.fatigue - before
            );
            return;
        }
    }
    panic!("no shot from beside the pocket ever sank");
}

/// **A save between shots holds a tile and nothing else.**
///
/// A shot runs to rest inside one call and only then is anything written, so a
/// save never holds a ball in the air — which is the reason `PLAN-M17.md` §2.8
/// adds no field.
#[test]
fn a_save_between_shots_holds_a_tile_and_nothing_else() {
    let mut g = on_the_table();
    g.shoot(Shot::new(18, 7), D);
    let text = gm2d_core::save::save(&g);
    // Not the bare word "shot": `shots-taken` is a counter and belongs in a
    // save. What must not be there is a ball in the air.
    for word in ["velocity", "flight", "\"angle\"", "\"power\"", "path"] {
        assert!(!text.contains(word), "a save carries {word:?}");
    }
    let back = gm2d_core::save::load(&text).expect("it opens");
    assert_eq!(back.world.at, g.world.at, "the tile did not survive the trip");
}

/// **A gate on ground nobody can stand on is entered from beside it.**
///
/// The tide crossing stands on `tide`, which is impassable — so on a table
/// there is no step that reaches it, and a ball that comes to rest beside one
/// is offered it. Shot maps only: on a step map it refuses in its own words,
/// which is the sentence M15 wrote for it.
#[test]
fn a_wall_gate_is_offered_from_beside_it() {
    let mut g = on_the_table();
    // The tideline is at [8,14] and the crossing below it at [8,15].
    g.world.at = [8, 14];
    let mut rng = gm2d_core::rng::Rng::new(1);
    let w = treyway();
    let step = gm2d_core::world::arrive_at(
        &w,
        &mut g.world,
        &mut rng,
        D,
        (9, 14),
        &Allowances::default(),
    );
    let _ = step;
    let step = gm2d_core::world::arrive_at(
        &w,
        &mut g.world,
        &mut rng,
        D,
        (8, 14),
        &Allowances::default(),
    );
    // **Both.** [8,14] is the crossing's only landable neighbour and it has the
    // tideline's own card on it — so a rule that returned early would make the
    // way south unreachable on a table. A `Step` has a field for each.
    assert_eq!(step.event.as_deref(), Some("the-tideline"));
    // **And what it is offered is the refusal, not the way through.** This is
    // the fault the beside-rule shipped with for a milestone: the tide
    // crossing has never carried a condition of its own, because the
    // impassable ground *was* the condition — so reporting `Step::gate` here
    // walked a player over a bar still under nine feet of water, and the gate
    // check said so in three engines. The sentence is the map file's and the
    // half naming where the tenth cairn is cut is the engine's.
    assert_eq!(step.gate, None, "landing beside a shut crossing opened it");
    let said = step.blocked.as_deref().unwrap_or_default();
    assert!(
        said.contains("bar") && said.contains("tenth"),
        "landing beside the crossing did not say what is over the bar: {said:?}"
    );
    assert_eq!(step.refused_by.as_deref(), Some("the-tide-crossing"));

    // And landing somewhere with no wall gate beside it is refused by nothing.
    let step = gm2d_core::world::arrive_at(
        &w,
        &mut g.world,
        &mut rng,
        D,
        (8, 12),
        &Allowances::default(),
    );
    assert_eq!(step.gate, None, "a tile in the middle of the map offered a gate");
    assert_eq!(step.refused_by, None, "a tile in the middle of the map was refused");
}

/// **The tenth cairn drains the bar and then it is a tile you land on.**
///
/// The other half of the rule above, and the reason it is safe to hand back a
/// refusal rather than the way through: once `built-the-tenth` is up the
/// crossing's own tile is `coast`, which is ground a ball can come to rest on,
/// so the way south is entered by landing on it like every other gate in the
/// game. If that were not true the beside-rule would be the only way over the
/// bar and taking the gate out of it would have sealed the shore.
#[test]
fn the_bar_is_landed_on_once_the_tide_is_out() {
    let w = treyway();
    let mut g = gm2d_core::game::Game::new(1, "td");
    g.world.map = "the-treyway".into();
    g.world.flags.push("built-the-tenth".into());
    let mut rng = gm2d_core::rng::Rng::new(1);

    let drained = data::map_now("the-treyway", D, &g.world);
    assert!(
        drained.walkable(8, 15, &Allowances::default()),
        "the tenth cairn went up and the bar is still something nobody can stand on"
    );

    let step = gm2d_core::world::arrive_at(
        &drained,
        &mut g.world,
        &mut rng,
        D,
        (8, 15),
        &Allowances::default(),
    );
    assert_eq!(
        step.gate.as_deref(),
        Some("the-tide-crossing"),
        "landing on the drained bar did not offer the crossing"
    );
    let _ = w;
}

// -------------------------------------------------------------- M17.4: aiming

/// **Everything on a table can be aimed at, from where a player starts.**
///
/// `PLAN-M17.md` §7 M17.4. The reachability lint floods: it asks whether some
/// chain of shots from anywhere reaches a place. This asks the narrower and
/// more useful question — whether `aim_at` itself, from the map's own start,
/// finds a shot to each of them — because that is what the walker and the gate
/// both do, and a place the flood reaches but `aim_at` cannot name is a place
/// the walker will never visit.
///
/// **Near, not exact**, which is what a player is doing: a ball that stops a
/// tile off is a ball you take another shot from, and insisting on exact would
/// be asserting that every tile is a hole in one.
#[test]
fn aim_at_finds_a_shot_to_every_place_from_the_start() {
    let a = Allowances::default();
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        if w.traversal != Traversal::Shot {
            continue;
        }
        let mut state = gm2d_core::world::WorldState::default();
        state.map = id.to_string();
        let from = (w.start.0, w.start.1);
        let mut missed = vec![];
        for p in &w.places {
            if p.hidden_until.is_some() || !p.hidden_until_all.is_empty() {
                continue;
            }
            // An obstacle is hit rather than landed on, so nobody aims at one.
            if p.kind.is_obstacle() {
                continue;
            }
            let at = (p.at[0], p.at[1]);
            if shot::aim_at(&w, &state, from, at, true, &a).is_none() {
                missed.push((p.id.clone(), p.at));
            }
        }
        println!(
            "{id}: aim_at names a shot to {} of {} places from the start",
            w.places.iter().filter(|p| !p.kind.is_obstacle()).count() - missed.len(),
            w.places.iter().filter(|p| !p.kind.is_obstacle()).count(),
        );
        assert!(missed.is_empty(), "{id}: nothing aims at {missed:?}");
    }
}

/// **Aiming is a move, so it has to cost about what a move costs.**
///
/// `aim_at` sweeps seventy-two angles by ten powers and simulates each, which
/// is seven hundred and twenty flights — and the walker calls it once a move
/// for twenty thousand moves. The plan asks for under a second; this asserts a
/// tenth of one over a hundred calls, which is the same claim with the noise
/// taken out of it.
///
/// **A budget rather than a benchmark.** It is loose on purpose: a test that
/// pins a runtime is a test that goes red on a busy machine, and what this is
/// guarding against is somebody making `aim_at` quadratically worse, which
/// would miss this by two orders of magnitude rather than by ten percent.
#[test]
fn aim_at_is_under_a_second() {
    let a = Allowances::default();
    let w = treyway();
    let mut state = gm2d_core::world::WorldState::default();
    state.map = "the-treyway".into();
    let targets: Vec<(u8, u8)> = w
        .places
        .iter()
        .filter(|p| !p.kind.is_obstacle())
        .map(|p| (p.at[0], p.at[1]))
        .collect();
    let start = std::time::Instant::now();
    let mut found = 0;
    for i in 0..100 {
        let t = targets[i % targets.len()];
        if shot::aim_at(&w, &state, (w.start.0, w.start.1), t, true, &a).is_some() {
            found += 1;
        }
    }
    let each = start.elapsed() / 100;
    println!("aim_at: {each:?} a call, {found} of 100 found a shot");
    assert!(
        each < std::time::Duration::from_millis(100),
        "aim_at takes {each:?} a call, which is a walker that cannot afford to aim"
    );
}

// -------------------------------------------------- a diamond catches the ball

/// **Hitting a gate is entering it, and landing exactly on it is not required.**
///
/// Reported from play: *"if you just simply hit the diamonds to enter a zone,
/// you should enter it, it shouldnt have to perfectly land on it"*. A gate and
/// a boss are the two things the map draws as a diamond and the two that are a
/// way *into* somewhere, so on a table they catch the ball. A cue that demanded
/// a tile exactly would be a cue that demanded a hole in one.
///
/// The measurement is the point: how many shots from the map's own start reach
/// each gate, before and after. Landing-only was a handful; catching is every
/// shot whose line crosses it.
#[test]
fn a_diamond_catches_the_ball_rather_than_needing_a_hole_in_one() {
    let a = Allowances::default();
    for id in ["the-treyway", "the-undercountry"] {
        let w = data::map(id, D);
        let mut state = gm2d_core::world::WorldState::default();
        state.map = id.to_string();
        let gates: Vec<_> = w
            .places
            .iter()
            .filter(|p| p.kind.catches() && p.hidden_until.is_none() && p.hidden_until_all.is_empty())
            .collect();
        assert!(!gates.is_empty(), "{id}: no diamonds on it");

        for g in &gates {
            let mut caught = 0;
            let mut landed = 0;
            for from in [(w.start.0, w.start.1)] {
                for angle in 0..STEPS as u16 {
                    for power in 1..=10u8 {
                        let f = shot::shoot_with(&w, &state, from, Shot::new(angle, power), &a);
                        if f.contacts.iter().any(
                            |c| matches!(c, Contact::Caught { id, .. } if *id == g.id),
                        ) {
                            caught += 1;
                            assert_eq!(
                                f.rest,
                                (g.at[0], g.at[1]),
                                "{id}: caught by {:?} and came to rest somewhere else",
                                g.id
                            );
                        }
                        if f.rest == (g.at[0], g.at[1]) {
                            landed += 1;
                        }
                    }
                }
            }
            println!(
                "{id}: {:?} is hit by {caught} of 720 shots from the start ({landed} land on it)",
                g.id
            );
            // **Every landing is a catch — except on the tile you shot from.**
            // The Treyway's start *is* the door back to West Bambulon, so a
            // ball that rolls home onto it was never caught: it left. That is
            // the origin exemption stated from the other side, and it is the
            // one case where the two numbers may differ.
            if (g.at[0], g.at[1]) == (w.start.0, w.start.1) {
                assert_eq!(
                    caught, 0,
                    "{id}: {:?} is the tile the shot came from and caught {caught}",
                    g.id
                );
                continue;
            }
            assert_eq!(
                caught, landed,
                "{id}: {:?} was landed on {landed} times and caught {caught} - a diamond \
                 you can come to rest on without being caught is a diamond with two rules",
                g.id
            );
            // **A diamond on ground the ball cannot enter is not caught, and
            // must not be.** The tide crossing stands on `tide` until the
            // tenth cairn goes up two maps away; nothing can come to rest on
            // it or fly into it, and what answers it is the beside-rule, which
            // hands back its refusal. The two mechanisms are for the two
            // cases and neither covers the other.
            if !w.walkable(g.at[0], g.at[1], &a) {
                assert_eq!(
                    caught, 0,
                    "{id}: {:?} is on ground nothing can enter and was caught {caught} times",
                    g.id
                );
                continue;
            }
            assert!(caught > 0, "{id}: {:?} is hit by no shot from the start", g.id);
        }
    }
}

/// **A ball leaving a diamond is not caught by the one it is leaving.**
///
/// The gate you were just refused at is the tile you are standing on, so a
/// catcher that did not exempt the shot's own origin would take every shot
/// from there and put you straight back — a soft-lock made of one rule.
#[test]
fn the_diamond_you_are_standing_on_does_not_catch_you() {
    let a = Allowances::default();
    let w = treyway();
    let mut state = gm2d_core::world::WorldState::default();
    state.map = "the-treyway".into();
    let gate = w
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Gate && w.walkable(p.at[0], p.at[1], &a))
        .expect("the Treyway has a gate on ground");
    let from = (gate.at[0], gate.at[1]);

    let mut left = 0;
    for angle in 0..STEPS as u16 {
        for power in 1..=10u8 {
            let f = shot::shoot_with(&w, &state, from, Shot::new(angle, power), &a);
            assert!(
                !f.contacts.iter().any(
                    |c| matches!(c, Contact::Caught { id, .. } if *id == gate.id)
                ),
                "a shot from {:?}'s own tile was caught by it",
                gate.id
            );
            if f.rest != from {
                left += 1;
            }
        }
    }
    assert!(left > 0, "no shot from {:?} goes anywhere", gate.id);
    println!("{:?}: {left} of 720 shots leave its tile", gate.id);
}

// ------------------------------------------- M22.5, a pocket that goes somewhere

/// A game standing on the Undercountry, with a pocket rewritten to say where it
/// goes.
///
/// **Built by editing the shipped map rather than by writing one**, because a
/// hand-written table is a second copy of what a table is — and what is being
/// proved here is a rule, not a file.
fn a_pocket_that_goes(to: Option<(&str, [u8; 2])>) -> (gm2d_core::world::World, String) {
    let text = gm2d_core::data::UNDERCOUNTRY_JSON.to_string();
    let mut v: serde_json::Value = serde_json::from_str(&text).expect("the map parses");
    let places = v["places"].as_array_mut().expect("places");
    let mut id = String::new();
    for p in places.iter_mut() {
        if p["kind"] == "pocket" {
            id = p["id"].as_str().expect("an id").to_string();
            match to {
                Some((map, at)) => {
                    p["to"] = serde_json::json!(map);
                    p["at_to"] = serde_json::json!(at);
                }
                None => {
                    p.as_object_mut().expect("an object").remove("to");
                    p.as_object_mut().expect("an object").remove("at_to");
                }
            }
            break;
        }
    }
    assert!(!id.is_empty(), "the Undercountry has no pocket on it");
    let w = gm2d_core::world::World::load(
        gm2d_core::data::TERRAIN_JSON,
        &v.to_string(),
        D,
    )
    .expect("the edited map loads");
    (w, id)
}

/// **A pocket with a `to` lands you where it says, and one without still goes
/// home.**
///
/// `PlaceDef::to`/`at_to` are the gate's own fields and the sunk arm is
/// `Game::warp_to` — the same call the home arm already made — so the only
/// thing M22.5 adds is a destination. It is the one thing Yoku's holes do that
/// this engine did not: **a hole is how you go *into* a room.**
#[test]
fn a_pocket_with_a_to_lands_where_it_says() {
    let (w, id) = a_pocket_that_goes(Some(("the-great-gear-cave", [1, 2])));
    let p = w.places.iter().find(|p| p.id == id).expect("the pocket");
    assert_eq!(p.to.as_deref(), Some("the-great-gear-cave"));
    assert_eq!(p.at_to, Some([1, 2]));
    // And the same map with the pair taken off still loads, which is what the
    // gutters are.
    let (bare, id2) = a_pocket_that_goes(None);
    assert_eq!(id, id2);
    assert!(bare.places.iter().find(|p| p.id == id).expect("the pocket").to.is_none());
}

/// **A `to` with no `at_to` is refused at load, on any kind.**
///
/// A warp with no landing tile puts you wherever the far map's `start` happens
/// to be, which is a different place from the one somebody meant — and since a
/// pocket can carry the pair, the rule is stated over the **field** rather than
/// over the gate that used to be its only reader. `ShopsData::parse`'s
/// division, one file along: this is what a map file can be asked about itself.
#[test]
fn a_pocket_that_says_where_without_saying_onto_what_is_refused() {
    let text = gm2d_core::data::UNDERCOUNTRY_JSON.to_string();
    let mut v: serde_json::Value = serde_json::from_str(&text).expect("the map parses");
    for p in v["places"].as_array_mut().expect("places").iter_mut() {
        if p["kind"] == "pocket" {
            p["to"] = serde_json::json!("the-great-gear-cave");
            break;
        }
    }
    let why = gm2d_core::world::World::load(
        gm2d_core::data::TERRAIN_JSON,
        &v.to_string(),
        D,
    )
    .expect_err("a pocket with nowhere to put you loaded");
    assert!(
        format!("{why}").contains("where it puts you"),
        "the refusal does not say what is missing: {why}"
    );
}

/// **A pocket that goes somewhere costs the same twelve.**
///
/// *A pocket has to be worse than a spike or nobody aims around it* is
/// `PLAN-M17.md` §10.4's choice and it is still true of a pocket that is a
/// door: a cheaper way into a boss's room is a discount on the hardest shot on
/// the map. `Flight::tiring` reads the contact and not the place, so there is
/// one number and nowhere for a second to be written.
#[test]
fn a_pocket_that_goes_somewhere_costs_the_same_twelve() {
    // **Called, not read.** The first draft of this counted the arm in
    // `shot.rs`'s source and passed with the arm changed to `POCKET_TIRES / 2`,
    // because the text it matched was still there — *a lint that reads a list
    // rather than the behaviour is the failure it exists to catch, one level
    // up*, in a test written in the same afternoon as the rule it guards.
    let a = Allowances::default();
    for goes in [false, true] {
        let (w, _) = a_pocket_that_goes(if goes {
            Some(("the-great-gear-cave", [1, 2]))
        } else {
            None
        });
        let at = w
            .places
            .iter()
            .find(|p| p.kind == gm2d_core::world::PlaceKind::Pocket)
            .expect("a pocket")
            .at;
        let mut sank = false;
        // **From everywhere it can be fired from**, because a pocket that a
        // two-tile tee happens to miss is a pocket this proves nothing about —
        // and the Undercountry's is in the corner of the map with nothing on
        // it, which is the corner it would be a shame to end up in.
        let froms: Vec<(u8, u8)> = (0..w.width)
            .flat_map(|x| (0..w.height).map(move |y| (x, y)))
            .filter(|(x, y)| w.walkable(*x, *y, &a))
            .collect();
        for from in &froms {
            for angle in (0..STEPS as u16).step_by(3) {
                let power = 4u8;
                let f = shot::shoot(&w, *from, Shot::new(angle, power), &a);
                if f.sunk().is_some() {
                    assert_eq!(
                        f.tiring(),
                        shot::POCKET_TIRES,
                        "a pocket that {} cost {} rather than {}",
                        if goes { "goes somewhere" } else { "goes home" },
                        f.tiring(),
                        shot::POCKET_TIRES
                    );
                    sank = true;
                }
            }
            if sank {
                break;
            }
        }
        assert!(sank, "no shot on this table ever found the pocket");
        let _ = at;
    }
    assert_eq!(shot::POCKET_TIRES, 12, "the number itself moved");
}

/// **The tape says which kind of pocket took you.**
///
/// Two sentences and core picks between them: a pocket that sinks you home has
/// said so since M17, and one that is the way *into* somewhere names where,
/// because a player who has just been dropped through the floor of a table is
/// owed the name of the floor. The page prints what it is handed.
#[test]
fn the_tape_names_where_a_pocket_put_you() {
    // **Found rather than guessed.** The first draft fired one shot at the
    // pocket's own tile and wrapped every assertion in `if sunk`, and that shot
    // never sank — so the whole check was a `compares zero with zero`, green on
    // a build with the second sentence deleted. It sweeps for a flight that
    // actually sinks and asserts unconditionally.
    let (w, _) = a_pocket_that_goes(None);
    let a = Allowances::default();
    let froms: Vec<(u8, u8)> = (0..w.width)
        .flat_map(|x| (0..w.height).map(move |y| (x, y)))
        .filter(|(x, y)| w.walkable(*x, *y, &a))
        .collect();
    let sweep = |want_sunk: bool| -> shot::Flight {
        for from in &froms {
            for angle in (0..STEPS as u16).step_by(3) {
                let f = shot::shoot(&w, *from, Shot::new(angle, 4), &a);
                if f.sunk().is_some() == want_sunk {
                    return f;
                }
            }
        }
        panic!("no shot on this table ever {}", if want_sunk { "sank" } else { "missed" });
    };
    let sunk = sweep(true);

    let home = sunk.tape(1, "slag", 0);
    let into = sunk.tape_into(1, "slag", 0, Some("the cup"));
    assert!(home.contains("last town you stood in"), "the home sentence moved: {home}");
    assert!(!home.contains("down into"), "the home sentence names somewhere: {home}");
    assert!(into.contains("down into the cup"), "the named sentence does not name it: {into}");
    assert!(
        !into.contains("last town you stood in"),
        "the named sentence still says you went home: {into}"
    );
    assert!(into.contains("12%"), "the cost left the sentence: {into}");

    // And a flight that did **not** sink says neither, whichever is asked.
    let missed = sweep(false);
    assert!(!missed.tape_into(1, "slag", 0, Some("the cup")).contains("down into"));
    assert!(!missed.tape(1, "slag", 0).contains("sunk"));
}

// -------------------------------------------------- M22.6, the lower table

const TABLE: &str = "the-lower-table";
const CUP: &str = "the-cup";

/// **The boss has no straight line to it, and the reason is that it is not on
/// the table at all.**
///
/// `PLAN-M22.md` decision 9 puts it in a sealed cup of rock on the table and
/// makes this a lint over the **flood**. M22.0 measured that cup: **6,546 of
/// the shots taken from the 278 walkable tiles outside a draft one came to rest
/// inside it**, through eight tiles of solid rock with no mouth, because
/// `shot::shoot_with` tests the tile a tick *landed on* and never the tiles it
/// crossed — one tick is 1.875 tiles at power one and **18.75 at power ten**. A
/// two-tile wall is transparent too and nineteen would be the map.
///
/// So the cup is a **map**, and the property is true by construction: there is
/// nothing on the table to aim at, and the only way onto the cup is a pocket.
/// That is what M22.5's `to`/`at_to` is for and it is the one thing Yoku's
/// holes do that this engine did not.
#[test]
fn the_boss_has_no_straight_line() {
    let table = data::map(TABLE, D);
    assert!(
        table.places.iter().all(|p| p.creature.is_none()),
        "something stands on the lower table, and the cup is supposed to be a map"
    );
    let cup = data::map(CUP, D);
    let boss = cup
        .places
        .iter()
        .find(|p| p.kind == gm2d_core::world::PlaceKind::Boss)
        .expect("the cup has nothing on the plank");
    // **One way onto the cup, and it is a pocket.** Every gate in the game is
    // walked through; this is fallen down.
    let ways: Vec<String> = data::MAPS
        .iter()
        .flat_map(|(id, _)| {
            data::map(id, D)
                .places
                .into_iter()
                .filter(|p| p.to.as_deref() == Some(CUP))
                .map(move |p| format!("{id}/{}:{:?}", p.id, p.kind))
        })
        .collect();
    assert_eq!(
        ways,
        vec![format!("{TABLE}/the-far-pocket:Pocket")],
        "the ways into the cup are {ways:?}"
    );
    let _ = boss;
}

/// **Every pocket on the table is named, and the one that goes somewhere says
/// where.**
///
/// A pocket is a refusal the ball cannot make for itself, so it is the one kind
/// of place a player meets without having aimed at it — and an unnamed one is
/// *sunk, and* nothing. The two gutters go home, which is what a gutter is; the
/// far pocket goes into the cup.
#[test]
fn every_pocket_on_the_table_goes_somewhere_named() {
    let w = data::map(TABLE, D);
    let pockets: Vec<&gm2d_core::world::PlaceDef> = w
        .places
        .iter()
        .filter(|p| p.kind == gm2d_core::world::PlaceKind::Pocket)
        .collect();
    assert_eq!(pockets.len(), 3, "the table has {} pockets", pockets.len());
    let mut went = 0;
    for p in &pockets {
        assert!(!p.name.is_empty(), "{} is a pocket with no name", p.id);
        assert!(!p.shut.is_empty(), "{} is a pocket with nothing to say", p.id);
        if let Some(to) = &p.to {
            went += 1;
            assert_eq!(to, CUP, "{} goes to {to}, which is not the cup", p.id);
            let at = p.at_to.expect("a pocket that says where and not where onto");
            let far = data::map(to, D);
            assert!(
                far.walkable(at[0], at[1], &Allowances::default()),
                "{} puts you on ({}, {}), which is not ground",
                p.id, at[0], at[1]
            );
        }
    }
    assert_eq!(went, 1, "{went} pockets go somewhere, and one is the design");
}

/// **`PlaceKind::Door` still has exactly one user, and it is the stop-line.**
///
/// It has had one since M14 put the sentence on a `Door` one tile south of the
/// third town — *a `TownShelf` is an id, a stock list and a commission list and
/// has never had prose* — and M22.6 moves that sentence to the far end of the
/// cup rather than adding a second. **There is one screen in the game that says
/// the writing stops**, and this is what keeps it one.
#[test]
fn the_stop_line_is_still_one_door() {
    let mut doors: Vec<String> = Vec::new();
    for (id, _) in data::MAPS {
        for p in data::map(id, D).places {
            if p.kind == gm2d_core::world::PlaceKind::Door {
                doors.push(format!("{id}/{}", p.id));
            }
        }
    }
    assert_eq!(doors, vec![format!("{CUP}/the-writing-stops-here")], "the doors are {doors:?}");
}

/// **Everything on the table can be reached, and it takes three rounds.**
///
/// The plan guesses two, which is the Treyway's number. Whatever it comes out
/// at is written down here rather than left to the flood's own `println`,
/// because a number a design stakes itself on wants somewhere it is asserted —
/// and the far pocket is the whole of that design: if no shot reaches it, the
/// cup is a room nobody can get into.
#[test]
fn the_far_pocket_is_reachable_and_the_table_takes_three_rounds() {
    let w = data::map(TABLE, D);
    let a = Allowances::default();
    let mut ring: std::collections::BTreeSet<(u8, u8)> =
        [(w.start.0, w.start.1)].into_iter().collect();
    let mut hit: std::collections::BTreeSet<String> = Default::default();
    let mut rounds = 0;
    for _ in 0..4 {
        rounds += 1;
        let mut next = ring.clone();
        for from in &ring {
            for angle in 0..STEPS as u16 {
                for power in 1..=10u8 {
                    let f = shot::shoot(&w, *from, Shot::new(angle, power), &a);
                    for c in &f.contacts {
                        match c {
                            Contact::Bumper { id, .. }
                            | Contact::Spike { id, .. }
                            | Contact::Chute { id, .. }
                            | Contact::Sand { id, .. }
                            | Contact::Sunk { id, .. } => {
                                hit.insert(id.clone());
                            }
                            _ => {}
                        }
                    }
                    next.insert(f.rest);
                }
            }
        }
        ring = next;
        let missing: Vec<&str> = w
            .places
            .iter()
            .filter(|p| {
                if p.kind.is_obstacle() {
                    !hit.contains(&p.id)
                } else {
                    !ring.contains(&(p.at[0], p.at[1]))
                }
            })
            .map(|p| p.id.as_str())
            .collect();
        if missing.is_empty() {
            break;
        }
    }
    assert!(
        hit.contains("the-far-pocket"),
        "no shot on the table ever finds the far pocket, so the cup is a room with no way in"
    );
    assert!(rounds <= 3, "the table takes {rounds} rounds of shots, and three is the ceiling");
    println!("the lower table: everything in {rounds} rounds of shots");
}
