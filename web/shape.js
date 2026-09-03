/// A component's outline, small.
///
/// The shop used to show a name, a slot and a price, which says everything
/// about a piece except the thing you are actually buying: **a component is a
/// shape**. Two blades at the same price are not the same purchase when one is
/// four cells in a line and the other is a cross.
///
/// Drawn with `paintMotif` rather than reimplemented, because the mark is the
/// *shape* half of the colourblind triple-encoding and every place that draws
/// a cell has to draw the same one.
import { paintMotif } from './board.js';

/// A canvas showing `cells` at up to `box` pixels a side.
///
/// Sized to the shape, not padded to a square: a one-cell ring next to a
/// twelve-cell base should look like a one-cell ring.
export function shapeCanvas(piece, box = 56, cellMax = 14) {
  const cells = piece.cells ?? [[0, 0]];
  const xs = cells.map(([x]) => x), ys = cells.map(([, y]) => y);
  const x0 = Math.min(...xs), y0 = Math.min(...ys);
  const w = Math.max(...xs) - x0 + 1, h = Math.max(...ys) - y0 + 1;
  const cell = Math.max(3, Math.min(cellMax, Math.floor(box / Math.max(w, h))));

  const c = document.createElement('canvas');
  const dpr = window.devicePixelRatio || 1;
  c.width = Math.round(w * cell * dpr);
  c.height = Math.round(h * cell * dpr);
  c.style.width = `${w * cell}px`;
  c.style.height = `${h * cell}px`;
  c.className = 'shape';
  const g = c.getContext('2d');
  g.setTransform(dpr, 0, 0, dpr, 0, 0);

  // Fill and mark first, edge to edge, so neighbouring cells of one component
  // meet without a seam — a four-cell blade is one blade, not four squares.
  for (const [x, y] of cells) {
    const px = (x - x0) * cell, py = (y - y0) * cell;
    g.fillStyle = piece.fill ?? '#888';
    g.fillRect(px, py, cell, cell);
    paintMotif(g, px, py, cell, piece.motif, piece.ink, piece.ink_alpha);
  }
  // Then the outside edge only. An edge is outside when nothing of this
  // component is across it.
  const own = new Set(cells.map(([x, y]) => `${x},${y}`));
  g.strokeStyle = 'rgba(0,0,0,.75)';
  g.lineWidth = Math.max(1, cell * 0.11);
  g.beginPath();
  for (const [x, y] of cells) {
    const a = (x - x0) * cell, b = (y - y0) * cell;
    if (!own.has(`${x},${y - 1}`)) { g.moveTo(a, b); g.lineTo(a + cell, b); }
    if (!own.has(`${x},${y + 1}`)) { g.moveTo(a, b + cell); g.lineTo(a + cell, b + cell); }
    if (!own.has(`${x - 1},${y}`)) { g.moveTo(a, b); g.lineTo(a, b + cell); }
    if (!own.has(`${x + 1},${y}`)) { g.moveTo(a + cell, b); g.lineTo(a + cell, b + cell); }
  }
  g.stroke();
  return c;
}

/// The hover card for one component: what it is, and what it does.
///
/// `lines` is `explain::piece_lines` — the engine's own words, grouped by when
/// they apply. The name above them is the theme's; these are not, for the same
/// reason a skill node's line is not (TONE.md 13a).
export function pieceCardHtml(p) {
  const GROUP = [
    ['standing', 'standing still'],
    ['activation', 'every activation'],
    ['assembled', 'when its item is finished'],
    // **Which set it belongs to, and what the set does.** M9.4 played the game
    // and picked up all three of the Mandate's components without ever being
    // told they made anything: Auto-pack packs for a rating and a set is only
    // the set, so nothing on any screen said the three of them were three of a
    // thing.
    ['set', 'part of a set'],
    // **And which instrument wants it.** The same lesson one block later: a
    // Map Shard off a tower floor is a two-cell component with three mind
    // damage on it, and without this there is nothing anywhere that says three
    // of them and two handfuls of earth make a golem.
    ['survey', 'part of a survey instrument'],
    ['effect', 'what it does to its neighbours'],
    ['trigger', ''],
  ];
  const lines = p.lines ?? [];
  const parts = [];
  for (const [key, head] of GROUP) {
    const mine = lines.filter((l) => l.where === key);
    if (!mine.length) continue;
    if (head) parts.push(`<span class="head">${head}</span>`);
    parts.push(`<ul class="stats">${
      mine.map((l) => `<li>${l.text}</li>`).join('')}</ul>`);
  }
  if (!parts.length) parts.push(`<ul class="stats"><li class="none">it takes up room, and that is all</li></ul>`);
  // What is bolted to this one. Its own group, because it is not something the
  // component does — it is something somebody did to the component.
  if (p.ench) {
    parts.push(`<span class="head">bolted on${p.ench.active ? '' : ' — switched off'}</span>`);
    parts.push(`<ul class="stats"><li><b>${p.ench.name}</b> — ${p.ench.spec}</li></ul>`);
  }
  const where = (p.slots ?? []).join(' or ');
  // Wrapped in `.made-item`, because that is what the card CSS is written
  // against — the first draft put these spans bare inside `.made` and every
  // one of them stayed inline, so the name, the kind and the first heading ran
  // together into one sentence.
  return `<div class="made-item">` +
    `<b>${p.name}</b>` +
    `<span class="built">${p.kind}${where ? ` · ${where}` : ''}</span>` +
    parts.join('') +
    `</div>`;
}
