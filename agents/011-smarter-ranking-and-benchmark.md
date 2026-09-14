## Task: Smarter Ranking Algorithm + Benchmark Satellite Binary

### Goal
Improve the probability of guessing the Wordle answer correctly (win within 6
guesses). Judge improvement with a measurable benchmark: simulate the game
against the real list of answers (`data/answers/wordle-answers-alphabetical.txt`,
2315 words) and compare ranking variants.

### Scope
- Replace static full-dictionary `LETTER_FREQ` scoring in `src/solver/rank.rs`
  with pool-adaptive weighted discovery of new common letters.
- Create a native satellite binary `src/bin/benchmark.rs` that simulates the
  game on every real answer and reports win% (≤6 guesses) + average guesses.
- Restructure crate into lib + bin so the benchmark bin can reuse the solver.
- In-pool ranking only. Off-pool probing from `data/guesses.txt` is explicitly
  out of scope for now (benchmarked later).

### Motivation (from repo history)
- `agents/001-game-solver-module.md:46` specified "count letter frequency
  **across remaining words**, score each word".
- `agents/001` Deviation 5 swapped that for a static build-time `LETTER_FREQ`
  table ("Avoids recomputing frequency on every call; build-time optimization").
- Result: current `score_word` uses a fixed dictionary-wide frequency and also
  double-counts repeated letters in a guess.
- This task reinstates the original adaptive intent, with the benchmark as the
  justification for the per-call cost.

### Acceptance Criteria
- [ ] `rank()` scores guesses by sum of pool frequency over **distinct**
      not-yet-guessed letters, pool = current candidate list (per-call count).
- [ ] Duplicate letters in a guess count once (fixes current double-count).
- [ ] Benchmark binary (native, std-only) runs each of the 2315 real answers:
      Wordle-faithful two-pass count-aware feedback; ≤6 guesses; best guess =
      first of `rank(pool, …)`.
- [ ] Benchmark prints win% (≤6) and avg guesses for both `static` and `pool`
      ranking variants; default `pool`.
- [ ] Crates are restructured as lib + bin; `main.rs`/visualizer behavior
      unchanged; wasm/trunk build still works.
- [ ] `cargo test` all green.

### Ranking Algorithm (`src/solver/rank.rs`)
Current (baseline, kept for comparison):
```
score(w) = Σ LETTER_FREQ[c]   for c in w, c ∉ guessed   (static, dups counted)
```
New (`--variant pool`):
```
pool_freq[c] = count of candidate words containing c
score(w)     = Σ pool_freq[c] for distinct c in w, c ∉ guessed
```
Implementation notes:
- `rank` already receives the filtered candidate slice — count `[u32; 26]` over
  it once per call, then score.
- `guessed` mask logic (`collect_guessed_mask`) unchanged.
- Variant selected via a small enum/flag so the benchmark can compare without
  shipping two code paths permanently (benchmark-only switch).

### Benchmark (`src/bin/benchmark.rs`)
```
for answer in WORDS:                        # 2315 real answers
    constraints = empty
    for round in 1..=6:
        pool       = filter(WORDS, constraints)
        candidates = rank(pool, constraints, variant)
        guess      = candidates[0]          # empty constraints → full words
        feedback   = feedback(guess, answer) # two-pass count-aware
        if all green: solved, record round; break
        else: constraints += translate(feedback)  → next round
    if not solved: count as loss (7+)
print: variant, win% (≤6), avg guesses, worst, guess-count distribution
```
- `feedback()`: mark greens first, then yellows from remaining counts, rest gray
  (matches real Wordle duplicate-tile rules).
- Feedback → solver constraints (mirrors what the UI can express):
  - green at i → `correct[i] = letter`
  - yellow at i  → misplaced pattern: avoid position i for that letter + require
    presence
  - gray letter with no green/yellow copy this guess → excluded char
  - gray letter that is also green/yellow this guess → extra avoided position in
    the misplaced pattern (no global exclusion)
- Opening guess: no constraints yet — `rank(WORDS, "     ", &[], "")` (bypasses
  `solve()`'s empty-input rejection).
- CLI: `--variant static|pool` (default `pool`).

### Files
```
src/lib.rs                    # NEW  pub mod solver; pub mod words;
src/main.rs                   # visualizer stays here; uses wordle_finder::*
src/solver/rank.rs            # EDIT pool-adaptive scoring + variant switch
src/bin/benchmark.rs          # NEW  simulation + comparison table
build.rs                      # unchanged
```

### Scope Limits
- Do NOT rank guesses outside the candidate pool (`data/guesses.txt` unused).
  Probing is a future follow-up if the benchmark demands it.
- Do NOT change `filter.rs` semantics (char-set exclusion stays, matching the
  UI model).
- Do NOT change `LETTER_FREQ` generation in `build.rs` — static variant baseline
  still derives from it.

### Verification
```bash
cargo test                                   # all unit tests pass
cargo run --bin benchmark -- --variant static
cargo run --bin benchmark -- --variant pool  # compare win% + avg guesses
cargo build --target wasm32-unknown-unknown  # UI path untouched
```

### Known Patterns
- WordBitmask (u32, 26 bits) for letter-set membership.
- `correct_letters`: 5-char string, space = unknown.
- `misplaced_letters`: `&[&str]` patterns (position + presence).
- `excluded_letters`: string of letters to exclude.
- Ranking is stable-ish: ties keep input order; benchmark only reads the top-1.

### Implementation Details and Deviations
<!-- Written after completing the task -->

**1. RankVariant dimensions**
- Original: `--variant static|pool` CLI flag on the benchmark.
- Actual: no CLI — `main()` always runs both variants and prints both summaries.
  The flag was a needless knob; compare-both is the only call that matters.

**2. `rank()` default**
- Original: rank keeps a pool default, static kept for comparison.
- Actual: `rank()` defaults to `RankVariant::Pool` (the shipped improvement);
  `rank_variant(..., RankVariant::Static)` is the benchmark-only baseline.

**3. Baseline preserved exactly**
- Static scoring keeps the historical behavior (per-occurrence static `LETTER_FREQ`,
  repeated letters double-counted) so the benchmark delta is honest.
- Pool scoring sums pool frequency over **distinct** not-yet-guessed letters.

**4. Benchmark results (2315 real answers, ≤6 guesses, release)**
- Static: win% 34.2% (791/2315), avg 4.18 wins, 1524 losses.
- Pool:   win% 98.6% (2283/2315), avg 3.65 wins, 32 losses.

**5. Benchmark internals**
- `feedback()`: greens first, then yellows by remaining counts (Wordle-faithful).
- Constraint translation mirrors the UI model: green→correct char, yellow→misplaced
  pattern + presence, gray→global excluded **only** if the letter never showed
  green/yellow; otherwise a gray copy is just another avoided position
  (`never_exclude` mask). `ponytail:` limitation — see comment in
  `src/bin/benchmark.rs`.
- Dropped `data/guesses.txt` probing (scope-limit held).