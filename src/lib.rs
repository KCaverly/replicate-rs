mod models;

use std::sync::OnceLock;
use std::env::var;
fn api_key() -> anyhow::Result<&'static str> {
    let api_key = var("REPLICATE_API_KEY")?;
    static REPLICATE_API_KEY: OnceLock<String> = OnceLock::new();
    anyhow::Ok(REPLICATE_API_KEY.get_or_init(|| {
        api_key
    }))
}
