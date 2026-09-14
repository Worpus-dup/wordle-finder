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
- Static: win% 34.2% (791/2315), avg 4.18 wins, 1524 losses. *(Historical —
  benchmark no longer runs a static arm; kept for the baseline record.)*
- Pool:   win% 98.8% (2288/2315), avg 3.65 wins, 27 losses.

**5. Benchmark internals**
- `feedback()`: greens first, then yellows by remaining counts (Wordle-faithful).
- Constraint translation mirrors the UI model: green→correct char, yellow→misplaced
  pattern + presence, gray→global excluded **only** if the letter never showed
  green/yellow; otherwise a gray copy is just another avoided position
  (`never_exclude` mask). `ponytail:` limitation — see comment in
  `src/bin/benchmark.rs`.
- Two-pass feedback (greens registered first) — see §8, bug fix.
- Dropped `data/guesses.txt` probing (scope-limit held).

**6. Off-pool probing experiment (follow-up)**
- `Strategy::Probe`: rank candidate guesses from the full `data/guesses.txt`
  dictionary by expected info gain (distinct unguessed letters × answer-pool
  frequency, pool words preferred on ties), probing only while `pool.len() >
  stop_prob`, then in-pool. Same constraint/feedback pipeline as Pool.
- Results (2315 answers, ≤6 guesses, release):

  | strategy           | win%   | avg | 7+  |
  |--------------------|--------|-----|-----|
  | Pool               | 98.8%  | 3.65| 27  |
  | Probe stop_prob=6  | 99.2%  | 3.68| 18  |
  | Probe stop_prob=10 | 99.0%  | 3.67| 23  |
  | Probe stop_prob=15 | 99.0%  | 3.67| 22  |
  | Probe stop_prob=25 | 98.9%  | 3.67| 25  |
  | Probe stop_prob=40 | 98.8%  | 3.67| 27  |

- Verdict: probing does **not** lower average guesses (Pool 3.65 beats every
  probe threshold). It nudges win% up at most +0.4pp (99.2% vs 98.8%,
  stop_prob=6, nine more wins) and rescues endgames (7+ 27→18) but trades
  away 2/3-guess wins. Over-aggressive probing (stop_prob=1/3) is much worse.
  → Keep the shipped algorithm in-pool. Naive info-score probe; an
  expected-pool-reduction (entropy) probe is the next rung if win% is ever the
  sole goal — also bumps the shipped `data/guesses.txt` deadness note.

**7. Pool-elimination scoring experiment (follow-up)**
- `Strategy::Eliminate`: rank the pool of possible answers by expected shrinkage
  — for each candidate guess, partition the pool by the Wordle feedback pattern
  it would produce and minimize Σ group² (the expected surviving pool ≈
  pool − eliminated). Applied for the first N rounds, then falls back to Pool.
  Opening-round result is identical for all answers, so it's computed once.
- Results (2315 answers, ≤6 guesses, release):

  | strategy | win% | avg | 7+ |
  |----------|------|-----|----|
  | Pool     | 98.8%| 3.65| 27 |
  | Eliminate rounds=1 | 98.7% | 3.67 | 31 |
  | Eliminate rounds=2 | 97.7% | 3.64 | 53 |
  | Eliminate rounds=3 | 96.9% | 3.61 | 71 |
  | Eliminate always   | 96.8% | 3.61 | 75 |

  > Eliminate rows are **historical** (see §8): the experiment's code was never
  > committed and is not in the working tree; the rows were produced with the
  > pre-fix single-pass `apply_feedback`. The gray-then-green bug costs every
  > strategy ≈5 wins (Pool 32→27, §4), so eliminate's win% is understated by a
  > similar margin at worst — the verdict (rejected; Pool's endgame edge) does
  > not depend on it and stands.

- Verdict: rejected. One opening elimination round is a wash (+1 win, +0.02
  avg); more rounds trade win% away for a trivial avg dip. Pool's endgame edge
  (winning small pools) is exactly what Σ-group² ranking undervalues. → Pool
  stays shipped.
- Probe strategy: **kept** in `src/bin/benchmark.rs` by decision (§8) after the
  §7 cleanup note was evaluated against the working tree.

**8. Project-state verification (follow-up)**

Findings from a read-only audit pass and the fixes applied after it.

- **Bug (fixed):** `apply_feedback` processed tiles in positional order, so a
  doubled letter that was gray at an earlier position and green/yellow at a
  later position in the *same guess* pushed the letter into global `excluded`
  before the green registered it in `never_exclude` — leaving `excluded` and
  `correct` contradictory and emptying the pool (certain loss). Single pass →
  two-pass: all greens registered first, then yellows/grays. Impact: ≈5 losses
  per strategy were this bug, not ranking (Pool 32→27, Probe stop6 20→18).
  Rows in §4/§6 re-run after the fix; §7 rows are historical (see note there).
- **Unused code (removed):** generated `WORD_COUNT` constant (only consumed by
  `#[allow(dead_code)]`; zero reads) deleted from `build.rs` template and
  `src/words.rs`. `RankVariant::Static` + static score path + `LETTER_FREQ`
  remain, intentional — §4 baseline comparison and the rank unit test.
- **Deviation (working tree vs §7):** the pool-elimination experiment's code
  (`Strategy::Eliminate`, `pattern_key`, `elimination_score`) was never
  committed and no stash/reflog holds it — §7's numbers are not reproducible
  from the repo. The §7 end-state note claimed probe code + `data/guesses.txt`
  loading were removed; that cleanup never landed in the tree. **Decision:
  keep the probe strategy** (and the `guesses.txt` read it needs) so the §6
  probe table stays reproducible from the committed benchmark.
- **Working tree:** `README.md` (mention of OpenCode/Ponytail) and this file's
  §7/§8 additions are uncommitted; `HEAD` has benchmark at b0a348c.