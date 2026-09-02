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
