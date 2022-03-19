use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use actix_utils::future::{ready, Ready};
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use actix_web::body::EitherBody;
use actix_web::dev::Payload;
use actix_web::guard::{Guard, GuardContext};
use actix_web::http::StatusCode;
use actix_web::web::Data;

use crate::models::auth::roles::{AuthOutcome, Role, RoleToken};
use crate::models::errors::AuthError;
use crate::services::auth::jwt::JwtAuthService;

const AUTHORIZATION: &str = "Authorization";

pub struct RoleTokenGuard {

    role: Role

}

impl Guard for RoleTokenGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        let extensions = ctx.req_data();

        let role_token_maybe: Option<&RoleToken> = extensions.get();

        match role_token_maybe {
            Some(token) => self.role.eq(token.get_role()),
            None => false
        }
    }
}

impl FromRequest for RoleToken {
    type Error = AuthError;
    type Future = Ready<Result<RoleToken, AuthError>>;

    fn from_request(req: &HttpRequest,
                    _: &mut Payload) -> Self::Future {
        ready(
            extract_role_token(req)
        )
    }
}

#[inline]
fn extract_role_token(req: &HttpRequest) -> Result<RoleToken, AuthError> {
    let auth_service_maybe = req.app_data::<Data<Arc<JwtAuthService>>>();

    if auth_service_maybe.is_none() {
        return Err(AuthError::MissingAuthService);
    }

    let auth_service = auth_service_maybe.unwrap();

    let auth_header_value_maybe = req.headers().get(AUTHORIZATION);

    if auth_header_value_maybe.is_none() {
        return Err(AuthError::MissingAuthHeader);
    }

    let auth_header_value = auth_header_value_maybe.unwrap();

    let auth_header = auth_header_value.to_str()
        .map_err(|_| AuthError::InvalidHeader)?;

    Ok(auth_service.extract(auth_header)?)
}
