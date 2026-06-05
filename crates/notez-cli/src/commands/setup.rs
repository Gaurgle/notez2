//! `notez setup`: interactive setup wizard.
//!
//! Minimal stub for the first milestone: writes the defaults to disk if the
//! config file is missing. A full interactive wizard with prompts comes next.

use anyhow::Result;

use notez_core::config::Config;

pub fn run() -> Result<()> {
    let path = notez_core::config::paths::config_file();
    if path.exists() {
        println!("Config already exists at {}.", path.display());
        println!("Interactive wizard coming in the next milestone.");
        println!("For now, edit the file directly or delete it to re-create defaults.");
        return Ok(());
    }

    let cfg = Config::defaults();
    cfg.save()?;
    println!("Wrote default config to {}.", path.display());
    println!("Notez root: {}", cfg.notez_root_path().display());
    println!("Tools detected: fzf={}, rg={}, yazi={}", cfg.tools.fzf, cfg.tools.rg, cfg.tools.yazi);
    println!();
    println!("Run `notez attach` inside a project root to register it on this machine.");
    Ok(())
}
