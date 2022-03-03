use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, patch};
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json, Path};

use crate::models::access::WildcardPatternConnectionIdAccessEntry;
use crate::models::updatable::{StringSetCommand, WildcardPatternSetCommand};
use crate::services::updatable::UpdatableService;
use crate::WildcardPatternConnectionIdAccessEntryService;

#[post("/v1/access/patterns")]
pub async fn create(service: Data<Arc<WildcardPatternConnectionIdAccessEntryService>>,
                    request: Json<WildcardPatternConnectionIdAccessEntry>) -> Result<HttpResponse> {
    match service.create(request.0) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[get("/v1/access/patterns/{id}")]
pub async fn get(service: Data<Arc<WildcardPatternConnectionIdAccessEntryService>>,
                 client_id: Path<String>) -> Result<HttpResponse> {
    match service.get(&client_id.into_inner()) {
        Ok(ace) => Ok(
            match ace {
                None => HttpResponse::NotFound().finish(),
                Some(ace_value) => HttpResponse::Ok().json(ace_value.value())
            }),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[patch("/v1/access/patterns/{id}")]
pub async fn update(service: Data<Arc<WildcardPatternConnectionIdAccessEntryService>>,
                    client_id: Path<String>,
                    request: Json<WildcardPatternSetCommand>) -> Result<HttpResponse> {
    match service.update(&client_id.into_inner(), request.0) {
        Ok(updated) => Ok(HttpResponse::Ok().json(updated)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}

#[delete("/v1/access/patterns/{id}")]
pub async fn delete(service: Data<Arc<WildcardPatternConnectionIdAccessEntryService>>,
                    client_id: Path<String>) -> Result<HttpResponse> {
    match service.delete(&client_id.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}