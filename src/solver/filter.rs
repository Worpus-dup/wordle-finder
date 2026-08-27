use crate::solver::validator::UNKNOWN;

pub fn filter<'a>(
    words: &'a [&str],
    correct: &str,
    misplaced: &[&str],
    excluded: &str,
) -> Vec<&'a str> {
    words
        .iter()
        .copied()
        .filter(|word| {
            matches_correct(word, correct)
                && matches_misplaced(word, misplaced)
                && matches_excluded(word, excluded)
        })
        .collect()
}

fn matches_correct(word: &str, correct: &str) -> bool {
    word.chars()
        .zip(correct.chars())
        .all(|(w, c)| c == UNKNOWN || w == c)
}

fn matches_misplaced(word: &str, misplaced: &[&str]) -> bool {
    for pattern in misplaced.iter() {
        let mut required: [usize; 26] = [0; 26];
        let mut positions: Vec<(usize, char)> = Vec::new();

        for (j, pattern_char) in pattern.chars().enumerate() {
            if pattern_char == UNKNOWN {
                continue;
            }
            if word.chars().nth(j) == Some(pattern_char) {
                return false;
            }
            required[(pattern_char as u8 - b'a') as usize] += 1;
            positions.push((j, pattern_char));
        }

        for (ch_idx, &count) in required.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let ch = (b'a' + ch_idx as u8) as char;
            let word_count = word.chars().filter(|&c| c == ch).count();
            if word_count < count {
                return false;
            }
        }

        let _ = positions;
    }
    true
}

fn matches_excluded(word: &str, excluded: &str) -> bool {
    !excluded.chars().any(|c| word.contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: &[&str] = &[
        "apple", "grape", "melon", "lemon", "olive", "peach", "plum", "mango",
    ];

    #[test]
    fn test_filter_exact_match() {
        let result = filter(&WORDS, "apple", &[], "");
        assert_eq!(result, vec!["apple"]);
    }

    #[test]
    fn test_filter_first_letter() {
        let result = filter(&WORDS, "a    ", &[], "");
        assert!(result.contains(&"apple"));
        assert!(!result.contains(&"grape"));
    }

    #[test]
    fn test_filter_excluded_letters() {
        let result = filter(&WORDS, "     ", &[], "ae");
        assert!(!result.contains(&"apple"));
        assert!(!result.contains(&"grape"));
        assert!(!result.contains(&"melon"));
    }

    #[test]
    fn test_filter_excluded_none_match() {
        let result = filter(&WORDS, "    ", &[], "aeiou");
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_misplaced_letter() {
        let result = filter(&WORDS, "     ", &[" a   "], "");
        for word in &result {
            assert!(word.contains('a'));
            assert_ne!(word.chars().nth(1), Some('a'));
        }
    }

    #[test]
    fn test_filter_misplaced_not_at_position() {
        let result = filter(&WORDS, "     ", &["a    "], "");
        for word in &result {
            assert!(word.contains('a'));
            assert_ne!(word.chars().nth(0), Some('a'));
        }
    }

    #[test]
    fn test_filter_multiple_misplaced() {
        let result = filter(&WORDS, "     ", &[" a  ", "  l  "], "");
        for word in &result {
            assert!(word.contains('a'));
            assert!(word.contains('l'));
            assert_ne!(word.chars().nth(1), Some('a'));
            assert_ne!(word.chars().nth(2), Some('l'));
        }
    }

    #[test]
    fn test_filter_combined_correct_and_excluded() {
        let result = filter(&WORDS, "  pp ", &[], "a");
        for word in &result {
            assert_eq!(word.chars().nth(2), Some('p'));
            assert_eq!(word.chars().nth(3), Some('p'));
            assert!(!word.contains('a'));
        }
    }

    #[test]
    fn test_filter_no_match() {
        let result = filter(&WORDS, "xyzwv", &[], "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_all_words_match() {
        let result = filter(&WORDS, "     ", &[], "");
        assert_eq!(result.len(), WORDS.len());
    }

    #[test]
    fn test_filter_single_letter_misplaced() {
        let test_words: &[&str] = &["abcde", "bacde", "cabde", "xyzab"];
        let result = filter(test_words, "     ", &["a    "], "");
        assert!(!result.contains(&"abcde"));
        assert!(result.contains(&"bacde"));
        assert!(result.contains(&"cabde"));
        assert!(result.contains(&"xyzab"));
    }

    #[test]
    fn test_filter_excluded_with_misplaced() {
        let test_words: &[&str] = &["abcde", "bacde", "adecf", "aecdf"];
        let result = filter(test_words, "     ", &[" a  "], "b");
        assert!(!result.contains(&"abcde"));
        assert!(!result.contains(&"bacde"));
        assert!(result.contains(&"adecf"));
        assert!(result.contains(&"aecdf"));
    }

    #[test]
    fn test_filter_duplicate_misplaced_letter() {
        let test_words: &[&str] = &["queue", "unial"];
        let result = filter(test_words, "     ", &["  u u"], "");
        assert!(result.contains(&"queue"));
        assert!(!result.contains(&"unial"));
    }

    #[test]
    fn test_filter_separate_misplaced_same_letter() {
        let test_words: &[&str] = &["apple", "apply", "grape"];
        let result = filter(test_words, "     ", &["  a  ", "   a "], "");
        assert!(result.contains(&"apple"));
        assert!(result.contains(&"apply"));
        assert!(!result.contains(&"grape"));
    }
}
