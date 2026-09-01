// The page. It draws what core says and asks core what happened; it decides
// nothing. Whether a tile is walkable, whether a fight starts, which creature
// it is, and whether a choice can be taken are all answered on the other side
// of the boundary — a page that works any of those out for itself is a second
// copy of the rules that will disagree with the first.
import init, {
  world_json, position, try_step, event_json, answer,
  save_json, load_json, new_game, apply_preset,
  shop_json, buy, reroll, pin,
  character_json, skills_json, take_skill,
  class_offer_json, choose_class, class_name, all_trees_json,
  gold, piece_count, version, save_version,
  board_json, legal_anchors, place, pick_up, rotate, toggle_lock, undo, clear_board,
  look_json, look_over,
  encounter_json, fight_json, settle_fight, flee,
} from './pkg/gm2d_wasm.js';
import { Board } from './board.js';
import { Theirs } from './theirs.js';
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

function draw() {
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

  // Places. A town is a filled square with a ring; an event is a small mark —
  // deliberately not a letter, because a letter on a 32px tile is a letter
  // nobody reads.
  for (const p of world.places) {
    const [x, y] = p.at;
    const cx = x * TILE + TILE / 2;
    const cy = y * TILE + TILE / 2;
    g.strokeStyle = ink();
    g.lineWidth = 2;
    if (p.kind === 'town') {
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

  if (debug) drawDebug(g, pos);

  if (blocked) {
    g.fillStyle = 'rgba(139,66,37,.9)';
    g.fillRect(0, c.height - 26, c.width, 26);
    g.fillStyle = '#f4f5ef';
    g.font = '13px ui-monospace, Menlo, monospace';
    g.fillText(blocked, 10, c.height - 8);
  }
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
      if (name === 'rock' || name === 'water') continue;
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
  const c = JSON.parse(character_json());
  $('level').textContent = c.level;
  $('xp').textContent = `${c.into} / ${c.needed}`;
  $('points').textContent = c.points;
  $('skills').classList.toggle('primary', c.points > 0);
  $('class').textContent = class_name() || '—';
  $('region').textContent = p.region ?? '—';
  $('terrain').textContent = p.terrain;
  $('coords').textContent = `${p.x}, ${p.y}`;
  $('chance').textContent = `${p.chance} / 1000`;
  $('danger').textContent = p.danger ?? '—';
  $('walked').textContent = p.walked;
  $('fights').textContent = p.fights;
  $('gold').textContent = gold();
}

function toggleDebug() {
  debug = !debug;
  $('numbers').textContent = debug ? 'Hide the numbers' : 'Show the numbers';
  $('numbers').setAttribute('aria-pressed', String(debug));
  draw();
}

function says(text, bad = false) {
  const el = $('says');
  el.textContent = text;
  el.hidden = !text;
  el.classList.toggle('bad', bad);
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
  $('card-close').hidden = choices.length > 0;
  $('card').hidden = false;
}

function closeCard() {
  $('card').hidden = true;
  $('map').focus();
}

function openEvent(id) {
  const e = JSON.parse(event_json(id));
  if (e.error) { says(e.error, true); return; }
  showCard(e.title, e.prose, e.choices, (i) => {
    const r = JSON.parse(answer(id, i));
    if (r.error) { says(r.error, true); return; }
    $('card-choices').replaceChildren();
    const box = $('card-receipt');
    box.replaceChildren(...(r.receipt.length ? r.receipt : ['Nothing you could point to'])
      .map((line) => { const p = document.createElement('p'); p.textContent = line; return p; }));
    box.hidden = false;
    $('card-close').hidden = false;
    paintPanel(); draw(); autosave();
  });
}

// ---------------------------------------------------------------- the fight

let board = null;
let replay = null;
let theirs = null;

function stage(which) {
  for (const s of ['board', 'replay', 'result']) $(`stage-${s}`).hidden = s !== which;
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
  const log = JSON.parse(fight_json());
  if (log.error) { boardSays(log.error); return; }
  stage('replay');
  replay.load(log);
  replay.onend = () => {
    const s = JSON.parse(settle_fight());
    stage('result');
    $('result-title').textContent =
      s.outcome === 'victory' ? 'It stops moving' : 'You stop moving';
    $('result-receipt').replaceChildren(...s.receipt.map((line) => {
      const p = document.createElement('p'); p.textContent = line; return p;
    }));
    autosave();
  };
  replay.play();
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
function cards(slots) {
  const parts = [];
  let any = false;

  const sign = (n, unit) => `${n > 0 ? '+' : ''}${n}${unit}`;
  const secs = (ms) => (ms / 1000).toFixed(2);

  for (const slot of slots) {
    if (!slot.items.length) continue;
    any = true;
    parts.push(`<p class="grid-of">${slot.slot}</p>`);
    for (const i of slot.items) {
      if (!i.assembled) {
        parts.push(
          `<div class="made-item short" data-key="${i.pieces.join(',')}">` +
          `<b>${i.short || 'loose pieces'}</b>` +
          `<span class="why">${i.status}</span></div>`);
        continue;
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

      parts.push(`
        <div class="made-item" data-key="${i.pieces.join(',')}">
          <b>${i.name}</b>
          <span class="built">${slot.slot} · built on a ${i.core ?? '—'}</span>
          <span class="rank"><span class="pips">${pips}</span> ${i.rarity.toUpperCase()}
            · rating ${i.rating}${next}</span>
          <span class="head">standing still</span>
          <ul class="stats">${passive}</ul>
          <span class="head">every activation — one every ${secs(i.cooldown_ms ?? 0)}s</span>
          <ul class="stats">${active.join('')}</ul>
        </div>`);
    }
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
    b.innerHTML = `<b>${c.name}</b>` +
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

function paintTree() {
  const all = JSON.parse(all_trees_json());
  $('tree-points').textContent = all.points;
  const box = $('nodes');
  box.replaceChildren();
  const nodes = all.trees.flatMap((t) => t.nodes.map((n) => ({ ...n, tree: t.name })));
  for (const n of nodes) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares' + (n.taken ? ' pinned' : '');
    b.disabled = n.taken || !n.takeable;
    const foot = n.taken ? 'taken'
      : n.takeable ? `${n.cost} point${n.cost > 1 ? 's' : ''}`
      : n.why;
    // **The name is the world's and the line under it is the engine's.**
    // A tree that described itself only in the world's words — "Nine hundred
    // feet of Deep Chocolate mine, and you never once came up early" — is a
    // tree nobody can spend a point in. The blurb still exists; it has just
    // stopped being the only thing on the button.
    b.innerHTML = `<b>${n.name}</b>` +
                  `<span class="spec">${n.effect}</span>` +
                  `<span class="cost">${foot}</span>` +
                  (nodes.some((o) => o.tree !== n.tree)
                    ? `<span class="meta">${n.tree}</span>` : '');
    // The rest — what those words mean, and the flavour — on hover and on
    // focus, so the button stays one line per node and a keyboard reaches it.
    const detail = () => hoverNode(b, n);
    b.onpointerenter = detail;
    b.onfocus = detail;
    b.onpointerleave = hideNode;
    b.onblur = hideNode;
    b.onclick = () => {
      const why = take_skill(n.id);
      treeSays(why, !!why);
      paintTree(); paintPanel(); autosave();
      const c = JSON.parse(character_json());
      $('tree-level').textContent = c.level;
      $('tree-next').textContent = c.next_grows ?? '—';
    };
    box.appendChild(b);
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
    p.textContent = 'Bare. Turn it over.';
    box.appendChild(p);
    return;
  }
  for (const w of s.shelf) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares' + (w.locked ? ' pinned' : '');
    b.disabled = !w.afford;
    b.innerHTML = `<b>${w.name}</b>` +
      `<span class="meta">${w.for} · ${w.kind.toLowerCase()} · rates ${w.rating}</span>` +
      `<span class="cost">${w.price} Fnorp${w.locked ? ' · pinned' : ''}</span>`;
    b.onclick = (e) => {
      if (e.shiftKey) { pin(w.slot); paintShelf(); return; }
      const why = buy(w.slot);
      townSays(why || `Bought ${w.name}.`, !!why);
      paintShelf(); paintPanel(); autosave();
    };
    box.appendChild(b);
  }
}

function townSays(text, bad = false) {
  const el = $('town-says');
  el.textContent = text; el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function closeTown() {
  $('town').hidden = true;
  paintPanel(); draw(); autosave();
  $('map').focus();
}

// ---------------------------------------------------------------- walking

function walk(dir) {
  if (!$('card').hidden || !$('fight').hidden || !$('town').hidden ||
      !$('tree').hidden || !$('fork').hidden) return;
  const r = JSON.parse(try_step(dir));
  blocked = r.moved ? null : r.blocked;
  paintPanel(); draw(); autosave();
  if (blocked) setTimeout(() => { blocked = null; draw(); }, 1100);
  if (r.town) openTown(r.town);
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
    // You, in the panel that is always up. `art.player` has existed since M6
    // and nothing read it.
    if (art.player) portrait($('player-art'), `assets/${art.player}.svg`, 'you');
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
    // The fork has no way out but through. It is the one screen in the game
    // that does not take Escape, because it is the one decision that does not
    // come off.
    if (!$('fork').hidden) return;
    if (e.key === 'Escape' && !$('card').hidden) { closeCard(); return; }
    // `d` is east on WASD, so the overlay gets its own key and a button.
    if (e.key === '`') { e.preventDefault(); toggleDebug(); return; }
    const dir = KEYS[e.key];
    if (dir) { e.preventDefault(); walk(dir); }
  });

  $('numbers').onclick = toggleDebug;

  board = new Board($('board'), {
    boardJson: board_json,
    legalAnchors: legal_anchors,
    place, pickUp: pick_up, rotate, toggleLock: toggle_lock,
    look: look_json, lookOver: look_over,
  });
  board.onsay = boardSays;
  board.onchange = (st) => {
    const made = st.slots.reduce((n, s) => n + s.items.filter((i) => i.assembled).length, 0);
    $('fight-yours').textContent = made;
    $('undo').disabled = !st.undoable;
    paintMade(st);
  };
  // Scoped to its own panel: both sides draw `.made-item` now, and an
  // unscoped query lit a creature's card when you pointed at your own blade.
  board.onpoint = (key) => lightCard($('panel-yours'), key);
  theirs = new Theirs($('theirs-board'));
  theirs.onpoint = (key) => lightCard($('panel-theirs'), key);
  $('tab-yours').onclick = () => showTab('yours');
  $('tab-theirs').onclick = () => showTab('theirs');
  board.onhold = (name) => {
    $('holding').textContent = name ? `carrying ${name} — right-click to turn it` : '';
  };
  replay = new Replay($('replay'));
  // Handles for testing/drive.py, which checks that what the board paints green
  // is exactly what core said was legal. Two references rather than one, so the
  // check compares the page's answer against core's rather than against itself.
  window.__board = board;
  window.__legalAnchors = legal_anchors;
  window.__replay = replay;
  window.__classOffer = () => JSON.parse(class_offer_json());
  window.__encounter = () => JSON.parse(encounter_json());

  $('skills').onclick = openTree;
  $('tree-done').onclick = closeTree;

  $('reroll').onclick = () => { const why = reroll(); townSays(why, !!why); paintShelf(); };
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
    $('go').hidden = true;
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
    if ($('go').hidden) { $('go').hidden = false; $('run').textContent = 'Walk away'; }
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
    says(`Saved ${name}.`);
  };
  $('reset').onclick = () => {
    new_game(Date.now());
    closeCard(); paintPanel(); draw(); autosave(); says('New game.');
  };
  $('file').onchange = async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    try {
      load_json(new TextDecoder().decode(await f.arrayBuffer()));
      closeCard(); $('fight').hidden = true;
      paintPanel(); draw(); autosave();
      says(`Loaded ${f.name}.`);
      if (JSON.parse(encounter_json())) openFight();
      else offerClass();
    } catch (err) {
      says(String(err?.message ?? err), true);
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
