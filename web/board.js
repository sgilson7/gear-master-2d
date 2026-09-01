// The gear board, in Canvas 2D.
//
// This file draws and hit-tests. It answers no rules: which cells a piece may
// occupy, which pieces form an item, whether that item assembled and what it is
// worth are all core's, fetched through `legal_anchors` and `board_json`. The
// green-and-red fit preview *is* the legal-anchor list rendered; if this file
// ever computes one, that is the bug.
//
// Built in the order PLAN.md M3 sets out — fit preview, then drag and drop,
// then rotate, then the assembly outline, then the cooldown bars — because the
// first of those is the one everything else is checked against.

const CELL = 24;
const GAP = 2;
const PAD = 8;

export class Board {
  constructor(canvas, api) {
    this.c = canvas;
    this.api = api;              // { boardJson, legalAnchors, place, pickUp, rotate, toggleLock }
    this.state = null;
    this.held = null;            // { id, from: slot|null, cells, name }
    this.legal = null;           // Set of "x,y" for the held piece in the hovered slot
    this.hover = null;           // { slot, x, y }
    this.onchange = () => {};
    this.slotOrder = ['weapon', 'helmet', 'chest', 'gloves', 'greaves'];

    canvas.addEventListener('mousemove', (e) => this.move(e));
    canvas.addEventListener('mouseleave', () => { this.hover = null; this.draw(); });
    canvas.addEventListener('mousedown', (e) => this.press(e));
    canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); this.rotateHeld(); });
  }

  refresh() {
    this.state = JSON.parse(this.api.boardJson());
    this.layout();
    this.draw();
    this.onchange(this.state);
  }

  // Where each grid sits on the canvas. Five grids across the top in a row that
  // wraps, and the bag underneath.
  layout() {
    const gw = 6 * (CELL + GAP) + PAD * 2;
    this.cols = Math.max(1, Math.floor((this.c.width - PAD) / (gw + PAD)));
    this.boxes = {};
    let x = PAD, y = 34, rowH = 0;
    for (const name of this.slotOrder) {
      const s = this.state.slots.find((s) => s.slot === name);
      const gh = s.rows * (CELL + GAP) + PAD * 2;
      if (x + gw > this.c.width - PAD && x > PAD) { x = PAD; y += rowH + 96; rowH = 0; }
      this.boxes[name] = { x, y, w: gw, h: gh, rows: s.rows };
      rowH = Math.max(rowH, gh);
      x += gw + PAD;
    }
    this.bagY = y + rowH + 96;
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
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / 132));
    const i = Math.floor((py - this.bagY) / 26) * perRow +
              Math.floor((px - PAD) / 132);
    if (py < this.bagY) return null;
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
    const changedSlot = cell?.slot !== this.hover?.slot;
    this.hover = cell;
    if (this.held && cell && (changedSlot || !this.legal)) this.askLegal(cell.slot);
    this.draw();
  }

  // The one question this file is not allowed to answer itself.
  askLegal(slot) {
    if (!this.held) { this.legal = null; return; }
    const list = JSON.parse(this.api.legalAnchors(this.held.id, slot));
    this.legal = new Set(list.map(([x, y]) => `${x},${y}`));
    this.legalSlot = slot;
  }

  press(e) {
    const r = this.c.getBoundingClientRect();
    const px = (e.clientX - r.left) * (this.c.width / r.width);
    const py = (e.clientY - r.top) * (this.c.height / r.height);

    if (e.button === 2) return;

    const cell = this.cellAt(px, py);
    if (this.held) {
      if (cell) {
        const why = this.api.place(this.held.id, cell.slot, cell.x, cell.y);
        if (why) { this.say(why); return; }
        this.held = null; this.legal = null;
        this.refresh();
      } else {
        // Dropped outside a grid: put it back in the bag.
        if (this.held.from) this.api.pickUp(this.held.id);
        this.held = null; this.legal = null;
        this.refresh();
      }
      return;
    }

    if (cell) {
      const p = this.pieceAt(cell.slot, cell.x, cell.y);
      if (!p) return;
      if (e.shiftKey) { this.api.toggleLock(p.id); this.refresh(); return; }
      const why = this.api.pickUp(p.id);
      if (why) { this.say(why); return; }
      this.held = { id: p.id, from: cell.slot, name: p.name };
      this.askLegal(cell.slot);
      this.refresh();
      return;
    }

    const bagged = this.bagAt(px, py);
    if (bagged) {
      this.held = { id: bagged.id, from: null, name: bagged.name, slot: bagged.slot };
      this.askLegal(bagged.slot);
      this.draw();
    }
  }

  rotateHeld() {
    const id = this.held?.id ?? (this.hover && this.pieceAt(this.hover.slot, this.hover.x, this.hover.y)?.id);
    if (id === undefined || id === null) return;
    const why = this.api.rotate(id);
    if (why) this.say(why);
    if (this.held) this.askLegal(this.legalSlot ?? this.held.slot);
    this.refresh();
  }

  say(text) { this.onsay?.(text); }

  // --------------------------------------------------------------- drawing

  colors() {
    const cs = getComputedStyle(document.body);
    const v = (n, d) => (cs.getPropertyValue(n) || d).trim();
    return {
      ink: v('--ink', '#171a19'),
      ink3: v('--ink-3', '#6e736c'),
      line: v('--line', '#c9ccc2'),
      surface: v('--surface', '#f4f5ef'),
      ground: v('--ground', '#e9eae4'),
      brass: v('--brass', '#8a5c1c'),
      rust: v('--rust', '#8b4225'),
      verd: v('--verdigris', '#2c6759'),
    };
  }

  draw() {
    if (!this.state) return;
    const g = this.c.getContext('2d');
    const C = this.colors();
    g.clearRect(0, 0, this.c.width, this.c.height);
    g.font = '11px ui-monospace, Menlo, monospace';

    for (const name of this.slotOrder) {
      const b = this.boxes[name];
      const s = this.state.slots.find((s) => s.slot === name);

      g.fillStyle = C.ink3;
      g.fillText(name.toUpperCase(), b.x, b.y - 8);

      g.fillStyle = C.surface;
      g.fillRect(b.x, b.y, b.w, b.h);
      g.strokeStyle = C.line;
      g.lineWidth = 1;
      g.strokeRect(b.x + .5, b.y + .5, b.w - 1, b.h - 1);

      for (let y = 0; y < s.rows; y++) {
        for (let x = 0; x < 6; x++) {
          const px = b.x + PAD + x * (CELL + GAP);
          const py = b.y + PAD + y * (CELL + GAP);
          g.fillStyle = C.ground;
          g.fillRect(px, py, CELL, CELL);
        }
      }

      // The fit preview: green where core says the held piece may sit, red on
      // the cell under the cursor when it may not.
      if (this.held && this.legalSlot === name && this.hover?.slot === name) {
        for (let y = 0; y < s.rows; y++) {
          for (let x = 0; x < 6; x++) {
            if (!this.legal.has(`${x},${y}`)) continue;
            const px = b.x + PAD + x * (CELL + GAP);
            const py = b.y + PAD + y * (CELL + GAP);
            g.fillStyle = 'rgba(44,103,89,.20)';
            g.fillRect(px, py, CELL, CELL);
          }
        }
        const ok = this.legal.has(`${this.hover.x},${this.hover.y}`);
        const px = b.x + PAD + this.hover.x * (CELL + GAP);
        const py = b.y + PAD + this.hover.y * (CELL + GAP);
        g.fillStyle = ok ? 'rgba(44,103,89,.55)' : 'rgba(139,66,37,.5)';
        g.fillRect(px, py, CELL, CELL);
      }

      // Pieces. Each one draws its own outer edge, so two pieces sitting
      // flush read as two pieces rather than one blob — which matters here
      // more than in most grids, because "are these one item or two" is the
      // question the whole board is about.
      for (const p of s.placed) {
        const own = new Set(p.cells.map(([x, y]) => `${x},${y}`));
        for (const [cx, cy] of p.cells) {
          const px = b.x + PAD + cx * (CELL + GAP);
          const py = b.y + PAD + cy * (CELL + GAP);
          g.fillStyle = p.locked ? C.brass : 'rgba(138,92,28,.45)';
          g.fillRect(px, py, CELL, CELL);
        }
        g.strokeStyle = C.ink;
        g.lineWidth = 1;
        g.globalAlpha = .45;
        for (const [cx, cy] of p.cells) {
          const px = b.x + PAD + cx * (CELL + GAP);
          const py = b.y + PAD + cy * (CELL + GAP);
          if (!own.has(`${cx},${cy - 1}`)) line(g, px, py, px + CELL, py);
          if (!own.has(`${cx},${cy + 1}`)) line(g, px, py + CELL, px + CELL, py + CELL);
          if (!own.has(`${cx - 1},${cy}`)) line(g, px, py, px, py + CELL);
          if (!own.has(`${cx + 1},${cy}`)) line(g, px + CELL, py, px + CELL, py + CELL);
        }
        g.globalAlpha = 1;
      }

      // The assembly outline. Gold when the item came together, dashed red
      // when it did not — core decides which, and `status` says what is
      // missing. Drawn last so nothing covers it.
      for (const item of s.items) {
        const cells = new Set(item.cells.map(([x, y]) => `${x},${y}`));
        g.strokeStyle = item.assembled ? C.brass : C.rust;
        g.lineWidth = item.assembled ? 3 : 1.5;
        if (!item.assembled) g.setLineDash([4, 3]);
        for (const key of cells) {
          const [x, y] = key.split(',').map(Number);
          const px = b.x + PAD + x * (CELL + GAP) - 1;
          const py = b.y + PAD + y * (CELL + GAP) - 1;
          const w = CELL + 2;
          if (!cells.has(`${x},${y - 1}`)) line(g, px, py, px + w, py);
          if (!cells.has(`${x},${y + 1}`)) line(g, px, py + w, px + w, py + w);
          if (!cells.has(`${x - 1},${y}`)) line(g, px, py, px, py + w);
          if (!cells.has(`${x + 1},${y}`)) line(g, px + w, py, px + w, py + w);
        }
        g.setLineDash([]);
      }

      // What each grid made, under it. The short name and the rating, because
      // those are the two things a player compares two arrangements by.
      let ty = b.y + b.h + 14;
      for (const item of s.items) {
        if (ty > b.y + b.h + 58) break;
        g.fillStyle = item.assembled ? C.ink : C.ink3;
        const label = item.assembled
          ? `${item.short}  ${item.rating}`
          : item.status;
        g.fillText(label.length > 30 ? label.slice(0, 29) + '…' : label, b.x, ty);
        ty += 14;
      }
    }

    this.drawBag(g, C);

    if (this.held && this.mouse) {
      g.fillStyle = C.ink;
      g.fillText(`carrying ${this.held.name} — right-click to turn`, this.mouse.px + 14, this.mouse.py - 8);
    }
  }

  drawBag(g, C) {
    g.fillStyle = C.ink3;
    g.fillText(`THE BAG — ${this.state.bag.length} loose`, PAD, this.bagY - 10);
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / 132));
    this.state.bag.forEach((p, i) => {
      const x = PAD + (i % perRow) * 132;
      const y = this.bagY + Math.floor(i / perRow) * 26;
      if (y > this.c.height - 12) return;
      const on = this.held?.id === p.id;
      g.fillStyle = on ? C.brass : C.surface;
      g.fillRect(x, y, 128, 22);
      g.strokeStyle = C.line;
      g.strokeRect(x + .5, y + .5, 127, 21);
      g.fillStyle = on ? C.surface : C.ink;
      const label = p.name.length > 17 ? p.name.slice(0, 16) + '…' : p.name;
      g.fillText(label, x + 6, y + 15);
    });
  }
}

function line(g, x1, y1, x2, y2) {
  g.beginPath(); g.moveTo(x1, y1); g.lineTo(x2, y2); g.stroke();
}
