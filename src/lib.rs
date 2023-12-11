//! A simple http client for interacting with [Replicate](https://replicate.com/).  
//! Provides simple async functionality for interacting with Replicate via
//! [serde](https://serde.rs) and [isahc](https://docs.rs/isahc/latest/isahc/).
//!
//! # Getting Started
//!
//! Add the following to your cargo toml
//! ```toml
//! replicate-rs = "0.7.0"
//! ```
//!
//! # Examples
//!
//! #### Create a Prediction
//!
//! Create a prediction, and get refreshed prediction data.
//!
//! ```rust
//! use replicate_rs::config::ReplicateConfig;
//! use replicate_rs::predictions::PredictionClient;
//! use serde::Serialize;
//! use serde_json::json;
//!
//! // The library is async agnostic, so you should be able to use any async runtime you please
//! //tokio_test::block_on(async move {
//! #[tokio::main]
//! async fn main() {
//!     tokio::spawn(async move {
//!
//!         let config = ReplicateConfig::new().unwrap();
//!         let prediction_client = PredictionClient::from(config);
//!
//!         // Create the prediction
//!         let mut prediction = prediction_client
//!             .create(
//!                 "replicate",
//!                 "hello-world",
//!                 json!({"text": "kyle"}),
//!                 false
//!             )
//!             .await
//!             .unwrap();
//!
//!         // Refresh the data
//!         prediction.reload().await;
//!     });
//! }
//! ```

#![warn(missing_docs)]

pub mod config;
pub mod errors;
pub mod models;
pub mod predictions;

use crate::errors::{ReplicateError, ReplicateResult};
use std::env::var;
use std::sync::OnceLock;

fn api_key() -> ReplicateResult<&'static str> {
    let api_key = var("REPLICATE_API_KEY").map_err(|_| {
        ReplicateError::MissingCredentials(
            "REPLICATE_API_KEY not available in environment variables.".to_string(),
        )
    })?;

    static REPLICATE_API_KEY: OnceLock<String> = OnceLock::new();
    Ok(REPLICATE_API_KEY.get_or_init(|| api_key))
}

fn base_url() -> &'static str {
    static REPLICATE_BASE_URL: OnceLock<&'static str> = OnceLock::new();
    REPLICATE_BASE_URL.get_or_init(|| "https://api.replicate.com/v1")
}
