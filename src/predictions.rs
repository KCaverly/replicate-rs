use std::io::Read;

use erased_serde::Serialize;
use isahc::{prelude::*, Request};
use serde_json::Value;

use crate::api_key;
use crate::models::Model;

#[derive(serde::Deserialize, Debug)]
pub struct Prediction {
    id: String,
    model: String,
    version: String,
    input: Value,
    status: String,
    created_at: String,
    urls: PredictionUrls,
}

#[derive(serde::Deserialize, Debug)]
struct PredictionUrls {
    cancel: String,
    get: String,
}

#[derive(serde::Serialize)]
struct PredictionInput {
    version: String,
    input: Box<dyn Serialize>,
}

impl Prediction {
    pub async fn create_from_model_details(
        owner: &str,
        name: &str,
        input: Box<dyn Serialize>,
    ) -> anyhow::Result<Prediction> {
        let api_key = api_key()?;
        let model = Model::get(owner, name).await?;

        let version = model.latest_version.id;

        let endpoint = "https://api.replicate.com/v1/predictions";

        let input = PredictionInput { version, input };

        let body = serde_json::to_string(&input)?;
        let mut response = Request::post(endpoint)
            .header("Authorization", format!("Token {api_key}"))
            .body(body)
            .unwrap()
            .send()
            .unwrap();

        let mut data = String::new();
        response.body_mut().read_to_string(&mut data)?;

        let prediction: Prediction = serde_json::from_str(data.as_str())?;

        anyhow::Ok(prediction)
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[tokio::test]
    async fn test_create_prediction() {
        #[derive(Serialize)]
        struct EmbeddingsInput {
            texts: String,
            batch_size: usize,
            normalize_embeddings: bool,
            convert_to_numpy: bool,
        }

        let input = Box::new(EmbeddingsInput {
            texts: r#"["In the water, fish are swimming.", "Fish swim in the water."]"#.to_string(),
            batch_size: 32,
            normalize_embeddings: true,
            convert_to_numpy: false,
        });

        Prediction::create_from_model_details("nateraw", "bge-large-en-v1.5", input)
            .await
            .unwrap();
    }
}
