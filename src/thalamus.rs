use std::ops::{ Range };
use std::collections::{ HashMap };
use std::iter;

use cmn;
use ocl;
use cortical_area:: { CorticalArea };
use proto::{ Protoareas, ProtoareasTrait, Protoarea, Protoregion, Protoregions, ProtoregionKind, layer };
use tests::input_czar;

/*	THALAMUS:
		- Input/Output is from a CorticalArea's point of view
			- input: to layer / area
			- output: from layer / area

*/
pub struct Thalamus {
	concourse: Vec<ocl::cl_uchar>,
	//tract_afferent_input: Vec<ocl::cl_uchar>,
	tract_afferent_output: Vec<ocl::cl_uchar>,
	//tract_efferent_input: Vec<ocl::cl_uchar>,
	tract_efferent_output: Vec<ocl::cl_uchar>,
	index: Vec<AreaInfo>, // <<<<< POSSIBLY CONVERT TO ARRAY FOR SIMPLICITY -- CONFUSING NAME
	map: HashMap<&'static str, usize>,
	protoareas: Protoareas, // <<<<< EVENTUALLY DISCARD
	protoregions: Protoregions,
	ocl: ocl::Ocl,
}

impl Thalamus {
	pub fn new(protoareas: Protoareas, protoregions: Protoregions, ocl: ocl::Ocl) -> Thalamus {
		let emsg = "thalamus::Thalamus::new(): ";
		let mut index = Vec::with_capacity(protoareas.len());
		let mut map = HashMap::with_capacity(protoareas.len());

		let (mut ai_len, mut ao_len, mut ei_len, mut eo_len) = (0, 0, 0, 0);

		let mut cc_len = 0usize;
		let mut i = 0usize;

		/*  <<<<< TAKE IN TO ACCOUNT MULTI-SLICE INPUT LAYERS >>>>>  */
		for (&pa_name, pa) in &protoareas {
			//print!("\nTHALAMUS::NEW(): Adding area: '{}'", pa_name);

			let pa_input_depth = protoregions[&pa.region_kind].layer_with_flag(layer::AFFERENT_INPUT)
				.expect(&format!("{}{}", emsg, "pa_input_depth -- flag not found")).depth;

			//let pa_columns = (pa.dims.u_size() * pa.dims.v_size()) as usize;
			let pa_columns = pa.dims.columns() as usize;
			let pa_len = pa_columns * pa_input_depth as usize;

			index.push(
				AreaInfo {
					cc_range: cc_len..(cc_len + pa_len),
					protoarea: pa.clone(),
				}
			);

			cc_len += pa_len;

			assert!(index[i].protoarea.name == pa_name);

			map.insert(pa.name, i);

			i += 1;
		}

		let concourse: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();

		//let tract_afferent_input: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();
		let tract_afferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();
		//let tract_efferent_input: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();
		let tract_efferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();

		//print!("\n\n##### THALAMUS::NEW(): \n\n    INDEX: {:?}\n\n    MAP: {:?}\n\n    CONCOURSE.LEN(): {}", index, map, concourse.len());

		Thalamus {
			concourse: concourse,
			//tract_afferent_input: tract_afferent_input,
			tract_afferent_output: tract_afferent_output,
			//tract_efferent_input: tract_efferent_input,
			tract_efferent_output: tract_efferent_output,
			index: index,
			map: map,
			protoareas: protoareas, // <<<<< EVENTUALLY DISCARD
			protoregions: protoregions,
			ocl: ocl,
		}
	}

	/* WRITE_INPUT(): TODO: RENAME OR DO SOMETHING ELSE WITH THIS */
	pub fn write_input(&self, sdr: &[ocl::cl_uchar], area: &mut CorticalArea) {
		// <<<<< TODO: CHECK SIZES AND SCALE WHEN NECESSARY >>>>>
		area.write_input(sdr, layer::AFFERENT_INPUT);
	}


	/*	THALAMUS::WRITE()
			<<<<< NEEDS UPDATING TO NEW SYSTEM - CALL AREA.WRITE() >>>>>
				- Change input param to &CorticalArea

			TODO: DEPRICATE
	*/ 
	pub fn write(&self, area_name: &str, layer_target: &'static str, 
				sdr: &[ocl::cl_uchar], areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = format!("cortex::Cortex::write_vec(): Invalid area name: {}", area_name);
		let area = areas.get(area_name).expect(&emsg);

		let ref region = self.protoregions[&ProtoregionKind::Sensory];
		let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
		for slc in axn_slcs { 
			let buffer_offset = cmn::axn_idx_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
			//let buffer_offset = cmn::SYNAPSE_REACH_LIN + (axn_slc as usize * self.cortical_area.axns.dims.width as usize);

			//println!("##### write_vec(): {} offset: axn_idx_2d(axn_slc: {}, dims.columns(): {}, region.hrz_demarc(): {}): {}, sdr.len(): {}", layer_target, slc, self.cortical_area.dims.columns(), region.hrz_demarc(), buffer_offset, sdr.len());

			//assert!(sdr.len() <= self.cortical_area.dims.columns() as usize); // <<<<< NEEDS CHANGING (for multi-slc inputs)

			ocl::enqueue_write_buffer(sdr, area.axns.states.buf, self.ocl.command_queue, buffer_offset);
		}
	}


	/*	READ_OUTPUT(): Read output (afferent or efferent) from a cortical area and store it 
		in our pseudo thalamus' cache (the 'tract').

			TODO: RENAME OR BREAK UP
			TODO: HANDLE MULTIPLE TARGET REGIONS
	*/
	pub fn forward_afferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &mut HashMap<&'static str, Box<CorticalArea>>,
	) {
		let area_index = self.map[src_area_name];
		let slc_range = self.index[area_index].cc_range.clone();

		let emsg = "thalamus::Thalamus::forward_afferent_output(): Area not found: ";

		let emsg1 = format!("{}'{}' ", emsg, src_area_name);
		areas.get_mut(src_area_name).expect(&emsg1)
			.read_output(&mut self.tract_afferent_output[slc_range.clone()], layer::AFFERENT_OUTPUT);

		let emsg2 = format!("{}'{}' ", emsg, tar_area_name);
		areas.get_mut(tar_area_name).expect(&emsg2)
			.write_input(&self.tract_afferent_output[slc_range.clone()], layer::AFFERENT_INPUT);

		//cmn::print_vec_simple(&self.tract_afferent_output[..]);
	}

	pub fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &mut HashMap<&'static str, Box<CorticalArea>>,
	) {
		let area_index = self.map[src_area_name];
		let slc_range = self.index[area_index].cc_range.clone();

		let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";

		let emsg1 = format!("{}'{}' ", emsg, src_area_name);
		areas.get_mut(src_area_name).expect(&emsg1)
			.read_output(&mut self.tract_efferent_output[slc_range.clone()], layer::EFFERENT_OUTPUT);


		/* TESTING */
		//let test_vec = input_czar::sdr_stripes(512, false, &mut self.tract_efferent_output[slc_range.clone()]);

		let emsg2 = format!("{}'{}' ", emsg, tar_area_name);
		areas.get_mut(tar_area_name).expect(&emsg2)
			.write_input(&self.tract_efferent_output[slc_range.clone()], layer::EFFERENT_INPUT);
 	}

	/*fn area_output_target(&self, src_area_name: &'static str) {
		let area = self.map[src_area.name];

	}*/
}

impl Drop for Thalamus {
    fn drop(&mut self) {
        print!("Releasing OCL Components...");
		self.ocl.release_components();
    }
}


#[derive(PartialEq, Debug, Clone, Eq)]
struct AreaInfo {
	cc_range: Range<usize>,		// RENAME / ELABORATE / EXPAND
	protoarea: Protoarea,
}



