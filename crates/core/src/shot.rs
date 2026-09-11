//! The country as a table: one shot, run to rest, in integers.
//!
//! **`PLAN-M17.md` §2.2, and the integers are the whole of why this is in
//! core.** A seeded walk has to produce the same flight in every browser, and
//! float rounding is the one thing that breaks that silently — the symptom
//! would be a shot that lands for the person who fired it and not for the
//! person they sent the save to. The same argument `world.rs` makes about
//! per-mille rolls, one screen along.
//!
//! So: positions and velocities are **sixteenths of a tile in `i32`**, angles
//! are one of seventy-two five-degree steps read out of a table of sixteenths,
//! and there is no `sqrt`, no `f32` and no trigonometry at runtime.
//! `a_shot_is_the_same_in_every_engine` hashes a fixed flight and the browser
//! gate checks the wasm build agrees.
//!
//! **The shim animates what this returns and decides nothing**, which is the
//! rule since M8: a physics loop in JavaScript would be the first thing in the
//! repository that ran differently in three engines.

use crate::world::{Allowances, World};

/// Sub-cells a tile is divided into.
///
/// Sixteen, because a five-degree step in angle has to change where a ten-tile
/// shot lands: at ten tiles a five-degree fan is about seven-eighths of a tile,
/// and sixteenths resolve that to fourteen distinguishable places.
pub const SUB: i32 = 16;

/// Sub-cells a tick moves, per point of power.
///
/// **Thirty, and `PLAN-M17.md` §3 says twenty-two.** The plan chose it on paper
/// and says the recon has to check it; swept over the Treyway from four tiles
/// and every angle, twenty-two is the one setting in the range where distance
/// stops being **monotone in power** — a player pulling back harder and landing
/// *nearer* is an unplayable cue, and it happens because a fast ball spends its
/// extra speed on ricochets rather than on ground.
///
/// At thirty the mean distance runs 4, 6, 7 and then flat, with no step
/// backwards anywhere.
pub const POWER_UNIT: i32 = 30;

/// What a bounce keeps, in percent.
///
/// **The plan's, kept, and the sweep is why it is worth saying.** Sixty,
/// forty-five and thirty-five were all measured and every one of them breaks
/// the monotonicity above: a bounce that costs too much makes a strong shot die
/// *at* the wall it hit, which is a table where the far side is unreachable.
/// Eighty and thirty is the only pair in the sweep with no step backwards.
pub const RESTITUTION: i32 = 80;

/// A ball slower than this is at rest.
pub const STOP_BELOW: i32 = 6;

/// A bound, never reached by a legal shot. `no_legal_shot_reaches_max_ticks`
/// is what says so.
pub const MAX_TICKS: u32 = 400;

/// What a bumper adds, in percent.
pub const BUMPER_BOOST: i32 = 30;

/// The angle table: five-degree steps, cosine and sine in sixteenths.
///
/// **Seventy-two entries and no trigonometry at runtime**, which is the whole
/// determinism argument: a `cos` is a float and a float is the one thing that
/// rounds differently in three engines. Generated once and written down, so it
/// is a table somebody can read rather than a call somebody has to trust —
/// `the_angle_table_is_a_circle` is what keeps it honest.
pub const STEPS: usize = 72;

/// `(cos, sin)` in sixteenths, for angle `i * 5` degrees.
pub static DIRS: [(i32, i32); STEPS] = build_dirs();

const fn build_dirs() -> [(i32, i32); STEPS] {
    // A quarter turn of sixteenths, worked out once and mirrored. Index `i` is
    // `cos(5i°) * 16` rounded; the sine is the cosine a quarter turn along.
    const COS: [i32; 19] = [
        16, 16, 16, 15, 15, 14, 14, 13, 12, 11, 10, 9, 8, 7, 6, 4, 3, 1, 0,
    ];
    let mut out = [(0, 0); STEPS];
    let mut i = 0;
    while i < STEPS {
        // cos(5i) by quadrant, and sin(5i) = cos(5(18 - i)).
        let c = quadrant(&COS, i);
        let s = quadrant(&COS, (i + STEPS - 18) % STEPS);
        out[i] = (c, s);
        i += 1;
    }
    out
}

const fn quadrant(cos: &[i32; 19], i: usize) -> i32 {
    if i <= 18 {
        cos[i]
    } else if i <= 36 {
        -cos[36 - i]
    } else if i <= 54 {
        -cos[i - 36]
    } else {
        cos[72 - i]
    }
}

/// One shot: an angle in five-degree steps and a power from one to ten.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shot {
    /// `0..STEPS`, each step five degrees anticlockwise from east.
    pub angle: u16,
    /// `1..=10`.
    pub power: u8,
}

impl Shot {
    pub fn new(angle: u16, power: u8) -> Shot {
        Shot { angle: angle % STEPS as u16, power: power.clamp(1, 10) }
    }

    /// The velocity this shot starts with, in sub-cells a tick.
    pub fn velocity(self) -> (i32, i32) {
        let (cx, cy) = DIRS[self.angle as usize % STEPS];
        let speed = self.power as i32 * POWER_UNIT;
        // **Screen coordinates, so up is negative y.** The table is a maths
        // circle and the map is a grid, and the flip belongs here rather than
        // in the shim — the shim draws what this returns.
        (cx * speed / SUB, -cy * speed / SUB)
    }
}

/// What the ball touched, in the order it touched it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Contact {
    /// Bounced off terrain or the map's edge.
    Wall { at: (u8, u8) },
    /// Crossed a tile of this terrain, which cost it speed.
    Crossed { at: (u8, u8), friction: i32 },
}

/// One shot, from a tile, run to rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flight {
    /// Every position, in sub-cells, one a tick. The shim draws this.
    pub path: Vec<(i32, i32)>,
    /// Everything it touched, in order.
    pub contacts: Vec<Contact>,
    /// The tile it came to rest on.
    pub rest: (u8, u8),
    /// How many ticks it took.
    pub ticks: u32,
}

impl Flight {
    /// How many walls it hit.
    pub fn bounces(&self) -> usize {
        self.contacts.iter().filter(|c| matches!(c, Contact::Wall { .. })).count()
    }

    /// A number that is the same in every engine, for
    /// `a_shot_is_the_same_in_every_engine` and for the browser gate to compare
    /// against.
    ///
    /// FNV-1a over the path and the rest, deliberately not cryptographic: what
    /// it detects is *the wasm build computed a different flight*, which is a
    /// difference of one sub-cell and would show.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |n: i64| {
            for b in n.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        for (x, y) in &self.path {
            eat(*x as i64);
            eat(*y as i64);
        }
        eat(self.rest.0 as i64);
        eat(self.rest.1 as i64);
        eat(self.ticks as i64);
        h
    }
}

/// What crossing one tile of this terrain costs, in sub-cells a tick.
///
/// **The table is the terrain table.** A road is fast and slag is slow, which is
/// what those words already mean — so this is read off the name rather than off
/// a new field in `terrain.json`, because friction is a fact about a *shot* and
/// the terrain file is shared by twenty-five maps that do not have one.
pub fn friction(terrain: &str) -> i32 {
    match terrain {
        "road" => 3,
        "coast" => 4,
        "plain" | "grass" | "town" => 5,
        "scrub" => 8,
        "cork" => 10,
        "wood" => 12,
        "lakebed" => 14,
        "slag" => 16,
        "silt" => 18,
        // Anything a ball cannot cross never reaches this, and anything new
        // that it can gets the middle of the table rather than a free ride.
        _ => 8,
    }
}

/// Fire, and run to rest.
///
/// **The whole of the physics, and it is forty lines every 2D tile game has.**
/// One tick moves by the velocity; crossing a tile boundary costs that tile's
/// friction; a wall reflects the normal component at [`RESTITUTION`] and keeps
/// the tangent. The loop ends at rest or at [`MAX_TICKS`], and a legal shot
/// never reaches the second.
pub fn shoot(world: &World, from: (u8, u8), shot: Shot, allowed: &Allowances) -> Flight {
    let (mut vx, mut vy) = shot.velocity();
    let mut x = from.0 as i32 * SUB + SUB / 2;
    let mut y = from.1 as i32 * SUB + SUB / 2;
    let mut path = vec![(x, y)];
    let mut contacts = Vec::new();
    let mut ticks = 0;

    let solid = |tx: i32, ty: i32| -> bool {
        if !world.in_bounds(tx, ty) {
            return true;
        }
        !world.walkable(tx as u8, ty as u8, allowed)
    };

    while ticks < MAX_TICKS {
        if vx * vx + vy * vy < STOP_BELOW * STOP_BELOW {
            break;
        }
        ticks += 1;
        let was = (x / SUB, y / SUB);
        let (nx, ny) = (x + vx, y + vy);
        let (tx, ty) = (nx.div_euclid(SUB), ny.div_euclid(SUB));

        // **Each axis on its own, and the shorter one first.** A diagonal entry
        // into a wall cell is the corner case `PLAN-M17.md` §3 names: resolving
        // both at once either sticks the ball in the corner or tunnels it
        // through. `a_bounce_keeps_the_tangent` is what holds this.
        let hit_x = solid(tx, y.div_euclid(SUB));
        let hit_y = solid(x.div_euclid(SUB), ty);
        if hit_x || hit_y {
            if hit_x {
                vx = -vx * RESTITUTION / 100;
                contacts.push(Contact::Wall { at: clamp_tile(world, tx, y.div_euclid(SUB)) });
            }
            if hit_y {
                vy = -vy * RESTITUTION / 100;
                contacts.push(Contact::Wall { at: clamp_tile(world, x.div_euclid(SUB), ty) });
            }
            // **And the corner nobody entered on either axis.** A ball moving
            // exactly into the diagonal hits a cell neither axis test saw, and
            // without this it tunnels through it.
            if !hit_x && !hit_y {
                vx = -vx * RESTITUTION / 100;
                vy = -vy * RESTITUTION / 100;
            }
            path.push((x, y));
            continue;
        }
        if solid(tx, ty) {
            vx = -vx * RESTITUTION / 100;
            vy = -vy * RESTITUTION / 100;
            contacts.push(Contact::Wall { at: clamp_tile(world, tx, ty) });
            path.push((x, y));
            continue;
        }

        x = nx;
        y = ny;
        path.push((x, y));

        // **Friction is per tile crossed, not per tick**, so a fast ball and a
        // slow one lose the same across the same ground — which is what makes
        // the terrain table a table of *ground* rather than a table of time.
        let now = (x / SUB, y / SUB);
        if now != was {
            let name = world.terrain_name(now.0 as u8, now.1 as u8).to_string();
            let f = friction(&name);
            contacts.push(Contact::Crossed { at: (now.0 as u8, now.1 as u8), friction: f });
            let speed = isqrt(vx * vx + vy * vy);
            if speed <= f {
                vx = 0;
                vy = 0;
            } else {
                vx = vx * (speed - f) / speed;
                vy = vy * (speed - f) / speed;
            }
        }
    }

    let rest = (
        (x.div_euclid(SUB)).clamp(0, world.width as i32 - 1) as u8,
        (y.div_euclid(SUB)).clamp(0, world.height as i32 - 1) as u8,
    );
    Flight { path, contacts, rest, ticks }
}

fn clamp_tile(world: &World, x: i32, y: i32) -> (u8, u8) {
    (
        x.clamp(0, world.width as i32 - 1) as u8,
        y.clamp(0, world.height as i32 - 1) as u8,
    )
}

/// Integer square root, Newton's way.
///
/// **No `f32::sqrt`**, which is the one place a physics loop reaches for a
/// float without noticing. Used once a tile boundary to scale a velocity down
/// by a friction, and exact enough that the scaling is the same everywhere.
pub fn isqrt(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
