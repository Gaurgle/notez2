//! External tool detection (yazi, fzf, rg).
//!
//! notez uses these tools when available and falls back to built-in behavior
//! when they are not. Detection runs once during setup and is cached in the
//! config; runtime checks may re-detect if a tool became available since.

/// Return `true` if the named binary is on `PATH`.
pub fn detect(name: &str) -> bool {
    which::which(name).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_a_tool_known_to_exist_or_not() {
        // We can't assume any particular tool exists in test env, but `sh`
        // is essentially guaranteed on Unix-like CI runners.
        #[cfg(unix)]
        assert!(detect("sh"));
    }

    #[test]
    fn missing_tool_returns_false() {
        assert!(!detect("definitely-not-a-real-binary-zzzz"));
    }
}
