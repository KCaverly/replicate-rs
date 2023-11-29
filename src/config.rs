use crate::{api_key, base_url};
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct ReplicateConfig {
    api_key: Option<&'static str>,
    base_url: String,
}

impl Default for ReplicateConfig {
    fn default() -> Self {
        ReplicateConfig {
            api_key: None,
            base_url: base_url().to_string(),
        }
    }
}

impl ReplicateConfig {
    pub fn new() -> anyhow::Result<Self> {
        let api_key = api_key()?;
        let base_url = base_url().to_string();
        anyhow::Ok(ReplicateConfig {
            api_key: Some(api_key),
            base_url,
        })
    }

    #[cfg(test)]
    pub fn test(base_url: String) -> anyhow::Result<Self> {
        anyhow::Ok(ReplicateConfig {
            api_key: Some("test-api-key"),
            base_url,
        })
    }

    pub(crate) fn get_api_key(&self) -> anyhow::Result<&'static str> {
        self.api_key.ok_or(anyhow!(
            "REPLICATE_API_KEY not provided in environment variable"
        ))
    }

    pub(crate) fn get_base_url(&self) -> String {
        self.base_url.clone()
    }
}
