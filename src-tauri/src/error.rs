use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("audio decode error: {0}")]
    Audio(#[from] symphonia::core::errors::Error),

    #[error("{0}")]
    Message(String),
}

impl AppError {
    pub fn msg(message: impl Into<String>) -> Self {
        AppError::Message(message.into())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_error_displays_literal_text() {
        assert_eq!(AppError::msg("something broke").to_string(), "something broke");
    }

    #[test]
    fn io_error_is_wrapped_with_prefix() {
        let error: AppError = std::io::Error::new(std::io::ErrorKind::NotFound, "nope").into();
        assert!(error.to_string().starts_with("io error:"));
    }

    #[test]
    fn serializes_as_plain_string() {
        let json = serde_json::to_string(&AppError::msg("boom")).unwrap();
        assert_eq!(json, "\"boom\"");
    }
}
