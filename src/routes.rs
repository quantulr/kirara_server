use actix_web::web::ServiceConfig;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::future::MaybeDone::Future;
use futures_util::TryFutureExt;
use reqwest;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::controller::user::api::{login, register};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<h1>Hello world!</h1>")
}

pub fn api(cfg: &mut ServiceConfig) {
    cfg.service(index)
        .service(web::scope("/user").service(register).service(login));
}
