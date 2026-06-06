// Cross-view per-project counts so either inspector can show both notes and
// todos for a project, regardless of which view you're hovering in. Both views
// are mounted at once; each fills its own half on load.
export const projectStats = $state<{
  notes: Record<string, number>;
  todos: Record<string, number>;
}>({ notes: {}, todos: {} });

export function countBy<T>(list: T[], key: (t: T) => string | null | undefined): Record<string, number> {
  const out: Record<string, number> = {};
  for (const item of list) {
    const k = key(item);
    if (k) out[k] = (out[k] ?? 0) + 1;
  }
  return out;
}
