//! Filename and project-name sanitization.
//!
//! notez accepts free-form titles and project names from user input. They are
//! sanitized into filesystem-safe slugs: lowercased, whitespace collapsed to
//! hyphens, only alphanumerics and hyphens kept.

/// Sanitize a free-form name into a filesystem-safe slug.
///
/// - Trims surrounding whitespace
/// - Lowercases
/// - Splits on whitespace and joins with `-`
/// - Strips non-alphanumeric non-hyphen characters
pub fn name(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases() {
        assert_eq!(name("MyProject"), "myproject");
    }

    #[test]
    fn collapses_whitespace_to_hyphen() {
        assert_eq!(name("my  cool  idea"), "my-cool-idea");
    }

    #[test]
    fn strips_special_chars() {
        assert_eq!(name("hello, world!"), "hello-world");
    }

    #[test]
    fn keeps_existing_hyphens() {
        assert_eq!(name("foo-bar"), "foo-bar");
    }

    #[test]
    fn empty_input() {
        assert_eq!(name(""), "");
        assert_eq!(name("   "), "");
    }

    #[test]
    fn unicode_alphanumerics_kept() {
        assert_eq!(name("åäö"), "åäö");
    }

    #[test]
    fn drops_underscores() {
        assert_eq!(name("foo_bar"), "foobar");
    }
}
