export type Scope = "local" | "personal" | "public" | "global";

/** Note vs project doc (markdown from a repo's docs/ directory). */
export type SourceKind = "note" | "doc";

export interface NoteListItem {
  path: string;
  name: string;
  scope: Scope;
  project: string | null;
  kind: SourceKind;
  flags: number;
  /** Last-modified time, seconds since the Unix epoch. */
  modified: number;
}

/** One folder in the global notez root, for the sidebar tree. */
export interface FolderNode {
  /** Last path segment, e.g. "Kotlin". */
  name: string;
  /** Path relative to the notez root, e.g. "reference/Kotlin". */
  rel: string;
  /** Notes in this subtree. */
  count: number;
  /** Newest modified epoch in this subtree. */
  latest: number;
  children: FolderNode[];
}

/** One full-text search hit, anchored at its first matching line. */
export interface SearchHit {
  path: string;
  name: string;
  scope: Scope;
  project: string | null;
  kind: SourceKind;
  /** 1-based line number of the first match. */
  line: number;
  /** The first matching line, trimmed. */
  snippet: string;
  /** Total matching lines in the file. */
  match_count: number;
  /** True when the filename itself also matches the query. */
  name_match: boolean;
}

// Two-axis scope language (decided 2026-07-06): accessibility (personal vs
// public) x binding (project vs global). "local" is kept on the wire but
// presented as scratch: this machine only, never syncs.
export const SCOPE_META: Record<
  Scope,
  { label: string; pill: string; hint: string; icon: string }
> = {
  personal: {
    label: "Personal",
    pill: "personal",
    hint: "Your private notez repo, synced across your machines",
    icon: "",
  },
  public: {
    label: "Public",
    pill: "public",
    hint: "Committed in the project repo, visible to collaborators",
    icon: "",
  },
  local: {
    label: "Scratch",
    pill: "scratch",
    hint: "This machine only, gitignored, never syncs",
    icon: "",
  },
  global: {
    label: "notez (global)",
    pill: "notez",
    hint: "Your private notez repo, notes bound to no project",
    icon: "",
  },
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
  /** owner/repo — the stable key used across the app. */
  full_name: string;
  owner: string;
  /** "User" | "Organization". */
  owner_type: string;
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

export interface GhDay {
  date: string;
  count: number;
}

/** The 5 tag flags, aligned 1:1 with notez-core::tags::FLAG_DEFS. */
export const TAG_DEFS = [
  { bit: 1 << 0, key: "important", label: "important", color: "#f38ba8" },
  { bit: 1 << 1, key: "prio", label: "priority", color: "#fab387" },
  { bit: 1 << 2, key: "longterm", label: "long-term", color: "#f9e2af" },
  { bit: 1 << 3, key: "idea", label: "idea", color: "#74c7ec" },
  { bit: 1 << 4, key: "blocked", label: "blocked", color: "#cba6f7" },
] as const;
