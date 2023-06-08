use std::sync::Arc;

use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;

use crate::controller::user::api::{login, register};
use crate::AppState;

async fn index() -> Html<&'static str> {
    Html("<h2 style='text-align: center;margin-top: 100px;'>hello, world</h2>")
}

pub fn create_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .nest(
            "/user",
            Router::new()
                .route("/login", post(login))
                .route("/register", post(register)),
        )
        .with_state(app_state)
}
