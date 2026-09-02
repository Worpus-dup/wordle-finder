## Task: Input Sanitization UI Feedback

### Scope
- Make input sanitization **visible** to the user instead of fully silent
- Gate tile navigation on *valid* letter input
- Run `solve` conditionally (only when the new symbol was accepted)
- Reuse the existing `#error` element and `SolverError` Display as the message channel

### Background (from `agents/007-refactor-draft.md`)
The draft recorded the two-layer validation model (visualizer sanitizes-and-replaces; solver validates defensively). DESIGN.md already documents that model. What is missing is **UI feedback**: today sanitization is entirely silent — an invalid character is silently mapped/dropped, `auto_advance` still advances (field is non-empty), and `solve` still runs, so the user never sees that their input was rejected.

### Acceptance Criteria
- [ ] Valid letter (A-Z) typed into a tile: normalized to lowercase, advances to the next tile, triggers a `solve()` recompute
- [ ] Whitespace (`" "`) typed into a tile: treated as blank-like `Space` (matches current placeholder semantics), field cleared to `""`, advances to the next tile, triggers a `solve()` recompute
- [ ] Invalid character (non-ASCII / symbol) typed into a tile: field is cleared, a message is shown, **no** navigation advance, **no** `solve()` call
- [ ] Multi-char paste into a single tile keeps the first char only; treated as valid/space/invalid by that char
- [ ] Backspace navigation unchanged (moves to previous tile when current tile is empty)
- [ ] Excluded-letters field keeps dropping invalid chars, but the drop is now reflected with a message
- [ ] Message (sanitization or solver error) clears on the next valid/accepted input event (inherited from existing `hide_error` on successful `solve`)
- [ ] `#error` used for both solver errors and sanitization messages (single channel)
- [ ] `SolverError` reused to render the message (no new message strings/parallel system)
- [ ] Solver module (`validate`/`solve`) unchanged — stays defensive
- [ ] `cargo test` green (82 existing + new unit tests for pure helpers), clippy zero warnings, release build succeeds
- [x] DESIGN.md `### User Input Validation` verified/updated to describe the visible-sanitization behavior

### Implementation Order (TDD where possible)

**Step 1: Pure classification helper (testable, TDD)**
- Add a DOM-free helper that classifies a tile input string:
  ```rust
  enum TileValue { Empty, Space, Valid(char), Invalid(char) }
  fn classify_tile(value: &str) -> TileValue
  ```
  - Empty string `""` → `Empty`
  - Single space `" "` → `Space` (matches current semantics: a space is treated as a placeholder/blank, not an error)
  - Single valid letter (reuse `sanitize_letter(c, false)` semantics: A-Z/a-z, no placeholder) → `Valid(lowercased)`
  - Other → `Invalid(first_char)` — for multi-char paste, consider the **first char only** (covers the existing `sanitize_tile` `.chars().next()` behavior); classify by that char (valid/space/invalid)
- Unit tests: valid/uppercase-lowered/empty/space/non-ASCII/symbol/multi-char-paste

**Step 2: Rework tile input routing in `visualizer.rs`**
- Replace the unconditional `handle_input` + `auto_advance` pair on the `input` listener with a routed handler:
  - `Valid(ch)` → lowercase-normalize the field value in the DOM, call the advance/focus-next step, then run `handle_input` (existing read + `solve()` path)
  - `Space` → clear the field value to `""`, call the advance/focus-next step, then run `handle_input` (recompute) — overwriting a filled tile with a space correctly blanks it
  - `Empty` → run `handle_input` (recompute; no advance) — preserves current behavior when a tile is cleared via backspace/delete
  - `Invalid(ch)` → clear the field value, show message via `SolverError::InvalidCharacter(ch)` using the existing `show_error` path, do **not** advance, do **not** run `solve`
- Message lifetime is inherited, not a separate step: the next successful `solve` already calls `hide_error` (`visualizer.rs` `Ok` branch), so a sanitization message clears on the next valid input
- Keep `backspace_navigate` unchanged (separate `keydown` listener)

**Step 3: Excluded-letters sanitization feedback**
- When the excluded field's raw value would drop characters, reflect it: sanitize the field's displayed value (or detect raw≠sanitized) and show a message with `SolverError::InvalidCharacter` for the first dropped char
- (Open to refinement during implementation — see decisions below)

**Step 4: DESIGN.md**
- Verify `### User Input Validation`; update wording so it reflects that sanitization is user-visible (message shown, invalid input blocked) rather than silent replace

### Known Patterns
- Single message channel: `#error` (already styled with `"> "` prefix + underline) for both solver errors and sanitization messages
- `SolverError::InvalidCharacter(c)` Display text: `Invalid character: 'c'` — reused verbatim for the sanitization message
- `sanitize_letter(letter, ignore_placeholder)` in `solver::validator` remains the single validation primitive
- Event delegation via a single `document` listener (unchanged); tab/row navigation order via `get_tile_inputs` DOM order (unchanged)

### Scope Limits
- Do NOT modify the solver (`validate`/`solve`) — it stays defensive
- Do NOT add a separate warning element / parallel message system — reuse `#error` + `SolverError`
- Do NOT implement whole-paste distribution across tiles (first-char-only is the agreed behavior)
- Do NOT add timers / auto-hide delay (message clears on next valid input)
- **Accepted edge case:** overwriting a *filled* tile with an invalid char clears the tile but does not recompute results (no `solve` on invalid input); results stay stale until the next valid input. This is the intended tradeoff for the "no solve on invalid" rule.
- Escalate if clearer wording requires a new `SolverError` variant — prefer reusing existing variants over adding new ones

### Verification
```bash
cargo test              # 82 existing + new pure-helper tests
cargo clippy --all-targets   # zero warnings
cargo build --release   # succeeds
trunk serve             # manual checks:
# - Valid letter advances + lowercase + recomputes results
# - Space advances to the next tile, clears to "", recomputes results
# - Invalid char clears the tile, shows "Invalid character: '...'", does not advance, does not recompute
# - Paste of multiple chars keeps the first char
# - Message clears on the next valid input
# - Backspace navigation still works
# - Excluded field drop reflected with a message
```

### Implementation Details and Deviations

<!-- Written after completing the step -->
1. **Step 1 — `classify_tile` via test macro** — the 10 identical `classify_tile_case!` tests are generated by a small `macro_rules!` macro (same style as `sanitize_letter_case!` / `validate_*!` in the solver tests). `TileValue` derives `Debug, Clone, Copy, PartialEq, Eq`. `#[allow(dead_code)]` on `classify_tile` added temporarily (no production consumer until Step 2); removed once wired. Test count: 82 → **92** (10 new).
2. **Step 2 — routed input handler + renames** — `handle_input` was split into two functions:
   - `refresh_results(document)` — the old read-and-solve recompute (renamed; called by `remove_row` and the routed handler).
   - `handle_input(document, event)` — the new routed `input`-event handler: non-`.word-row` targets (excluded field) recompute as before; tiles are classified via `classify_tile` and routed per variant.
   - `auto_advance` replaced by `focus_next(document, target)` (drops the "value non-empty" guard, since `Space` must still advance after clearing to `""`); advance is now decided per variant in the routed handler.
   - `init()` input closure now calls only `handle_input(&doc, &event)`; `SolverError` import added to `visualizer.rs`.
   - `#[allow(dead_code)]` removed from `classify_tile` (now consumed here).
   - Tile detection via `target.closest(".word-row")`; fallback recompute if the check fails. Test count stays 92 (DOM handler not unit-testable, consistent with prior steps).
3. **Step 3 — excluded-letters feedback via `first_dropped_char`** — new pure helper `first_dropped_char(input) -> Option<char>` returns the first char `sanitize_letter(c, false)` rejects; tested with a `first_dropped_char_case!` macro (7 tests). The non-`.word-row` branch of `handle_input` now: sanitizes the field's displayed value in place (`set_value(sanitize_excluded(raw))` when different) and shows `SolverError::InvalidCharacter(first_dropped)` only when a char was actually dropped (uppercase → lowercase produces no message). `refresh_results` still reads through `read_excluded` (unchanged). Test count: 92 → **99**.
4. **Step 4 — DESIGN.md `### User Input Validation` rewrite + related sections** — user decision points: (1) document the stale-results edge case with a `Design note:` paragraph; (2) paste limited to first char via a scoped sentence ("As pasting behaviour is outside the current scope..."); (3) advance/blank behavior noted in the UI `### Tile Inputs` too (UI section carries logic descriptions) with a cross-ref to "User Input Validation"; (4) detail kept. Also updated: Visualizer `#### Algorithms and Processing Logic` (rejected input made visible; advance gated to valid/blank), `#### User-Facing Errors` (added bullet: sanitisation feedback reuses `#error` and does not clear results), UI `### Error Field` (dual purpose: cannot-solve AND rejected/dropped input). No Rust changes; no `cargo test` run.
5. **Bug fix — excluded-field sanitization message was erased in the same event (extension of this task)** — `refresh_results`'s solve-`Ok` branch calls `hide_error`, so a sanitization message shown for a dropped excluded char was cleared instantly (or overwritten by a solver error). Fix, per user decision: the excluded branch now runs `refresh_results` **first**, then shows the sanitization message only when `first_dropped_char` found a char **and** no error is currently visible — a let-chain guard `if let Some(c) = first_dropped_char(&raw) && !is_error_visible(document)` (edition 2024, rustc 1.88+). New helper `is_error_visible(document) -> bool` reads the `#error` `hidden` attribute (same toggle `show_error`/`hide_error` use); missing element → `false`. Solver errors therefore take precedence and are preserved. `let _ =` intentionally omitted from the `refresh_results(document);` call (clippy/silent, no must_use warning). Test count stays 99 (DOM logic not unit-testable, consistent with prior steps).