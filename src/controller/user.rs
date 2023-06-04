use std::collections::HashMap;

use actix_web::web::to;
use actix_web::{get, post, HttpResponse, Responder};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod request;
