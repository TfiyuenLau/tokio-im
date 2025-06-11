#[derive(Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl User {
    #[allow(dead_code)]
    pub fn new(username: String, password: String) -> Self {
        User { username, password }
    }
}