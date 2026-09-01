use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct WordBitmask(u32);

impl WordBitmask {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn push(&mut self, c: char) {
        if c.is_ascii_lowercase() {
            self.0 |= 1 << (c as u8 - b'a');
        }
    }

    pub const fn contains(&self, c: char) -> bool {
        c.is_ascii_lowercase() && (self.0 & (1 << (c as u8 - b'a'))) != 0
    }
}

impl FromStr for WordBitmask {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mask = WordBitmask::new();
        for c in s.chars() {
            mask.push(c);
        }
        Ok(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let mask = WordBitmask::new();
        assert!(!mask.contains('a'));
        assert!(!mask.contains('z'));
    }

    #[test]
    fn test_push_then_contains() {
        let mut mask = WordBitmask::new();
        mask.push('a');
        assert!(mask.contains('a'));
        assert!(!mask.contains('b'));
    }

    #[test]
    fn test_push_multiple_letters() {
        let mut mask = WordBitmask::new();
        mask.push('c');
        mask.push('x');
        assert!(mask.contains('c'));
        assert!(mask.contains('x'));
        assert!(!mask.contains('d'));
    }

    #[test]
    fn test_push_ignores_non_lowercase() {
        let mut mask = WordBitmask::new();
        mask.push('A');
        mask.push(' ');
        mask.push('.');
        assert!(!mask.contains('a'));
        assert!(!mask.contains(' '));
        assert!(!mask.contains('A'));
    }

    #[test]
    fn test_contains_ignores_non_lowercase() {
        let mask = WordBitmask::from_str("a").unwrap();
        assert!(!mask.contains('A'));
        assert!(!mask.contains(' '));
        assert!(!mask.contains('.'));
    }

    #[test]
    fn test_push_deduplicates() {
        let mut mask = WordBitmask::new();
        mask.push('e');
        mask.push('e');
        assert!(mask.contains('e'));
        assert_eq!(mask, WordBitmask::from_str("e").unwrap());
    }

    #[test]
    fn test_from_str_builds_mask() {
        let mask = WordBitmask::from_str("a c").unwrap();
        assert!(mask.contains('a'));
        assert!(mask.contains('c'));
        assert!(!mask.contains('b'));
        assert!(!mask.contains(' '));
    }

    #[test]
    fn test_default_is_empty() {
        let mask = WordBitmask::default();
        assert_eq!(mask, WordBitmask::new());
        assert!(!mask.contains('m'));
    }
}
