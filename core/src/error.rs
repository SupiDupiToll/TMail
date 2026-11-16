//! Error types for the headless core.
//!
//! Errors never carry message bodies, tokens, passwords or key material.
//! `Display` is safe to show to a user and to log.

use tuta_sdk::login::LoginError;
use tuta_sdk::ApiCallError;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("not signed in")]
    NotSignedIn,
    #[error("this account needs two-factor authentication")]
    SecondFactorRequired,
    #[error("this account uses a second factor TMail cannot handle yet (only TOTP is supported)")]
    UnsupportedSecondFactor,
    #[error("the saved session is no longer valid")]
    SessionInvalid,
    #[error("Tuta rejected this client version; update TMail")]
    ClientRejected,
    #[error("Tuta is temporarily unavailable; the saved session was kept")]
    TemporarilyUnavailable,
    #[error("Tuta is rate-limiting requests; try again shortly")]
    RateLimited,
    #[error("sign-in failed: check the address and password")]
    BadCredentials,
    #[error("the authenticator code was not accepted; try the next one")]
    BadTotp,
    #[error("the OS keyring is unavailable")]
    KeyringUnavailable,
    #[error("the encrypted cache could not be opened")]
    CacheUnavailable,
    #[error("the local cache is locked")]
    CacheLocked,
    #[error("the requested item was not found")]
    NotFound,
    #[error("the server returned data in an unexpected shape: {0}")]
    Protocol(String),
    #[error("network request failed: {0}")]
    Network(String),
    #[error("Tuta API error: {0}")]
    Api(String),
    #[error("storage error")]
    Storage,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("operation cancelled")]
    Cancelled,
}

impl CoreError {
    /// Whether an interactive sign-in (rather than a retry) fixes this.
    #[must_use]
    pub fn needs_reauth(&self) -> bool {
        matches!(
            self,
            Self::SessionInvalid | Self::BadCredentials | Self::NotSignedIn
        )
    }

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotSignedIn => "not_signed_in",
            Self::SecondFactorRequired => "second_factor_required",
            Self::UnsupportedSecondFactor => "unsupported_second_factor",
            Self::SessionInvalid => "session_invalid",
            Self::ClientRejected => "client_rejected",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
            Self::RateLimited => "rate_limited",
            Self::BadCredentials => "bad_credentials",
            Self::BadTotp => "bad_totp",
            Self::KeyringUnavailable => "keyring_unavailable",
            Self::CacheUnavailable | Self::CacheLocked => "cache_unavailable",
            Self::NotFound => "not_found",
            Self::Protocol(_) => "protocol",
            Self::Network(_) => "network",
            Self::Api(_) => "api",
            Self::Storage => "storage",
            Self::InvalidInput(_) => "invalid_input",
            Self::Cancelled => "cancelled",
        }
    }
}

impl From<ApiCallError> for CoreError {
    fn from(value: ApiCallError) -> Self {
        match &value {
            ApiCallError::RestClient { .. } => Self::Network("connection failed".into()),
            ApiCallError::ServerResponseError { source } => match source {
                tuta_sdk::rest_error::HttpError::NotAuthenticatedError
                | tuta_sdk::rest_error::HttpError::SessionExpiredError
                | tuta_sdk::rest_error::HttpError::AccessExpiredError
                | tuta_sdk::rest_error::HttpError::AccessDeactivatedError
                | tuta_sdk::rest_error::HttpError::AccessBlockedError => Self::SessionInvalid,
                tuta_sdk::rest_error::HttpError::InvalidSoftwareVersionError => {
                    Self::ClientRejected
                }
                tuta_sdk::rest_error::HttpError::NotFoundError => Self::NotFound,
                tuta_sdk::rest_error::HttpError::TooManyRequestsError { .. } => Self::RateLimited,
                tuta_sdk::rest_error::HttpError::ConnectionError
                | tuta_sdk::rest_error::HttpError::RequestTimeoutError
                | tuta_sdk::rest_error::HttpError::InternalServerError
                | tuta_sdk::rest_error::HttpError::BadGatewayError
                | tuta_sdk::rest_error::HttpError::ServiceUnavailableError { .. } => {
                    Self::TemporarilyUnavailable
                }
                other => Self::Api(other.to_string()),
            },
            ApiCallError::InternalSdkError { error_message } => {
                Self::Protocol(format!("SDK internal error: {error_message}"))
            }
        }
    }
}

impl From<LoginError> for CoreError {
    fn from(value: LoginError) -> Self {
        match &value {
            LoginError::InvalidPassphrase { .. } | LoginError::InvalidKey { .. } => {
                Self::BadCredentials
            }
            LoginError::InvalidAccessToken { .. } => Self::SessionInvalid,
            LoginError::ApiCall { source } => match source {
                ApiCallError::ServerResponseError { source } => match source {
                    tuta_sdk::rest_error::HttpError::NotAuthenticatedError
                    | tuta_sdk::rest_error::HttpError::SessionExpiredError
                    | tuta_sdk::rest_error::HttpError::AccessExpiredError => Self::SessionInvalid,
                    tuta_sdk::rest_error::HttpError::InvalidSoftwareVersionError => {
                        Self::ClientRejected
                    }
                    tuta_sdk::rest_error::HttpError::TooManyRequestsError { .. } => {
                        Self::RateLimited
                    }
                    _ => Self::Api(source.to_string()),
                },
                ApiCallError::RestClient { .. } => Self::TemporarilyUnavailable,
                ApiCallError::InternalSdkError { .. } => {
                    Self::Protocol("login failed inside the SDK".into())
                }
            },
        }
    }
}

impl From<tuta_sdk::bindings::rest_client::RestClientError> for CoreError {
    fn from(_: tuta_sdk::bindings::rest_client::RestClientError) -> Self {
        Self::Network("connection setup failed".into())
    }
}

impl From<rusqlite::Error> for CoreError {
    fn from(value: rusqlite::Error) -> Self {
        log::error!("sqlite error: {value}");
        Self::Storage
    }
}

impl From<std::io::Error> for CoreError {
    fn from(value: std::io::Error) -> Self {
        log::error!("io error: {}", value.kind());
        Self::Storage
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(value: serde_json::Error) -> Self {
        log::error!("json error at line {}", value.line());
        Self::Protocol(format!("invalid JSON at line {}", value.line()))
    }
}

pub type Result<T, E = CoreError> = std::result::Result<T, E>;
