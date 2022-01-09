use std::sync::Arc;

use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::Data;

use crate::models::payloads::QueryRequest;
use crate::queries::QueryService;

#[post("/query")]
pub async fn query(service: Data<Arc<QueryService>>,
                   request: ProtoBuf<QueryRequest>) -> Result<HttpResponse> {
    let result = service.query(&request.0).await;

    match result {
        Ok(response) => HttpResponse::Ok().protobuf(response),
        Err(_) => Result::Ok(HttpResponse::InternalServerError().finish())
    }
}
