use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, patch};
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json, Path};

use crate::JwtAuthService;
use crate::models::auth::jwt::{JwtKey, JwtKeyCreateCommand, JwtKeyUpdateCommand};
use crate::services::updatable::UpdatableService;

#[post("/v1/keys")]
pub async fn create(service: Data<Arc<JwtAuthService>>,
                    request: Json<JwtKeyCreateCommand>) -> Result<HttpResponse> {
    match service.create(request.0.into()) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[get("/v1/keys/{id}")]
pub async fn get(service: Data<Arc<JwtAuthService>>,
                 id: Path<String>) -> Result<HttpResponse> {
    match service.get(&id.into_inner()) {
        Ok(ace) => Ok(
            match ace {
                None => HttpResponse::NotFound().finish(),
                Some(value) => HttpResponse::Ok().json(value.value())
            }),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[patch("/v1/keys/{id}")]
pub async fn update(service: Data<Arc<JwtAuthService>>,
                    id: Path<String>,
                    request: Json<JwtKeyUpdateCommand>) -> Result<HttpResponse> {
    match service.update(&id.into_inner(), request.0) {
        Ok(updated) => Ok(HttpResponse::Ok().json(updated)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[delete("/v1/keys/{id}")]
pub async fn delete(service: Data<Arc<JwtAuthService>>,
                    id: Path<String>) -> Result<HttpResponse> {
    match service.delete(&id.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}