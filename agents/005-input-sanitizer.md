## Task: Input Sanitizer

### Scope
- Extract per-letter validation logic in the solver into a public `sanitize_letter()` helper.
- Sanitize user inputs in the visualizer (lowercase + drop invalid characters) before solving.
- Show a user-facing `#error` element for any solver error (currently the critical case is all-inputs-empty).
- Add/update tests: new `sanitize_letter` tests; remove duplicate `validate_correct`/`validate_excluded` tests.

### Acceptance Criteria
- [x] `sanitize_letter(letter: char, ignore_placeholder: bool) -> Result<char, SolverError>` is public in `src/solver/validator.rs`
- [x] `validate_correct` and `validate_excluded` use `sanitize_letter` (per-letter logic removed from both)
- [x] Visualizer lowers cases and drops invalid characters using `sanitize_letter`
- [x] Tile inputs: invalid/non-alpha char treated as placeholder (empty); uppercase lowered
- [x] Excluded input: non-alpha chars dropped; uppercase lowered
- [x] `#error` element added before `#results` in `index.html`, hidden by default
- [x] Visualizer shows `#error` for **any** `solve()` error, with the error message; hides on success
- [x] `#error` styled using existing palette (`--color-absent` or `--color-misplaced`)
- [x] `cargo test` passes (updated test suite)
- [x] Manual `trunk serve` check passes

### Implementation Order (TDD)

**Step 1: Extract `sanitize_letter` + update tests (`src/solver/validator.rs`)**
- Add public function:
  ```rust
  pub fn sanitize_letter(letter: char, ignore_placeholder: bool) -> Result<char, SolverError> {
      if ignore_placeholder && letter == UNKNOWN {
          Ok(UNKNOWN)
      } else if letter.is_ascii_lowercase() {
          Ok(letter)
      } else if letter.is_ascii_uppercase() {
          Ok(letter.to_ascii_lowercase())
      } else {
          Err(SolverError::InvalidCharacter(letter))
      }
  }
  ```
- Refactor `validate_correct`: per-letter loop uses `sanitize_letter(c, true)?`
- Refactor `validate_excluded`: per-letter loop uses `sanitize_letter(c, false)?`
- `validate_misplaced` already delegates to `validate_correct` (no change)
- Add `sanitize_letter` tests (lowercase, uppercase, placeholder both flags, invalid both flags, unicode)
- Remove duplicate tests (per-letter logic now covered by `sanitize_letter`):
  - `test_validate_correct_lowercase`, `_uppercase`, `_mixed_case`, `_invalid_character`, `_invalid_unicode`
  - `test_validate_excluded_valid`, `_uppercase`, `_invalid_character`
- Keep: `test_validate_correct_with_unknown` (placeholder preservation), length tests, misplaced delegation tests, all_empty/is_correct_empty tests, multiple-errors test.

**Step 2: Visualizer sanitization (`src/visualizer.rs`)**
- Add pure helper `sanitize_tile(value: &str) -> char`:
  - `sanitize_letter(value.chars().next().unwrap_or(UNKNOWN), true).unwrap_or(UNKNOWN)`
  - uppercase lowered; invalid → placeholder
- Add pure helper `sanitize_excluded(input: &str) -> String`:
  - iterate chars; keep `Ok` results; drop `Err` (invalid) chars
- Wire into `read_correct`, `read_misplaced` (via tiles), `read_excluded`
- Unit tests for both new helpers

**Step 3: Error display (`index.html`, `static/style.css`, `src/visualizer.rs`)**
- `index.html`: add `<div id="error" hidden></div>` in `#outputs`, before `#results`
- `static/style.css`: style `#error` (e.g. `color: var(--color-misplaced)` or a visible value from palette)
- `src/visualizer.rs`: in `handle_input`, on `solve()` `Err(e)` → set `#error` text to `e.to_string()`, remove `hidden`; on `Ok` → hide `#error` (and render results)

**Step 4: Verification**
- `cargo test`
- `trunk build`
- `trunk serve` manual check: non-alpha in tiles/excluded handled gracefully; clearing all inputs shows `#error`

### Known Patterns
- Reuse `solver::validator::UNKNOWN`
- Keep pure logic unit-testable (no DOM in tests)
- Guard missing DOM nodes with `Option`/early-return, not `expect`
- General `#error` handling (works for any future `SolverError`), not special-cased to `EmptyInputs`

### Scope Limits
- Do NOT add result styling (separate task)
- Do NOT rewrite keyboard-navigation JS in Rust (separate task)
- Do NOT add `#error` styling beyond a minimal visible style
- Escalate if web-sys API mismatch or DOM query issues

### Verification
```bash
cargo test           # updated suite passes
cargo build          # compiles
trunk serve          # manual: sanitize + #error display
```

### File Structure
```
wordle-finder/
├── src/
│   ├── solver/
│   │   └── validator.rs      # extract sanitize_letter + update tests
│   ├── visualizer.rs         # sanitize + error display + tests
│   ├── main.rs               # (unchanged)
│   └── words.rs              # (unchanged)
├── index.html                # add #error element
├── static/
│   └── style.css             # #error styling
└── agents/
    └── 005-input-sanitizer.md  # this task
```

### Implementation Details and Deviations

<!-- Written after completing each step -->

#### Step 1: Extract `sanitize_letter` + update tests (COMPLETE)
- Added public `sanitize_letter`; refactored `validate_correct` (uses `true`) and `validate_excluded` (uses `false`); `validate_misplaced` unchanged.
- **Deviation**: `sanitize_letter` tests use a parameterized `macro_rules!` (`sanitize_letter_case!`) with full identifier at call sites. Initial attempt `fn test_sanitize_letter_$name()` failed — identifier concatenation is not stable (needs nightly `concat_idents!`). Discussed with user; resolved by writing the full `test_sanitize_letter_*` ident at each call site.
- All 64 tests pass; `cargo build` succeeds (only pre-existing `WORD_COUNT` warning).

#### Step 2: Visualizer sanitization (COMPLETE)
- Added `sanitize_tile(value: &str) -> char` (lowercase; invalid/empty → `UNKNOWN`) and `sanitize_excluded(input: &str) -> String` (keep `Ok` chars, drop invalid).
- **Deviation (per user)**: `sanitize_tile` was folded into `tiles_to_pattern` (which now maps via `sanitize_tile`), rather than applied as a separate step before building patterns. Existing `tiles_to_pattern` tests unchanged/passing.
- `read_excluded` now returns `sanitize_excluded(&value)`.
- **Deviation (per user)**: the new helper tests use plain `#[test]` functions (not the parameterized macro) — macro reuse deferred.
- 72 tests pass; `cargo build` succeeds (only pre-existing `WORD_COUNT` warning).

#### Step 3: Error display (COMPLETE)
- `index.html`: added `<div id="error" hidden></div>` in `#outputs`, before `#results`.
- `static/style.css`: styled `#error` with `color: var(--color-misplaced)`, `font-weight: bold`, `margin-bottom: 0.5rem`.
- `src/visualizer.rs`: `handle_input` now calls `hide_error` on `Ok` and `show_error` on `Err`. Added helpers:
  - `show_error`: sets `#error` text via `set_text_content` + `remove_attribute("hidden")`
  - `hide_error`: `set_text_content(None)` + `set_attribute("hidden", "")`
- Uses `set_text_content` (safe against HTML injection) and the semantic `hidden` attribute.
- 72 tests pass; `trunk build` succeeds.

#### Step 4: Verification (COMPLETE)
- `cargo test` passes (72 tests).
- **Manual `trunk serve` check passed (user-confirmed)**:
  - Uppercase tiles treated as lowercase; non-alpha tiles treated as placeholder
  - Excluded field lowercased, invalid chars dropped
  - Clearing all inputs shows `#error` ("All inputs are empty"); adding input back hides it and renders results
  - Correct/misplaced filtering + ranking, add/remove rows, keyboard navigation all intact

#### Task complete
- All acceptance criteria met. 72 tests passing.
