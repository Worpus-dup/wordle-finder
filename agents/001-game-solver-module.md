## Task: Implement Game Solver Module

### Scope
- Create build script to generate word bank from data file
- Implement input validation with `SolverError` enum
- Implement word filtering (correct letters, misplaced letters, excluded letters)
- Implement result ranking by letter frequency
- Expose public `solve()` API function

### Acceptance Criteria
- [ ] `solve()` accepts correct_letters, misplaced_letters, excluded_letters
- [ ] Filtering matches letters at exact positions
- [ ] Filtering excludes words with excluded letters
- [ ] Filtering handles misplaced letters (letter exists but wrong position)
- [ ] Case-insensitive input handling
- [ ] Returns empty vec when no words match
- [ ] Results ranked by most common not-yet-guessed letters
- [ ] Input validation rejects non-ASCII, wrong length strings
- [ ] Unit tests cover all filtering edge cases
- [ ] Unit tests cover validation error cases

### Implementation Order (TDD)

**Step 1: Build Script & Word Bank**
- Create `build.rs` to read `data/answers/wordle-answers-alphabetical.txt`
- Generate `src/words.rs` with `WORD_COUNT` and `WORDS` constants
- Add `println!("cargo:rerun-if-changed=data/");` for rebuild triggers
- Update DESIGN.md: note word count may change with game updates

**Step 2: Error Types**
- Create `src/solver/error.rs` with `SolverError` enum
- Variants: `InvalidCharacter(char)`, `InvalidLength(usize)`, `EmptyInput`

**Step 3: Input Validation**
- Create `src/solver/validator.rs`
- Write tests first for: non-ASCII rejection, length validation, case normalization

**Step 4: Word Filtering**
- Create `src/solver/filter.rs`
- Write tests first for: exact position match, misplaced letter match, excluded letter exclusion
- Algorithm: iterate words, check each constraint

**Step 5: Result Ranking**
- Create `src/solver/rank.rs`
- Write tests first for: frequency-based sorting
- Algorithm: count letter frequency across remaining words, score each word

**Step 6: Public API**
- Create `src/solver/mod.rs` exposing `solve()` function
- Wire together validator → filter → rank

### File Structure
```
wordle-finder/
├── build.rs                    # Generates words.rs from data
├── src/
│   ├── main.rs                 # (existing, unchanged)
│   ├── words.rs                # Generated word bank constant
│   └── solver/
│       ├── mod.rs              # Public solve() API
│       ├── error.rs            # SolverError enum
│       ├── validator.rs        # Input validation
│       ├── filter.rs           # Word filtering logic
│       └── rank.rs             # Result ranking
├── data/
│   └── answers/
│       └── wordle-answers-alphabetical.txt  # Source word list
└── tests/
    └── solver_tests.rs         # Integration tests (optional)
```

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

**1. Error Types (Step 2)**
- Original: `EmptyInput` variant
- Actual: `EmptyInputs` variant with message "All inputs are empty"
- Rationale: More descriptive name when all three inputs are empty

**2. Unknown Position Constant**
- Original: Not specified
- Actual: `UNKNOWN` constant (`' '`) in `validator.rs` for unknown letter positions
- Rationale: Named constant improves readability over magic character

**3. Input Validation (Step 3)**
- Original: Focus on non-ASCII rejection
- Actual: Added `is_correct_empty()` and `all_empty()` helpers
- Added `validate()` returns `Result<(String, Vec<String>, String), SolverError>` (owned types)
- Empty check moved into `validate()` per user request for cleaner `solve()` API

**4. Filter/Filter API**
- Original: Not specified
- Actual: `filter(words: &[&str], correct: &str, misplaced: &[&str], excluded: &str) -> Vec<&str>`
- Misplaced letters use per-pattern duplicate counting (not cross-pattern)

**5. Rank API**
- Original: "count letter frequency across remaining words, score each word"
- Actual: Pre-computed `LETTER_FREQ: [u32; 26]` from build script
- Added `max_results: usize` parameter to limit output (default 100)
- Rationale: Avoids recomputing frequency on every call; build-time optimization

**6. Build Script Enhancement**
- Original: Generate `WORD_COUNT` and `WORDS`
- Actual: Also generates `LETTER_FREQ` array from full word bank
- Added `words_as_strs()` helper in `mod.rs` to convert `WORDS: &[u8]` to `Vec<&str>`

**7. Public API (Step 6)**
- Original: "Wire together validator → filter → rank"
- Actual: Added `misplaced_refs: Vec<&str>` conversion for type compatibility
- Returns `Result<Vec<String>, SolverError>` (owned Strings for WASM interop)

## Bugs

**1. `all_empty` Whitespace-only misplaced letters**
- When misplaced letters were presented as array of correctly "empty" strings it was tipping off the `misplaced.is_empty()` part.
- Fix: use of `is_correct_empty` for misplaced letters as well `misplaced.iter().cloned().all(is_correct_empty)`.
- Added test case for this situation.
