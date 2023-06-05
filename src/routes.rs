use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;

use crate::controller::user::api::login;

async fn index() -> Html<&'static str> {
    Html("<h2 style='text-align: center;margin-top: 100px;'>hello, world</h2>")
}

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .nest("/user", Router::new().route("/login", post(login)))
}
