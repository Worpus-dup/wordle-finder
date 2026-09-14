use crate::solver::bitmask::WordBitmask;
use crate::words::LETTER_FREQ;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RankVariant {
    Static,
    Pool,
}

pub fn rank<'a>(
    words: &'a [&str],
    correct: &str,
    misplaced: &[&str],
    excluded: &str,
) -> Vec<&'a str> {
    rank_variant(words, correct, misplaced, excluded, RankVariant::Pool)
}

pub fn rank_variant<'a>(
    words: &'a [&str],
    correct: &str,
    misplaced: &[&str],
    excluded: &str,
    variant: RankVariant,
) -> Vec<&'a str> {
    let guessed = collect_guessed_mask(correct, misplaced, excluded);
    let pool_freq = match variant {
        RankVariant::Static => None,
        RankVariant::Pool => Some(count_pool_freq(words)),
    };
    let mut scored: Vec<(&str, u32)> = words
        .iter()
        .map(|&w| (w, score_word(w, guessed, pool_freq.as_ref())))
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

fn count_pool_freq(words: &[&str]) -> [u32; 26] {
    let mut freq = [0u32; 26];
    for word in words {
        for b in word.bytes() {
            freq[(b - b'a') as usize] += 1;
        }
    }
    freq
}

fn score_word(word: &str, guessed: WordBitmask, pool_freq: Option<&[u32; 26]>) -> u32 {
    match pool_freq {
        None => word
            .chars()
            .filter(|&c| !guessed.contains(c))
            .map(|c| LETTER_FREQ[(c as u8 - b'a') as usize])
            .sum(),
        Some(freq) => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask_of(s: &str) -> WordBitmask {
        let mut mask = WordBitmask::new();
        for c in s.chars() {
            mask.push(c);
        }
        mask
    }

    fn pool_score(word: &str, words: &[&str], guessed: WordBitmask) -> u32 {
        score_word(word, guessed, Some(&count_pool_freq(words)))
    }

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
                let prev_score = pool_score(result[i - 1], words, mask_of("a"));
                let curr_score = pool_score(word, words, mask_of("a"));
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
                let prev_score = pool_score(result[i - 1], words, mask_of("a"));
                let curr_score = pool_score(word, words, mask_of("a"));
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
                let prev_score = pool_score(result[i - 1], words, mask_of("e"));
                let curr_score = pool_score(word, words, mask_of("e"));
                assert!(prev_score >= curr_score);
            }
        }
    }

    #[test]
    fn test_rank_pool_prefers_pool_common_letters() {
        let words = &["qazws", "wshba"];
        let pool = rank_variant(words, "     ", &[], "", RankVariant::Pool);
        let stat = rank_variant(words, "     ", &[], "", RankVariant::Static);
        assert_eq!(pool[0], "qazws");
        assert_eq!(stat[0], "wshba");
    }

    #[test]
    fn test_rank_pool_counts_distinct_letters_once() {
        let words = &["ellee", "alpha"];
        let result = rank(words, "     ", &[], "");
        assert_eq!(result, vec!["alpha", "ellee"]);
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
