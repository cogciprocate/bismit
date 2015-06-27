
use cmn;
use proto::{ Protoregion, Protoregions, ProtoregionKind, Protoareas, ProtoareasTrait, Protoarea, ProtolayerKind, ProtoaxonKind, layer, Protocell };
use cortex::{ self, Cortex };
use super::input_czar::{ self, InputCzar, InputVecKind };
use super::hybrid;


pub fn define_prtrgns() -> Protoregions {
	Protoregions::new()
		.region(Protoregion::new(ProtoregionKind::Sensory)
			.layer("thal_t", 1, layer::AFFERENT_INPUT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))
			.layer("out_t", 1, layer::AFFERENT_OUTPUT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))
			.layer("iv_t", 1, layer::SPATIAL_ASSOCIATIVE, Protocell::new_spiny_stellate(5, vec!["thal_t"], 256))  // , "motor"
			.layer("iv_inhib_t", 0, layer::DEFAULT, Protocell::new_inhibitory(4, "iv_t"))
			.layer("iii_t", 4, layer::TEMPORAL_ASSOCIATIVE, Protocell::new_pyramidal(2, 5, vec!["iii_t"], 256))
			.layer("motor_t", 1, layer::DEFAULT, ProtolayerKind::Axonal(ProtoaxonKind::Horizontal))
			.freeze()
		)
}

pub fn define_prtareas() -> Protoareas {
	Protoareas::new().area("v1_t", 64, 64, ProtoregionKind::Sensory, None)
}


/* IDEAS FOR TESTS:
	- set synapse src_ids, src_ofs, strs to 0
		- test some specific inputs and make sure that synapses are responding exactly
*/
#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());
	let area_name = "v1_t";

	hybrid::test_cycles(&mut cortex, area_name);
}



#[test]
fn test_learning() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());
	let area_name = "v1_t";
	let si_layer_name = "iv_inhib_t";

	hybrid::test_learning(&mut cortex, si_layer_name, area_name);
}
