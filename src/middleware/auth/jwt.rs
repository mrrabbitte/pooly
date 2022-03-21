use std::fmt;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError};
use actix_web::body::EitherBody;
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;

use crate::models::auth::roles::{AdminToken, AuthOutcome, ClientServiceToken, Role, RoleToken};
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
                    extractor: Rc::new(RoleTokenExtractor { role: self.role.clone() })
                }
            )
        )
    }

}

pub struct AuthGuardMiddleware<S> {

    service: Rc<S>,
    extractor: Rc<RoleTokenExtractor>

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
        let extractor  = Rc::clone(&self.extractor);

        async move {
            match extractor.extract(&req) {
                Ok(token) => {
                    req.extensions_mut().insert(token);

                    service.call(req).await.map(|res| res.map_into_left_body())
                },
                Err(err) => Ok(req.error_response(err).map_into_right_body())
            }
        }.boxed_local()
    }

}

#[derive(Clone, Copy)]
struct RoleTokenExtractor {

    role: Role

}

impl RoleTokenExtractor {

    fn extract(self,
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

        let role_token = auth_service.extract(auth_header)?;

        if role_token.get_role().ne(&self.role) {
            return Err(AuthError::Forbidden);
        }

        Ok(role_token)
    }
}

impl FromRequest for AdminToken {
    type Error = AuthError;
    type Future = Ready<Result<AdminToken, AuthError>>;

    fn from_request(req: &HttpRequest,
                    _: &mut Payload) -> Self::Future {
        ready(
            match req.extensions().get::<RoleToken>() {
                Some(RoleToken::Admin(token, _)) => Ok(token.clone()),
                _ => Err(AuthError::Forbidden)
            }
        )
    }
}

impl FromRequest for ClientServiceToken {
    type Error = AuthError;
    type Future = Ready<Result<ClientServiceToken, AuthError>>;

    fn from_request(req: &HttpRequest,
                    _: &mut Payload) -> Self::Future {
        ready(
            match req.extensions().get::<RoleToken>() {
                Some(RoleToken::ClientService(token, _)) => Ok(token.clone()),
                _ => Err(AuthError::Forbidden)
            }
        )
    }
}
