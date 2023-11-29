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
//! use replicate_rs::client::ReplicateClient;
//! use replicate_rs::predictions::PredictionClient;
//! use serde::Serialize;
//!
//! let client = ReplicateClient::new()?;
//! let prediction_client = PredictionClient::from(client);
//!
//! #[derive(Serialize)]
//! struct HelloWorldInput {
//!     text: "kyle"
//! }
//!
//! // Create the prediction
//! let prediction = prediction_client.create("replicate", "hello-world", prediction_input).await?;
//!
//! // Refresh the data
//! prediction.reload()
//! ```

mod client;
mod models;
mod predictions;

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
