use reqwest::Client;

const URL: &str = "https://google.com";

#[tokio::main]
async fn main() {
    let client = Client::new();

    tokio::spawn(async move { client.get(URL).send().await.unwrap() })
        .await
        .unwrap();

    // `client` is no longer a valid reference at this point
    // it was permanently moved into the `spawn`'d closure
}
