//! Real GitHub data for the desktop views, sourced by shelling out to the
//! already-authenticated `gh` CLI. No HTTP client, no token handling: `gh`
//! owns auth, and this module just parses its JSON.
//!
//! Repos are identified everywhere by their full `owner/repo` name, so personal
//! repos and several orgs coexist without collisions. Everything here is
//! best-effort: a missing `gh`, an unreachable repo, or an empty repo all
//! degrade to an empty list rather than an error the UI has to special-case.
//!
//! Responsiveness rules (the UI froze without them):
//! - Every command is `async`. In Tauri v2 a synchronous command runs on the
//!   MAIN thread, so a sync `gh` call blocks the whole window for its full
//!   network round-trip.
//! - At most [`MAX_CONCURRENT_GH`] `gh` processes run at once, app-wide.
//!   Each `gh` is a full Go binary doing network I/O and JSON; an unbounded
//!   fan-out lags the entire machine, not just the app.
//! - Read endpoints go through a stale-while-revalidate disk cache: a fresh
//!   entry answers instantly with no process spawn, a stale entry answers
//!   instantly AND refreshes in the background (a `github:refreshed` event
//!   tells the frontend when the new data landed), and only a cold miss
//!   actually waits on the network.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

/// One repository the signed-in user can reach.
#[derive(Serialize)]
pub struct GhRepo {
    pub name: String,
    pub full_name: String, // owner/repo: the stable key used everywhere
    pub owner: String,
    pub owner_type: String, // "User" | "Organization"
    pub description: String,
    pub language: Option<String>,
    pub pushed_at: String,
    pub open_issues: u32,
    pub url: String,
    pub is_private: bool,
}

/// One commit, flattened for the recent-activity feed.
#[derive(Serialize)]
pub struct GhCommit {
    pub sha: String,
    pub repo: String, // full owner/repo name
    pub message: String,
    pub author: String,
    pub author_login: Option<String>,
    pub avatar_url: Option<String>,
    pub date: String,
}

/// One issue, shaped for the ticketz board.
#[derive(Serialize)]
pub struct GhIssue {
    pub number: u64,
    pub repo: String, // full owner/repo name
    pub title: String,
    pub body: String,
    pub state: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub author: String,
    pub avatar_url: Option<String>,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
    /// Story points parsed from a `sp:N` / `points:N` / `N-points` label.
    pub points: Option<u32>,
}

/// The signed-in GitHub identity.
#[derive(Serialize)]
pub struct GhUser {
    pub login: String,
    pub name: String,
    pub avatar_url: String,
}

/// A repo contributor, for team/member widgets.
#[derive(Serialize)]
pub struct GhContributor {
    pub login: String,
    pub avatar_url: String,
    pub contributions: u32,
}

/// One day in the contribution calendar (the GitHub "green squares").
#[derive(Serialize)]
pub struct GhDay {
    pub date: String,
    pub count: u32,
}

// --- gh process gate -------------------------------------------------------

/// App-wide cap on concurrent `gh` processes.
const MAX_CONCURRENT_GH: usize = 3;
static GH_GATE: Semaphore = Semaphore::const_new(MAX_CONCURRENT_GH);

/// Cache keys with a background refresh already in flight, so several views
/// hitting the same stale entry spawn one refresh, not one each.
static REFRESHING: Mutex<Option<HashSet<String>>> = Mutex::new(None);

fn refresh_begin(key: &str) -> bool {
    let mut guard = REFRESHING.lock().unwrap();
    guard.get_or_insert_with(HashSet::new).insert(key.to_string())
}

fn refresh_end(key: &str) {
    if let Some(set) = REFRESHING.lock().unwrap().as_mut() {
        set.remove(key);
    }
}

/// Run `gh` with the given args, returning stdout on success. Falls back to the
/// Homebrew path when `gh` is not on the app's `PATH` (the case when the app is
/// launched from Finder rather than a terminal). Waits for a gate permit first.
async fn run_gh(args: &[String]) -> Result<String, String> {
    let _permit = GH_GATE.acquire().await.map_err(|e| e.to_string())?;

    let attempt = |bin: &str| tokio::process::Command::new(bin).args(args).output();
    let output = match attempt("gh").await {
        Ok(o) => o,
        Err(_) => attempt("/opt/homebrew/bin/gh")
            .await
            .map_err(|e| format!("could not run gh: {e}"))?,
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// --- stale-while-revalidate disk cache -------------------------------------

/// One cached gh response. `args` is stored so mutations can invalidate every
/// entry mentioning a repo without knowing the exact keys.
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    ts: u64,
    args: Vec<String>,
    body: String,
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("epoz")
        .join("gh")
}

/// Deterministic key for an args vector. `DefaultHasher::new()` uses fixed
/// SipHash keys, so keys are stable across app restarts of the same build;
/// a toolchain upgrade at worst costs one cold cache.
fn cache_key(args: &[String]) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut h = DefaultHasher::new();
    args.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache_from(dir: &Path, key: &str) -> Option<CacheEntry> {
    let raw = std::fs::read_to_string(dir.join(key).with_extension("json")).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_cache_to(dir: &Path, key: &str, args: &[String], body: &str) {
    let entry = CacheEntry {
        ts: now_secs(),
        args: args.to_vec(),
        body: body.to_string(),
    };
    if std::fs::create_dir_all(dir).is_ok() {
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = std::fs::write(dir.join(key).with_extension("json"), json);
        }
    }
}

/// Delete every cache entry whose args mention `needle` (an `owner/repo`).
/// Called after mutations so the next read refetches instead of serving the
/// pre-mutation snapshot.
fn bust_cache_containing(needle: &str) {
    let dir = cache_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(entry) = serde_json::from_str::<CacheEntry>(&raw) {
            if entry.args.iter().any(|a| a.contains(needle)) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

fn is_fresh(entry_ts: u64, now: u64, ttl: Duration) -> bool {
    now.saturating_sub(entry_ts) <= ttl.as_secs()
}

/// Cached `gh` call. Fresh entry: instant, no spawn. Stale entry: instant
/// stale answer + background refresh that emits `github:refreshed` with `tag`
/// when new data is on disk. Cold: gated fetch. `force` skips the cache read
/// entirely (still gated, still written back) for user-initiated refreshes.
async fn gh_cached(
    app: &AppHandle,
    args: Vec<String>,
    ttl: Duration,
    tag: String,
    force: bool,
) -> Result<String, String> {
    let dir = cache_dir();
    let key = cache_key(&args);

    if let Some(entry) = read_cache_from(&dir, &key).filter(|_| !force) {
        if is_fresh(entry.ts, now_secs(), ttl) {
            return Ok(entry.body);
        }
        // Serve stale immediately; refresh behind the gate unless a refresh
        // for this key is already running.
        if refresh_begin(&key) {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(body) = run_gh(&args).await {
                    write_cache_to(&cache_dir(), &cache_key(&args), &args, &body);
                    let _ = app.emit("github:refreshed", tag);
                }
                refresh_end(&cache_key(&args));
            });
        }
        return Ok(entry.body);
    }

    let body = run_gh(&args).await?;
    write_cache_to(&dir, &key, &args, &body);
    Ok(body)
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// Per-endpoint TTLs. Short enough to feel live, long enough that tab-hopping
// and repo-toggling never re-spawn gh for data just fetched.
const TTL_USER: Duration = Duration::from_secs(3600);
const TTL_REPOS: Duration = Duration::from_secs(600);
const TTL_ACTIVITY: Duration = Duration::from_secs(180);
const TTL_CONTRIBUTORS: Duration = Duration::from_secs(600);
const TTL_CALENDAR: Duration = Duration::from_secs(600);

// --- parsing helpers --------------------------------------------------------

fn parse(json: &str) -> Result<Value, String> {
    serde_json::from_str(json).map_err(|e| format!("bad gh json: {e}"))
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

/// Pull a story-point count out of a label like `sp:3`, `points: 5`,
/// `3-points`, or a bare `8`.
fn points_from_label(label: &str) -> Option<u32> {
    let lower = label.to_ascii_lowercase();
    let digits: String = lower.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let is_point_label = lower.contains("sp")
        || lower.contains("point")
        || lower.contains("story")
        || lower.chars().all(|c| c.is_ascii_digit());
    if is_point_label {
        digits.parse().ok()
    } else {
        None
    }
}

// --- commands ---------------------------------------------------------------

/// The signed-in GitHub user.
#[tauri::command]
pub async fn github_user(app: AppHandle) -> Result<GhUser, String> {
    let out = gh_cached(&app, args(&["api", "user"]), TTL_USER, "user".into(), false).await?;
    let v = parse(&out)?;
    let login = str_field(&v, "login");
    Ok(GhUser {
        name: v
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(&login)
            .to_string(),
        avatar_url: str_field(&v, "avatar_url"),
        login,
    })
}

/// Every repo the user can reach (owned, collaborator, or org member), newest
/// push first. One paginated call; the list payload already carries the open
/// issue count and owner, so no per-repo follow-ups are needed here.
#[tauri::command]
pub async fn github_all_repos(app: AppHandle, force: Option<bool>) -> Result<Vec<GhRepo>, String> {
    let out = gh_cached(
        &app,
        args(&[
            "api",
            "--paginate",
            "--slurp",
            "user/repos?affiliation=owner,collaborator,organization_member&sort=pushed&per_page=100",
        ]),
        TTL_REPOS,
        "repos".into(),
        force.unwrap_or(false),
    )
    .await?;
    // `--slurp` returns an array of *pages* (each page is itself an array of
    // repos), so flatten one level. Tolerate a flat array too, just in case.
    let v = parse(&out)?;
    let mut repos: Vec<GhRepo> = Vec::new();
    if let Some(pages) = v.as_array() {
        for page in pages {
            match page.as_array() {
                Some(items) => repos.extend(items.iter().map(map_user_repo)),
                None => repos.push(map_user_repo(page)),
            }
        }
    }
    repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));
    Ok(repos)
}

/// Map a REST `/user/repos` object to our DTO.
fn map_user_repo(v: &Value) -> GhRepo {
    let owner = v.get("owner");
    GhRepo {
        name: str_field(v, "name"),
        full_name: str_field(v, "full_name"),
        owner: owner.map(|o| str_field(o, "login")).unwrap_or_default(),
        owner_type: owner.map(|o| str_field(o, "type")).unwrap_or_default(),
        description: str_field(v, "description"),
        language: v.get("language").and_then(Value::as_str).map(String::from),
        pushed_at: str_field(v, "pushed_at"),
        open_issues: v
            .get("open_issues_count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        url: str_field(v, "html_url"),
        is_private: v.get("private").and_then(Value::as_bool).unwrap_or(false),
    }
}

/// Commits for one repo, cached per repo so each entry refreshes on its own
/// clock and mutations can bust a single repo.
async fn commits_for(app: &AppHandle, repo: &str, per_page: &str, force: bool) -> Vec<GhCommit> {
    let path = format!("repos/{repo}/commits?per_page={per_page}");
    // A brand-new/empty repo 409s here; treat any failure as "no commits".
    let Ok(out) = gh_cached(
        app,
        args(&["api", &path]),
        TTL_ACTIVITY,
        format!("repo_activity:{repo}"),
        force,
    )
    .await
    else {
        return Vec::new();
    };
    let Ok(v) = parse(&out) else {
        return Vec::new();
    };
    let Some(arr) = v.as_array() else {
        return Vec::new();
    };

    arr.iter()
        .map(|c| {
            let commit = c.get("commit");
            let author_obj = c.get("author");
            GhCommit {
                sha: str_field(c, "sha"),
                repo: repo.to_string(),
                message: first_line(
                    commit
                        .and_then(|x| x.get("message"))
                        .and_then(Value::as_str)
                        .unwrap_or(""),
                ),
                author: commit
                    .and_then(|x| x.get("author"))
                    .map(|a| str_field(a, "name"))
                    .unwrap_or_default(),
                author_login: author_obj
                    .and_then(|a| a.get("login"))
                    .and_then(Value::as_str)
                    .map(String::from),
                avatar_url: author_obj
                    .and_then(|a| a.get("avatar_url"))
                    .and_then(Value::as_str)
                    .map(String::from),
                date: commit
                    .and_then(|x| x.get("author"))
                    .map(|a| str_field(a, "date"))
                    .unwrap_or_default(),
            }
        })
        .collect()
}

/// Recent commits across the given `owner/repo` repos, merged newest first.
/// `limit` caps commits fetched per repo before merging.
#[tauri::command]
pub async fn github_commits(
    app: AppHandle,
    repos: Vec<String>,
    limit: u32,
) -> Result<Vec<GhCommit>, String> {
    let per_page = limit.clamp(1, 100).to_string();
    let mut commits: Vec<GhCommit> = Vec::new();
    for repo in &repos {
        commits.extend(commits_for(&app, repo, &per_page, false).await);
    }
    commits.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(commits)
}

/// Issues for one repo (open + closed), cached per repo. PRs are excluded;
/// `gh issue list` already does this.
async fn issues_for(app: &AppHandle, repo: &str, force: bool) -> Vec<GhIssue> {
    let Ok(out) = gh_cached(
        app,
        args(&[
            "issue",
            "list",
            "-R",
            repo,
            "--state",
            "all",
            "--limit",
            "100",
            "--json",
            "number,title,body,state,labels,assignees,author,url,createdAt,updatedAt",
        ]),
        TTL_ACTIVITY,
        format!("repo_activity:{repo}"),
        force,
    )
    .await
    else {
        return Vec::new();
    };
    let Ok(v) = parse(&out) else {
        return Vec::new();
    };
    let Some(arr) = v.as_array() else {
        return Vec::new();
    };

    arr.iter()
        .map(|it| {
            let labels: Vec<String> = it
                .get("labels")
                .and_then(Value::as_array)
                .map(|a| a.iter().map(|l| str_field(l, "name")).collect())
                .unwrap_or_default();
            let points = labels.iter().find_map(|l| points_from_label(l));
            let assignees: Vec<String> = it
                .get("assignees")
                .and_then(Value::as_array)
                .map(|a| a.iter().map(|u| str_field(u, "login")).collect())
                .unwrap_or_default();

            GhIssue {
                number: it.get("number").and_then(Value::as_u64).unwrap_or(0),
                repo: repo.to_string(),
                title: str_field(it, "title"),
                body: str_field(it, "body"),
                state: str_field(it, "state").to_ascii_lowercase(),
                labels,
                assignees,
                author: it
                    .get("author")
                    .map(|a| str_field(a, "login"))
                    .unwrap_or_default(),
                avatar_url: None,
                url: str_field(it, "url"),
                created_at: str_field(it, "createdAt"),
                updated_at: str_field(it, "updatedAt"),
                points,
            }
        })
        .collect()
}

/// All issues (open + closed) across the given `owner/repo` repos.
#[tauri::command]
pub async fn github_issues(app: AppHandle, repos: Vec<String>) -> Result<Vec<GhIssue>, String> {
    let mut issues: Vec<GhIssue> = Vec::new();
    for repo in &repos {
        issues.extend(issues_for(&app, repo, false).await);
    }
    issues.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(issues)
}

/// Create a real issue in the `owner/repo`. Returns the new issue number so the
/// caller can refetch the board. User-initiated only (the New button). Never
/// cached, and busts every cache entry for the repo so the refetch sees it.
#[tauri::command]
pub async fn github_create_issue(
    repo: String,
    title: String,
    body: String,
) -> Result<u64, String> {
    if title.trim().is_empty() {
        return Err("issue title is empty".into());
    }
    // gh prints the new issue's URL on success; the number is its last segment.
    let out = run_gh(&args(&[
        "issue", "create", "-R", &repo, "--title", &title, "--body", &body,
    ]))
    .await?;
    bust_cache_containing(&repo);
    out.trim()
        .rsplit('/')
        .next()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .ok_or_else(|| format!("could not parse new issue number from: {}", out.trim()))
}

/// Commits + issues for a single repo, in one IPC round-trip. The frontend
/// calls this per repo through a small concurrency pool; each call is async
/// (never on the main thread) and both halves are individually cached.
#[derive(Serialize)]
pub struct GhRepoActivity {
    pub repo: String,
    pub commits: Vec<GhCommit>,
    pub issues: Vec<GhIssue>,
}

#[tauri::command]
pub async fn github_repo_activity(
    app: AppHandle,
    repo: String,
    commit_limit: u32,
    force: Option<bool>,
) -> Result<GhRepoActivity, String> {
    let per_page = commit_limit.clamp(1, 100).to_string();
    let f = force.unwrap_or(false);
    let commits = commits_for(&app, &repo, &per_page, f).await;
    let issues = issues_for(&app, &repo, f).await;
    Ok(GhRepoActivity {
        repo,
        commits,
        issues,
    })
}

/// The signed-in user's contribution calendar (last ~year), independent of any
/// repo selection. This is the GitHub "green squares" activity, sourced in one
/// GraphQL call so the dashboard heatmap stays stable as repos are toggled.
#[tauri::command]
pub async fn github_contribution_calendar(
    app: AppHandle,
    force: Option<bool>,
) -> Result<Vec<GhDay>, String> {
    let query = "query { viewer { contributionsCollection { contributionCalendar { \
                 weeks { contributionDays { date contributionCount } } } } } }";
    let out = gh_cached(
        &app,
        args(&["api", "graphql", "-f", &format!("query={query}")]),
        TTL_CALENDAR,
        "calendar".into(),
        force.unwrap_or(false),
    )
    .await?;
    let v = parse(&out)?;
    let weeks = v
        .pointer("/data/viewer/contributionsCollection/contributionCalendar/weeks")
        .and_then(Value::as_array)
        .ok_or("unexpected contribution-calendar shape")?;
    let mut days = Vec::new();
    for week in weeks {
        if let Some(list) = week.get("contributionDays").and_then(Value::as_array) {
            for d in list {
                days.push(GhDay {
                    date: str_field(d, "date"),
                    count: d
                        .get("contributionCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32,
                });
            }
        }
    }
    Ok(days)
}

/// Contributors to a single `owner/repo`, most commits first.
#[tauri::command]
pub async fn github_contributors(
    app: AppHandle,
    repo: String,
) -> Result<Vec<GhContributor>, String> {
    let path = format!("repos/{repo}/contributors?per_page=20");
    let out = match gh_cached(
        &app,
        args(&["api", &path]),
        TTL_CONTRIBUTORS,
        format!("contributors:{repo}"),
        false,
    )
    .await
    {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };
    let v = parse(&out)?;
    Ok(v.as_array()
        .map(|arr| {
            arr.iter()
                .map(|c| GhContributor {
                    login: str_field(c, "login"),
                    avatar_url: str_field(c, "avatar_url"),
                    contributions: c.get("contributions").and_then(Value::as_u64).unwrap_or(0)
                        as u32,
                })
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic_and_distinct() {
        let a = args(&["api", "user"]);
        let b = args(&["api", "user"]);
        let c = args(&["api", "repos/x/y/commits?per_page=15"]);
        assert_eq!(cache_key(&a), cache_key(&b));
        assert_ne!(cache_key(&a), cache_key(&c));
    }

    #[test]
    fn freshness_respects_ttl() {
        let now = 10_000;
        assert!(is_fresh(now - 100, now, Duration::from_secs(180)));
        assert!(!is_fresh(now - 200, now, Duration::from_secs(180)));
        // Clock skew (entry from the "future") must not underflow.
        assert!(is_fresh(now + 50, now, Duration::from_secs(1)));
    }

    #[test]
    fn cache_round_trips_and_busts_by_repo() {
        let dir = tempfile::tempdir().unwrap();
        let a = args(&["api", "repos/me/proj/commits?per_page=15"]);
        let key = cache_key(&a);
        write_cache_to(dir.path(), &key, &a, "[1,2,3]");

        let entry = read_cache_from(dir.path(), &key).expect("entry");
        assert_eq!(entry.body, "[1,2,3]");
        assert_eq!(entry.args, a);

        // Busting scans by args content, not by key.
        let other = args(&["api", "repos/other/repo/commits"]);
        write_cache_to(dir.path(), &cache_key(&other), &other, "[]");
        // Inline bust logic against the temp dir (bust_cache_containing uses
        // the real cache dir; the scan logic is identical).
        for e in std::fs::read_dir(dir.path()).unwrap().flatten() {
            let raw = std::fs::read_to_string(e.path()).unwrap();
            let entry: CacheEntry = serde_json::from_str(&raw).unwrap();
            if entry.args.iter().any(|s| s.contains("me/proj")) {
                std::fs::remove_file(e.path()).unwrap();
            }
        }
        assert!(read_cache_from(dir.path(), &key).is_none());
        assert!(read_cache_from(dir.path(), &cache_key(&other)).is_some());
    }

    #[test]
    fn points_parse_from_labels() {
        assert_eq!(points_from_label("sp:3"), Some(3));
        assert_eq!(points_from_label("points: 5"), Some(5));
        assert_eq!(points_from_label("8"), Some(8));
        assert_eq!(points_from_label("bug-2"), None);
    }
}
