use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::Data;

use crate::services::secrets::LocalSecretsService;

#[post("/v1/secrets/initialize")]
pub async fn initialize(service: Data<Arc<LocalSecretsService>>) -> Result<HttpResponse> {
    match service.initialize() {
        Ok(shares) => Ok(HttpResponse::Ok().json(shares)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[post("/v1/secrets/unseal")]
pub async fn unseal(service: Data<Arc<LocalSecretsService>>) -> Result<HttpResponse> {
    match service.unseal() {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}
