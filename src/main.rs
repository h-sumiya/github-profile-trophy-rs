mod constants;
mod error;
mod github;
mod html;
mod models;
mod params;
mod svg;
mod themes;
mod trophy;

use std::{env, sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{OriginalUri, RawQuery, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use constants::{
    CACHE_MAX_AGE, CDN_CACHE_MAX_AGE, DEFAULT_MARGIN_H, DEFAULT_MARGIN_W, DEFAULT_MAX_COLUMN,
    DEFAULT_MAX_ROW, DEFAULT_NO_BACKGROUND, DEFAULT_NO_FRAME, DEFAULT_PANEL_SIZE,
    STALE_WHILE_REVALIDATE, SVG_CACHE_TTL_SECS, USER_CACHE_TTL_SECS,
};
use error::ServiceError;
use github::GithubClient;
use moka::future::Cache;
use params::ParsedParams;
use svg::Card;
use themes::resolve_theme;
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    github: Arc<GithubClient>,
    default_username: Option<String>,
    user_cache: Cache<String, Arc<models::UserInfo>>,
    svg_cache: Cache<String, Bytes>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "github_profile_trophy_rs=info,tower_http=info".into()),
        )
        .compact()
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let github_api = env::var("GITHUB_API").ok();
    let mut tokens = vec![
        env::var("GITHUB_TOKEN1").ok(),
        env::var("GITHUB_TOKEN2").ok(),
    ]
    .into_iter()
    .flatten()
    .filter(|token| !token.trim().is_empty())
    .collect::<Vec<_>>();

    if let Ok(token) = env::var("GITHUB_TOKEN")
        && !token.trim().is_empty()
    {
        tokens.push(token);
    }

    if tokens.is_empty() {
        warn!(
            "No GitHub token found. Set GITHUB_TOKEN1/GITHUB_TOKEN2 (or GITHUB_TOKEN) to avoid GraphQL auth failures."
        );
    }

    let single_token_mode = tokens.len() == 1;
    let github = Arc::new(GithubClient::new(github_api, tokens)?);
    let default_username = if single_token_mode {
        match github.request_authenticated_username().await {
            Ok(username) => {
                info!("single token mode enabled for username='{username}'");
                Some(username)
            }
            Err(err) => {
                warn!("failed to resolve username from single token: {err}");
                None
            }
        }
    } else {
        None
    };

    let user_cache = Cache::builder()
        .max_capacity(20_000)
        .time_to_live(Duration::from_secs(USER_CACHE_TTL_SECS))
        .build();

    let svg_cache = Cache::builder()
        .max_capacity(20_000)
        .time_to_live(Duration::from_secs(SVG_CACHE_TTL_SECS))
        .build();

    let state = AppState {
        github,
        default_username,
        user_cache,
        svg_cache,
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/healthz", get(health_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, port)).await?;
    info!("listening on 0.0.0.0:{port}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn index_handler(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let params = ParsedParams::from_raw(raw_query.as_deref());

    let username = match resolve_username(
        params.get_optional_string("username"),
        state.default_username.as_deref(),
    ) {
        Some(value) => value,
        None => {
            let body = html::missing_username_page(uri.path());
            return html_response(StatusCode::BAD_REQUEST, body);
        }
    };
    let include_private = should_include_private(state.default_username.as_deref(), &username);

    let row = params.get_number_value("row", DEFAULT_MAX_ROW).max(1);
    let mut column = params.get_number_value("column", DEFAULT_MAX_COLUMN);
    if column != -1 && column < 1 {
        column = DEFAULT_MAX_COLUMN;
    }

    let theme_name = params.get_string_value("theme", "default");
    let theme = resolve_theme(&theme_name);

    let margin_width = params.get_number_value("margin-w", DEFAULT_MARGIN_W);
    let margin_height = params.get_number_value("margin-h", DEFAULT_MARGIN_H);
    let no_background = params.get_boolean_value("no-bg", DEFAULT_NO_BACKGROUND);
    let no_frame = params.get_boolean_value("no-frame", DEFAULT_NO_FRAME);
    let titles = params.get_all_csv("title");
    let ranks = params.get_all_csv("rank");

    let request_cache_key = cache_key(uri.path(), raw_query.as_deref());
    if let Some(svg) = state.svg_cache.get(&request_cache_key).await {
        return svg_response(svg);
    }

    let user_key_cache = format!("v2-{username}-private={include_private}");
    let user_info = if let Some(cached) = state.user_cache.get(&user_key_cache).await {
        cached
    } else {
        match state
            .github
            .request_user_info(&username, include_private)
            .await
        {
            Ok(user_info) => {
                let user_info = Arc::new(user_info);
                state
                    .user_cache
                    .insert(user_key_cache.clone(), user_info.clone())
                    .await;
                user_info
            }
            Err(err) => {
                error!("GitHub API error for username='{username}': {err}");
                return error_response(err);
            }
        }
    };

    let svg = Card::new(
        titles,
        ranks,
        column,
        row,
        DEFAULT_PANEL_SIZE,
        margin_width,
        margin_height,
        no_background,
        no_frame,
    )
    .render(&user_info, theme);

    let svg_bytes = Bytes::from(svg);
    state
        .svg_cache
        .insert(request_cache_key, svg_bytes.clone())
        .await;

    svg_response(svg_bytes)
}

async fn health_handler() -> impl IntoResponse {
    "ok"
}

fn cache_key(path: &str, raw_query: Option<&str>) -> String {
    let query = raw_query.unwrap_or_default();
    format!("v1:{path}?{query}")
}

fn resolve_username(
    requested_username: Option<String>,
    default_username: Option<&str>,
) -> Option<String> {
    requested_username.or_else(|| default_username.map(str::to_string))
}

fn should_include_private(default_username: Option<&str>, requested_username: &str) -> bool {
    default_username
        .map(|username| username.eq_ignore_ascii_case(requested_username))
        .unwrap_or(false)
}

fn cache_control_header() -> String {
    format!(
        "public, max-age={CACHE_MAX_AGE}, s-maxage={CDN_CACHE_MAX_AGE}, stale-while-revalidate={STALE_WHILE_REVALIDATE}"
    )
}

fn svg_response(svg: Bytes) -> Response {
    let mut response = Response::new(Body::from(svg));
    *response.status_mut() = StatusCode::OK;

    let cache_control = cache_control_header();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml"),
    );
    if let Ok(value) = HeaderValue::from_str(&cache_control) {
        headers.insert(header::CACHE_CONTROL, value);
    }

    response
}

fn html_response(status_code: StatusCode, body: String) -> Response {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status_code;

    let cache_control = cache_control_header();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    if let Ok(value) = HeaderValue::from_str(&cache_control) {
        headers.insert(header::CACHE_CONTROL, value);
    }

    response
}

fn error_response(error: ServiceError) -> Response {
    let body = html::error_page(&error);
    let status = StatusCode::from_u16(error.status_code()).unwrap_or(StatusCode::BAD_REQUEST);
    html_response(status, body)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("failed to install Ctrl+C handler: {err}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
            sigterm.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_username, should_include_private};

    #[test]
    fn resolve_username_prefers_request_param() {
        let resolved = resolve_username(Some("requested".to_string()), Some("default"));
        assert_eq!(resolved, Some("requested".to_string()));
    }

    #[test]
    fn resolve_username_falls_back_to_default() {
        let resolved = resolve_username(None, Some("default"));
        assert_eq!(resolved, Some("default".to_string()));
    }

    #[test]
    fn resolve_username_missing_both_returns_none() {
        let resolved = resolve_username(None, None);
        assert_eq!(resolved, None);
    }

    #[test]
    fn should_include_private_when_usernames_match() {
        assert!(should_include_private(Some("Alice"), "alice"));
    }

    #[test]
    fn should_not_include_private_when_usernames_differ() {
        assert!(!should_include_private(Some("alice"), "bob"));
    }
}
