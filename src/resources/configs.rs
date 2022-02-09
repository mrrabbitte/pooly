use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json};

use crate::services::connections::config::ConnectionConfigService;
use crate::models::connections::ConnectionConfig;

#[post("/v1/configs")]
pub async fn create(service: Data<Arc<ConnectionConfigService>>,
                    request: Json<ConnectionConfig>) -> Result<HttpResponse> {
    match service.put(request.0) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}