use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}
