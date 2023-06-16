//! src/routes/health_check.rs

use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}