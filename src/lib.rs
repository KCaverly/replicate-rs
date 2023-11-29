//! A simple http client for interacting with [Replicate](https://replicate.com/).  
//! Provides simple async functionality for interacting with Replicate via
//! [serde](https://serde.rs) and [isahc](https://docs.rs/isahc/latest/isahc/).
//!
//! # Getting Started
//!
//! Add the following to your cargo toml
//! ```toml
//! replicate-rs = "0.2.0"
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
//!
//! let config = ReplicateConfig::new().unwrap();
//! let prediction_client = PredictionClient::from(config);
//!
//! #[derive(Serialize)]
//! struct HelloWorldInput {
//!     text: String
//! }
//!
//! // The library is async agnostic, so you should be able to use any async runtime you please
//! tokio_test::block_on(async move {
//!
//!     // Create the prediction
//!     let prediction_input = Box::new(HelloWorldInput{ text: "kyle".to_string() });
//!     let mut prediction = prediction_client
//!         .create(
//!             "replicate",
//!             "hello-world",
//!             prediction_input
//!         )
//!         .await
//!         .unwrap();
//!
//!     // Refresh the data
//!     prediction.reload().await;
//! })
//! ```

#![warn(missing_docs)]

pub mod config;
pub mod models;
pub mod predictions;

use std::env::var;
use std::sync::OnceLock;

fn api_key() -> anyhow::Result<&'static str> {
    let api_key = var("REPLICATE_API_KEY")?;
    static REPLICATE_API_KEY: OnceLock<String> = OnceLock::new();
    anyhow::Ok(REPLICATE_API_KEY.get_or_init(|| api_key))
}

fn base_url() -> &'static str {
    static REPLICATE_BASE_URL: OnceLock<&'static str> = OnceLock::new();
    REPLICATE_BASE_URL.get_or_init(|| "https://api.replicate.com/v1")
}
