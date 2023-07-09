use std::env;
use std::sync::Arc;

use sea_orm::{Database, DatabaseConnection};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use utils::dir;

use crate::routes::create_routes;

mod controller;
mod entities;
mod middleware;
mod routes;
mod utils;

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
    upload_path: String,
    jwt_secret: String,
    wsrv_nl_port: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let upload_path = env::var("UPLOAD_PATH").expect("UPLOAD_PATH is not set in .env file");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is not set in .env file");
    let wsrv_nl_port = env::var("WSRV_NL_PORT").expect("WSRV_NL_PORT is not set in .env file");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建上传文件夹
    dir::create_dir(upload_path.as_str())
        .await
        .expect("directory create failed!");

    // 连接数据库
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let state = AppState {
        conn,
        upload_path,
        jwt_secret,
        wsrv_nl_port,
    };
    let app = create_routes(Arc::new(state)).layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
