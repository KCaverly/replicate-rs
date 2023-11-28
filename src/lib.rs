mod models;
mod predictions;

use std::env::var;
use std::sync::OnceLock;
fn api_key() -> anyhow::Result<&'static str> {
    let api_key = var("REPLICATE_API_KEY")?;
    static REPLICATE_API_KEY: OnceLock<String> = OnceLock::new();
    anyhow::Ok(REPLICATE_API_KEY.get_or_init(|| api_key))
}
