//! Tag system shared by tree and todoz.
//!
//! Five tag flags packed into a `u8`. Each tag has a hashtag spelling that
//! survives a round-trip through TODO.md and `.tags` files. The colors are
//! defined alongside in [`super::theme`] to keep palette and tag-meaning
//! in sync.
//!
//! Filter syntax is implemented separately in [`super::filter`]; this
//! module only owns the bit definitions, parse, and serialize.

use ratatui::style::Color;

use super::theme;

/// Definition of one tag flag.
pub struct FlagDef {
    pub bit: u8,
    /// Short display label (e.g. `"important"`).
    pub label: &'static str,
    /// Hashtag-style key used in markdown (without the leading `#`).
    pub key: &'static str,
}

pub const FLAG_IMPORTANT: u8 = 1 << 0;
pub const FLAG_PRIO: u8 = 1 << 1;
pub const FLAG_LONGTERM: u8 = 1 << 2;
pub const FLAG_IDEA: u8 = 1 << 3;
pub const FLAG_BLOCKED: u8 = 1 << 4;

pub const FLAG_DEFS: [FlagDef; 5] = [
    FlagDef {
        bit: FLAG_IMPORTANT,
        label: "important",
        key: "important",
    },
    FlagDef {
        bit: FLAG_PRIO,
        label: "priority",
        key: "prio",
    },
    FlagDef {
        bit: FLAG_LONGTERM,
        label: "long-term",
        key: "longterm",
    },
    FlagDef {
        bit: FLAG_IDEA,
        label: "idea",
        key: "idea",
    },
    FlagDef {
        bit: FLAG_BLOCKED,
        label: "blocked",
        key: "blocked",
    },
];

/// The 5 colors aligned 1:1 with [`FLAG_DEFS`]. Re-exported from
/// [`super::theme::FLAG_COLORS`] so callers do not need to import both.
pub const FLAG_COLORS: [Color; 5] = theme::FLAG_COLORS;

/// Parse `#tag` markers out of `text` and return `(stripped_text, flags)`.
///
/// Strips any trailing `#prio`, `#important`, `#longterm`, `#idea`,
/// `#blocked` markers (case-insensitive, separated by whitespace). The
/// returned string has the surrounding whitespace tidied so the line can
/// be rewritten cleanly.
pub fn parse_flags(text: &str) -> (String, u8) {
    let mut flags: u8 = 0;
    let mut kept: Vec<&str> = Vec::new();

    for word in text.split_whitespace() {
        let lower = word.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix('#') {
            if let Some(d) = FLAG_DEFS.iter().find(|d| d.key.eq_ignore_ascii_case(rest)) {
                flags |= d.bit;
                continue;
            }
        }
        kept.push(word);
    }

    (kept.join(" "), flags)
}

/// Append `#tag` markers for set bits to `text`. Markers are emitted in
/// [`FLAG_DEFS`] order so the result is stable across calls.
pub fn serialize_flags(text: &str, flags: u8) -> String {
    let mut out = text.trim_end().to_string();
    for def in &FLAG_DEFS {
        if flags & def.bit != 0 {
            out.push(' ');
            out.push('#');
            out.push_str(def.key);
        }
    }
    out
}

/// Render the 5 dot slots as a `(Color, &str)` per slot, with set bits
/// showing the tag color and unset bits showing a dim middle-dot.
///
/// Used by both tree and todoz to render the per-row tag column. The
/// caller is responsible for laying out the cells with the correct
/// spacing (one space between dots).
pub fn flags_slots(flags: u8) -> [(Color, &'static str); 5] {
    let mut out = [(Color::Reset, "·"); 5];
    for (i, def) in FLAG_DEFS.iter().enumerate() {
        if flags & def.bit != 0 {
            out[i] = (FLAG_COLORS[i], "\u{f111}"); // nerdfont filled dot
        } else {
            out[i] = (theme::OVERLAY, "·");
        }
    }
    out
}

/// Toggle a single bit in `flags`.
pub fn toggle(flags: u8, bit: u8) -> u8 {
    flags ^ bit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_strips_known_hashtags() {
        let (text, flags) = parse_flags("buy milk #prio #important");
        assert_eq!(text, "buy milk");
        assert_eq!(flags, FLAG_IMPORTANT | FLAG_PRIO);
    }

    #[test]
    fn parse_keeps_unknown_hashtags() {
        let (text, flags) = parse_flags("buy milk #unknown #prio");
        assert_eq!(text, "buy milk #unknown");
        assert_eq!(flags, FLAG_PRIO);
    }

    #[test]
    fn parse_is_case_insensitive() {
        let (_, flags) = parse_flags("foo #IMPORTANT #Idea");
        assert_eq!(flags, FLAG_IMPORTANT | FLAG_IDEA);
    }

    #[test]
    fn parse_empty_text_returns_no_flags() {
        let (text, flags) = parse_flags("");
        assert_eq!(text, "");
        assert_eq!(flags, 0);
    }

    #[test]
    fn parse_handles_hashtags_anywhere() {
        // Tags interspersed; word order otherwise preserved.
        let (text, flags) = parse_flags("#prio buy #important milk");
        assert_eq!(text, "buy milk");
        assert_eq!(flags, FLAG_IMPORTANT | FLAG_PRIO);
    }

    #[test]
    fn serialize_emits_tags_in_canonical_order() {
        let s = serialize_flags("buy milk", FLAG_BLOCKED | FLAG_IMPORTANT);
        assert_eq!(s, "buy milk #important #blocked");
    }

    #[test]
    fn serialize_with_no_flags_is_identity() {
        let s = serialize_flags("buy milk", 0);
        assert_eq!(s, "buy milk");
    }

    #[test]
    fn serialize_trims_trailing_space_before_appending() {
        let s = serialize_flags("buy milk   ", FLAG_PRIO);
        assert_eq!(s, "buy milk #prio");
    }

    #[test]
    fn round_trip_preserves_text_and_flags() {
        let original_text = "fix the build";
        let original_flags = FLAG_IMPORTANT | FLAG_IDEA;
        let line = serialize_flags(original_text, original_flags);
        let (text, flags) = parse_flags(&line);
        assert_eq!(text, original_text);
        assert_eq!(flags, original_flags);
    }

    #[test]
    fn toggle_sets_and_clears() {
        let f = 0;
        let f = toggle(f, FLAG_PRIO);
        assert_eq!(f, FLAG_PRIO);
        let f = toggle(f, FLAG_PRIO);
        assert_eq!(f, 0);
    }

    #[test]
    fn slots_distinguish_set_and_unset() {
        let slots = flags_slots(FLAG_IMPORTANT | FLAG_BLOCKED);
        // Important is set (slot 0), blocked is set (slot 4), rest unset.
        assert_ne!(slots[0].1, "·");
        assert_eq!(slots[1].1, "·");
        assert_eq!(slots[2].1, "·");
        assert_eq!(slots[3].1, "·");
        assert_ne!(slots[4].1, "·");
    }
}
