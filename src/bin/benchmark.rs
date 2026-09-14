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

fn best_guess(state: &Constraints, variant: RankVariant) -> Option<String> {
    let correct: String = state.correct.iter().collect();
    let misplaced: Vec<&str> = state.misplaced.iter().map(String::as_str).collect();
    // ponytail: in-pool only — never probes data/guesses.txt for info-gathering.
    // Shaves avg guesses; add when the benchmark stops sagging.
    let pool = filter(all_words(), &correct, &misplaced, &state.excluded);
    let ranked = rank_variant(&pool, &correct, &misplaced, &state.excluded, variant);
    ranked.first().map(|w| (*w).to_string())
}

fn solve_guesses(answer: &str, variant: RankVariant) -> usize {
    let mut state = Constraints {
        correct: [' '; 5],
        misplaced: Vec::new(),
        excluded: String::new(),
        never_exclude: WordBitmask::new(),
    };
    for round in 1..=MAX_GUESSES {
        let Some(guess) = best_guess(&state, variant) else {
            return MAX_GUESSES + 1;
        };
        if guess == answer {
            return round;
        }
        apply_feedback(&mut state, &guess, feedback(&guess, answer));
    }
    MAX_GUESSES + 1
}

fn main() {
    let words = all_words();
    for variant in [RankVariant::Static, RankVariant::Pool] {
        let mut rounds = [0usize; MAX_GUESSES + 2];
        let mut wins = 0;
        let mut win_guesses = 0usize;
        for &answer in words {
            let r = solve_guesses(answer, variant);
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
        println!("variant: {variant:?}");
        println!("win% (<= {MAX_GUESSES}): {:.1}% ({wins}/{})", 100.0 * wins as f64 / words.len() as f64, words.len());
        println!("avg guesses (wins only): {avg:.2}");
        println!("rounds: 1..6 = {:?}, 7+ = {}", &rounds[1..=MAX_GUESSES], rounds[MAX_GUESSES + 1]);
        println!();
    }
}