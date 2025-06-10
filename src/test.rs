#[tokio::test]
async fn test_client() {
    use crate::model::message_type::MessageType;
    use crate::model::user::User;
    use crate::net::message_codec::{MessageDecoder, MessageEncoder};
    use crate::utils::io_utils::async_read_line;
    use futures::StreamExt;
    use futures::sink::SinkExt;
    use tokio::net::TcpStream;
    use tokio_util::codec::FramedRead;
    use tokio_util::codec::FramedWrite;

    // 初始化日志记录器
    tracing_subscriber::fmt::try_init().expect("Failed to initialize logger");

    // 连接到端口对应的IM服务器
    let stream = TcpStream::connect("127.0.0.1:8888")
        .await
        .expect("Failed to connect to server");
    let (reader, writer) = tokio::io::split(stream);
    tracing::debug!("Connected to server");

    // 定义行编码器
    let mut rd = FramedRead::new(reader, MessageDecoder::new());
    let mut wt = FramedWrite::new(writer, MessageEncoder::new());

    // 尝试登录
    tracing::info!("Type your login message.");
    let mut user: Option<User> = None;
    loop {
        tracing::info!("Input your username.");
        let mut username = async_read_line().await;
        tracing::info!("Input your password.");
        let mut password = async_read_line().await;

        let send = (
            MessageType::LoginMessage as usize,
            format!("{}#{}", username, password,),
        );
        wt.send(send).await.unwrap();

        if let Some(result) = rd.next().await {
            match result {
                Ok((_, message)) => match message.as_str() {
                    "Invalid login attempt" => {
                        tracing::info!("Login failed.")
                    }
                    _ => {
                        tracing::info!("Login successful.");
                        user.replace(User::new(username, password));
                        break;
                    }
                },
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
            let (t, message) = result.unwrap();
            let message_type: MessageType = MessageType::from_index(t).unwrap();
            match message_type {
                MessageType::LoginMessage => {}
                MessageType::BroadcastMessage => {
                    let mut split = message.split("#");
                    let username = split.next().unwrap();
                    let content = split.next().unwrap();
                    tracing::info!("Broadcast from {}: {}", username, content);
                }
                MessageType::ChatToUserMessage => {
                    let mut split = message.split("#");
                    let username = split.next().unwrap();
                    let to_username = split.next().unwrap();
                    let content = split.next().unwrap();
                    tracing::info!("Received from {} to {}: {}", username, to_username, content);
                }
            }
        }
    });

    // 向服务器发送消息
    tracing::info!("Operation Menu:");
    tracing::info!("1. broadcast your message to all users.");
    tracing::info!("2. chat to a user.");
    tracing::info!("3. quit.");
    tracing::info!("Input 'back' when your want back to menu.");
    loop {
        tracing::info!("Type a number to select.");
        let mut input = async_read_line().await; // 等待用户输入

        match input.as_str() {
            "1" => {
                input.clear();
                tracing::info!("Type your message.");
                input = async_read_line().await;

                if input == "back".to_string() {
                    input.clear();
                    continue;
                }

                let message = (
                    MessageType::BroadcastMessage as usize,
                    format!("{}#{}", user.clone().unwrap().username, input),
                );
                wt.send(message).await.unwrap();
                tracing::info!("Message sent.");
                input.clear();
            }
            "2" => {
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

                let message = (
                    MessageType::ChatToUserMessage as usize,
                    format!(
                        "{}#{}#{}",
                        user.clone().unwrap().username,
                        to_username,
                        input
                    ),
                );
                wt.send(message).await.unwrap();
                tracing::info!("Message sent.");
                input.clear();
            }
            "3" => {
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
