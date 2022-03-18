use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, HttpResponse, ResponseError};
use actix_web::body::EitherBody;
use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;

use crate::models::auth::api_key::InitializeApiKey;
use crate::models::errors::AuthError;

const AUTHORIZATION: &str = "Authorization";

pub struct InitializationGuard {

    api_key: InitializeApiKey

}

impl InitializationGuard {

    pub fn new(api_key: InitializeApiKey) -> InitializationGuard {
        InitializationGuard {
            api_key
        }
    }

}

impl<S, B> Transform<S, ServiceRequest> for InitializationGuard
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = InitializationGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(
            Ok(
                InitializationGuardMiddleware {
                    validator: Rc::new(ApiKeyValidator { api_key: self.api_key.clone() }),
                    service: Rc::new(service)
                }
            )
        )
    }
}

pub struct InitializationGuardMiddleware<S> {

    validator: Rc<ApiKeyValidator>,
    service: Rc<S>

}

impl<S, B> Service<ServiceRequest> for InitializationGuardMiddleware<S>
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

struct ApiKeyValidator {

    api_key: InitializeApiKey

}

impl ApiKeyValidator {

    fn validate(&self,
                req: &ServiceRequest) -> Result<(), AuthError> {
        match req.headers().get(AUTHORIZATION) {
            None => Err(AuthError::MissingAuthHeader),
            Some(auth_header_value) => {
                if self.api_key
                    .get_value()
                    .ne(auth_header_value.to_str().map_err(|_| AuthError::InvalidToken)?) {
                    return Err(AuthError::BadCredentials);
                }

                Ok(())
            }
        }
    }

}
