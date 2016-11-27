use std::error::{Error};
use std::fmt;
// use std::ops::Deref;
use ocl::Error as OclError;
use cmn::CmnResult;


/// An enum containing either a `String` or one of several other error types.
///
/// Implements the usual error traits.
///
/// ## Stability
///
/// The `String` variant may eventually be removed. Many more variants and
/// sub-types will be added as time goes on and things stabilize.
///
#[derive(Debug)]
pub enum CmnError {
    Unknown,
    String(String),
    OclError(OclError),
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
    pub fn prepend(mut self, txt: &str) -> CmnError {
        if let CmnError::String(ref mut string) = self {
            string.reserve_exact(txt.len());
            let old_string_copy = string.clone();
            string.clear();
            string.push_str(txt.as_ref());
            string.push_str(&old_string_copy);
        } else {
            panic!("Cannot prepend to a non-string error.");
        }

        self
    }
}

impl Error for CmnError {
    fn description(&self) -> &str {
        match *self {
            CmnError::String(ref desc) => desc,
            _ => unimplemented!(),
        }
    }
}

impl Into<String> for CmnError {
    fn into(self) -> String {
        use std::error::Error;
        self.description().to_string()
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

impl From<OclError> for CmnError {
    fn from(e: OclError) -> CmnError {
        CmnError::OclError(e)
    }
}

impl fmt::Display for CmnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}