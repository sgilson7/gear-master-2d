# Prompt: generate slide figures as editable TikZ instead of raster images

Paste the block below into Claude/Gemini/ChatGPT, then fill in the `[...]` fields.

---

You are producing a figure for a lecture slide. Do **not** generate a raster image. Instead, write a complete, compilable LaTeX/TikZ document that draws the figure, so that it can be read, checked, and edited as text before it is rendered in Overleaf.

**Figure to draw:** [one-paragraph description of what the figure shows, e.g. "an SFML game window with a labeled coordinate grid, two trees standing on a ground strip, and a player sprite with its bounding box"]

**Facts the figure must get right:** [list every concrete constraint — these are the things a generated image usually gets wrong]
- [e.g. Origin (0,0) is the top-left corner; X increases to the right; Y increases downward. No negative coordinates anywhere.]
- [e.g. Window is 800×600 px; grid lines every 100 px; every intersection labeled "(x,y)".]
- [e.g. The sprite's origin marker is at its texture center.]

**Audience / style:** [e.g. CSC 484/584 Game AI students; dark "engine viewport" look; sans-serif labels; must stay legible when the figure is ~4 in wide on a 16:9 slide]

**Requirements for your output:**
1. Use `\documentclass[tikz,border=4pt]{standalone}` so it compiles on its own in Overleaf with pdflatex. Only use standard packages/TikZ libraries (`arrows.meta`, `shapes.geometric`, `positioning`, `calc`, `decorations`).
2. Put every tunable quantity (sizes, spacing, colors, object positions) in `\newcommand` or `\definecolor` at the top, with a comment explaining each.
3. Express positions in the figure's own units (e.g. pixels) and set the TikZ `x=`/`y=` scale once, so numbers in the code match the numbers shown in the labels. If the domain's Y axis points down, flip it with `y=-<scale>` and comment that you did so.
4. Generate repeated elements (grid lines, labels, trees, ticks) with `\foreach`, not by hand.
5. Group the code into commented sections in drawing order: background → grid/axes → scene objects → annotations → frame/caption.
6. Draw annotation elements (arrows, labels, highlights) last so nothing covers them.
7. Use simple geometric primitives (rectangles, circles, triangles, paths) for objects — no external images.
8. After the code, give a 3–5 line "self-check" listing the facts from the constraint list and stating where in the code each one is enforced.
9. Return only the `.tex` file and the self-check — no prose introduction.

---

## Tips for using the output
- Paste the file into a new Overleaf project and click *Recompile*; the PDF it produces can be dropped into Google Slides/PowerPoint, or the `tikzpicture` environment can be pasted straight into a Beamer frame.
- Because the figure is text, revisions are one-line edits ("change `\Step` to 50", "move the sprite to (300,390)") instead of re-rolling an image generator and hoping.
- If the figure has many labels, check legibility at slide size first; reduce `\foreach` density before shrinking the font.
