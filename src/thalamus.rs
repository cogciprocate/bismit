use std::ops::{ Range };
use std::collections::{ HashMap };
use std::iter;

use cmn;
use ocl;
use cortical_area:: { CorticalArea };
use proto::{ Protoareas, ProtoareasTrait, Protoarea, Protoregion, Protoregions, ProtoregionKind, layer, Sensory };
use tests::input_czar;

/*	THALAMUS:
		- Input/Output is from a CorticalArea's point of view
			- input: to layer / area
			- output: from layer / area

*/
pub struct Thalamus {
	//concourse: Vec<ocl::cl_uchar>,
	//tract_afferent_input: Vec<ocl::cl_uchar>,
	//tract_efferent_input: Vec<ocl::cl_uchar>,
	// tract_afferent_output: Vec<ocl::cl_uchar>,
	// index: Vec<AreaInfo>, 
	// map: HashMap<&'static str, usize>,
	// tract_efferent_output: Vec<ocl::cl_uchar>,
	// index: Vec<AreaInfo>,
	// map: HashMap<&'static str, usize>,
	//protoareas: Protoareas, // <<<<< EVENTUALLY DISCARD
	//protoregions: Protoregions,
	//ocl: ocl::OclProgQueue,
	tract_afferent_output: ThalamicTract,
	tract_efferent_output: ThalamicTract,
}

impl Thalamus { // , protoregions: Protoregions
	pub fn new(areas: &HashMap<&'static str, Box<CorticalArea>>) -> Thalamus {
		let epre = "thalamus::Thalamus::new(): ";

		let mut tao = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));

		let mut teo = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));


		// let mut index_eff = Vec::with_capacity(areas.len());
		// let mut map_eff = HashMap::with_capacity(areas.len());		

		//let (mut ai_len, mut ao_len, mut ei_len, mut eo_len) = (0, 0, 0, 0);

		print!("\n\n");
		let mut i = 0usize;

		/*  <<<<< TAKE IN TO ACCOUNT MULTI-SLICE INPUT LAYERS >>>>>  */
		for (&area_name, ref area) in areas {
			//print!("\nTHALAMUS::NEW(): Adding area: '{}'", area_name);
			//if area.region_kind != Sensory { continue; };
			//let emsga = format!("{}{}", epre, "area_input_depth -- flag not found");

			// let area_input_depth = protoregions[&area.region_kind]
			// 	.layer_with_flag(layer::AFFERENT_INPUT).expect(&emsga).depth;

			// //let area_columns = (area.dims.u_size() * area.dims.v_size()) as usize;
			// let area_columns = area.dims.columns() as usize;
			// let area_len = area_columns * area_input_depth as usize;

			let aff_len = area.axn_range(layer::AFFERENT_INPUT).len();
			tao.add_area(area_name, i, aff_len);

			let eff_len = area.axn_range(layer::EFFERENT_INPUT).len();
			teo.add_area(area_name, i, eff_len);

			//let cc_area_len = if aff_in_len > eff_in_len { aff_in_len } else { eff_in_len };

			print!("\nTHALAMUS::NEW(): Area: '{}', aff_len: {}, eff_len: {}", epre, area_name, aff_len, eff_len);			

			// tao.index.push(AreaInfo { range: tao.axn_len..(tao.axn_len + aff_len) });
			// tao.axn_len += aff_len;
			// tao.map.insert(area.name, i);

			// teo.index.push(AreaInfo { range: teo.axn_len..(teo.axn_len + aff_len) });
			// teo.axn_len += aff_len;
			// teo.map.insert(area.name, i);			

			i += 1;
		}

		//let concourse: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();

		//let tract_afferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(aff_len).collect();
		//let tract_efferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(aff_len).collect();

		//print!("\n\n##### THALAMUS::NEW(): \n\n    INDEX: {:?}\n\n    MAP: {:?}\n\n    CONCOURSE.LEN(): {}", index, map, concourse.len());

		Thalamus {			
			//concourse: concourse,
			//tract_afferent_input: tract_afferent_input,
			//tract_afferent_output: tract_afferent_output,
			//tract_efferent_input: tract_efferent_input,
			//tract_efferent_output: tract_efferent_output,
			//index: index,
			//map: map,
			//protoareas: protoareas, // <<<<< EVENTUALLY DISCARD
			//protoregions: protoregions,
			//ocl: ocl,
			tract_afferent_output: tao,
			tract_efferent_output: teo,
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

		//let ref region = self.protoregions[&ProtoregionKind::Sensory];
		let region = area.protoregion();
		let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
		for slc in axn_slcs { 
			let buffer_offset = cmn::axn_idx_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
			//let buffer_offset = cmn::SYNAPSE_REACH_LIN + (axn_slc as usize * self.cortical_area.axns.dims.width as usize);

			//println!("##### write_vec(): {} offset: axn_idx_2d(axn_slc: {}, dims.columns(): {}, region.hrz_demarc(): {}): {}, sdr.len(): {}", layer_target, slc, self.cortical_area.dims.columns(), region.hrz_demarc(), buffer_offset, sdr.len());

			//assert!(sdr.len() <= self.cortical_area.dims.columns() as usize); // <<<<< NEEDS CHANGING (for multi-slc inputs)

			ocl::enqueue_write_buffer(sdr, area.axns.states.buf, area.ocl().queue(), buffer_offset);
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
		let area_index = self.tract_afferent_output.map[src_area_name];
		let slc_range = self.tract_afferent_output.index[area_index].range.clone();

		let emsg = "thalamus::Thalamus::forward_afferent_output(): Area not found: ";

		{
			let emsg1 = format!("{}'{}' ", emsg, src_area_name);
			let src_area = areas.get_mut(src_area_name).expect(&emsg1);
			src_area.read_output(&mut self.tract_afferent_output.ganglion[slc_range.clone()], layer::AFFERENT_OUTPUT);
		}

		let emsg2 = format!("{}'{}' ", emsg, tar_area_name);
		let tar_area = areas.get_mut(tar_area_name).expect(&emsg2);
		tar_area.write_input(&self.tract_afferent_output.ganglion[slc_range.clone()], layer::AFFERENT_INPUT);

		//cmn::print_vec_simple(&self.tract_afferent_output[..]);
	}

	pub fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &mut HashMap<&'static str, Box<CorticalArea>>,
	) {
		let area_index = self.tract_efferent_output.map[src_area_name];
		let slc_range = self.tract_efferent_output.index[area_index].range.clone();

		let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";

		{
			let emsg1 = format!("{}'{}' ", emsg, src_area_name);
			let src_area = areas.get_mut(src_area_name).expect(&emsg1);		
			src_area.read_output(&mut self.tract_efferent_output.ganglion[slc_range.clone()], layer::EFFERENT_OUTPUT);
		}

		/* TESTING */
		//let test_vec = input_czar::sdr_stripes(512, false, &mut self.tract_efferent_output[slc_range.clone()]);

		let emsg2 = format!("{}'{}' ", emsg, tar_area_name);
		let tar_area = areas.get_mut(tar_area_name).expect(&emsg2);		
		tar_area.write_input(&self.tract_efferent_output.ganglion[slc_range.clone()], layer::EFFERENT_INPUT);
 	}

	/*fn area_output_target(&self, src_area_name: &'static str) {
		let area = self.map[src_area.name];

	}*/
}

struct ThalamicTract {
	ganglion: Vec<ocl::cl_uchar>,
	index: Vec<AreaInfo>, 
	map: HashMap<&'static str, usize>,
	ttl_len: usize,
}

impl ThalamicTract {
	fn new(
				ganglion: Vec<ocl::cl_uchar>,
				index: Vec<AreaInfo>, 
				map: HashMap<&'static str, usize>,
	) -> ThalamicTract {
		ThalamicTract {
			ganglion: ganglion,
			index: index,
			map: map,
			ttl_len: 0,
		}
	}

	fn add_area(&mut self, area_name: &'static str, idx: usize, len: usize) {
		self.index.push(AreaInfo { range: self.ttl_len..(self.ttl_len + len) });
		self.map.insert(area_name, idx);
		self.ttl_len += len;

		// tao.index.push(AreaInfo { range: tao.axn_len..(tao.axn_len + aff_len) });
		// tao.axn_len += aff_len;
		// tao.map.insert(area.name, i);
	}

}

#[derive(PartialEq, Debug, Clone, Eq)]
struct AreaInfo {
	range: Range<usize>,		// RENAME / ELABORATE / EXPAND
}



