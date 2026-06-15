//! Real GitHub data for the desktop views, sourced by shelling out to the
//! already-authenticated `gh` CLI. No HTTP client, no token handling: `gh`
//! owns auth, and this module just parses its JSON.
//!
//! Repos are identified everywhere by their full `owner/repo` name, so personal
//! repos and several orgs coexist without collisions. Everything here is
//! best-effort: a missing `gh`, an unreachable repo, or an empty repo all
//! degrade to an empty list rather than an error the UI has to special-case.

use serde::Serialize;
use serde_json::Value;
use std::process::Command;

/// One repository the signed-in user can reach.
#[derive(Serialize)]
pub struct GhRepo {
    pub name: String,
    pub full_name: String, // owner/repo — the stable key used everywhere
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

/// Run `gh` with the given args, returning stdout on success. Falls back to the
/// Homebrew path when `gh` is not on the app's `PATH` (the case when the app is
/// launched from Finder rather than a terminal).
fn run_gh(args: &[&str]) -> Result<String, String> {
    let attempt = |bin: &str| Command::new(bin).args(args).output();

    let output = match attempt("gh") {
        Ok(o) => o,
        Err(_) => attempt("/opt/homebrew/bin/gh").map_err(|e| format!("could not run gh: {e}"))?,
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

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

/// The signed-in GitHub user.
#[tauri::command]
pub fn github_user() -> Result<GhUser, String> {
    let out = run_gh(&["api", "user"])?;
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
pub fn github_all_repos() -> Result<Vec<GhRepo>, String> {
    let out = run_gh(&[
        "api",
        "--paginate",
        "--slurp",
        "user/repos?affiliation=owner,collaborator,organization_member&sort=pushed&per_page=100",
    ])?;
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

/// Recent commits across the given `owner/repo` repos, merged newest first.
/// `limit` caps commits fetched per repo before merging.
#[tauri::command]
pub fn github_commits(repos: Vec<String>, limit: u32) -> Result<Vec<GhCommit>, String> {
    let per_page = limit.clamp(1, 100).to_string();
    let mut commits: Vec<GhCommit> = Vec::new();

    for repo in &repos {
        let path = format!("repos/{repo}/commits?per_page={per_page}");
        // A brand-new/empty repo 409s here; treat any failure as "no commits".
        let Ok(out) = run_gh(&["api", &path]) else {
            continue;
        };
        let Ok(v) = parse(&out) else { continue };
        let Some(arr) = v.as_array() else { continue };

        for c in arr {
            let commit = c.get("commit");
            let author_obj = c.get("author");
            commits.push(GhCommit {
                sha: str_field(c, "sha"),
                repo: repo.clone(),
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
            });
        }
    }

    commits.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(commits)
}

/// All issues (open + closed) across the given `owner/repo` repos. PRs are
/// excluded; `gh issue list` already does this.
#[tauri::command]
pub fn github_issues(repos: Vec<String>) -> Result<Vec<GhIssue>, String> {
    let mut issues: Vec<GhIssue> = Vec::new();

    for repo in &repos {
        let Ok(out) = run_gh(&[
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
        ]) else {
            continue;
        };
        let Ok(v) = parse(&out) else { continue };
        let Some(arr) = v.as_array() else { continue };

        for it in arr {
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

            issues.push(GhIssue {
                number: it.get("number").and_then(Value::as_u64).unwrap_or(0),
                repo: repo.clone(),
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
            });
        }
    }

    issues.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(issues)
}

/// Create a real issue in the `owner/repo`. Returns the new issue number so the
/// caller can refetch the board. User-initiated only (the New button).
#[tauri::command]
pub fn github_create_issue(repo: String, title: String, body: String) -> Result<u64, String> {
    if title.trim().is_empty() {
        return Err("issue title is empty".into());
    }
    // gh prints the new issue's URL on success; the number is its last segment.
    let out = run_gh(&[
        "issue", "create", "-R", &repo, "--title", &title, "--body", &body,
    ])?;
    out.trim()
        .rsplit('/')
        .next()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .ok_or_else(|| format!("could not parse new issue number from: {}", out.trim()))
}

/// Contributors to a single `owner/repo`, most commits first.
#[tauri::command]
pub fn github_contributors(repo: String) -> Result<Vec<GhContributor>, String> {
    let path = format!("repos/{repo}/contributors?per_page=20");
    let out = match run_gh(&["api", &path]) {
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
