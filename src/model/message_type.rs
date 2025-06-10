pub enum MessageType {
    LoginMessage,
    BroadcastMessage,
    ChatToUserMessage,
}

impl MessageType {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(MessageType::LoginMessage),
            1 => Some(MessageType::BroadcastMessage),
            2 => Some(MessageType::ChatToUserMessage),
            _ => None,
        }
    }
}