use std::sync::Arc;

use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::Request;
use axum::response::Html;
use axum::routing::{get, patch, post};
use axum::Router;
use http_body::Limited;
use tower::{service_fn, ServiceExt};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeFile;

use crate::controller::media::api::{get_media, /*test_static_server,*/ upload_media};
use crate::controller::{
    image::api::{get_image, get_image_history, image_thumbnail, upload_image},
    user::api::{login, register, update_user, user_info},
};
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
                .route("/register", post(register))
                .route("/info", get(user_info))
                .route("/update", patch(update_user)),
        )
        .nest(
            "/v",
            Router::new()
                .route("/upload", post(upload_media))
                .route("/:year/:month/:day/:file_name", get(get_media))
                .route_service(
                    "/s/:year/:month/:day/:file_name",
                    service_fn(move |req: Request<Limited<Body>>| async move {
                        ServeFile::new(format!(
                            "C:/Users/quant/Pictures/upload_path/{}",
                            req.uri().path().replace("/s/", "")
                        ))
                        .oneshot(req)
                        .await
                        .map_err(|e| {
                            println!("error: {}", e);
                            e
                        })
                    }),
                ),
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
