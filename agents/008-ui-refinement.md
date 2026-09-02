## Task: UI Refinement (Tile Styling, Output Words, Error Field, Headers)

### Scope
- Restyle Wordle tile inputs closer to the original game (sizing, border, background)
- Add styling to the output word boxes (side-by-side)
- Add styling to the error field (`> ` prefix + thin underline)
- Center the page headers
- Update DESIGN.md UI section to match

### Acceptance Criteria
- [ ] Tile size uses `min(4rem, 15vw)` (≈62px at max, responsive)
- [ ] Tile gap stays `4px` (pending visual tuning)
- [ ] Tile font uses `calc(1.5rem + min(1vw, 1vh))`
- [ ] Unfilled tile: `1px` `--color-tile` border + `--color-bg` background
- [ ] Focus indicator uses `--color-absent` (via border, not outline)
- [ ] Filled tile states unchanged (correct → `--color-correct`, misplaced → `--color-misplaced`)
- [ ] `h1` and `h2` centered
- [ ] Error field shows `> ` prefix (via CSS `::before`) + thin underline in the error colour
- [ ] Output words styled as distinct boxes (`--color-tile` background) laid out side-by-side, evenly spaced (`space-around`), equal-width and centre-aligned (`min-width: calc(1.5rem + 6ch)`)
- [ ] No Rust code changes (CSS-only + DESIGN.md only)
- [ ] DESIGN.md UI section updated to reflect all of the above

### Implementation Order

**Step 1: Tile sizing/gap/font (`static/style.css` `:root`)**
- `--tile-size: 3rem` → `min(4rem, 15vw)`
- `--tile-gap: 4px` (unchanged for now)
- `--tile-font: 1.5rem` → `calc(1.5rem + min(1vw, 1vh))`

**Step 2: Tile appearance (`static/style.css` `.word-row input`)**
- `border: none` → `border: 1px solid var(--color-tile)`
- `background: var(--color-tile)` → `background: var(--color-bg)`
- Keep `outline: none`; change focus rule to `border-color: var(--color-absent)`
- Filled states (`#correct-letters` / `#misplaced-letters` `:not(:placeholder-shown)`) unchanged

**Step 3: Center headers (`static/style.css`)**
- Add `text-align: center` to `h1` and `h2`

**Step 4: Error field styling (`static/style.css` `#error`)**
- Add `::before { content: "> "; }` prefix
- Add thin underline: `border-bottom: 1px solid <error colour>`
- Keep current error colour (`--color-misplaced`)

**Step 5: Output words (`static/style.css` `#results > div`)**
- `background: var(--color-tile)`, `border: 1px solid var(--color-tile)`, uppercase, bold, padding
- Make `#results` a wrapping flex/grid so word boxes sit side-by-side (wrap on overflow)
- Add `justify-content: space-around` for even spacing; give each box `min-width: calc(1.5rem + 6ch)` + `text-align: center` for equal-width, centre-aligned boxes (proportional font)
- No Rust change to `render_results` (still emits one `<div>word</div>` per word)

**Step 6: DESIGN.md UI section update (lines 297–314)**
- Document tile sizing (`min(4rem, 15vw)` ≈ 62px max, responsive), gap `4px`, tile font formula
- Document unfilled tile appearance (bg + border, focus = `--color-absent`)
- Document filled states unchanged
- Document output word boxes (`--color-tile` bg, side-by-side)
- Document error field (`> ` prefix + thin underline, current colour)
- Document centered headers

### Known Patterns
- CSS-only; no changes to Rust visualizer, `index.html`, or inline behaviour
- All colours constrained to the existing 5-colour palette + derived text vars
- Uses rem/em/vw units (accessible, scales with user preferences)
- Output words are the plain `<div>word</div>` elements produced by `render_results`

### Colour Palette (from `static/style.css`, unchanged)
```css
--color-bg: #f6f4ee;
--color-tile: #d3d6da;
--color-correct: #6aaa64;
--color-misplaced: #c9b458;
--color-absent: #787c7e;
--color-text: #1a1a1a;
--color-text-tile: #ffffff;
```

### Scope Limits
- Do NOT modify any Rust source (`src/`) — this is a CSS + DESIGN.md task only
- Do NOT add new palette colours
- Do NOT split output words into per-letter tiles (no Rust `render_results` change here)
- Do NOT change tile gap value (keep `4px`; tune visually in a later pass)
- Escalate if any change requires touching `src/` or `index.html`

### Verification
```bash
trunk serve   # Dev server with hot-reload
# Manual visual check:
# - Tile size/border/background (unfilled vs filled)
# - Output word boxes (background, border, side-by-side wrapping)
# - Error field shows "> " prefix + underline
# - Headers centered
# - Responsive on mobile (tile scales with 15vw, font scales with vmin)
```
Note: `cargo test`/`cargo build` are NOT required for this task (no Rust changes).

### Implementation Details and Deviations

<!-- Written after completing the step -->
1. **Excluded-letters focus left unchanged** — the excluded single text field still styles focus via `outline: 2px solid var(--color-absent)` (its own `:focus` rule), while the tile inputs now use `border-color: var(--color-absent)`. Deliberately out of scope (it is a text field, not a tile); the two focus styles now differ.
2. **Error underline uses `currentColor`** — `#error { border-bottom: 1px solid currentColor; }` so the underline automatically matches the error text colour (`--color-misplaced`), staying within the palette without a second colour reference. Added `padding-bottom: 0.25rem` for breathing room.
3. **Output word box border blends with its background** — both the `#results > div` background and its `1px` border are `--color-tile`. The visual distinction comes from the box's `--color-tile` background against the `#results` container's page-bg padding area (the user requested the `--color-tile` background specifically to make boxes distinct). Worth confirming visually.
4. **`#results` keeps `min-height: 200px` and its outer `1px --color-tile` border** — unchanged from before (not in scope to alter); the inner word boxes sit inside it.
5. **No Rust / HTML changes required** — `render_results` still emits one `<div>word</div>` per word and `#error` still uses `set_text_content`, so all changes are CSS-only as planned.
6. **Output words: `space-around` + fixed `min-width` (added post-review)** — on visual review, words without `justify-content` piled unevenly to one side. Added `#results { justify-content: space-around; }`.
   - Because the font is proportional, differing glyph widths made the boxes uneven. Fixed with `#results > div { min-width: calc(1.5rem + 6ch); text-align: center; }` (`6ch` spans the 5 proportional glyphs; `1.5rem` covers the `0.75rem` horizontal padding on each side). All 5-letter words now render as equal-width boxes with centred text.
   - Grid equal-width columns were tried but rejected (compressed the words too much), per user decision.
   - **Noted, out of scope for now:** a monospace font for the word boxes would equalize widths without sizing math, but was deliberately NOT added in this task (user decision). Revisit later if the `calc`-based approach proves brittle.
7. **DESIGN.md UI section rewritten (generic desired-behaviour, no hard CSS values)** — the `## User Interface` section was reorganized from loose prose + a `#### Misplaced Letters Input` subsubsection into coherent `###` subsections: `### Headers`, `### Tile Inputs`, `### Misplaced Letters`, `### Excluded Letters`, `### Results`, `### Error Field`. Colours are referenced by palette *role* (background, tile, correct, misplaced, absent), not by value.
   - `#### Misplaced Letters Input` was flattened to `### Misplaced Letters` for a consistent flow.
   - `### Headers` placed as a page-level concern right after the layout/clear-all paragraphs (before `### Tile Inputs`), rather than between `Results` and `Error Field`.
   - Smoothed two pre-existing wordings carried from the old text for consistency with the neutral third-person style of the other subsections ("there must be a mechanism", "It is proposed to have a small button… deletes … adds").
