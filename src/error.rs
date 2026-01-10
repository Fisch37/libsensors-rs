use std::{ffi::{c_int, c_uint}, error::Error as StdError, fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct SensorsError {
    pub code: i32
}
impl SensorsError {
    /// Converts an i32 into a result with this error type.
    /// If code > 0, Ok(code) will be returned,
    /// else Err with the correct Error structure
    pub(crate) fn convert_cint(code: c_int) -> std::result::Result<c_uint, Self> {
        if code < 0 {
            Err(SensorsError { code })
        } else {
            Ok(code as c_uint)
        }
    }
}
impl Display for SensorsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown error during libsensors call: {}", self.code)
    }
}
impl StdError for SensorsError { }

#[derive(Debug)]
pub enum Error {
    Sensors(SensorsError),
    Loading(libloading::Error),
    UnexpectedWildcard(i64)
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensors(e) => write!(f, "Sensors({e})"),
            Self::Loading(e) => write!(f, "Loading({e})"),
            Self::UnexpectedWildcard(value) => write!(f, "Unexpected wildcard value: {value}")
        }
    }
}
impl StdError for Error { }
impl From<SensorsError> for Error {
    fn from(value: SensorsError) -> Self {
        Self::Sensors(value)
    }
}
impl From<libloading::Error> for Error {
    fn from(value: libloading::Error) -> Self {
        Self::Loading(value)
    }
}