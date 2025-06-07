mod model;
mod service;
mod test;
mod net;

use crate::model::message_type::MessageType;
use crate::service::user_service::login;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::net::tcp::WriteHalf;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
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
        // 使用行解码器解析消息
        let decoder = LinesCodec::new();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            let mut rd = FramedRead::new(reader, decoder.clone());
            let mut wt = FramedWrite::new(writer, decoder.clone());

            // 匹配消息类型
            while let Some(line) = rd.next().await {
                match line {
                    Ok(message) => {
                        tracing::debug!("Message: {}", message);
                        route_line(&mut wt, message).await; // 调用路由函数处理对应类型的消息
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
async fn route_line(wt: &mut FramedWrite<WriteHalf<'_>, LinesCodec>, message: String) {
    // 处理message格式，格式为 [type#body]
    let mut split = message.split("#");
    let t = split.next().unwrap().to_string().parse::<usize>().unwrap();
    let username = split.next().unwrap();
    
    // 匹配消息类型
    match MessageType::from_index(t).unwrap() {
        // 登录：body格式为 [username#password]
        MessageType::LoginMessage => {
            tracing::info!("Received login message: {}", username);
            let password = split.next().unwrap();
            match login(username.to_string(), password.to_string()).await {
                Some(user) => {
                    tracing::info!("User {} logged in", user.username);
                    wt.send(user.username).await.unwrap();
                }
                None => {
                    tracing::info!("Invalid login attempt");
                    wt.send("Invalid login attempt".to_string()).await.unwrap();
                }
            }
        }
        // 与服务器对话：body格式为 [username#chat]
        MessageType::ChatMessage => {
            let chat_message = split.next().unwrap();
            tracing::info!("Received chat message from {}: {}", username, chat_message);
        }
    }
}
