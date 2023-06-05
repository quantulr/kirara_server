use axum::handler::HandlerWithoutStateExt;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::routes::create_routes;

mod controller;
mod entities;
mod routes;

#[tokio::main]
async fn main() {
    let app = create_routes();
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
