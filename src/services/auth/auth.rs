use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};

use crate::entities::user::Entity as User;
use crate::entities::user::Column as UserColumn;
use crate::entities::user::Model as UserModel;
use crate::services::auth::errors::AuthError;

pub struct AuthImpl;

impl AuthImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl super::Auth for AuthImpl {
    async fn find_user_by_email_password(
        &self,
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> Result<UserModel, AuthError> {
        let user = User::find()
            .filter(UserColumn::Email.eq(email))
            .one(db)
            .await?;

        let user = user.ok_or(AuthError::InvalidCredentials)?;

        if user.password != password {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user)
    }
}
