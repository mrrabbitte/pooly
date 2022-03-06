use std::detect::__is_feature_detected::sha;
use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json};

use crate::models::secrets::MasterKeySharePayload;
use crate::services::secrets::shares::MasterKeySharesService;

#[post("/v1/secrets/shares")]
pub async fn add_share(service: Data<Arc<MasterKeySharesService>>,
                       request: Json<MasterKeySharePayload>) -> Result<HttpResponse> {
    match request.0.try_into() {
        Ok(share) => {
            match service.add(share) {
                Ok(()) => Ok(HttpResponse::Ok().finish()),
                Err(_) => Ok(HttpResponse::BadRequest().finish())
            }
        },
        Err(_) => Ok(HttpResponse::BadRequest().finish())
    }
}

#[post("/v1/secrets/clear")]
pub async fn clear_shares(service: Data<Arc<MasterKeySharesService>>) -> Result<HttpResponse> {
    service.clear();

    Ok(HttpResponse::Ok().finish())
}