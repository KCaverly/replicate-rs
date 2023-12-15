use replicate_rs::config::ReplicateConfig;
use replicate_rs::models::ModelClient;

#[tokio::main]
async fn main() {
    let config = ReplicateConfig::new().unwrap();
    let client = ModelClient::from(config);

    let deleted = client
        .delete_version(
            "kcaverly",
            "dolphin-2.5-mixtral-8x7b-gguf",
            "5813632838e1bd7ba82e0a005d33693fa3ceb67fd51f81f54b4d249ef9ddf1cc",
        )
        .await;

    println!("DELETED: {:?}", deleted);
}
