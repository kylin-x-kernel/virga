use virga::client::{VirgeClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🧪 测试Yamux客户端修复...");

    let config = ClientConfig::default();
    let mut client = VirgeClient::new(config);

    println!("📡 连接到服务器...");
    match client.connect().await {
        Ok(()) => println!("✅ 连接成功"),
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            return Err(e.into());
        }
    }

    // 测试is_connected
    if client.is_connected() {
        println!("✅ 连接状态检查: 已连接");
    } else {
        println!("❌ 连接状态检查: 未连接");
        return Err("连接状态错误".into());
    }

    println!("🎉 Yamux客户端修复测试完成！");
    Ok(())
}