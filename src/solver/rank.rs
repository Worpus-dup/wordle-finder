use crate::solver::bitmask::WordBitmask;
use crate::words::LETTER_FREQ;

pub fn rank<'a>(
    words: &'a [&str],
    correct: &str,
    misplaced: &[&str],
    excluded: &str,
) -> Vec<&'a str> {
    let guessed = collect_guessed_mask(correct, misplaced, excluded);
    let mut scored: Vec<(&str, u32)> = words
        .iter()
        .map(|&w| (w, score_word(w, guessed)))
        .collect();
    scored.sort_by_key(|b| std::cmp::Reverse(b.1));
    scored.into_iter().map(|(w, _)| w).collect()
}

fn collect_guessed_mask(correct: &str, misplaced: &[&str], excluded: &str) -> WordBitmask {
    let mut mask = WordBitmask::new();
    for c in correct.chars().chain(excluded.chars()) {
        mask.push(c);
    }
    for pattern in misplaced {
        for c in pattern.chars() {
            mask.push(c);
        }
    }
    mask
}

fn score_word(word: &str, guessed: WordBitmask) -> u32 {
    word.chars()
        .filter(|&c| !guessed.contains(c))
        .map(|c| LETTER_FREQ[(c as u8 - b'a') as usize])
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_empty_input() {
        let result = rank(&[], "     ", &[], "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_rank_single_word() {
        let words = &["apple"];
        let result = rank(words, "     ", &[], "");
        assert_eq!(result, vec!["apple"]);
    }

    #[test]
    fn test_rank_uncommon_letters_first() {
        let words = &["aaaaa", "jqxzw"];
        let result = rank(words, "     ", &[], "");
        assert_eq!(result, vec!["aaaaa", "jqxzw"]);
    }

    #[test]
    fn test_rank_excludes_guessed_letters() {
        let words = &["abcde", "fghij"];
        let result = rank(words, "     ", &[], "abcde");
        assert_eq!(result, vec!["fghij", "abcde"]);
    }

    #[test]
    fn test_rank_correct_letters_excluded() {
        let words = &["apple", "grape"];
        let result = rank(words, "a    ", &[], "");
        for (i, word) in result.iter().enumerate() {
            if i > 0 {
                let prev_score = score_word(result[i - 1], "a".parse().unwrap());
                let curr_score = score_word(word, "a".parse().unwrap());
                assert!(prev_score >= curr_score);
            }
        }
    }

    #[test]
    fn test_rank_misplaced_letters_excluded() {
        let words = &["apple", "grape"];
        let result = rank(words, "     ", &[" a  "], "");
        for (i, word) in result.iter().enumerate() {
            if i > 0 {
                let prev_score = score_word(result[i - 1], "a".parse().unwrap());
                let curr_score = score_word(word, "a".parse().unwrap());
                assert!(prev_score >= curr_score);
            }
        }
    }

    #[test]
    fn test_rank_excluded_letters_excluded() {
        let words = &["apple", "grape"];
        let result = rank(words, "     ", &[], "e");
        for (i, word) in result.iter().enumerate() {
            if i > 0 {
                let prev_score = score_word(result[i - 1], "e".parse().unwrap());
                let curr_score = score_word(word, "e".parse().unwrap());
                assert!(prev_score >= curr_score);
            }
        }
    }

    #[test]
    fn test_collect_guessed_mask() {
        let mask = collect_guessed_mask("a  b", &[" c"], "d");
        for &letter in &['a', 'b', 'c', 'd'] {
            assert!(mask.contains(letter));
        }
        assert!(!mask.contains('e'));
    }
}
