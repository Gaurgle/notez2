//! `notez init <shell>`: print shell-integration snippets for `eval` consumption.

use anyhow::{Result, bail};

pub fn run(shell: &str) -> Result<()> {
    match shell.to_lowercase().as_str() {
        "zsh" => {
            // noglob wrappers so users can write `zlog did you fix the bug?`
            // without zsh trying to glob-expand `?` and `*`.
            println!("# notez zsh integration");
            println!("alias zlog='noglob zlog'");
            println!("alias znote='noglob znote'");
        }
        "bash" | "fish" => {
            println!("# notez {} integration", shell);
            println!("# (no helpers needed; bash/fish do not glob-expand command args)");
        }
        other => bail!("unsupported shell: {} (try zsh, bash, fish)", other),
    }
    Ok(())
}
