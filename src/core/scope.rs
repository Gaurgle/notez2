//! Note scope: private, public, or global.
//!
//! Every notez command resolves to a scope:
//!
//! - `Private`: under `<cwd>/.notez/`, auto-gitignored
//! - `Public`: under `<cwd>/notez/`, committed with the project
//! - `Global`: under `~/notez/`, the user's cross-project notes

use std::fmt;

/// The three note scopes.
///
/// Derived from CLI flags: default = `Private`, `-p` = `Public`, `-g` = `Global`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Private,
    Public,
    Global,
}

impl Scope {
    /// Resolve from the two boolean CLI flags `-g` and `-p`.
    ///
    /// `-g` wins over `-p` (you can't have a "global public" scope; global
    /// is its own thing). This matches notez-cli's behavior.
    pub fn from_flags(global: bool, public: bool) -> Self {
        if global {
            Self::Global
        } else if public {
            Self::Public
        } else {
            Self::Private
        }
    }

    /// The directory name relative to a project root.
    ///
    /// Returns `.notez` for private, `notez` for public. Global scope has no
    /// project-relative directory (it lives at `~/notez/`); calling this on
    /// `Global` returns `None`.
    pub fn project_subdir(&self) -> Option<&'static str> {
        match self {
            Self::Private => Some(".notez"),
            Self::Public => Some("notez"),
            Self::Global => None,
        }
    }

    /// Lock-icon for private, globe-icon for public/global.
    ///
    /// Uses the same nerdfont glyphs as notez-cli.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Private => "\u{f023}", // lock
            Self::Public | Self::Global => "\u{f0ac}", // globe
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Private => "private",
            Self::Public => "public",
            Self::Global => "global",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_private() {
        assert_eq!(Scope::from_flags(false, false), Scope::Private);
    }

    #[test]
    fn public_flag_picks_public() {
        assert_eq!(Scope::from_flags(false, true), Scope::Public);
    }

    #[test]
    fn global_flag_picks_global() {
        assert_eq!(Scope::from_flags(true, false), Scope::Global);
    }

    #[test]
    fn global_wins_over_public() {
        assert_eq!(Scope::from_flags(true, true), Scope::Global);
    }

    #[test]
    fn project_subdirs() {
        assert_eq!(Scope::Private.project_subdir(), Some(".notez"));
        assert_eq!(Scope::Public.project_subdir(), Some("notez"));
        assert_eq!(Scope::Global.project_subdir(), None);
    }

    #[test]
    fn icons_differ_per_scope() {
        assert_ne!(Scope::Private.icon(), Scope::Public.icon());
        assert_eq!(Scope::Public.icon(), Scope::Global.icon());
    }
}
