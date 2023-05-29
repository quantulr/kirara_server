use std::collections::HashMap;

use actix_web::{get, HttpResponse, post, Responder};
use actix_web::web::to;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod request;