use std::fmt;
use thiserror::Error;

/// Result alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for this library.
#[derive(Error, Debug)]
pub struct Error {
    pub msg: String,
    #[cfg(feature = "nightly")]
    backtrace: std::backtrace::Backtrace,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

// Implement the Display trait for our Error type.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

// Implement Default for Error
impl Default for Error {
    fn default() -> Self {
        Error {
            msg: "".to_string(),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: None,
        }
    }
}

impl Error {
    /// Create a new Error instance.
    pub fn new(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: None,
        }
    }
    /// Create a new Error instance with a source error.
    pub fn with_source(msg: &str, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error {
            msg: msg.to_string(),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(source),
        }
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error {
            msg: String::from("Config Error"),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error {
            msg: String::from("Poison Error"),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            msg: String::from("IO Error"),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        Error {
            msg: String::from("Clap Error"),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(err: log::SetLoggerError) -> Self {
        Error {
            msg: String::from("Logger Error"),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error {
            msg: err.to_string(),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(err),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error {
            msg: format!("JSON error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error {
            msg: format!("YAML error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json5::Error> for Error {
    fn from(err: serde_json5::Error) -> Self {
        Error {
            msg: format!("JSON5 error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Error {
            msg: format!("Hex error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<ed25519_dalek::SignatureError> for Error {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        Error {
            msg: format!("Ed25519 error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error {
            msg: format!("UUID error: {}", err),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: Some(Box::new(err)),
        }
    }
}
