fn main() {
    let client = HttpClient::new();
    
    tokio::spawn(async move {
        client.get(...)
    })
}
