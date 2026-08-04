pub mod entries;
pub mod serialization;

pub fn subsequence_match_ignore_case(src: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();

    for c in src.chars() {
        let Some(pattern_c) = pattern_chars.peek() else {
            return true;
        };

        if pattern_c.to_ascii_lowercase() == c.to_ascii_lowercase() {
            pattern_chars.next();
        }
    }

    return !pattern_chars.peek().is_some();
}