export type Scope = "local" | "personal" | "public" | "global";

export interface NoteListItem {
  path: string;
  name: string;
  scope: Scope;
  project: string | null;
  flags: number;
  /** Last-modified time, seconds since the Unix epoch. */
  modified: number;
}

export const SCOPE_META: Record<Scope, { label: string; icon: string }> = {
  personal: { label: "Personal", icon: "" },
  public: { label: "Public", icon: "" },
  local: { label: "Local", icon: "" },
  global: { label: "Global", icon: "" },
};

export interface ProjectInfo {
  name: string;
  local_path: string;
  reachable: boolean;
}

export interface PlanItem {
  name: string;
  repo_path: string;
  from: string;
  to: string;
  note: string;
}

export type CheckState = "unchecked" | "half" | "checked";

export interface TodoTask {
  id: number;
  text: string;
  state: CheckState;
  depth: number;
  flags: number;
  has_subtasks: boolean;
  collapsed: boolean;
  is_header: boolean;
  is_code_todo: boolean;
  source: string;
  section: string;
}

export interface TodoBoard {
  items: TodoTask[];
}

// --- GitHub (real org data via the authed gh CLI) ---

export interface GhRepo {
  name: string;
  description: string;
  language: string | null;
  pushed_at: string;
  open_issues: number;
  url: string;
  is_private: boolean;
}

export interface GhCommit {
  sha: string;
  repo: string;
  message: string;
  author: string;
  author_login: string | null;
  avatar_url: string | null;
  date: string;
}

export interface GhIssue {
  number: number;
  repo: string;
  title: string;
  body: string;
  state: string;
  labels: string[];
  assignees: string[];
  author: string;
  avatar_url: string | null;
  url: string;
  created_at: string;
  updated_at: string;
  points: number | null;
}

export interface GhUser {
  login: string;
  name: string;
  avatar_url: string;
}

export interface GhContributor {
  login: string;
  avatar_url: string;
  contributions: number;
}

/** The 5 tag flags, aligned 1:1 with notez-core::tags::FLAG_DEFS. */
export const TAG_DEFS = [
  { bit: 1 << 0, key: "important", label: "important", color: "#f38ba8" },
  { bit: 1 << 1, key: "prio", label: "priority", color: "#fab387" },
  { bit: 1 << 2, key: "longterm", label: "long-term", color: "#f9e2af" },
  { bit: 1 << 3, key: "idea", label: "idea", color: "#74c7ec" },
  { bit: 1 << 4, key: "blocked", label: "blocked", color: "#cba6f7" },
] as const;
