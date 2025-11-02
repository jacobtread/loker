use crate::{
    handlers::error::{
        IncompleteSignature, InternalServiceError, IntoErrorResponse, InvalidClientTokenId,
        InvalidRequestException, MissingAuthenticationToken, SignatureDoesNotMatch,
    },
    utils::{
        aws_sig_v4::parse_auth_header,
        date::{chrono_to_system_time, parse_amz_date, parse_http_date},
    },
};
use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4::SigningParams,
};
use axum::{
    body::Body,
    http::{
        Request,
        header::{AUTHORIZATION, ToStrError},
    },
    response::Response,
};
use chrono::Utc;
use futures::future::BoxFuture;
use http_body_util::BodyExt;
use std::mem::swap;
use tower::{Layer, Service};

/// Middleware provider layer
#[derive(Clone)]
pub struct AwsSigV4AuthLayer {
    credentials: Credentials,
}

impl AwsSigV4AuthLayer {
    /// Create a new AWS SigV4 layer using the provided credentials
    pub fn new(credentials: Credentials) -> Self {
        Self { credentials }
    }
}

impl<S> Layer<S> for AwsSigV4AuthLayer {
    type Service = AwsSigV4AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AwsSigV4AuthMiddleware {
            inner,
            credentials: self.credentials.clone(),
        }
    }
}

/// Middleware structure
#[derive(Clone)]
pub struct AwsSigV4AuthMiddleware<S> {
    inner: S,
    credentials: Credentials,
}

impl<S> Service<Request<Body>> for AwsSigV4AuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let credential = self.credentials.clone();

        // Swap to ensure we get the service that was ready and not the cloned one
        swap(&mut inner, &mut self.inner);

        Box::pin(async move {
            let (parts, body) = req.into_parts();

            let authorization = match parts.headers.get(AUTHORIZATION) {
                Some(value) => match value.to_str() {
                    Ok(value) => value,
                    // Invalid auth header
                    Err(_) => {
                        return Ok(InvalidRequestException.into_error_response());
                    }
                },
                None => {
                    // Unauthorized missing header
                    return Ok(MissingAuthenticationToken.into_error_response());
                }
            };

            // Extract the AWS specific date header
            let amz_date = match parts.headers.get("x-amz-date") {
                Some(value) => {
                    let value = match value.to_str() {
                        Ok(value) => value,
                        Err(_) => {
                            // Date header is invalid
                            return Ok(InvalidRequestException.into_error_response());
                        }
                    };

                    let value = match parse_amz_date(value) {
                        Ok(value) => value,
                        Err(_) => {
                            // Date header is invalid
                            return Ok(InvalidRequestException.into_error_response());
                        }
                    };

                    Some(value)
                }
                None => None,
            };

            // Extract the generic Date header
            let http_date = match parts.headers.get("date") {
                Some(value) => {
                    let value = match value.to_str() {
                        Ok(value) => value,
                        Err(_) => {
                            // Date header is invalid
                            return Ok(InvalidRequestException.into_error_response());
                        }
                    };

                    let value = match parse_http_date(value) {
                        Ok(value) => value,
                        Err(_) => {
                            // Date header is invalid
                            return Ok(InvalidRequestException.into_error_response());
                        }
                    };

                    Some(value)
                }
                None => None,
            };

            let date = match amz_date.or(http_date) {
                Some(value) => value,
                None => {
                    // No date present on the request
                    return Ok(InvalidRequestException.into_error_response());
                }
            };

            let now = Utc::now();
            let time_diff_now = now.timestamp().saturating_sub(date.timestamp()).abs();
            if time_diff_now > 60 * 5 {
                // Request date is not within the expected 5 minute tolerance window
                // of the server time
                return Ok(InvalidRequestException.into_error_response());
            }

            let auth = match parse_auth_header(authorization) {
                Ok(value) => value,
                Err(_) => {
                    return Ok(IncompleteSignature.into_error_response());
                }
            };

            // Missing the aws4_request portion of the credential
            if auth.signing_scope.aws4_request != "aws4_request" {
                return Ok(IncompleteSignature.into_error_response());
            }

            if auth.signing_scope.access_key_id != credential.access_key_id() {
                // Invalid access key
                return Ok(InvalidClientTokenId.into_error_response());
            }

            let body = match body.collect().await {
                Ok(value) => value.to_bytes(),
                Err(_) => {
                    // Failed to ready body
                    return Ok(InvalidRequestException.into_error_response());
                }
            };

            // Convert request date into a [SystemTime] timestamp for AWS-SigV4
            let time = match chrono_to_system_time(date) {
                Some(value) => value,
                None => {
                    return Ok(InvalidRequestException.into_error_response());
                }
            };

            // Setup the signing settings
            let identity = credential.into();
            let signing_settings = SigningSettings::default();
            let signing_params = match SigningParams::builder()
                .identity(&identity)
                .region(auth.signing_scope.region)
                .name(auth.signing_scope.service)
                .time(time)
                .settings(signing_settings)
                .build()
            {
                Ok(value) => value.into(),
                Err(_error) => {
                    return Ok(InternalServiceError.into_error_response());
                }
            };

            // Collect request headers that were included in the signed request
            let headers =
                match parts
                    .headers
                    .iter()
                    .try_fold(Vec::new(), |mut headers, (name, value)| {
                        let name = name.as_str();
                        if auth.signed_headers.contains(&name) {
                            let value = value.to_str()?;
                            headers.push((name, value));
                        }

                        Ok::<_, ToStrError>(headers)
                    }) {
                    Ok(value) => value,
                    Err(_error) => {
                        return Ok(InvalidRequestException.into_error_response());
                    }
                };

            // Create the signable request
            let signable_request = match SignableRequest::new(
                parts.method.as_str(),
                parts.uri.to_string(),
                headers.into_iter(),
                SignableBody::Bytes(&body),
            ) {
                Ok(value) => value,
                Err(_error) => {
                    //
                    return Ok(InvalidRequestException.into_error_response());
                }
            };

            let (_signing_instructions, signature) = match sign(signable_request, &signing_params) {
                Ok(value) => value.into_parts(),
                Err(_error) => {
                    //
                    return Ok(InternalServiceError.into_error_response());
                }
            };

            if signature != auth.signature {
                // Verify failure, bad signature
                return Ok(SignatureDoesNotMatch.into_error_response());
            }

            // Re-create the body since we consumed the previous one
            let body = Body::from(body);

            let request = Request::from_parts(parts, body);

            inner.call(request).await
        })
    }
}
