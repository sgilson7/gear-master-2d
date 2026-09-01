/// The creature's board, drawn but not touched.
///
/// The same cells the player packs, in the same hues and the same marks — a
/// creature seats gear through the identical pipeline, and until this existed
/// the only thing the fight screen could say about six items of somebody
/// else's gear was a list of names.
///
/// Read-only on purpose. There is no `held`, no legality, no hover-to-seat:
/// everything that makes `board.js` big is about *changing* a board, and none
/// of it is wanted here.
import { paintMotif } from './board.js';

const CELL = 20;
const SHAKE_PX = 3.2;   // how far a component moves on the tick it fires
const GAP = 16;          // between one grid and the next
const LABEL = 19;        // room above a grid for its name
const INK = '#e8e8f0';
const INK3 = '#8f8fa4';
const LINE = '#2a2a38';
const GROUND = '#101018';

/// Lay the grids that have anything in them across the canvas.
///
/// Returns the item key under the cursor when asked, so pointing at a cell
/// lights the matching card — the same affordance the player's board has.
export class Theirs {
  constructor(canvas) {
    this.c = canvas;
    this.g = canvas.getContext('2d');
    this.slots = [];
    this.pointed = null;
    this.onpoint = null;
    /// `key -> [cells]` for anything currently going off, with how far through
    /// its shake it is. Set by whoever is driving; this file only draws it.
    this.shaking = [];
    canvas.addEventListener('pointermove', (e) => this.point(e));
    canvas.addEventListener('pointerleave', () => this.point(null));
  }

  /// Which item each cell belongs to, so a hover can name one.
  ///
  /// Built from the item's own cell list rather than from the pieces, because
  /// an item *is* its set of pieces here and that is what the card is keyed by.
  load(slots) {
    this.slots = (slots ?? []).filter((s) => s.placed.length);
    this.layout();
    this.draw();
  }

  layout() {
    let x = 0;
    this.boxes = this.slots.map((s) => {
      const box = { slot: s, x, y: LABEL, w: s.cols * CELL, h: s.rows * CELL };
      x += box.w + GAP;
      return box;
    });
    const w = Math.max(x - GAP, 1);
    const h = LABEL + Math.max(...this.slots.map((s) => s.rows), 1) * CELL;
    // Size the backing store to the box, the same rule the main board follows:
    // a canvas is a replaced element, so leaving height to CSS re-derives it
    // from the intrinsic aspect ratio and the drawing lands at the wrong scale.
    const dpr = window.devicePixelRatio || 1;
    this.c.width = Math.round(w * dpr);
    this.c.height = Math.round(h * dpr);
    this.c.style.width = `${w}px`;
    this.c.style.height = `${h}px`;
    this.g.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.w = w; this.h = h;
  }

  draw() {
    const g = this.g;
    g.clearRect(0, 0, this.w, this.h);
    if (!this.slots.length) return;
    g.fillStyle = GROUND;
    g.fillRect(0, 0, this.w, this.h);

    for (const box of this.boxes) {
      const s = box.slot;
      g.fillStyle = INK3;
      g.font = '11px ui-monospace, SFMono-Regular, Menlo, monospace';
      g.textBaseline = 'alphabetic';
      g.fillText(s.slot, box.x, LABEL - 4);

      // Empty cells first, so a component laid over them has no seam.
      g.strokeStyle = LINE;
      g.lineWidth = 1;
      for (let y = 0; y < s.rows; y++) {
        for (let x = 0; x < s.cols; x++) {
          g.strokeRect(box.x + x * CELL + 0.5, box.y + y * CELL + 0.5, CELL - 1, CELL - 1);
        }
      }
      for (const p of s.placed) {
        // **The jolt an item gives when it goes off.** A decaying wobble
        // rather than a flash: a fight is five or six items on two boards all
        // coming round at their own rates, and colour would be five things
        // changing at once. Movement reads as "that one, now" and nothing else
        // on the board moves.
        const [ox, oy] = this.jolt(p.cells);
        for (const [cx, cy] of p.cells) {
          const px = box.x + cx * CELL + ox, py = box.y + cy * CELL + oy;
          g.fillStyle = p.fill;
          g.fillRect(px, py, CELL, CELL);
          paintMotif(g, px, py, CELL, p.motif, p.ink, p.ink_alpha);
        }
      }
      // Each assembled item's outline, in the same white the player's board
      // uses for "this came together".
      for (const i of s.items) {
        if (!i.assembled) continue;
        this.edge(i.cells, box, i.pieces.join(',') === this.pointed ? '#69cdeb' : '#ffffff');
      }
    }
  }

  /// How far to push a component this frame, if its item just fired.
  ///
  /// Amplitude decays over the shake so it settles rather than stopping dead,
  /// and the phase is driven off the cell so two items firing together do not
  /// move in lockstep.
  jolt(cells) {
    if (!this.shaking.length) return [0, 0];
    const key = `${cells[0]?.[0]},${cells[0]?.[1]}`;
    for (const sh of this.shaking) {
      if (!sh.cells.some(([x, y]) => `${x},${y}` === key)) continue;
      const a = SHAKE_PX * (1 - sh.at);
      const t = sh.at * Math.PI * 6;
      return [Math.sin(t) * a, Math.cos(t * 1.3) * a * 0.5];
    }
    return [0, 0];
  }

  edge(cells, box, colour) {
    const g = this.g;
    const own = new Set(cells.map(([x, y]) => `${x},${y}`));
    g.save();
    g.strokeStyle = colour;
    g.lineWidth = colour === '#ffffff' ? 2 : 3;
    g.beginPath();
    for (const [x, y] of cells) {
      const a = box.x + x * CELL, b = box.y + y * CELL;
      if (!own.has(`${x},${y - 1}`)) { g.moveTo(a, b); g.lineTo(a + CELL, b); }
      if (!own.has(`${x},${y + 1}`)) { g.moveTo(a, b + CELL); g.lineTo(a + CELL, b + CELL); }
      if (!own.has(`${x - 1},${y}`)) { g.moveTo(a, b); g.lineTo(a, b + CELL); }
      if (!own.has(`${x + 1},${y}`)) { g.moveTo(a + CELL, b); g.lineTo(a + CELL, b + CELL); }
    }
    g.stroke();
    g.restore();
  }

  /// The item under the cursor, by the same key the cards carry.
  point(e) {
    let key = null;
    if (e) {
      const r = this.c.getBoundingClientRect();
      const mx = e.clientX - r.left, my = e.clientY - r.top;
      outer: for (const box of this.boxes) {
        const cx = Math.floor((mx - box.x) / CELL), cy = Math.floor((my - box.y) / CELL);
        if (cx < 0 || cy < 0 || cx >= box.slot.cols || cy >= box.slot.rows) continue;
        for (const i of box.slot.items) {
          if (i.cells.some(([x, y]) => x === cx && y === cy)) { key = i.pieces.join(','); break outer; }
        }
      }
    }
    if (key === this.pointed) return;
    this.pointed = key;
    this.draw();
    this.onpoint?.(key);
  }
}
