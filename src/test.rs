#[tokio::test]
async fn test_client() {
    use crate::model::message_type::MessageType;
    use crate::model::user::User;
    use crate::net::message_codec::{MessageDecoder, MessageEncoder};
    use futures::StreamExt;
    use futures::sink::SinkExt;
    use std::io;
    use tokio::net::TcpStream;
    use tokio_util::codec::FramedRead;
    use tokio_util::codec::FramedWrite;
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // 初始化日志记录器
    tracing_subscriber::registry().with(fmt::layer()).init();

    // 连接到端口对应的IM服务器
    let mut stream = TcpStream::connect("127.0.0.1:8888")
        .await
        .expect("Failed to connect to server");
    let (reader, writer) = stream.split();
    tracing::debug!("Connected to server");

    // 定义行编码器
    let decoder = MessageDecoder::new();
    let encoder = MessageEncoder::new();
    let mut rd = FramedRead::new(reader, decoder);
    let mut wt = FramedWrite::new(writer, encoder);

    // 尝试登录
    tracing::info!("Type your login message.");
    let mut username = String::new();
    let mut password = String::new();
    loop {
        tracing::info!("Input your username.");
        io::stdin().read_line(&mut username).unwrap();
        tracing::info!("Input your password.");
        io::stdin().read_line(&mut password).unwrap();

        username = username.trim().to_string();
        password = password.trim().to_string();
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

    // 设置登录用户
    let user = User::new(username, password);

    // 向服务器发送消息
    tracing::info!("Type your message.");
    tracing::info!("Input 'exit' when your want quit.");
    let mut line = String::new();
    loop {
        io::stdin().read_line(&mut line).unwrap();
        let input = line.trim();
        if input == "exit" {
            tracing::info!("Quit.");
            break;
        }
        let message = (
            MessageType::ChatToServerMessage as usize,
            format!("{}#{}", user.username, input),
        );
        wt.send(message).await.unwrap();
        line.clear();
        tracing::info!("Message sent.");
    }
}
