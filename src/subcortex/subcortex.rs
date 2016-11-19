use subcortex::Thalamus;

// [NOTES]:
//
// VentralLateralNucleus -- Inputs from the basal nuclei which includes the
// substantia nigra and the globus pallidus (via the thalamic fasciculus). It
// also has inputs from the cerebellum (dentate nucleus, via the
// dentatothalamic tract). It sends neuronal output to the primary motor
// cortex and premotor cortex
//
// The function of the ventral lateral nucleus is to target efferents
// including the motor cortex, premotor cortex, and supplementary motor
// cortex. Therefore, its function helps the coordination and planning of
// movement. It also plays a role in the learning of movement.

// VentralAnteriorNucleus -- Receives neuronal inputs from the basal ganglia.
// Its main afferent fibres are from the globus pallidus. The efferent fibres
// from this nucleus pass into the premotor cortex for initiation and planning
// of movement.
//
// It helps to function in movement by providing feedback for the outputs of the basal ganglia.



pub trait SubcorticalNucleus {
    fn area_name<'a>(&'a self) -> &'a str;
}



pub struct TestScNucleus {
    area_name: String,
}

impl TestScNucleus {
    pub fn new<'a>(area_name: &'a str) -> TestScNucleus {
        TestScNucleus {
            area_name: area_name.into(),
        }
    }
}

impl SubcorticalNucleus for TestScNucleus {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }
}



pub struct Subcortex {
    nuclei: Vec<Box<SubcorticalNucleus>>,
}

impl Subcortex {
    pub fn new() -> Subcortex {
        Subcortex {
            nuclei: Vec::with_capacity(16),
        }
    }

    pub fn nucleus(mut self, nucleus: Box<SubcorticalNucleus>) -> Subcortex {
        self.nuclei.push(nucleus);
        self
    }

    pub fn cycle(&mut self, thal: &Thalamus) {
        for nucleus in self.nuclei.iter_mut() {
            thal.area_maps();
            let _ = nucleus;
        }
    }
}