use std::env;
use std::sync::Arc;

use sea_orm::{Database, DatabaseConnection};

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
}

async fn create_dir(path: &str) -> Result<(), std::io::Error> {
    println!("{}", path);
    // 使用 Tokio 的异步文件系统操作检查目录是否存在
    let metadata = tokio::fs::metadata(path).await;

    // 检查目录是否存在
    if let Ok(metadata) = metadata {
        if !metadata.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "文件名与目录名冲突！",
            ));
        }
    } else {
        // 目录不存在，创建目录
        if let Err(_err) = tokio::fs::create_dir_all(path).await {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "目录创建失败",
            ));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let upload_path = env::var("UPLOAD_PATH").expect("UPLOAD_PATH is not set in .env file");
    create_dir(upload_path.as_str())
        .await
        .expect("directory create failed!");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let state = AppState { conn, upload_path };
    let app = create_routes(Arc::new(state));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
