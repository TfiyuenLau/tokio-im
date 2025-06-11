mod common;
mod model;
mod net;
mod service;
mod test;

use crate::common::io_utils::match_message_type;
use crate::common::user_manager::{UserManager, register_user, unregister_user};
use crate::model::user::User;
use crate::net::protobuf_codec::ProtobufCodec;
use crate::service::user_service::login;
use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Sender, channel};
use tokio_im::protobuf::im::im_message::Payload;
use tokio_im::protobuf::im::{
    BroadcastDto, GetAliveListResponse, ImMessage, LoginResponse, MessageType,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // 初始化日志记录器
    tracing_subscriber::registry().with(fmt::layer()).init();
    // 读取环境配置
    dotenv().ok();
    let port = env::var("PORT").unwrap_or("8888".to_string());

    // 绑定到指定端口，监听传入的连接
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind");

    // 创建用户管理器
    let users: UserManager = Arc::new(Mutex::new(
        HashMap::<String, Sender<(MessageType, Payload)>>::new(),
    ));

    // 循环异步处理连接，避免阻塞主循环
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let users: UserManager = Arc::clone(&users);
        tracing::info!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            handle_connection(socket, users).await;
        });
    }
}

// 处理客户端的连接请求
async fn handle_connection(socket: TcpStream, users: UserManager) {
    let mut current_username: Option<String> = None;

    // 使用自定义Codec实现消息编解码
    let (reader, writer) = tokio::io::split(socket);
    let mut wt = FramedWrite::new(writer, ProtobufCodec::new());
    let mut rd = FramedRead::new(reader, ProtobufCodec::new());

    // 通过消息传递实现异步任务通信
    let (tx, mut rx) = channel::<(MessageType, Payload)>(32);

    // 异步接收并处理通道消息
    tokio::spawn(async move {
        while let Some((message_type, payload)) = rx.recv().await {
            match message_type {
                MessageType::LoginMessage => {
                    let send = ImMessage {
                        message_type: MessageType::LoginMessage as i32,
                        payload: Some(payload),
                    };
                    wt.send(send).await.unwrap();
                }
                MessageType::BroadcastMessage => {
                    let send = ImMessage {
                        message_type: MessageType::BroadcastMessage as i32,
                        payload: Some(payload),
                    };
                    wt.send(send).await.unwrap();
                }
                MessageType::GetAliveListMessage => {
                    let send = ImMessage {
                        message_type: MessageType::GetAliveListMessage as i32,
                        payload: Some(payload),
                    };
                    wt.send(send).await.unwrap();
                }
                MessageType::ChatToUserMessage => {
                    let send = ImMessage {
                        message_type: MessageType::ChatToUserMessage as i32,
                        payload: Some(payload),
                    };
                    wt.send(send).await.unwrap();
                }
            }
        }
    });

    // 主循环读取客户端发送的消息并路由
    while let Some(received) = rd.next().await {
        match received {
            Ok(im_message) => {
                // 处理im_message
                let message_type = match_message_type(im_message.message_type).unwrap();
                let payload = im_message.payload.as_ref().unwrap();
                tracing::debug!("Message: {:?}", payload);

                // 匹配消息类型
                match message_type {
                    // 用户登录请求
                    MessageType::LoginMessage => {
                        if let Payload::LoginRequest(message) = payload {
                            tracing::info!("Received login message: {}", message.username);

                            let user = User {
                                username: message.clone().username,
                                password: message.clone().password,
                            };
                            match login(user).await {
                                Some(user) => {
                                    tracing::info!("User {} logged in", user.username);
                                    current_username.replace(user.username.clone());
                                    register_user(&users, user.username.clone(), tx.clone());

                                    let send = (
                                        MessageType::LoginMessage,
                                        Payload::LoginResponse(LoginResponse {
                                            username: message.clone().username,
                                        }),
                                    );
                                    tx.send(send).await.unwrap();
                                }
                                None => {
                                    tracing::info!("Invalid login attempt");
                                    let send = (
                                        MessageType::LoginMessage,
                                        Payload::LoginResponse(LoginResponse {
                                            username: "Invalid login attempt".to_string(),
                                        }),
                                    );
                                    tx.send(send).await.unwrap();
                                }
                            }
                        }
                    }
                    // 与服务器对话并广播
                    MessageType::BroadcastMessage => {
                        if let Payload::BroadcastDto(message) = payload {
                            tracing::info!(
                                "Received chat message from {}: {}",
                                message.username,
                                message.content
                            );

                            let txs: Vec<Sender<(MessageType, Payload)>> = {
                                let users_lock = users.lock().unwrap();
                                users_lock.values().cloned().collect()
                            }; // 销毁users_lock(MutexGuard)变量

                            // 将消息广播给所有用户
                            for tx in txs {
                                let send = (
                                    MessageType::BroadcastMessage,
                                    Payload::BroadcastDto(BroadcastDto {
                                        username: "".to_string(),
                                        content: message.clone().content,
                                    }),
                                );
                                tx.send(send).await.unwrap();
                            }
                        }
                    }
                    // 获取在线用户列表
                    MessageType::GetAliveListMessage => {
                        if let Payload::GetAliveListRequest(message) = payload {
                            let username_vec: Vec<String> = {
                                let users_lock = users.lock().unwrap();
                                users_lock.keys().cloned().collect()
                            };
                            let users_str = username_vec.join(", ");

                            tracing::info!(
                                "Requested alive list from {}: {}",
                                message.username,
                                users_str
                            );
                            let send = (
                                MessageType::GetAliveListMessage,
                                Payload::GetAliveListResponse(GetAliveListResponse {
                                    usernames: users_str,
                                }),
                            );
                            tx.send(send).await.unwrap();
                        }
                    }
                    // 与指定用户对话
                    MessageType::ChatToUserMessage => {
                        if let Payload::ChatToUserDto(message) = payload {
                            tracing::info!(
                                "From {} to {}: {}",
                                message.from_username,
                                message.to_username,
                                message.content
                            );

                            // 获取消息接收方的发送通道
                            let recv_tx = {
                                let users_lock = users.lock().unwrap();
                                match users_lock.get(&message.to_username) {
                                    Some(tx) => tx.clone(),
                                    None => {
                                        tracing::warn!(
                                            "Target user {} not found",
                                            message.to_username
                                        );
                                        continue;
                                    }
                                }
                            }; // 销毁users_lock(MutexGuard)变量

                            let send = (
                                MessageType::ChatToUserMessage,
                                Payload::ChatToUserDto(message.clone()),
                            );
                            recv_tx.send(send).await.unwrap();
                        }
                    }
                }
            }

            Err(error) => {
                tracing::error!("Error reading message: {}", error);
                break;
            }
        }
    }

    // 处理用户登出
    if let Some(username) = current_username {
        unregister_user(&users, &username);
        tracing::info!("User {} disconnected", username);
    } else {
        tracing::info!("Anonymous user disconnected");
    }
}
