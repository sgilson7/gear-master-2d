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
  gold, piece_count, version, save_version,
  board_json, legal_anchors, place, pick_up, rotate, toggle_lock, undo, clear_board,
  encounter_json, fight_json, settle_fight, flee,
} from './pkg/gm2d_wasm.js';
import { Board } from './board.js';
import { Replay } from './replay.js';

const $ = (id) => document.getElementById(id);
const TILE = 32;
const AUTOSAVE = 'gm2d.autosave';

let world = null;
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

function stage(which) {
  for (const s of ['board', 'replay', 'result']) $(`stage-${s}`).hidden = s !== which;
}

function openFight() {
  const m = JSON.parse(encounter_json());
  if (!m) return;
  $('fight-rank').textContent = m.rank === 'ordinary' ? 'an encounter' : m.rank;
  $('fight-name').textContent = m.name;
  $('fight-note').textContent = m.note ?? '';
  $('fight-rating').textContent = m.rating;
  $('fight-bounty').textContent = m.bounty;
  $('fight').hidden = false;
  stage('board');
  board.refresh();
}

function closeFight() {
  $('fight').hidden = true;
  paintPanel(); draw(); autosave();
  $('map').focus();
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

function boardSays(text) {
  const el = $('board-says');
  el.textContent = text;
  el.hidden = !text;
  el.classList.add('bad');
  clearTimeout(boardSays.t);
  boardSays.t = setTimeout(() => { el.hidden = true; }, 2600);
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
  const t = JSON.parse(skills_json());
  $('tree-points').textContent = t.points;
  const box = $('nodes');
  box.replaceChildren();
  for (const n of t.nodes) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'wares' + (n.taken ? ' pinned' : '');
    b.disabled = n.taken || !n.takeable;
    const foot = n.taken ? 'taken'
      : n.takeable ? `${n.cost} point${n.cost > 1 ? 's' : ''}`
      : n.why;
    b.innerHTML = `<b>${n.name}</b><span class="meta">${n.blurb}</span>` +
                  `<span class="cost">${foot}</span>`;
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

function treeSays(text, bad = false) {
  const el = $('tree-says');
  el.textContent = text; el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function closeTree() {
  $('tree').hidden = true;
  paintPanel(); draw(); $('map').focus();
}

// ---------------------------------------------------------------- the town

function openTown(id) {
  const place = world.places.find((p) => p.id === id);
  $('town-name').textContent = place?.name ?? id;
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
  if (!$('card').hidden || !$('fight').hidden || !$('town').hidden || !$('tree').hidden) return;
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

async function main() {
  try { await init(); } catch (e) {
    $('status').textContent = `the engine did not load: ${e}`;
    $('status').classList.add('bad');
    throw e;
  }

  world = JSON.parse(world_json());

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
  });
  board.onsay = boardSays;
  board.onchange = (st) => {
    const made = st.slots.reduce((n, s) => n + s.items.filter((i) => i.assembled).length, 0);
    $('fight-yours').textContent = made;
    $('undo').disabled = !st.undoable;
  };
  replay = new Replay($('replay'));
  // Handles for testing/drive.py, which checks that what the board paints green
  // is exactly what core said was legal. Two references rather than one, so the
  // check compares the page's answer against core's rather than against itself.
  window.__board = board;
  window.__legalAnchors = legal_anchors;

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
  if (JSON.parse(encounter_json())) openFight(); else $('map').focus();
  $('status').textContent =
    `core: ${piece_count()} pieces · v${version()} · save v${save_version()}`;
}

main();
