use std::ops::{ Range };
use std::collections::{ HashMap };
use std::iter;

use cmn;
use ocl;
use cortical_area:: { CorticalArea };
use proto::{ Protoareas, ProtoareasTrait, Protoarea, Protoregion, Protoregions, 
	RegionKind, layer, Sensory, Thalamic };
use encode:: { IdxReader };
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
		//let epre = "thalamus::Thalamus::new(): ";

		let mut tao = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));

		let mut teo = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));


		// let mut index_eff = Vec::with_capacity(areas.len());
		// let mut map_eff = HashMap::with_capacity(areas.len());		

		//let (mut ai_len, mut ao_len, mut ei_len, mut eo_len) = (0, 0, 0, 0);

		//println!("\n");
		let mut i = 0usize;

		/*  <<<<< TAKE IN TO ACCOUNT MULTI-SLICE INPUT LAYERS >>>>>  */
		for (&area_name, ref area) in areas {
			//println!("THALAMUS::NEW(): Adding area: '{}'", area_name);
			//if area.region_kind != Sensory { continue; };
			//let emsga = format!("{}{}", epre, "area_input_depth -- flag not found");

			// let area_input_depth = protoregions[&area.region_kind]
			// 	.layer_with_flag(layer::AFFERENT_INPUT).expect(&emsga).depth;

			// //let area_columns = (area.dims.u_size() * area.dims.v_size()) as usize;
			// let area_columns = area.dims.columns() as usize;
			// let area_len = area_columns * area_input_depth as usize;

			let aff_len = area.axn_range(layer::AFFERENT_INPUT).len();
			let eff_len = area.axn_range(layer::EFFERENT_INPUT).len();

			tao.add_area(area_name, i, aff_len, area.input_src_area_names(layer::AFFERENT_INPUT));
			teo.add_area(area_name, i, eff_len,	area.input_src_area_names(layer::EFFERENT_INPUT));

			//let cc_area_len = if aff_in_len > eff_in_len { aff_in_len } else { eff_in_len };

			println!("THALAMUS::NEW(): Area: '{}', aff_len: {}, eff_len: {}", area_name, aff_len, eff_len);			

			// tao.index.push(AreaInfo { range: tao.axn_len..(tao.axn_len + aff_len) });
			// tao.axn_len += aff_len;
			// tao.map.insert(area.name, i);

			// teo.index.push(AreaInfo { range: teo.axn_len..(teo.axn_len + aff_len) });
			// teo.axn_len += aff_len;
			// teo.map.insert(area.name, i);			

			i += 1;
		}

		tao.init();
		teo.init();
		//let concourse: Vec<ocl::cl_uchar> = iter::repeat(0).take(cc_len).collect();

		//let tract_afferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(aff_len).collect();
		//let tract_efferent_output: Vec<ocl::cl_uchar> = iter::repeat(0).take(aff_len).collect();

		//println!("\n##### THALAMUS::NEW(): \n\n    INDEX: {:?}\n\n    MAP: {:?}\n\n    CONCOURSE.LEN(): {}", index, map, concourse.len());

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


	// THALAMUS::WRITE(): USED FOR TESTING PURPOSES
	// 	<<<<< NEEDS UPDATING TO NEW SYSTEM - CALL AREA.WRITE() >>>>>
	// 		- Change input param to &CorticalArea			
	// 	TODO: DEPRICATE
	
	pub fn write(&self, area_name: &str, layer_target: &'static str, 
				sdr: &[ocl::cl_uchar], areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = format!("cortex::Cortex::write_vec(): Invalid area name: {}", area_name);
		let area = areas.get(area_name).expect(&emsg);

		//let ref region = self.protoregions[&RegionKind::Sensory];
		let region = area.protoregion();
		let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
		for slc in axn_slcs { 
			let buffer_offset = cmn::axn_idz_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
			//let buffer_offset = cmn::SYNAPSE_REACH_LIN + (axn_slc as usize * self.cortical_area.axns.dims.width as usize);

			//println!("##### write_vec(): {} offset: axn_idz_2d(axn_slc: {}, dims.columns(): {}, region.hrz_demarc(): {}): {}, sdr.len(): {}", layer_target, slc, self.cortical_area.dims.columns(), region.hrz_demarc(), buffer_offset, sdr.len());

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
				 areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = "thalamus::Thalamus::forward_afferent_output(): Area not found: ";

		let emsg1 = format!("{}'{}' ", emsg, src_area_name);
		areas.get(src_area_name).expect(&emsg1).read_output(
			self.tract_afferent_output.output_ganglion(src_area_name, tar_area_name),
			layer::AFFERENT_OUTPUT, 
		);
		
		let emsg2 = format!("{}'{}' ", emsg, tar_area_name);
		areas.get(tar_area_name).expect(&emsg2).write_input(
			self.tract_afferent_output.input_ganglion(tar_area_name),
			layer::AFFERENT_INPUT,
		);

		//cmn::print_vec_simple(&self.tract_afferent_output[..]);
	}

	pub fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";
		let emsg_src = format!("{}'{}' ", emsg, src_area_name);
		let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

		match areas.get(tar_area_name) {
			Some(area) => if area.protoregion().kind == Thalamic { return; },
			None => return,
		}
		
		areas.get(src_area_name).expect(&emsg_src).read_output(
			self.tract_efferent_output.output_ganglion(src_area_name, tar_area_name), 
			layer::EFFERENT_OUTPUT,
		);
	
		/* TEST */
		//let test_vec = input_czar::sdr_stripes(512, false, &mut self.tract_efferent_output[slc_range.clone()]);
		
		areas.get(tar_area_name).expect(&emsg_tar).write_input(
			self.tract_efferent_output.input_ganglion(tar_area_name), 
			layer::EFFERENT_INPUT,
		);
 	}

	/*fn area_output_target(&self, src_area_name: &'static str) {
		let area = self.map[src_area.name];

	}*/
}

// THALAMICTRACT: A BUFFER FOR COMMUNICATION BETWEEN CORTICAL AREAS
struct ThalamicTract {
	ganglion: Vec<ocl::cl_uchar>,			// BUFFER DIVIDED UP BY AREA
	area_info: Vec<AreaInfo>,				// INFO ABOUT TARGET AREAS
	area_map: HashMap<&'static str, usize>,	// MAP OF TARGET AREA NAMES -> AREA INFO INDEXES
	ttl_len: usize,
}

impl ThalamicTract {
	fn new(		ganglion: Vec<ocl::cl_uchar>,
				area_info: Vec<AreaInfo>, 
				area_map: HashMap<&'static str, usize>,
	) -> ThalamicTract {
		ThalamicTract {
			ganglion: ganglion,
			area_info: area_info,
			area_map: area_map,
			ttl_len: 0,
		}
	}

	fn add_area(&mut self, tar_area_name: &'static str, idx: usize, len: usize, src_areas: Vec<&'static str>) {
		// self.area_info.push(AreaInfo { 
		// 	range: self.ttl_len..(self.ttl_len + len),
		// 	src_areas: src_areas,
		// });
		self.area_info.push(AreaInfo::new(self.ttl_len, len, src_areas));
		self.area_map.insert(tar_area_name, idx);
		self.ttl_len += len;
	}

	fn init(&mut self) {
		self.ganglion.resize(self.ttl_len, 0);
	}

	fn input_ganglion(&self, tar_area_name: &str) -> &[u8] {
		let range = self.input_range(tar_area_name);
		&self.ganglion[range]
	}

	fn output_ganglion(&mut self, src_area_name: &str, tar_area_name: &str) -> &mut [u8] {
		let range = self.output_range(src_area_name, tar_area_name);
		&mut self.ganglion[range]
	}

	//  OUTPUT_RANGE(): RANGE OF THE TRACT DESIGNATED TO BUFFER OUTPUT FROM 
	//	THE 'OUTPUT' CORTICAL AREA DESTINED FOR THE 'INPUT' CORTICAL AREA(S).
	//		- LENGTH WILL EQUAL THE NUMBER OF COLUMNS FOR THE LARGER OF THE TWO AREAS.
	fn output_range(&self, src_area_name: &str, tar_area_name: &str) -> Range<usize> {
		//let area_count = self.info(tar_area_name).src_areas.len();
		// let mut slc_range = self.info(tar_area_name).range.clone();
		// slc_range.end = slc_range.start 
		// 	+ (self.info(tar_area_name).output_len());

		// return slc_range;
		self.info(tar_area_name).src_area_range(src_area_name)
	}

	//  INPUT_RANGE(): RANGE OF THE TRACT DESIGNATED TO BUFFER THE CONCATENATED
	// 	OUTPUTS FROM THE 'OUTPUT' CORTICAL AREAS TO AN 'INPUT' CORTICAL AREA
	// 		- IN INSTANCES WHERE THE 'INPUT' CORTICAL AREA IS RECEIVING INPUT FROM MULTIPLE AREAS:
	//			- THE TOTAL LENGTH WILL BE THE SUM OF THE COLUMN COUNT OF EVERY INPUT AREA:
	//		   		- DEPTH_TOTAL * COLUMNS
	// 			- THE RANGE WILL ENCOMPASS THE RANGES USED PREVIOUSLY FOR OUTPUTS
	// 		- IN CASES WHERE A CORTICAL AREA HAS ONLY ONE INPUT SOURCE AREA, INPUT RANGE WILL
	//		  EQUAL OUTPUT RANGE.
	fn input_range(&self, tar_area_name: &str) -> Range<usize> {
		self.info(tar_area_name).range.clone()
	}

	fn info(&self, tar_area_name: &str) -> &AreaInfo {
		let idx = self.area_map[tar_area_name];
		&self.area_info[idx]
	}

}

#[derive(PartialEq, Debug, Clone, Eq)]
struct AreaInfo {
	range: Range<usize>,
	src_areas: Vec<&'static str>,
	output_len: usize,
}

impl AreaInfo {
	fn new(range_start: usize, range_len: usize, src_areas: Vec<&'static str>) -> AreaInfo {
		AreaInfo { 
			range: range_start..(range_start + range_len),			
			output_len: if src_areas.len() == 0 { 0 } else { range_len / src_areas.len() },
			src_areas: src_areas,
		}
	}

	fn src_area_range(&self, src_area_name: &str) -> Range<usize> {
		let start = self.range.start + (self.src_area_index(src_area_name) * self.output_len());
		return start..(start + self.output_len());
	}

	fn src_area_index(&self, src_area_name: &str) -> usize {
		let mut idx = 0;

		for san in self.src_areas.iter() {
			if &src_area_name == san { 
				break; 
			} else {
				idx += 1;
			}
		}
		
		idx
	}

	fn output_len(&self) -> usize {
		self.output_len
	}
}




pub enum InputKind {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader(Box<IdxReader>),
}

pub struct InputSource {
	kind: InputKind,
	target_area_name: &'static str,	
}

impl InputSource {
	pub fn new(kind: InputKind, target_area_name: &'static str) -> InputSource {
		InputSource {
			target_area_name: target_area_name,
			kind: kind,
		}
	}
}
