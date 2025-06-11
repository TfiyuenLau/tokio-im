#[tokio::test]
async fn test_client() {
    use crate::common::io_utils::async_read_line;
    use crate::common::io_utils::match_message_type;
    use crate::model::user::User;
    use crate::net::protobuf_codec::ProtobufCodec;
    use dotenv::dotenv;
    use futures::StreamExt;
    use futures::sink::SinkExt;
    use std::env;
    use tokio::net::TcpStream;
    use tokio_im::protobuf::im::ImMessage;
    use tokio_im::protobuf::im::LoginRequest;
    use tokio_im::protobuf::im::MessageType;
    use tokio_im::protobuf::im::im_message::Payload;
    use tokio_im::protobuf::im::{BroadcastDto, ChatToUserDto, GetAliveListRequest};
    use tokio_util::codec::FramedRead;
    use tokio_util::codec::FramedWrite;

    // 初始化日志记录器
    tracing_subscriber::fmt::try_init().expect("Failed to initialize logger");
    // 读取配置
    dotenv().ok();
    let server_addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("8888".to_string());

    // 连接到端口对应的IM服务器
    let stream = TcpStream::connect(format!("{}:{}", server_addr, port))
        .await
        .expect("Failed to connect to server");
    tracing::debug!("Connected to server");

    // 定义行编码器
    let (reader, writer) = tokio::io::split(stream);
    let mut rd = FramedRead::new(reader, ProtobufCodec::new());
    let mut wt = FramedWrite::new(writer, ProtobufCodec::new());

    // 尝试登录
    tracing::info!("Type your login message.");
    let mut user: Option<User> = None;
    loop {
        tracing::info!("Input your username.");
        let mut username = async_read_line().await;
        tracing::info!("Input your password.");
        let mut password = async_read_line().await;

        let send = ImMessage {
            message_type: MessageType::LoginMessage as i32,
            payload: Some(Payload::LoginRequest(LoginRequest {
                username: username.clone(),
                password: password.clone(),
            })),
        };
        wt.send(send).await.unwrap();

        if let Some(result) = rd.next().await {
            match result {
                Ok(im_message) => {
                    let payload = im_message.payload.as_ref().unwrap();
                    if let Payload::LoginResponse(message) = payload {
                        match message.username.as_str() {
                            "Invalid login attempt" => {
                                tracing::error!("Login failed.")
                            }
                            _ => {
                                tracing::info!("Login successful.");
                                user.replace(User::new(username, password));
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("Error reading line: {}", err);
                    break;
                }
            }
        }

        username.clear();
        password.clear();
    }

    // 启动新线程处理接收到的消息
    tokio::spawn(async move {
        tracing::info!("Waiting for message...");
        while let Some(result) = rd.next().await {
            let im_message = result.unwrap();
            let message_type: MessageType = match_message_type(im_message.message_type).unwrap();
            let payload = im_message.payload.as_ref().unwrap();

            match message_type {
                MessageType::LoginMessage => {}
                MessageType::BroadcastMessage => {
                    if let Payload::BroadcastDto(message) = payload {
                        tracing::info!("Broadcast from {}: {}", message.username, message.content);
                    }
                }
                MessageType::GetAliveListMessage => {
                    if let Payload::GetAliveListResponse(message) = payload {
                        tracing::info!("Alive list: {}", message.usernames);
                    }
                }
                MessageType::ChatToUserMessage => {
                    if let Payload::ChatToUserDto(message) = payload {
                        tracing::info!("Chat from {}: {}", message.from_username, message.content);
                    }
                }
            }
        }
    });

    // 向服务器发送消息
    tracing::info!("Operation Menu:");
    tracing::info!("1. get alive user list.");
    tracing::info!("2. broadcast your message to all users.");
    tracing::info!("3. chat to a user.");
    tracing::info!("9. quit.");
    tracing::info!("Input 'back' when your want back to menu.");
    loop {
        tracing::info!("Type a number to select.");
        let mut input = async_read_line().await; // 等待用户输入

        match input.as_str() {
            "1" => {
                input.clear();
                let send = ImMessage {
                    message_type: MessageType::GetAliveListMessage as i32,
                    payload: Some(Payload::GetAliveListRequest(GetAliveListRequest {
                        username: user.clone().unwrap().username,
                    })),
                };
                wt.send(send).await.unwrap();
                input.clear()
            }
            "2" => {
                input.clear();
                tracing::info!("Type your message.");
                input = async_read_line().await;

                if input == "back".to_string() {
                    input.clear();
                    continue;
                }

                let send = ImMessage {
                    message_type: MessageType::BroadcastMessage as i32,
                    payload: Some(Payload::BroadcastDto(BroadcastDto {
                        username: user.clone().unwrap().username,
                        content: input.clone(),
                    })),
                };
                wt.send(send).await.unwrap();
                tracing::info!("Message sent.");

                input.clear();
            }
            "3" => {
                input.clear();

                tracing::info!("Type a username.");
                let mut to_username = async_read_line().await;
                if to_username == "back".to_string() {
                    to_username.clear();
                    continue;
                }

                tracing::info!("Type your message.");
                input = async_read_line().await;
                if input == "back".to_string() {
                    input.clear();
                    continue;
                }

                let send = ImMessage {
                    message_type: MessageType::ChatToUserMessage as i32,
                    payload: Some(Payload::ChatToUserDto(ChatToUserDto {
                        from_username: user.clone().unwrap().username,
                        to_username,
                        content: input.clone(),
                    })),
                };
                wt.send(send).await.unwrap();
                tracing::info!("Message sent.");

                input.clear();
            }
            "9" => {
                tracing::info!("Quit.");
                break;
            }
            _ => {
                tracing::info!("Invalid operation: '{}'", input);
                input.clear();
            }
        }
    }
}
