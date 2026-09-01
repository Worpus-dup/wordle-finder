## Task: Code Quality Refactor

### Scope
- Fix all clippy warnings (9 total)
- Eliminate redundant allocations in solver hot path
- Improve `visualizer.rs` error handling with `?` syntax and `Option<()>` returns (side-effect functions only)
- Fix O(n²) `collect_guessed` in rank module using a single aggregate letter bitmask
- Remove unused code and dead allocations
- Add test parameterization for `validator.rs` (like `sanitize_letter` macro)

### Acceptance Criteria
- [ ] `cargo clippy` — zero warnings
- [ ] `cargo test` — all 72 tests pass (plus new parameterized tests)
- [ ] `solve()` no longer allocates `Vec<&str>` of 2315 words on every call (verified via code review)
- [ ] `visualizer.rs` side-effect functions return `Option<()>` with `?` propagation instead of early returns
- [ ] `collect_guessed` uses O(1) lookup via a single aggregate guessed bitmask
- [ ] Unused `WORD_COUNT` constant marked `#[allow(dead_code)]` (kept until project restructure uses it)
- [ ] Console logging in visualizer gated behind `cfg(debug_assertions)`
- [ ] No behavioral changes — refactor only

### Implementation Order (TDD)

**Step 1: Clippy Fixes (Mechanical)**
- [ ] Mark `WORD_COUNT` in `src/words.rs` with `#[allow(dead_code)]` (kept until project restructure; do NOT remove it or edit `build.rs`, since `build.rs` regenerates it)
- [ ] Fix needless borrow in `build.rs:57`
- [ ] Replace `sort_by` with `sort_by_key` in `src/solver/rank.rs:15`
- [ ] Remove `let _ = row.remove()` in `src/visualizer.rs:83`
- [ ] Collapse 6 collapsible `if` statements in `src/visualizer.rs` using `&& let`

**Step 2: Solver Hot Path Allocations**
- [ ] Cache word list as `&'static [&'static str]` via `OnceLock` (std, no new deps)

> **Note (deferred):** The planned validator `Cow<'a, str>` change (avoiding the owned `Vec<String>` produced by `validate`) is **out of scope** for now and deferred for later rework. `validate` keeps returning `(String, Vec<String>, String)` for now.

**Step 3: `collect_guessed` O(1) via aggregate letter bitmask**
- [x] Refactor `rank.rs::collect_guessed` → `collect_guessed_mask` to build a single aggregate `WordBitmask` instead of `Vec<char>` (O(1) `contains` via bit test)
- [x] Refactor `score_word` to use the bitmask (`WordBitmask::contains`)
- [x] Verify ranking behavior unchanged (existing rank tests + new tests pass)

> **Note (deferred):** A parallel per-word `WORD_MASKS: &[u32]` array (per DESIGN.md) is **out of scope** for this task and not implemented here. `filter.rs` keeps its current behavior. Paired with the upcoming project restructure.

**Step 4: Visualizer Error Handling Refactor**
- [x] Change side-effect functions to return `Option<()>`: `add_row`, `remove_row`, `clear_row_inputs`, `auto_advance`, `backspace_navigate`, `handle_input`, `handle_click`
- [x] Use `?` for early returns instead of nested `if let`/`match`
- [x] Keep `Closure::forget()` pattern (WASM requirement) but document why
- [x] **Explicitly out of scope**: data-producing functions `read_correct`, `read_misplaced`, `read_excluded`, `collect_tile_values` are unchanged — they keep returning their data values (`String`/`Vec<String>`)
- [x] **Deviation**: `clear_all` kept as `()` (no `?` benefit — all lookups are best-effort tolerants)

**Step 5: Test Parameterization**
- [x] Convert `validator.rs` `test_validate_*` tests to parameterized macros (`validate_error!`, `validate_ok!`)
- [x] Evaluate `visualizer.rs` tests — deferred/not parameterized (out of scope this task; pure helpers left as-is)

**Step 6: Misc Cleanup**
- [x] Remove unused `positions` vec in `src/solver/filter.rs:53`
- [x] Gate `console::log_3` in `handle_input` behind `cfg(debug_assertions)`
- [x] Document `Closure::forget()` memory leak as intentional for WASM lifecycle (done in Step 4)

### Known Patterns
- All solver inputs are lowercase, 5 chars, space = unknown
- Word bank: 2315 words, 5 bytes each, continuous in `WORDS: &[u8]`
- Letter frequency in `LETTER_FREQ: [u32; 26]` (precomputed at build)
- **In scope**: single aggregate guessed `u32` bitmask in `rank.rs` for O(1) `contains`
- **Out of scope / deferred**: per-word parallel `WORD_MASKS: &[u32]` array (see Step 3 note)
- Visualizer uses `web-sys` DOM API directly, no framework

### Scope Limits
- Do NOT change solver public API (`solve()` signature)
- Do NOT change visualizer public API (`init()` signature)
- Do NOT add new dependencies (use `std` only; `OnceLock` available in Rust 1.70+)
- Do NOT change `filter.rs` behavior (bitmask-based filter optimization out of scope here; only `rank.rs` aggregate-mask change is in scope)
- Escalate if any test fails — behavioral change detected

### Verification
```bash
cargo test           # All unit tests pass (including new parameterized tests)
cargo clippy         # Zero warnings
cargo build --release  # Build succeeds
```

### Implementation Details and Deviations

#### Step 1 — Clippy Fixes (COMPLETE)

1. **`cargo clippy --test target` deviation** — The task's "9 total" clippy count was for the **main bin** only (`cargo clippy`). Adopting `cargo clippy --all-targets` as the standard surfaced an additional **11 pre-existing warnings in the `filter.rs` test module** (lines 72-144) that were never part of the 9-count:
   - 10× `needless_borrow`: `filter(&WORDS, ...)` → `filter(WORDS, ...)` in test assertions.
   - 1× `clippy::iter_nth_zero`: `word.chars().nth(0)` → `word.chars().next()`.
   - Resolved via `cargo clippy --fix --tests` (user-applied). All are test-only cosmetics, no production behavior change.
2. **1a template whitespace** — Fixing `#[allow(dead_code)]` inline on the same template line caused the regenerated `words.rs` to leave leading whitespace before `pub const WORD_COUNT`. Corrected the `build.rs` template to use a line continuation (`\n\`) so the regenerated output stays clean (`#[allow(dead_code)]\npub const WORD_COUNT ...`).
3. **1a persistence confirmed** — `cargo build` regenerates `src/words.rs` and the `#[allow(dead_code)]` attribute survives (verified in regenerated output).

**Step 1 verification:** `cargo clippy --all-targets` → zero warnings; `cargo test` → 72 passed; `cargo build --release` → succeeds.

#### Step 2 — Solver Hot Path Allocations (COMPLETE)

1. **`Cow` change dropped / deferred** — The task planned changing `validate` to return `Cow<'a, str>` to avoid the owned `Vec<String>`. On review this was deferred entirely (user decision); `validate` still returns `(String, Vec<String>, String)`. Updated Step 2 plan + Known Patterns note. Scope of Step 2 reduced to the cache only.
2. **`words_as_strs()` → cached `words()`** — Replaced the per-call allocation of a 2315-element `Vec<&str>` with `OnceLock<Vec<&'static str>>` (`WORDS_CACHE`), using `std::sync::OnceLock` (no new deps). `solve()` now calls `words()` returning `&'static [&'static str]`, initialized once. The temporary `misplaced_refs: Vec<&str>` (needed by `filter`/`rank`'s `&[&str]` API) remains unchanged for now.
3. **New test added (TDD)** — `test_words_cache_returns_same_backing` in `mod.rs`: asserts `words()` returns the same backing slice on consecutive calls (`as_ptr()` equality), verifying the cache actually avoids re-allocation. Behavior guard via existing solve tests.

**Step 2 verification:** `cargo clippy --all-targets` → zero warnings; `cargo test` → 73 passed (72 + new cache test); `cargo build --release` → succeeds.

#### Step 3 — `collect_guessed` O(1) via aggregate letter bitmask (COMPLETE)

1. **New type `WordBitmask`** (instead of raw `u32` arithmetic) — created `src/solver/bitmask.rs`: a newtype wrapper over `u32` with `const fn new()`/`push()`/`contains()` and a `FromStr` impl (`from_str` builds a mask, ignoring non-lowercase chars). Registered as `pub mod bitmask` in `mod.rs`. Design decision with user: prefer a readable type over scattered `1 << (c - b'a')` bit arithmetic.
   - `push`/`contains` silently ignore non-lowercase (replacing the old explicit `c != UNKNOWN` guards); since input is validated lowercase, behavior is equivalent.
   - `FromStr` (trait) chosen over an inherent `const fn from_str` — const string iteration is awkward; `FromStr` is idiomatic (`"a".parse()`) and internal `push`/`new`/`contains` remain `const`.
2. **`collect_guessed` → `collect_guessed_mask` returning `WordBitmask`** — no more `Vec<char>` + `Vec::contains` (was O(n²)); O(1) via bit test. `correct.chars().chain(excluded.chars())` folded into one pass (safe because `push` ignores non-letters; the old correct/misplaced-skip-UNKNOWN vs excluded-no-skip distinction is now moot).
3. **`score_word(word, guessed: WordBitmask)`** — takes the mask by value (`Copy`); `filter(|&c| !guessed.contains(c))`.
4. **Test call-site churn** — the 3 existing rank tests called `score_word(..., &['a'])`/`&['e'])` with the old slice signature; updated to `score_word(word, "a"/"e".parse().unwrap())`.
5. **New tests** — 8 in `bitmask.rs` (new/push/contains/from_str/dedup/non-lowercase-ignored) + `test_collect_guessed_mask` in `rank.rs` (`"a  b" + [" c"] + "d"` → contains a,b,c,d, not e).

**Step 3 verification:** `cargo clippy --all-targets` → zero warnings; `cargo test` → 82 passed (73 + 8 bitmask + 1 mask test); `cargo build --release` → succeeds.

#### Step 4 — Visualizer Error Handling Refactor (COMPLETE)

1. **`?` works directly on `Option`; `.ok()?` for `Result`** — web-sys `get_element_by_id`/`event.target()`/`first_element_child` return `Option` → `?` directly; `query_selector`/`clone_node_with_deep`/`dyn_into` return `Result` → `.ok()?`.
2. **Scope deviation — `clear_all` left as `()`** — all of its lookups are best-effort/tolerant (`if let`), with no hard-required element. Converting to `?` would change behavior (abort on the first missing element instead of continuing to clear what it can). Kept as `()`. Documented in plan.
3. **`handle_click` also converted to `Option<()>`** (user decision) — its first two assignments (`event.target()?`, `dyn_into().ok()?`) mirror the converted style; always returns `Some(())`.
4. **`handle_input` → `Option<()>` for API uniformity** — has no early-return `?` sites (body is `read_*` + `match solve()`); returns `Some(())` at the end.
5. **Data producers untouched** — `read_correct`, `read_misplaced`, `read_excluded`, `collect_tile_values` remain `String`/`Vec<String>` (out of scope); `get_tile_inputs` also unchanged.
6. **Call sites** — event closures in `init()` now `let _ = fn(...)`; `add_row`/`remove_row` use `let _ = clear_row_inputs(...)` / `let _ = handle_input(...)` to preserve tolerant continue-on-failure semantics.
7. **`Closure::forget()` documented** — added an explanatory comment on `input_closure.forget()` noting the intentional lifetime leak required by the wasm-bindgen `Closure` contract.
8. **No new tests** — these are DOM/web-sys functions not unit-testable in `cargo test`; behavioral guarantee is compile + existing 82 tests + manual `trunk serve` check (as in Task 006).

**Step 4 verification:** `cargo clippy --all-targets` → zero warnings; `cargo test` → 82 passed (unchanged); `cargo build --release` → succeeds; `trunk build --release` → succeeds. Manual browser check (via `trunk serve`) recommended to confirm no behavioral change.

#### Step 5 — Test Parameterization (COMPLETE)

1. **Two macros, not one** — the `test_validate_*` tests were NOT uniform (they span 3 shapes: success-tuple-destructuring, error-type assertion, plain `is_ok`), so a single macro wouldn't fit. Added `validate_error!` (asserts `.unwrap_err()` equals an expected `SolverError`) and `validate_ok!` (asserts `.is_ok()`), mirroring the existing `sanitize_letter_case!` macro style.
2. **Converted via macros:** 6 error tests → `validate_error!`; 3 `is_ok` tests → `validate_ok!`.
3. **Group A kept as plain `#[test]`** — the success-tuple-destructuring tests (`correct_with_unknown`, `misplaced_valid`, `misplaced_uppercase`) destructure different tuple fields, so parameterization wouldn't help; left unchanged.
4. **`is_correct_empty`/`all_empty` left out** — not `test_validate_*` (out of the plan's stated scope); left as plain tests (user decision).
5. **`visualizer.rs` not parameterized** — evaluated post-Step-4; its pure helpers (`tiles_to_pattern`, `sanitize_tile`, `sanitize_excluded`) are uniform and could parameterize, but left as-is by user decision.
6. **No test count change** — each macro invocation generates one `#[test]` replacing an existing plain one 1:1. Count stays 82.

**Step 5 verification:** `cargo clippy --all-targets` → zero warnings; `cargo test` → 82 passed (unchanged); `cargo build --release` → succeeds.

#### Step 6 — Misc Cleanup (COMPLETE)

1. **Removed dead `positions` vec** (`src/solver/filter.rs`) — `positions: Vec<(usize, char)>` was allocated and pushed to in `matches_misplaced` but never read; it was silenced by `let _ = positions;`. Removed the declaration, the `positions.push(...)`, and the `let _ = positions;`, eliminating a per-pattern Vec allocation in the filter hot path. `j` stays (still used by the `word.chars().nth(j)` mismatch check).
2. **Gated `console::log_3`** (`src/visualizer.rs`) — wrapped the `handle_input` debug logging in `#[cfg(debug_assertions)]` so production/release builds emit no console logging. `console::error_1` (the error-arm diagnostic) left ungated, per plan.
3. **`Closure::forget()` documentation** — already completed in Step 4 (explanatory comment on the `input_closure.forget()` site); confirmed present. No new change this step.

**Step 6 verification:** `cargo clippy --all-targets` → zero warnings (after removing `positions`, no dead-code warning returned); `cargo test` → 82 passed (unchanged); `cargo build --release` → succeeds.

---

## Code Quality Issues (from code review)

### Clippy Warnings (9 total)

| Location | Issue | Fix |
|----------|-------|-----|
| `src/words.rs:4` | Unused constant `WORD_COUNT` | Mark `#[allow(dead_code)]` (kept for upcoming restructure; `build.rs` regenerates it) |
| `build.rs:57` | Needless borrow `&dest_path` | Use `dest_path` directly |
| `src/solver/rank.rs:15` | Use `sort_by_key` instead of `sort_by` | `scored.sort_by_key(|b| std::cmp::Reverse(b.1))` |
| `src/visualizer.rs:83` | Unnecessary `let _ = row.remove()` | Just `row.remove()` |
| `src/visualizer.rs:94,111,124,246,257,271` | Collapsible `if` statements | Use `&& let` pattern |

### Logic/Performance Issues

1. **`src/solver/filter.rs:53`** - Unused `positions` vector allocated in loop
2. **`src/solver/mod.rs:24-29`** - `words_as_strs()` allocates new `Vec<&str>` (2315 elements) on every `solve()` call — should cache or use lazy static
3. **`src/solver/mod.rs:17-18`** - Redundant conversion: validator returns `Vec<String>`, then converted to `Vec<&str>` — avoid allocation by changing return type or using references
4. **`src/solver/validator.rs:34-36`** - `validate_correct` always allocates new `String` — could use `Cow<str>` or return `&str` when already valid
5. **`src/solver/rank.rs:19-39`** - `collect_guessed` uses `Vec::contains` in loop (O(n²)) — replace with a single aggregate `u32` guessed bitmask for O(1) lookup. (Per-word `WORD_MASKS` deferred — see Step 3 note.)
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
- **New**: Parameterized tests for `validator.rs` (Step 5)
- **Deferred**: Parameterized tests for `visualizer.rs` (after Step 4)