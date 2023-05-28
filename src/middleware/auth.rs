use std::collections::HashMap;
use std::future::{ready, Ready};

use actix_web::{body::EitherBody, dev::{self, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse};
use futures_util::future::LocalBoxFuture;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Authentication;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Authentication
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        println!("{path}");
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?.map_into_left_body();
            Ok(res)
        })
        // if path == "/user/ip_addr" {
        //     let fut = self.service.call(req);
        //     Box::pin(async move {
        //         let res = fut.await?.map_into_left_body();
        //         Ok(res)
        //     })
        // } else {
        //     Box::pin(async move {
        //         let (req, _res) = req.into_parts();
        //         let mut resp_json = HashMap::new();
        //         resp_json.insert("msg", "未登录");
        //         let res = HttpResponse::Unauthorized().json(resp_json).map_into_right_body();
        //         let srv = ServiceResponse::new(req, res);
        //         Ok(srv)
        //     })
        // }
    }
}

fn get_user_id_from_token(req: &ServiceRequest) -> Result<&str, &str> {
    req.headers().get("Authorization")
        .ok_or("can't get token from header")
        .and_then(|auth_header| auth_header.to_str().map_err(|_err| "can't stringify"))
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer") {
                Ok(auth_str)
            } else {
                Err("Invalid token")
            }
        })
}

fn should_skip_auth(req: &ServiceRequest) -> bool {
    true
}