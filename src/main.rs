use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sea_orm::{Database, DatabaseConnection};

use middleware::auth::Authentication;

mod controller;
mod entities;
mod middleware;
mod routes;

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let db_url = String::from("mysql://raichi:raichi169482@192.168.0.101/kirara");
    let conn = Database::connect(&db_url).await.unwrap();
    let state = AppState { conn };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Authentication)
            .configure(routes::api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
