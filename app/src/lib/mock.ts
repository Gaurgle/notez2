// Placeholder ("mock") metadata for previewing the authorship / time UI before
// GitHub auth + a real date model land. Deterministic from a seed so it stays
// stable across renders. Swap these for real data once the model exists.

const AUTHORS = ["you", "alex", "mira", "sam", "nora", "kai"];
const AGO = ["now", "1h ago", "3h ago", "9h ago", "1d ago", "2d ago", "5d ago", "1w ago", "2w ago"];

export function hashStr(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (Math.imul(h, 31) + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

export function mockAuthor(seed: number): string {
  return AUTHORS[Math.abs(seed) % AUTHORS.length];
}

export function mockAgo(seed: number): string {
  return AGO[(Math.abs(seed) * 5) % AGO.length];
}

/** A small, stable set of "involved" authors for a project. */
export function mockProjectAuthors(project: string): string[] {
  const h = hashStr(project);
  const n = 1 + (h % 4); // 1..4 people
  const out: string[] = [];
  for (let i = 0; i < n; i++) out.push(AUTHORS[(h + i * 7) % AUTHORS.length]);
  return [...new Set(out)];
}

/** Real relative-time formatting (used for notes, which carry a real mtime). */
export function relativeTime(unixSeconds: number): string {
  if (!unixSeconds) return "";
  const diff = Math.max(0, Date.now() / 1000 - unixSeconds);
  const mins = Math.floor(diff / 60);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d ago`;
  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks}w ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  return `${Math.floor(days / 365)}y ago`;
}
