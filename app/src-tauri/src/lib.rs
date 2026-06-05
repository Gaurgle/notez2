//! notez2 desktop backend. The brains live in `notez-core`; this crate only
//! exposes them over Tauri's IPC and owns window/app lifecycle.

mod commands;
mod dto;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::BoardState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::list_notes_in_scope,
            commands::read_note,
            commands::create_note,
            commands::save_note,
            commands::append_log,
            commands::load_todo_board,
            commands::load_scope_todo_board,
            commands::toggle_task,
            commands::set_task_flags,
            commands::edit_task,
            commands::add_todo,
            commands::remove_todo,
            commands::reorder_task,
            commands::toggle_collapse,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
