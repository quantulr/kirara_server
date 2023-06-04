use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Signup {
    pub user: SignupUser,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignupUser {
    pub username: String,
    pub email: String,
    pub password: String,
}
