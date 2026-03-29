use thiserror::Error;

#[derive(Debug, Error)]
pub enum TgvmaxError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    Parse(String),

    #[error("API returned error: {status} - {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, TgvmaxError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_api() {
        let err = TgvmaxError::Api {
            status: 403,
            message: "Forbidden".to_string(),
        };
        assert_eq!(err.to_string(), "API returned error: 403 - Forbidden");
    }

    #[test]
    fn error_display_parse() {
        let err = TgvmaxError::Parse("invalid json".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to parse API response: invalid json"
        );
    }
}
