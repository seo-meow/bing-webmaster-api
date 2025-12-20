use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Bing API error codes
///
/// This enum represents all possible error codes that can be returned by the Bing Webmaster API.
/// Each variant corresponds to a specific error condition documented in the API specification.
///
/// Reference: Microsoft.Bing.Webmaster.Api.Interfaces.ApiErrorCode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum BingErrorCode {
    /// No error
    None = 0,
    /// Internal server error
    InternalError = 1,
    /// Unknown error occurred
    UnknownError = 2,
    /// Invalid or missing API key
    InvalidApiKey = 3,
    /// User request rate limit exceeded
    ThrottleUser = 4,
    /// Host request rate limit exceeded
    ThrottleHost = 5,
    /// User has been blocked
    UserBlocked = 6,
    /// URL format is invalid
    InvalidUrl = 7,
    /// Request parameter is invalid
    InvalidParameter = 8,
    /// Too many sites associated with this account
    TooManySites = 9,
    /// User not found
    UserNotFound = 10,
    /// Requested resource not found
    NotFound = 11,
    /// Resource already exists
    AlreadyExists = 12,
    /// Operation not allowed
    NotAllowed = 13,
    /// User not authorized for this operation
    NotAuthorized = 14,
    /// Resource in unexpected state
    UnexpectedState = 15,
    /// API method is deprecated
    Deprecated = 16,
}

impl BingErrorCode {
    /// Create from integer value
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::InternalError),
            2 => Some(Self::UnknownError),
            3 => Some(Self::InvalidApiKey),
            4 => Some(Self::ThrottleUser),
            5 => Some(Self::ThrottleHost),
            6 => Some(Self::UserBlocked),
            7 => Some(Self::InvalidUrl),
            8 => Some(Self::InvalidParameter),
            9 => Some(Self::TooManySites),
            10 => Some(Self::UserNotFound),
            11 => Some(Self::NotFound),
            12 => Some(Self::AlreadyExists),
            13 => Some(Self::NotAllowed),
            14 => Some(Self::NotAuthorized),
            15 => Some(Self::UnexpectedState),
            16 => Some(Self::Deprecated),
            _ => None,
        }
    }

    /// Get integer value
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for BingErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::None => "None",
            Self::InternalError => "InternalError",
            Self::UnknownError => "UnknownError",
            Self::InvalidApiKey => "InvalidApiKey",
            Self::ThrottleUser => "ThrottleUser",
            Self::ThrottleHost => "ThrottleHost",
            Self::UserBlocked => "UserBlocked",
            Self::InvalidUrl => "InvalidUrl",
            Self::InvalidParameter => "InvalidParameter",
            Self::TooManySites => "TooManySites",
            Self::UserNotFound => "UserNotFound",
            Self::NotFound => "NotFound",
            Self::AlreadyExists => "AlreadyExists",
            Self::NotAllowed => "NotAllowed",
            Self::NotAuthorized => "NotAuthorized",
            Self::UnexpectedState => "UnexpectedState",
            Self::Deprecated => "Deprecated",
        };
        write!(f, "{}", name)
    }
}

/// Internal wrapper for error code deserialization
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ErrorCodeWrapper {
    /// Structured error code
    Structured { value: i32 },
    /// Raw integer value
    Raw(i32),
}

impl ErrorCodeWrapper {
    fn value(&self) -> i32 {
        match self {
            Self::Structured { value } => *value,
            Self::Raw(value) => *value,
        }
    }
}

/// Internal structure for deserializing Bing API error responses
///
/// Reference: Microsoft.Bing.Webmaster.Api.Interfaces.ApiFault
#[derive(Debug, Clone, Deserialize)]
struct ApiErrorResponse {
    #[serde(rename = "ErrorCode")]
    error_code: ErrorCodeWrapper,
    #[serde(rename = "Message")]
    message: String,
}

/// Errors that can occur when interacting with the Bing Webmaster API
#[derive(Debug, Error)]
pub enum WebmasterApiError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Middleware request failed
    #[error("Middleware request failed: {0}")]
    MiddlewareHttpError(#[from] reqwest_middleware::Error),

    /// Failed to parse response
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// API returned a structured error
    #[error("API error ({error_code_raw}): {message}")]
    ApiError {
        /// HTTP status code (if available)
        status: Option<u16>,
        /// Error code enum (if recognized)
        error_code: Option<BingErrorCode>,
        /// Raw error code value
        error_code_raw: i32,
        /// Error message from the API
        message: String,
    },

    /// HTTP status error without structured API response
    #[error("HTTP {status}: {message}")]
    HttpStatusError {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
        /// Optional response body
        response_body: Option<String>,
    },

    /// Invalid API response format
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// Authentication failed
    #[error("Authentication failed: missing or invalid API key")]
    AuthenticationError,

    /// Other errors (for anyhow integration)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl WebmasterApiError {
    /// Create an API error from error code and message
    pub fn api_error(error_code_raw: i32, message: String, status: Option<u16>) -> Self {
        let error_code = BingErrorCode::from_i32(error_code_raw);
        Self::ApiError {
            status,
            error_code,
            error_code_raw,
            message,
        }
    }

    /// Create an HTTP status error
    pub fn http_status(status: u16, message: impl Into<String>) -> Self {
        Self::HttpStatusError {
            status,
            message: message.into(),
            response_body: None,
        }
    }

    /// Create an HTTP status error with response body
    pub fn http_status_with_body(
        status: u16,
        message: impl Into<String>,
        response_body: impl Into<String>,
    ) -> Self {
        Self::HttpStatusError {
            status,
            message: message.into(),
            response_body: Some(response_body.into()),
        }
    }

    /// Create an invalid response error
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse(message.into())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // Network errors might be retryable
            Self::HttpError(_) | Self::MiddlewareHttpError(_) => true,
            // Some HTTP status codes are retryable
            Self::HttpStatusError { status, .. } => {
                matches!(status, 429 | 500 | 502 | 503 | 504)
            }
            // Some API errors are retryable
            Self::ApiError { error_code, .. } => {
                matches!(
                    error_code,
                    Some(BingErrorCode::ThrottleUser) | Some(BingErrorCode::ThrottleHost)
                )
            }
            _ => false,
        }
    }

    /// Check if this error is related to authentication
    pub fn is_authentication_error(&self) -> bool {
        match self {
            Self::AuthenticationError => true,
            Self::HttpStatusError { status, .. } => *status == 401 || *status == 403,
            Self::ApiError { error_code, .. } => {
                matches!(
                    error_code,
                    Some(BingErrorCode::InvalidApiKey) | Some(BingErrorCode::NotAuthorized)
                )
            }
            _ => false,
        }
    }

    /// Check if this error is related to rate limiting
    pub fn is_rate_limit_error(&self) -> bool {
        match self {
            Self::HttpStatusError { status, .. } => *status == 429,
            Self::ApiError { error_code, .. } => {
                matches!(
                    error_code,
                    Some(BingErrorCode::ThrottleUser) | Some(BingErrorCode::ThrottleHost)
                )
            }
            _ => false,
        }
    }

    /// Get the HTTP status code if available
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::HttpStatusError { status, .. } => Some(*status),
            Self::ApiError { status, .. } => *status,
            Self::HttpError(err) => err.status().map(|s| s.as_u16()),
            _ => None,
        }
    }

    /// Get the response body if available
    pub fn response_body(&self) -> Option<&str> {
        match self {
            Self::HttpStatusError { response_body, .. } => response_body.as_deref(),
            _ => None,
        }
    }

    /// Get the error code if this is an API error
    ///
    /// Returns the BingErrorCode if recognized, otherwise returns the raw error code value
    pub fn error_code(&self) -> Option<std::result::Result<BingErrorCode, i32>> {
        match self {
            Self::ApiError {
                error_code,
                error_code_raw,
                ..
            } => Some(error_code.ok_or(*error_code_raw)),
            _ => None,
        }
    }

    /// Get the error message if this is an API error
    pub fn api_message(&self) -> Option<&str> {
        match self {
            Self::ApiError { message, .. } => Some(message.as_str()),
            _ => None,
        }
    }
}

/// Result type alias for Bing Webmaster API operations
pub type Result<T> = std::result::Result<T, WebmasterApiError>;

/// Helper function to map HTTP status codes to appropriate errors
pub fn map_status_error(status: reqwest::StatusCode, response_text: String) -> WebmasterApiError {
    let status_code = status.as_u16();
    let message = match status_code {
        400 => "Bad Request - The request is invalid or malformed",
        401 => "Unauthorized - Invalid or missing API key",
        403 => "Forbidden - Access denied to the requested resource",
        404 => "Not Found - The requested resource was not found",
        429 => "Too Many Requests - Rate limit exceeded",
        500 => "Internal Server Error - Server encountered an error",
        502 => "Bad Gateway - Invalid response from upstream server",
        503 => "Service Unavailable - Service temporarily unavailable",
        504 => "Gateway Timeout - Request timeout",
        _ => "HTTP request failed",
    };

    match status_code {
        401 | 403 => WebmasterApiError::AuthenticationError,
        404 => WebmasterApiError::InvalidResponse(format!("{}: {}", message, response_text)),
        _ => WebmasterApiError::http_status_with_body(status_code, message, response_text),
    }
}

/// Helper function to parse API error responses from response text
///
/// Returns a tuple of (error_code, message) if parsing succeeds
pub fn try_parse_api_error(response_text: &str) -> Option<(i32, String)> {
    // Try to parse as JSON response wrapper first
    if let Ok(wrapper) =
        serde_json::from_str::<crate::dto::ResponseWrapper<ApiErrorResponse>>(response_text)
    {
        return Some((wrapper.d.error_code.value(), wrapper.d.message));
    }

    // Try to parse as direct ApiErrorResponse
    serde_json::from_str::<ApiErrorResponse>(response_text)
        .ok()
        .map(|r| (r.error_code.value(), r.message))
}
