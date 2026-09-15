## Task: Conflict Diagnostics (K)

### Scope
Detect two contradictory-letter classes in `validate()` and surface them as
distinct `SolverError` variants instead of a generic "No words match these
clues":

- Excluded-vs-required: a letter in the excluded field is also a required
  letter (present in the correct row or any misplaced pattern).
- Correct-vs-misplaced: a misplaced pattern locks the same letter onto an index
  where the correct row already places it (green + yellow on the same square).

Kept distinct (per user): one message names the letter, the other names the
letter + position; merging them would lose the actionable difference.

### Acceptance Criteria
- [ ] `SolverError::ConflictingLetter(char)` + Display
      "Letter 'x' cannot be both required and excluded"
- [ ] `SolverError::ConflictingPosition(char, usize)` + Display
      "Letter 'x' cannot be both correct and misplaced at position N"
      (usize 0-based internally, message 1-based)
- [ ] `validate()` returns `ConflictingLetter` when an excluded letter is also
      required (correct or misplaced); precedence: field validity errors, then
      ConflictingLetter, then ConflictingPosition
- [ ] `validate()` returns `ConflictingPosition` when a misplaced pattern
      collides with correct at the same index
- [ ] No false positives: separate letters, required-in-both-correct-and-
      misplaced, same letter at different positions, all pass validation
- [ ] Existing errors/order unchanged (`test_validate_multiple_errors_first_error`)
- [ ] Unit tests in `error.rs` (Display) + `validator.rs` (conflicts & no-conflicts)
- [ ] No visualizer change required — existing `Err` path shows the message and
      clears results; benchmark unaffected (drives filter/rank directly)
- [ ] `cargo test --all-targets` green, `cargo clippy --all-targets` clean,
      wasm32 build ok, `trunk build` ok
- [ ] Manual browser check: green `a` + excluded `a` → letter error; green `a`
      at pos 1 + misplaced `a` in same row at pos 1 → position error; fixing
      either restores results

### Implementation Order (TDD)

**Step 1: `src/solver/error.rs`**
- Add `ConflictingLetter(char)` and `ConflictingPosition(char, usize)` to the
  enum; add Display arms; add Display tests.

**Step 2: `src/solver/validator.rs`**
- Tests first via existing `validate_error!` / `validate_ok!` macros:
  - ConflictingLetter: correct `"a    "` + excluded `"a"`; misplaced
    `["b    "]` + excluded `"b"`; excluded `"ab"` + correct `"a    "`
    (first match wins); excluded `"A"` + correct `"a    "` (case-unified).
  - ConflictingPosition: correct `"a    "` + misplaced `["a    "]` → pos 0;
    middle-index case; second-row case.
  - No-conflict: separate letters; same letter correct + misplaced at *other*
    indexes; required-in-both (no excluded overlap).
- Helpers:
  - `conflicting_letter(correct, misplaced, excluded) -> Option<char>`:
    required = letters in correct (non-space) ∪ letters in misplaced patterns;
    return first excluded letter present in required.
  - `conflicting_position(correct, misplaced) -> Option<(char, usize)>`:
    first (pattern index, correct index) where `correct[i] != UNKNOWN` and
    `pattern[i] == correct[i]`.
- Call both at the end of `validate()` (after field validation), in the stated
  precedence order.

**Step 3: Verification**
- `cargo test --all-targets`, `cargo clippy --all-targets`,
  `cargo build --target wasm32-unknown-unknown`, `trunk build`.
- Manual browser pass (see criteria).
- Update this file's Deviations section.

### Known Patterns
- `sanitize_letter` guarantees lowercase-ASCII letters (or `UNKNOWN`) post-
  validation, so helpers index `[c as u8 - b'a']` and compare chars directly.
- `validate_error!` macro pattern: `validate(correct, misplaced, excluded).unwrap_err()`.
- `SolverError` derives `Debug, PartialEq, Eq`; Display drives the `#error` text.

### Scope Limits
- No visualizer changes; no solver/filter/rank changes; benchmark untouched.
- Deferred (still out of scope): field-level hints on multiple simultaneous
  conflicts (first-wins report), gray-vs-excluded consistency checks.

### Verification
```bash
cargo test --all-targets
cargo clippy --all-targets
cargo build --target wasm32-unknown-unknown
trunk build
```

### Implementation Details and Deviations

**Step 1: `src/solver/error.rs`**
- Added `ConflictingLetter(char)` and `ConflictingPosition(char, usize)`.
- Display uses 1-based position in the message (`pos + 1`); the enum value stays
  0-based. Added two Display tests.

**Step 2: `src/solver/validator.rs`**
- `conflicting_letter`: required set = letters in `correct` (non-space) ∪ letters
  in all misplaced patterns (indexed into a `[bool; 26]` — safe post-validation,
  which guarantees lowercase-ASCII); returns the first `excluded` char present.
- `conflicting_position`: first `(pattern index, correct char)` where
  `correct[i] != UNKNOWN && pattern[i] == correct[i]`.
- Wired at the end of `validate()` in precedence order: field validity errors
  first, then `ConflictingLetter`, then `ConflictingPosition`.
- Deviation: the initial precedence test did not actually present both conflict
  classes simultaneously (`"a    "`` + `[" a   "]` + `"a"` — the pattern char
  lands on an empty correct slot). Corrected to `"a    "` + `["a    "]` + `"a"`
  so both letter- and position-conflicts exist and the error order is really
  exercised. Middle-index position-conflict case added as well.

**Step 3: Verification**
- `cargo test --all-targets`: 83 lib (71 → +8 validator conflict, +2 error
  Display) + 37 bin, all green. `cargo clippy --all-targets` zero warnings,
  wasm32 build ok, `trunk build` ok.
- No changes to `src/solver/filter.rs`, `rank.rs`, `mod.rs` solve flow,
  `src/visualizer.rs`, or the benchmark.
- Manual browser pass pending.