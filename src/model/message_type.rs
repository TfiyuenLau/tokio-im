pub enum MessageType {
    LoginMessage,
    BroadcastMessage,
    GetAliveListMessage,
    ChatToUserMessage,
}

impl MessageType {
    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(MessageType::LoginMessage),
            1 => Some(MessageType::BroadcastMessage),
            2 => Some(MessageType::GetAliveListMessage),
            3 => Some(MessageType::ChatToUserMessage),
            _ => None,
        }
    }
}
