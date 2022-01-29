use std::sync::Arc;

use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json};
use uuid::Uuid;

use crate::ConnectionConfigService;
use crate::models::connections::ConnectionConfig;
use crate::models::payloads::QueryRequest;
use crate::services::queries::QueryService;

#[post("/v1/configs")]
pub async fn create(service: Data<Arc<ConnectionConfigService>>,
                    request: Json<ConnectionConfig>) -> Result<HttpResponse> {
    match service.put(request.0) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(HttpResponse::InternalServerError().json(err))
    }
}