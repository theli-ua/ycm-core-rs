pub mod identifier;

pub fn byte_off_to_unicode_off(s: &str, byte_off: usize) -> usize {
    unsafe { std::str::from_utf8_unchecked(&s.as_bytes()[..byte_off - 1]) }
        .chars()
        .count()
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_off() {
        let cases = [
            // Simple ascii strings.
            (("test", 1), 1),
            (("test", 4), 4),
            (("test", 5), 5),
            // Unicode char at beginning.
            (("†est", 1), 1),
            (("†est", 4), 2),
            (("†est", 6), 4),
            (("†est", 7), 5),
            // Unicode char at end.
            (("tes†", 1), 1),
            (("tes†", 2), 2),
            (("tes†", 4), 4),
            (("tes†", 7), 5),
            // Unicode char in middle.
            (("tes†ing", 1), 1),
            (("tes†ing", 2), 2),
            (("tes†ing", 4), 4),
            (("tes†ing", 7), 5),
            (("tes†ing", 9), 7),
            (("tes†ing", 10), 8),
        ];
        for ((s, n), expected) in std::array::IntoIter::new(cases) {
            println!("case: {}, {}", s, n);
            assert_eq!(byte_off_to_unicode_off(s, n), expected);
        }
    }
}
