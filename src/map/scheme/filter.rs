
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct FilterScheme {
    filter_name: String,
    cl_file_name: Option<String>,
}

impl FilterScheme {
    pub fn new(filter_name: &str, cl_file_name: Option<&str>) -> FilterScheme {

        let clfn_opt = match cl_file_name {
            Some(ref clfn) => Some(clfn.to_string()),
            None => None,
        };

        FilterScheme {
            filter_name: filter_name.to_string(),
            cl_file_name: clfn_opt,
        }
    }

    pub fn filter_name(&self) -> String {
        self.filter_name.clone()
    }

    pub fn cl_file_name(&self) -> Option<String> {
        match self.cl_file_name {
            Some(ref clfn) => Some(clfn.clone()),
            None => None,
        }
    }
}

impl<'a> From<(&'a str, Option<&'a str>)> for FilterScheme {
    fn from(tup: (&'a str, Option<&'a str>)) -> FilterScheme {
        FilterScheme::new(tup.0, tup.1)
    }
}

impl<'a,> From<&'a (&'a str, Option<&'a str>)> for FilterScheme {
    fn from(tup_ref: &'a (&'a str, Option<&'a str>)) -> FilterScheme {
        FilterScheme::new(tup_ref.0, tup_ref.1)
    }
}