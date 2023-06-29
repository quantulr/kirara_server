use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::controller::user::response::Claims;
use crate::entities::users;

pub async fn get_user_from_token(
    token: &str,
    secret: &str,
    conn: &DatabaseConnection,
) -> Option<users::Model> {
    // 从token中获取用户名
    let username = match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(data) => data.claims.username,
        Err(_err) => {
            return None;
        }
    };
    // 从数据库中获取用户
    if let Ok(Some(user)) = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(conn)
        .await
    {
        Some(user)
    } else {
        None
    }
}
