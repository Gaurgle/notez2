import { invoke } from "@tauri-apps/api/core";
import type {
  GhCommit,
  GhContributor,
  GhIssue,
  GhRepo,
  GhUser,
  NoteListItem,
  PlanItem,
  ProjectInfo,
  Scope,
  TodoBoard,
} from "./types";

/** Every note across all four scopes (the headline aggregate view). */
export const listNotes = () => invoke<NoteListItem[]>("list_notes");

/** Notes for a single scope, resolved against the current directory. */
export const listNotesInScope = (scope: Scope) =>
  invoke<NoteListItem[]>("list_notes_in_scope", { scope });

/** Read a markdown note's contents. */
export const readNote = (path: string) => invoke<string>("read_note", { path });

/** Set a note's importance flags (zero clears). */
export const setNoteTags = (path: string, flags: number) =>
  invoke<void>("set_note_tags", { path, flags });

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
export const setTaskState = (idx: number, state: "unchecked" | "half" | "checked") =>
  invoke<TodoBoard>("set_task_state", { idx, state });
export const moveTodo = (idx: number, up: boolean) =>
  invoke<TodoBoard>("move_todo", { idx, up });
export const collapseAll = (collapsed: boolean) =>
  invoke<TodoBoard>("collapse_all", { collapsed });
export const createCategory = (name: string) =>
  invoke<TodoBoard>("create_category", { name });

// --- projects & sync ---

export const listProjects = () => invoke<ProjectInfo[]>("list_projects");
export const attachProject = (name: string, path: string) =>
  invoke<ProjectInfo>("attach_project", { name, path });
export const detachProject = (name: string) => invoke<void>("detach_project", { name });
export const syncNotez = () => invoke<string>("sync");

/** This machine's hostname, for the per-machine sidebar avatar. */
export const machineName = () => invoke<string>("machine_name");

export const migratePreview = () => invoke<PlanItem[]>("migrate_preview");
export const migrateApply = () => invoke<string[]>("migrate_apply");

// --- GitHub (real org data via the authed gh CLI) ---

/** The default org backing the desktop views. */
export const GITHUB_ORG = "airwavez";

export const githubUser = () => invoke<GhUser>("github_user");
export const githubRepos = (org: string = GITHUB_ORG) =>
  invoke<GhRepo[]>("github_repos", { org });
export const githubCommits = (repos: string[], limit = 10, org: string = GITHUB_ORG) =>
  invoke<GhCommit[]>("github_commits", { org, repos, limit });
export const githubIssues = (repos: string[], org: string = GITHUB_ORG) =>
  invoke<GhIssue[]>("github_issues", { org, repos });
/** Create a real issue; resolves to the new issue number. */
export const githubCreateIssue = (
  repo: string,
  title: string,
  body: string,
  org: string = GITHUB_ORG
) => invoke<number>("github_create_issue", { org, repo, title, body });
export const githubContributors = (repo: string, org: string = GITHUB_ORG) =>
  invoke<GhContributor[]>("github_contributors", { org, repo });
