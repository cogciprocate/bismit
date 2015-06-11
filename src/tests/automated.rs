
use cmn;
use proto::*;
use cortex::{ self, Cortex };
use super::input_czar::{ self, InputCzar, InputVecKind };
use super::hybrid;



pub fn define_prtrgns() -> Protoregions {
	Protoregions::new()
		.region(Protoregion::new(ProtoregionKind::Sensory)
			.layer("thal", 1, layer::DEFAULT, Axonal(Spatial))
			.layer("out", 1, layer::COLUMN_OUTPUT, Axonal(Spatial))
			.layer("iv", 1, layer::COLUMN_INPUT, Protocell::new_spiny_stellate(5, vec!["thal", "thal", "thal", "motor"], 256))  // , "motor"
			.layer("iv_inhib", 0, layer::DEFAULT, Protocell::new_inhibitory(4, "iv"))
			.layer("iii", 4, layer::DEFAULT, Protocell::new_pyramidal(2, 5, vec!["iii"], 256))
			.layer("motor", 1, layer::DEFAULT, Axonal(Horizontal))
			.freeze()
		)
}

pub fn define_prtareas() -> Protoareas {
	Protoareas::new().area("v1", 6, 6, ProtoregionKind::Sensory)
}


/* IDEAS FOR TESTS:
	- set synapse src_ids, src_ofs, strs to 0
		- test some specific inputs and make sure that synapses are responding exactly
*/
#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());

	hybrid::test_cycles(&mut cortex);
}



#[test]
fn test_learning() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());

	hybrid::test_learning(&mut cortex);
}
