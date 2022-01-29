use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::post;
use actix_web::Result;
use actix_web::web::{Data, Json};

use crate::MasterKeySharesService;
use crate::models::secrets::{MasterKeyShare, MasterKeySharePayload};

#[post("/v1/shares")]
pub async fn add_share(service: Data<Arc<MasterKeySharesService>>,
                       request: Json<MasterKeySharePayload>) -> Result<HttpResponse> {


    match request.0.try_into() {
        Ok(share) => {
            service.add(share);

            Ok(HttpResponse::Ok().finish())
        },
        Err(_) => Ok(HttpResponse::BadRequest().finish())
    }
}