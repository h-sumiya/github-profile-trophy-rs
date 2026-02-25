pub const CACHE_MAX_AGE: u32 = 18_800;
pub const CDN_CACHE_MAX_AGE: u32 = 28_800;
pub const STALE_WHILE_REVALIDATE: u32 = 86_400;

pub const DEFAULT_PANEL_SIZE: i32 = 110;
pub const DEFAULT_MAX_COLUMN: i32 = 8;
pub const DEFAULT_MAX_ROW: i32 = 3;
pub const DEFAULT_MARGIN_W: i32 = 0;
pub const DEFAULT_MARGIN_H: i32 = 0;
pub const DEFAULT_NO_BACKGROUND: bool = false;
pub const DEFAULT_NO_FRAME: bool = false;

pub const DEFAULT_GITHUB_API: &str = "https://api.github.com/graphql";
pub const DEFAULT_GITHUB_RETRY_DELAY_MS: u64 = 500;

pub const SVG_CACHE_TTL_SECS: u64 = 60 * 60;
pub const USER_CACHE_TTL_SECS: u64 = 60 * 60 * 4;
