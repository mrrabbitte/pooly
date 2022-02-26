
use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_httpauth::extractors::bearer::BearerAuth;


pub async fn validate(req: ServiceRequest, token: BearerAuth) -> Result<ServiceRequest, Error> {
 Ok(req)
}