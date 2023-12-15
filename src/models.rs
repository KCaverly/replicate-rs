//! Utilities for interacting with models endpoints.
//!
//! This includes the following:
//! - [Get a Model](https://replicate.com/docs/reference/http#models.get)
//! - [Get a Model Version](https://replicate.com/docs/reference/http#models.versions.get)
//! - [List a Model's Versions](https://replicate.com/docs/reference/http#models.versions.list)
//! - [List all Public Models](https://replicate.com/docs/reference/http#models.list)
//!
use anyhow::anyhow;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;

use crate::config::ReplicateConfig;
use crate::errors::{get_error, ReplicateError, ReplicateResult};

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

/// Paginated view of all available models
#[derive(Debug, Deserialize)]
pub struct Models {
    /// Place in pagination
    pub next: Option<String>,
    /// Place in pagination
    pub previous: Option<String>,
    /// List of all versions available
    pub results: Vec<Model>,
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
        let client = reqwest::Client::new();
        let response = client
            .get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .send()
            .await?;

        let data = response.text().await?;
        let model: Model = serde_json::from_str(&data)?;
        anyhow::Ok(model)
    }

    /// Retrieve details for a specific model's version
    pub async fn get_specific_version(
        &self,
        owner: &str,
        name: &str,
        version_id: &str,
    ) -> ReplicateResult<Model> {
        let api_key = self.client.get_api_key()?;
        let base_url = self.client.get_base_url();
        let endpoint = format!("{base_url}/models/{owner}/{name}/versions/{version_id}");
        let client = reqwest::Client::new();
        let response = client
            .get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .send()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;

        let data = response
            .text()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;
        let model: Model = serde_json::from_str(&data)
            .map_err(|err| ReplicateError::SerializationError(err.to_string()))?;
        Ok(model)
    }

    /// Delete specific model version
    pub async fn delete_version(
        &self,
        owner: &str,
        name: &str,
        version_id: &str,
    ) -> ReplicateResult<()> {
        let api_key = self.client.get_api_key()?;
        let base_url = self.client.get_base_url();
        let endpoint = format!("{base_url}/models/{owner}/{name}/versions/{version_id}");
        let client = reqwest::Client::new();
        let response = client
            .delete(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .send()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ReplicateError::Misc("delete request failed".to_string()))
        }
    }

    /// Retrieve details for latest version of a specific model
    pub async fn get_latest_version(
        &self,
        owner: &str,
        name: &str,
    ) -> ReplicateResult<ModelVersion> {
        let all_versions = self.list_versions(owner, name).await?;
        let latest_version = all_versions.results.get(0).ok_or(ReplicateError::Misc(
            "no versions found for {owner}/{name}".to_string(),
        ))?;
        Ok(latest_version.clone())
    }

    /// Retrieve list of all available versions of a specific model
    pub async fn list_versions(&self, owner: &str, name: &str) -> ReplicateResult<ModelVersions> {
        let base_url = self.client.get_base_url();
        let api_key = self.client.get_api_key()?;
        let endpoint = format!("{base_url}/models/{owner}/{name}/versions");
        let client = reqwest::Client::new();
        let response = client
            .get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .send()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;

        let status = response.status();
        let data = response
            .text()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;

        return match status.clone() {
            reqwest::StatusCode::OK => {
                let data: ModelVersions = serde_json::from_str(&data)
                    .map_err(|err| ReplicateError::SerializationError(err.to_string()))?;
                Ok(data)
            }
            _ => Err(get_error(status, data.as_str())),
        };
    }

    /// Retrieve all publically and private available models
    pub async fn get_models(&self) -> ReplicateResult<Models> {
        let base_url = self.client.get_base_url();
        let api_key = self.client.get_api_key()?;
        let endpoint = format!("{base_url}/models");
        let client = reqwest::Client::new();
        let response = client
            .get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .send()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;

        let data = response
            .text()
            .await
            .map_err(|err| ReplicateError::ClientError(err.to_string()))?;
        let models: Models = serde_json::from_str(&data)
            .map_err(|err| ReplicateError::SerializationError(err.to_string()))?;
        Ok(models)
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

    #[tokio::test]
    async fn test_get_models() {
        let mock_server = MockServer::start();

        // Model endpoints
        let model_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/models");
            then.status(200).json_body_obj(&json!({
                    "next": "some pagination string or null",
                    "previous": "some pagination string or null",
            "results": [
                {
                "url": "https://modelhomepage.example.com",
                "owner": "jdoe",
                "name": "super-cool-model",
                "description": "A model that predicts something very cool.",
                "visibility": "public",
                "github_url": "https://github.com/jdoe/super-cool-model",
                "paper_url": "https://research.example.com/super-cool-model-paper.pdf",
                "license_url": null,
                "run_count": 420,
                "cover_image_url": "https://cdn.example.com/images/super-cool-model-cover.jpg",
                "default_example": {
                    "input": "Example input data for the model."
                },
                "latest_version": {
                    "id": "v1.0.0",
                    "created_at": "2022-01-01T12:00:00Z",
                    "cog_version": "0.2",
                    "openapi_schema": null
                }
                },
                {
                "url": "https://anothermodelhomepage.example.com",
                "owner": "asmith",
                "name": "another-awesome-model",
                "description": "This model does awesome things with data.",
                "visibility": "private",
                "github_url": "https://github.com/asmith/another-awesome-model",
                "paper_url": null,
                "license_url": "https://licenses.example.com/another-awesome-model-license.txt",
                "run_count": 150,
                "cover_image_url": "https://cdn.example.com/images/another-awesome-model-cover.jpg",
                "default_example": {
                    "input": "Some example input for this awesome model."
                },
                "latest_version": {
                    "id": "v1.2.3",
                    "created_at": "2023-02-15T08:30:00Z",
                    "cog_version": "0.2",
                    "openapi_schema": null
                }
            }
        ]}));
        });

        let client = ReplicateConfig::test(mock_server.base_url()).unwrap();
        let model_client = ModelClient::from(client);
        model_client.get_models().await.unwrap();

        model_mock.assert();
    }
}
