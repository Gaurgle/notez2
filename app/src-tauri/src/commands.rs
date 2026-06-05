//! Tauri command surface. Each command wraps a notez-core function and
//! returns either serializable data or a rendered error string.

use std::sync::Mutex;

use notez_core::config::{paths, Config, NotezMetadata, ProjectRegistry};
use notez_core::core::{aggregate, note, resolve, Note, Project, Scope};
use notez_core::todo::{self, Task};

use crate::dto::{NoteListItem, TodoBoard};

/// The live todoz board, held in Tauri-managed state so the indices the
/// frontend sends stay valid between calls.
pub type BoardState = Mutex<Vec<Task>>;

/// Render any error as a string for the IPC boundary.
fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e:#}")
}

/// Every note across all four scopes (global + each registered project's
/// local/public/personal trees). This is the headline aggregate view and
/// needs no current-directory context.
#[tauri::command]
pub fn list_notes() -> Result<Vec<NoteListItem>, String> {
    let config = Config::load().map_err(err)?;
    let registry = ProjectRegistry::load().map_err(err)?;
    let metadata_path = paths::metadata_file(&config.notez_root_path());
    let metadata = NotezMetadata::load_from(&metadata_path).map_err(err)?;

    let entries = aggregate::collect_all(&config, &registry, &metadata).map_err(err)?;
    Ok(entries.iter().map(NoteListItem::from).collect())
}

/// Notes for a single scope, resolved against the current directory's
/// project (if any). Global resolves without a project; the project-relative
/// scopes fall back to empty when launched outside a project.
#[tauri::command]
pub fn list_notes_in_scope(scope: Scope) -> Result<Vec<NoteListItem>, String> {
    let config = Config::load().map_err(err)?;
    let cwd_project = Project::try_detect();
    let entries = aggregate::collect_in_scope(scope, &config, cwd_project.as_ref());
    Ok(entries.iter().map(NoteListItem::from).collect())
}

/// Read a markdown note's contents. Refuses anything that isn't a `.md` file;
/// deeper path-jailing to known notez roots lands in M6.
#[tauri::command]
pub fn read_note(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if p.extension().and_then(|s| s.to_str()) != Some("md") {
        return Err("refusing to read a non-markdown file".into());
    }
    std::fs::read_to_string(p).map_err(err)
}

/// Create a new note in the given scope. Mirrors `notez add`: empty titles
/// become "untitled", the body is optional. Returns the new file's path.
#[tauri::command]
pub fn create_note(scope: Scope, title: String, body: Option<String>) -> Result<String, String> {
    let config = Config::load().map_err(err)?;
    let body = body.filter(|b| !b.trim().is_empty());
    let note = Note::new(title, body);

    let dir = resolve::quick_notes(scope, &config).map_err(err)?;
    std::fs::create_dir_all(&dir).map_err(err)?;

    let path = dir.join(note.filename());
    std::fs::write(&path, note.rendered()).map_err(err)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Overwrite a note's contents. Refuses non-`.md` paths; deeper path-jailing
/// lands in M6.
#[tauri::command]
pub fn save_note(path: String, content: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.extension().and_then(|s| s.to_str()) != Some("md") {
        return Err("refusing to write a non-markdown file".into());
    }
    std::fs::write(p, content).map_err(err)
}

/// Append a timestamped entry to today's daily log. Mirrors `notez log`.
/// Returns the log file's path.
#[tauri::command]
pub fn append_log(scope: Scope, message: String) -> Result<String, String> {
    let message = message.trim().to_string();
    if message.is_empty() {
        return Err("log message cannot be empty".into());
    }
    let config = Config::load().map_err(err)?;

    let dir = resolve::daily_logs(scope, &config).map_err(err)?;
    std::fs::create_dir_all(&dir).map_err(err)?;

    let path = dir.join(note::todays_log_filename());
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let updated = note::append_log_entry(&existing, &message);
    std::fs::write(&path, updated).map_err(err)?;
    Ok(path.to_string_lossy().into_owned())
}

// --- todoz ---

fn lock(state: &BoardState) -> Result<std::sync::MutexGuard<'_, Vec<Task>>, String> {
    state.lock().map_err(|_| "todoz board lock poisoned".to_string())
}

/// Persist every touched TODO.md and return the refreshed board.
fn persisted(guard: &[Task]) -> Result<TodoBoard, String> {
    todo::save_all_todos(guard).map_err(err)?;
    Ok(TodoBoard::from_tasks(guard))
}

/// Load the global todoz board (all scopes) into managed state.
#[tauri::command]
pub fn load_todo_board(state: tauri::State<BoardState>) -> Result<TodoBoard, String> {
    let config = Config::load().map_err(err)?;
    let registry = ProjectRegistry::load().map_err(err)?;
    let board = todo::load_board(&config, &registry);
    let mut guard = lock(&state)?;
    *guard = board;
    Ok(TodoBoard::from_tasks(&guard))
}

/// Load the single-scope todoz board into managed state.
#[tauri::command]
pub fn load_scope_todo_board(
    scope: Scope,
    state: tauri::State<BoardState>,
) -> Result<TodoBoard, String> {
    let config = Config::load().map_err(err)?;
    let board = todo::load_scope_board(scope, &config).map_err(err)?;
    let mut guard = lock(&state)?;
    *guard = board;
    Ok(TodoBoard::from_tasks(&guard))
}

#[tauri::command]
pub fn toggle_task(idx: usize, state: tauri::State<BoardState>) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    todo::toggle_done(&mut guard, idx);
    persisted(&guard)
}

#[tauri::command]
pub fn set_task_flags(
    idx: usize,
    flags: u8,
    state: tauri::State<BoardState>,
) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    todo::set_flags(&mut guard, idx, flags);
    persisted(&guard)
}

#[tauri::command]
pub fn edit_task(
    idx: usize,
    text: String,
    state: tauri::State<BoardState>,
) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    todo::edit_text(&mut guard, idx, text);
    persisted(&guard)
}

#[tauri::command]
pub fn add_todo(
    after: usize,
    depth: u8,
    text: String,
    state: tauri::State<BoardState>,
) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    todo::add_task(&mut guard, after, depth, text);
    persisted(&guard)
}

#[tauri::command]
pub fn remove_todo(idx: usize, state: tauri::State<BoardState>) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    todo::remove_task(&mut guard, idx);
    persisted(&guard)
}

#[tauri::command]
pub fn reorder_task(
    start: usize,
    target: usize,
    state: tauri::State<BoardState>,
) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    if todo::can_drag(&guard, start, target) {
        todo::perform_drag_move(&mut guard, start, target);
        todo::derive_parent_states(&mut guard);
        persisted(&guard)
    } else {
        // Invalid drag: return the board unchanged.
        Ok(TodoBoard::from_tasks(&guard))
    }
}

/// Toggle a header/parent's collapsed state. View-only, never persisted.
#[tauri::command]
pub fn toggle_collapse(idx: usize, state: tauri::State<BoardState>) -> Result<TodoBoard, String> {
    let mut guard = lock(&state)?;
    if let Some(t) = guard.get_mut(idx) {
        if t.is_header || t.has_subtasks {
            t.collapsed = !t.collapsed;
        }
    }
    Ok(TodoBoard::from_tasks(&guard))
}
