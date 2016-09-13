use subcortex::Thalamus;


pub trait SubcorticalArea {
    fn area_name<'a>(&'a self) -> &'a str;
}



pub struct TestScArea {
    area_name: String,
}

impl TestScArea {
    pub fn new<'a>(area_name: &'a str) -> TestScArea {
        TestScArea {
            area_name: area_name.into(),
        }
    }
}

impl SubcorticalArea for TestScArea {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }
}



pub struct Subcortex {
    areas: Vec<Box<SubcorticalArea>>,
}

impl Subcortex {
    pub fn new() -> Subcortex {
        Subcortex {
            areas: Vec::with_capacity(16),
        }
    }

    pub fn cycle(thal: &Thalamus) {

    }
}