//! Text utilities shared by all TUIs.
//!
//! - [`fuzzy_match`]: case-insensitive substring match used by the tree
//!   browser's filename filter and the todoz quick-pick.
//! - [`prev_char_boundary`] / [`next_char_boundary`]: UTF-8-safe cursor
//!   movement over a `String` buffer. `String::insert` and `String::remove`
//!   require byte indices that fall on character boundaries, so any
//!   left-arrow / right-arrow / backspace logic that touches strings with
//!   non-ASCII characters (`å`, `ö`, emoji) must use these helpers.
//! - [`mouse_x_to_filter_dot`]: maps a mouse column onto the 5-slot filter
//!   strip the tree and todoz both render. The anchor is documented inline.

/// Case-insensitive substring match. Returns true when `needle` is empty.
pub fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Return the byte index of the previous char boundary at or before `idx`.
///
/// Returns 0 when `idx` is 0 or already past the start.
pub fn prev_char_boundary(s: &str, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }
    let mut i = idx.min(s.len()).saturating_sub(1);
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Return the byte index of the next char boundary after `idx`.
///
/// Returns `s.len()` when `idx` is at or past the end.
pub fn next_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    let mut i = idx + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Map a mouse x-column onto one of the 5 tag-filter dot positions.
///
/// The filter strip renders five dots at fixed positions starting at
/// `column_offset`. Each dot occupies one cell and has one cell of trailing
/// space, so the strip spans 10 columns total. Returns `Some(0..5)` when
/// the mouse falls on a dot, `None` when it falls between dots or outside
/// the strip.
///
/// notez-cli anchors the filter strip at `column_offset = x_origin + 5`:
/// 4 columns of highlight indent on each row plus 1 column for the
/// flag-leading space. That alignment makes the strip's dots line up
/// vertically with each row's flag dots, so a click on the strip's column 3
/// also matches column 3 of any row.
pub fn mouse_x_to_filter_dot(mouse_x: u16, strip_x: u16) -> Option<u8> {
    if mouse_x < strip_x {
        return None;
    }
    let rel = (mouse_x - strip_x) as usize;
    if rel >= 10 {
        return None;
    }
    // Even offsets (0, 2, 4, 6, 8) are dot positions.
    if rel % 2 != 0 {
        return None;
    }
    Some((rel / 2) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_empty_needle_matches_anything() {
        assert!(fuzzy_match("anything", ""));
        assert!(fuzzy_match("", ""));
    }

    #[test]
    fn fuzzy_match_is_case_insensitive() {
        assert!(fuzzy_match("Hello World", "hello"));
        assert!(fuzzy_match("hello world", "WORLD"));
    }

    #[test]
    fn fuzzy_match_substring() {
        assert!(fuzzy_match("hello-world.md", "world"));
        assert!(!fuzzy_match("hello-world.md", "earth"));
    }

    #[test]
    fn prev_boundary_handles_ascii() {
        let s = "abcdef";
        assert_eq!(prev_char_boundary(s, 0), 0);
        assert_eq!(prev_char_boundary(s, 1), 0);
        assert_eq!(prev_char_boundary(s, 3), 2);
        assert_eq!(prev_char_boundary(s, 6), 5);
        assert_eq!(prev_char_boundary(s, 100), 5);
    }

    #[test]
    fn prev_boundary_handles_multibyte() {
        // "åö" is 4 bytes: å = 2, ö = 2.
        let s = "åö";
        assert_eq!(s.len(), 4);
        // From byte 4 (end), prev boundary is 2 (start of ö).
        assert_eq!(prev_char_boundary(s, 4), 2);
        // From byte 2 (start of ö), prev boundary is 0.
        assert_eq!(prev_char_boundary(s, 2), 0);
        // From byte 3 (middle of ö), prev boundary is 2.
        assert_eq!(prev_char_boundary(s, 3), 2);
    }

    #[test]
    fn next_boundary_handles_ascii() {
        let s = "abc";
        assert_eq!(next_char_boundary(s, 0), 1);
        assert_eq!(next_char_boundary(s, 2), 3);
        assert_eq!(next_char_boundary(s, 3), 3);
        assert_eq!(next_char_boundary(s, 100), 3);
    }

    #[test]
    fn next_boundary_handles_multibyte() {
        let s = "åö"; // 4 bytes
        // From 0, next boundary is 2 (after å).
        assert_eq!(next_char_boundary(s, 0), 2);
        // From 2, next boundary is 4.
        assert_eq!(next_char_boundary(s, 2), 4);
        // From 1 (middle of å), next boundary is 2.
        assert_eq!(next_char_boundary(s, 1), 2);
    }

    #[test]
    fn mouse_dot_maps_to_each_of_five_positions() {
        // Strip starts at column 5. Dots are at columns 5, 7, 9, 11, 13.
        assert_eq!(mouse_x_to_filter_dot(5, 5), Some(0));
        assert_eq!(mouse_x_to_filter_dot(7, 5), Some(1));
        assert_eq!(mouse_x_to_filter_dot(9, 5), Some(2));
        assert_eq!(mouse_x_to_filter_dot(11, 5), Some(3));
        assert_eq!(mouse_x_to_filter_dot(13, 5), Some(4));
    }

    #[test]
    fn mouse_dot_returns_none_between_dots() {
        assert_eq!(mouse_x_to_filter_dot(6, 5), None);
        assert_eq!(mouse_x_to_filter_dot(8, 5), None);
        assert_eq!(mouse_x_to_filter_dot(12, 5), None);
    }

    #[test]
    fn mouse_dot_returns_none_before_strip() {
        assert_eq!(mouse_x_to_filter_dot(0, 5), None);
        assert_eq!(mouse_x_to_filter_dot(4, 5), None);
    }

    #[test]
    fn mouse_dot_returns_none_past_strip() {
        // Strip occupies columns 5..15 (10 cells), so anything at 15+ is past.
        assert_eq!(mouse_x_to_filter_dot(15, 5), None);
        assert_eq!(mouse_x_to_filter_dot(100, 5), None);
    }
}
