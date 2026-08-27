use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SolverError {
    InvalidCharacter(char),
    InvalidLength(usize),
    EmptyInputs,
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::InvalidCharacter(c) => write!(f, "Invalid character: '{}'", c),
            SolverError::InvalidLength(len) => write!(f, "Invalid length: {}, expected 5", len),
            SolverError::EmptyInputs => write!(f, "All inputs are empty"),
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
    fn test_debug_format() {
        let err = SolverError::InvalidCharacter('a');
        assert_eq!(format!("{:?}", err), "InvalidCharacter('a')");
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(
            SolverError::InvalidCharacter('a'),
            SolverError::InvalidCharacter('a')
        );
        assert_ne!(
            SolverError::InvalidCharacter('a'),
            SolverError::InvalidCharacter('b')
        );
        assert_eq!(SolverError::EmptyInputs, SolverError::EmptyInputs);
    }
}
