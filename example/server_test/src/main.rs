use std::io::{Read, Write};
use std::time::Instant;
use virga::{VirgeServer, server::{ServerConfig, ServerManager}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = ServerConfig::new(0xFFFFFFFF, 1234, 1024, false);

    let mut manager = ServerManager::new(config);
    manager.start()?;

    if let Ok(mut server) = manager.accept() {
        println!("there is a new virgeserver");

        // test_1(&mut server)?;
        // test_2(&mut server)?;
        // test_3(&mut server)?;
        test_4(&mut server)?;
        
        // 断开连接
        // server.disconnect()?;
    }

    Ok(())
}

fn test_3(server: &mut VirgeServer) -> Result<(), Box<dyn std::error::Error>> {
    let recvdata = server.recv()?;
    println!("recvdata len = {}", recvdata.len());
    
    let sendlen = server.send(recvdata)?;
    println!("sendlen = {}", sendlen);

    Ok(())
}

fn test_1(server: &mut VirgeServer) -> Result<(), Box<dyn std::error::Error>> {
    // 处理接收数据, 先接收数据长度，然后创建一个足够长的databuf，最后接收数据
    let mut buf: [u8; 8] = [0u8; 8];
    server.read(&mut buf)?;
    let data_len = usize::from_be_bytes(buf);
    println!("data_len: {data_len}");
    
    let mut data = vec![0; data_len];
    let actual_data_len_ = server.read(&mut data)?;
    assert_eq!(data_len, actual_data_len_);
    
    // 处理发送数据, 先发送数据长度，然后发送数据
    server.write_all(&data.len().to_be_bytes())?;
    server.write_all(&data)?;
    Ok(())
}

fn test_2(server: &mut VirgeServer) -> Result<(), Box<dyn std::error::Error>> {
    // 处理接收数据, databuf不足够长，循环接收
    let mut buf = [0u8; 8];
    let mut count = 0;
    let mut total_len = 0;
    loop {
        let len = server.read(&mut buf)?; 
        count+=1;
        total_len+=len;
        println!("No.{}, len = {}", count, len);
        
        if server.no_has_data(){
            break;
        }
    }
    assert_eq!(total_len, 512);
    
    // 处理发送数据, 
    let data = vec![1; 512];
    server.write_all(&data)?;
    Ok(())
}

fn test_4(server: &mut VirgeServer) -> Result<(), Box<dyn std::error::Error>> {
    // 性能测试：接收和发送大量数据（回显服务器）
    const ITERATIONS: usize = 10;

    println!("\n=== 服务端性能测试开始 ===");
    println!("测试次数: {}", ITERATIONS);
    
    let mut total_recv_time = 0u128;
    let mut total_send_time = 0u128;
    let mut total_bytes_received = 0usize;
    let mut total_bytes_sent = 0usize;

    for i in 1..=ITERATIONS {
        // 接收数据
        let start = Instant::now();
        let data = server.recv()?;
        let recv_duration = start.elapsed();
        total_recv_time += recv_duration.as_millis();
        total_bytes_received += data.len();

        println!("第 {} 次 - 接收了 {} bytes ({:.2} KB) in {:.2} ms",
                 i, data.len(), data.len() as f64 / 1024.0,
                 recv_duration.as_secs_f64() * 1000.0);
        
        // 回显数据
        let start = Instant::now();
        let sent = server.send(data)?;
        let send_duration = start.elapsed();
        total_send_time += send_duration.as_millis();
        total_bytes_sent += sent;

        println!("第 {} 次 - 发送了 {} bytes ({:.2} KB) in {:.2} ms",
                 i, sent, sent as f64 / 1024.0,
                 send_duration.as_secs_f64() * 1000.0);
    }

    // 计算统计数据
    let avg_recv_time = total_recv_time as f64 / ITERATIONS as f64;
    let avg_send_time = total_send_time as f64 / ITERATIONS as f64;
    let total_data_kb = (total_bytes_received + total_bytes_sent) as f64 / 1024.0;
    let total_time_sec = (total_recv_time + total_send_time) as f64 / 1000.0;
    let throughput = total_data_kb / total_time_sec;

    println!("\n=== 服务端性能测试结果 ===");
    println!("平均接收时间: {:.2} ms", avg_recv_time);
    println!("平均发送时间: {:.2} ms", avg_send_time);
    println!("总数据量: {:.2} KB", total_data_kb);
    println!("总耗时: {:.2} 秒", total_time_sec);
    println!("平均吞吐量: {:.2} KB/s", throughput);

    // 计算接收和发送速度
    let recv_speed = (total_bytes_received as f64 / 1024.0) / (total_recv_time as f64 / 1000.0);
    let send_speed = (total_bytes_sent as f64 / 1024.0) / (total_send_time as f64 / 1000.0);
    println!("接收速度: {:.2} KB/s", recv_speed);
    println!("发送速度: {:.2} KB/s", send_speed);

    Ok(())
}