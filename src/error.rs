#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    HttpError(hyper::http::Error),
    HyperError(hyper::Error),
    SystemTimeError(std::time::SystemTimeError),
    Utf8Error(std::str::Utf8Error),
    MissingCacheItem(String),
    ToStrError(hyper::header::ToStrError),
    JsonError(serde_json::Error),
    InvalidStatusCode(http::status::InvalidStatusCode),
    InvalidHeaderValue(http::header::InvalidHeaderValue),
    MutexError,
    InvalidCredentials,
    AuthenticationFailed,
    TrustFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::IOError(err) => {
                write!(f, "{}", err)
            }
            Error::HttpError(err) => {
                write!(f, "{}", err)
            }
            Error::HyperError(err) => {
                write!(f, "{}", err)
            }
            Error::SystemTimeError(err) => {
                write!(f, "{}", err)
            }
            Error::Utf8Error(err) => {
                write!(f, "{}", err)
            }
            Error::MissingCacheItem(err) => {
                write!(f, "{}", err)
            }
            Error::ToStrError(err) => {
                write!(f, "{}", err)
            }
            Error::JsonError(err) => {
                write!(f, "{}", err)
            }
            Error::InvalidStatusCode(err) => {
                write!(f, "{}", err)
            }
            Error::InvalidHeaderValue(err) => {
                write!(f, "{}", err)
            }
            Error::MutexError => {
                write!(f, "Mutex error")
            }
            Error::InvalidCredentials => {
                write!(f, "Invalid credentials.")
            }
            Error::AuthenticationFailed => {
                write!(f, "Authentication failed.")
            }
            Error::TrustFailed => {
                write!(f, "Trust failed.")
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IOError(error)
    }
}

impl From<hyper::http::Error> for Error {
    fn from(error: hyper::http::Error) -> Error {
        Error::HttpError(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::HyperError(error)
    }
}

impl From<hyper::header::ToStrError> for Error {
    fn from(error: hyper::header::ToStrError) -> Error {
        Error::ToStrError(error)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(error: std::time::SystemTimeError) -> Error {
        Error::SystemTimeError(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error::Utf8Error(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::JsonError(error)
    }
}

impl From<http::status::InvalidStatusCode> for Error {
    fn from(error: http::status::InvalidStatusCode) -> Error {
        Error::InvalidStatusCode(error)
    }
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(error: http::header::InvalidHeaderValue) -> Error {
        Error::InvalidHeaderValue(error)
    }
}
