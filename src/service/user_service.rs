use std::collections::HashMap;
use crate::model::user::User;

// 登录验证
pub async fn login(username: String, password: String) -> Option<User> {
    let mut users: HashMap<String, String> = HashMap::new();
    users.insert("zhangsan".to_string(), "123456".to_string());
    users.insert("lisi".to_string(), "123456".to_string());
    users.insert("wangwu".to_string(), "123456".to_string());
    
    if !users.contains_key(&username) || users.get(&username).unwrap() != &password {
        None
    } else {
        let user = User {
            username,
            password,
        };
        Some(user)
    }
}
