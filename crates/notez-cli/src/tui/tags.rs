//! TUI rendering glue for the tag system.
//!
//! The pure flag logic — bit definitions, parse, serialize, toggle — lives
//! in [`notez_core::tags`]. This module re-exports it and adds the ratatui
//! color rendering used by the tree and todoz TUIs, keeping the core free
//! of any terminal dependency.

use ratatui::style::Color;

pub use notez_core::tags::*;

use super::theme;

/// The 5 colors aligned 1:1 with [`notez_core::tags::FLAG_DEFS`].
pub const FLAG_COLORS: [Color; 5] = theme::FLAG_COLORS;

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

#[cfg(test)]
mod tests {
    use super::*;

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
