use actix_utils::future::{ready, Ready};
use actix_web::http::header::{self, HeaderMap};
use actix_web::http::Uri;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::{HttpMessage, ResponseError};
use async_trait::async_trait;
use futures_util::future::LocalBoxFuture;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::rc::Rc;

#[async_trait]
pub trait TokenChecker<T>
where
    T: Sized,
{
    /// This function will return option
    /// if the request token valid return
    /// Some with sized data to pass to the router
    /// otherwise retun None to response with status code 401
    /// Unauthorized
    ///
    /// This function returns the verifyed user ID
    async fn get_user_id(
        &self,
        req_addr: SocketAddr,
        headers: HeaderMap,
        uri: Uri,
        request_token: &str,
    ) -> Result<T, Box<dyn ResponseError>>
    where
        Self: Sized;

    /// Error when no token were sent and the token is required
    async fn token_not_found_error(&self) -> Box<dyn ResponseError>;
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.

#[derive(Clone, Default)]
pub struct TokenAuth<F, Type> {
    authorization_header_required: bool,
    finder: F,
    phantom_type: PhantomData<Type>,
}

impl<F, Type> TokenAuth<F, Type>
where
    F: TokenChecker<Type>,
    Type: Sized,
{
    /// Construct `TokenAuth` middleware.
    pub fn new(finder: F, authorization_header_required: bool) -> Self {
        Self {
            authorization_header_required,
            finder,
            phantom_type: PhantomData,
        }
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B, F, T> Transform<S, ServiceRequest> for TokenAuth<F, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    F: TokenChecker<T> + Clone + 'static,
    T: Sized + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TokenAuthMiddleware<S, F, T>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenAuthMiddleware {
            service: Rc::new(service),
            token_finder: self.finder.clone(),
            authorization_header_required: self.authorization_header_required,
            phantom_type: PhantomData,
        }))
    }
}

pub struct TokenAuthMiddleware<S, F, Type> {
    service: Rc<S>,
    token_finder: F,
    authorization_header_required: bool,
    phantom_type: PhantomData<Type>,
}

impl<S, B, F, Type> Service<ServiceRequest> for TokenAuthMiddleware<S, F, Type>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    F: TokenChecker<Type> + Clone + 'static,
    Type: Sized + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let token_finder = self.token_finder.clone();
        let header_required = self.authorization_header_required;

        Box::pin(async move {
            match req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|token| token.to_str().ok())
            {
                Some(token) => match token_finder
                    .get_user_id(
                        req.request().peer_addr().unwrap(),
                        req.request().headers().clone(),
                        req.request().uri().clone(),
                        token,
                    )
                    .await
                {
                    Ok(data) => {
                        req.extensions_mut().insert(data);
                        let res = service.call(req).await?;

                        Ok(res)
                    }

                    Err(error) => Err(Error::from(error)),
                },
                None => {
                    if header_required {
                        return Err(Error::from(token_finder.token_not_found_error().await));
                    }

                    let res = service.call(req).await?;

                    Ok(res)
                }
            }
        })
    }
}
