use eventsource_stream::Eventsource;
use futures_lite::StreamExt;
use replicate_rs::config::ReplicateConfig;
use replicate_rs::predictions::PredictionClient;
use serde_json::json;

#[tokio::main]
async fn main() {
    let config = ReplicateConfig::new().unwrap();
    let client = PredictionClient::from(config);

    let prompt = "this is a test";

    let mut prediction = client
        .create(
            "meta",
            "llama-2-70b-chat",
            json!({"prompt": prompt, "system_prompt": "You are a helpful assistant"}),
            true,
        )
        .await
        .unwrap();

    let mut stream = prediction.get_stream().await.unwrap();

    while let Some(event) = stream.next().await {
        println!("RECEIVED EVENT: {:?}", event);
    }
}
