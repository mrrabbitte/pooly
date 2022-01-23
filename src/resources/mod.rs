use std::sync::Arc;

use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::Data;
use uuid::Uuid;

use crate::models::payloads::QueryRequest;
use crate::services::queries::QueryService;

#[post("/v1/query")]
pub async fn query(service: Data<Arc<QueryService>>,
                   request: ProtoBuf<QueryRequest>) -> Result<HttpResponse> {
    let correlation_id = Uuid::new_v4().to_string();

    let response = service.query(&request.0, &correlation_id).await;

    HttpResponse::Ok().protobuf(response)
}
