use std::fmt;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, HttpResponse, ResponseError};
use actix_web::body::EitherBody;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;

use crate::models::auth::roles::{AuthOutcome, Role, RoleToken};
use crate::models::errors::AuthError;
use crate::services::auth::jwt::JwtAuthService;

const AUTHORIZATION: &str = "Authorization";

pub struct AuthGuard {

    role: Role

}

impl AuthGuard {

    pub fn admin() -> AuthGuard {
        AuthGuard {
            role: Role::Admin
        }
    }

    pub fn client() -> AuthGuard {
        AuthGuard {
            role: Role::ClientService
        }
    }

}

impl<S, B> Transform<S, ServiceRequest> for AuthGuard
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(
            Ok(
                AuthGuardMiddleware {
                    service: Rc::new(service),
                    validator: Rc::new(RequestValidator { role: self.role.clone() })
                }
            )
        )
    }
}

pub struct AuthGuardMiddleware<S> {

    service: Rc<S>,
    validator: Rc<RequestValidator>

}

impl<S, B> Service<ServiceRequest> for AuthGuardMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let validator = Rc::clone(&self.validator);

        async move {
            match validator.validate(&req) {
                Ok(_) => service.call(req).await.map(|res| res.map_into_left_body()),
                Err(err) => Ok(req.error_response(err).map_into_right_body())
            }
        }.boxed_local()
    }
}

#[derive(Clone)]
struct RequestValidator {

    role: Role

}

impl RequestValidator {

    fn validate(&self,
                req: &ServiceRequest) -> Result<RoleToken, AuthError> {
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

        Ok(auth_service.validate_and_extract(auth_header, &self.role)?)
    }

}
