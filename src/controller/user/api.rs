use std::collections::HashMap;

use actix_web::{HttpResponse, post, Responder, web};
use diesel::query_builder::InsertStatement;
use jsonwebtoken::{EncodingKey, Header};
use serde_derive::{Deserialize, Serialize};

use crate::controller::user::request;
use crate::DbPool;
use crate::models::user::{NewUser, User};
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: String,
    username: String,
    email: String,
}

#[post("/register")]
pub async fn register(pool: web::Data<DbPool>, form: web::Json<request::Signup>) -> impl Responder {
    println!("{}", form.user.email);
    if let Ok(mut coon) = pool.get() {
        use crate::schema::users;
        let new_user = NewUser {
            username: form.user.username.to_string(),
            password: form.user.password.to_string(),
            email: form.user.email.to_string(),
        };
        let coom = &mut coon;
        NewUser::new(coom);
        // let user = &new_user;
        // let res = diesel::insert_into(users::table).values(user);
        HttpResponse::Ok().body("success")
    } else {
        HttpResponse::InternalServerError().body("注册失败")
    }
}

#[post("/login")]
pub async fn login(state: web::Data<String>, form: web::Json<request::Signup>) -> impl Responder {
    println!("{}", form.user.email);
    let my_claims = Claims {
        id: String::from("123"),
        username: String::from("ayaya"),
        email: String::from("ayaya@gmail.com"),
    };
    if let Ok(token) = jsonwebtoken::encode(&Header::default(), &my_claims, &EncodingKey::from_secret("qwerty123456".as_ref()))
    {
        let mut resp_json = HashMap::new();
        resp_json.insert("token", token);
        HttpResponse::Ok().json(resp_json)
    } else {
        HttpResponse::InternalServerError().body("error")
    }
}
