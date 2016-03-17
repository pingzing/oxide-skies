extern crate serde_json;
extern crate hyper;

use std::error;
use std::io;

use std::fmt;

#[derive(Debug)]
pub enum ErrorExt {
    IoError(io::Error),
    SerdeError(serde_json::Error),
    HyperError(hyper::Error),
    DataTooOld
}

impl fmt::Display for ErrorExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorExt::IoError(ref err) => write!(f, "IO error: {}", err),
            ErrorExt::SerdeError(ref err) => write!(f, "Serde Error: {}", err),
            ErrorExt::HyperError(ref err) => write!(f, "Hyper Error: {}", err),
            ErrorExt::DataTooOld => write!(f, "Data Too Old error.")
        }
    }
}

impl error::Error for ErrorExt {
    fn description(&self) -> &str {
        match *self {
            ErrorExt::IoError(ref err) => err.description(),
            ErrorExt::SerdeError(ref err) => err.description(),
            ErrorExt::HyperError(ref err) => err.description(),
            ErrorExt::DataTooOld => return "data too old",
        }
    }
    
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ErrorExt::IoError(ref err) => Some(err),
            ErrorExt::SerdeError(ref err) => Some(err),
            ErrorExt::HyperError(ref err) => Some(err),
            ErrorExt::DataTooOld => None,
        }
    }       
}

impl From<io::Error> for ErrorExt {
    fn from(err: io::Error) -> ErrorExt {
        ErrorExt::IoError(err)
    }
}

impl From<serde_json::Error> for ErrorExt {
    fn from(err: serde_json::Error) -> ErrorExt {
        ErrorExt::SerdeError(err)
    }
}

impl From<hyper::Error> for ErrorExt {
    fn from(err: hyper::Error) -> ErrorExt {
        ErrorExt::HyperError(err)
    }
}