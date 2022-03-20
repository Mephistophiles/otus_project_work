use std::{pin::Pin, sync::Arc};
use tokio::sync::Mutex;

use actix_web::{
    dev::Payload, error::InternalError, http::StatusCode, web, FromRequest, HttpRequest,
};
use futures::{future, Future, FutureExt};

use crate::services::jwt::{JWTToken, Jwt};

impl FromRequest for JWTToken {
    type Error = InternalError<&'static str>;

    // we return a Future that is either
    // - immediately ready (on a bad request with a missing or malformed Authorization header)
    type Future = future::Either<
        future::Ready<Result<Self, Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + 'static>>,
    >;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // get request headers
        let headers = req.headers();

        // check that Authorization header exists
        let maybe_auth = headers.get("Authorization");
        if maybe_auth.is_none() {
            return future::err(InternalError::new(
                "missing Authorization header",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check Authorization header is valid utf-8
        let auth_config = maybe_auth.unwrap().to_str();
        if auth_config.is_err() {
            return future::err(InternalError::new(
                "malformed Authorization header",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check Authorization header specifies some authorization strategy
        let mut auth_config_parts = auth_config.unwrap().split_ascii_whitespace();
        let maybe_auth_type = auth_config_parts.next();
        if maybe_auth_type.is_none() {
            return future::err(InternalError::new(
                "missing Authorization type",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check that authorization strategy is using a bearer token
        let auth_type = maybe_auth_type.unwrap();
        if auth_type != "Bearer" {
            return future::err(InternalError::new(
                "unsupported Authorization type",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        // check that bearer token is present
        let maybe_token_id = auth_config_parts.next();
        if maybe_token_id.is_none() {
            return future::err(InternalError::new(
                "missing Bearer token",
                StatusCode::BAD_REQUEST,
            ))
            .left_future();
        }

        let jwt = req.app_data::<web::Data<Arc<Mutex<Jwt>>>>();
        if jwt.is_none() {
            return future::err(InternalError::new(
                "internal error",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
            .left_future();
        }

        // clone these so that we can return an impl Future + 'static
        let token_id = maybe_token_id.unwrap().to_owned();
        let jwt = jwt.unwrap().clone();

        async move {
            jwt.lock()
                .await
                .verify_token(token_id)
                .ok_or_else(|| InternalError::new("invalid Bearer token", StatusCode::UNAUTHORIZED))
        }
        .boxed_local()
        .right_future()
    }
}
