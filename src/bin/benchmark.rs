// Simulates Wordle against the real answer list and compares ranking variants.

use wordle_finder::solver::all_words;
use wordle_finder::solver::bitmask::WordBitmask;
use wordle_finder::solver::filter::filter;
use wordle_finder::solver::rank::rank_variant;
use wordle_finder::solver::rank::RankVariant;

const MAX_GUESSES: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Green,
    Yellow,
    Gray,
}

fn feedback(guess: &str, answer: &str) -> [Tile; 5] {
    let mut tiles = [Tile::Gray; 5];
    let mut remaining = [0u8; 26];
    for b in answer.bytes() {
        remaining[(b - b'a') as usize] += 1;
    }
    for (i, (g, a)) in guess.bytes().zip(answer.bytes()).enumerate() {
        if g == a {
            tiles[i] = Tile::Green;
            remaining[(g - b'a') as usize] -= 1;
        }
    }
    for (i, g) in guess.bytes().enumerate() {
        let idx = (g - b'a') as usize;
        if tiles[i] != Tile::Green && remaining[idx] > 0 {
            tiles[i] = Tile::Yellow;
            remaining[idx] -= 1;
        }
    }
    tiles
}

struct Constraints {
    correct: [char; 5],
    misplaced: Vec<String>,
    excluded: String,
    never_exclude: WordBitmask,
}

// ponytail: char-set exclusion, matching the UI model — a letter is excluded
// globally only if it never showed green/yellow anywhere; extra gray copies of
// a known letter are just blocked at those positions. Per-position item counts
// (real Wordle hard mode) would be the upgrade if the pool ever dies early.
fn apply_feedback(state: &mut Constraints, guess: &str, tiles: [Tile; 5]) {
    let mut pattern = [' '; 5];
    for (i, (g, t)) in guess.bytes().zip(tiles.iter()).enumerate() {
        let c = g as char;
        let idx = (g - b'a') as usize;
        match t {
            Tile::Green => {
                state.correct[i] = c;
                state.never_exclude.push(c);
            }
            Tile::Yellow => {
                state.never_exclude.push(c);
                pattern[i] = c;
            }
            Tile::Gray => {
                if state.never_exclude.contains(c) {
                    pattern[i] = c;
                } else {
                    let ch = (b'a' + idx as u8) as char;
                    if !state.excluded.contains(ch) {
                        state.excluded.push(ch);
                    }
                }
            }
        }
    }
    if pattern.iter().any(|&c| c != ' ') {
        state.misplaced.push(pattern.iter().collect());
    }
}

fn pool_freq(pool: &[&str]) -> [u32; 26] {
    let mut freq = [0u32; 26];
    for word in pool {
        for b in word.bytes() {
            freq[(b - b'a') as usize] += 1;
        }
    }
    freq
}

fn guessed_mask(state: &Constraints) -> WordBitmask {
    let mut mask = WordBitmask::new();
    for &c in &state.correct {
        mask.push(c);
    }
    for pattern in &state.misplaced {
        for c in pattern.chars() {
            mask.push(c);
        }
    }
    for c in state.excluded.chars() {
        mask.push(c);
    }
    mask
}

fn info_score(word: &str, guessed: WordBitmask, freq: &[u32; 26]) -> u32 {
    let mut seen = 0u32;
    let mut score = 0u32;
    for b in word.bytes() {
        let idx = (b - b'a') as usize;
        let bit = 1 << idx;
        if guessed.contains(b as char) || seen & bit != 0 {
            continue;
        }
        seen |= bit;
        score += freq[idx];
    }
    score
}

#[derive(Debug)]
enum Strategy {
    Pool,
    Probe { stop_prob: usize },
}

fn best_guess(state: &Constraints, strategy: &Strategy, dict: &[&str]) -> Option<String> {
    let correct: String = state.correct.iter().collect();
    let misplaced: Vec<&str> = state.misplaced.iter().map(String::as_str).collect();
    let pool = filter(all_words(), &correct, &misplaced, &state.excluded);

    match strategy {
        Strategy::Pool => {
            let ranked = rank_variant(&pool, &correct, &misplaced, &state.excluded, RankVariant::Pool);
            ranked.first().map(|w| (*w).to_string())
        }
        Strategy::Probe { stop_prob } => {
            if pool.len() <= *stop_prob {
                let ranked = rank_variant(&pool, &correct, &misplaced, &state.excluded, RankVariant::Pool);
                return ranked.first().map(|w| (*w).to_string());
            }
            let freq = pool_freq(&pool);
            let guessed = guessed_mask(state);
            let pool_set: std::collections::HashSet<&str> = pool.iter().copied().collect();
            let mut candidates: Vec<&str> = pool.to_vec();
            for &w in dict {
                if !pool_set.contains(w)
                    && !w.bytes().any(|b| state.excluded.contains(b as char))
                {
                    candidates.push(w);
                }
            }
            // ponytail: expected-gain is the pool-frequency of unguessed letters;
            // probe costs a never-winning guess, so stop_prob keeps the endgame honest.
            let mut scored: Vec<(&str, u32)> = candidates
                .iter()
                .map(|&w| (w, info_score(w, guessed, &freq)))
                .collect();
            scored.sort_by_key(|b| std::cmp::Reverse(b.1));
            scored.first().map(|(w, _)| (*w).to_string())
        }
    }
}

fn solve_guesses(answer: &str, strategy: &Strategy, dict: &[&str]) -> usize {
    let mut state = Constraints {
        correct: [' '; 5],
        misplaced: Vec::new(),
        excluded: String::new(),
        never_exclude: WordBitmask::new(),
    };
    for round in 1..=MAX_GUESSES {
        let Some(guess) = best_guess(&state, strategy, dict) else {
            return MAX_GUESSES + 1;
        };
        if guess == answer {
            return round;
        }
        apply_feedback(&mut state, &guess, feedback(&guess, answer));
    }
    MAX_GUESSES + 1
}

fn run_strategy(words: &[&str], strategy: &Strategy, dict: &[&str]) {
    let mut rounds = [0usize; MAX_GUESSES + 2];
    let mut wins = 0;
    let mut win_guesses = 0usize;
    for &answer in words {
        let r = solve_guesses(answer, strategy, dict);
        if r <= MAX_GUESSES {
            wins += 1;
            win_guesses += r;
        }
        rounds[r.min(MAX_GUESSES + 1)] += 1;
    }
    let avg = if wins > 0 {
        win_guesses as f64 / wins as f64
    } else {
        0.0
    };
    println!("strategy: {strategy:?}");
    println!("win% (<= {MAX_GUESSES}): {:.1}% ({wins}/{})", 100.0 * wins as f64 / words.len() as f64, words.len());
    println!("avg guesses (wins only): {avg:.2}");
    println!("rounds: 1..6 = {:?}, 7+ = {}", &rounds[1..=MAX_GUESSES], rounds[MAX_GUESSES + 1]);
    println!();
}

fn main() {
    let words = all_words();
    let contents = std::fs::read_to_string("data/guesses.txt")
        .expect("data/guesses.txt");
    let dict: Vec<&str> = contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::trim)
        .collect();
    let strategies = [
        Strategy::Pool,
        Strategy::Probe { stop_prob: 6 },
        Strategy::Probe { stop_prob: 10 },
        Strategy::Probe { stop_prob: 15 },
        Strategy::Probe { stop_prob: 25 },
        Strategy::Probe { stop_prob: 40 },
    ];
    for strategy in &strategies {
        run_strategy(words, strategy, &dict);
    }
}