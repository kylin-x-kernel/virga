use virga::client::{VirgeClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let config = ClientConfig::new(2, 1234, 1024, false);
    let mut client = VirgeClient::new(config);
    client.connect().await?;
    
    client.send(vec![1; 512]).await?;
    let data = client.recv().await?;
    println!("{}", data.len());
    
    client.disconnect().await?;
    Ok(())
}
