use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SolverError {
    InvalidCharacter(char),
    InvalidLength(usize),
    EmptyInputs,
    ConflictingLetter(char),
    ConflictingPosition(char, usize),
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::InvalidCharacter(c) => write!(f, "Invalid character: '{}'", c),
            SolverError::InvalidLength(len) => write!(f, "Invalid length: {}, expected 5", len),
            SolverError::EmptyInputs => write!(f, "All inputs are empty"),
            SolverError::ConflictingLetter(c) => {
                write!(f, "Letter '{}' cannot be both required and excluded", c)
            }
            SolverError::ConflictingPosition(c, pos) => {
                write!(f, "Letter '{}' cannot be both correct and misplaced at position {}", c, pos + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_invalid_character() {
        let err = SolverError::InvalidCharacter('ñ');
        assert_eq!(format!("{}", err), "Invalid character: 'ñ'");
    }

    #[test]
    fn test_display_invalid_length() {
        let err = SolverError::InvalidLength(3);
        assert_eq!(format!("{}", err), "Invalid length: 3, expected 5");
    }

    #[test]
    fn test_display_empty_inputs() {
        let err = SolverError::EmptyInputs;
        assert_eq!(format!("{}", err), "All inputs are empty");
    }

    #[test]
    fn test_display_conflicting_letter() {
        let err = SolverError::ConflictingLetter('a');
        assert_eq!(format!("{}", err), "Letter 'a' cannot be both required and excluded");
    }

    #[test]
    fn test_display_conflicting_position() {
        let err = SolverError::ConflictingPosition('a', 0);
        assert_eq!(format!("{}", err), "Letter 'a' cannot be both correct and misplaced at position 1");
    }
}
