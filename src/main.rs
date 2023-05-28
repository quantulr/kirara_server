use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};

use middleware::auth::Authentication;

mod routes;
mod middleware;
mod controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Authentication)
            .configure(routes::api)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
