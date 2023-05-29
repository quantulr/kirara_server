use diesel;
use diesel::{PgConnection, RunQueryDsl};
use diesel::result::Error;
use serde::{Deserialize, Serialize};

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
}

/// New user details.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

impl NewUser {
    /// Constructs new user details from name.

    pub fn new(coon: &mut PgConnection
    ) -> Result<User, Error> {
        let record = NewUser {
            username: "ayaya".to_owned(),
            email: "ayaya@gmail.com".to_owned(),
            password: "123456".to_owned(),
        };
        let user = diesel::insert_into(users::table).values(&record).get_result(coon)?;
        Ok(user)
    }
}