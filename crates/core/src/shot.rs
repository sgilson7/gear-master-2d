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

use crate::world::{Allowances, PlaceKind, World, WorldState};

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
///
/// **Measured on the bare map**, before M17.1 put eight obstacles on it. With
/// the obstacles on, both settings are within a tile and the measurement stops
/// discriminating — a bumper's whole job is to send a shot somewhere it would
/// not otherwise go. So the constant is the bare table's answer and
/// `pulling_back_harder_never_lands_you_nearer` is what still holds on the
/// dressed one.
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

/// What a bumper kicks the ball away with, in sub-cells a tick.
///
/// **Added, not multiplied, and that is the whole of it.** `PLAN-M17.md` §4
/// says a bumper *adds 30% speed*, and a percentage compounds: reflected at
/// `RESTITUTION` and boosted thirty percent, a ball between a bumper and a wall
/// gains four percent a round trip — for ever. The first draft ran shots to
/// `MAX_TICKS` and broke `pulling_back_harder_never_lands_you_nearer`, because
/// a ricochet off the pass could outrun a full-power shot.
///
/// A fixed kick cannot: reflect at four fifths and add `k`, and the round trip
/// settles at about `2.2k` whatever it started from. Three power-units puts
/// that at roughly a power-seven shot, which is a bumper that is worth aiming
/// at and is not a second cue.
pub const BUMPER_KICK: i32 = 3 * POWER_UNIT;

/// The fastest a bumper may ever send the ball away, whatever the arithmetic.
///
/// Belt-and-braces behind the paragraph above: a full-power shot's own speed is
/// the ceiling, which is legible as well as safe — the strongest thing a spring
/// can do is throw the ball as hard as you could have.
pub const BUMPER_CAP: i32 = 10 * POWER_UNIT;

/// How many times one flight may be kicked, however many bumpers it finds.
///
/// **A bumper is the only thing on the table that adds energy, and friction
/// cannot be relied on to take it back**: friction is charged per *tile
/// crossed*, and a ball bouncing between a bumper and the wall beside it
/// crosses none. A fixed kick settles to a fixed point rather than running
/// away, and the fixed point is a ball that never stops — which is what
/// `no_legal_shot_reaches_max_ticks` reported, twice, on two different drafts
/// of the arithmetic.
///
/// So the pump is counted. Three is enough to read as a bumper and bounded
/// enough that every flight ends; after the third the bumper is a wall, which
/// is what it already is to a ball approaching it.
pub const BUMPER_KICKS: u32 = 3;

/// What touching a spike costs, in percentage points of fatigue.
pub const SPIKE_TIRES: u32 = 8;

/// What sinking in a pocket costs, on top of the walk home.
///
/// **Twelve and not the walk alone**, which is `PLAN-M17.md` §10.4's choice:
/// a pocket has to be worse than a spike or nobody aims around it.
pub const POCKET_TIRES: u32 = 12;

/// What a landing rolls, as a percentage of the tile's own rate.
///
/// A landing is a longer stay than a step, and a shot map has far fewer
/// landings than a walk has steps.
pub const LANDING_MULT: i32 = 150;

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
    /// Hit a bumper, which threw it back faster than it arrived.
    Bumper { id: String, at: (u8, u8) },
    /// Went through a spike, which cost it nothing and cost **you** eight.
    Spike { id: String, at: (u8, u8) },
    /// Went in a chute here and came out at the other end, keeping its speed
    /// and its heading.
    Chute { id: String, from: (u8, u8), to: (u8, u8) },
    /// Stopped dead in sand.
    Sand { id: String, at: (u8, u8) },
    /// Came to rest in a pocket and sank.
    Sunk { id: String, at: (u8, u8) },
    /// Ran into a gate or a boss, which caught it. The flight ends there.
    Caught { id: String, at: (u8, u8) },
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

    /// Fatigue this flight cost, in percentage points.
    ///
    /// **A shot is free and a contact is not.** Walking costs nothing today and
    /// a shot is the same; what tires you is the table — eight for a spike,
    /// twelve for a pocket — so this is the one number the tape has to carry
    /// that a step never did.
    pub fn tiring(&self) -> u32 {
        self.contacts
            .iter()
            .map(|c| match c {
                Contact::Spike { .. } => SPIKE_TIRES,
                Contact::Sunk { .. } => POCKET_TIRES,
                _ => 0,
            })
            .sum()
    }

    /// One sentence saying what the shot did.
    ///
    /// **Counted, never dramatised** — `TONE.md` rule 3 and rule 4: what it
    /// hit, in order, and where it stopped, with the number in it. The shim
    /// prints this and does not compose one, which is the *page draws what core
    /// sent it* rule applied to prose.
    ///
    /// Named things are named: a bumper and a chute have ids and a wall does
    /// not, so a run of walls is *two off the range* rather than two sentences.
    pub fn tape(&self, n: u32, terrain: &str, per_mille: i32) -> String {
        let mut bits: Vec<String> = Vec::new();
        let walls = self.bounces();
        if walls > 0 {
            bits.push(format!("{walls} off the rock"));
        }
        for c in &self.contacts {
            match c {
                Contact::Bumper { .. } => bits.push("the bumper".into()),
                Contact::Chute { .. } => bits.push("down the drove way".into()),
                Contact::Spike { .. } => {
                    bits.push(format!("a spike ({SPIKE_TIRES})"))
                }
                Contact::Sand { .. } => bits.push("into the drift".into()),
                _ => {}
            }
        }
        if let Some(_) = self.sunk() {
            return format!(
                "Shot {n}. {} — sunk, and back to the last town you stood in with \
                 {POCKET_TIRES}% off you.",
                if bits.is_empty() { "Straight in".to_string() } else { bits.join(", ") }
            );
        }
        let how = if bits.is_empty() { String::new() } else { format!("{}, and ", bits.join(", ")) };
        format!(
            "Shot {n}. {how}stopped on {terrain} at [{}, {}]. {per_mille}‰.",
            self.rest.0, self.rest.1
        )
    }

    /// Where a pocket sank it, if one did.
    pub fn sunk(&self) -> Option<&str> {
        self.contacts.iter().find_map(|c| match c {
            Contact::Sunk { id, .. } => Some(id.as_str()),
            _ => None,
        })
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
    shoot_with(world, &WorldState::default(), from, shot, allowed)
}

/// The same, knowing what has been answered — so a hidden obstacle is not
/// there.
///
/// **A hidden place is ground to a ball**, exactly as it is to a foot, and
/// `place_is_there` is the one answer to whether a place is there at all.
pub fn shoot_with(
    world: &World,
    state: &WorldState,
    from: (u8, u8),
    shot: Shot,
    allowed: &Allowances,
) -> Flight {
    let (mut vx, mut vy) = shot.velocity();
    let mut x = from.0 as i32 * SUB + SUB / 2;
    let mut y = from.1 as i32 * SUB + SUB / 2;
    let mut path = vec![(x, y)];
    let mut contacts = Vec::new();
    let mut ticks = 0;
    let mut kicks = 0u32;

    // **The obstacles on this table, by tile**, resolved once rather than
    // searched for every tick: a flight is four hundred ticks at worst and a
    // linear scan of a map's places inside that is a scan a table does not
    // need.
    let here = |tx: i32, ty: i32| -> Option<&crate::world::PlaceDef> {
        if !world.in_bounds(tx, ty) {
            return None;
        }
        world
            .place_now(state, tx as u8, ty as u8, allowed)
            .filter(|p| p.kind.is_obstacle())
    };
    // **What a diamond does.** `PlaceKind::catches` is gates and bosses, and
    // they stop the ball rather than being flown over — reported from play as
    // *it shouldnt have to perfectly land on it*. Not the tile the shot was
    // taken from: a ball leaving a gate it has just been refused at must be
    // able to leave.
    let catcher = |tx: i32, ty: i32| -> Option<&crate::world::PlaceDef> {
        if !world.in_bounds(tx, ty) || (tx, ty) == (from.0 as i32, from.1 as i32) {
            return None;
        }
        world
            .place_now(state, tx as u8, ty as u8, allowed)
            .filter(|p| p.kind.catches())
    };
    // A bumper is solid to the ball the way a wall is, and it is the only
    // place in the game that is.
    let solid = |tx: i32, ty: i32| -> bool {
        if !world.in_bounds(tx, ty) {
            return true;
        }
        if here(tx, ty).is_some_and(|p| p.kind == PlaceKind::Bumper) {
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
            // **A bumper throws it back faster than it arrived**, which is the
            // one thing on the table that adds energy — so it is read before
            // the reflection rather than after, and it replaces the
            // restitution rather than multiplying it.
            let bounce = |v: i32, tx: i32, ty: i32, kicks: &mut u32| -> (i32, Option<String>) {
                let back = -v * RESTITUTION / 100;
                match here(tx, ty) {
                    Some(p) if p.kind == PlaceKind::Bumper && *kicks < BUMPER_KICKS => {
                        *kicks += 1;
                        let kick = if back >= 0 { BUMPER_KICK } else { -BUMPER_KICK };
                        ((back + kick).clamp(-BUMPER_CAP, BUMPER_CAP), Some(p.id.clone()))
                    }
                    Some(p) if p.kind == PlaceKind::Bumper => (back, Some(p.id.clone())),
                    _ => (back, None),
                }
            };
            if hit_x {
                let ty_ = y.div_euclid(SUB);
                let at = clamp_tile(world, tx, ty_);
                let (v, bump) = bounce(vx, tx, ty_, &mut kicks);
                vx = v;
                contacts.push(match bump {
                    Some(id) => Contact::Bumper { id, at },
                    None => Contact::Wall { at },
                });
            }
            if hit_y {
                let tx_ = x.div_euclid(SUB);
                let at = clamp_tile(world, tx_, ty);
                let (v, bump) = bounce(vy, tx_, ty, &mut kicks);
                vy = v;
                contacts.push(match bump {
                    Some(id) => Contact::Bumper { id, at },
                    None => Contact::Wall { at },
                });
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

        // **A diamond catches, and that is the end of the flight.** Before the
        // obstacles, because being let into somewhere is not a thing the ball
        // can then roll out of — and the ball is put on the tile's centre so
        // that where it stopped is legibly *on* the thing it hit rather than
        // wherever the tick happened to land.
        if let Some(p) = catcher(x.div_euclid(SUB), y.div_euclid(SUB)) {
            let at = (x.div_euclid(SUB) as u8, y.div_euclid(SUB) as u8);
            contacts.push(Contact::Caught { id: p.id.clone(), at });
            x = at.0 as i32 * SUB + SUB / 2;
            y = at.1 as i32 * SUB + SUB / 2;
            path.push((x, y));
            // No need to stop the ball: the loop is over. Zeroing the velocity
            // here would be two ways of saying the same thing and the compiler
            // says so.
            break;
        }

        // **Three of the five happen where the ball *is*, and two of them end
        // the flight.** In this order because that is the order they happen in:
        // a chute moves you, sand stops you, and a spike is something you went
        // through on the way.
        if let Some(p) = here(x.div_euclid(SUB), y.div_euclid(SUB)) {
            match p.kind {
                PlaceKind::Chute => {
                    let to = p.at_to.unwrap_or([from.0, from.1]);
                    contacts.push(Contact::Chute {
                        id: p.id.clone(),
                        from: (x.div_euclid(SUB) as u8, y.div_euclid(SUB) as u8),
                        to: (to[0], to[1]),
                    });
                    // **Speed and heading kept**, which is what a pinball lane
                    // is: it carries the ball rather than throwing it.
                    x = to[0] as i32 * SUB + SUB / 2;
                    y = to[1] as i32 * SUB + SUB / 2;
                    path.push((x, y));
                }
                PlaceKind::Sand => {
                    contacts.push(Contact::Sand {
                        id: p.id.clone(),
                        at: (x.div_euclid(SUB) as u8, y.div_euclid(SUB) as u8),
                    });
                    vx = 0;
                    vy = 0;
                }
                PlaceKind::Spike => {
                    // **Once a flight, however many ticks are spent on it.** A
                    // spike that charged per tick would charge a slow ball
                    // nine times and a fast one once for the same contact.
                    if !contacts.iter().any(|c| matches!(c, Contact::Spike { id, .. } if *id == p.id))
                    {
                        contacts.push(Contact::Spike {
                            id: p.id.clone(),
                            at: (x.div_euclid(SUB) as u8, y.div_euclid(SUB) as u8),
                        });
                    }
                }
                _ => {}
            }
        }

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
    // **A pocket is read at rest and nowhere else.** The ball goes over one at
    // speed, which is the whole of what makes it a pocket rather than a hole:
    // what sinks you is stopping on it.
    if let Some(p) = here(rest.0 as i32, rest.1 as i32) {
        if p.kind == PlaceKind::Pocket {
            contacts.push(Contact::Sunk { id: p.id.clone(), at: rest });
        }
    }
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


/// A shot from `from` that comes to rest on `target`, or near it.
///
/// **Seven hundred and twenty shots, tried, and the first that lands is the
/// answer.** Deterministic, integer, and about a millisecond a shot, so a full
/// search is well under a second — which is what makes it three things at once:
/// the walker's move on a shot map, the reachability lint's flood, and the
/// ghost an assist would draw if the human ever wants one.
///
/// **Ordered by power and then by angle**, so the answer is the *gentlest* shot
/// that gets there rather than whichever the loop happened to find. A player
/// pulls back as far as they need to.
pub fn aim_at(
    world: &World,
    state: &WorldState,
    from: (u8, u8),
    target: (u8, u8),
    near: bool,
    allowed: &Allowances,
) -> Option<Shot> {
    let close = |r: (u8, u8)| {
        r == target
            || (near
                && (r.0 as i32 - target.0 as i32).abs() + (r.1 as i32 - target.1 as i32).abs() <= 1)
    };
    for power in 1..=10u8 {
        for angle in 0..STEPS as u16 {
            let shot = Shot::new(angle, power);
            if close(shoot_with(world, state, from, shot, allowed).rest) {
                return Some(shot);
            }
        }
    }
    None
}
