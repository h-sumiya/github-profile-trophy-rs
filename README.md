# github-profile-trophy-rs

A Rust port of [`github-profile-trophy`](https://github.com/ryo-ma/github-profile-trophy).  
It runs as a standalone single binary without Docker, based on `axum`.

## Features

- High-throughput HTTP server using `axum` + `tokio`
- Parallel execution of 4 GitHub GraphQL queries
- Reuses `reqwest` connection pools
- In-memory TTL cache
  - User information: 4 hours
  - Generated SVG: 1 hour
- When using a single token, it resolves `viewer.login` at startup, allowing you to omit the `username` parameter
- Includes private repositories in the aggregation when requesting data for the single token's owner
- Compatible with existing query parameters: `username`, `title`, `rank`, `row`, `column`, `theme`, `margin-w`, `margin-h`, `no-bg`, `no-frame`

## Requirements

- Rust (stable)
- GitHub Personal Access Token (recommended to use the GraphQL API)

## Environment Variables

- `PORT` (default: `8080`)
- `GITHUB_API` (default: `https://api.github.com/graphql`)
- `GITHUB_TOKEN1`
- `GITHUB_TOKEN2`
- `GITHUB_TOKEN` (Use this if you only want to provide a single token as an alternative to `GITHUB_TOKEN1/2`)

## Usage

```bash
cargo run --release

```

Access examples:

```text
http://localhost:8080/?username=h-sumiya
http://localhost:8080/?username=h-sumiya&theme=onedark&column=6
http://localhost:8080/                         # Only available when using a single token
```

## Building a Single Binary

```bash
cargo build --release

```

Output:

```text
./target/release/github-profile-trophy-rs

```

You can run this standalone binary directly.

## Implementation Differences

- Removed dependencies on Redis / Docker (targeted for non-Docker environments)
- Server-side caching is replaced with an in-memory implementation
