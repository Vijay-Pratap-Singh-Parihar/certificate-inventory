//! Centralized application error (mirrors telephony ApplicationError).
//! Use cases and repository traits return `Result<T, ApplicationError>`.
//! Adapters map this to HTTP status and body.

use serde_json::Value;
use std::error::Error;
use std::panic::Location;
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub struct ApplicationError {
    pub(crate) message: String,
    pub(crate) error_type: &'static str,
    pub(crate) error_code: Option<u16>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_details: Option<Value>,
}

impl Error for ApplicationError {}

impl ApplicationError {
    pub(crate) fn new(
        message: String,
        error_type: &'static str,
        error_code: Option<u16>,
        error_message: Option<String>,
        error_details: Option<Value>,
    ) -> Self {
        Self {
            message,
            error_type,
            error_code,
            error_message,
            error_details,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn error_code(&self) -> Option<u16> {
        self.error_code
    }

    pub fn error_type(&self) -> &str {
        self.error_type
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn error_details(&self) -> Option<&Value> {
        self.error_details.as_ref()
    }

    #[track_caller]
    pub fn into<E: Error + 'static>(
        err: Option<E>,
        msg: Option<impl Into<String>>,
        error_code: Option<u16>,
        err_details: Option<Value>,
    ) -> Self {
        let location = Location::caller();

        match (err, msg) {
            (Some(e), Some(m)) => {
                let message = m.into();
                error!(
                    "Error at {}:{} - {:?}, {}",
                    location.file(),
                    location.line(),
                    &e,
                    &message
                );
                Self::new(
                    message,
                    std::any::type_name::<E>(),
                    error_code,
                    Some(e.to_string()),
                    err_details,
                )
            }
            (Some(e), None) => {
                error!("Error at {}:{} - {:?}", location.file(), location.line(), &e);
                Self::new(
                    e.to_string(),
                    std::any::type_name::<E>(),
                    error_code,
                    Some(e.to_string()),
                    err_details,
                )
            }
            (None, Some(m)) => {
                let message = m.into();
                error!("Error at {}:{} - {}", location.file(), location.line(), &message);
                Self::new(
                    message,
                    std::any::type_name::<()>(),
                    error_code,
                    None,
                    err_details,
                )
            }
            (None, None) => {
                let message = "Unknown error occurred".to_string();
                error!("Unknown error at {}:{}", location.file(), location.line());
                Self::new(message, std::any::type_name::<()>(), error_code, None, err_details)
            }
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Default for ApplicationError {
    fn default() -> Self {
        Self {
            message: "An unknown error occurred".to_string(),
            error_type: "Unknown",
            error_code: None,
            error_message: None,
            error_details: None,
        }
    }
}
