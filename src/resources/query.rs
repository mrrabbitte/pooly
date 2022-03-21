use std::sync::Arc;

use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::Data;
use uuid::Uuid;
use crate::models::auth::roles::{ClientServiceToken};

use crate::models::payloads::{QueryRequest, TxBulkQueryRequest};
use crate::models::responses::ResponseWithCode;
use crate::services::queries::QueryService;

#[post("/v1/bulk")]
pub async fn bulk(service: Data<Arc<QueryService>>,
                  request: ProtoBuf<TxBulkQueryRequest>,
                  token: ClientServiceToken) -> Result<HttpResponse> {
    let correlation_id = Uuid::new_v4().to_string();

    let response = service.bulk_tx(token.get_client_id(), &request.0, &correlation_id).await;

    build_response(response)
}

#[post("/v1/query")]
pub async fn query(service: Data<Arc<QueryService>>,
                   request: ProtoBuf<QueryRequest>,
                   token: ClientServiceToken) -> Result<HttpResponse> {
    let correlation_id = Uuid::new_v4().to_string();

    let response = service.query(token.get_client_id(), &request.0, &correlation_id).await;

    build_response(response)
}

fn build_response<T: prost::Message>(response: ResponseWithCode<T>) -> Result<HttpResponse> {
    HttpResponse::build(
        StatusCode::from_u16(response.1)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .protobuf(response.0)
}
