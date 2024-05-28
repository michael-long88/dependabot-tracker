use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::app::DependabotTrackerError;
use crate::dependabot::{Dependabot, DependabotSeverity, DependabotState, GithubDependabot};
use crate::repository_list::RepositoryList;
use crate::trace_dbg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    id: u32,
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
    archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u32,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub url: String,
    pub archived: bool,
    pub dependabots: Vec<Dependabot>,
    pub low_alerts: usize,
    pub medium_alerts: usize,
    pub high_alerts: usize,
    pub critical_alerts: usize,
    pub total_active_alerts: usize,
}

pub fn fetch_github_repos(
    username: &str,
    token: &str,
) -> Result<RepositoryList, DependabotTrackerError> {
    let url = "https://api.github.com/user/repos?affiliation=owner&per_page=100";

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| Box::new(e) as DependabotTrackerError)?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .map_err(|e| Box::new(e) as DependabotTrackerError)?;

    let repos: Vec<GitHubRepository> = response
        .json()
        .map_err(|e| Box::new(e) as DependabotTrackerError)?;

    let updated_repos = fetch_dependabot_alerts(token, username, &repos)?;

    let file_location = PathBuf::from(".").join("data").join("repositories.json");
    let file = std::fs::File::create(file_location).unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, &updated_repos).unwrap();

    Ok(RepositoryList::with_respositories(updated_repos))
}

fn fetch_dependabot_alerts(
    token: &str,
    username: &str,
    repositories: &[GitHubRepository],
) -> Result<Vec<Repository>, DependabotTrackerError> {
    let client = reqwest::blocking::Client::new();

    let updated_repos: Vec<Repository> = repositories
        .iter()
        .map(|repo| fetch_repo_depenabot_alerts(token, username, repo, &client))
        .filter_map(|result| result.ok())
        .collect();

    Ok(updated_repos)
}

fn fetch_repo_depenabot_alerts(
    token: &str,
    username: &str,
    repository: &GitHubRepository,
    client: &Client,
) -> Result<Repository, DependabotTrackerError> {
    let fetch_repo_dependabot_alert_trace =
        format!("fetching dependabot alerts for {}", repository.name);
    trace_dbg!(level: tracing::Level::INFO, fetch_repo_dependabot_alert_trace);

    let url = format!(
        "https://api.github.com/repos/{}/{}/dependabot/alerts?per_page=100",
        username, repository.name
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .map_err(|e| Box::new(e) as DependabotTrackerError)?;

    if response.status().is_client_error() {
        let repo_dependabot_not_enabled =
            format!("Dependabot alerts not enable for {}", repository.name);
        trace_dbg!(level: tracing::Level::WARN, repo_dependabot_not_enabled);

        return Ok(Repository {
            id: repository.id,
            name: repository.name.clone(),
            full_name: repository.full_name.clone(),
            private: repository.private,
            url: repository.html_url.clone(),
            archived: repository.archived,
            dependabots: Vec::new(),
            low_alerts: 0,
            medium_alerts: 0,
            high_alerts: 0,
            critical_alerts: 0,
            total_active_alerts: 0,
        });
    }

    let github_dependabots: Vec<GithubDependabot> = response
        .json()
        .map_err(|e| Box::new(e) as DependabotTrackerError)?;

    let dependabots: Vec<Dependabot> = github_dependabots
        .into_iter()
        .map(|github_dependabot| Dependabot {
            number: github_dependabot.number,
            state: github_dependabot.state,
            severity: github_dependabot.security_vulnerability.severity,
            html_url: github_dependabot.html_url,
            created_at: github_dependabot.created_at,
            updated_at: github_dependabot.updated_at,
            dismissed_at: github_dependabot.dismissed_at,
            dependency_ecosystem: github_dependabot.security_vulnerability.package.ecosystem,
            dependency_name: github_dependabot.security_vulnerability.package.name,
        })
        .collect();

    let low_alerts = dependabots
        .iter()
        .filter(|dependabot| {
            dependabot.state == DependabotState::Open
                && dependabot.severity == DependabotSeverity::Low
        })
        .count();
    let medium_alerts = dependabots
        .iter()
        .filter(|dependabot| {
            dependabot.state == DependabotState::Open
                && dependabot.severity == DependabotSeverity::Medium
        })
        .count();
    let high_alerts = dependabots
        .iter()
        .filter(|dependabot| {
            dependabot.state == DependabotState::Open
                && dependabot.severity == DependabotSeverity::High
        })
        .count();
    let critical_alerts = dependabots
        .iter()
        .filter(|dependabot| {
            dependabot.state == DependabotState::Open
                && dependabot.severity == DependabotSeverity::Critical
        })
        .count();
    let total_active_alerts = low_alerts + medium_alerts + high_alerts + critical_alerts;

    Ok(Repository {
        id: repository.id,
        name: repository.name.clone(),
        full_name: repository.full_name.clone(),
        private: repository.private,
        url: repository.html_url.clone(),
        archived: repository.archived,
        dependabots,
        low_alerts,
        medium_alerts,
        high_alerts,
        critical_alerts,
        total_active_alerts,
    })
}
