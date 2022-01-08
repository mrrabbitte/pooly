use std::sync::Arc;

use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::Data;

use crate::queries::QueryService;
use crate::values::payloads::QueryRequest;

#[post("/query")]
pub async fn query(service: Data<Arc<QueryService>>,
                   request: ProtoBuf<QueryRequest>) -> Result<HttpResponse> {
    HttpResponse::Ok().protobuf(service.query(&request.0).await)
}