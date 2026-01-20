use virga::server::{ServerManager, VirgeServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = ServerConfig::default();

    println!("=== ServerManager + VirgeServer 示例 ===\n");

    // 示例1：简单回显服务器
    println!("1. 简单回显服务器示例：");
    println!("   let manager = ServerManager::new(config);");
    println!("   manager.run_simple().await?; // 自动回显所有接收的数据\n");

    // 示例2：自定义连接处理器
    println!("2. 自定义连接处理器示例：");
    println!("   manager.run(|mut server: VirgeServer| async move {");
    println!("       let data = server.recv().await?; // 接收数据");
    println!("       server.send(processed_data).await?; // 发送响应");
    println!("       server.disconnect().await?; // 断开连接");
    println!("   }).await?;\n");

    // 示例3：手动管理连接
    println!("3. 手动管理连接示例：");
    println!("   let mut manager = ServerManager::new(config);");
    println!("   manager.start().await?; // 启动监听");
    println!("   ");
    println!("   while let Ok(mut server) = manager.accept().await {");
    println!("       tokio::spawn(async move {");
    println!("           // VirgeServer 专注于数据收发，与 VirgeClient 类似");
    println!("           let data = server.recv().await?;");
    println!("           server.send(data).await?;");
    println!("           server.disconnect().await?;");
    println!("       });");
    println!("   }\n");

    println!("✅ ServerManager 负责连接管理");
    println!("✅ VirgeServer 负责数据传输，与 VirgeClient 结构相似");
    println!("✅ 架构清晰，职责分离");

    Ok(())
}