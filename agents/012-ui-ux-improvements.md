## Task: UI/UX Improvements (A–F)

> Review process: this task is mostly visual. After each step, the user checks
> the result in the browser and gives feedback before the next step starts.

### Scope
- A: Add `noscript` fallback message (JS/WASM unavailable). Fixes unimplemented DESIGN.md "message for user without JS/WASM support" (DESIGN.md:40).
- B: Dedicated error color (`--color-error`) with accessible contrast; stop overloading the "misplaced" yellow for errors.
- C: Empty-state message ("No words match these clues") when `solve` returns `Ok([])`, and a result count ("N possible words") when it returns words.
- D: `aria-live="polite"` on `#error` and `#results` so screen readers announce live updates.
- E: Disable "+ Add Row" (with `disabled` styling) once the 5-row cap is reached; re-enable on row removal.
- F: Paste a full word into a tile row spreads letters across that row's 5 tiles (currently paste truncates to 1 char via `maxlength="1"`).

Out of scope / deferred: G (row-scoped aria-labels), H (hint line), I (chip display cap),
J (suggested-guess spotlight), K (conflict diagnostics).

### Acceptance Criteria
- [ ] A: `<noscript>` block present and styled; trunk build succeeds
- [ ] B: `#error` uses `--color-error`; contrast ≥ 4.5:1 against `--color-bg`
- [ ] C: empty result set shows a readable "no words match" message, distinct from hard errors; non-empty shows a count line above the chips. Pure logic unit-tested
- [ ] D: `aria-live="polite"` on `#error` and `#results`
- [ ] E: button disabled at 5 rows, re-enabled below; disabled style present; pure cap logic unit-tested
- [ ] F: pasting a word into any tile fills that row from the pasted tile onward; invalid/extra chars dropped; row cap respected; refresh runs. Pure spread logic unit-tested
- [ ] No JS added anywhere (Rust-only held)
- [ ] `cargo test` green, `cargo clippy --all-targets` zero warnings, `cargo build --target wasm32-unknown-unknown` ok, `trunk build` ok
- [ ] User browser-pass at each step

### Implementation Order (TDD)

**Step A: `noscript` fallback (`index.html`, `static/style.css`)**
- Add `<noscript>` immediately after `<body>` with a `.noscript` paragraph.
- Style `.noscript` with existing palette vars (absent background, white text, centered).
- Verify: source contains block; trunk build.
- **User review point.**

**Step B: error colour stop (`static/style.css`)**
- Add `--color-error` (dark red, contrast ≥4.5:1 on `--color-bg`) to `:root`.
- `#error { color: var(--color-error) }`; keep `> ` prefix + underline.
- Verify: contrast calculation; trunk build.
- **User review point.**

**Step C: empty state + result count (`src/visualizer.rs`, `static/style.css`)**
- Pure helper `result_summary(word_count) -> Option<&'static str>` (None when empty inputs → caller skips; "1 possible word" / "N possible words"; a distinct "No words match these clues" marker) → unit tests first.
- `render_results`: prepend `.results-summary` element (full-width, distinct look) when words non-empty; single "no words match" element (dim, non-red) when empty.
- Note: `solve` already returns `Err(EmptyInputs)` for all-empty input, so `Ok([])` only means "clues given but nothing matches".
- Verify: unit tests + cargo test; trunk build.
- **User review point.**

**Step D: `aria-live` (`index.html`)**
- Add `aria-live="polite"` to `#error` and `#results`.
- Verify: attributes present in trunk-served HTML.
- **User review point.**

**Step E: disable "+ Add Row" at cap (`src/visualizer.rs`, `static/style.css`, `index.html`)**
- Pure `at_row_cap(row_count) -> bool` == `>= 5` → unit test.
- `add_row` returns `bool` (added?); new `sync_add_row_button(document)` toggles `.disabled` from row count; call after add/remove and once in `init`.
- CSS `button:disabled { opacity: .5; cursor: not-allowed }`.
- Verify: unit tests + cargo test; trunk build.
- **User review point.**

**Step F: paste-to-fill row (`src/visualizer.rs`, `Cargo.toml`, `static/style.css`)**
- Add web-sys features `ClipboardEvent`, `DataTransfer`.
- Pure `spread_plan(row_values: &[&str], start_index, text) -> (Vec<Option<char>>, Option<usize>)`: clean via `sanitize_letter`, fill from `start_index`, cap at row end, return per-slot chars + focus index → unit tests first.
- Document-level `paste` listener: only tile inputs in `.word-row`; read `clipboardData.get_data("text")`, `prevent_default`, apply plan to the 5 row inputs, focus first unfilled slot, `refresh_results`.
- Verify: unit tests + cargo test; wasm32 build (proves new features compile); trunk build.
- **User review point.** (interactive — browser paste test)

**Step 7: Final verification sweep**
- `cargo test --all-targets`, `cargo clippy --all-targets`, wasm build, `trunk build`.
- Update DESIGN.md UI section if the count/no-match elements are worth documenting.
- Final user browser pass.

### Known Patterns
- Rust-only: inputs/events handled via web-sys; document-level delegated listeners with `Closure::forget()` (leak intentional, see `src/visualizer.rs`).
- `sanitize_letter(c, allowed)` in `solver::validator`; `UNKNOWN` = placeholder char.
- Pure DOM-free helpers are unit-tested in `#[cfg(test)]` (e.g. `tiles_to_pattern`, `classify_tile`).
- `#misplaced-rows` children cap is 5 (`add_row` currently no-ops at the cap; see `src/visualizer.rs:61`).
- `maxlength="1"` truncates multi-char paste before `input` fires → paste must be read from the `paste` event's `ClipboardEvent`, not from input values.

### Scope Limits
- No JS in `index.html`; no new dependencies beyond web-sys features.
- Do NOT change solver/filter/rank semantics (char-set exclusion model stays).
- DESIGN simplicity: no new visual clutter or chrome.

### Verification
```bash
cargo test --all-targets
cargo clippy --all-targets
cargo build --target wasm32-unknown-unknown
trunk build
```

### Implementation Details and Deviations

**A. `noscript` fallback**
- `<noscript>` placed immediately after `<body>`, `.noscript` class styled with
  absent-bg + text-tile foreground, centred. Verified present in `dist/index.html`.
- Deviation: none.

**B. Error colour**
- Added `--color-error: #a63d2c`. Contrast vs `--color-bg` (#f6f4ee) ≈ 5.7:1
  (passes AA). Underline + `> ` prefix unchanged (`currentColor`).
- Deviation: none.

**C. Empty state + count**
- Chose a single pure `summary_html(count) -> String` returning the full
  summary element HTML (0 → "No words match these clues", 1 → "1 possible
  word", n → "n possible words"); `render_results` prepends it.
- Review catch: `.results-summary` (single class) lost against the existing
  `#results > div` (ID wins) — the overrides were ignored by the browser. Fixed
  by raising specificity to `#results > .results-summary` (and
  `#results > .results-summary.no-match`), later in the file. Cleaned the
  mutation into no-match style.
- `Ok([])` only occurs with non-empty clues (all-empty already yields
  `Err(EmptyInputs)`), so "no match" is always an empty-state, never an error.

**D. `aria-live`**
- `aria-live="polite"` on `#error` and `#results` (attributes in `index.html`).
- Deviation: none.

**E. Disable "+ Add Row" at cap**
- `at_row_cap(u32) -> bool` pure helper (`>= 5`); `add_row` now returns `bool`
  (added or at-cap); new `sync_add_row_button` toggles the `disabled` attribute
  from the row count; called in `handle_click` after add, inside `remove_row`,
  and once in `init()`. CSS `button:disabled { opacity: .5; cursor: not-allowed }`.
- Deviation: `sync_add_row_button` also invoked from the remove path rather
  than only "after add/remove" call sites, so a stale state on load is covered.

**F. Paste-to-fill**
- `maxlength="1"` truncates pasted text before the `input` event, so paste is
  read from the document-level `paste` listener's `ClipboardEvent`
  (`clipboard_data().get_data("text")`), `prevent_default()` given, and applied
  to the 5 row inputs before `refresh_results`.
- Pure `spread_plan(row_len, start, raw) -> (Vec<Option<char>>, Option<usize>)`
  (fills + next-focus index); sanitizes via `sanitize_letter(c, false)` (letters
  only), fills from pasted tile onward, caps at row end, drops invalid/extra.
- Cargo.toml: added web-sys features `ClipboardEvent`, `DataTransfer`.
- Deviation: initial redundant `ClipboardEvent::is_none()` guard removed after
  clippy flagged it — the `dyn_ref` + `?` already handles the non-paste case.

**Verification:** 71 lib + 37 bin tests (was 27: +3 summary, +2 at_row_cap,
+5 spread_plan), clippy `--all-targets` zero warnings, wasm32 build ok,
`trunk build` ok. DESIGN.md UI section updated (noscript, paste, add-row
disable, results summary / no-match, aria, error role colour).

**Deferred (unchanged):** G (row-scoped aria-labels), H (hint line), I (chip
display cap), J (spotlight), K (conflict diagnostics).