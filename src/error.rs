use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ServiceError {
    #[error("rate limit exceeded")]
    RateLimit,
    #[error("not found")]
    NotFound,
}

impl ServiceError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::RateLimit => 419,
            Self::NotFound => 404,
        }
    }
}
