import { invoke } from "@tauri-apps/api/core";
import type { NoteListItem, Scope, TodoBoard } from "./types";

/** Every note across all four scopes (the headline aggregate view). */
export const listNotes = () => invoke<NoteListItem[]>("list_notes");

/** Notes for a single scope, resolved against the current directory. */
export const listNotesInScope = (scope: Scope) =>
  invoke<NoteListItem[]>("list_notes_in_scope", { scope });

/** Read a markdown note's contents. */
export const readNote = (path: string) => invoke<string>("read_note", { path });

/** Create a new note; resolves to the new file's path. */
export const createNote = (scope: Scope, title: string, body: string | null) =>
  invoke<string>("create_note", { scope, title, body });

/** Overwrite a note's contents. */
export const saveNote = (path: string, content: string) =>
  invoke<void>("save_note", { path, content });

/** Append a timestamped entry to today's daily log; resolves to the log path. */
export const appendLog = (scope: Scope, message: string) =>
  invoke<string>("append_log", { scope, message });

// --- todoz ---

export const loadTodoBoard = () => invoke<TodoBoard>("load_todo_board");
export const toggleTask = (idx: number) => invoke<TodoBoard>("toggle_task", { idx });
export const setTaskFlags = (idx: number, flags: number) =>
  invoke<TodoBoard>("set_task_flags", { idx, flags });
export const editTask = (idx: number, text: string) =>
  invoke<TodoBoard>("edit_task", { idx, text });
export const addTodo = (after: number, depth: number, text: string) =>
  invoke<TodoBoard>("add_todo", { after, depth, text });
export const removeTodo = (idx: number) => invoke<TodoBoard>("remove_todo", { idx });
export const reorderTask = (start: number, target: number) =>
  invoke<TodoBoard>("reorder_task", { start, target });
export const toggleCollapse = (idx: number) =>
  invoke<TodoBoard>("toggle_collapse", { idx });
