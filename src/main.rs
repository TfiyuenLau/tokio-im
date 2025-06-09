mod model;
mod net;
mod service;
mod test;

use crate::model::message_type::MessageType;
use crate::model::user::User;
use crate::net::message_codec::{MessageDecoder, MessageEncoder};
use crate::service::user_service::login;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::net::tcp::WriteHalf;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // 初始化日志记录器
    tracing_subscriber::registry().with(fmt::layer()).init();

    // 绑定到指定地址，监听传入的连接
    let listener = TcpListener::bind("127.0.0.1:8888")
        .await
        .expect("Failed to bind");

    // 循环异步处理连接，避免阻塞主循环
    loop {
        // 接收连接
        let (mut socket, addr) = listener.accept().await.unwrap();
        tracing::info!("Accepted connection from: {}", addr);
        // 创建自定义编解码器
        let decoder = MessageDecoder::new();
        let encoder = MessageEncoder::new();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            let mut rd = FramedRead::new(reader, decoder);
            let mut wt = FramedWrite::new(writer, encoder);

            // 匹配消息类型
            while let Some(message) = rd.next().await {
                match message {
                    Ok((message_type, message)) => {
                        tracing::debug!("Message: {}", message);

                        // 调用路由函数处理对应类型的消息
                        let message_type = MessageType::from_index(message_type);
                        route_line(&mut wt, message_type.unwrap(), message).await;
                    }
                    Err(error) => {
                        tracing::error!("Error reading message: {}", error);
                        break;
                    }
                }
            }
        });
    }
}

// 手动处理路由
async fn route_line(
    wt: &mut FramedWrite<WriteHalf<'_>, MessageEncoder>,
    message_type: MessageType,
    message: String,
) {
    // 处理message格式
    let mut split = message.split("#");
    let username = split.next().unwrap();

    // 匹配消息类型
    match message_type {
        // 登录：body格式为 [username#password]
        MessageType::LoginMessage => {
            tracing::info!("Received login message: {}", username);
            let password = split.next().unwrap();
            let user = User::new(username.to_string(), password.to_string());
            match login(user).await {
                Some(user) => {
                    tracing::info!("User {} logged in", user.username);
                    let send = (MessageType::LoginMessage as usize, user.username);
                    wt.send(send).await.unwrap();
                }
                None => {
                    tracing::info!("Invalid login attempt");
                    let send = (
                        MessageType::LoginMessage as usize,
                        "Invalid login attempt".to_string(),
                    );
                    wt.send(send).await.unwrap();
                }
            }
        }
        // 与服务器对话：body格式为 [username#chat]
        MessageType::ChatToServerMessage => {
            let chat_message = split.next().unwrap();
            tracing::info!("Received chat message from {}: {}", username, chat_message);
        }
    }
}
