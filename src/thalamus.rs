use std::ops::{ Range };
use std::collections::{ HashMap };
use std::iter;

use cmn;
use ocl;
use cortical_area:: { CorticalArea, CorticalAreas };
use proto::{ Protoareas, Protoarea, ProtolayerMap, ProtolayerMaps, 
	RegionKind, layer, Sensory, Thalamic };
use encode:: { IdxReader };
use input_source::{ InputSource };
use tests::input_czar;


//	THALAMUS:
//	- Input/Output is from a CorticalArea's point of view
// 		- input: to layer / area
// 		- output: from layer / area
pub struct Thalamus {
	tract_afferent_output: ThalamicTract,
	tract_efferent_output: ThalamicTract,
	input_sources: Vec<InputSource>,
}

impl Thalamus {
	pub fn new(areas: &HashMap<&'static str, Box<CorticalArea>>, protolayer_maps: ProtolayerMaps,
				protoareas: Protoareas,
	) -> Thalamus {		
		let mut tao = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));

		let mut teo = ThalamicTract::new(Vec::with_capacity(0), 
			Vec::with_capacity(areas.len()), HashMap::with_capacity(areas.len()));

		let mut input_sources = Vec::new();

		for (_, pa) in protoareas.map().iter().filter(|&(_, pa)| 
					protolayer_maps[pa.region_name].kind == Thalamic) 
		{			
			input_sources.push(InputSource::new(pa));
		}

		let mut i = 0usize;
		/*  <<<<< TODO: TAKE IN TO ACCOUNT MULTI-SLICE INPUT LAYERS >>>>>  */
		for (&area_name, ref area) in areas {
			let aff_len = area.axn_range(layer::AFFERENT_INPUT).len();
			let eff_len = area.axn_range(layer::EFFERENT_INPUT).len();

			tao.add_area(area_name, i, aff_len, area.input_src_area_names(layer::AFFERENT_INPUT));
			teo.add_area(area_name, i, eff_len,	area.input_src_area_names(layer::EFFERENT_INPUT));

			println!("THALAMUS::NEW(): Area: '{}', aff_len: {}, eff_len: {}", area_name, aff_len, eff_len);			
			
			i += 1;
		}

		Thalamus {
			tract_afferent_output: tao.init(),
			tract_efferent_output: teo.init(),
			input_sources: input_sources,
		}
	}

	fn cycle_external_ganglions(&mut self, areas: &CorticalAreas) {
		for src in self.input_sources.iter_mut() {
			src.next(areas);
			//let input_gang = input_sources
			//area.write_input(input_gang, layer::AFFERENT_INPUT);
		}		
	}

	pub fn cycle_cortical_ganglions(&mut self, areas: &CorticalAreas) {
		for (area_name, area) in areas.iter() {
			for aff_area_name in area.afferent_target_names().iter() {
				//println!("Forwarding from: '{}' to '{}'", area_name, aff_area_name);
				self.forward_afferent_output(area_name, aff_area_name, &areas);
			}

			for eff_area_name in area.efferent_target_names().iter() {
				//println!("Backwarding from: '{}' to '{}'", area_name, eff_area_name);
				self.backward_efferent_output(area_name, eff_area_name, &areas);
			}
		}

		self.cycle_external_ganglions(areas);
	}


	// WRITE_INPUT(): <<<<< TODO: CHECK SIZES AND SCALE WHEN NECESSARY >>>>>
	pub fn write_input(&self, sdr: &[ocl::cl_uchar], area: &mut CorticalArea) {		
		area.write_input(sdr, layer::AFFERENT_INPUT);
	}


	// THALAMUS::WRITE(): USED FOR TESTING PURPOSES
	// 	<<<<< NEEDS UPDATING TO NEW SYSTEM - CALL AREA.WRITE() >>>>>
	// 		- Change input param to &CorticalArea			
	// 	TODO: DEPRICATE
	// pub fn write(&self, area_name: &str, layer_target: &'static str, 
	// 			sdr: &[ocl::cl_uchar], areas: &HashMap<&'static str, Box<CorticalArea>>,
	// ) {
	// 	let emsg = format!("cortex::Cortex::write_vec(): Invalid area name: {}", area_name);
	// 	let area = areas.get(area_name).expect(&emsg);

	// 	//let ref region = self.protolayer_maps[&RegionKind::Sensory];
	// 	let region = area.protolayer_map();
	// 	let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
	// 	for slc in axn_slcs { 
	// 		//let buffer_offset = cmn::axn_idz_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
	// 		let buffer_offset = self.area_map.axn_idz(slc);
	// 		ocl::enqueue_write_buffer(sdr, area.axns.states.buf, area.ocl().queue(), buffer_offset);
	// 	}
	// }
	

	/*	FORWARD_AFFERENT_OUTPUT(): Read afferent output from a cortical area and store it 
		in our pseudo thalamus' cache (the 'tract').

			TODO: RENAME OR BREAK UP
			TODO: HANDLE MULTIPLE TARGET REGIONS
	*/
	fn forward_afferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = "thalamus::Thalamus::forward_afferent_output(): Area not found: ";
		let emsg_src = format!("{}'{}' ", emsg, src_area_name);
		let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

		areas.get(src_area_name).expect(&emsg_src).read_output(
			self.tract_afferent_output.output_ganglion(src_area_name, tar_area_name),
			layer::AFFERENT_OUTPUT, 
		);		
		
		areas.get(tar_area_name).expect(&emsg_tar).write_input(
			self.tract_afferent_output.input_ganglion(tar_area_name),
			layer::AFFERENT_INPUT,
		);

	}

	// BACKWARD_EFFERENT_OUTPUT():  Cause an efferent frame to descend
	fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
				 areas: &HashMap<&'static str, Box<CorticalArea>>,
	) {
		let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";
		let emsg_src = format!("{}'{}' ", emsg, src_area_name);
		let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

		match areas.get(tar_area_name) {
			Some(area) => if area.protolayer_map().kind == Thalamic { return; },
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
		self.area_info.push(AreaInfo::new(self.ttl_len, len, src_areas));
		self.area_map.insert(tar_area_name, idx);
		self.ttl_len += len;
	}

	fn init(mut self) -> ThalamicTract {
		self.ganglion.resize(self.ttl_len, 0);
		println!("THALAMICTRACT::INIT(): area_map: {:?}, area_info: {:?}", self.area_map, self.area_info);
		self
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


mod tests {
	
}
