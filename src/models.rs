use anyhow::anyhow;
use futures_lite::io::AsyncReadExt;
use isahc::{prelude::*, Request};
use serde::Deserialize;
use serde_json::Value;
use std::io::Read;

use crate::api_key;

#[derive(Debug, Deserialize)]
pub struct ModelVersionError {
    detail: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelVersion {
    pub(crate) id: String,
    created_at: String,
    cog_version: String,
    openapi_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ModelVersions {
    next: Option<String>,
    previous: Option<String>,
    results: Vec<ModelVersion>,
}

#[derive(Deserialize, Debug)]
pub struct Model {
    url: String,
    owner: String,
    name: String,
    description: String,
    visibility: String,
    github_url: String,
    paper_url: Option<String>,
    license_url: Option<String>,
    run_count: usize,
    cover_image_url: String,
    default_example: Value,
    pub(crate) latest_version: ModelVersion,
}

impl Model {
    pub async fn get(owner: &str, name: &str) -> anyhow::Result<Self> {
        let api_key = api_key()?;
        let endpoint = format!("https://api.replicate.com/v1/models/{owner}/{name}");
        let mut response = Request::get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let model: Model = serde_json::from_str(data.as_str())?;

        anyhow::Ok(model)
    }

    pub async fn get_latest_version(owner: &str, name: &str) -> anyhow::Result<ModelVersion> {
        let all_versions = Self::list_versions(owner, name).await?;

        let latest_version = all_versions
            .results
            .get(0)
            .ok_or(anyhow!("no versions found for {owner}/{name}"))?;

        anyhow::Ok(latest_version.clone())
    }

    pub async fn list_versions(owner: &str, name: &str) -> anyhow::Result<ModelVersions> {
        let api_key = api_key()?;
        let endpoint = format!("https://api.replicate.com/v1/models/{owner}/{name}/versions");
        let mut response = Request::get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        if response.status().is_success() {
            let data: ModelVersions = serde_json::from_str(data.as_str())?;
            anyhow::Ok(data)
        } else {
            let data: ModelVersionError = serde_json::from_str(data.as_str())?;
            Err(anyhow!(data.detail))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_model() {
        Model::get("nateraw", "bge-large-en-v1.5").await.unwrap();
    }

    #[tokio::test]
    async fn test_list_model_versions() {
        Model::list_versions("nateraw", "bge-large-en-v1.5")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        Model::get_latest_version("nateraw", "bge-large-en-v1.5")
            .await
            .unwrap();
    }
}
