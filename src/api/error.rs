use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Authentication failed")]
    Authentication,

    #[error("Resource not found")]
    NotFound,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Rate limiter error: {0}")]
    RateLimiter(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::RateLimit => true,
            ApiError::ServiceUnavailable => true,
            ApiError::Http(e) => e.is_timeout() || e.is_connect(),
            ApiError::Api { status, .. } => {
                *status == 429
                    || *status == 500
                    || *status == 502
                    || *status == 503
                    || *status == 504
            }
            _ => false,
        }
    }

    pub fn should_retry_after_delay(&self) -> bool {
        matches!(self, ApiError::RateLimit | ApiError::ServiceUnavailable)
    }
}
