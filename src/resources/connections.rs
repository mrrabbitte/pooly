use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, patch};
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json, Path};

use crate::models::connections::{ConnectionConfig, ConnectionConfigUpdateCommand};
use crate::services::connections::config::ConnectionConfigService;
use crate::services::updatable::UpdatableService;

#[post("/v1/connections")]
pub async fn create(service: Data<Arc<ConnectionConfigService>>,
                    request: Json<ConnectionConfig>) -> Result<HttpResponse> {
    match service.create(request.0) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[get("/v1/connections/{id}")]
pub async fn get(service: Data<Arc<ConnectionConfigService>>,
                 client_id: Path<String>) -> Result<HttpResponse> {
    match service.get(&client_id.into_inner()) {
        Ok(config) => Ok(
            match config {
                None => HttpResponse::NotFound().finish(),
                Some(config_value) => HttpResponse::Ok().json(config_value.value())
            }),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[patch("/v1/connections/{id}")]
pub async fn update(service: Data<Arc<ConnectionConfigService>>,
                    client_id: Path<String>,
                    request: Json<ConnectionConfigUpdateCommand>) -> Result<HttpResponse> {
    match service.update(&client_id.into_inner(), request.0) {
        Ok(updated) => Ok(HttpResponse::Ok().json(updated)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[delete("/v1/connections/{id}")]
pub async fn delete(service: Data<Arc<ConnectionConfigService>>,
                    client_id: Path<String>) -> Result<HttpResponse> {
    match service.delete(&client_id.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}