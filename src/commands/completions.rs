//! `notez completions <shell>`: print shell-completion scripts to stdout.

use std::io;

use anyhow::{Result, bail};
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::Cli;

pub fn run(shell: &str) -> Result<()> {
    let s = match shell.to_lowercase().as_str() {
        "zsh" => Shell::Zsh,
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        other => bail!("unsupported shell: {} (try zsh, bash, fish)", other),
    };

    let mut cmd = Cli::command();
    generate(s, &mut cmd, "notez", &mut io::stdout());
    Ok(())
}
