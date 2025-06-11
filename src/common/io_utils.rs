use tokio_im::protobuf::im::MessageType;

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

/// 将ImMessage消息的message_type(i32)匹配至其MessageType枚举
pub fn match_message_type(message_type: i32) -> Option<MessageType> {
    match message_type {
        0 => Some(MessageType::LoginMessage),
        1 => Some(MessageType::BroadcastMessage),
        2 => Some(MessageType::GetAliveListMessage),
        3 => Some(MessageType::ChatToUserMessage),
        _ => None,
    }
}
