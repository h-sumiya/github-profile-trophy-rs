# github-profile-trophy-rs

`github-profile-trophy` の Rust 移植版です。  
`axum` ベース、非 Docker、単一バイナリで動作します。

## 特徴

- `axum` + `tokio` による高スループット HTTP サーバー
- GitHub GraphQL 4クエリを並列実行
- `reqwest` 接続プール再利用
- インメモリ TTL キャッシュ
  - ユーザー情報: 4時間
  - 生成 SVG: 1時間
- 既存クエリ互換: `username`, `title`, `rank`, `row`, `column`, `theme`, `margin-w`, `margin-h`, `no-bg`, `no-frame`

## 必要環境

- Rust (stable)
- GitHub Personal Access Token (GraphQL 利用のため推奨)

## 環境変数

- `PORT` (default: `8080`)
- `GITHUB_API` (default: `https://api.github.com/graphql`)
- `GITHUB_TOKEN1`
- `GITHUB_TOKEN2`
- `GITHUB_TOKEN` (`GITHUB_TOKEN1/2` の代替として1つだけ使いたい場合)

## 実行

```bash
cargo run --release
```

アクセス例:

```text
http://localhost:8080/?username=h-sumiya
http://localhost:8080/?username=h-sumiya&theme=onedark&column=6
```

## 単一バイナリ作成

```bash
cargo build --release
```

出力:

```text
./target/release/github-profile-trophy-rs
```

このバイナリ単体で実行できます。

## 実装上の差分

- Redis / Docker 依存は排除（非 Docker 運用向け）
- サーバー内キャッシュはインメモリ実装に変更
