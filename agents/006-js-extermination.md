## Task: JS Extermination — Replace Inline JavaScript with Rust

### Scope
- Remove the entire inline `<script>` block from `index.html`.
- Reimplement all its behavior in Rust (`src/visualizer.rs`).
- Implement the new DESIGN.md UI requirements: a "clear all inputs" button and the modified misplaced-row deletion logic.
- Update HTML/CSS for the new clear-all button.

### Acceptance Criteria
- [x] No `<script>` remains in `index.html`
- [x] Auto-advance to next tile on input (when a tile has a value and is not the last)
- [x] Backspace auto-navigation to previous tile (when tile empty and not first)
- [x] Add misplaced row via `#add-row` (max 5 rows), new row empty; does NOT refresh results
- [x] Remove misplaced row: if >1 row → delete the row and refresh results; if ==1 row → clear that row's tile inputs and refresh results
- [x] Clear-all button (after the inputs): clears correct tiles, all misplaced tile inputs, excluded input, and clears `#error` + `#results`
- [x] `#error` styling and `#add-row`/`.remove-row`/clear-all button styling reuse existing look
- [x] All new logic in Rust (no JS)
- [x] `cargo test` passes
- [x] Manual `trunk serve` check passes

### Implementation Order (TDD)

**Step 1: web-sys features (`Cargo.toml`)**
- Add features required for DOM manipulation/events:
  - `Node` (clone_node, append_child, remove, first_child)
  - `KeyboardEvent` (read `key()` in keydown)
  - `DomTokenList` (`class_list` to detect `.remove-row`)
  - `HtmlCollection` (for `children()` on `#misplaced-rows`)
- Note: `HtmlButtonElement` was initially added but later removed (see Step 6).

**Step 2: Event listeners + navigation (`src/visualizer.rs`)**
- Add `keydown` document-level listener (Backspace navigation)
- Add `click` document-level listener (delegation: `#add-row`, `.remove-row`, clear-all)
- `get_tile_inputs(document) -> Vec<HtmlInputElement>`: queries all `.word-row input` in DOM order (mirrors JS `getTileInputs`)
- Auto-advance: in the existing `input` handler, if target is a `.word-row input` with a value and not last → `.focus()` next tile
- Backspace: on `keydown` with `key == "Backspace"`, target empty `.word-row input`, not first → `.focus()` previous tile
- Unit-testable pure helpers where applicable (e.g. row index / next-index logic) — keep DOM-free

**Step 3: Row add/remove + clear-all (`src/visualizer.rs`)**
- Add row (`#add-row` click): if `#misplaced-rows` has <5 children → clone first `.word-row`, clear its input values, append. Do NOT refresh results (new row empty)
- Remove row (`.remove-row` click): locate `.word-row`
  - if >1 row → remove the row; refresh results
  - if ==1 row → clear that row's tile inputs; refresh results
- Clear-all button click: clear correct tiles + all misplaced tile inputs + excluded input; clear `#error`; clear `#results`. (No `handle_input` call — explicit blank state)

**Step 4: HTML/CSS (`index.html`, `static/style.css`)**
- `index.html`: remove `<script>` block; add clear-all button after the inputs (e.g. after the `#excluded-letters` fieldset, inside `#inputs`);
- `static/style.css`: style clear-all button reusing existing button look;

**Step 5: Verification**
- `cargo test`
- `trunk build`
- `trunk serve` manual check:
  - typing advances tiles; backspace walks back
  - add rows (max 5); remove deletes when >1, clears when 1
  - results update on remove; not on add
  - clear-all empties everything and clears error + results
  - no JS in page

### Known Patterns
- Reuse document-level event delegation (matches prior JS + existing Rust `input` listener)
- Keep pure logic unit-testable (no DOM in tests)
- Guard missing DOM nodes with `Option`/early-return, not `expect`
- Programmatic `set_value` does NOT fire `input` events → refresh results/error explicitly after DOM-changing operations
- Max misplaced rows = 5

### Scope Limits
- Do NOT add result styling (separate task)
- Do NOT change the remove-button position (stays right; DESIGN.md typo fixed)
- Do NOT refresh results on add-row (new row is empty — no change)
- Escalate if web-sys API mismatch or DOM query issues

### Verification
```bash
cargo test           # updated suite passes
cargo build          # compiles
trunk serve          # manual: navigation, add/remove, clear-all, no JS
```

### File Structure
```
wordle-finder/
├── src/
│   ├── visualizer.rs         # all event handlers + navigation (JS removed)
│   ├── main.rs               # (unchanged)
│   └── ...
├── index.html                # remove <script>; add clear-all button
├── static/
│   └── style.css             # clear-all button styling
└── agents/
    └── 006-js-extermination.md  # this task
```

### Implementation Details and Deviations

<!-- Written after completing each step -->

#### Step 1: web-sys features (COMPLETE)
- Added to `Cargo.toml`: `HtmlButtonElement`, `Node`, `KeyboardEvent`, `DomTokenList`, `HtmlCollection` (plus existing `Document`, `Element`, `HtmlElement`, `HtmlInputElement`, `NodeList`, `Event`, `EventTarget`, `Window`, `console`).
- `cargo build` succeeds (only pre-existing `WORD_COUNT` warning). Features will be pruned if any turn out unused during implementation.

#### Step 2: Event listeners + navigation (COMPLETE)
- `init()` now attaches `input` (handle_input + auto_advance) and `keydown` (backspace_navigate) document-level listeners, each capturing its own `document.clone()`.
- Added:
  - `get_tile_inputs(document) -> Vec<HtmlInputElement>`: all `.word-row input` in DOM order.
  - `auto_advance`: on `input`, if a `.word-row input` with a value and not last → focus next tile. Excluded/other inputs naturally skipped (not in `.word-row input`).
  - `backspace_navigate`: on `keydown` `"Backspace"` on an empty `.word-row input` not first → focus previous tile.
- Navigation logic kept inline (per user; no separate pure helper/tests — verified manually in Step 5).
- 72 tests pass; trunk build succeeds.

#### Step 3: Row add/remove + clear-all (COMPLETE)
- Added `click` document-level listener dispatching via `handle_click` (checks `#add-row`, `.remove-row` via `closest`, `#clear-all`).
- `add_row`: if `#misplaced-rows` has <5 children → clone first `.word-row`, clear its inputs, append. Does NOT refresh results.
- `remove_row`: if >1 row → `row.remove()`; else → `clear_row_inputs(row)`. Both branches call `handle_input(document)` to refresh results/error.
- `clear_all`: clears tile inputs, excluded input, and clears `#error` + `#results`. No `handle_input` call (explicit blank state).
- `clear_row_inputs(row)`: clears the 5 inputs of a `.word-row` (reused by add/single-remove).
- **Deviation (per user)**: `clear_all` reuses `get_tile_inputs(document)` to clear all tile inputs (rather than a separate `query_selector_all`) — since `get_tile_inputs` already returns correct + misplaced tile elements.
- `#clear-all` button referenced by handler but not yet in HTML (added in Step 4); branch no-ops until then.
- 72 tests pass; trunk build succeeds.

#### Step 4: HTML/CSS (COMPLETE)
- `index.html`: removed the entire inline `<script>` block (all navigation/add/remove JS gone). Added `<button type="button" id="clear-all">Clear All</button>` after the `#excluded-letters` fieldset inside `#inputs`.
- `static/style.css`: **no new rule needed** — `#clear-all` and `#add-row` both inherit the existing global `button` styling (padding, font, background, hover). `.remove-row` unchanged.
- Verified: built `dist/index.html` contains no `getTileInputs`/`addEventListener`/`document.getElementById` — only the new clear-all button remains in markup.
- 72 tests pass; trunk build succeeds.

#### Step 5: Bug fix — remove-row button (COMPLETE)
- **Bug**: clicking `.remove-row` removed only the button, not the row.
- **Root cause**: `handle_click` used `target.closest(".remove-row")`, which returns the `.remove-row` **button** itself (the button carries that class). `remove_row` then called `row.remove()` on the button, leaving the `.word-row` intact.
- **Fix**: mirror the original JS (`e.target.classList.contains('remove-row')` + `closest('.word-row')`):
  - `handle_click` now checks `target.class_list().contains("remove-row")`, then finds the enclosing `.word-row` via `target.closest(".word-row")` and passes it to `remove_row`.
  - `remove_row` now operates on the `.word-row` (removes 5 inputs + button when >1 rows; clears its inputs when ==1 row).
- **Deviation note**: `class_list().contains()` returns `bool` (not `Result`), so no `.unwrap_or` needed.
- 72 tests pass; trunk build succeeds.

#### Step 6: web-sys feature pruning (COMPLETE)
- Removed `HtmlButtonElement` from `Cargo.toml` — unused (buttons handled via `Element::id()`/`class_list()`; never cast to `HtmlButtonElement`).
- Verified: tests pass, trunk build succeeds.
- Features still used: `Document`, `Element`, `HtmlElement`, `HtmlInputElement`, `Node`, `NodeList`, `Event`, `EventTarget`, `KeyboardEvent`, `DomTokenList` (via `class_list()`), `HtmlCollection` (via `children()`), `Window`, `console`.

#### Step 7: Verification (COMPLETE)
- `cargo test` passes (72 tests).
- **Manual `trunk serve` check passed (user-confirmed)**: navigation, add/remove rows (with delete/clear logic), clear-all, and remove-row fix all work as intended. No JS in page.

#### Task complete
- All acceptance criteria met. 72 tests passing. Inline JavaScript fully replaced by Rust.


