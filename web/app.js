// The page. It draws what core says and asks core what happened; it decides
// nothing. Whether a tile is walkable, whether a fight starts, which creature
// it is, and whether a choice can be taken are all answered on the other side
// of the boundary — a page that works any of those out for itself is a second
// copy of the rules that will disagree with the first.
import init, {
  world_json, position, try_step, event_json, answer, to_last_town,
  save_json, load_json, new_game, apply_preset,
  gold, piece_count, version, save_version,
} from './pkg/gm2d_wasm.js';

const $ = (id) => document.getElementById(id);
const TILE = 32;
const AUTOSAVE = 'gm2d.autosave';

let world = null;
let debug = false;
let blocked = null;   // the last refusal, drawn for one frame
let lastFight = null; // the creature met on the last step

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

// M3 replaces this with the board and the fight. For now it says what would
// happen and who it would happen to, which is what gate 3 asks for.
function openFight(m) {
  lastFight = m;
  showCard(m.name, [
    m.note ? m.note : 'It has seen you.',
    `A fight would happen here. It rates ${m.rating} on the shared scale, which is what the region's danger is the mean of.`,
    'The gear board and the fight itself are M3.',
  ], [], () => {});
  $('card-close').hidden = false;
}

function openTown(id) {
  const place = world.places.find((p) => p.id === id);
  showCard(place?.name ?? id, [
    'Nothing follows you in here, and nothing rolls while you stand in it.',
    'A shop and a rest point are M3, when there is something to spend Fnorp on and something to rest from.',
  ], [], () => {});
  $('card-close').hidden = false;
}

// ---------------------------------------------------------------- walking

function walk(dir) {
  if (!$('card').hidden) return;
  const r = JSON.parse(try_step(dir));
  blocked = r.moved ? null : r.blocked;
  paintPanel(); draw(); autosave();
  if (blocked) setTimeout(() => { blocked = null; draw(); }, 1100);
  if (r.town) openTown(r.town);
  else if (r.event) openEvent(r.event);
  else if (r.encounter) openFight(r.encounter);
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
  if (!restored) { new_game(Date.now()); apply_preset(); }

  addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !$('card').hidden) { closeCard(); return; }
    // `d` is east on WASD, so the overlay gets its own key and a button.
    if (e.key === '`') { e.preventDefault(); toggleDebug(); return; }
    const dir = KEYS[e.key];
    if (dir) { e.preventDefault(); walk(dir); }
  });

  $('numbers').onclick = toggleDebug;

  $('card-close').onclick = closeCard;
  $('map').onclick = () => $('map').focus();

  $('download').onclick = () => {
    const name = stamp();
    download(name, save_json());
    says(`Saved ${name}.`);
  };
  $('reset').onclick = () => {
    new_game(Date.now()); apply_preset();
    closeCard(); paintPanel(); draw(); autosave(); says('New game.');
  };
  $('file').onchange = async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    try {
      load_json(new TextDecoder().decode(await f.arrayBuffer()));
      closeCard(); paintPanel(); draw(); autosave();
      says(`Loaded ${f.name}.`);
    } catch (err) {
      says(String(err?.message ?? err), true);
    }
    e.target.value = '';
  };

  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', draw);

  paintPanel();
  draw();
  $('map').focus();
  $('status').textContent =
    `core: ${piece_count()} pieces · v${version()} · save v${save_version()}`;
}

main();
