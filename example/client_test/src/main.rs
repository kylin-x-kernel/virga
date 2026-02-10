use std::io::{Read, Write};
use std::time::Instant;

use virga::client::{VirgeClient, ClientConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let config = ClientConfig::new(103, 1234, 1024, false);
    let mut client = VirgeClient::new(config);
    client.connect()?;

    // test_1(&mut client)?;
    // test_2(&mut client)?;
    // test_3(&mut client)?;
    test_4(&mut client)?;
    
    
    // 断开连接
    client.disconnect()?;

    Ok(())
}

fn test_3(client: &mut VirgeClient) -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![1; 512];
    let sendlen = client.send(data)?;

    let recvdata = client.recv()?;
    assert_eq!(sendlen, recvdata.len());
    assert_eq!(recvdata, vec![1; 512]);

    Ok(())
}

fn test_1(client: &mut VirgeClient) -> Result<(), Box<dyn std::error::Error>> {
    // 处理发送数据, 先发送数据长度，然后发送数据
    let data = vec![1; 512];
    client.write(&data.len().to_be_bytes())?;
    client.write(&data)?;


    // 处理接收数据, 先接收数据长度，然后创建一个足够长的databuf，最后接收数据
    let mut buf = [0u8; 8];
    client.read_exact(&mut buf)?;
    let data_len = usize::from_be_bytes(buf);

    let mut data = vec![0; data_len];
    let actual_data_len_ = client.read(&mut data)?;
    assert_eq!(data_len, actual_data_len_);
    println!("len date = {}", actual_data_len_);
    Ok(())
}


fn test_2(client: &mut VirgeClient) -> Result<(), Box<dyn std::error::Error>> {
    // 处理发送数据
    let data = vec![1; 512];
    client.write(&data)?;

    // 处理接收数据, databuf不足够长，循环接收
    let mut buf = [0u8; 8];
    let mut count = 0;
    let mut total_len = 0;
    loop {
        let len = client.read(&mut buf)?; 
        count+=1;
        total_len+=len;
        println!("No.{}, len = {}", count, len);
        
        if client.no_has_data(){
            break;
        }
    }
    assert_eq!(total_len, 512);

    Ok(())
}

fn test_4(client: &mut VirgeClient) -> Result<(), Box<dyn std::error::Error>> {
    // 性能测试：发送和接收大量数据
    const DATA_SIZE: usize = 500 * 1024; // 1 MB
    const ITERATIONS: usize = 10;

    println!("\n=== 性能测试开始 ===");
    println!("数据大小: {} KB", DATA_SIZE / 1024);
    println!("测试次数: {}", ITERATIONS);
    
    let test_data = vec![0xAB; DATA_SIZE];
    
    let mut total_send_time = 0u128;
    let mut total_recv_time = 0u128;
    let mut total_bytes_sent = 0usize;
    let mut total_bytes_received = 0usize;

    for i in 1..=ITERATIONS {
        // 发送测试
        let start = Instant::now();
        let sent = client.send(test_data.clone())?;
        let send_duration = start.elapsed();
        total_send_time += send_duration.as_millis();
        total_bytes_sent += sent;

        println!("第 {} 次 - 发送了 {} bytes ({:.2} KB) in {:.2} ms",
                 i, sent, sent as f64 / 1024.0,
                   send_duration.as_secs_f64() * 1000.0);
        
        // 接收测试
        let start = Instant::now();
        let received = client.recv()?;
        let recv_duration = start.elapsed();
        total_recv_time += recv_duration.as_millis();
        total_bytes_received += received.len();
        
        // 验证数据正确性
        assert_eq!(sent, received.len(), "第 {} 次测试：发送和接收的数据大小不一致", i);

        println!("第 {} 次 - 接收: {:.2} ms, 发送: {:.2} ms",
                 i, recv_duration.as_secs_f64() * 1000.0,
                 send_duration.as_secs_f64() * 1000.0);
    }

    // 计算统计数据
    let avg_send_time = total_send_time as f64 / ITERATIONS as f64;
    let avg_recv_time = total_recv_time as f64 / ITERATIONS as f64;
    let total_data_kb = (total_bytes_sent + total_bytes_received) as f64 / 1024.0;
    let total_time_sec = (total_send_time + total_recv_time) as f64 / 1000.0;
    let throughput = total_data_kb / total_time_sec;

    println!("\n=== 性能测试结果 ===");
    println!("平均发送时间: {:.2} ms", avg_send_time);
    println!("平均接收时间: {:.2} ms", avg_recv_time);
    println!("总数据量: {:.2} KB", total_data_kb);
    println!("总耗时: {:.2} 秒", total_time_sec);
    println!("平均吞吐量: {:.2} KB/s", throughput);

    // 计算发送和接收速度
    let send_speed = (total_bytes_sent as f64 / 1024.0) / (total_send_time as f64 / 1000.0);
    let recv_speed = (total_bytes_received as f64 / 1024.0) / (total_recv_time as f64 / 1000.0);
    println!("发送速度: {:.2} KB/s", send_speed);
    println!("接收速度: {:.2} KB/s", recv_speed);

    Ok(())
}