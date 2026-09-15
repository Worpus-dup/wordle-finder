use crate::solver::error::SolverError;

pub const UNKNOWN: char = ' ';

pub fn is_correct_empty(correct: &str) -> bool {
    correct.chars().all(|c| c == UNKNOWN)
}

pub fn all_empty(correct: &str, misplaced: &[&str], excluded: &str) -> bool {
    is_correct_empty(correct)
    && misplaced.iter().cloned().all(is_correct_empty)
    && excluded.is_empty()
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
    if let Some(c) = conflicting_letter(&correct, &misplaced, &excluded) {
        return Err(SolverError::ConflictingLetter(c));
    }
    if let Some((c, pos)) = conflicting_position(&correct, &misplaced) {
        return Err(SolverError::ConflictingPosition(c, pos));
    }
    Ok((correct, misplaced, excluded))
}

fn conflicting_letter(correct: &str, misplaced: &[String], excluded: &str) -> Option<char> {
    let mut required = [false; 26];
    for c in correct.chars().chain(misplaced.iter().flat_map(|p| p.chars())) {
        if c != UNKNOWN {
            required[(c as u8 - b'a') as usize] = true;
        }
    }
    excluded
        .chars()
        .find(|&c| required[(c as u8 - b'a') as usize])
}

fn conflicting_position(correct: &str, misplaced: &[String]) -> Option<(char, usize)> {
    for pattern in misplaced {
        for (i, (g, m)) in correct.chars().zip(pattern.chars()).enumerate() {
            if g != UNKNOWN && g == m {
                return Some((g, i));
            }
        }
    }
    None
}

fn validate_correct(input: &str) -> Result<String, SolverError> {
    if input.len() != 5 {
        return Err(SolverError::InvalidLength(input.len()));
    }
    let mut result = String::with_capacity(5);
    for c in input.chars() {
        result.push(sanitize_letter(c, true)?);
    }
    Ok(result)
}

pub fn sanitize_letter(letter: char, ignore_placeholder: bool) -> Result<char, SolverError> {
    if ignore_placeholder && letter == UNKNOWN {
        Ok(UNKNOWN)
    } else if letter.is_ascii_lowercase() {
        Ok(letter)
    } else if letter.is_ascii_uppercase() {
        Ok(letter.to_ascii_lowercase())
    } else {
        Err(SolverError::InvalidCharacter(letter))
    }
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
        result.push(sanitize_letter(c, false)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! sanitize_letter_case {
        ($name:ident, $letter:expr, $ignore:expr => $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(sanitize_letter($letter, $ignore), $expected);
            }
        };
    }

    macro_rules! validate_error {
        ($name:ident, $correct:expr, $misplaced:expr, $excluded:expr => $err:expr) => {
            #[test]
            fn $name() {
                let e = validate($correct, $misplaced, $excluded).unwrap_err();
                assert_eq!(e, $err);
            }
        };
    }

    macro_rules! validate_ok {
        ($name:ident, $correct:expr, $misplaced:expr, $excluded:expr) => {
            #[test]
            fn $name() {
                assert!(validate($correct, $misplaced, $excluded).is_ok());
            }
        };
    }

    sanitize_letter_case!(test_sanitize_letter_lowercase, 'a', true => Ok('a'));
    sanitize_letter_case!(test_sanitize_letter_uppercase, 'A', true => Ok('a'));
    sanitize_letter_case!(test_sanitize_letter_placeholder_allowed, ' ', true => Ok(' '));
    sanitize_letter_case!(test_sanitize_letter_placeholder_forbidden, ' ', false => Err(SolverError::InvalidCharacter(' ')));
    sanitize_letter_case!(test_sanitize_letter_invalid_allowed, '.', true => Err(SolverError::InvalidCharacter('.')));
    sanitize_letter_case!(test_sanitize_letter_invalid_forbidden, '.', false => Err(SolverError::InvalidCharacter('.')));
    sanitize_letter_case!(test_sanitize_letter_unicode_allowed, 'ñ', true => Err(SolverError::InvalidCharacter('ñ')));

    #[test]
    fn test_validate_correct_with_unknown() {
        let (correct, _, _) = validate("a p l", &[], "").unwrap();
        assert_eq!(correct, "a p l");
    }

    validate_error!(test_validate_correct_invalid_length_short, "abc", &[], "" => SolverError::InvalidLength(3));
    validate_error!(test_validate_correct_invalid_length_long, "abcdef", &[], "" => SolverError::InvalidLength(6));

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

    validate_error!(test_validate_misplaced_invalid_length, "     ", &["abc"], "" => SolverError::InvalidLength(3));
    validate_error!(test_validate_misplaced_invalid_character, "     ", &["a.b.d"], "" => SolverError::InvalidCharacter('.'));
    validate_error!(test_validate_all_empty, "     ", &[], "" => SolverError::EmptyInputs);

    validate_error!(test_validate_conflict_correct_and_excluded, "a    ", &[], "a" => SolverError::ConflictingLetter('a'));
    validate_error!(test_validate_conflict_misplaced_and_excluded, "     ", &["b    "], "b" => SolverError::ConflictingLetter('b'));
    validate_error!(test_validate_conflict_first_match_wins, "a    ", &[], "ab" => SolverError::ConflictingLetter('a'));
    validate_error!(test_validate_conflict_case_unified, "a    ", &[], "A" => SolverError::ConflictingLetter('a'));
    validate_error!(test_validate_conflict_same_letter_same_position, "a    ", &["a    "], "" => SolverError::ConflictingPosition('a', 0));
    validate_error!(test_validate_conflict_position_middle_index, "  a  ", &["  a  "], "" => SolverError::ConflictingPosition('a', 2));
    validate_error!(test_validate_conflict_letter_takes_precedence, "a    ", &["a    "], "a" => SolverError::ConflictingLetter('a'));

    validate_ok!(test_validate_no_conflict_separate_letters, "a    ", &["b    "], "c");
    validate_ok!(test_validate_no_conflict_same_letter_other_positions, "a    ", &[" a   "], "");
    validate_ok!(test_validate_no_conflict_required_in_both, " a   ", &["a    "], "");
    validate_ok!(test_validate_not_all_empty_correct, "a    ", &[], "");
    validate_ok!(test_validate_not_all_empty_misplaced, "     ", &["a    "], "");
    validate_ok!(test_validate_not_all_empty_excluded, "     ", &[], "a");
    validate_error!(test_validate_multiple_errors_first_error, "abc", &["def"], "ghi.jkl" => SolverError::InvalidLength(3));

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
    fn test_all_empty_misplaced_only_spaces() {
        assert!(all_empty("     ", &["     ", "     "], ""));
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
