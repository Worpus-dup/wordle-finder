## Task: Partial Visualizer Module

### Scope
- Create a Rust visualizer module that reads user inputs from the page, calls the solver, and renders results
- Keep existing inline JS for keyboard navigation (temporary)
- No new JS logic — Rust owns input reading + result rendering
- Defer input sanitization to a later task (verify basic operation first)

### Acceptance Criteria
- [ ] Rust reads correct letters from the 5 `#correct-letters` tile inputs
- [ ] Rust reads misplaced letters from 1+ `#misplaced-rows` tile rows (dynamic)
- [ ] Rust reads excluded letters from the `#excluded-letters` input
- [ ] Rust calls `solver::solve()` and renders results as one `<div>` per word into `#results`
- [ ] On `solve()` error, results are cleared and error is printed to console
- [ ] Uses per-event re-query strategy (handles dynamic misplaced rows)
- [ ] Attaches a document-level `input` event listener via web-sys
- [ ] Existing keyboard-navigation JS untouched
- [ ] `cargo test` passes (1-3 unit tests for pure helper)
- [ ] Manual `trunk serve` check passes

### Implementation Order

**Step 1: Input reading (Rust)**
- Expand `Cargo.toml` web-sys features:
  - `HtmlInputElement` (read `.value()`)
  - `NodeList` (iterate `querySelectorAll` results)
  - `Event`, `EventTarget` (addEventListener)
  - `console` (console logging)
  - `HtmlCollection` (iterate row children if needed)
- Create `src/visualizer.rs`:
  - Pure, unit-testable helper `tiles_to_pattern(&[&str; 5]) -> String` that maps 5 tile values to a 5-char string, using UNKNOWN space for empty tiles
  - 1-3 unit tests for the helper
  - `pub fn init()` with a document-level `input` listener that re-queries inputs per event and logs the assembled inputs to console (verify reading works)
- Wire `main()` to call `visualizer::init()`
- Run `cargo test`

**Step 2: Simplest rendering**
- In the same handler: assemble `correct`, `misplaced` (Vec), `excluded`
- Call `solver::solve()`
- On `Ok(words)`: write one `<div>` per word into `#results` via `set_inner_html`
- On `Err(e)`: clear `#results`, log error via `console::error`
- Run `cargo test`

**Step 3: Manual check**
- `trunk serve`
- Verify typing letters updates results
- Verify add/remove misplaced rows reflect in results
- Verify excluded letters work
- Verify error path logs to console

**Step 4: (Deferred — future task)**
- Input sanitization / user-facing error messages (visualizer responsibility, but defer)

### File Structure
```
wordle-finder/
├── Cargo.toml             # Add web-sys features
├── src/
│   ├── main.rs            # Call visualizer::init()
│   ├── visualizer.rs      # NEW: input reading + rendering
│   ├── solver/            # (existing, unchanged)
│   └── words.rs           # (existing, unchanged)
├── index.html             # (existing, unchanged — JS nav stays)
├── static/
│   └── style.css          # (existing, unchanged)
└── ...
```

### Known Patterns
- `main()` runs automatically on WASM load (trunk); no `#[wasm_bindgen(start)]` needed
- DOM is loaded when `main()` executes
- Use per-event re-query (mirrors JS delegation) due to dynamic misplaced rows
- Document-level `input` listener attached once; nodes re-queried fresh per event
- Guard missing nodes (early-return) rather than panicking with `expect`

### Scope Limits
- Do NOT add new JS logic
- Do NOT rewrite keyboard-navigation JS in Rust yet
- Do NOT implement input sanitization / user-facing errors
- Do NOT implement result styling (tiles/colors for results) — plain `<div>` only
- Escalate if web-sys API mismatch or DOM query issues

### Verification
```bash
cargo test           # Pure helper + solver tests pass
cargo build          # Compiles for wasm target
trunk serve          # Manual check: inputs → results, errors → console
```

### Implementation Details and Deviations

#### Step 1: Input reading (Rust)

1. **`tiles_to_pattern` signature** changed from `&[&str; 5]` to `&[&str]`.
   - Reason: avoids the temporary-borrow compile issue when collecting `HtmlInputElement::value()` results (`String`) into an array of references.
   - Caller builds a `Vec<String>` of raw values, then derives `Vec<&str>` via `iter().map(String::as_str)` before calling the helper.

2. **Reuse `UNKNOWN` constant**: `tiles_to_pattern` uses `crate::solver::validator::UNKNOWN` (the existing `' '` constant) instead of a hardcoded space for empty tiles.

3. **2 unit tests only** (per user): all-empty tiles → `"     "` (5 spaces); mixed pattern → letters placed, empties/spaces map to `UNKNOWN`.

4. **web-sys features added** to `Cargo.toml`: `HtmlInputElement`, `NodeList`, `Event`, `EventTarget`, `console`.
   - **Deviation**: `HtmlCollection` was listed in the plan but is **not used** in the final implementation (rows are iterated via `NodeList` from `query_selector_all`). The feature was removed from `Cargo.toml`.

5. **Step 1 deliverable**: `init()` attaches a document-level `input` listener; the handler re-queries inputs per event (handles dynamic misplaced rows) and logs the assembled `correct` / `misplaced` / `excluded` values to the console via `console::log` to verify reading works. No `solve()` call yet (that is Step 2).

6. **Missing-node guarding**: query helpers return `Option`/default rather than `expect` panicking; `init()` uses `expect` only for window/document (guaranteed in browser context).

7. **Step 1 complete** (64 tests pass, wasm build succeeds). Acceptance criteria Status:
   - [x] `tiles_to_pattern` helper + 2 tests
   - [x] web-sys features expanded
   - [x] `init()` + per-event re-query read helpers
   - [x] `main()` wires `visualizer::init()`
   - [x] console-log verification of read inputs
   - [ ] Solve + rendering (Step 2)

#### Step 2: Simplest rendering (planned)
- In the same handler: call `solve()`, render `<div>` per word into `#results`; on `Err`, clear + `console::error`. This resolves the solver `dead_code` warnings.

#### Step 2: Complete
- `handle_input` now calls `solver::solve(&correct, &misplaced_refs, &excluded)`.
- `Ok(words)` → `render_results` writes one `<div>` per word into `#results` via `set_inner_html`.
- `Err(e)` → `clear_results` + `console::error_1` with the error message.
- `tiles_to_pattern` generalized by user to `fn tiles_to_pattern<T: AsRef<str>>(&[T]) -> String` (removes verbose `Vec<String>`→`&[&str]` conversion at read sites).
- Solver `dead_code` warnings resolved. Only remaining warning: pre-existing `WORD_COUNT`.
- 64 tests pass; wasm build succeeds.
