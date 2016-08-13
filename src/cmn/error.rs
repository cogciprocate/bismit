use std::error::{Error};
use std::fmt;
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
/// `Status` will eventually be changed internally to contain a sub-error type
/// unique to each function which generates it (yeah that'll be fun to
/// implement).
///
/// `UnspecifiedDimensions` may be moved into a sub-type.
///
/// For now, don't assume the existence of or check for any of the above.
///
#[derive(Debug)]
pub enum CmnError {
    Unknown,
    String(String),
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
    pub fn prepend<'s, S: AsRef<&'s str>>(&'s mut self, txt: S) {
        if let &mut CmnError::String(ref mut string) = self {
            string.reserve_exact(txt.as_ref().len());
            let old_string_copy = string.clone();
            string.clear();
            string.push_str(txt.as_ref());
            string.push_str(&old_string_copy);
        }
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

impl fmt::Display for CmnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

// impl fmt::Debug for CmnError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_str(self.description())
//     }
// }


// use std::error::{Error};
// use std::fmt;

// pub struct CmnErrorTest<'a> {
//     description: fmt::Arguments<'a>,
// }

// impl<'a> CmnErrorTest<'a> {
//     pub fn new(desc: fmt::Arguments<'a>) -> CmnErrorTest {
//         CmnErrorTest { description: desc }
//     }
// }

// impl<'a> Error for CmnErrorTest<'a> {
//     fn description(&self) -> &str {
//         &self.description
//     }
// }

// impl<'a> From<String> for CmnErrorTest<'a> {
//     fn from(desc: String) -> CmnErrorTest<'a> {
//         CmnErrorTest::new(desc)
//     }
// }

// impl<'a> fmt::Display for CmnErrorTest<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_fmt(self.description)
//     }
// }

// impl<'a> fmt::Debug for CmnErrorTest<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_fmt(self.description)
//     }
// }
