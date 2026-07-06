//! `notez search` / `findz`: full-text search across every note source.

use anyhow::Result;
use console::Style;

use notez_core::config::{Config, ProjectRegistry};
use notez_core::core::aggregate::SourceKind;
use notez_core::search::search_notes;
use notez_core::util::tilde;

pub fn run(term: String, config: &Config) -> Result<()> {
    let registry = ProjectRegistry::load().unwrap_or_default();
    let hits = search_notes(&term, config, &registry)?;

    if hits.is_empty() {
        println!("no matches for \"{term}\"");
        return Ok(());
    }

    let path_style = Style::new().color256(110);
    let line_style = Style::new().color256(244);
    let scope_style = Style::new().color256(140);

    for hit in &hits {
        let source = match hit.entry.kind {
            SourceKind::Doc => "docs".to_string(),
            SourceKind::Note => hit.entry.scope.to_string(),
        };
        let scope_label = match &hit.entry.project {
            Some(p) => format!("{} ({})", p, source),
            None => source,
        };
        let location = format!("{}:{}", tilde::contract(&hit.entry.path), hit.line);
        let extra = if hit.match_count > 1 {
            format!("  (+{} more)", hit.match_count - 1)
        } else {
            String::new()
        };
        println!(
            "{}  {}{}",
            path_style.apply_to(location),
            scope_style.apply_to(scope_label),
            line_style.apply_to(extra),
        );
        if !hit.snippet.is_empty() {
            println!("    {}", hit.snippet);
        }
    }
    println!("\n{} file(s) matched \"{term}\"", hits.len());
    Ok(())
}
