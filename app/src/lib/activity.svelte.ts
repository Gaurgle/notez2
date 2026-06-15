// Shared, cached, concurrency-bounded repo activity (commits + issues).
//
// All views stay mounted, so without this each one would independently spawn a
// `gh` storm on every repo toggle and peg the CPU. Instead every view reads the
// same reactive cache and calls ensureActivity(); fetches run through a small
// pool (POOL at a time) on Tauri background threads and stream into the cache,
// so the UI stays responsive and shows per-repo progress.

import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { githubRepoActivity } from "./ipc";
import type { GhCommit, GhIssue } from "./types";

interface Activity {
  commits: GhCommit[];
  issues: GhIssue[];
  ts: number;
}

const TTL = 5 * 60 * 1000; // re-fetch a repo at most every 5 minutes
const COMMIT_LIMIT = 15;
const POOL = 3; // max repos fetched at once — the cap that keeps the CPU calm

/** repo full_name -> activity. Reactive so views re-derive as data streams in. */
export const activityCache = new SvelteMap<string, Activity>();
/** repos currently being fetched, for per-repo loading indicators. */
export const loadingRepos = new SvelteSet<string>();

function fresh(a: Activity | undefined): a is Activity {
  return !!a && Date.now() - a.ts < TTL;
}

async function pool<T>(items: T[], size: number, fn: (it: T) => Promise<void>) {
  const queue = [...items];
  const worker = async () => {
    for (let it = queue.shift(); it !== undefined; it = queue.shift()) await fn(it);
  };
  await Promise.all(Array.from({ length: Math.min(size, queue.length) }, worker));
}

/** Ensure each repo has fresh activity cached. Cheap (no-op) when everything is
 *  already fresh; otherwise fetches the missing ones POOL at a time. */
export async function ensureActivity(names: string[]) {
  const missing = names.filter((n) => !fresh(activityCache.get(n)) && !loadingRepos.has(n));
  if (missing.length === 0) return;
  await pool(missing, POOL, async (n) => {
    loadingRepos.add(n);
    try {
      const act = await githubRepoActivity(n, COMMIT_LIMIT);
      activityCache.set(n, { commits: act.commits, issues: act.issues, ts: Date.now() });
    } catch {
      /* leave uncached; retried on the next pass */
    } finally {
      loadingRepos.delete(n);
    }
  });
}

/** Drop a repo's cached activity so the next ensureActivity refetches it. */
export function invalidate(full: string) {
  activityCache.delete(full);
}

export const commitsFor = (full: string): GhCommit[] => activityCache.get(full)?.commits ?? [];
export const issuesFor = (full: string): GhIssue[] => activityCache.get(full)?.issues ?? [];
