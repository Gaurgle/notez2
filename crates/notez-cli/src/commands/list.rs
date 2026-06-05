//! `notez list`: print registered projects on this machine.

use anyhow::Result;
use console::Style;

use notez_core::config::{NotezMetadata, ProjectRegistry};
use notez_core::config::paths;
use notez_core::config::Config;

pub fn run(config: &Config) -> Result<()> {
    let reg = ProjectRegistry::load()?;
    let metadata_path = paths::metadata_file(&config.notez_root_path());
    let meta = NotezMetadata::load_from(&metadata_path)?;

    if reg.projects.is_empty() {
        println!("No projects attached on this machine.");
        println!("Run `notez attach` inside a project root to register it.");
        return Ok(());
    }

    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let warn = Style::new().yellow();

    println!("{}", bold.apply_to("Attached projects:"));
    for (name, path) in reg.iter_resolved() {
        let display = meta.display_for(name);
        let exists = path.exists();
        let marker = if exists { "" } else { "  (not found)" };
        let marker_style = if exists { Style::new() } else { warn.clone() };
        let path_str = notez_core::util::tilde::contract(&path);

        if display == name {
            println!(
                "  {}  {}{}",
                bold.apply_to(name),
                dim.apply_to(&path_str),
                marker_style.apply_to(marker),
            );
        } else {
            println!(
                "  {}  {}  {}{}",
                bold.apply_to(display),
                dim.apply_to(format!("[{}]", name)),
                dim.apply_to(&path_str),
                marker_style.apply_to(marker),
            );
        }
    }

    Ok(())
}
