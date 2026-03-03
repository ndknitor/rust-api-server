mod auth;
mod errors;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::entities::user::Model as UserModel;

pub use auth::AuthImpl;
pub use errors::AuthError;

#[async_trait]
pub trait Auth: Send + Sync {
    async fn find_user_by_email_password(
        &self,
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> Result<UserModel, AuthError>;
}
