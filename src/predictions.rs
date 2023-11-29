//! Utilities for interacting with all prediction endpoints.
//!
//! This includes the following:
//! - [Create Prediction](https://replicate.com/docs/reference/http#predictions.create)
//! - [Get Prediction](https://replicate.com/docs/reference/http#predictions.get)
//!
//! # Example
//! ```rust
//! use replicate_rs::client::ReplicateClient;
//! use replicate_rs::predictions::PredictionClient;
//! ```

use crate::client::ReplicateClient;

use erased_serde::Serialize;
use futures_lite::io::AsyncReadExt;
use isahc::{prelude::*, Request};
use serde_json::Value;

use crate::models::ModelClient;
use crate::{api_key, base_url};

#[derive(Debug)]
pub struct PredictionClient {
    client: ReplicateClient,
}

#[derive(serde::Deserialize, Debug)]
pub struct Prediction {
    pub id: String,
    pub model: String,
    pub version: String,
    pub input: Value,
    pub status: PredictionStatus,
    pub created_at: String,
    pub urls: PredictionUrls,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PredictionStatus {
    Starting,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(serde::Deserialize, Debug)]
pub struct PredictionUrls {
    pub cancel: String,
    pub get: String,
}

#[derive(serde::Serialize)]
struct PredictionInput {
    version: String,
    input: Box<dyn Serialize>,
}

impl PredictionClient {
    pub fn from(client: ReplicateClient) -> Self {
        PredictionClient { client }
    }
    pub async fn create(
        &self,
        owner: &str,
        name: &str,
        input: Box<dyn Serialize>,
    ) -> anyhow::Result<Prediction> {
        let api_key = api_key()?;
        let base_url = base_url();

        let model_client = ModelClient::from(self.client.clone());
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

        dbg!(&data);

        let prediction: Prediction = serde_json::from_str(data.as_str())?;

        anyhow::Ok(prediction)
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serde::Serialize;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_create() {
        let server = MockServer::start();

        let prediction_mock = server.mock(|when, then| {
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

        let model_mock = server.mock(|when, then| {
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

        let mock_url = server.base_url();
        let client = ReplicateClient::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        let prediction = prediction_client
            .create(
                "replicate",
                "hello-world",
                Box::new(json!({"text": "This is test input"})),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_reload() {
        let server = MockServer::start();

        let prediction_mock = server.mock(|when, then| {
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

        let model_mock = server.mock(|when, then| {
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

        let mock_url = server.base_url();
        let client = ReplicateClient::test(server.base_url()).unwrap();

        let prediction_client = PredictionClient::from(client);
        let prediction = prediction_client
            .create(
                "replicate",
                "hello-world",
                Box::new(json!({"text": "This is test input"})),
            )
            .await
            .unwrap();
    }
}
