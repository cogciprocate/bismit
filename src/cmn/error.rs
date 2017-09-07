use std::io;
use std::error::{Error};
use std::fmt;
// use std::ops::Deref;
use futures::sync::mpsc::SendError;
use futures::sync::oneshot::Canceled;
use ocl::{self, async};
// use cmn::CmnResult;
use map::ExecutionGraphError;


pub type CmnResult<T> = Result<T, CmnError>;

// [NOTE]: Implement this someday:
//
// impl<T> From<Result<T, OclError>> for Result<T, CmnError> {
//     fn from(result: Result<T, OclError>) -> Result<T, CmnError> {
//         match result {
//             Ok(value) => Ok(value),
//             Err(err) => Err(err.into()),
//         }
//     }
// }


/// An enum containing either a `String` or one of several other error types.
///
/// Implements the usual error traits.
///
/// ## Stability
///
/// The `String` variant may eventually be removed. Many more variants and
/// sub-types will be added as time goes on and things stabilize.
///
pub enum CmnError {
    Unknown,
    String(String),
    IoError(io::Error),
    OclError(ocl::Error),
    AsyncError(async::Error),
    ExecutionGraphError(ExecutionGraphError),
}

impl CmnError {
    /// Returns a new `Error` with the description string: `desc`.
    pub fn new<S: Into<String>>(desc: S) -> CmnError {
        CmnError::String(desc.into())
    }

    /// Returns a new `ocl::Result::Err` containing an `ocl::Error::String`
    /// variant with the given description.
    pub fn err<T, S: Into<String>>(desc: S) -> CmnResult<T> {
        Err(CmnError::String(desc.into()))
    }

    /// If this is a `String` variant, concatenate `txt` to the front of the
    /// contained string. Otherwise, do nothing at all.
    pub fn prepend<S: AsRef<str>>(mut self, txt: S) -> CmnError {
        if let CmnError::String(ref mut string) = self {
            string.reserve_exact(txt.as_ref().len());
            let old_string_copy = string.clone();
            string.clear();
            string.push_str(txt.as_ref());
            string.push_str(&old_string_copy);
        } else {
            panic!("Cannot prepend to a non-string error.");
        }

        self
    }

    pub fn from_ocl_result<T>(result: ocl::Result<T>) -> CmnResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => Err(err.into()),
        }
    }
}

impl Error for CmnError {
    fn description(&self) -> &str {
        match *self {
            CmnError::Unknown => "Unknown error.",
            CmnError::String(ref msg) => msg,
            CmnError::IoError(ref err) => err.description(),
            CmnError::OclError(ref err) => err.description(),
            CmnError::AsyncError(ref err) => err.description(),
            CmnError::ExecutionGraphError(ref err) => err.description(),
        }
    }
}

// impl Into<String> for CmnError {
//     fn into(self) -> String {
//         use std::error::Error;
//         self.description().to_string()
//     }
// }

impl From<()> for CmnError {
    fn from(_: ()) -> CmnError {
        CmnError::Unknown
    }
}

impl From<String> for CmnError {
    fn from(desc: String) -> CmnError {
        CmnError::new(desc)
    }
}

impl<'a> From<&'a str> for CmnError {
    fn from(desc: &'a str) -> CmnError {
        CmnError::new(String::from(desc))
    }
}

impl From<io::Error> for CmnError {
    fn from(e: io::Error) -> CmnError {
        CmnError::IoError(e)
    }
}

impl From<ocl::Error> for CmnError {
    fn from(e: ocl::Error) -> CmnError {
        CmnError::OclError(e)
    }
}

impl From<async::Error> for CmnError {
    fn from(e: async::Error) -> CmnError {
        CmnError::AsyncError(e)
    }
}

impl<T> From<SendError<T>> for CmnError {
    fn from(e: SendError<T>) -> CmnError {
        CmnError::AsyncError(async::Error::from(e))
    }
}

impl From<Canceled> for CmnError {
    fn from(e: Canceled) -> CmnError {
        CmnError::AsyncError(async::Error::from(e))
    }
}

impl From<ExecutionGraphError> for CmnError {
    fn from(e: ExecutionGraphError) -> CmnError {
        CmnError::ExecutionGraphError(e)
    }
}

impl fmt::Display for CmnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmnError::String(ref msg) => f.write_str(msg),
            CmnError::OclError(ref err) => write!(f, "{}", err),
            CmnError::AsyncError(ref err) => write!(f, "{}", err),
            CmnError::ExecutionGraphError(ref err) => {
                write!(f, "ExecutionGraph error: ").and(fmt::Display::fmt(err, f))
            },
            ref err @ _ => write!(f, "{}", err.description()),
        }
    }
}

impl fmt::Debug for CmnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}