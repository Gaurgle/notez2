//! Wire-format DTOs for the frontend.
//!
//! Core types carry `PathBuf`s; the frontend only ever sees plain strings,
//! so paths are stringified at this boundary and never cross IPC as a
//! `PathBuf`-shaped value.

use serde::Serialize;

use notez_core::core::aggregate::{NoteEntry, SourceKind};
use notez_core::core::Scope;
use notez_core::search::SearchHit;
use notez_core::todo::{CheckState, Task};

/// One row in the note list.
#[derive(Serialize)]
pub struct NoteListItem {
    /// Absolute path on disk, as a string.
    pub path: String,
    /// Filename only (e.g. `2026-05-13-my-idea.md`).
    pub name: String,
    /// Which scope this note came from.
    pub scope: Scope,
    /// Owning project, if any.
    pub project: Option<String>,
    /// Note vs project doc ("note" | "doc").
    pub kind: SourceKind,
    /// 5-bit importance tag flags from the root `.tags` file.
    pub flags: u8,
    /// Last-modified time, seconds since the Unix epoch (0 if unavailable).
    pub modified: u64,
}

impl From<&NoteEntry> for NoteListItem {
    fn from(entry: &NoteEntry) -> Self {
        let modified = std::fs::metadata(&entry.path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            path: entry.path.to_string_lossy().into_owned(),
            name: entry.name.clone(),
            scope: entry.scope,
            project: entry.project.clone(),
            kind: entry.kind,
            flags: 0,
            modified,
        }
    }
}

/// One full-text search hit, anchored at its first matching line.
#[derive(Serialize)]
pub struct SearchHitDto {
    pub path: String,
    pub name: String,
    pub scope: Scope,
    pub project: Option<String>,
    pub kind: SourceKind,
    /// 1-based line number of the first match.
    pub line: usize,
    /// The first matching line, trimmed.
    pub snippet: String,
    /// Total matching lines in the file.
    pub match_count: usize,
    /// True when the filename itself also matches the query.
    pub name_match: bool,
}

impl From<&SearchHit> for SearchHitDto {
    fn from(hit: &SearchHit) -> Self {
        Self {
            path: hit.entry.path.to_string_lossy().into_owned(),
            name: hit.entry.name.clone(),
            scope: hit.entry.scope,
            project: hit.entry.project.clone(),
            kind: hit.entry.kind,
            line: hit.line,
            snippet: hit.snippet.clone(),
            match_count: hit.match_count,
            name_match: hit.name_match,
        }
    }
}

/// One todoz row. `id` is the index into the board's flat task list and is
/// what the frontend passes back to mutating commands.
#[derive(Serialize)]
pub struct TodoTaskDto {
    pub id: usize,
    pub text: String,
    pub state: &'static str,
    pub depth: u8,
    pub flags: u8,
    pub has_subtasks: bool,
    pub collapsed: bool,
    pub is_header: bool,
    pub is_code_todo: bool,
    pub source: String,
    pub section: String,
}

/// A registered project on this machine.
#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    /// Tilde-contracted path for display.
    pub local_path: String,
    /// Whether the path exists on this machine.
    pub reachable: bool,
}

/// The whole board as a flat, depth-encoded list. The frontend rebuilds the
/// tree from `depth` + order.
#[derive(Serialize)]
pub struct TodoBoard {
    pub items: Vec<TodoTaskDto>,
}

impl TodoBoard {
    pub fn from_tasks(tasks: &[Task]) -> Self {
        let items = tasks
            .iter()
            .enumerate()
            .map(|(id, t)| TodoTaskDto {
                id,
                text: t.text.clone(),
                state: match t.state {
                    CheckState::Unchecked => "unchecked",
                    CheckState::Half => "half",
                    CheckState::Checked => "checked",
                },
                depth: t.depth,
                flags: t.flags,
                has_subtasks: t.has_subtasks,
                collapsed: t.collapsed,
                is_header: t.is_header,
                is_code_todo: t.is_code_todo,
                source: t.source.to_string_lossy().into_owned(),
                section: t.section.clone(),
            })
            .collect();
        TodoBoard { items }
    }
}
