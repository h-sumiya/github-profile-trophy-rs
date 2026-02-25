use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::json;
use tokio::try_join;

use crate::{
    constants::{DEFAULT_GITHUB_API, DEFAULT_GITHUB_RETRY_DELAY_MS},
    error::ServiceError,
    models::{UserActivity, UserInfo, UserIssue, UserPullRequest, UserRepository},
};

const QUERY_USER_ACTIVITY: &str = r#"
query userInfo($username: String!) {
  user(login: $username) {
    createdAt
    contributionsCollection {
      totalCommitContributions
      restrictedContributionsCount
      totalPullRequestReviewContributions
    }
    organizations(first: 1) {
      totalCount
    }
    followers(first: 1) {
      totalCount
    }
  }
}
"#;

const QUERY_USER_ISSUE: &str = r#"
query userInfo($username: String!) {
  user(login: $username) {
    openIssues: issues(states: OPEN) {
      totalCount
    }
    closedIssues: issues(states: CLOSED) {
      totalCount
    }
  }
}
"#;

const QUERY_USER_PULL_REQUEST: &str = r#"
query userInfo($username: String!) {
  user(login: $username) {
    pullRequests(first: 1) {
      totalCount
    }
  }
}
"#;

const QUERY_USER_REPOSITORY: &str = r#"
query userInfo($username: String!) {
  user(login: $username) {
    repositories(first: 50, ownerAffiliations: OWNER, orderBy: {direction: DESC, field: STARGAZERS}) {
      totalCount
      nodes {
        languages(first: 3, orderBy: {direction: DESC, field: SIZE}) {
          nodes {
            name
          }
        }
        stargazers {
          totalCount
        }
        createdAt
      }
    }
  }
}
"#;

#[derive(Clone)]
pub struct GithubClient {
    http_client: reqwest::Client,
    github_api: String,
    tokens: Vec<String>,
}

impl GithubClient {
    pub fn new(github_api: Option<String>, tokens: Vec<String>) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("github-profile-trophy-rs"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .pool_max_idle_per_host(16)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .build()?;

        Ok(Self {
            http_client,
            github_api: github_api.unwrap_or_else(|| DEFAULT_GITHUB_API.to_string()),
            tokens,
        })
    }

    pub async fn request_user_info(&self, username: &str) -> Result<UserInfo, ServiceError> {
        let repository = self.request_user_repository(username);
        let activity = self.request_user_activity(username);
        let issue = self.request_user_issue(username);
        let pull_request = self.request_user_pull_request(username);

        let (repository, activity, issue, pull_request) =
            try_join!(repository, activity, issue, pull_request)?;

        Ok(UserInfo::from_parts(
            activity,
            issue,
            pull_request,
            repository,
        ))
    }

    pub async fn request_user_repository(
        &self,
        username: &str,
    ) -> Result<UserRepository, ServiceError> {
        self.execute_query(QUERY_USER_REPOSITORY, username).await
    }

    pub async fn request_user_activity(
        &self,
        username: &str,
    ) -> Result<UserActivity, ServiceError> {
        self.execute_query(QUERY_USER_ACTIVITY, username).await
    }

    pub async fn request_user_issue(&self, username: &str) -> Result<UserIssue, ServiceError> {
        self.execute_query(QUERY_USER_ISSUE, username).await
    }

    pub async fn request_user_pull_request(
        &self,
        username: &str,
    ) -> Result<UserPullRequest, ServiceError> {
        self.execute_query(QUERY_USER_PULL_REQUEST, username).await
    }

    async fn execute_query<T: DeserializeOwned>(
        &self,
        query: &str,
        username: &str,
    ) -> Result<T, ServiceError> {
        let payload = json!({
            "query": query,
            "variables": {
                "username": username,
            }
        });

        let attempts = self.tokens.len().max(1);
        let mut last_error = ServiceError::NotFound;

        for attempt in 0..attempts {
            let token = self
                .tokens
                .get(attempt)
                .map(String::as_str)
                .unwrap_or_default();

            match self.execute_query_once::<T>(&payload, token).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = err;

                    if attempt + 1 < attempts {
                        tokio::time::sleep(Duration::from_millis(DEFAULT_GITHUB_RETRY_DELAY_MS))
                            .await;
                    }
                }
            }
        }

        Err(last_error)
    }

    async fn execute_query_once<T: DeserializeOwned>(
        &self,
        payload: &serde_json::Value,
        token: &str,
    ) -> Result<T, ServiceError> {
        let mut request = self.http_client.post(&self.github_api).json(payload);

        if !token.is_empty() {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(|_| ServiceError::NotFound)?;
        let body: GraphqlResponse<T> = response.json().await.map_err(|_| ServiceError::NotFound)?;

        let is_rate_limited = body.is_rate_limited();

        if let Some(data) = body.data
            && let Some(user) = data.user
        {
            return Ok(user);
        }

        if is_rate_limited {
            return Err(ServiceError::RateLimit);
        }

        Err(ServiceError::NotFound)
    }
}

#[derive(Debug, Deserialize)]
struct GraphqlResponse<T> {
    data: Option<GraphqlData<T>>,
    #[serde(default)]
    errors: Vec<GraphqlError>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphqlData<T> {
    user: Option<T>,
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    message: String,
}

impl<T> GraphqlResponse<T> {
    fn is_rate_limited(&self) -> bool {
        self.message
            .as_ref()
            .map(|message| message.to_ascii_lowercase().contains("rate limit"))
            .unwrap_or(false)
            || self.errors.iter().any(|error| {
                error.r#type.to_ascii_uppercase().contains("RATE_LIMIT")
                    || error.message.to_ascii_lowercase().contains("rate limit")
            })
    }
}
