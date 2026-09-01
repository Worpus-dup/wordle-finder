## Task: Refactor Draft (DRAFT — for planning, not yet sanctioned)

> **Status**: DRAFT / placeholder. This document captures design notes and
> behavior clarifications that surfaced during the DESIGN.md review. It is **not**
> an approved implementation task. Details must be finalized and approved before
> any coding.

### Background (from DESIGN.md review)

During a point-by-point comparison of DESIGN.md against the implementation, the
user clarified a design/intent gap regarding input sanitization. This draft
records that decision so it is not lost and can be turned into a future task.

### Clarification: Two-layer input validation behavior

DESIGN.md currently describes invalid input in two places with different intent:

1. **User Input Validation (Data Design)** — *"Any non-ASCII letter character in
   user input should be treated as error and block any additional data
   manipulation."*
2. **Visualizer (Algorithms and Processing Logic)** — *"If any error detected in
   user inputs, sanitise incorrect data and replace the original."*

These appear contradictory. The user clarified the intent:

- Sentence (1) was written about the **solver module** layer.
- Sentence (2) describes the **visualizer** layer, and is the **preferred**
  end-user-facing behavior.

**Preferred behavior** (matches current implementation, Task 005):
- The **visualizer** sanitizes user input before calling the solver:
  - Tile inputs: non-alpha/invalid chars treated as placeholder (`' '`); uppercase lowered.
  - Excluded input: invalid chars dropped; uppercase lowered.
  - `#error` shown only for solver errors (notably all-inputs-empty), **not** for individual invalid characters.
- The **solver** still validates defensively and returns `SolverError::InvalidCharacter` on unsanitized input (remains a safety layer, but is not normally reached through the visualizer).

### Potential follow-up actions (not yet approved)
- Reword DESIGN.md's User Input Validation section to describe the two-layer
  model (visualizer sanitizes → solver validates defensively) instead of the
  single "error and block" framing.

### Open questions
- Whether any refactor beyond documenting/rewording DESIGN.md is desired.
- Whether `sanitize_tile` / `sanitize_excluded` should remain as-is.

### Verification
- N/A (draft only — no code changes yet).

---

## Code Quality Issues (from code review)

### Clippy Warnings (9 total)

| Location | Issue | Fix |
|----------|-------|-----|
| `src/words.rs:4` | Unused constant `WORD_COUNT` | Remove or use it |
| `build.rs:57` | Needless borrow `&dest_path` | Use `dest_path` directly |
| `src/solver/rank.rs:15` | Use `sort_by_key` instead of `sort_by` | `scored.sort_by_key(|b| std::cmp::Reverse(b.1))` |
| `src/visualizer.rs:83` | Unnecessary `let _ = row.remove()` | Just `row.remove()` |
| `src/visualizer.rs:94,111,124,246,257,271` | Collapsible `if` statements | Use `&& let` pattern |

### Logic/Performance Issues

1. **`src/solver/filter.rs:53`** - Unused `positions` vector allocated in loop
2. **`src/solver/mod.rs:24-29`** - `words_as_strs()` allocates new `Vec<&str>` (2315 elements) on every `solve()` call — should cache or use lazy static
3. **`src/solver/mod.rs:17-18`** - Redundant conversion: validator returns `Vec<String>`, then converted to `Vec<&str>` — avoid allocation by changing return type or using references
4. **`src/solver/validator.rs:34-36`** - `validate_correct` always allocates new `String` — could use `Cow<str>` or return `&str` when already valid
5. **`src/solver/rank.rs:19-39`** - `collect_guessed` uses `Vec::contains` in loop (O(n²)) — use `bool[26]` array or fixed-size bitset for O(1) lookups
6. **`src/visualizer.rs:183-187`** - Console logging in production code — should be behind `cfg(debug_assertions)` or feature flag

### Design/Architecture Concerns

1. **`src/words.rs`** - Word bank (~11KB) embedded as giant byte string in source — makes git diffs noisy, code review difficult. Consider keeping data separate (generated at build time is fine, but format could be more diff-friendly)
2. **Memory/Performance** - Every `solve()` call allocates multiple `Vec`s:
   - Creates `Vec<&str>` of all 2315 words
   - Filters creating new `Vec`
   - Ranks creating another `Vec`
   - Converts to `Vec<String>` for return
   Could optimize with streaming/iterator approach or object pooling
3. **`src/visualizer.rs`** - Event handlers use `Closure::forget()` which leaks memory (intentional for WASM lifecycle but worth documenting)

### Test Coverage Notes

- All 72 tests pass
- Tests are inline with modules (per DESIGN.md)
- No integration/WASM tests (per DESIGN.md: "not yet needed")
