## Task: HTML Page UI

### Scope
- Create semantic HTML structure for Wordle solver interface
- Implement Wordle-like tile input styling
- Add responsive CSS with proper units (rem/em)
- Implement auto-advance/backspace keyboard behaviour
- Support dynamic misplaced letters rows

### Acceptance Criteria
- [ ] HTML uses semantic elements (`<main>`, `<section>`, `<fieldset>`, `<legend>`)
- [ ] Correct letters input: 1 row of 5 tile inputs
- [ ] Misplaced letters input: 1+ rows of 5 tile inputs with add/remove
- [ ] Excluded letters input: single text field
- [ ] Tile inputs styled like original Wordle (light theme, coloured states)
- [ ] Auto-advance to next tile on letter input
- [ ] Backspace moves to previous tile when current is empty
- [ ] Responsive layout: desktop (side-by-side), mobile (stacked)
- [ ] Uses rem/em units for accessibility (scales with user font preferences)
- [ ] Add row button works for misplaced letters
- [ ] Remove row button works (except first row)

### Implementation Order

**Step 1: HTML Structure**
- Update `index.html` with semantic layout
- Add `<section id="inputs">` with fieldsets for each input type
- Add `<section id="outputs">` for results display
- Add dynamic row structure for misplaced letters

**Step 2: CSS Styling**
- Update `static/style.css` with CSS variables for responsive units
- Implement light theme colour palette (5 colours from Wordle)
- Style tile inputs (3rem, font size, gaps)
- Add responsive breakpoints for mobile layout
- Style add/remove row buttons

**Step 3: JavaScript Behaviour**
- Add inline `<script>` in `index.html`
- Implement auto-advance on letter input
- Implement backspace navigation
- Implement add/remove misplaced rows
- Limit max rows to 5 (Wordle max guesses)

**Step 4: Verification**
- Visual check with `trunk serve`
- Test keyboard navigation between tiles
- Test responsive layout on different screen sizes
- Test add/remove misplaced rows

### File Structure
```
wordle-finder/
├── index.html              # HTML structure + inline script
├── static/
│   └── style.css           # Styling with CSS variables
├── src/                    # (existing, unchanged)
├── data/                   # (existing, unchanged)
└── ...
```

### Known Patterns
- Use PicoCSS-inspired semantic HTML (minimal classes)
- CSS variables for theming and responsive units
- Light theme with 5-color palette (easy to swap for dark theme later)
- Tile size: `--tile-size: 3rem` (scales with user font preferences)
- Gap between tiles: `4px` (fixed, small spacing)

### Colour Palette
```css
:root {
    /* 5-Color Palette */
    --color-bg: #ffffff;
    --color-tile: #d3d6da;
    --color-correct: #6aaa64;
    --color-misplaced: #c9b458;
    --color-absent: #787c7e;

    /* Text (derived) */
    --color-text: #1a1a1a;
    --color-text-tile: #ffffff;
}
```

### Scope Limits
- Do NOT implement WASM bindings (future task)
- Do NOT implement Rust visualizer module (future task)
- Do NOT add paste handling (can add later)
- Do NOT implement result rendering (future task)
- Escalate if auto-advance behaviour conflicts with accessibility

### Verification
```bash
trunk serve              # Dev server with hot-reload
# Manual testing:
# - Type letters in tiles, verify auto-advance
# - Backspace on empty tile, verify moves to previous
# - Add/remove misplaced rows
# - Check responsive layout (resize browser)
```

### Implementation Details and Deviations

**1. Keyboard navigation implementation**
- Original: Per-input event listeners attached at load, re-attached manually after row clone
- Actual: Event delegation via single `document` listener (`input` + `keydown`), with `getTileInputs()` recomputing the tile set on every event
- Rationale: Consistent navigation after dynamic add/remove; no manual re-attaching; automatically handles cloned rows

**2. Cross-word navigation**
- Original: Not specified (implied per-word only)
- Actual: Auto-advance/backspace crosses between words (rows) in DOM order (correct row first, then misplaced rows)
- Rationale: User decided cross-word focus jump is acceptable if consistent; never crosses into excluded input or buttons (they lack `.word-row input`)

**3. Filled tile coloring**
- Original: "coloured states" implementation unspecified
- Actual: CSS-only via `:placeholder-shown` / `:not(:placeholder-shown)` pseudo-classes; `placeholder=" "` added to tile inputs. Correct row filled → `--color-correct` (green), misplaced row filled → `--color-misplaced` (yellow)
- Rationale: No JavaScript needed; user-requested CSS-only approach; `--color-correct`/`--color-misplaced` now used for filled tile states

**4. Focus outline color**
- Original: Not specified
- Actual: `--color-absent` (gray) for both tile inputs and excluded input focus
