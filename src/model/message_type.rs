pub enum MessageType {
    LoginMessage,
    ChatToServerMessage,
}

impl MessageType {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(MessageType::LoginMessage),
            1 => Some(MessageType::ChatToServerMessage),
            _ => None,
        }
    }
}