use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use diesel::{PgConnection, r2d2};
use diesel::pg::Pg;

use middleware::auth::Authentication;
#[macro_use]
extern crate diesel;
mod routes;
mod middleware;
mod controller;
mod models;
mod schema;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let pool = initialize_db_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Authentication)
            .configure(routes::api)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
