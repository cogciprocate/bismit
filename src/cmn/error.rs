use std::error::{ Error };
use std::fmt;

pub struct CmnError {
	description: String,
}

impl CmnError {
	pub fn new(desc: String) -> CmnError {
		CmnError { description: desc }
	}
}

impl Error for CmnError {
    fn description(&self) -> &str {
        &self.description
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
        f.write_str(&self.description)
    }
}

impl fmt::Debug for CmnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description)
    }
}


// use std::error::{ Error };
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
