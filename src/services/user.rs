use async_trait::async_trait;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_users(&self) -> Vec<String>;
}

pub struct UserServiceImpl;

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_users(&self) -> Vec<String> {
        vec![
            "Alice".to_string(),
            "Bob".to_string(),
        ]
    }
}
