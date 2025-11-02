use crate::{
    database::DbPool,
    handlers::{
        batch_get_secret_value::BatchGetSecretValueHandler,
        create_secret::CreateSecretHandler,
        delete_secret::DeleteSecretHandler,
        describe_secret::DescribeSecretHandler,
        error::{AwsError, IntoErrorResponse},
        get_random_password::GetRandomPasswordHandler,
        get_secret_value::GetSecretValueHandler,
        list_secret_version_ids::ListSecretVersionIdsHandler,
        list_secrets::ListSecretsHandler,
        put_secret_value::PutSecretValueHandler,
        restore_secret::RestoreSecretHandler,
        tag_resource::TagResourceHandler,
        untag_resource::UntagResourceHandler,
        update_secret::UpdateSecretHandler,
        update_secret_version_stage::UpdateSecretVersionStageHandler,
    },
};
use axum::{
    Json,
    body::Body,
    http::Request,
    response::{IntoResponse, Response},
};
use error::{
    InternalServiceError, InvalidParameterException, InvalidRequestException, NotImplemented,
};
use futures::future::BoxFuture;
use garde::Validate;
use http_body_util::BodyExt;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, convert::Infallible, sync::Arc, task::Poll};
use tower::Service;

pub(crate) mod error;
pub(crate) mod models;

mod batch_get_secret_value;
mod create_secret;
mod delete_secret;
mod describe_secret;
mod get_random_password;
mod get_secret_value;
mod list_secret_version_ids;
mod list_secrets;
mod put_secret_value;
mod restore_secret;
mod tag_resource;
mod untag_resource;
mod update_secret;
mod update_secret_version_stage;

pub fn create_handlers() -> HandlerRouter {
    HandlerRouter::default()
        .add_handler("secretsmanager.CreateSecret", CreateSecretHandler)
        .add_handler("secretsmanager.DeleteSecret", DeleteSecretHandler)
        .add_handler("secretsmanager.DescribeSecret", DescribeSecretHandler)
        .add_handler("secretsmanager.GetSecretValue", GetSecretValueHandler)
        .add_handler("secretsmanager.ListSecrets", ListSecretsHandler)
        .add_handler("secretsmanager.PutSecretValue", PutSecretValueHandler)
        .add_handler("secretsmanager.UpdateSecret", UpdateSecretHandler)
        .add_handler("secretsmanager.RestoreSecret", RestoreSecretHandler)
        .add_handler("secretsmanager.TagResource", TagResourceHandler)
        .add_handler("secretsmanager.UntagResource", UntagResourceHandler)
        .add_handler("secretsmanager.GetRandomPassword", GetRandomPasswordHandler)
        .add_handler(
            "secretsmanager.ListSecretVersionIds",
            ListSecretVersionIdsHandler,
        )
        .add_handler(
            "secretsmanager.UpdateSecretVersionStage",
            UpdateSecretVersionStageHandler,
        )
        .add_handler(
            "secretsmanager.BatchGetSecretValue",
            BatchGetSecretValueHandler,
        )
}

#[derive(Default)]
pub struct HandlerRouter {
    handlers: HashMap<String, Box<dyn ErasedHandler>>,
}

impl HandlerRouter {
    fn add_handler<H: Handler>(mut self, target: &str, handler: H) -> Self {
        self.handlers.insert(
            target.to_string(),
            Box::new(HandlerBase { _handler: handler }),
        );
        self
    }

    fn get_handler(&self, target: &str) -> Option<&dyn ErasedHandler> {
        self.handlers.get(target).map(|value| value.as_ref())
    }

    pub fn into_service(self) -> HandlerRouterService {
        HandlerRouterService {
            router: Arc::new(self),
        }
    }
}

/// Service that handles routing AWS handler requests
#[derive(Clone)]
pub struct HandlerRouterService {
    router: Arc<HandlerRouter>,
}

impl Service<Request<Body>> for HandlerRouterService {
    type Response = Response;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let handlers = self.router.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();

            let db = parts
                .extensions
                .get::<DbPool>()
                .expect("handler router service missing db pool");

            let target = match parts
                .headers
                .get("x-amz-target")
                .and_then(|v| v.to_str().ok())
            {
                Some(value) => value,
                None => {
                    return Ok(InvalidRequestException.into_error_response());
                }
            };

            let handler = handlers.get_handler(target);

            let body = match body.collect().await {
                Ok(value) => value.to_bytes(),
                Err(error) => {
                    tracing::error!(?error, "failed to collect bytes");
                    return Ok(InternalServiceError.into_error_response());
                }
            };

            Ok(match handler {
                Some(value) => value.handle(db, &body).await,
                None => NotImplemented.into_error_response(),
            })
        })
    }
}

/// Handler for handling a specific request
pub trait Handler: Send + Sync + 'static {
    type Request: DeserializeOwned + Validate<Context = ()> + Send + 'static;
    type Response: Serialize + Send + 'static;

    fn handle<'d>(
        db: &'d DbPool,
        request: Self::Request,
    ) -> impl Future<Output = Result<Self::Response, AwsError>> + Send + 'd;
}

/// Associated type erased [Handler] that takes a generic request and provides
/// a generic response
pub trait ErasedHandler: Send + Sync + 'static {
    fn handle<'r>(&self, db: &'r DbPool, request: &'r [u8]) -> BoxFuture<'r, Response>;
}

/// Handler that takes care of the process of deserializing the request
/// type and serializing the response type to create a generic [ErasedHandler]
pub struct HandlerBase<H: Handler> {
    _handler: H,
}

impl<H: Handler> ErasedHandler for HandlerBase<H> {
    fn handle<'r>(&self, db: &'r DbPool, request: &'r [u8]) -> BoxFuture<'r, Response> {
        Box::pin(async move {
            let request: H::Request = match serde_json::from_slice(request) {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to parse request");
                    return InvalidRequestException.into_error_response();
                }
            };

            if let Err(_error) = request.validate() {
                // TODO: Share the error message with the user
                return InvalidParameterException.into_error_response();
            }

            match H::handle(db, request).await {
                Ok(response) => Json(response).into_response(),
                Err(error) => error.into_error_response(),
            }
        })
    }
}
