use reqwest::StatusCode;
use serde::Deserialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ReplicateError {
    MissingCredentials(String),
    InvalidCredentials(String),
    PaymentNeeded(String),
    SerializationError(String),
    ClientError(String),
    InvalidRequest(String),
    Misc(String),
}

impl fmt::Display for ReplicateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplicateError::MissingCredentials(message)
            | ReplicateError::PaymentNeeded(message)
            | ReplicateError::ClientError(message)
            | ReplicateError::Misc(message)
            | ReplicateError::SerializationError(message) => {
                write!(f, "{message}")
            }
            _ => {
                write!(f, "unknown replicate error")
            }
        }
    }
}

#[derive(Deserialize)]
struct ErrorData {
    title: String,
    detail: String,
}

/// Result Alias for Replicate Output and Errors
pub type ReplicateResult<T> = std::result::Result<T, ReplicateError>;

pub(crate) fn get_error(status: reqwest::StatusCode, data: &str) -> ReplicateError {
    match status {
        StatusCode::PAYMENT_REQUIRED => {
            let data: Option<ErrorData> = serde_json::from_str(data).ok();
            if let Some(data) = data {
                ReplicateError::PaymentNeeded(format!("{}: {}", data.title, data.detail))
            } else {
                ReplicateError::PaymentNeeded("error details not available".to_string())
            }
        }
        StatusCode::UNAUTHORIZED => {
            let data: Option<ErrorData> = serde_json::from_str(data).ok();
            if let Some(data) = data {
                ReplicateError::InvalidCredentials(format!("{}: {}", data.title, data.detail))
            } else {
                ReplicateError::InvalidCredentials("error details not available".to_string())
            }
        }
        _ => {
            println!("DATA: {:?}", data);
            let data: Option<ErrorData> = serde_json::from_str(data).ok();
            if let Some(data) = data {
                ReplicateError::Misc(format!("{}: {}", data.title, data.detail))
            } else {
                ReplicateError::Misc("error details not available".to_string())
            }
        }
    }
}
