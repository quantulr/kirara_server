use std::sync::Arc;

use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;

use crate::controller::user::api::{login, register};
use crate::middleware::auth::auth;
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
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth,
        ))
        .with_state(app_state)
}
