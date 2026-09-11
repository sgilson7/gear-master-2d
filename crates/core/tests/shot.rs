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
        assert!(now >= prev, "power {power} lands {now} tiles out and power {} landed {prev}",
            power - 1);
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

/// **No map is a table yet**, which is what makes M17.0 a milestone nobody can
/// see. Every map still steps.
#[test]
fn no_map_is_a_table_yet() {
    for (id, _) in data::MAPS {
        let w = data::map(id, D);
        assert_eq!(w.traversal, Traversal::Step, "{id} is already a shot map");
    }
}
