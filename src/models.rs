use std::collections::HashSet;

use chrono::{Datelike, TimeZone, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserActivity {
    pub created_at: String,
    pub contributions_collection: ContributionsCollection,
    pub organizations: TotalCount,
    pub followers: TotalCount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionsCollection {
    pub total_commit_contributions: i64,
    pub restricted_contributions_count: i64,
    pub total_pull_request_review_contributions: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIssue {
    pub open_issues: TotalCount,
    pub closed_issues: TotalCount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPullRequest {
    pub pull_requests: TotalCount,
}

#[derive(Debug, Deserialize)]
pub struct UserRepository {
    pub repositories: Repositories,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repositories {
    pub total_count: i64,
    pub nodes: Vec<Option<RepositoryNode>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryNode {
    pub languages: Languages,
    pub stargazers: TotalCount,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Languages {
    pub nodes: Vec<Option<LanguageNode>>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageNode {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalCount {
    pub total_count: i64,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub total_commits: i64,
    pub total_followers: i64,
    pub total_issues: i64,
    pub total_organizations: i64,
    pub total_pull_requests: i64,
    pub total_reviews: i64,
    pub total_stargazers: i64,
    pub total_repositories: i64,
    pub language_count: i64,
    pub duration_year: i64,
    pub duration_days: i64,
    pub ancient_account: i64,
    pub joined_2020: i64,
    pub og_account: i64,
}

impl UserInfo {
    pub fn from_parts(
        user_activity: UserActivity,
        user_issue: UserIssue,
        user_pull_request: UserPullRequest,
        user_repository: UserRepository,
    ) -> Self {
        let total_commits = user_activity
            .contributions_collection
            .restricted_contributions_count
            + user_activity
                .contributions_collection
                .total_commit_contributions;

        let mut total_stargazers = 0i64;
        let mut languages: HashSet<String> = HashSet::new();

        let mut earliest_repo_date = user_activity.created_at.clone();
        let mut earliest_ts = parse_rfc3339_to_millis(&earliest_repo_date).unwrap_or(0);

        for repo in user_repository.repositories.nodes.iter().flatten() {
            total_stargazers += repo.stargazers.total_count;

            for lang in repo.languages.nodes.iter().flatten() {
                languages.insert(lang.name.clone());
            }

            if let Some(ts) = parse_rfc3339_to_millis(&repo.created_at)
                && ts < earliest_ts
            {
                earliest_ts = ts;
                earliest_repo_date = repo.created_at.clone();
            }
        }

        let now_ts = Utc::now().timestamp_millis();
        let duration_time = (now_ts - earliest_ts).max(0);
        let duration_year = Utc
            .timestamp_millis_opt(duration_time)
            .single()
            .map(|dt| i64::from(dt.year() - 1970))
            .unwrap_or(0);
        let duration_days = duration_time / (1000 * 60 * 60 * 24) / 100;

        let earliest_year = parse_rfc3339_to_year(&earliest_repo_date).unwrap_or(1970);

        Self {
            total_commits,
            total_followers: user_activity.followers.total_count,
            total_issues: user_issue.open_issues.total_count + user_issue.closed_issues.total_count,
            total_organizations: user_activity.organizations.total_count,
            total_pull_requests: user_pull_request.pull_requests.total_count,
            total_reviews: user_activity
                .contributions_collection
                .total_pull_request_review_contributions,
            total_stargazers,
            total_repositories: user_repository.repositories.total_count,
            language_count: languages.len() as i64,
            duration_year,
            duration_days,
            ancient_account: i64::from(earliest_year <= 2010),
            joined_2020: i64::from(earliest_year == 2020),
            og_account: i64::from(earliest_year <= 2008),
        }
    }
}

fn parse_rfc3339_to_millis(input: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(input)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

fn parse_rfc3339_to_year(input: &str) -> Option<i32> {
    chrono::DateTime::parse_from_rfc3339(input)
        .ok()
        .map(|dt| dt.year())
}
