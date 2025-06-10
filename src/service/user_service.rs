use crate::model::user::User;
use std::collections::HashMap;

// 登录验证
pub async fn login(user: User) -> Option<User> {
    // 模拟用户数据
    let mut users: HashMap<String, String> = HashMap::new();
    users.insert("zhangsan".to_string(), "123".to_string());
    users.insert("lisi".to_string(), "123".to_string());
    users.insert("wangwu".to_string(), "123".to_string());

    // 登录逻辑实现
    if !users.contains_key(&user.username) || users.get(&user.username).unwrap() != &user.password {
        None
    } else {
        Some(user)
    }
}
