## Task: Implement Game Solver Module

### Scope
- Create build script to generate word bank from data file
- Implement input validation with `SolverError` enum
- Implement result ranking by letter frequency
- Expose public `solve()` API function

### Acceptance Criteria
- [ ] `solve()` accepts correct_letters, misplaced_letters, excluded_letters
- [ ] Filtering matches letters at exact positions
- [ ] Case-insensitive input handling
- [ ] Returns empty vec when no words match
- [ ] Input validation rejects non-ASCII, wrong length strings
- [ ] Unit tests cover all filtering edge cases

### Implementation Order (TDD)

**Step 1: Build Script & Word Bank**
- Create `build.rs` to read `data/answers/wordle-answers-alphabetical.txt`
- …

**Step 2: Error Types**
- Create `src/solver/error.rs` with `SolverError` enum
- …

### Known Patterns
- All words lowercase, 5 chars each
- Bitmask representation for letter sets (u32, 26 bits)
- `correct_letters`: use space for unknown positions (e.g., "a  p l e")
- `misplaced_letters`: array of 5-char strings with letter+position info
- `excluded_letters`: string of letters to exclude

### Scope Limits
- Do NOT implement WASM bindings yet (future task)
- Do NOT implement UI/visualizer (future task)
- Do NOT optimize for performance yet (baseline brute-force only)
- Escalate if word bank format differs from expected

### Verification
```bash
cargo test           # All unit tests pass
cargo clippy         # No warnings
cargo build          # Build script generates words.rs
```

### Implementation Details and Deviations

<!-- Written after completing the step -->
1. **`tiles_to_pattern` signature** changed from `&[&str; 5]` to `&[&str]`.
   - Reason: avoids the temporary-borrow compile issue when collecting `HtmlInputElement::value()` results (`String`) into an array of references.
   - Caller builds a `Vec<String>` of raw values, then derives `Vec<&str>` via `iter().map(String::as_str)` before calling the helper.
