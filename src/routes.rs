use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::response::Html;
use axum::Router;
use axum::routing::{get, patch, post, put};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeFile;

use crate::AppState;
use crate::controller::{
    image::api::{get_image, get_image_history, image_thumbnail, upload_image},
    user::api::{login, register, update_user, user_info},
};
use crate::controller::media::api::{get_media, upload_media};
use crate::middleware::auth::auth;

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
                .route("/register", post(register))
                .route("/info", get(user_info))
                .route("/update", patch(update_user)),
        )
        .nest(
            "/v",
            Router::new()
                .route("/upload", post(upload_media))
                .route("/:year/:month/:day/:file_name", get(get_media))
                .route_service("/tv/2023/06/30/910a081669fc458c9fae01f3ba88b351.mp4", ServeFile::new("/Volumes/iMac Doc/Pictures/upload/2023/06/30/910a081669fc458c9fae01f3ba88b351.mp4")),
        )
        .nest(
            "/image",
            Router::new()
                .route("/upload", post(upload_image))
                .route("/:year/:month/:day/:file_name", get(get_image))
                .route(
                    "/thumbnail/:year/:month/:day/:file_name",
                    get(image_thumbnail),
                )
                .route("/history", get(get_image_history)),
        )
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
