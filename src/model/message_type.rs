pub enum MessageType {
    LoginMessage,
    ChatMessage,
}

impl MessageType {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(MessageType::LoginMessage),
            1 => Some(MessageType::ChatMessage),
            _ => None,
        }
    }
}