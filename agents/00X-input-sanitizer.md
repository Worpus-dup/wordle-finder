## Task: Input Sanitizer (DRAFT — not started)

> Status: DRAFT / STUB. The input sanitizer was deferred from task `003-partial-visualizer.md`
> (Step 4). This document captures the initial plan discussion for when we pick this up.

### Background
- Task `003-partial-visualizer.md` (completed) implemented input reading + result rendering.
- Currently, `solve()` errors (per `SolverError`) are handled by clearing `#results` and logging to the console via `console::error`.
- DESIGN.md (Visualizer responsibilities): "If any error detected in user inputs, sanitise incorrect data and replace the original. If error occurred in the solver or too critical to continue processing, show error and stop processing. Processing should restart when inputs are corrected."

### Scope
- Sanitize user inputs before passing to the solver.
- Show a user-facing error message on the page for critical/unsanitizable cases (per DESIGN.md), rather than only logging to console.

### Open Design Questions (to resolve before implementation)
- **Q1: Scope of "sanitize"**:
  - (a) Sanitize-only: coerce bad input to valid before solving (e.g., lowercase all letters, drop non a-z), then render normally.
  - (b) Error-display: keep meaningful errors but show a user-facing message on the page.
  - (c) Both (fits DESIGN.md best): sanitize what's recoverable AND show a message for critical cases (e.g., all-empty).
- **Q2: Where sanitization lives**: per DESIGN.md it's a visualizer responsibility → likely in `src/visualizer.rs` (UI layer), before calling `solve()`. Alternative: add a solver-layer function.
  - Note: 5-tile inputs accept at most 1 char each; realistic sanitization targets are **case** (uppercase) and the **excluded letters** free-text input.
- **Q3: New task file vs. extend 003**: prefer a new task file (e.g., `004-input-sanitizer.md`).

### Likely Acceptance Criteria (Initial)
- [ ] Uppercase letters are lowercased before solving
- [ ] Non-alphabetic characters in excluded input are handled/sanitized
- [ ] Critical errors (e.g., all-empty / invalid) show a user-facing message on the page
- [ ] Results update as inputs are corrected (processing restarts)
- [ ] `cargo test` passes

### Scope Limits (Initial)
- Do NOT add result styling (separate task)
- Do NOT rewrite keyboard-navigation JS in Rust (separate task)
