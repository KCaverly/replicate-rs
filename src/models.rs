use anyhow::anyhow;
use futures_lite::io::AsyncReadExt;
use isahc::{prelude::*, Request};
use serde::Deserialize;
use serde_json::Value;
use std::io::Read;

use crate::client::ReplicateClient;

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

pub struct ModelClient {
    client: ReplicateClient,
}

impl ModelClient {
    pub fn from(client: ReplicateClient) -> Self {
        ModelClient { client }
    }

    pub async fn get(&self, owner: &str, name: &str) -> anyhow::Result<Model> {
        let api_key = self.client.get_api_key()?;
        let base_url = self.client.get_base_url();
        let endpoint = format!("{base_url}/models/{owner}/{name}");
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

    pub async fn get_latest_version(
        &self,
        owner: &str,
        name: &str,
    ) -> anyhow::Result<ModelVersion> {
        let all_versions = self.list_versions(owner, name).await?;
        let latest_version = all_versions
            .results
            .get(0)
            .ok_or(anyhow!("no versions found for {owner}/{name}"))?;
        anyhow::Ok(latest_version.clone())
    }

    pub async fn list_versions(&self, owner: &str, name: &str) -> anyhow::Result<ModelVersions> {
        let base_url = self.client.get_base_url();
        let api_key = self.client.get_api_key()?;
        let endpoint = format!("{base_url}/models/{owner}/{name}/versions");
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
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_model() {
        let mock_server = MockServer::start();

        let model_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/models/replicate/hello-world");
            then.status(200).json_body_obj(&json!({
                "url": "https://replicate.com/replicate/hello-world",
                "owner": "replicate",
                "name": "hello-world",
                "description": "A tiny model that says hello",
                "visibility": "public",
                "github_url": "https://github.com/replicate/cog-examples",
                "paper_url": null,
                "license_url": null,
                "run_count": 5681081,
                "cover_image_url": "...",
                "default_example": null,
                "latest_version": {
                    "id": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "created_at": "2022-04-26T19:29:04.418669Z",
                    "cog_version": "0.3.0",
                    "openapi_schema": {}
                }
            }));
        });

        let client = ReplicateClient::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        let model = model_client.get("replicate", "hello-world").await.unwrap();

        model_mock.assert();
    }

    #[tokio::test]
    async fn test_list_model_versions() {
        let mock_server = MockServer::start();

        // Model endpoints
        let model_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/models/replicate/hello-world/versions");

            then.status(200).json_body_obj(&json!({
                "next": null,
                "previous": null,
                "results": [{
                    "id": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "created_at": "2022-04-26T19:29:04.418669Z",
                    "cog_version": "0.3.0",
                    "openapi_schema": null
                }]
            }));
        });

        let client = ReplicateClient::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        let model = model_client
            .list_versions("replicate", "hello-world")
            .await
            .unwrap();

        model_mock.assert();
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let mock_server = MockServer::start();

        // Model endpoints
        let model_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/models/replicate/hello-world/versions");

            then.status(200).json_body_obj(&json!({
                "next": null,
                "previous": null,
                "results": [{
                    "id": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "created_at": "2022-04-26T19:29:04.418669Z",
                    "cog_version": "0.3.0",
                    "openapi_schema": null
                }]
            }));
        });

        let client = ReplicateClient::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        let model = model_client
            .get_latest_version("replicate", "hello-world")
            .await
            .unwrap();

        model_mock.assert();
    }
}
