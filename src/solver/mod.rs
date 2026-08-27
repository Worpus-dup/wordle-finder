pub mod error;
pub mod filter;
pub mod rank;
pub mod validator;

use error::SolverError;
use filter::filter;
use rank::rank;
use validator::validate;

pub fn solve(
    correct_letters: &str,
    misplaced_letters: &[&str],
    excluded_letters: &str,
) -> Result<Vec<String>, SolverError> {
    let (correct, misplaced, excluded) = validate(correct_letters, misplaced_letters, excluded_letters)?;
    let misplaced_refs: Vec<&str> = misplaced.iter().map(|s| s.as_str()).collect();
    let words = words_as_strs();
    let filtered = filter(&words, &correct, &misplaced_refs, &excluded);
    let ranked = rank(&filtered, &correct, &misplaced_refs, &excluded);
    Ok(ranked.into_iter().map(String::from).collect())
}

fn words_as_strs() -> Vec<&'static str> {
    crate::words::WORDS
        .chunks(5)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_all_empty() {
        let result = solve("     ", &[], "");
        assert_eq!(result.unwrap_err(), SolverError::EmptyInputs);
    }

    #[test]
    fn test_solve_invalid_character() {
        let result = solve("ab.de", &[], "");
        assert_eq!(result.unwrap_err(), SolverError::InvalidCharacter('.'));
    }

    #[test]
    fn test_solve_invalid_length() {
        let result = solve("abc", &[], "");
        assert_eq!(result.unwrap_err(), SolverError::InvalidLength(3));
    }

    #[test]
    fn test_solve_valid_input() {
        let result = solve("     ", &[], "");
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_with_correct_letters() {
        let result = solve("a    ", &[], "").unwrap();
        assert!(!result.is_empty());
        for word in &result {
            assert!(word.starts_with('a'));
        }
    }

    #[test]
    fn test_solve_with_excluded_letters() {
        let result = solve("     ", &[], "xyzwv").unwrap();
        assert!(!result.is_empty());
        for word in &result {
            assert!(!word.contains('x'));
            assert!(!word.contains('y'));
            assert!(!word.contains('z'));
            assert!(!word.contains('w'));
            assert!(!word.contains('v'));
        }
    }

    #[test]
    fn test_solve_all_letters_excluded() {
        let result = solve("     ", &[], "abcdefghijklmnopqrstuvwxyz").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_solve_with_misplaced_letters() {
        let result = solve("     ", &[" a   "], "").unwrap();
        assert!(!result.is_empty());
        for word in &result {
            assert!(word.contains('a'));
            assert_ne!(word.chars().nth(1), Some('a'));
        }
    }

    #[test]
    fn test_solve_returns_ranked_results() {
        let result = solve("a    ", &[], "").unwrap();
        assert!(!result.is_empty());
        for pair in result.windows(2) {
            let w1 = &pair[0];
            let w2 = &pair[1];
            assert!(w1.starts_with('a'));
            assert!(w2.starts_with('a'));
        }
    }
}
