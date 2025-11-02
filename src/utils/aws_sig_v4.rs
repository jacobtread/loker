use thiserror::Error;

/// Parsed AWS SigV4 header
pub struct AwsSigV4Auth<'a> {
    pub signing_scope: SigningScope<'a>,
    pub signed_headers: Vec<&'a str>,
    pub signature: &'a str,
}

#[derive(Debug, Error)]
pub enum AuthHeaderError {
    #[error("invalid header parts")]
    InvalidHeader,

    #[error("unsupported algorithm, this implementation only supports AWS4-HMAC-SHA256")]
    UnsupportedAlgorithm,

    #[error("invalid key value pair")]
    InvalidKeyValue,

    #[error("missing Credential")]
    MissingCredential,

    #[error("missing SignedHeaders")]
    MissingSignedHeaders,

    #[error("missing Signature")]
    MissingSignature,

    #[error("invalid scope")]
    InvalidScope,
}

/// Parse the Authorization header value to extract the AWS SigV4 data
pub fn parse_auth_header<'a>(header: &'a str) -> Result<AwsSigV4Auth<'a>, AuthHeaderError> {
    let mut parts = header.splitn(2, ' ');

    // AWS4-HMAC-SHA256
    let algorithm = parts
        .next()
        .ok_or(AuthHeaderError::InvalidHeader)?
        .to_string();

    if algorithm != "AWS4-HMAC-SHA256" {
        return Err(AuthHeaderError::UnsupportedAlgorithm);
    }

    let kv_string = parts.next().ok_or(AuthHeaderError::InvalidHeader)?;

    let mut credential: Option<&str> = None;
    let mut signed_headers: Option<&str> = None;
    let mut signature: Option<&str> = None;

    for kv in kv_string.split(", ") {
        let mut split = kv.splitn(2, '=');
        let key = split.next().ok_or(AuthHeaderError::InvalidKeyValue)?;
        let value = split.next().ok_or(AuthHeaderError::InvalidKeyValue)?;
        match key {
            "Credential" => {
                credential = Some(value);
            }
            "SignedHeaders" => {
                signed_headers = Some(value);
            }
            "Signature" => {
                signature = Some(value);
            }

            _ => {}
        }
    }

    let credential = credential.ok_or(AuthHeaderError::MissingCredential)?;
    let signed_headers = signed_headers.ok_or(AuthHeaderError::MissingSignedHeaders)?;
    let signature = signature.ok_or(AuthHeaderError::MissingSignature)?;

    let signed_headers: Vec<&str> = signed_headers.split(';').collect();

    let signing_scope = parse_signing_scope(credential).ok_or(AuthHeaderError::InvalidScope)?;

    Ok(AwsSigV4Auth {
        signing_scope,
        signed_headers,
        signature,
    })
}

pub struct SigningScope<'a> {
    pub access_key_id: &'a str,
    #[allow(unused)]
    pub date_yyyymmdd: &'a str,
    pub region: &'a str,
    pub service: &'a str,
    pub aws4_request: &'a str,
}

pub fn parse_signing_scope(value: &str) -> Option<SigningScope<'_>> {
    let mut parts = value.split('/');
    let access_key_id = parts.next()?;
    let date_yyyymmdd = parts.next()?;
    let region = parts.next()?;
    let service = parts.next()?;
    let aws4_request = parts.next()?;

    Some(SigningScope {
        access_key_id,
        date_yyyymmdd,
        region,
        service,
        aws4_request,
    })
}
