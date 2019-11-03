use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AppError {
    InputError(String),
    IOError(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::InputError(ref s) => write!(f, "InputError: {}", s),
            AppError::IOError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for AppError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            AppError::InputError(_) => None,
            AppError::IOError(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::IOError(err)
    }
}
