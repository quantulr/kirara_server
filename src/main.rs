use std::env;
use std::sync::Arc;

use axum::handler::HandlerWithoutStateExt;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::{Database, DatabaseConnection};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::routes::create_routes;

mod controller;
mod entities;
mod routes;

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let state = AppState { conn };
    let app = create_routes(Arc::new(state));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
