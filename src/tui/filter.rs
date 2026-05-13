//! Filter parsing for the TUI search input.
//!
//! Users type into a single search buffer that can mix free text (fuzzy
//! matched against item text) and `#tag` tokens (matched against the
//! 5-bit flag column). Multiple tokens combine with **AND across tokens
//! but OR within a token**:
//!
//! - `important fix` matches items whose text contains both substrings.
//! - `#prio #blocked` matches items tagged BOTH prio AND blocked.
//! - `#13` matches items tagged tag-1 OR tag-3 (a single token, "OR within").
//! - `important #1` matches items containing "important" in text AND
//!   tagged with tag-1.
//!
//! Prefix matching for tag names: `#i` matches both `#important` and
//! `#idea`; `#imp` matches only `#important`. A bare `#` matches anything
//! that has any tag set.

use super::tags::{FLAG_DEFS, parse_flags};

/// Parsed filter: text substrings to match (AND) plus tag sets to match
/// (each set OR'd internally, sets AND'd across).
pub struct Filter {
    pub text_tokens: Vec<String>,
    pub tag_sets: Vec<u8>,
}

impl Filter {
    /// Returns true when the filter has no text and no tag constraints.
    pub fn is_empty(&self) -> bool {
        self.text_tokens.is_empty() && self.tag_sets.is_empty()
    }

    /// Match `text` and `flags` against this filter.
    pub fn matches(&self, text: &str, flags: u8) -> bool {
        let lower = text.to_lowercase();
        for tok in &self.text_tokens {
            if !lower.contains(&tok.to_lowercase()) {
                return false;
            }
        }
        for set in &self.tag_sets {
            if flags & set == 0 {
                return false;
            }
        }
        true
    }
}

/// Split a raw filter buffer into a [`Filter`].
pub fn parse(buffer: &str) -> Filter {
    let mut text_tokens: Vec<String> = Vec::new();
    let mut tag_sets: Vec<u8> = Vec::new();

    for token in buffer.split_whitespace() {
        if let Some(rest) = token.strip_prefix('#') {
            tag_sets.push(match_tag_token(rest));
        } else {
            text_tokens.push(token.to_string());
        }
    }

    Filter { text_tokens, tag_sets }
}

/// Resolve a single `#token` (without the leading `#`) into a bit set.
///
/// Order of resolution:
/// 1. Empty -> all 5 bits (matches anything tagged at all)
/// 2. All-digit (`13`) -> OR of tags at those 1-based indexes
/// 3. Prefix match against [`FLAG_DEFS`] names (case-insensitive)
/// 4. Unknown -> 0
pub fn match_tag_token(token: &str) -> u8 {
    if token.is_empty() {
        let mut all = 0u8;
        for def in &FLAG_DEFS {
            all |= def.bit;
        }
        return all;
    }

    if token.chars().all(|c| c.is_ascii_digit()) {
        let mut set = 0u8;
        for ch in token.chars() {
            let idx = (ch as u8 - b'0') as usize;
            if idx >= 1 && idx <= FLAG_DEFS.len() {
                set |= FLAG_DEFS[idx - 1].bit;
            }
        }
        return set;
    }

    let lower = token.to_ascii_lowercase();
    let mut set = 0u8;
    for def in &FLAG_DEFS {
        if def.key.starts_with(&lower) {
            set |= def.bit;
        }
    }
    set
}

/// Toggle the presence of `#tag` (by `FLAG_DEFS` index) in a filter buffer.
///
/// Mirrors notez-cli's behavior: clicking a dot on the filter strip adds
/// or removes a `#tagname` token in the buffer. Used by mouse-driven tag
/// toggling inside the TUI.
pub fn toggle_tag_in_buffer(buffer: &str, tag_idx: usize) -> String {
    let Some(def) = FLAG_DEFS.get(tag_idx) else {
        return buffer.to_string();
    };
    let token = format!("#{}", def.key);

    let mut tokens: Vec<String> = buffer
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut found = false;
    tokens.retain(|t| {
        if t.eq_ignore_ascii_case(&token) {
            found = true;
            false
        } else {
            true
        }
    });

    if !found {
        tokens.push(token);
    }

    tokens.join(" ")
}

/// Flat-OR view of the filter's tag bits. Used by the filter-strip
/// renderer to know which dots to light, without caring about the
/// AND-across-tokens structure.
pub fn active_tag_bits(buffer: &str) -> u8 {
    let f = parse(buffer);
    f.tag_sets.into_iter().fold(0u8, |acc, b| acc | b)
}

/// Round-trip helper: strip any inline `#tag` markers from `text` so the
/// filter matcher does not double-count them once they have been moved to
/// the flags field. Used by callers that have a single string field
/// holding both free text and persisted tags; the TUI normalizes the
/// stored value via [`parse_flags`] when loading.
pub fn cleaned_text_for_matching(text: &str) -> String {
    parse_flags(text).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::tags::{FLAG_BLOCKED, FLAG_IDEA, FLAG_IMPORTANT, FLAG_LONGTERM, FLAG_PRIO};

    #[test]
    fn empty_buffer_is_empty_filter() {
        let f = parse("");
        assert!(f.is_empty());
    }

    #[test]
    fn text_tokens_separated_by_whitespace() {
        let f = parse("foo bar");
        assert_eq!(f.text_tokens, vec!["foo", "bar"]);
        assert!(f.tag_sets.is_empty());
    }

    #[test]
    fn tag_token_resolves_by_name() {
        let f = parse("#important");
        assert_eq!(f.tag_sets, vec![FLAG_IMPORTANT]);
    }

    #[test]
    fn tag_token_prefix_matches_multiple() {
        // "i" prefixes both `important` and `idea`.
        let f = parse("#i");
        assert_eq!(f.tag_sets, vec![FLAG_IMPORTANT | FLAG_IDEA]);
    }

    #[test]
    fn tag_token_digits_resolve_to_indices() {
        // `#13` -> tag 1 OR tag 3.
        let f = parse("#13");
        assert_eq!(f.tag_sets, vec![FLAG_IMPORTANT | FLAG_LONGTERM]);
    }

    #[test]
    fn bare_hash_means_any_tag() {
        let f = parse("#");
        let all = FLAG_IMPORTANT | FLAG_PRIO | FLAG_LONGTERM | FLAG_IDEA | FLAG_BLOCKED;
        assert_eq!(f.tag_sets, vec![all]);
    }

    #[test]
    fn unknown_tag_resolves_to_zero() {
        let f = parse("#zzz");
        assert_eq!(f.tag_sets, vec![0]);
    }

    #[test]
    fn mixed_text_and_tags() {
        let f = parse("buy #prio milk");
        assert_eq!(f.text_tokens, vec!["buy", "milk"]);
        assert_eq!(f.tag_sets, vec![FLAG_PRIO]);
    }

    #[test]
    fn match_text_and_in() {
        let f = parse("buy milk");
        assert!(f.matches("buy milk now", 0));
        assert!(!f.matches("buy bread", 0));
    }

    #[test]
    fn match_tag_sets_use_and_across_tokens() {
        let f = parse("#prio #important");
        // Both tags required.
        assert!(f.matches("x", FLAG_PRIO | FLAG_IMPORTANT));
        assert!(!f.matches("x", FLAG_PRIO));
        assert!(!f.matches("x", FLAG_IMPORTANT));
    }

    #[test]
    fn match_tag_set_uses_or_within_token() {
        // `#13` matches tag 1 OR tag 3.
        let f = parse("#13");
        assert!(f.matches("x", FLAG_IMPORTANT));
        assert!(f.matches("x", FLAG_LONGTERM));
        assert!(!f.matches("x", FLAG_PRIO));
    }

    #[test]
    fn toggle_adds_then_removes() {
        let b = "";
        let b = toggle_tag_in_buffer(b, 0); // important
        assert_eq!(b, "#important");
        let b = toggle_tag_in_buffer(&b, 0);
        assert_eq!(b, "");
    }

    #[test]
    fn toggle_preserves_other_tokens() {
        let b = "buy milk #prio";
        let b = toggle_tag_in_buffer(b, 0); // add important
        assert_eq!(b, "buy milk #prio #important");
    }

    #[test]
    fn toggle_unknown_index_is_noop() {
        let b = "hello";
        let b = toggle_tag_in_buffer(b, 99);
        assert_eq!(b, "hello");
    }

    #[test]
    fn active_tag_bits_collapses_sets() {
        let bits = active_tag_bits("#prio #important");
        assert_eq!(bits, FLAG_PRIO | FLAG_IMPORTANT);
    }

    #[test]
    fn cleaned_text_strips_inline_hashtags() {
        assert_eq!(cleaned_text_for_matching("buy milk #prio"), "buy milk");
    }
}
