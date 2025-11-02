use std::any::type_name;

use axum::{
    Json,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

use crate::database::DbErr;

pub trait IntoErrorResponse {
    fn type_name(&self) -> &'static str;

    fn into_error_response(self) -> Response;
}

trait AwsBasicError: std::error::Error {
    const STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;
}

/// Get the short version of a type name (Last portion only strips module prefix)
fn type_name_short<T>() -> &'static str {
    let full = type_name::<T>();
    match full.rsplit_once("::") {
        Some((_path, name)) => name,
        None => full,
    }
}

impl<A: AwsBasicError> IntoErrorResponse for A {
    fn type_name(&self) -> &'static str {
        type_name_short::<A>()
    }

    fn into_error_response(self) -> Response {
        simple_error_response(self.type_name(), self.to_string(), A::STATUS_CODE)
    }
}

#[derive(Debug, Error)]
#[error("The X.509 certificate or AWS access key ID provided does not exist in our records.")]
pub struct InvalidClientTokenId;

impl AwsBasicError for InvalidClientTokenId {
    const STATUS_CODE: StatusCode = StatusCode::FORBIDDEN;
}

#[derive(Debug, Error)]
#[error(
    "The request signature we calculated does not match the signature you provided. \
    Check your AWS Secret Access Key and signing method. Consult the service documentation for details."
)]
pub struct SignatureDoesNotMatch;

impl AwsBasicError for SignatureDoesNotMatch {
    const STATUS_CODE: StatusCode = StatusCode::FORBIDDEN;
}

#[derive(Debug, Error)]
#[error("Missing Authentication Token")]
pub struct MissingAuthenticationToken;

impl AwsBasicError for MissingAuthenticationToken {}

#[derive(Debug, Error)]
#[error("The request signature does not conform to AWS standards.")]
pub struct IncompleteSignature;

impl AwsBasicError for IncompleteSignature {}

#[derive(Debug, Error)]
#[error("A parameter value is not valid for the current state of the resource.")]
pub struct InvalidRequestException;

impl AwsBasicError for InvalidRequestException {}

#[derive(Debug, Error)]
#[error("The parameter name or value is invalid.")]
pub struct InvalidParameterException;

impl AwsBasicError for InvalidParameterException {}

#[derive(Debug, Error)]
#[error("Secrets Manager can't find the resource that you asked for.")]
pub struct ResourceNotFoundException;

impl AwsBasicError for ResourceNotFoundException {}

#[derive(Debug, Error)]
#[error("A resource with the ID you requested already exists.")]
pub struct ResourceExistsException;

impl AwsBasicError for ResourceExistsException {}

#[derive(Debug, Error)]
#[error("This operation is not implemented in this server")]
pub struct NotImplemented;

impl AwsBasicError for NotImplemented {}

#[derive(Debug, Error)]
#[error("An error occurred on the server side.")]
pub struct InternalServiceError;

impl AwsBasicError for InternalServiceError {}

#[derive(Debug, Error)]
pub enum AwsError {
    #[error(transparent)]
    InvalidClientTokenId(#[from] InvalidClientTokenId),

    #[error(transparent)]
    SignatureDoesNotMatch(#[from] SignatureDoesNotMatch),

    #[error(transparent)]
    MissingAuthenticationToken(#[from] MissingAuthenticationToken),

    #[error(transparent)]
    IncompleteSignature(#[from] IncompleteSignature),

    #[error(transparent)]
    InvalidRequestException(#[from] InvalidRequestException),

    #[error(transparent)]
    InvalidParameterException(#[from] InvalidParameterException),

    #[error(transparent)]
    ResourceNotFoundException(#[from] ResourceNotFoundException),

    #[error(transparent)]
    ResourceExistsException(#[from] ResourceExistsException),

    #[error(transparent)]
    NotImplemented(#[from] NotImplemented),

    #[error(transparent)]
    InternalServiceError(#[from] InternalServiceError),
}

// Database errors can be turned directly into [InternalServiceError]
impl From<DbErr> for AwsError {
    fn from(_value: DbErr) -> Self {
        InternalServiceError.into()
    }
}

impl IntoErrorResponse for AwsError {
    fn type_name(&self) -> &'static str {
        match self {
            AwsError::InvalidClientTokenId(error) => error.type_name(),
            AwsError::SignatureDoesNotMatch(error) => error.type_name(),
            AwsError::MissingAuthenticationToken(error) => error.type_name(),
            AwsError::IncompleteSignature(error) => error.type_name(),
            AwsError::InvalidRequestException(error) => error.type_name(),
            AwsError::InvalidParameterException(error) => error.type_name(),
            AwsError::ResourceNotFoundException(error) => error.type_name(),
            AwsError::ResourceExistsException(error) => error.type_name(),
            AwsError::NotImplemented(error) => error.type_name(),
            AwsError::InternalServiceError(error) => error.type_name(),
        }
    }

    fn into_error_response(self) -> Response {
        match self {
            AwsError::InvalidClientTokenId(error) => error.into_error_response(),
            AwsError::SignatureDoesNotMatch(error) => error.into_error_response(),
            AwsError::MissingAuthenticationToken(error) => error.into_error_response(),
            AwsError::IncompleteSignature(error) => error.into_error_response(),
            AwsError::InvalidRequestException(error) => error.into_error_response(),
            AwsError::InvalidParameterException(error) => error.into_error_response(),
            AwsError::ResourceNotFoundException(error) => error.into_error_response(),
            AwsError::ResourceExistsException(error) => error.into_error_response(),
            AwsError::NotImplemented(error) => error.into_error_response(),
            AwsError::InternalServiceError(error) => error.into_error_response(),
        }
    }
}

#[derive(Serialize)]
struct AwsErrorResponse<'a> {
    __type: &'a str,
    message: &'a str,
}

fn simple_error_response(
    __type: &'static str,
    message: impl AsRef<str>,
    status_code: StatusCode,
) -> Response {
    let mut response = (
        status_code,
        Json(AwsErrorResponse {
            __type,
            message: message.as_ref(),
        }),
    )
        .into_response();
    response
        .headers_mut()
        .insert("x-amzn-errortype", HeaderValue::from_static(__type));
    response
}
