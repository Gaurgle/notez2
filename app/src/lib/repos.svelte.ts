// Shared "active repos" store. Loaded once from GitHub, it holds every repo the
// user can reach and a persisted selection of which ones are *active* (shown
// across the dashboard, ticketz, and spaze). Selection is global so the views
// agree. Repos are keyed by their full `owner/repo` name.
//
// Repos with no push in the last ARCHIVE_DAYS are hidden from the picker by
// default (the user has ~90 repos; most are dormant). A toggle reveals them.

import { SvelteSet } from "svelte/reactivity";
import { githubAllRepos } from "./ipc";
import type { GhRepo } from "./types";

const SEL_KEY = "notez-active-repos-v1";
/** Repos untouched for longer than this are archived out of the picker. */
const ARCHIVE_DAYS = 180;
/** How recent a repo must be to be auto-selected on first run. */
const DEFAULT_ACTIVE_DAYS = 60;
/** Cap the first-run default so the initial data fetch stays snappy. */
const DEFAULT_MAX = 8;

/** One owner's repos, for the grouped picker. */
export interface RepoGroup {
  owner: string;
  type: string; // "User" | "Organization"
  isMe: boolean;
  repos: GhRepo[];
}

const isRecent = (r: GhRepo, days: number) =>
  Date.parse(r.pushed_at) >= Date.now() - days * 86_400_000;

class RepoStore {
  /** Every reachable repo (including archived/dormant). */
  all = $state<GhRepo[]>([]);
  selected = new SvelteSet<string>(); // full_name keys
  me = $state<string>("");
  loading = $state(true);
  error = $state<string | null>(null);
  showArchived = $state(false);
  #loaded = false;

  async ensure(meLogin?: string) {
    if (meLogin) this.me = meLogin;
    if (this.#loaded) return;
    this.#loaded = true;
    this.loading = true;
    this.error = null;
    try {
      this.all = await githubAllRepos();
      this.#initSelection();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  #initSelection() {
    try {
      const stored = localStorage.getItem(SEL_KEY);
      if (stored) {
        for (const full of JSON.parse(stored) as string[]) this.selected.add(full);
        return;
      }
    } catch {
      /* fall through to defaults */
    }
    // First run: the user's own + org repos pushed recently, most-recent first,
    // capped so the initial fetch isn't huge.
    const primary = this.all
      .filter((r) => this.#isPrimaryOwner(r) && isRecent(r, DEFAULT_ACTIVE_DAYS))
      .slice(0, DEFAULT_MAX);
    for (const r of primary) this.selected.add(r.full_name);
    this.#persist();
  }

  /** Personal repos and repos in an org the user belongs to (not one-off
   *  collaborations on other people's repos). */
  #isPrimaryOwner(r: GhRepo): boolean {
    return r.owner === this.me || r.owner_type === "Organization";
  }

  #persist() {
    try {
      localStorage.setItem(SEL_KEY, JSON.stringify([...this.selected]));
    } catch {
      /* storage may be unavailable */
    }
  }

  toggle(full: string) {
    if (this.selected.has(full)) this.selected.delete(full);
    else this.selected.add(full);
    this.#persist();
  }

  setGroup(repos: GhRepo[], on: boolean) {
    for (const r of repos) {
      if (on) this.selected.add(r.full_name);
      else this.selected.delete(r.full_name);
    }
    this.#persist();
  }

  /** Repos shown in the picker: active by default, or everything when the
   *  archived toggle is on. A selected repo always shows even if dormant. */
  repos = $derived.by<GhRepo[]>(() =>
    this.showArchived
      ? this.all
      : this.all.filter((r) => isRecent(r, ARCHIVE_DAYS) || this.selected.has(r.full_name))
  );

  /** How many dormant repos are currently hidden. */
  archivedCount = $derived(
    this.all.filter((r) => !isRecent(r, ARCHIVE_DAYS) && !this.selected.has(r.full_name)).length
  );

  /** The selected repos as full GhRepo objects (regardless of archive state). */
  activeRepos = $derived(this.all.filter((r) => this.selected.has(r.full_name)));

  /** Selected repos as full `owner/repo` names. */
  activeNames = $derived(this.activeRepos.map((r) => r.full_name));

  /** Repos grouped by owner: the signed-in user first, then orgs, then others. */
  groups = $derived.by<RepoGroup[]>(() => {
    const byOwner = new Map<string, GhRepo[]>();
    for (const r of this.repos) {
      const list = byOwner.get(r.owner) ?? [];
      list.push(r);
      byOwner.set(r.owner, list);
    }
    const groups: RepoGroup[] = [...byOwner.entries()].map(([owner, repos]) => ({
      owner,
      type: repos[0]?.owner_type ?? "User",
      isMe: owner === this.me,
      repos,
    }));
    const rank = (g: RepoGroup) => (g.isMe ? 0 : g.type === "Organization" ? 1 : 2);
    groups.sort((a, b) => rank(a) - rank(b) || a.owner.localeCompare(b.owner));
    return groups;
  });
}

export const repoStore = new RepoStore();
