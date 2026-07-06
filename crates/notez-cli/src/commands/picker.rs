//! Shared interactive line picker: fzf when available, a numbered stdin
//! prompt otherwise (the fallback the legacy README promises).

use anyhow::{Context, Result, bail};

/// Present `lines` and return the selected index.
///
/// Lines are fed to fzf with an index prefix and mapped back by parsing it,
/// so duplicate display strings can never be misresolved.
pub fn pick(prompt: &str, lines: &[String], use_fzf: bool) -> Result<usize> {
    if lines.is_empty() {
        bail!("nothing to pick from");
    }
    if lines.len() == 1 {
        return Ok(0);
    }
    if use_fzf {
        pick_fzf(prompt, lines)
    } else {
        pick_numbered(lines)
    }
}

fn pick_fzf(prompt: &str, lines: &[String]) -> Result<usize> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let input: String = lines
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{:>4}  {}", i + 1, l))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .args(["--prompt", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch fzf")?;
    child
        .stdin
        .as_mut()
        .expect("stdin was piped")
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("picker cancelled");
    }
    let selected = String::from_utf8_lossy(&output.stdout);
    match parse_index(&selected) {
        Some(i) if i >= 1 && i <= lines.len() => Ok(i - 1),
        _ => bail!("picker cancelled"),
    }
}

fn pick_numbered(lines: &[String]) -> Result<usize> {
    use std::io::Write;

    for (i, l) in lines.iter().enumerate() {
        println!("  {:>3}  {}", i + 1, l);
    }
    print!("select (1-{}): ", lines.len());
    std::io::stdout().flush().ok();

    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .context("failed to read selection")?;
    match answer.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= lines.len() => Ok(n - 1),
        _ => bail!("invalid selection"),
    }
}

/// Parse the leading index out of a picked line (`"  12  label"` -> 12).
fn parse_index(line: &str) -> Option<usize> {
    line.trim_start()
        .split_whitespace()
        .next()?
        .parse::<usize>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_leading_index() {
        assert_eq!(parse_index("   3  ~/notez/foo.md"), Some(3));
        assert_eq!(parse_index("12  a  b"), Some(12));
        assert_eq!(parse_index("nope"), None);
    }

    #[test]
    fn single_candidate_skips_the_picker() {
        let one = vec!["only".to_string()];
        assert_eq!(pick("x> ", &one, true).unwrap(), 0);
    }

    #[test]
    fn empty_candidates_bail() {
        assert!(pick("x> ", &[], true).is_err());
    }
}
