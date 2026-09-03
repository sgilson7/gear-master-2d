// The page. It draws what core says and asks core what happened; it decides
// nothing. Whether a tile is walkable, whether a fight starts, which creature
// it is, and whether a choice can be taken are all answered on the other side
// of the boundary — a page that works any of those out for itself is a second
// copy of the rules that will disagree with the first.
import init, {
  world_json, position, try_step, event_json, answer,
  save_json, load_json, new_game, apply_preset,
  shop_json, bench_json, buy, buy_supply, buy_ench, use_supply, quests_json, take_quest, hand_in_quest, bank_xp,
  quest_log_json, guide_json, pin_quest,
  character_json, skills_json, take_skill,
  class_offer_json, choose_class, class_name, all_trees_json,
  gold, piece_count, version, save_version,
  board_json, legal_anchors, place, pick_up, rotate, toggle_lock, undo, clear_board,
  look_json, look_over,
  encounter_json, fight_json, settle_fight, flee,
  errand_marks_json, go_home,
  ench_rack_json, attach_ench, detach_ench, toggle_ench,
} from './pkg/gm2d_wasm.js';
import { Board } from './board.js';
import { Theirs } from './theirs.js';
import { shapeCanvas, pieceCardHtml } from './shape.js';
import { Replay } from './replay.js';

const $ = (id) => document.getElementById(id);
const TILE = 32;
const AUTOSAVE = 'gm2d.autosave';

let world = null;
let art = { creatures: {}, places: {} };

// A subject with no figure draws nothing. That is what makes `art.json` safe to
// be incomplete — and it will be incomplete for a long time, because there are
// fifty creatures and seven figures.
function figure(kind, key) {
  const name = art[kind]?.[key];
  return name ? `assets/${name}.svg` : null;
}
let debug = false;
/// Where there is an errand you could act on. Core's answer, refreshed
/// whenever anything could have changed it.
let errandMarks = [];
/// What the map is pointing at: the pinned errand, or whichever the cursor is
/// over in the log. Core's answer, in tiles — the page never works out where a
/// creature lives or which town stocks a thing.
let pinGuide = null;
let hoverGuide = null;
let blocked = null;   // the last refusal, drawn for one frame

// Terrain colours, in the palette the rest of the page uses. Two per terrain:
// the fill, and a slightly shifted second used to break up large expanses so a
// twenty-by-twenty grid does not read as flat blocks of colour.
const LIGHT = {
  road:  ['#e6dcc4', '#e2d7bd'],
  grass: ['#bfcfa6', '#b8c99e'],
  scrub: ['#cfc794', '#c8c08b'],
  wood:  ['#7d9a6d', '#759266'],
  slag:  ['#a49c92', '#9d958b'],
  cork:  ['#cf9f66', '#c9985e'],
  town:  ['#8a5c1c', '#8a5c1c'],
  rock:  ['#77796f', '#71736a'],
  water: ['#7ba0ad', '#7399a7'],
  // The Treyway's, at the scale of a country. Paler and flatter than
  // Bambulon's on purpose: a plain is a week of walking drawn as one tile, and
  // it should not read as a field you can see the far side of.
  plain: ['#d6d8bc', '#d1d4b6'],
  coast: ['#e8e2c8', '#e4dec2'],
  range: ['#8c8478', '#867e72'],
  sea:   ['#5d8494', '#567d8e'],
  // The Drambus Stack. Two hundred and ten feet of it, drawn as itself rather
  // than as rock, which is the whole reason it is its own terrain.
  curd:  ['#e8cf7a', '#e3c96f'],
  // What is under the lake once the Stack comes down. Drawn in no map file:
  // it is what `water` becomes.
  lakebed: ['#8f8567', '#89805f'],
};
const DARK = {
  road:  ['#4a4132', '#514837'],
  grass: ['#2c3d2a', '#31432f'],
  scrub: ['#413c26', '#48432b'],
  wood:  ['#1f3524', '#243b29'],
  slag:  ['#38352f', '#3e3b34'],
  cork:  ['#54402a', '#5c472f'],
  town:  ['#d3a05a', '#d3a05a'],
  rock:  ['#232622', '#282b27'],
  water: ['#223941', '#264048'],
  plain: ['#3a3f2f', '#404535'],
  coast: ['#4a4a36', '#50503b'],
  range: ['#2a2723', '#302d28'],
  sea:   ['#1a2c33', '#1e323a'],
  curd:  ['#6a5c26', '#71622b'],
  lakebed: ['#3b3524', '#423b29'],
};

function dark() {
  const t = document.documentElement.dataset.theme;
  if (t === 'dark') return true;
  if (t === 'light') return false;
  return matchMedia('(prefers-color-scheme: dark)').matches;
}

function ink() {
  return getComputedStyle(document.body).getPropertyValue('--ink').trim() || '#171a19';
}

/// The canvas is the map's size, not a fixed square.
///
/// The overworld is twenty by twenty and the first dungeon is nine by five; a
/// canvas pinned to the larger left the cave floating in a screen of nothing.
function fitMap() {
  const c = $('map');
  const w = (world?.width ?? 20) * TILE;
  const h = (world?.height ?? 20) * TILE;
  if (c.width !== w) c.width = w;
  if (c.height !== h) c.height = h;
}

function draw() {
  fitMap();
  const c = $('map');
  const g = c.getContext('2d');
  const pal = dark() ? DARK : LIGHT;
  const pos = JSON.parse(position());

  g.clearRect(0, 0, c.width, c.height);
  for (let y = 0; y < world.height; y++) {
    for (let x = 0; x < world.width; x++) {
      const name = world.rows[y][x];
      const pair = pal[name] || ['#f0f', '#f0f'];
      g.fillStyle = pair[(x + y) % 2];
      g.fillRect(x * TILE, y * TILE, TILE, TILE);
    }
  }

  // **A ring where there is something to do.** Drawn over whatever the place
  // already is, rather than as a fourth shape, because "there is an errand
  // here" is a fact *about* a place and not a kind of place. Gold to take one,
  // brass-on-white to hand one in — the second is the better news, so it is
  // the louder mark.
  for (const m of errandMarks) {
    const [mx, my] = m.at;
    const ccx = mx * TILE + TILE / 2, ccy = my * TILE + TILE / 2;
    g.beginPath();
    g.arc(ccx, ccy, TILE / 2 - 2, 0, Math.PI * 2);
    g.strokeStyle = m.mark === 'hand-in' ? '#f0c85a' : pal.town[0];
    g.lineWidth = m.mark === 'hand-in' ? 3 : 2;
    g.setLineDash(m.mark === 'hand-in' ? [] : [5, 4]);
    g.stroke();
    g.setLineDash([]);
  }

  // **What the pin points at, as motion.** The map already carries terrain hue,
  // region shade, place marks and the player; a fifth hue would be a fifth
  // thing to tell apart. So the region breathes and the ring's dashes march,
  // and neither reads as a new kind of ground.
  //
  // Core says which tiles. The page never works out where a Whisperling lives.
  const guide = hoverGuide ?? pinGuide;
  if (guide) {
    const t = performance.now();
    const breathe = 0.06 + 0.09 * (0.5 + 0.5 * Math.sin(t / 420));
    g.fillStyle = `rgba(240,200,90,${breathe.toFixed(3)})`;
    for (const [x, y] of guide.regions ?? []) g.fillRect(x * TILE, y * TILE, TILE, TILE);
    g.strokeStyle = '#f0c85a';
    g.lineWidth = 3;
    g.setLineDash([6, 5]);
    g.lineDashOffset = -(t / 40) % 11;
    for (const [x, y] of guide.places ?? []) {
      g.beginPath();
      g.arc(x * TILE + TILE / 2, y * TILE + TILE / 2, TILE / 2 - 1, 0, Math.PI * 2);
      g.stroke();
    }
    g.setLineDash([]);
    g.lineDashOffset = 0;
  }

  // Places. A town is a filled square with a ring; an event is a small mark —
  // deliberately not a letter, because a letter on a 32px tile is a letter
  // nobody reads.
  for (const p of world.places) {
    const [x, y] = p.at;
    const cx = x * TILE + TILE / 2;
    const cy = y * TILE + TILE / 2;
    g.strokeStyle = ink();
    g.lineWidth = 2;
    if (p.kind === 'gate' || p.kind === 'boss') {
      // A gate is a way off this map and a boss is a thing standing on it;
      // both are worth more than the small mark an event gets.
      // A diamond, drawn off the tile centre this loop already has — the
      // first version reached for `px`/`py`, which belong to the terrain loop
      // above and are out of scope here.
      const r = TILE / 2 - 3;
      g.fillStyle = p.kind === 'gate' ? pal.town[0] : pal.rock[0];
      g.beginPath();
      g.moveTo(cx, cy - r);
      g.lineTo(cx + r, cy);
      g.lineTo(cx, cy + r);
      g.lineTo(cx - r, cy);
      g.closePath();
      g.fill();
      g.strokeStyle = ink();
      g.lineWidth = 1.5;
      g.stroke();
    } else if (p.kind === 'door') {
      // **Its own mark, not the diamond.** A gate leads to another map and a
      // door leads out of everything that is written; drawing them the same
      // would say you could come back from one the way you come back from the
      // other. An arch with a keyhole in it, on the tile rather than over it.
      const w = TILE - 12, h = TILE - 8;
      const ax = x * TILE + 6, ay = y * TILE + TILE - 4;
      g.fillStyle = '#f0c85a';
      g.beginPath();
      g.moveTo(ax, ay);
      g.lineTo(ax, ay - h + w / 2);
      g.arc(ax + w / 2, ay - h + w / 2, w / 2, Math.PI, 0);
      g.lineTo(ax + w, ay);
      g.closePath();
      g.fill();
      g.strokeStyle = ink();
      g.lineWidth = 1.5;
      g.stroke();
      g.fillStyle = ink();
      g.beginPath();
      g.arc(ax + w / 2, ay - h / 2, 2.4, 0, Math.PI * 2);
      g.fill();
      g.fillRect(ax + w / 2 - 1, ay - h / 2, 2, 5);
    } else if (p.kind === 'bench') {
      // **A table, and it is the only one.** Not a town's filled square — he
      // is not a place, he is a man who is there this week. Two legs and a top,
      // in the page's ink, which is a shape nothing else on this map draws.
      const w = TILE - 12, top = y * TILE + 10, bot = y * TILE + TILE - 6;
      g.strokeStyle = ink();
      g.lineWidth = 3;
      g.beginPath();
      g.moveTo(cx - w / 2, top); g.lineTo(cx + w / 2, top);
      g.moveTo(cx - w / 2 + 2, top); g.lineTo(cx - w / 2 + 2, bot);
      g.moveTo(cx + w / 2 - 2, top); g.lineTo(cx + w / 2 - 2, bot);
      g.stroke();
      g.lineWidth = 2;
    } else if (p.kind === 'crossing') {
      // **Its own mark, and neither of the two it is nearest.** A gate is a
      // diamond and leads off the map; a door is an arch and leads out of what
      // is written. A crossing is a milestone on a road that carries on, so it
      // is a post with a bar across it — an upright and a crossbar, which is
      // the one shape on this map that is a line rather than a body.
      //
      // Checked against the three channels `look.rs` keeps: the mark is a
      // shape nothing else here draws, it is the page's ink rather than a
      // sixth hue, and it stands a whole tile tall so it survives 32px. The
      // crossbar goes from the post rather than through it, so it reads as a
      // barrier and not as a plus sign.
      const bx = cx, top = y * TILE + 5, bot = y * TILE + TILE - 5;
      g.strokeStyle = ink();
      g.lineWidth = 3;
      g.beginPath();
      g.moveTo(bx, top); g.lineTo(bx, bot);
      g.moveTo(bx, top + 4); g.lineTo(bx + TILE / 2 - 4, top + 4);
      g.stroke();
      g.lineWidth = 2;
    } else if (p.kind === 'town') {
      g.fillStyle = pal.town[0];
      g.fillRect(x * TILE + 6, y * TILE + 6, TILE - 12, TILE - 12);
      g.strokeRect(x * TILE + 6, y * TILE + 6, TILE - 12, TILE - 12);
    } else {
      g.beginPath();
      g.moveTo(cx, cy - 7); g.lineTo(cx + 7, cy); g.lineTo(cx, cy + 7); g.lineTo(cx - 7, cy);
      g.closePath();
      g.stroke();
    }
  }

  // The player: a gear, because that is what you are.
  const px = pos.x * TILE + TILE / 2;
  const py = pos.y * TILE + TILE / 2;
  g.fillStyle = ink();
  g.beginPath();
  g.arc(px, py, 9, 0, Math.PI * 2);
  g.fill();
  g.strokeStyle = pal.town[0];
  g.lineWidth = 3;
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2;
    g.beginPath();
    g.moveTo(px + Math.cos(a) * 9, py + Math.sin(a) * 9);
    g.lineTo(px + Math.cos(a) * 13, py + Math.sin(a) * 13);
    g.stroke();
  }

  if (debug && world.scouting) drawDebug(g, pos);

  if (blocked) {
    g.fillStyle = 'rgba(139,66,37,.9)';
    g.fillRect(0, c.height - 26, c.width, 26);
    g.fillStyle = '#f4f5ef';
    g.font = '13px ui-monospace, Menlo, monospace';
    g.fillText(blocked, 10, c.height - 8);
  }
}

/// Keep redrawing while something is pointing.
///
/// **Only while there is something to point at.** A map that repainted for ever
/// would burn a core to draw a picture that has not changed. Throttled to about
/// twelve frames a second because the animation is a breath and a crawl, and
/// each frame costs a call across the boundary for the player's position.
let pulsing = null;
let pulsedAt = 0;

function pulse(now) {
  if (!(hoverGuide ?? pinGuide)) { pulsing = null; return; }
  if (now - pulsedAt > 80) { pulsedAt = now; draw(); }
  pulsing = requestAnimationFrame(pulse);
}

function startPulse() {
  if (pulsing === null && (hoverGuide ?? pinGuide)) pulsing = requestAnimationFrame(pulse);
}

// The overlay the milestone asks for: every tile's encounter chance, drawn on
// the tile. Danger is a region property and is shown in the panel; what
// changes tile to tile is the terrain underneath it, and this is that number.
function drawDebug(g, pos) {
  g.font = '9px ui-monospace, Menlo, monospace';
  g.textAlign = 'center';
  for (let y = 0; y < world.height; y++) {
    for (let x = 0; x < world.width; x++) {
      const name = world.rows[y][x];
      if (!world.walk[y][x]) continue;
      const chance = world.chances[y][x];
      g.fillStyle = 'rgba(0,0,0,.55)';
      g.fillRect(x * TILE, y * TILE + TILE - 12, TILE, 12);
      g.fillStyle = '#f4f5ef';
      g.fillText(String(chance), x * TILE + TILE / 2, y * TILE + TILE - 3);
    }
  }
  g.textAlign = 'left';
  g.fillStyle = 'rgba(0,0,0,.7)';
  g.fillRect(0, 0, 300, 20);
  g.fillStyle = '#f4f5ef';
  g.font = '11px ui-monospace, Menlo, monospace';
  g.fillText(`per-mille chance per tile · region danger ${pos.danger ?? '—'}`, 6, 14);
}

function paintPanel() {
  const p = JSON.parse(position());
  // **The map the page is holding is the map core says you are on.**
  //
  // The grid is cached because building it is four hundred strings, and a
  // cached grid goes stale exactly when the player is *moved* rather than
  // walked. Seven call sites re-read it by hand and an eighth did not: a
  // defeat walks you home, home can be on another map since M8, and nothing
  // on that path told the page — so dying in the Cave left the canvas drawing
  // a nine-by-five room with the player standing at (1, 18) of it. Found by
  // `make play` on the Treyway, where the same defeat is three times likelier.
  //
  // Fixed as a class rather than as an eighth call site: every path that moves
  // anybody repaints this panel, and the cheap call now carries the map id.
  if (world && p.map && p.map !== world.id) {
    world = JSON.parse(world_json());
    draw();
  }
  const c = JSON.parse(character_json());
  $('level').textContent = c.level;
  $('xp').textContent = `${c.into} / ${c.needed}`;
  // What a defeat would cost. Named on the standing panel because it is the
  // number the next step out of town is weighed against.
  $('carrying-panel').textContent = c.carried > 0 ? `${c.carried} at risk` : 'nothing';
  $('fatigue').textContent = c.fatigue > 0
    ? `${c.fatigue}%${c.fatigue >= c.fatigue_cap ? ' — all of it' : ''}` : 'not at all';
  paintPack(c);
  $('points').textContent = c.points;
  $('skills').classList.toggle('primary', c.points > 0);
  $('class').textContent = class_name() || '—';
  $('region').textContent = p.region ?? '—';
  $('terrain').textContent = p.terrain;
  $('coords').textContent = `${p.x}, ${p.y}`;
  // Null rather than zero until the tree grants the reading: zero is a number
  // and would be a lie, and a screen cannot tell a lie from a bug.
  $('chance').textContent = p.scouting ? `${p.chance} / 1000` : 'you could not say';
  $('danger').textContent = p.scouting ? (p.danger ?? '—') : 'you could not say';
  // **The lens, and what it is doing.** Numbers core computed; the page prints
  // them. Hidden on every map that is not being surveyed, which is all of them
  // but one.
  const lens = world?.survey ?? null;
  $('survey-row').hidden = !lens;
  if (lens) {
    const bits = [];
    if (lens.encounter_pct) bits.push(`${lens.encounter_pct > 0 ? '+' : ''}${lens.encounter_pct}% encounters`);
    if (lens.drops_per_mille) bits.push(`+${lens.drops_per_mille}/1000 drops`);
    if (lens.xp_pct) bits.push(`+${lens.xp_pct}% experience`);
    if (lens.golem) bits.push('a golem takes the first fight');
    $('survey').textContent = `${lens.kind}${bits.length ? ` — ${bits.join(', ')}` : ''}`;
  }
  // The button is the skill's, so it is not there without it.
  $('scout').hidden = !p.scouting;
  // And the way home is the set's. Offering a click that will be refused is a
  // worse screen than not offering it — the rack's lesson, and the same answer.
  $('homeward').hidden = !p.homeward;
  if (!p.scouting && debug) toggleScout();
  $('walked').textContent = p.walked;
  $('fights').textContent = p.fights;
  $('gold').textContent = gold();
  paintSheet(c);
  paintYou(c.class);
  refreshErrandMarks();
}

/// The character sheet: what you are, and what you walk into a fight holding.
///
/// Reported by core and printed unchanged. Nothing showed this before, so a
/// point spent on strength or max health produced no visible change anywhere —
/// which is indistinguishable from a skill that does nothing.
/// Your own figure, which becomes your class's once you have one.
///
/// The Sprocketman is who you are before anybody has decided what you are; the
/// fork is where that stops being true, and it does not come off. A panel that
/// went on drawing the generic figure afterwards would be the one screen in
/// the game that had not noticed.
function paintYou(canonical) {
  const cls = canonical ?? JSON.parse(character_json()).class;
  const src = (cls && figure('classes', cls)) || (art.player ? `assets/${art.player}.svg` : null);
  portrait($('player-art'), src, cls ? class_name() : 'you');
}

/// The tins you are carrying, usable where you stand.
///
/// On the standing panel rather than in town, because the decision fatigue
/// exists to create is the one on the road: another fight, or open the tin, or
/// turn round. A restorative you could only drink somewhere safe would be a
/// restorative you never needed.
function paintPack(c) {
  const box = $('kit');
  const tins = c.supplies ?? [];
  if (!tins.length) {
    box.innerHTML = c.fatigue > 0
      ? `<p class="note">Nothing to take for it. A town sells tins.</p>` : '';
    return;
  }
  box.replaceChildren();
  for (const t of tins) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'tin';
    b.disabled = c.fatigue === 0;
    b.innerHTML = `<b>${t.name} ×${t.n}</b><span class="meta">takes off ${t.restores}%</span>`;
    b.title = t.blurb;
    b.onclick = () => {
      const r = JSON.parse(use_supply(t.id));
      log(r.error || `${t.name}. ${r.took}% of it comes off; ${r.fatigue}% left.`, !!r.error);
      paintPanel(); draw(); autosave();
    };
    box.appendChild(b);
  }
}

function paintSheet(c) {
  const rows = (c.stats ?? [])
    .filter((s) => s.n)
    .map((s) => `<li><b>${s.n}${s.unit}</b> ${s.label}</li>`);
  // Both numbers, because "160, and 24 of it is missing" is the pair a player
  // decides on.
  if (c.fatigue > 0) {
    const now = (c.stats ?? []).find((s) => s.label === 'max health')?.n ?? 0;
    rows.push(`<li class="dim">${c.rested_health} rested — ` +
              `<b>${c.rested_health - now}</b> of it worn off</li>`);
  }
  // Armour and mana are the odd pair: at the character level they are only
  // meaningful as what you *begin* a fight with, which is the tree's doing.
  //
  // **The engine's words, matching the node exactly.** A node that reads
  // "start every fight with 12 armor" and a sheet that reads "12 cork" are the
  // same number wearing two names, and the entire job of this line is to let
  // somebody who spent a point confirm they got what it said. An item card
  // still says Cork, because a card is about the item rather than about a
  // promise somebody is checking.
  const held = c.held ?? {};
  for (const [n, label] of [[held.armor, 'armor'], [held.mana, 'mana']]) {
    if (n) rows.push(`<li class="dim">you start every fight holding <b>${n}</b> ${label}</li>`);
  }
  // **And the rules**, from the tree and from whatever is assembled on the
  // board. A rule moves no bar and prints no number of its own, so without
  // this there is no way at all to tell one that works from one that does not
  // — which is how eight nodes cost a point and did nothing for two
  // milestones. The line is core's, unthemed, and the hover explains it.
  for (const r of c.rules ?? []) {
    rows.push(`<li class="rule" title="${(r.detail ?? []).join(' ')}">${r.line}</li>`);
  }
  $('sheet').innerHTML = rows.join('') || `<li class="none">nothing yet</li>`;
}

/// Ask core where the errands are. Cheap, and called wherever one could have
/// moved: a panel repaint covers taking, handing in, walking and loading.
function refreshErrandMarks() {
  const m = JSON.parse(errand_marks_json());
  errandMarks = m.map === (world?.id ?? m.map) ? m.places : [];
  refreshPin();
}

/// What the pinned errand points at here, if anything.
///
/// `guide_json` answers `null` for an errand whose target is on another map,
/// which is the honest answer: the cave has no Whisperlings in it.
function refreshPin() {
  const errands = JSON.parse(quest_log_json());
  pinGuide = errands.pinned ? JSON.parse(guide_json(errands.pinned)) : null;
  startPulse();
}

/// The per-tile odds, for somebody who has earned them.
///
/// **Not a debug overlay any more.** `#numbers` handed the map's danger and
/// its odds to everybody for nothing, which made a skill that grants them a
/// skill that grants nothing. The button only exists once the node is taken,
/// and core decides that — the page asks and draws.
function toggleScout() {
  debug = !debug;
  $('scout').textContent = debug ? 'Hide the odds' : 'Odds on every tile';
  $('scout').setAttribute('aria-pressed', String(debug));
  draw();
}

// ------------------------------------------------------------------- the log

/// Everything the game has said this sitting, in order.
///
/// **Session-only, and that is a decision** (`PLAN-M11.md` §8 row 10): a save
/// carries the world, not what the screen said about it. A transcript in every
/// save file would be a save seam and a diary.
const history = [];

/// How many lines the always-up strip keeps. The rest is behind HISTORY.
const TAPE = 4;

/// **The one place the game talks.**
///
/// Every message the world sends the player goes through here — a crossing
/// refusing, an errand taken or handed in, a drop, a tin, a banking, a save.
/// It used to be a `<p id="says">` below the save panel: a slot that existed
/// because nothing owned it, which is exactly the shape that ships a feature
/// invisible (M8's curses worked for three milestones and no screen said so).
/// One function owns the slot now, so a message cannot miss it.
///
/// Presentation only. Core sends what it sent before; this decides where it
/// lands and nothing about what it says.
function log(text, bad = false) {
  const line = String(text ?? '').trim();
  if (!line) return;
  history.push({ text: line, bad: !!bad });
  paintTape();
  if (!$('history').hidden) paintHistory();
}

function lines(list, into) {
  into.replaceChildren(...list.map((e) => {
    const li = document.createElement('li');
    li.textContent = e.text;
    li.classList.toggle('bad', e.bad);
    return li;
  }));
}

function paintTape() {
  lines(history.slice(-TAPE), $('tape'));
}

function paintHistory() {
  lines(history, $('history-list'));
  // The end of it is what you opened it to read.
  const box = $('history-list');
  box.lastElementChild?.scrollIntoView({ block: 'nearest' });
}

function openHistory() {
  paintHistory();
  $('history').hidden = false;
  $('history-close').focus();
}

function closeHistory() {
  $('history').hidden = true;
  $('map').focus();
}

function autosave() {
  try { localStorage.setItem(AUTOSAVE, save_json()); } catch { /* private window */ }
}

// ---------------------------------------------------------------- the card

function showCard(title, prose, choices, onPick) {
  $('card-title').textContent = title;
  $('card-prose').replaceChildren(...prose.map((t) => {
    const p = document.createElement('p');
    p.textContent = t;
    return p;
  }));
  const box = $('card-choices');
  box.replaceChildren();
  choices.forEach((c, i) => {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'choice';
    b.disabled = c.takeable === false;
    b.innerHTML = `<b>${c.label}</b><span>${c.takeable === false ? c.unmet : c.blurb}</span>`;
    b.onclick = () => onPick(i);
    box.appendChild(b);
  });
  $('card-receipt').hidden = true;
  $('card-bar').hidden = choices.length > 0;
  $('card').hidden = false;
}

function closeCard() {
  $('card-errands').hidden = true;
  $('card').hidden = true;
  $('map').focus();
}

function openEvent(id) {
  const e = JSON.parse(event_json(id));
  if (e.error) { log(e.error, true); return; }
  paintErrands($('card-errands'), null);
  // **Spent doors are not offered again.** The card reopens because the place
  // may still have an errand on it; the choices were answered once and that
  // was right.
  showCard(e.title, e.prose, e.spent ? [] : e.choices, (i) => {
    const r = JSON.parse(answer(id, i));
    if (r.error) { log(r.error, true); return; }
    $('card-choices').replaceChildren();
    const box = $('card-receipt');
    box.replaceChildren(...(r.receipt.length ? r.receipt : ['Nothing you could point to'])
      .map((line) => { const p = document.createElement('p'); p.textContent = line; return p; }));
    box.hidden = false;
    $('card-bar').hidden = false;
    paintPanel(); draw(); autosave();
  });
}

// ---------------------------------------------------------------- the fight

let board = null;
let replay = null;
let theirs = null;

/// Packing in a town rather than in front of something. There is nothing to
/// fight, so the advance slot belongs to the way out.
let packingOnly = false;

/// Which stage is showing, and what the one action row holds while it does.
///
/// **The bar does not move; its contents change.** The advance button is
/// always first and always the same width, so Fight, Skip to the end and Walk
/// on are one target. Before this each stage carried its own row and the three
/// stages are 15, 19 and 3 lines tall.
function stage(which) {
  for (const s of ['board', 'replay', 'result']) $(`stage-${s}`).hidden = s !== which;
  const board = which === 'board';
  $('go').hidden = !board || packingOnly;
  $('skip').hidden = which !== 'replay';
  $('done').hidden = which !== 'result';
  for (const id of ['run', 'undo', 'preset', 'clear', 'fight-save']) $(id).hidden = !board;
  // In a town there is nothing to fight, so the way out takes the slot rather
  // than leaving it empty and sliding everything left.
  const takes = packingOnly && board;
  $('run').classList.toggle('advance', takes);
  $('run').classList.toggle('primary', takes);
}

function openFight() {
  const m = JSON.parse(encounter_json());
  if (!m) return;
  $('fight-rank').textContent = m.rank === 'ordinary' ? 'an encounter' : m.rank;
  $('fight-name').textContent = m.name;
  portrait($('fight-art'), figure('creatures', m.canonical), m.name);
  $('fight-note').textContent = m.note ?? '';
  $('fight-rating').textContent = m.rating;
  $('fight-bounty').textContent = m.bounty;
  paintTheirs(m);
  showTab('yours');
  $('fight').hidden = false;
  stage('board');
  board.refresh();
}

/// What you are about to fight, in the same cards as your own gear.
///
/// A creature packs a board through the identical pipeline in core — it seats
/// components, they assemble or they do not, and what comes out is items with
/// stats and a cadence. For six milestones the page threw all of that away and
/// printed a name.
function paintTheirs(m) {
  $('theirs-title').textContent = m.name;
  portrait($('theirs-art'), figure('creatures', m.canonical), m.name);

  const secs = (ms) => (ms / 1000).toFixed(2);
  const body = [
    `<li><b>${m.health}</b> health</li>`,
    `<li><b>${m.strength}</b> strength</li>`,
  ];
  if (m.regen) body.push(`<li><b>${m.regen}</b> health a second</li>`);
  // Its own teeth, which stand on no gear and are the one thing it always has.
  for (const a of m.attacks ?? []) {
    const what = [
      a.damage ? `${a.damage} damage` : null,
      a.mind ? `${a.mind} to your maximum health` : null,
      a.armor ? `${a.armor} armor for itself` : null,
    ].filter(Boolean).join(', ') || 'nothing';
    body.push(`<li>${a.name} — <b>${what}</b> <span class="dim">every ${secs(a.cooldown_ms)}s</span></li>`);
  }
  $('theirs-body').innerHTML = body.join('');

  const { html, any } = cards(m.slots ?? []);
  $('theirs-cards').innerHTML = any ? html
    : `<p class="empty">It is wearing nothing. Everything it does, it does with its own body.</p>`;
  theirs.load(m.slots ?? []);
  // A creature with no gear gets no grid: an empty black box under the stats
  // says "something failed to draw", not "it is wearing nothing".
  $('theirs-board').closest('.theirs-grids').hidden = !theirs.slots.length;
  $('tab-theirs').textContent =
    `What it is wearing${any ? ` (${(m.items ?? []).length})` : ''}`;
}

/// Light the card for the item being pointed at, on either side.
function lightCard(root, key) {
  let target = null;
  for (const el of root.querySelectorAll('.made-item')) {
    const on = el.dataset.key === key;
    el.classList.toggle('pointed', on);
    if (on) target = el;
  }
  if (target) target.scrollIntoView({ block: 'nearest' });
}

function showTab(which) {
  for (const w of ['yours', 'theirs']) {
    const on = w === which;
    $(`panel-${w}`).hidden = !on;
    $(`tab-${w}`).classList.toggle('on', on);
    $(`tab-${w}`).setAttribute('aria-selected', String(on));
  }
}

function closeFight() {
  $('fight').hidden = true;
  paintPanel(); draw(); autosave();
  // A fight is where a level lands, so it is where the fork is offered.
  if (!offerClass()) $('map').focus();
}

function runFight() {
  const fought = JSON.parse(fight_json());
  if (fought.error) { boardSays(fought.error); return; }
  stage('replay');
  replay.load(fought);
  replay.onend = () => {
    const s = JSON.parse(settle_fight());
    stage('result');
    $('result-title').textContent =
      s.outcome === 'victory' ? 'It stops moving' : 'You stop moving';
    $('result-receipt').replaceChildren(...s.receipt.map((line) => {
      const p = document.createElement('p'); p.textContent = line; return p;
    }));
    // The receipt is read on the result screen and then walked away from. It
    // is also the only place a drop is ever named, so it goes in the log —
    // where a player can go back and find out what that thing was called.
    for (const line of s.receipt) log(line, s.outcome !== 'victory');
    // **Repaint behind the result.** A fight settles here — the purse, what is
    // carried and how worn out you are all move — and the standing panel used
    // to keep the pre-fight numbers until the result was dismissed.
    paintPanel();
    autosave();
  };
  replay.play();
}

// ---------------------------------------------------------------- the rack

/// The ench in hand, waiting for a component to go on. Null when nothing is
/// picked up.
let holdingEnch = null;

/// What you own loose, and what is already bolted to something.
///
/// **Only for a licensee.** Enching is what the Kaklon Patent is, so the rack
/// arrives with the class rather than with a point spent inside it — and
/// whether this character has it is core's answer, not a class name the page
/// compared for itself.
function paintRack() {
  const r = JSON.parse(ench_rack_json());
  // **An ench you own is shown whether or not you can use one.**
  //
  // An errand pays The Yodregar Index and pays it to everybody — `quest.rs`
  // says why in as many words: *a reward that vanished for three players in
  // four would be worse than one they cannot use yet.* That reasoning assumes
  // the player can see they have it, and they could not: the rack is the only
  // screen an ench appears on and it was hidden outright unless you were a
  // Kaklon Licensee. So the town said it had paid you, the save carried it, and
  // no screen in the game would show it. Reported as *"the quest the frame that
  // stands did not pay the yodregar index"*, and from where the player sits
  // that is exactly what happened.
  //
  // Still hidden for somebody holding none, which is what it was for: a rack
  // of nothing on a screen you cannot use is noise.
  const shelved = (r.loose ?? []).length || (r.on ?? []).length;
  $('rack').hidden = !r.licensed && !shelved;
  $('rack-note').textContent = r.licensed
    ? `Click an ench, then click the component you want it on. Click a bolted ` +
      `one to switch it off; click it again to take it back. One ench a component.`
    : `Yours, and nothing you can do with one yet — bolting an ench to a ` +
      `component is the Kaklon Patent's, and you are not a licensee.`;
  if (!r.licensed) {
    holdingEnch = null;
    $('rack-on').replaceChildren();
    const loose = $('rack-loose');
    loose.replaceChildren();
    for (const e of r.loose) {
      const d = document.createElement('div');
      // Not a button. The gesture is refused in core — `attach_ench` says "The
      // bench is not for you" — and offering a click that is going to be
      // refused is a worse screen than not offering it.
      d.className = 'wares ench sold';
      d.dataset.ench = e.id;
      d.innerHTML = `<b>${e.name}${e.have > 1 ? ` ×${e.have}` : ''}</b>` +
        `<span class="spec">${e.spec}</span>` +
        `<span class="flavour">${e.blurb}</span>` +
        `<span class="cost">waiting on a licence</span>`;
      loose.appendChild(d);
    }
    return;
  }

  const loose = $('rack-loose');
  loose.replaceChildren();
  if (!r.loose.length) {
    const p = document.createElement('p');
    p.className = 'note';
    p.textContent = 'Nothing loose. What you have is bolted to something.';
    loose.appendChild(p);
  }
  for (const e of r.loose) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares ench' + (holdingEnch === e.id ? ' pin' : '');
    b.dataset.ench = e.id;
    b.innerHTML = `<b>${e.name}${e.have > 1 ? ` ×${e.have}` : ''}</b>` +
      `<span class="spec">${e.spec}</span>` +
      `<span class="flavour">${e.blurb}</span>` +
      `<span class="cost">${holdingEnch === e.id ? 'now click a component' : 'pick it up'}</span>`;
    b.onclick = () => {
      holdingEnch = holdingEnch === e.id ? null : e.id;
      boardSays(holdingEnch ? `${e.name}. Click the component it goes on.` : '');
      paintRack();
    };
    loose.appendChild(b);
  }

  const on = $('rack-on');
  on.replaceChildren();
  for (const e of r.on) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares ench' + (e.active ? '' : ' sold');
    b.dataset.enchOn = String(e.piece);
    b.innerHTML = `<b>${e.name}</b>` +
      `<span class="spec">${e.spec}</span>` +
      `<span class="meta">on ${e.on}</span>` +
      `<span class="cost">${e.active ? 'switched on — click to switch off'
                                     : 'switched off — click to take it back'}</span>`;
    // One button, two steps, and in that order on purpose: switching off is
    // the reversible half and is what somebody trying an arrangement wants,
    // so it is the first click rather than the second.
    b.onclick = () => {
      if (e.active) {
        toggle_ench(e.piece);
        boardSays(`${e.name} switched off.`);
      } else {
        detach_ench(e.piece);
        boardSays(`${e.name} back in the rack.`);
      }
      $('board-says').classList.remove('bad');
      board.refresh();
      paintRack();
      autosave();
    };
    on.appendChild(b);
  }
}

/// A component was clicked while an ench was in hand.
function bolt(pieceId) {
  const why = attach_ench(holdingEnch, pieceId);
  if (why) { boardSays(why); return false; }
  holdingEnch = null;
  board.refresh();
  paintRack();
  autosave();
  return true;
}

// What the five grids made, beside the board rather than crammed under it.
//
// Every judgement here is core's: which pieces form an item, whether it came
// together, what it is called, what it is worth, and what it is missing if it
// did not. The panel renders `status` unchanged — that sentence is the engine's
// and it is better than any summary of it.
function paintMade(st) {
  const { html, any } = cards(st.slots);
  $('panel-yours').innerHTML = `<h4>What the frames made</h4>` +
    (any ? html
         : `<p class="empty">Nothing seated yet. Click a component in the bag, then click a cell.</p>`);
}

/// Every item on a set of grids, as cards.
///
/// One builder for both sides. The creature's gear goes through the identical
/// pipeline in core, so it deserves the identical card — and a second copy of
/// this would be a second answer to "is cork a standing stat", which is the
/// question the two halves below exist to settle.
const sign = (n, unit) => `${n > 0 ? '+' : ''}${n}${unit}`;
// "built on a Oak Handle" was on every card with a vowel-headed core.
const article = (n) => (n ? `${/^[aeiou]/i.test(n) ? 'an' : 'a'} ${n}` : '—');
const secs = (ms) => (ms / 1000).toFixed(2);

/// One item, as a card.
///
/// **One builder, four panels.** Your board, the creature's board and both
/// sides of the replay draw this. Two copies would be two answers to "is cork
/// a standing stat", which is the question the two halves exist to settle.
function oneCard(i, where) {
  if (!i.assembled) {
    return (
  `<div class="made-item short" data-key="${i.pieces.join(',')}">` +
  `<b>${i.short || 'loose pieces'}</b>` +
  `<span class="why">${i.status}</span></div>`);
  }
  // Rarity as pips as well as a word, so the tier reads without the
  // colours needing to be told apart — same rule as on the board.
  const pips = '◆'.repeat(i.marks) + '◇'.repeat(3 - i.marks);
  const next = i.next_at !== null && i.next_at !== undefined
    ? `<span class="next">${i.next_at} more for the next mark</span>` : '';

  // Standing still: what it contributes whether or not a fight is on.
  const passive = i.passive.length
    ? i.passive.map((s) => `<li>${sign(s.n, s.unit)} ${s.label}</li>`).join('')
    : `<li class="none">nothing — it only acts in a fight</li>`;

  // Every activation. Cork, the Funny, fury, devotion and harvest live
  // here, not above: they are laid down per tick and reset each fight, and
  // listing them beside max health said they were something you wear.
  const active = [];
  if (i.hit_for > 0) {
    // One kind: name it, because "hits for 35" says how hard and nothing
    // about what answers it. Several: show the split, since the total is
    // already printed.
    const k = i.damage_kinds ?? [];
    const kind = k.length === 1 ? ` ${k[0].split(' ')[1]}`
  : k.length > 1 ? ` <span class="dim">(${k.join(' + ')})</span>` : '';
    active.push(`<li>hits for <b>${i.hit_for}</b>${kind}${
  i.dps ? ` <span class="dim">— ${i.dps.toFixed(1)} a second</span>` : ''}</li>`);
  }
  for (const a of i.active) active.push(`<li><b>${a.n}</b> ${a.label}</li>`);
  if (i.casts > 0) {
    active.push(`<li class="dim">costs ${i.cast_cost} of the Funny ${
  i.casts > 1 ? `an activation, whichever of its ${i.casts} spells come up` : 'a cast'}</li>`);
  }
  for (const n of i.notes) active.push(`<li>${n}</li>`);
  if (i.power && i.power !== 100) {
    active.push(`<li class="dim">everything above already carries this item's own ×${
  (i.power / 100).toFixed(2)}</li>`);
  }
  if (!active.length) active.push(`<li class="none">it holds its shape and nothing else</li>`);

  // **The spin, and the line that tells a player to repack.** An item that
  // cannot turn where it stands banks nothing at all, and the card is the only
  // place that can say so — the board shows a footprint that never moves,
  // which reads as a feature that is broken rather than as a board that is
  // full.
  if (i.spins) {
    active.push(i.cycle > 1
      ? `<li>turns every second — <b>+${(i.spin_pct / 100).toFixed(2)}×</b> power a turn, ` +
        `spent when it goes off <span class="dim">(${i.cycle} ways round)</span></li>`
      : `<li class="none">it would turn every second, and there is no room where it is</li>`);
  }

  // **What it does to them.** A third half, and the one that was missing:
  // fifty-nine components in the catalogue apply a curse and this card had no
  // arm for one, so a Greave Mold's whole point never reached the screen that
  // exists to explain it. Core's sentence, which names who it lands on — a
  // piece that curses its own wearer reads as the downside it is.
  const curses = (i.curses ?? []).length
    ? `<span class="head">curses</span><ul class="stats curses">` +
      i.curses.map((c) => `<li>${c}</li>`).join('') + `</ul>`
    : '';

  // **What a set does that no stat can.** Only on a whole set — core answers
  // that, and the card and the rule read the same answer. Unthemed and with
  // the number in it, the same register the sheet prints a rule in: the name
  // above carries the world and this carries the rule.
  const rules = (i.rules ?? []).length
    ? `<span class="head">the whole set</span><ul class="stats set-rules">` +
      i.rules.map((r) => `<li title="${(r.detail ?? []).join(' ')}">${r.line}</li>`).join('') +
      `</ul>`
    : '';

  return (`
    <div class="made-item${i.set ? ' is-set' : ''}" data-key="${i.pieces.join(',')}">
  <b>${i.name}</b>
  <span class="built">${i.set ? 'a set · ' : ''}${where} · built on ${article(i.core)}</span>
  <span class="rank"><span class="pips">${pips}</span> ${i.rarity.toUpperCase()}
    · rating ${i.rating}${next}</span>
  <span class="head">standing still</span>
  <ul class="stats">${passive}</ul>
  <span class="head">every activation — one every ${secs(i.cooldown_ms ?? 0)}s</span>
  <ul class="stats">${active.join('')}</ul>
  ${curses}
  ${rules}
    </div>`);
}

/// Every item on a set of grids, as cards.
function cards(slots) {
  const parts = [];
  let any = false;
  for (const slot of slots) {
    if (!slot.items.length) continue;
    any = true;
    parts.push(`<p class="grid-of">${slot.slot}</p>`);
    for (const i of slot.items) parts.push(oneCard(i, slot.slot));
  }
  return { html: parts.join(''), any };
}

function boardSays(text) {
  const el = $('board-says');
  el.textContent = text;
  el.hidden = !text;
  el.classList.add('bad');
  clearTimeout(boardSays.t);
  boardSays.t = setTimeout(() => { el.hidden = true; }, 2600);
  // **A refusal about the *board* stays on the board; a refusal about the
  // *game* goes in the log.** "Does not fit there" is the first kind and there
  // are twenty of them a minute. The weapon grid refusing an instrument beside
  // a blade is the second: it is a rule about what you may be, it happens
  // twice in a playthrough, and a player who reads it two seconds later on a
  // strip that has since cleared has learned nothing.
  if (/instrument|grid/.test(text || '')) log(text, true);
}

// ---------------------------------------------------------------- the fork

// Offered the moment the level lands, and offered again on every load until it
// is answered — a save made before five is still asked, and so is one made at
// nine by somebody who closed the tab.
function offerClass() {
  const o = JSON.parse(class_offer_json());
  if (!o) return false;
  const box = $('fork-choices');
  box.replaceChildren();
  for (const c of o.classes) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares';
    // The spec first and the flavour after it. `promise` is the engine's own
    // sentence, unthemed and with the numbers in it — this is the screen where
    // somebody picks a class for the rest of the run, and "Fury, and something
    // heavy to spend it on" is not a thing anybody can decide on.
    // The figure first. This is the one irreversible choice in the game and
    // three walls of text is not a choice anybody makes with any pleasure.
    const art = figure('classes', c.canonical);
    b.innerHTML = (art ? `<img class="portrait wide" src="${art}" alt="">` : '') +
                  `<b>${c.name}</b>` +
                  // `promise`, not `short`: this choice does not come off, and
                  // the compact version only repeated the first clause of it.
                  `<span class="promise">${c.promise}</span>` +
                  `<span class="flavour">${c.blurb}</span>` +
                  `<span class="meta">${c.nodes} skills of its own to spend points on</span>`;
    b.onclick = () => {
      const why = choose_class(c.canonical);
      if (why) {
        const el = $('fork-says');
        el.textContent = why; el.hidden = false; el.classList.add('bad');
        return;
      }
      $('fork').hidden = true;
      paintYou(c.canonical);
      paintPanel(); draw(); autosave();
      openTree();
    };
    box.appendChild(b);
  }
  $('fork').hidden = false;
  return true;
}

// ---------------------------------------------------------------- the tree

function openTree() {
  const c = JSON.parse(character_json());
  $('tree-level').textContent = c.level;
  $('tree-points').textContent = c.points;
  $('tree-next').textContent = c.next_grows ?? '—';
  paintTree();
  $('tree').hidden = false;
}

/// Which tree is on screen. Kept across repaints so taking a node does not
/// throw you back to the first tab.
let openTreeId = null;

function paintTree() {
  const all = JSON.parse(all_trees_json());
  $('tree-points').textContent = all.points;
  const trees = all.trees;
  if (!trees.some((t) => t.id === openTreeId)) openTreeId = trees[0]?.id ?? null;

  // --- the tabs ------------------------------------------------------------
  const tabs = $('tree-tabs');
  tabs.replaceChildren();
  tabs.hidden = trees.length < 2;
  for (const t of trees) {
    const b = document.createElement('button');
    b.type = 'button';
    b.setAttribute('role', 'tab');
    b.className = t.id === openTreeId ? 'on' : '';
    b.setAttribute('aria-selected', String(t.id === openTreeId));
    b.dataset.tree = t.id;
    const left = t.nodes.filter((n) => !n.taken).length;
    b.innerHTML = `${t.name}<span class="meta">${left} left</span>`;
    b.onclick = () => { openTreeId = t.id; hideNode(); paintTree(); };
    tabs.appendChild(b);
  }

  const tree = trees.find((t) => t.id === openTreeId);
  $('tree-which').textContent = tree?.name ?? '';
  const box = $('nodes');
  box.replaceChildren();
  if (!tree) return;

  // --- the tree ------------------------------------------------------------
  //
  // **Rows are depth, and depth is core's answer.** A node with nothing to
  // take first is on the top row; everything else sits below the deepest thing
  // it asks for. It was one flat rack of buttons before, which told you what
  // existed and nothing about what led to what.
  //
  // Ordered within a row by the average position of its parents, which is the
  // cheapest thing that stops the lines crossing: a node sits over the things
  // that need it.
  const byDepth = [];
  for (const n of tree.nodes) (byDepth[n.depth] ??= []).push(n);
  const at = new Map();
  byDepth.forEach((row, d) => {
    if (d > 0) {
      row.sort((a, b) => mean(a) - mean(b));
    }
    row.forEach((n, i) => at.set(n.id, i));
  });
  function mean(n) {
    const ps = n.requires.map((r) => at.get(r)).filter((v) => v !== undefined);
    return ps.length ? ps.reduce((a, b) => a + b, 0) / ps.length : 99;
  }

  // The lines go under the buttons, in one SVG sized to the whole tree.
  const wires = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  wires.setAttribute('class', 'wires');
  box.appendChild(wires);

  const el = new Map();
  byDepth.forEach((row, d) => {
    const line = document.createElement('div');
    line.className = 'tier';
    line.dataset.depth = d;
    for (const n of row) line.appendChild(nodeButton(n, tree));
    box.appendChild(line);
    for (const n of row) el.set(n.id, line.querySelector(`[data-node="${n.id}"]`));
  });

  drawWires(box, wires, tree, el);
  // The rows reflow with the window, so the lines have to be redrawn with it.
  paintTree.onresize ??= () => { if (!$('tree').hidden) paintTree(); };
  window.removeEventListener('resize', paintTree.onresize);
  window.addEventListener('resize', paintTree.onresize);
}

/// One node, as a button. Unchanged in what it says — the name is the world's
/// and the line under it is the engine's — only where it sits is new.
function nodeButton(n, tree) {
  const b = document.createElement('button');
  b.type = 'button';
  b.dataset.node = n.id;
  b.className = 'wares node' + (n.taken ? ' pinned' : '') + (n.takeable ? ' open' : '');
  b.disabled = n.taken || !n.takeable;
  const foot = n.taken ? 'taken'
    : n.takeable ? `${n.cost} point${n.cost > 1 ? 's' : ''}`
    : n.why;
  b.innerHTML = `<b>${n.name}</b>` +
                `<span class="spec">${n.effect}</span>` +
                `<span class="cost">${foot}</span>`;
  const detail = () => hoverNode(b, n);
  b.onpointerenter = detail;
  b.onfocus = detail;
  b.onpointerleave = hideNode;
  b.onblur = hideNode;
  b.onclick = () => {
    const why = take_skill(n.id);
    treeSays(why, !!why);
    hideNode();
    // A node can change what the map is allowed to say, and `chances` rides
    // in `world_json` — so the map is re-read rather than merely redrawn.
    world = JSON.parse(world_json());
    paintTree(); paintPanel(); draw(); autosave();
    const c = JSON.parse(character_json());
    $('tree-level').textContent = c.level;
    $('tree-next').textContent = c.next_grows ?? '—';
  };
  return b;
}

/// A line from each prerequisite to the node that wants it.
///
/// Measured off the laid-out buttons rather than computed from a grid: the
/// rows are flex and wrap, so where a node actually *is* is the only thing
/// that can be trusted. Drawn as an elbow — down out of the parent, across,
/// down into the child — because a diagonal through three rows of buttons is
/// unreadable and a curve is worse.
function drawWires(box, svg, tree, el) {
  const b = box.getBoundingClientRect();
  svg.setAttribute('width', b.width);
  svg.setAttribute('height', b.height);
  svg.setAttribute('viewBox', `0 0 ${b.width} ${b.height}`);
  svg.replaceChildren();
  for (const n of tree.nodes) {
    const to = el.get(n.id);
    if (!to) continue;
    for (const r of n.requires) {
      const from = el.get(r);
      if (!from) continue;
      const a = from.getBoundingClientRect(), c = to.getBoundingClientRect();
      const x1 = a.left - b.left + a.width / 2, y1 = a.bottom - b.top;
      const x2 = c.left - b.left + c.width / 2, y2 = c.top - b.top;
      const mid = y1 + (y2 - y1) / 2;
      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      path.setAttribute('d', `M ${x1} ${y1} V ${mid} H ${x2} V ${y2}`);
      // A wire into something already taken is spent; one into something you
      // could take now is lit. The rest is scaffolding.
      const cls = tree.nodes.find((m) => m.id === n.id)?.taken ? 'done'
        : el.get(r).classList.contains('pinned') ? 'live' : '';
      path.setAttribute('class', cls);
      svg.appendChild(path);
    }
  }
}

/// What a node means, for somebody who has not read the source.
///
/// The button carries the arithmetic; this carries the definitions behind it —
/// what mind resistance actually resists, what an assembly bonus actually is —
/// and, last and in italics, the sentence about the mine.
function hoverNode(button, n) {
  const box = $('node-detail');
  box.innerHTML =
    `<b>${n.name}</b>` +
    `<p class="spec">${n.effect}</p>` +
    (n.detail ?? []).map((d) => `<p>${d}</p>`).join('') +
    `<p class="flavour">${n.blurb}</p>` +
    (n.taken ? '' : `<p class="cost">${n.cost} point${n.cost > 1 ? 's' : ''}${
      n.takeable ? '' : ` — ${n.why}`}</p>`);
  box.hidden = false;
  // Pinned to the button, then nudged back inside the viewport. Fixed rather
  // than absolute because the tree scrolls and the card must not scroll with
  // the row it is describing.
  const r = button.getBoundingClientRect();
  const w = box.offsetWidth, h = box.offsetHeight;
  const left = Math.min(Math.max(8, r.left), window.innerWidth - w - 8);
  const top = r.bottom + 8 + h > window.innerHeight ? Math.max(8, r.top - h - 8) : r.bottom + 8;
  box.style.left = `${left}px`;
  box.style.top = `${top}px`;
}

function hideNode() {
  $('node-detail').hidden = true;
}

function treeSays(text, bad = false) {
  const el = $('tree-says');
  el.textContent = text; el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function closeTree() {
  hideNode();
  $('tree').hidden = true;
  paintPanel(); draw(); $('map').focus();
}

// ------------------------------------------------------------- the ending

/// The one screen that is not a loop.
///
/// It does not take the fork's treatment — you *can* back out of it, because
/// the world is still there behind you and there is an errand about the door
/// to hand in. What it must not do is pretend there is more.
function openEnding(e) {
  $('ending-name').textContent = e.name || 'the door in the wall';
  $('ending-prose').replaceChildren(...(e.prose ?? []).map((t) => {
    const p = document.createElement('p');
    p.textContent = t;
    return p;
  }));
  $('ending').hidden = false;
}

function closeEnding() {
  $('ending').hidden = true;
  paintPanel(); draw(); autosave();
  $('map').focus();
}

// ---------------------------------------------------------------- the log

/// Everything you said you would do, and where it wants you.
///
/// **A different question from the town's board.** That one is "what does this
/// place want", which is a property of where you are standing; this is "what am
/// I carrying", which follows you around. Two questions, two calls.
function openLog() {
  paintLog();
  $('log').hidden = false;
}

function paintLog() {
  const all = JSON.parse(quest_log_json());
  const live = all.errands.filter((q) => q.stage !== 'done');
  $('log-carrying').textContent = live.length;
  $('log-finished').textContent = all.errands.length - live.length;
  const box = $('log-list');
  box.replaceChildren();
  if (!all.errands.length) {
    const p = document.createElement('p');
    p.className = 'note';
    p.textContent = 'Nothing yet. Towns and the people standing about in fields both ask.';
    box.appendChild(p);
    return;
  }
  // Live first, finished after: a log is a list of what is still owed, with a
  // record of what is not underneath it.
  for (const q of [...live, ...all.errands.filter((x) => x.stage === 'done')]) {
    const done = q.stage === 'done';
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares errand' + (done ? ' sold' : '') + (q.pinned ? ' pin' : '');
    b.dataset.errand = q.id;
    b.disabled = done;
    const foot = done ? 'finished'
      : q.pinned ? 'pinned — click to unpin'
      : q.on_this_map ? 'pin it to the map'
      : 'not on this map';
    // **Where it points, and whether you can get there.** Core's sentence. A
    // log that pointed a level-one player north past a crossing and said
    // nothing about it reads as a log that is wrong rather than a road that is
    // shut — which is what the M9.4 playthrough did for nine thousand steps.
    const shut = !done && q.shut ? `<span class="why">${q.shut}</span>` : '';
    b.innerHTML = `<b>${q.name}</b>` +
      `<span class="spec">${q.asks}</span>` +
      `<span class="flavour">${q.brief}</span>` +
      `<span class="meta">${q.where} · pays ${q.pays.join(', ')}</span>` +
      shut +
      `<span class="cost">${foot}</span>`;
    // Hover, and the map answers — before anything is committed to. The pin is
    // what makes the answer outlive the screen.
    const show = () => { hoverGuide = JSON.parse(guide_json(q.id)); startPulse(); draw(); };
    const drop = () => { hoverGuide = null; draw(); };
    b.onpointerenter = show;
    b.onfocus = show;
    b.onpointerleave = drop;
    b.onblur = drop;
    b.onclick = () => {
      const why = pin_quest(q.id);
      logSays(why, !!why);
      hoverGuide = null;
      refreshPin();
      paintLog();
      draw();
      autosave();
    };
    box.appendChild(b);
  }
}

function logSays(text, bad = false) {
  const el = $('log-says');
  el.textContent = text; el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function closeLog() {
  hoverGuide = null;
  $('log').hidden = true;
  paintPanel(); draw(); $('map').focus();
}

// ---------------------------------------------------------------- the town

function portrait(el, src, alt) {
  if (!el) return;
  if (src) { el.src = src; el.alt = alt; el.hidden = false; }
  else { el.hidden = true; el.removeAttribute('src'); }
}

function openTown(id) {
  const place = world.places.find((p) => p.id === id);
  $('town-name').textContent = place?.name ?? id;
  portrait($('town-art'), figure('places', id), place?.name ?? id);
  paintShelf();
  paintQuests();
  paintTins();
  paintCarrying();
  $('town').hidden = false;
}

function paintShelf() {
  const s = JSON.parse(shop_json());
  $('town-gold').textContent = s.gold;
  const box = $('shelf');
  box.replaceChildren();
  if (!s.shelf.length) {
    const p = document.createElement('p');
    p.className = 'note';
    p.textContent = 'Nothing for sale here.';
    box.appendChild(p);
    return;
  }
  for (const w of s.shelf) {
    const b = document.createElement('button');
    b.type = 'button';
    // A sold entry stays where it was and greys out. Dropping it would
    // renumber the shelf, and an index is what a save records — but it is also
    // just how a shop reads: the gap is the memory of what you took.
    b.className = 'wares' + (w.sold ? ' sold' : '');
    b.disabled = !w.afford;
    // **The shape, because a component is a shape.** Two blades at one price
    // are not the same purchase when one is four cells in a line and the other
    // is a cross, and the shelf used to say the price and not the shape.
    b.innerHTML = `<span class="ware-top"></span><b>${w.name}</b>` +
      `<span class="meta">${w.for} · ${w.kind} · rates ${w.rating}</span>` +
      `<span class="cost">${w.sold ? 'yours' : `${w.price} Fnorp`}</span>`;
    b.querySelector('.ware-top').appendChild(shapeCanvas(w));
    const read = () => showPiece(b, w);
    b.onpointerenter = read;
    b.onfocus = read;
    b.onpointerleave = hidePiece;
    b.onblur = hidePiece;
    b.onclick = () => {
      const why = buy(w.slot);
      townSays(why || `Bought ${w.name}.`, !!why);
      paintShelf(); paintPanel(); autosave();
    };
    box.appendChild(b);
  }
}

/// What the town wants, which is the other half of what a town is for.
///
/// An errand states what it asks for in the engine's words and a number — the
/// same rule the skill tree follows — and says it in the world's words in the
/// brief above it. The two registers do not mix.
/// The tins a town sells. Every town sells them: a place that had run out of
/// the only thing that undoes tiredness would be a place you could strand
/// yourself at.
function paintTins() {
  const s = JSON.parse(shop_json());
  const box = $('tins');
  box.replaceChildren();
  for (const t of s.supplies ?? []) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares';
    b.disabled = !t.afford;
    b.innerHTML = `<b>${t.name}</b>` +
      `<span class="spec">takes off ${t.restores}% of the tiredness</span>` +
      `<span class="flavour">${t.blurb}</span>` +
      `<span class="cost">${t.price} Fnorp${t.have ? ` · ${t.have} in the pack` : ''}</span>`;
    b.onclick = () => {
      const why = buy_supply(t.id);
      townSays(why || `Bought ${t.name}.`, !!why);
      paintTins(); paintShelf(); paintPanel(); autosave();
    };
    box.appendChild(b);
  }
}

/// What the van has on the table.
///
/// **No town sells an ench.** Every trading town kept a bench until M10, which
/// made an ench a thing you bought rather than a thing you went and got. What a
/// skill tree does not award is sold here, by one man, on one tile, who is not
/// there below level ten.
///
/// Sold entries stay on the table, greyed — the town's shelf rule, and the gap
/// is the memory of what you took.
function paintVendor() {
  const v = JSON.parse(bench_json());
  if (!v) return;
  $('vendor-name').textContent = v.name || v.id;
  $('vendor-gold').textContent = v.gold;
  $('vendor-prose').replaceChildren(...(v.prose ?? []).map((t) => {
    const p = document.createElement('p');
    p.textContent = t;
    return p;
  }));
  const box = $('vendor-stock');
  box.replaceChildren();
  for (const e of v.stock ?? []) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares ench' + (e.sold ? ' sold' : '');
    b.dataset.buyEnch = e.id;
    b.disabled = e.sold || !e.afford;
    b.innerHTML = `<b>${e.name}</b>` +
      `<span class="spec">${e.spec}</span>` +
      `<span class="flavour">${e.blurb}</span>` +
      `<span class="cost">${e.sold ? 'gone' : `${e.price} Fnorp`}` +
      `${e.have ? ` · ${e.have} in the rack` : ''}</span>`;
    b.onclick = () => {
      const why = buy_ench(e.id);
      vendorSays(why || `Bought ${e.name}. It goes on when you pack.`, !!why);
      paintVendor(); paintPanel(); autosave();
    };
    box.appendChild(b);
  }
  // **He will take the money either way.** Being handed an ench and being able
  // to bolt one on are two questions — the rule `quest::hand_in` has followed
  // since M8, and the one the rack was breaking until it was reported. So this
  // says so rather than refusing the sale.
  if (!v.licensed) {
    const p = document.createElement('p');
    p.className = 'note';
    p.textContent = 'He does not ask what you are. Bolting one onto a component is the '
      + 'Kaklon Patent\u2019s, and you are not a licensee — what you buy here goes in the '
      + 'rack and waits.';
    box.appendChild(p);
  }
}

function vendorSays(text, bad) {
  log(text, bad);
  vendorSlot(text, bad);
}

function vendorSlot(text, bad) {
  const el = $('vendor-says');
  el.textContent = text;
  el.hidden = !text;
  el.classList.toggle('bad', !!bad);
}

function openVendor() {
  paintVendor();
  vendorSlot('');
  $('vendor').hidden = false;
}

function closeVendor() {
  $('vendor').hidden = true;
  paintPanel(); draw(); autosave();
  $('map').focus();
}

function paintQuests() {
  paintErrands($('quests'), $('errands'));
}

/// The errands here, wherever here is.
///
/// **One renderer for a counter and a field.** An errand given by a woman on a
/// kitchen chair and one given by a clerk are the same object and the same
/// question; only the room changes.
function paintErrands(box, wrapper) {
  const all = JSON.parse(quests_json());
  if (wrapper) wrapper.hidden = !all.length;
  if (box.id === 'card-errands') box.hidden = !all.length;
  box.replaceChildren();
  for (const q of all) {
    const b = document.createElement('button');
    b.type = 'button';
    const done = q.stage === 'done';
    const locked = q.stage === 'locked';
    b.className = 'wares errand' + (done || locked ? ' sold' : '')
                + (q.stage === 'ready' && q.here_takes ? ' ready' : '');
    // What you can do about it *here*. An errand taken in a field and reported
    // in town is not actionable in the field, and saying so is the whole of
    // what makes "go and tell them" legible.
    // Carrying counts as actionable where it is handed in: clicking says how
    // far along you are, which is information rather than an error, and a
    // button that will not answer that question is a button that has nothing
    // to say about the only thing you want to know.
    const actionable = locked ? false
      : q.stage === 'offered' ? q.here_gives
      : q.stage === 'done' ? false
      : q.here_takes;
    b.disabled = !actionable;
    const foot = locked ? 'something else first'
      : q.stage === 'offered' ? (q.here_gives ? 'Take it on' : `Ask at ${q.giver}`)
      : q.stage === 'ready' ? (q.here_takes ? 'Hand it in' : `Take it back to ${q.back_to}`)
      : `${q.have} of ${q.want}` + (q.here_takes ? '' : ` · back to ${q.back_to}`);
    b.innerHTML = `<b>${q.name}</b>` +
      `<span class="spec">${q.asks}</span>` +
      `<span class="flavour">${q.brief}</span>` +
      `<span class="meta">pays ${q.pays.join(', ')}</span>` +
      `<span class="cost">${foot}</span>`;
    b.onclick = () => {
      const say = box.id === 'card-errands' ? log : townSays;
      if (q.stage === 'offered') {
        const why = take_quest(q.id);
        say(why || `Taken. ${q.asks}.`, !!why);
      } else {
        const r = JSON.parse(hand_in_quest(q.id));
        if (r.error) say(r.error, true);
        else say(`${r.thanks} — ${r.given.join(' and ')}.`);
      }
      paintErrands(box, wrapper);
      if (box.id === 'quests') paintShelf();
      paintPanel(); autosave();
    };
    box.appendChild(b);
  }
}

/// What is in your pocket, and what spending it would buy.
///
/// A town is the only place experience becomes a level, so this is the only
/// place the number means anything but risk.
function paintCarrying() {
  const c = JSON.parse(character_json());
  const n = c.carried ?? 0;
  $('carrying').textContent = n > 0
    ? `${n} experience, and it is only yours once it is spent. ` +
      `You are ${c.needed - c.into} short of level ${c.level + 1}.`
    : 'Nothing. Everything you have won is already spent.';
  $('bank').disabled = n <= 0;
  $('bank').classList.toggle('primary', n > 0);
}

/// What one component is, pinned near whatever is being pointed at.
///
/// The same card wherever a component appears — on a shelf, in the bag, or
/// seated on a grid — because it is the same question.
function showPiece(anchor, p) {
  const box = $('piece-card');
  box.innerHTML = pieceCardHtml(p);
  box.hidden = false;
  const r = anchor instanceof Element ? anchor.getBoundingClientRect() : anchor;
  const w = box.offsetWidth, h = box.offsetHeight;
  const left = Math.min(Math.max(8, r.left), window.innerWidth - w - 8);
  const top = r.bottom + 8 + h > window.innerHeight ? Math.max(8, r.top - h - 8) : r.bottom + 8;
  box.style.left = `${left}px`;
  box.style.top = `${top}px`;
}

function hidePiece() {
  $('piece-card').hidden = true;
}

/// The town's own slot, on the town's own screen — and the log as well.
///
/// A message printed on a screen you walk out of is a message you cannot go
/// back and read. The screen keeps its sentence because that is where you are
/// standing; the log keeps it because the log is the transcript.
function townSays(text, bad = false) {
  log(text, bad);
  townSlot(text, bad);
}

function townSlot(text, bad = false) {
  const el = $('town-says');
  el.textContent = text; el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function closeTown() {
  hidePiece();
  $('town').hidden = true;
  paintPanel(); draw(); autosave();
  $('map').focus();
}

// ---------------------------------------------------------------- walking

function walk(dir) {
  if (!$('card').hidden || !$('fight').hidden || !$('town').hidden ||
      !$('tree').hidden || !$('fork').hidden || !$('log').hidden ||
      !$('history').hidden ||
      !$('ending').hidden || !$('vendor').hidden) return;
  const r = JSON.parse(try_step(dir));
  blocked = r.moved ? null : r.blocked;
  // **A crossing gets the message panel, a cliff gets the flash.** Core says
  // which this was; the page does not read the sentence to work it out. A
  // refusal that is a fact about where the game goes next is worth more than
  // one second of 13px text along the bottom of a canvas.
  if (r.crossing) log(r.blocked, true);
  paintPanel(); draw(); autosave();
  if (blocked) setTimeout(() => { blocked = null; draw(); }, 1100);
  // Arriving is the doing: an errand that says "go and talk to them" is
  // finished by standing there, and core says so on the step.
  if ((r.spoke ?? []).length) log(`You have been. ${r.spoke.join(', ')}.`);
  // **Through a gate is a different map.** The ground, the places and the
  // player's own position all changed, so the page reloads the map rather
  // than redrawing the one it had.
  // **The key turned and it is gone.** Said before "you go through", because
  // that is the order it happens in, and said at all because a thing that
  // leaves your bag without a word reads as a bug. Core decides that a key is
  // spent and which one; the page prints the sentence.
  if (r.turned) log(`${r.turned} turns once and is gone. The way stays open.`);
  if (r.went) {
    world = JSON.parse(world_json());
    log(`You go through.`);
    paintPanel(); draw(); autosave();
    // A gate may carry a paragraph, and one does: the door in the western wall
    // is a crossing that happens once. Core says whether this was the once —
    // the page does not keep a list of which doors it has read.
    if ((r.went.prose ?? []).length) {
      showCard((r.went.name || 'the way through').toUpperCase(), r.went.prose, [], null);
    }
  }
  if (r.shut) log(r.shut, true);
  if (r.ending) openEnding(r.ending);
  if (r.mended > 0) log(`Somebody puts a chair out. ${r.mended}% of you comes back.`);
  // **A creature that gave up.** No fight screen and no replay, because there
  // was no fight — core settled the encounter where it stood and handed over
  // the receipt. Printed, never worked out: the page has no idea what a rout
  // pays and must not learn.
  if (r.routed) log(r.routed.receipt.join(' '));
  if (r.town) openTown(r.town);
  else if (r.bench) openVendor();
  else if (r.event) openEvent(r.event);
  else if (r.encounter) openFight();
}

const KEYS = {
  ArrowUp: 'n', ArrowDown: 's', ArrowLeft: 'w', ArrowRight: 'e',
  w: 'n', s: 's', a: 'w', d: 'e',
  W: 'n', S: 's', A: 'w', D: 'e',
};

function download(name, text) {
  const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  const a = document.createElement('a');
  a.href = url; a.download = name; a.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

function stamp() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, '0');
  return `gear-master-2d-${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}` +
         `-${p(d.getHours())}${p(d.getMinutes())}.json`;
}

// GitHub Pages serves index.html with `Cache-Control: max-age=600`, and every
// other asset is content-hashed — so a browser holding a stale index.html keeps
// loading the *old* app.js and the *old* wasm from URLs that will be served
// forever. A fix can be deployed, verified, and still not reach somebody whose
// tab is pinned to the previous entry point. That is how a player stayed stuck
// in the rock after the repair was live.
//
// So the page checks. One fetch of index.html with a cache-busting query, its
// build stamp against the one baked into this file, and a single reload if they
// differ. Guarded by sessionStorage so a genuine mismatch cannot loop.
const BUILD = '__BUILD__';

async function freshEnough() {
  if (sessionStorage.getItem('gm2d.reloaded') === BUILD) return true;
  try {
    const res = await fetch(`./index.html?cb=${Date.now()}`, { cache: 'no-store' });
    const html = await res.text();
    const live = html.match(/app\.js\?v=([a-f0-9]+)/)?.[1];
    if (live && live !== BUILD) {
      sessionStorage.setItem('gm2d.reloaded', live);
      // Navigate to a different URL rather than reloading. `location.reload()`
      // is allowed to re-serve the same cached document, which would land back
      // here and loop; a query the browser has never seen forces a fresh fetch
      // of the entry point, and that is the whole problem being solved.
      location.replace(`${location.pathname}?v=${live}`);
      return false;
    }
  } catch {
    // Offline, or the fetch was blocked. Carry on with what we have: a stale
    // page is better than no page.
  }
  return true;
}

async function main() {
  if (!(await freshEnough())) return;
  try { await init(); } catch (e) {
    $('status').textContent = `the engine did not load: ${e}`;
    $('status').classList.add('bad');
    throw e;
  }

  world = JSON.parse(world_json());
  try {
    art = await (await fetch('data/art.json')).json();
    paintYou();
  } catch {
    // No art file, or it will not parse. The game draws headings instead,
    // which is exactly what it did before there was any art at all.
  }

  let restored = false;
  try {
    const saved = localStorage.getItem(AUTOSAVE);
    if (saved) { load_json(saved); restored = true; }
  } catch { localStorage.removeItem(AUTOSAVE); }
  if (!restored) new_game(Date.now());

  addEventListener('keydown', (e) => {
    if (!$('fight').hidden) {
      if (e.key === 'r' || e.key === 'R') { e.preventDefault(); board.rotateHeld(); }
      return;
    }
    if (!$('town').hidden) {
      if (e.key === 'Escape') closeTown();
      return;
    }
    if (!$('tree').hidden) {
      if (e.key === 'Escape') closeTree();
      return;
    }
    if (!$('log').hidden) {
      if (e.key === 'Escape') closeLog();
      return;
    }
    if (!$('history').hidden) {
      if (e.key === 'Escape') closeHistory();
      return;
    }
    if (!$('ending').hidden) {
      if (e.key === 'Escape') closeEnding();
      return;
    }
    // The fork has no way out but through. It is the one screen in the game
    // that does not take Escape, because it is the one decision that does not
    // come off.
    if (!$('fork').hidden) return;
    if (e.key === 'Escape' && !$('card').hidden) { closeCard(); return; }
    // `d` is east on WASD, so the overlay gets its own key and a button.
    if (e.key === '`' && !$('scout').hidden) { e.preventDefault(); toggleScout(); return; }
    const dir = KEYS[e.key];
    if (dir) { e.preventDefault(); walk(dir); }
  });

  $('scout').onclick = toggleScout;
  $('homeward').onclick = () => {
    const r = JSON.parse(go_home());
    if (r.error) { log(r.error, true); return; }
    // The map changed, the purse did not, and a tin is gone. Core said all
    // three; the page prints them.
    world = JSON.parse(world_json());
    paintPanel(); draw(); autosave();
    log(`The gear takes you back. It drinks the ${r.fare} on the way.`);
    if (r.mended > 0) {
      log(`Somebody puts a chair out. ${r.mended}% of you comes back.`);
    }
  };
  $('history-open').onclick = openHistory;
  $('history-close').onclick = closeHistory;
  paintTape();

  board = new Board($('board'), {
    boardJson: board_json,
    legalAnchors: legal_anchors,
    place, pickUp: pick_up, rotate, toggleLock: toggle_lock,
    look: look_json, lookOver: look_over,
  });
  board.onsay = boardSays;
  // An ench in hand takes the click instead of the board: picking a component
  // up and bolting something to it are two different gestures on one target,
  // and which one is happening is decided by whether anything is in hand.
  board.onclaim = (pieceId) => (holdingEnch ? bolt(pieceId) : false);
  board.onchange = (st) => {
    const made = st.slots.reduce((n, s) => n + s.items.filter((i) => i.assembled).length, 0);
    $('fight-yours').textContent = made;
    $('undo').disabled = !st.undoable;
    paintMade(st);
    paintRack();
  };
  // Scoped to its own panel: both sides draw `.made-item` now, and an
  // unscoped query lit a creature's card when you pointed at your own blade.
  board.onpoint = (key) => lightCard($('panel-yours'), key);
  // And the component itself, wherever the cursor is on it.
  board.onpiece = (p, box, px, py) => {
    if (!p) { hidePiece(); return; }
    // Anchored to the cursor rather than to the canvas: a board is one element
    // and the thing being described is a few cells of it.
    showPiece({ left: box.left + px + 14, right: box.left + px + 14,
                top: box.top + py, bottom: box.top + py + 8 }, p);
  };
  theirs = new Theirs($('theirs-board'));
  theirs.onpoint = (key) => lightCard($('panel-theirs'), key);
  $('tab-yours').onclick = () => showTab('yours');
  $('tab-theirs').onclick = () => showTab('theirs');
  board.onhold = (name) => {
    $('holding').textContent = name ? `carrying ${name} — right-click to turn it` : '';
  };
  replay = new Replay($('replay'), {
    you: $('ticks-you'),
    them: $('ticks-them'),
    // Both boards, drawn read-only by the same painter the creature panel uses.
    boards: { player: new Theirs($('board-you')), enemy: new Theirs($('board-them')) },
  });
  // Pointing at a row on either side reads that item, in the same card the
  // packing panel draws.
  replay.onpoint = (card, slot) => {
    const box = $('tick-card');
    if (!card) { box.hidden = true; return; }
    box.innerHTML = oneCard(card, slot);
    box.hidden = false;
  };
  // Handles for testing/drive.py, which checks that what the board paints green
  // is exactly what core said was legal. Two references rather than one, so the
  // check compares the page's answer against core's rather than against itself.
  window.__board = board;
  window.__legalAnchors = legal_anchors;
  window.__replay = replay;
  window.__classOffer = () => JSON.parse(class_offer_json());
  window.__encounter = () => JSON.parse(encounter_json());
  window.__character = () => JSON.parse(character_json());
  window.__trees = () => JSON.parse(all_trees_json());
  window.__places = () => world.places;
  window.__world = () => world;
  window.__fightJson = () => fight_json();
  window.__save = () => save_json();
  window.__errandMarks = () => JSON.parse(errand_marks_json()).places;
  window.__log = () => JSON.parse(quest_log_json());
  window.__guide = (id) => JSON.parse(guide_json(id));
  window.__hoverGuide = () => hoverGuide;
  window.__position = () => position();
  window.__rack = () => ench_rack_json();
  // Stand somewhere, so a planted check can walk one step into a place rather
  // than a hundred steps to it. Not a cheat in shipped play: nothing calls it.
  //
  // **Two of them, because there are four maps.** `__standAt` puts you on the
  // *first* map — it always has, by writing an empty `map`, which was invisible
  // while there was one — and every check written before M11 means that.
  // `__standHere` keeps the map you are on, which is what a check about the
  // Treyway or the field wants. Naming them apart rather than adding an
  // argument, because the argument would have a default and the default is
  // exactly the thing that was silent.
  const standOn = (at, map) => {
    const save = JSON.parse(save_json());
    save.state.world.at = at;
    if (map !== null) save.state.world.map = map;
    load_json(JSON.stringify(save));
    world = JSON.parse(world_json());
    paintPanel(); draw();
  };
  window.__standAt = (at) => standOn(at, '');
  window.__standHere = (at) => standOn(at, null);
  // Layout is the one claim reading the source cannot settle, so the gate has
  // to be able to put the fight screen on each stage and measure it.
  window.__stage = (which) => stage(which);
  window.__save = () => save_json();
  window.__errandMarks = () => JSON.parse(errand_marks_json()).places;

  $('skills').onclick = openTree;
  $('tree-done').onclick = closeTree;
  $('vendor-close').onclick = closeVendor;
  $('errands-open').onclick = openLog;
  $('log-close').onclick = closeLog;
  $('ending-close').onclick = closeEnding;

  $('bank').onclick = () => {
    const r = JSON.parse(bank_xp());
    if (r.error) { townSays(r.error, true); return; }
    // Separated, because banking a full pocket crosses several levels at
              // once and four rows in a row read as one sentence otherwise.
    townSays(r.receipt.join('  ·  '), false);
    // **Re-read the map, not just repaint it.** A place can be hidden until a
    // level and a level lands *here* — so without this the van is on the road,
    // in the save, steppable, and invisible until you happen to walk through a
    // gate or reload. `PLAN-M10.md` named this as the easy thing to miss and it
    // was right: the map is drawn from `world`, which is fetched, and banking
    // was the one moment nothing re-fetched it.
    world = JSON.parse(world_json());
    paintCarrying(); paintPanel(); draw(); autosave();
    // A level lands here now, so the fork is offered here.
    offerClass();
  };
  $('leave').onclick = closeTown;
  // Packing in town: the same board, with the fight buttons swapped for a way
  // back out. A player who cannot re-pack between fights is a player who
  // bought a component they cannot use.
  $('pack').onclick = () => {
    $('town').hidden = true;
    $('fight-rank').textContent = 'in town';
    $('fight-name').textContent = 'Your frames';
    $('fight-note').textContent = 'Nothing is waiting. Pack, then go back out.';
    $('fight-rating').textContent = '—';
    $('fight-bounty').textContent = '—';
    // Nothing is waiting, so nothing is pictured. The screen is shared with
    // encounters and was keeping the last creature's portrait up while you
    // packed in a town.
    portrait($('fight-art'), null, '');
    packingOnly = true;
    $('run').textContent = 'Done';
    $('fight').hidden = false;
    stage('board');
    board.refresh();
  };

  // A player standing in front of a creature can save. The encounter is state,
  // so the file they get reopens onto this fight rather than onto the map.
  $('fight-save').onclick = () => {
    const n = stamp();
    download(n, save_json());
    boardSays(`Saved ${n}.`);
    $('board-says').classList.remove('bad');
  };

  $('go').onclick = runFight;
  $('undo').onclick = () => { undo(); board.refresh(); };
  $('preset').onclick = () => { apply_preset(); board.refresh(); };
  $('clear').onclick = () => { clear_board(); board.refresh(); };
  $('run').onclick = () => {
    if (packingOnly) { packingOnly = false; $('run').textContent = 'Walk away'; }
    else flee();
    closeFight();
  };
  $('skip').onclick = () => replay.finish();
  $('done').onclick = closeFight;

  $('card-close').onclick = closeCard;
  $('map').onclick = () => $('map').focus();

  $('download').onclick = () => {
    const name = stamp();
    download(name, save_json());
    log(`Saved ${name}.`);
  };
  $('reset').onclick = () => {
    new_game(Date.now());
    closeCard();
    world = JSON.parse(world_json());
    paintPanel(); draw(); autosave(); log('New game.');
  };
  $('file').onchange = async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    try {
      load_json(new TextDecoder().decode(await f.arrayBuffer()));
      closeCard(); $('fight').hidden = true;
      // **Re-read the map, not just repaint it.** A save carries which map it
      // was on and what the character may read of it, and both ride in
      // `world_json` — a loaded file was drawing the map the page happened to
      // start with. Only a step through a gate used to refresh this, so a save
      // taken in the cave opened onto the overworld's ground until you moved.
      world = JSON.parse(world_json());
      paintPanel(); draw(); autosave();
      log(`Loaded ${f.name}.`);
      if (JSON.parse(encounter_json())) openFight();
      else offerClass();
    } catch (err) {
      log(String(err?.message ?? err), true);
    }
    e.target.value = '';
  };

  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', draw);

  paintPanel();
  draw();
  // A save taken mid-fight comes back mid-fight. The creature is in the file,
  // so there is one waiting whether or not this page has seen it before.
  if (JSON.parse(encounter_json())) openFight();
  else if (!offerClass()) $('map').focus();
  $('status').textContent =
    `core: ${piece_count()} pieces · v${version()} · save v${save_version()}`;
}

main();
