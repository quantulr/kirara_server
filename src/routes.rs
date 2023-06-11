use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use sea_orm::sea_query::ColumnSpec::Default;
use tower_http::limit::{RequestBodyLimit, RequestBodyLimitLayer};

use crate::controller::{
    image::api::upload_image,
    user::api::{login, register},
};
use crate::middleware::auth::auth;
use crate::AppState;

async fn index() -> Html<&'static str> {
    Html("<h2 style='text-align: center;margin-top: 100px;'>hello, world</h2>")
}

// async fn favicon() {
//     // ServeFile::new("/assets/favicon.png")
// }

pub fn create_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        // .route_service("/favicon.ico", ServeFile::new("/assets/favicon.png"))
        // .route("/favicon.ico", get())
        .nest(
            "/user",
            Router::new()
                .route("/login", post(login))
                .route("/register", post(register)),
        )
        .nest("/image", Router::new().route("/upload", post(upload_image)))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth,
        ))
        .with_state(app_state)
}
