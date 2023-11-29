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
