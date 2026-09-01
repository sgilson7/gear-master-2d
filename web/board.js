// The gear board, in Canvas 2D.
//
// This file draws and hit-tests. It answers no rules and it chooses no colours.
//
// Which cells a component may occupy, which pieces form an item, whether that
// item assembled, what it is called, what it is worth — and, since the board
// was rebuilt against the original's design, **what colour it is and what mark
// it wears** — are all core's, fetched through `board_json`, `legal_anchors`
// and `look_json`. The accessibility contract lives in `core::look` and is held
// by `tests/look.rs`; a page that picked its own colours would be a page with
// its own untested story about colour blindness.
//
// The system it draws, in one line: **slot → a motif and a hue, role →
// brightness.** Any two of those three channels can be lost and the board still
// says which grid a piece belongs to and which part of a recipe it is.

const CELL = 26;
const GAP = 2;
const PAD = 8;

export class Board {
  constructor(canvas, api) {
    this.c = canvas;
    this.api = api;   // { boardJson, legalAnchors, place, pickUp, rotate, toggleLock, look, lookOver }
    this.state = null;
    this.look = JSON.parse(api.look());
    this.held = null;
    this.legal = null;
    this.legalSlot = null;
    this.hover = null;
    this.onchange = () => {};
    this.slotOrder = ['weapon', 'helmet', 'chest', 'gloves', 'greaves'];

    canvas.addEventListener('mousemove', (e) => this.move(e));
    canvas.addEventListener('mouseleave', () => { this.hover = null; this.draw(); });
    canvas.addEventListener('mousedown', (e) => this.press(e));
    canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); this.rotateHeld(); });

    // The assembled outline pulses, so the board has to keep drawing.
    const tick = () => { if (this.state) this.draw(); requestAnimationFrame(tick); };
    requestAnimationFrame(tick);
  }

  refresh() {
    this.state = JSON.parse(this.api.boardJson());
    this.layout();
    this.draw();
    this.onchange(this.state);
  }

  layout() {
    const gw = 6 * (CELL + GAP) + PAD * 2;
    this.boxes = {};
    let x = PAD, y = 30, rowH = 0;
    for (const name of this.slotOrder) {
      const s = this.state.slots.find((s) => s.slot === name);
      const gh = s.rows * (CELL + GAP) + PAD * 2;
      if (x + gw > this.c.width - PAD && x > PAD) { x = PAD; y += rowH + 84; rowH = 0; }
      this.boxes[name] = { x, y, w: gw, h: gh, rows: s.rows };
      rowH = Math.max(rowH, gh);
      x += gw + PAD;
    }
    this.bagY = y + rowH + 84;
  }

  cellAt(px, py) {
    for (const name of this.slotOrder) {
      const b = this.boxes[name];
      const gx = Math.floor((px - b.x - PAD) / (CELL + GAP));
      const gy = Math.floor((py - b.y - PAD) / (CELL + GAP));
      if (gx >= 0 && gx < 6 && gy >= 0 && gy < b.rows &&
          px >= b.x && px <= b.x + b.w && py >= b.y && py <= b.y + b.h) {
        return { slot: name, x: gx, y: gy };
      }
    }
    return null;
  }

  bagAt(px, py) {
    if (py < this.bagY) return null;
    const COL = 168, ROW = CELL + 8;
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / COL));
    const i = Math.floor((py - this.bagY) / ROW) * perRow + Math.floor((px - PAD) / COL);
    return this.state.bag[i] ?? null;
  }

  pieceAt(slot, x, y) {
    const s = this.state.slots.find((s) => s.slot === slot);
    return s.placed.find((p) => p.cells.some(([cx, cy]) => cx === x && cy === y)) ?? null;
  }

  // --------------------------------------------------------------- input

  move(e) {
    const r = this.c.getBoundingClientRect();
    const px = (e.clientX - r.left) * (this.c.width / r.width);
    const py = (e.clientY - r.top) * (this.c.height / r.height);
    this.mouse = { px, py };
    const cell = this.cellAt(px, py);
    const changed = cell?.slot !== this.hover?.slot;
    this.hover = cell;
    if (this.held && cell && (changed || !this.legal)) this.askLegal(cell.slot);
    this.draw();
  }

  // The one question this file is not allowed to answer itself.
  askLegal(slot) {
    if (!this.held) { this.legal = null; return; }
    this.legal = new Set(
      JSON.parse(this.api.legalAnchors(this.held.id, slot)).map(([x, y]) => `${x},${y}`));
    this.legalSlot = slot;
    // And how it would read once it crossed in — grey outside, the grid's
    // colour and mark inside.
    this.held.over = JSON.parse(this.api.lookOver(this.held.id, slot));
  }

  press(e) {
    if (e.button === 2) return;
    const r = this.c.getBoundingClientRect();
    const px = (e.clientX - r.left) * (this.c.width / r.width);
    const py = (e.clientY - r.top) * (this.c.height / r.height);
    const cell = this.cellAt(px, py);

    if (this.held) {
      if (cell) {
        const why = this.api.place(this.held.id, cell.slot, cell.x, cell.y);
        if (why) { this.say(why); return; }
      } else if (this.held.from) {
        this.api.pickUp(this.held.id);
      }
      this.held = null; this.legal = null;
      this.refresh();
      return;
    }

    if (cell) {
      const p = this.pieceAt(cell.slot, cell.x, cell.y);
      if (!p) return;
      if (e.shiftKey) { this.api.toggleLock(p.id); this.refresh(); return; }
      const why = this.api.pickUp(p.id);
      if (why) { this.say(why); return; }
      // Shape relative to its own anchor, so the footprint preview and the
      // cursor draw the piece rather than a single square.
      const anchor = p.cells.reduce((a, b) => (b[1] < a[1] || (b[1] === a[1] && b[0] < a[0]) ? b : a));
      const rel = p.cells.map(([x, y]) => [x - p.x, y - p.y]);
      this.held = { id: p.id, from: cell.slot, name: p.name, slot: cell.slot, cells: rel };
      void anchor;
      this.refresh();
      this.askLegal(cell.slot);
      this.draw();
      return;
    }

    const loose = this.bagAt(px, py);
    if (loose) {
      this.held = {
        id: loose.id, from: null, name: loose.name, slot: loose.slot,
        look: loose, cells: loose.cells,
      };
      this.askLegal(loose.slot);
      this.draw();
    }
  }

  rotateHeld() {
    const id = this.held?.id
      ?? (this.hover && this.pieceAt(this.hover.slot, this.hover.x, this.hover.y)?.id);
    if (id === undefined || id === null) return;
    const why = this.api.rotate(id);
    if (why) this.say(why);
    this.refresh();
    if (this.held) this.askLegal(this.legalSlot ?? this.held.slot);
    this.draw();
  }

  say(text) { this.onsay?.(text); }

  // --------------------------------------------------------------- drawing

  /// The board draws on its own dark ground, so its text is the game's ink
  /// rather than the page's — which flips with the viewer's theme and would
  /// leave dark labels on a near-black panel half the time.
  chrome() {
    return { ink: '#e8e8f0', ink3: '#8f8fa4', line: '#2a2a38' };
  }

  /// One cell: the fill, then the slot's mark over it.
  ///
  /// No inset and no per-cell border — neighbouring cells of the same component
  /// have to meet without a seam, or a four-cell blade reads as four squares.
  cellFill(g, px, py, look) {
    g.fillStyle = look.fill;
    g.fillRect(px, py, CELL, CELL);
    this.motif(g, px, py, CELL, look.motif, look.ink, look.ink_alpha);
  }

  /// The mark, at the fractions the original uses so it reads at any size.
  motif(g, x, y, cell, kind, ink, alpha) {
    const t = Math.max(cell * 0.11, 1.5);
    g.save();
    g.globalAlpha = alpha ?? 0.4;
    g.strokeStyle = ink ?? '#fff';
    g.fillStyle = ink ?? '#fff';
    g.lineWidth = t;
    g.lineCap = 'butt';
    const f = (n) => n * cell;
    const line = (x1, y1, x2, y2) => {
      g.beginPath(); g.moveTo(x + f(x1), y + f(y1)); g.lineTo(x + f(x2), y + f(y2)); g.stroke();
    };
    if (kind === 'diagonal') line(0.24, 0.76, 0.76, 0.24);
    else if (kind === 'dome') {
      g.beginPath(); g.arc(x + f(0.5), y + f(0.52), f(0.20), 0, Math.PI * 2); g.fill();
    } else if (kind === 'bands') { line(0.22, 0.34, 0.78, 0.34); line(0.22, 0.64, 0.78, 0.64); }
    else if (kind === 'weave') { line(0.5, 0.22, 0.5, 0.78); line(0.22, 0.5, 0.78, 0.5); }
    else if (kind === 'straps') { line(0.34, 0.22, 0.34, 0.78); line(0.64, 0.22, 0.64, 0.78); }
    else if (kind === 'shared') {
      const r = 0.26, p = [[0.5, 0.5 - r], [0.5 + r, 0.5], [0.5, 0.5 + r], [0.5 - r, 0.5]];
      for (let i = 0; i < 4; i++) {
        const a = p[i], b = p[(i + 1) % 4];
        line(a[0], a[1], b[0], b[1]);
      }
    }
    g.restore();
  }

  /// Trace the outside edge of a set of cells and nothing else.
  ///
  /// Used for a component's own edge, for an item's outline, and for a lock —
  /// all three want "where does this shape end", and none of them wants a line
  /// through the middle of it.
  edge(g, cells, origin, colour, width, inset = 0, dash = null) {
    const own = new Set(cells.map(([x, y]) => `${x},${y}`));
    g.save();
    g.strokeStyle = colour;
    g.lineWidth = width;
    if (dash) g.setLineDash(dash);
    g.beginPath();
    for (const [x, y] of cells) {
      const [px, py] = origin(x, y);
      const a = px - inset, b = py - inset, w = CELL + inset * 2;
      if (!own.has(`${x},${y - 1}`)) { g.moveTo(a, b); g.lineTo(a + w, b); }
      if (!own.has(`${x},${y + 1}`)) { g.moveTo(a, b + w); g.lineTo(a + w, b + w); }
      if (!own.has(`${x - 1},${y}`)) { g.moveTo(a, b); g.lineTo(a, b + w); }
      if (!own.has(`${x + 1},${y}`)) { g.moveTo(a + w, b); g.lineTo(a + w, b + w); }
    }
    g.stroke();
    g.restore();
  }

  draw() {
    if (!this.state) return;
    const g = this.c.getContext('2d');
    const C = this.chrome();
    const L = this.look;
    g.clearRect(0, 0, this.c.width, this.c.height);
    // The grid colours were picked against a near-black panel, and read as
    // floating holes on anything else. The board brings its own ground.
    g.fillStyle = '#101018';
    g.fillRect(0, 0, this.c.width, this.c.height);
    g.font = '11px ui-monospace, Menlo, monospace';
    g.textBaseline = 'alphabetic';

    // The pulse the assembled outline rides on.
    const t = performance.now() / 1000;
    const [lo, hi] = L.assembled_alpha;
    const pulse = lo + (hi - lo) * (Math.sin(t * L.pulse_hz * Math.PI * 2) * 0.5 + 0.5);

    for (const name of this.slotOrder) {
      const b = this.boxes[name];
      const s = this.state.slots.find((s) => s.slot === name);
      const origin = (x, y) => [b.x + PAD + x * (CELL + GAP), b.y + PAD + y * (CELL + GAP)];

      g.fillStyle = C.ink3;
      g.fillText(name.toUpperCase(), b.x, b.y - 8);

      // The empty grid: low contrast on purpose. It is a ruler, not a subject.
      for (let y = 0; y < s.rows; y++) {
        for (let x = 0; x < 6; x++) {
          const [px, py] = origin(x, y);
          g.fillStyle = (x + y) % 2 === 0 ? L.cell_a : L.cell_b;
          g.fillRect(px, py, CELL, CELL);
        }
      }

      // The drag footprint: the cells this drop would actually claim, not
      // every cell it could go in. Green only when the whole shape lands.
      if (this.held && this.legalSlot === name && this.hover?.slot === name) {
        const ok = this.legal.has(`${this.hover.x},${this.hover.y}`);
        const shape = this.held.cells ?? [[0, 0]];
        g.save();
        g.globalAlpha = L.footprint_alpha;
        g.fillStyle = ok ? L.legal : L.illegal;
        for (const [dx, dy] of shape) {
          const cx = this.hover.x + dx, cy = this.hover.y + dy;
          if (cx < 0 || cy < 0 || cx >= 6 || cy >= s.rows) continue;
          const [px, py] = origin(cx, cy);
          g.fillRect(px, py, CELL, CELL);
        }
        g.restore();
      }

      // Components. Fill and mark first with no inset, then each one's own
      // outer edge — so a piece reads as one shape and the lines you see
      // inside an item are the seams between its parts.
      for (const p of s.placed) {
        for (const [cx, cy] of p.cells) {
          const [px, py] = origin(cx, cy);
          this.cellFill(g, px, py, p);
        }
        this.edge(g, p.cells, origin, L.piece_edge, Math.min(Math.max(CELL * 0.09, 1.5), 3));
      }

      // Effect and trigger markers, so a board shows where its rules live.
      for (const p of s.placed) {
        if (!p.effect && !p.trigger) continue;
        const [cx, cy] = p.cells[0];
        const [px, py] = origin(cx, cy);
        let mx = px + CELL - 5;
        if (p.trigger) { g.fillStyle = L.trigger; g.beginPath(); g.arc(mx, py + 5, 2.2, 0, 7); g.fill(); mx -= 6; }
        if (p.effect) { g.fillStyle = L.effect; g.beginPath(); g.arc(mx, py + 5, 2.2, 0, 7); g.fill(); }
      }

      // Item outlines. Assembled is brightness and weight — pulsing white,
      // thick — and not-assembled is near-black and thin.
      //
      // **Never gold against red.** That pair is the one distinction red-green
      // colour blindness is worst at, and the gold would collide with the
      // greaves hue besides. This board shipped that way for two milestones.
      for (const item of s.items) {
        const cells = item.cells;
        if (!cells.length) continue;
        if (item.assembled) {
          g.save();
          g.globalAlpha = pulse;
          this.edge(g, cells, origin, L.assembled, L.assembled_width, 1);
          g.restore();
          this.pips(g, origin, cells, item.rating);
        } else {
          this.edge(g, cells, origin, L.unassembled, L.unassembled_width, 1);
        }
      }

      // A lock is solid gold, so "I decided this" reads differently from "this
      // happens to be assembled".
      const locked = s.placed.filter((p) => p.locked);
      if (locked.length) {
        const groups = new Map();
        for (const p of locked) {
          const item = s.items.find((i) => i.pieces.includes(p.id));
          const key = item ? item.pieces.join(',') : String(p.id);
          if (!groups.has(key)) groups.set(key, []);
          groups.get(key).push(...p.cells);
        }
        for (const cells of groups.values()) {
          this.edge(g, cells, origin, L.locked, L.locked_width, 1);
        }
      }

      // What the grid made, under it.
      let ty = b.y + b.h + 14;
      for (const item of s.items) {
        if (ty > b.y + b.h + 52) break;
        g.fillStyle = item.assembled ? C.ink : C.ink3;
        const label = item.assembled ? `${item.short}  ${item.rating}` : item.status;
        g.fillText(label.length > 26 ? `${label.slice(0, 25)}…` : label, b.x, ty);
        ty += 13;
      }
    }

    this.drawBag(g, C);

    // The held component rides the cursor wearing what it would become.
    if (this.held && this.mouse) {
      const look = this.held.over ?? this.held.look ?? { fill: '#888', motif: 'shared', ink: '#fff', ink_alpha: 0.4 };
      const { px, py } = this.mouse;
      g.save();
      g.globalAlpha = 0.92;
      const shape = this.held.cells ?? [[0, 0]];
      for (const [dx, dy] of shape) {
        this.cellFill(g, px + 10 + dx * CELL, py + 10 + dy * CELL, look);
      }
      this.edge(g, shape, (x, y) => [px + 10 + x * CELL, py + 10 + y * CELL],
                L.piece_edge, 2);
      g.restore();
      g.fillStyle = C.ink;
      g.fillText(`${this.held.name} — right-click to turn`, px + 14, py - 6);
    }
  }

  /// Rarity pips: diamonds on the item's topmost-then-leftmost cell.
  ///
  /// Brightness climbs with the tier, so the pips read as a rank without the
  /// colours needing to be told apart — the count alone carries it.
  pips(g, origin, cells, rating) {
    const n = rating >= 170 ? 3 : rating >= 130 ? 2 : rating >= 90 ? 1 : 0;
    if (!n) return;
    const top = cells.reduce((a, b) => (b[1] < a[1] || (b[1] === a[1] && b[0] < a[0]) ? b : a));
    const [px, py] = origin(top[0], top[1]);
    const shade = n === 3 ? '#f0d890' : n === 2 ? '#c8c8dc' : '#9a9ab0';
    g.save();
    g.fillStyle = shade;
    for (let i = 0; i < n; i++) {
      const cx = px + 5 + i * 7, cy = py + 5, r = 2.6;
      g.beginPath();
      g.moveTo(cx, cy - r); g.lineTo(cx + r, cy); g.lineTo(cx, cy + r); g.lineTo(cx - r, cy);
      g.closePath(); g.fill();
    }
    g.restore();
  }

  drawBag(g, C) {
    const L = this.look;
    g.fillStyle = C.ink3;
    g.fillText(`THE BAG — ${this.state.bag.length} loose`, PAD, this.bagY - 10);
    const COL = 168, ROW = CELL + 8;
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / COL));
    this.state.bag.forEach((p, i) => {
      const x = PAD + (i % perRow) * COL;
      const y = this.bagY + Math.floor(i / perRow) * ROW;
      if (y + CELL > this.c.height - 4) return;
      const on = this.held?.id === p.id;
      if (on) {
        g.fillStyle = 'rgba(240,200,90,.16)';
        g.fillRect(x - 3, y - 3, COL - 8, CELL + 6);
      }
      // A swatch showing what it is: grey and a diamond while it is ambiguous,
      // its grid's colour and mark when it only ever goes one place.
      this.cellFill(g, x, y, p);
      this.edge(g, [[0, 0]], () => [x, y], L.piece_edge, 1.5);
      g.fillStyle = on ? '#f0c85a' : C.ink;
      const room = COL - CELL - 20;
      let label = p.name;
      while (g.measureText(label).width > room && label.length > 2) label = label.slice(0, -1);
      if (label !== p.name) label = `${label.slice(0, -1)}…`;
      g.fillText(label, x + CELL + 7, y + CELL * 0.5 + 4);
    });
  }
}
