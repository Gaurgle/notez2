//! Real GitHub data for the desktop views, sourced by shelling out to the
//! already-authenticated `gh` CLI. No HTTP client, no token handling: `gh`
//! owns auth, and this module just parses its JSON.
//!
//! Everything here is best-effort. A missing `gh`, a private repo the user
//! can't see, or an empty repo all degrade to an empty list rather than an
//! error the UI has to special-case, so the views stay calm when offline.

use serde::Serialize;
use serde_json::Value;
use std::process::Command;

/// One repository in an org.
#[derive(Serialize)]
pub struct GhRepo {
    pub name: String,
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
    pub repo: String,
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
    pub repo: String,
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
        // `gh` not on PATH (NotFound): retry the known Homebrew location.
        Err(_) => attempt("/opt/homebrew/bin/gh")
            .map_err(|e| format!("could not run gh: {e}"))?,
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Parse a JSON string into a value, mapping errors to strings.
fn parse(json: &str) -> Result<Value, String> {
    serde_json::from_str(json).map_err(|e| format!("bad gh json: {e}"))
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// First line of a commit/issue body, trimmed.
fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

/// Pull a story-point count out of a label name like `sp:3`, `points: 5`,
/// `3-points`, or a bare `8`. Returns the first plausible number found.
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
    Ok(GhUser {
        login: str_field(&v, "login"),
        name: v
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(&str_field(&v, "login"))
            .to_string(),
        avatar_url: str_field(&v, "avatar_url"),
    })
}

/// Every repo in `org`, newest push first.
#[tauri::command]
pub fn github_repos(org: String) -> Result<Vec<GhRepo>, String> {
    let out = run_gh(&[
        "repo",
        "list",
        &org,
        "--limit",
        "100",
        "--json",
        "name,description,primaryLanguage,pushedAt,isPrivate,url",
    ])?;
    let v = parse(&out)?;
    let mut repos: Vec<GhRepo> = v
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| GhRepo {
                    name: str_field(r, "name"),
                    description: str_field(r, "description"),
                    language: r
                        .get("primaryLanguage")
                        .and_then(|l| l.get("name"))
                        .and_then(Value::as_str)
                        .map(String::from),
                    pushed_at: str_field(r, "pushedAt"),
                    open_issues: 0,
                    url: str_field(r, "url"),
                    is_private: r.get("isPrivate").and_then(Value::as_bool).unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();
    repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));
    Ok(repos)
}

/// Recent commits across the given repos, merged and sorted newest first.
/// `limit` caps how many commits are fetched *per repo* before merging.
#[tauri::command]
pub fn github_commits(
    org: String,
    repos: Vec<String>,
    limit: u32,
) -> Result<Vec<GhCommit>, String> {
    let per_page = limit.clamp(1, 100).to_string();
    let mut commits: Vec<GhCommit> = Vec::new();

    for repo in &repos {
        let path = format!("repos/{org}/{repo}/commits?per_page={per_page}");
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

/// All issues (open + closed) across the given repos, for the ticketz board.
/// Pull requests are excluded; `gh issue list` already does this.
#[tauri::command]
pub fn github_issues(org: String, repos: Vec<String>) -> Result<Vec<GhIssue>, String> {
    let mut issues: Vec<GhIssue> = Vec::new();

    for repo in &repos {
        let slug = format!("{org}/{repo}");
        let Ok(out) = run_gh(&[
            "issue",
            "list",
            "-R",
            &slug,
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

/// Contributors to a single repo, most commits first.
#[tauri::command]
pub fn github_contributors(org: String, repo: String) -> Result<Vec<GhContributor>, String> {
    let path = format!("repos/{org}/{repo}/contributors?per_page=20");
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
                    contributions: c
                        .get("contributions")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32,
                })
                .collect()
        })
        .unwrap_or_default())
}
