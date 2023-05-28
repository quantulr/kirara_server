use std::collections::HashMap;

use actix_web::{get, HttpResponse, post, Responder};
use actix_web::web::to;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: String,
    username: String,
    email: String,
}


#[post("/login")]
pub async fn login() -> impl Responder {
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