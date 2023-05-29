use actix_web::{get, HttpResponse, post, Responder, web};
use actix_web::web::ServiceConfig;
use futures_util::future::MaybeDone::Future;
use futures_util::TryFutureExt;
use reqwest;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::controller::user::api::{login, register};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


#[get("/list")]
async fn list_user() -> impl Responder { HttpResponse::Ok().body("users list") }


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpJsonResp {
    pub status: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub zip: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub isp: String,
    pub org: String,
    #[serde(rename = "as")]
    pub as_field: String,
    pub query: String,
}

#[get("/ip_addr")]
async fn get_ip() -> impl Responder {
    if let Ok(resp) = reqwest::get("http://ip-api.com/json").await {
        if let Ok(result) = resp.json::<IpJsonResp>().await {
            HttpResponse::Ok().json(result)
        } else {
            HttpResponse::Ok().body("error")
        }
    } else {
        HttpResponse::Ok().body("error")
    }
}

pub fn api(cfg: &mut ServiceConfig) {
    cfg.service(hello).service(
        web::scope("/user")
            .service(list_user)
            .service(login)
            .service(register)
            .service(get_ip)
    );
}