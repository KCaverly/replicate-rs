use anyhow::anyhow;
use futures_lite::io::AsyncReadExt;
use isahc::{prelude::*, Request};
use serde::Deserialize;
use serde_json::Value;

use crate::config::ReplicateConfig;

#[derive(Debug, Deserialize)]
struct ModelVersionError {
    detail: String,
}

/// Version details for a particular model
#[derive(Debug, Deserialize, Clone)]
pub struct ModelVersion {
    /// Id of the model
    pub id: String,
    /// Time in which the model was created
    pub created_at: String,
    /// Version of cog used to create the model
    pub cog_version: String,
    /// OpenAPI Schema of model input and outputs
    pub openapi_schema: serde_json::Value,
}

/// Paginated view of all versions for a particular model
#[derive(Debug, Deserialize)]
pub struct ModelVersions {
    /// Place in pagination
    pub next: Option<String>,
    /// Place in pagination
    pub previous: Option<String>,
    /// List of all versions available
    pub results: Vec<ModelVersion>,
}

/// All details available for a particular Model
#[derive(Deserialize, Debug)]
pub struct Model {
    /// URL for model homepage
    pub url: String,
    /// The owner of the model
    pub owner: String,
    /// The name of the model
    pub name: String,
    /// A brief description of the model
    pub description: String,
    /// Whether the model is public or private
    pub visibility: String,
    /// Github URL for the associated repo
    pub github_url: String,
    /// Url for an associated paper
    pub paper_url: Option<String>,
    /// Url for the model's license
    pub license_url: Option<String>,
    /// How many times the model has been run
    pub run_count: usize,
    /// Image URL to show on Replicate's Model page
    pub cover_image_url: String,
    /// A simple example to show model's use
    pub default_example: Value,
    /// The latest version's details
    pub latest_version: ModelVersion,
}

/// A client for interacting with `models` endpoints
pub struct ModelClient {
    client: ReplicateConfig,
}

impl ModelClient {
    /// Create a new `ModelClient` based upon a `ReplicateConfig` object
    pub fn from(client: ReplicateConfig) -> Self {
        ModelClient { client }
    }

    /// Retrieve details for a specific model
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

    /// Retrieve details for a specific model's version
    pub async fn get_specific_version(
        &self,
        owner: &str,
        name: &str,
        version_id: &str,
    ) -> anyhow::Result<Model> {
        let api_key = self.client.get_api_key()?;
        let base_url = self.client.get_base_url();
        let endpoint = format!("{base_url}/models/{owner}/{name}/versions/{version_id}");
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

    /// Retrieve details for latest version of a specific model
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

    /// Retrieve list of all available versions of a specific model
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

        let client = ReplicateConfig::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        model_client.get("replicate", "hello-world").await.unwrap();

        model_mock.assert();
    }

    #[tokio::test]
    async fn test_get_specific_version() {
        let mock_server = MockServer::start();

        let model_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/models/replicate/hello-world/versions/1234");
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
                    "id": "1234",
                    "created_at": "2022-04-26T19:29:04.418669Z",
                    "cog_version": "0.3.0",
                    "openapi_schema": {}
                }
            }));
        });

        let client = ReplicateConfig::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        model_client
            .get_specific_version("replicate", "hello-world", "1234")
            .await
            .unwrap();

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

        let client = ReplicateConfig::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        model_client
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

        let client = ReplicateConfig::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        model_client
            .get_latest_version("replicate", "hello-world")
            .await
            .unwrap();

        model_mock.assert();
    }
}
