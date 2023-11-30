//! Utilities for interacting with all prediction endpoints.
//!
//! This includes the following:
//! - [Create Prediction](https://replicate.com/docs/reference/http#predictions.create)
//! - [Get Prediction](https://replicate.com/docs/reference/http#predictions.get)
//! - [List Predictions](https://replicate.com/docs/reference/http#predictions.list)
//!

use crate::config::ReplicateConfig;

use erased_serde::Serialize;
use futures_lite::io::AsyncReadExt;
use isahc::{prelude::*, Request};
use serde_json::Value;

use crate::models::ModelClient;
use crate::{api_key, base_url};

/// Status of a retrieved or created prediction
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PredictionStatus {
    /// The prediction is starting up. If this status lasts longer than a few seconds, then it's
    /// typically because a new worker is being started to run the prediction.
    Starting,
    /// The `predict()` method of the model is currently running.
    Processing,
    /// The prediction completed successfully.
    Succeeded,
    /// The prediction was canceled by its creator.
    Failed,
    /// The prediction was canceled by its creator.
    Canceled,
}

/// Provided urls to either cancel or retrieve updated details for the specific prediction.
#[derive(serde::Deserialize, Debug)]
pub struct PredictionUrls {
    /// Url endpoint to cancel the specific prediction
    pub cancel: String,
    /// Url endpoint to retrieve the specific prediction
    pub get: String,
}

/// Details for a specific prediction
#[derive(serde::Deserialize, Debug)]
pub struct Prediction {
    /// Id of the prediction
    pub id: String,
    /// Model used during the prediction
    pub model: String,
    /// Specific version used during prediction
    pub version: String,
    /// The inputs provided for the specific prediction
    pub input: Value,
    /// The current status of the prediction
    pub status: PredictionStatus,
    /// The created time for the prediction
    pub created_at: String,
    /// Urls to either retrieve or cancel details for this prediction
    pub urls: PredictionUrls,
}

/// Paginated list of available predictions
#[derive(serde::Deserialize, Debug)]
pub struct Predictions {
    // Identify for status in pagination
    next: Option<String>,
    // Identify for status of pagination
    previous: Option<String>,
    // List of predictions
    results: Vec<Prediction>,
}

impl Prediction {
    /// Leverage the get url provided, to refresh struct attributes
    pub async fn reload(&mut self) -> anyhow::Result<()> {
        let api_key = api_key()?;
        let endpoint = self.urls.get.clone();
        let mut response = Request::get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let prediction: Prediction = serde_json::from_str(data.as_str())?;
        *self = prediction;
        anyhow::Ok(())
    }
}

/// A client namespace for interacting with 'predictions' endpoint
#[derive(Debug)]
pub struct PredictionClient {
    config: ReplicateConfig,
}

#[derive(serde::Serialize)]
struct PredictionInput {
    version: String,
    input: Box<dyn Serialize>,
}

impl PredictionClient {
    /// Create a new `PredictionClient` based upon a `ReplicateConfig` object
    pub fn from(config: ReplicateConfig) -> Self {
        PredictionClient { config }
    }
    /// Create a new prediction
    pub async fn create(
        &self,
        owner: &str,
        name: &str,
        input: Box<dyn Serialize>,
    ) -> anyhow::Result<Prediction> {
        let api_key = api_key()?;
        let base_url = base_url();

        let model_client = ModelClient::from(self.config.clone());
        let version = model_client.get_latest_version(owner, name).await?.id;

        let endpoint = format!("{base_url}/predictions");
        let input = PredictionInput { version, input };
        let body = serde_json::to_string(&input)?;
        let mut response = Request::post(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body(body)?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let prediction: Prediction = serde_json::from_str(data.as_str())?;

        anyhow::Ok(prediction)
    }

    /// Get details for an existing prediction
    pub async fn get(&self, id: String) -> anyhow::Result<Prediction> {
        let api_key = self.config.get_api_key()?;
        let base_url = self.config.get_base_url();

        let endpoint = format!("{base_url}/predictions/{id}");
        let mut response = Request::get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let prediction: Prediction = serde_json::from_str(data.as_str())?;

        anyhow::Ok(prediction)
    }

    /// List all existing predictions for the current user
    pub async fn list(&self) -> anyhow::Result<Predictions> {
        let api_key = self.config.get_api_key()?;
        let base_url = self.config.get_base_url();

        let endpoint = format!("{base_url}/predictions");
        let mut response = Request::get(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let predictions: Predictions = serde_json::from_str(data.as_str())?;

        anyhow::Ok(predictions)
    }

    /// Cancel an existing prediction
    pub async fn cancel(&self, id: String) -> anyhow::Result<Prediction> {
        let api_key = self.config.get_api_key()?;
        let base_url = self.config.get_base_url();
        let endpoint = format!("{base_url}/predictions/{id}/cancel");
        let mut response = Request::post(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body({})?
            .send_async()
            .await?;

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data).await?;

        let prediction: Prediction = serde_json::from_str(data.as_str())?;
        anyhow::Ok(prediction)
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_get() {
        let server = MockServer::start();

        let prediction_mock = server.mock(|when, then| {
            when.method(GET).path("/predictions/1234");
            then.status(200).json_body_obj(&json!(
                {
                    "id": "1234",
                    "model": "replicate/hello-world",
                    "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "input": {
                        "text": "Alice"
                    },
                    "logs": "",
                    "error": null,
                    "status": "starting",
                    "created_at": "2023-09-08T16:19:34.765994657Z",
                    "urls": {
                        "cancel": "https://api.replicate.com/v1/predictions/1234/cancel",
                        "get": "https://api.replicate.com/v1/predictions/1234"
                    }
                }
            ));
        });

        let client = ReplicateConfig::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        prediction_client.get("1234".to_string()).await.unwrap();

        prediction_mock.assert();
    }

    #[tokio::test]
    async fn test_create() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(POST).path("/predictions");
            then.status(200).json_body_obj(&json!(
                {
                    "id": "gm3qorzdhgbfurvjtvhg6dckhu",
                    "model": "replicate/hello-world",
                    "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "input": {
                        "text": "Alice"
                    },
                    "logs": "",
                    "error": null,
                    "status": "starting",
                    "created_at": "2023-09-08T16:19:34.765994657Z",
                    "urls": {
                        "cancel": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu/cancel",
                        "get": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu"
                    }
                }
            ));
        });

        server.mock(|when, then| {
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

        let client = ReplicateConfig::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        prediction_client
            .create(
                "replicate",
                "hello-world",
                Box::new(json!({"text": "This is test input"})),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_predictions() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/predictions");
            then.status(200).json_body_obj(&json!(
                { "next": null,
                  "previous": null,
                  "results": [
                    {
                        "id": "gm3qorzdhgbfurvjtvhg6dckhu",
                        "model": "replicate/hello-world",
                        "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                        "input": {
                            "text": "Alice"
                        },
                        "logs": "",
                        "error": null,
                        "status": "starting",
                        "created_at": "2023-09-08T16:19:34.765994657Z",
                        "urls": {
                            "cancel": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu/cancel",
                            "get": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu"
                        }
                    },
                    {
                        "id": "gm3qorzdhgbfurvjtvhg6dckhu",
                        "model": "replicate/hello-world",
                        "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                        "input": {
                            "text": "Alice"
                        },
                        "logs": "",
                        "error": null,
                        "status": "starting",
                        "created_at": "2023-09-08T16:19:34.765994657Z",
                        "urls": {
                            "cancel": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu/cancel",
                            "get": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu"
                        }
                    }
                ]}
            ));
        });

        let client = ReplicateConfig::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        prediction_client.list().await.unwrap();
    }

    #[tokio::test]
    async fn test_create_and_reload() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(POST).path("/predictions");
            then.status(200).json_body_obj(&json!(
                {
                    "id": "gm3qorzdhgbfurvjtvhg6dckhu",
                    "model": "replicate/hello-world",
                    "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "input": {
                        "text": "Alice"
                    },
                    "logs": "",
                    "error": null,
                    "status": "starting",
                    "created_at": "2023-09-08T16:19:34.765994657Z",
                    "urls": {
                        "cancel": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu/cancel",
                        "get": "https://api.replicate.com/v1/predictions/gm3qorzdhgbfurvjtvhg6dckhu"
                    }
                }
            ));
        });

        server.mock(|when, then| {
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

        let client = ReplicateConfig::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        let mut prediction = prediction_client
            .create(
                "replicate",
                "hello-world",
                Box::new(json!({"text": "This is test input"})),
            )
            .await
            .unwrap();

        prediction.reload().await.unwrap();
    }

    #[tokio::test]
    async fn test_cancel() {
        let server = MockServer::start();

        let prediction_mock = server.mock(|when, then| {
            when.method(POST).path("/predictions/1234/cancel");
            then.status(200).json_body_obj(&json!(
                {
                    "id": "1234",
                    "model": "replicate/hello-world",
                    "version": "5c7d5dc6dd8bf75c1acaa8565735e7986bc5b66206b55cca93cb72c9bf15ccaa",
                    "input": {
                        "text": "Alice"
                    },
                    "logs": "",
                    "error": null,
                    "status": "starting",
                    "created_at": "2023-09-08T16:19:34.765994657Z",
                    "urls": {
                        "cancel": "https://api.replicate.com/v1/predictions/1234/cancel",
                        "get": "https://api.replicate.com/v1/predictions/1234"
                    }
                }
            ));
        });

        let config = ReplicateConfig::test(server.base_url()).unwrap();
        let prediction_client = PredictionClient::from(config);

        prediction_client.cancel("1234".to_string()).await.unwrap();

        prediction_mock.assert();
    }
}
