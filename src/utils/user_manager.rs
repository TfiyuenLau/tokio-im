use crate::model::message_type::MessageType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

pub type UserManager = Arc<Mutex<HashMap<String, Sender<(MessageType, String)>>>>;

// 登录时注册用户
pub fn register_user(pool: &UserManager, username: String, sender: Sender<(MessageType, String)>) {
    pool.lock().unwrap().insert(username, sender);
}

// 登出时移除用户
pub fn unregister_user(pool: &UserManager, username: &str) {
    pool.lock().unwrap().remove(username);
}
