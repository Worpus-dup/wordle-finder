use crate::solver::error::SolverError;

pub const UNKNOWN: char = ' ';

pub fn is_correct_empty(correct: &str) -> bool {
    correct.chars().all(|c| c == UNKNOWN)
}

pub fn all_empty(correct: &str, misplaced: &[&str], excluded: &str) -> bool {
    is_correct_empty(correct) && misplaced.is_empty() && excluded.is_empty()
}

pub fn validate(
    correct_letters: &str,
    misplaced_letters: &[&str],
    excluded_letters: &str,
) -> Result<(String, Vec<String>, String), SolverError> {
    if all_empty(correct_letters, misplaced_letters, excluded_letters) {
        return Err(SolverError::EmptyInputs);
    }
    let correct = validate_correct(correct_letters)?;
    let misplaced = validate_misplaced(misplaced_letters)?;
    let excluded = validate_excluded(excluded_letters)?;
    Ok((correct, misplaced, excluded))
}

fn validate_correct(input: &str) -> Result<String, SolverError> {
    if input.len() != 5 {
        return Err(SolverError::InvalidLength(input.len()));
    }
    let mut result = String::with_capacity(5);
    for c in input.chars() {
        if c == UNKNOWN {
            result.push(UNKNOWN);
        } else if c.is_ascii_lowercase() {
            result.push(c);
        } else if c.is_ascii_uppercase() {
            result.push(c.to_ascii_lowercase());
        } else {
            return Err(SolverError::InvalidCharacter(c));
        }
    }
    Ok(result)
}

fn validate_misplaced(inputs: &[&str]) -> Result<Vec<String>, SolverError> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let validated = validate_correct(input)?;
        result.push(validated);
    }
    Ok(result)
}

fn validate_excluded(input: &str) -> Result<String, SolverError> {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_ascii_lowercase() {
            result.push(c);
        } else if c.is_ascii_uppercase() {
            result.push(c.to_ascii_lowercase());
        } else {
            return Err(SolverError::InvalidCharacter(c));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_correct_lowercase() {
        let (correct, _, _) = validate("abcde", &[], "").unwrap();
        assert_eq!(correct, "abcde");
    }

    #[test]
    fn test_validate_correct_uppercase() {
        let (correct, _, _) = validate("ABCDE", &[], "").unwrap();
        assert_eq!(correct, "abcde");
    }

    #[test]
    fn test_validate_correct_mixed_case() {
        let (correct, _, _) = validate("aBcDe", &[], "").unwrap();
        assert_eq!(correct, "abcde");
    }

    #[test]
    fn test_validate_correct_with_unknown() {
        let (correct, _, _) = validate("a p l", &[], "").unwrap();
        assert_eq!(correct, "a p l");
    }

    #[test]
    fn test_validate_correct_invalid_length_short() {
        let err = validate("abc", &[], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidLength(3));
    }

    #[test]
    fn test_validate_correct_invalid_length_long() {
        let err = validate("abcdef", &[], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidLength(6));
    }

    #[test]
    fn test_validate_correct_invalid_character() {
        let err = validate("ab.de", &[], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidCharacter('.'));
    }

    #[test]
    fn test_validate_correct_invalid_unicode() {
        let err = validate("ab\0de", &[], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidCharacter('\0'));
    }

    #[test]
    fn test_validate_misplaced_valid() {
        let (_, misplaced, _) = validate("     ", &["a    ", "b    "], "").unwrap();
        assert_eq!(misplaced, vec!["a    ", "b    "]);
    }

    #[test]
    fn test_validate_misplaced_uppercase() {
        let (_, misplaced, _) = validate("     ", &["A    "], "").unwrap();
        assert_eq!(misplaced, vec!["a    "]);
    }

    #[test]
    fn test_validate_misplaced_invalid_length() {
        let err = validate("     ", &["abc"], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidLength(3));
    }

    #[test]
    fn test_validate_misplaced_invalid_character() {
        let err = validate("     ", &["a.b.d"], "").unwrap_err();
        assert_eq!(err, SolverError::InvalidCharacter('.'));
    }

    #[test]
    fn test_validate_excluded_valid() {
        let (_, _, excluded) = validate("     ", &[], "abc").unwrap();
        assert_eq!(excluded, "abc");
    }

    #[test]
    fn test_validate_excluded_uppercase() {
        let (_, _, excluded) = validate("     ", &[], "ABC").unwrap();
        assert_eq!(excluded, "abc");
    }

    #[test]
    fn test_validate_excluded_invalid_character() {
        let err = validate("     ", &[], "ab.c").unwrap_err();
        assert_eq!(err, SolverError::InvalidCharacter('.'));
    }

    #[test]
    fn test_validate_all_empty() {
        let result = validate("     ", &[], "");
        assert_eq!(result.unwrap_err(), SolverError::EmptyInputs);
    }

    #[test]
    fn test_validate_not_all_empty_correct() {
        let result = validate("a    ", &[], "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_all_empty_misplaced() {
        let result = validate("     ", &["a    "], "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_all_empty_excluded() {
        let result = validate("     ", &[], "a");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiple_errors_first_error() {
        let err = validate("abc", &["def"], "ghi.jkl").unwrap_err();
        assert_eq!(err, SolverError::InvalidLength(3));
    }

    #[test]
    fn test_is_correct_empty_all_spaces() {
        assert!(is_correct_empty("     "));
    }

    #[test]
    fn test_is_correct_empty_has_letter() {
        assert!(!is_correct_empty("a    "));
    }

    #[test]
    fn test_is_correct_empty_mixed() {
        assert!(!is_correct_empty("a  p l"));
    }

    #[test]
    fn test_all_empty_true() {
        assert!(all_empty("     ", &[], ""));
    }

    #[test]
    fn test_all_empty_correct_has_letter() {
        assert!(!all_empty("a    ", &[], ""));
    }

    #[test]
    fn test_all_empty_misplaced_not_empty() {
        assert!(!all_empty("     ", &["a    "], ""));
    }

    #[test]
    fn test_all_empty_excluded_not_empty() {
        assert!(!all_empty("     ", &[], "a"));
    }
}
