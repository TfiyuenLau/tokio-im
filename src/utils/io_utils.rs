/// 阻塞当前线程但不阻塞子线程，等待用户输入（用于测试客户端）
#[allow(dead_code)]
pub async fn async_read_line() -> String {
    tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    })
    .await
    .unwrap()
}
