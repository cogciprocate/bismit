use std::ops::{ Range };
use std::collections::{ HashMap };
// use std::iter;

use cmn::{ self, Sdr };
use map::{ AreaMap };
use ocl;
use cortical_area:: { CorticalArea, CorticalAreas };
use proto::{ ProtoAreaMaps, /*ProtoAreaMap, ProtoLayerMap,*/ ProtoLayerMaps, 
	/*RegionKind,*/ layer, /*Sensory,*/ Thalamic };
//use encode:: { IdxReader };
use input_source::{ InputSource };
//use tests::input_czar;


//	THALAMUS:
//	- Input/Output is from a CorticalArea's point of view
// 		- input: to layer / area
// 		- output: from layer / area
// [FIXME] TODO: This system will need revamping. Inputs with multiple source areas (such as multiple
// afferent source areas) will need to be broken down into separate layers rather than just separate rows.
pub struct Thalamus {
	afferent_tract: ThalamicTract,
	efferent_tract: ThalamicTract,
	input_sources: Vec<InputSource>,
	area_maps: HashMap<&'static str, AreaMap>,
}

impl Thalamus {
	pub fn new(plmaps: &ProtoLayerMaps,	pamaps: &ProtoAreaMaps) -> Thalamus {
		let area_count = pamaps.maps().len();

		let mut tao = ThalamicTract::new(area_count);
		let mut teo = ThalamicTract::new(area_count);

		let mut input_sources = Vec::new();
		let mut area_maps = HashMap::new();

		/*=============================================================================
		=================================== THALAMIC ==================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter().filter(|&(_, pa)| 
					plmaps[pa.region_name].kind == Thalamic) 
		{			
			input_sources.push(InputSource::new(pa));

			// let aff_len = pamaps[area_name].aff_areas
			// tao.add_area(area_name, /*i,*/ aff_len, area_map.input_src_area_names_by_flag(layer::AFFERENT_INPUT));

			// println!("{}THALAMUS::NEW(): Area: '{}', aff_len: {}, eff_len: {}", cmn::MT, area_name, aff_len, eff_len);
		}


		/*=============================================================================
		================================= NON-THALAMIC ================================
		=============================================================================*/
		let mut i = 0usize;		

		for (&area_name, pa) in pamaps.maps().iter().filter(|&(_, pa)|
					plmaps[pa.region_name].kind != Thalamic)
		{	
			let area_map = AreaMap::new(pa, plmaps, pamaps);

			let aff_len = area_map.axn_range_by_flag(layer::AFFERENT_INPUT).len();
			let eff_len = area_map.axn_range_by_flag(layer::EFFERENT_INPUT).len();

			tao.add_area(area_name, /*i,*/ aff_len, area_map.input_src_area_names_by_flag(layer::AFFERENT_INPUT));
			teo.add_area(area_name, /*i,*/ eff_len,	area_map.input_src_area_names_by_flag(layer::EFFERENT_INPUT));

			println!("{}THALAMUS::NEW(): Area: '{}', aff_len: {}, eff_len: {}", cmn::MT, area_name, aff_len, eff_len);
			assert!(aff_len > 0 || eff_len > 0, "Areas must have at least one afferent or efferent area.");

			area_map.slices().print_debug();

			area_maps.insert(area_name, area_map);		
			
			i += 1;
		}

		Thalamus {
			afferent_tract: tao.init(),
			efferent_tract: teo.init(),
			input_sources: input_sources,
			area_maps: area_maps,
		}
	}

	// Multiple source output areas disabled.
	pub fn cycle_external_ganglions(&mut self, areas: &mut CorticalAreas) {
		for src in self.input_sources.iter_mut() {
			// let mut ganglion = self.afferent_tract.output_ganglion(src.area_name(), src.area_name()).unwrap();
			// let aff_tract = &mut self.afferent_tract;
			// src.next(areas);
			// src.next(&mut ganglion);
			// src.next(aff_tract);
			src.next(&mut self.afferent_tract);
			//let input_gang = input_sources
			//area.write_input(input_gang, layer::AFFERENT_INPUT);
		}		
	}

	// pub fn cycle_cortical_ganglions(&mut self, areas: &mut CorticalAreas) {
	// 	// for (area_name, area) in areas.iter() {
	// 	// 	for aff_area_name in area.afferent_target_names().iter() {
	// 	// 		//println!("Forwarding from: '{}' to '{}'", area_name, aff_area_name);
	// 	// 		self.forward_afferent_output(area_name, aff_area_name, areas);
	// 	// 	}

	// 	// 	for eff_area_name in area.efferent_target_names().iter() {
	// 	// 		//println!("Backwarding from: '{}' to '{}'", area_name, eff_area_name);
	// 	// 		self.backward_efferent_output(area_name, eff_area_name, areas);
	// 	// 	}
	// 	// }

	// 	// for (area_name, area_map) in self.area_maps.iter() {
	// 	// 	for aff_area_name in area_map.aff_areas().iter() {
	// 	// 		//println!("Forwarding from: '{}' to '{}'", area_name, aff_area_name);
	// 	// 		self.forward_afferent_output(area_name, aff_area_name, areas);
	// 	// 	}

	// 	// 	for eff_area_name in area_map.eff_areas().iter() {
	// 	// 		//println!("Backwarding from: '{}' to '{}'", area_name, eff_area_name);
	// 	// 		self.backward_efferent_output(area_name, eff_area_name, areas);
	// 	// 	}
	// 	// }
	// }


	// WRITE_INPUT(): <<<<< TODO: CHECK SIZES AND SCALE WHEN NECESSARY >>>>>
	pub fn write_input(&self, sdr: &Sdr, area: &mut CorticalArea) {		
		area.write_input(sdr, layer::AFFERENT_INPUT);
	}	

	/*	FORWARD_AFFERENT_OUTPUT(): Read afferent output from a cortical area and store it 
		in our thalamus' cache (the 'tract').

			TODO: RENAME OR BREAK UP
			TODO: HANDLE MULTIPLE TARGET REGIONS
	*/
	// pub fn forward_afferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
	// 			 areas: &mut CorticalAreas) 
	// {
	// 	let emsg = "thalamus::Thalamus::forward_afferent_output(): Area not found: ";
	// 	let emsg_src = format!("{}'{}' ", emsg, src_area_name);
	// 	let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

	// 	//println!("\n ##### FORWARDING AFFERENT OUTPUT from: '{}' to: '{}'", src_area_name, tar_area_name);

	// 	//if self.area_maps[

	// 	areas.get(src_area_name).expect(&emsg_src).read_output(
	// 		self.afferent_tract.output_ganglion(src_area_name, tar_area_name),
	// 		layer::AFFERENT_OUTPUT, 
	// 	);		
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.afferent_tract.input_ganglion(tar_area_name),
	// 		layer::AFFERENT_INPUT,
	// 	);

	// }

	// pub fn read_afferent_output(&mut self, src_area_name &str, 

	// BACKWARD_EFFERENT_OUTPUT():  Cause an efferent frame to descend
	// pub fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
	// 			 areas: &mut CorticalAreas) 
	// {
	// 	let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";
	// 	let emsg_src = format!("{}'{}' ", emsg, src_area_name);
	// 	let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

	// 	match areas.get(tar_area_name) {
	// 		Some(area) => if self.area_maps[tar_area_name].proto_layer_map().kind == Thalamic { return; },
	// 		None => return,
	// 	}

	// 	//println!("\n ##### BACKWARDING EFFERENT OUTPUT from: '{}' to: '{}'", src_area_name, tar_area_name);
		
	// 	areas.get(src_area_name).expect(&emsg_src).read_output(
	// 		self.efferent_tract.output_ganglion(src_area_name, tar_area_name), 
	// 		layer::EFFERENT_OUTPUT,
	// 	);
	
	// 	/* TEST */
	// 	//let test_vec = input_czar::sdr_stripes(512, false, &mut self.efferent_tract[slc_range.clone()]);
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.efferent_tract.input_ganglion(tar_area_name), 
	// 		layer::EFFERENT_INPUT,
	// 	);
 // 	}

 	pub fn aff_tract(&mut self) -> &mut ThalamicTract {
 		&mut self.afferent_tract
	}

	pub fn eff_tract(&mut self) -> &mut ThalamicTract {
 		&mut self.efferent_tract
	}

 	pub fn area_maps(&self) -> &HashMap<&'static str, AreaMap> {
 		&self.area_maps
	}

 	pub fn area_map(&self, area_name: &'static str) -> &AreaMap {
 		&self.area_maps[area_name]
	}
}

// THALAMICTRACT: A BUFFER FOR COMMUNICATION BETWEEN CORTICAL AREAS
pub struct ThalamicTract {
	ganglion: Vec<ocl::cl_uchar>,			// BUFFER DIVIDED UP BY AREA
	//area_info: Vec<TractArea>,				// INFO ABOUT TARGET AREAS
	tract_areas: HashMap<&'static str, TractArea>,	// MAP OF TARGET AREA NAMES -> AREA INFO INDEXES
	ttl_len: usize,
}

impl ThalamicTract {
	fn new(area_count: usize,
				//ganglion: Vec<ocl::cl_uchar>,
				//area_info: Vec<TractArea>, 
				//tract_areas: HashMap<&'static str, TractArea>,
			) -> ThalamicTract 
	{

		let ganglion = Vec::with_capacity(0);
		//Vec::with_capacity(area_count), 
		let tract_areas = HashMap::with_capacity(area_count);

		ThalamicTract {
			ganglion: ganglion,
			//area_info: area_info,
			tract_areas: tract_areas,
			ttl_len: 0,
		}
	}

	fn add_area(&mut self, tar_area_name: &'static str, /*idx: usize,*/ len: usize, src_areas: &Vec<&'static str>) {
		//self.area_info.push(TractArea::new(self.ttl_len, len, src_areas));
		//self.tract_areas.insert(tar_area_name, idx);
		self.tract_areas.insert(tar_area_name, TractArea::new(self.ttl_len, len, src_areas));
		self.ttl_len += len;
	}

	fn init(mut self) -> ThalamicTract {
		self.ganglion.resize(self.ttl_len, 0);
		println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
		self
	}

	pub fn input_ganglion(&self, tar_area_name: &str) -> &Sdr {
		// let range = match self.input_range(tar_area_name) {
		// 	Some(r) => r,
		// 	None => return None,
		// };
			// .expect("ThalamicTract::input_ganglion(): Invalid target name.");
		let range = self.input_range(tar_area_name);

		// println!(" ### ThalamicTract.input_ganglion(): range: {:?}", range);
		debug_assert!(range.end <= self.ganglion.len(), "ThalamicTract::input_ganglion(): \
			Index range for target area: '{}' exceeds the boundaries of the input tract \
			(length: {}).", 
			tar_area_name, self.ganglion.len());
		
		&self.ganglion[range]
	}

	pub fn output_ganglion(&mut self, src_area_name: &str, tar_area_name: &str) -> &mut Sdr {
		// let range = match self.output_range(src_area_name, tar_area_name) {
		// 	Some(r) => r,
		// 	None => return None,
		// };
			// .expect("ThalamicTract::output_ganglion(): Invalid target name.");
		let range = self.output_range(src_area_name, tar_area_name);

		// println!(" ### ThalamicTract.output_ganglion(): range: {:?}", range);
		debug_assert!(range.end <= self.ganglion.len(), "ThalamicTract::output_ganglion(): \
			Index range for source area: '{}' and target area: '{}' exceeds the boundaries \
			of the output tract (length: {}).", src_area_name, tar_area_name, self.ganglion.len());
		
		&mut self.ganglion[range]
	}

	//  OUTPUT_RANGE(): RANGE OF THE TRACT DESIGNATED TO BUFFER OUTPUT FROM 
	//	THE 'OUTPUT' CORTICAL AREA DESTINED FOR THE 'INPUT' CORTICAL AREA(S).
	//		- [out of date] LENGTH WILL EQUAL THE NUMBER OF COLUMNS FOR THE LARGER OF THE TWO AREAS. 
	fn output_range(&self, src_area_name: &str, tar_area_name: &str) -> Range<usize> {
		// println!("  ### ThalamicTract.output_range(): src_area_name:{}, tar_area_name: {}", 
			// src_area_name, tar_area_name);

		// match self.tract_areas.get(tar_area_name) {
		// 	Some(ref info) => Some(info.src_area_range(src_area_name)),
		// 	None => None,
		// }
		debug_assert!(self.tract_areas.contains_key(tar_area_name), "Thalamus::output_range(): \
			Invalid target area name: '{}'");

		self.tract_areas.get(tar_area_name).expect(&format!("ThalamicTract.output_range(): Invalid \
			target area name: '{}'.", tar_area_name)).src_area_range(src_area_name)
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
		// println!("  ### ThalamicTract.input_range(): tar_area_name: {}", 
			// tar_area_name);
		// self.info(tar_area_name).expect("ThalamicTract::input_range()").range.clone()
		// match self.tract_areas.get(tar_area_name) {
		// 	Some(ref info) => Some(info.full_range()),
		// 	None => None,
		// }
		debug_assert!(self.tract_areas.contains_key(tar_area_name), "Thalamus::input_range(): \
			Invalid target area name: '{}'");

		self.tract_areas.get(tar_area_name).expect(&format!("ThalamicTract.output_range(): Invalid \
			target area name: '{}'.", tar_area_name)).full_range()
	}

	// fn info(&self, tar_area_name: &str) -> Option<&TractArea> {
	// 	//let idx = self.area_map[tar_area_name];
	// 	//&self.area_info[idx]
	// 	println!("   ### ThalamicTract.info(): tar_area_name: {}", tar_area_name);

	// 	self.tract_areas.get(tar_area_name)
	// }

}

#[derive(PartialEq, Debug, Clone, Eq)]
// [FIXME] TODO: This system will need revamping. Inputs with multiple source areas (such as multiple
// afferent source areas) will need to be broken down into separate layers rather than just separate rows.
struct TractArea {
	range: Range<usize>,
	src_areas: Vec<&'static str>,
	output_len: usize,
}

impl TractArea {
	fn new(range_start: usize, range_len: usize, src_areas: &Vec<&'static str>) -> TractArea {
		TractArea { 
			range: range_start..(range_start + range_len),			
			output_len: if src_areas.len() == 0 { 0 } else { range_len / src_areas.len() },
			src_areas: src_areas.clone(),
		}
	}

	// [FIXME] Change to option or result return val.
	fn src_area_range(&self, src_area_name: &str) -> Range<usize> {
		let start = self.range.start + (self.src_area_index(src_area_name) * self.output_len());
		// println!("    ### TAI::src_area_range(): src_area_name: {}, self.range.start: {}, 
		// 	start: {}, end: {}", src_area_name, self.range.start, start, start + self.output_len());
		return start..(start + self.output_len());
	}

	// Returns the index of the source area in the src_areas vector.
	// Returns 0 if src_area_name was not found.
	// [FIXME] This is seriously wonky. Return an option or result?
	fn src_area_index(&self, src_area_name: &str) -> usize {
		let mut idx = 0;

		for san in self.src_areas.iter() {
			if &src_area_name == san { 
				break; 
			} else {
				idx += 1;
			}
		}
		
		// if idx == self.src_areas.len() {
		// 	0
		// } else {
		// 	idx
		// }

		debug_assert!(idx != self.src_areas.len());
		idx		
	}

	fn output_len(&self) -> usize {
		self.output_len
	}

	fn full_range(&self) -> Range<usize> {
		self.range.clone()
	}
}


pub mod tests {
	
}



	// THALAMUS::WRITE(): USED FOR TESTING PURPOSES
	// 	<<<<< NEEDS UPDATING TO NEW SYSTEM - CALL AREA.WRITE() >>>>>
	// 		- Change input param to &CorticalArea			
	// 	TODO: DEPRICATE
	// pub fn write(&self, area_name: &str, layer_target: &'static str, 
	// 			sdr: &Sdr, areas: &HashMap<&'static str, Box<CorticalArea>>,
	// ) {
	// 	let emsg = format!("cortex::Cortex::write_vec(): Invalid area name: {}", area_name);
	// 	let area = areas.get(area_name).expect(&emsg);

	// 	//let ref region = self.plmaps[&RegionKind::Sensory];
	// 	let region = area.proto_layer_maps();
	// 	let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
	// 	for slc in axn_slcs { 
	// 		//let buffer_offset = cmn::axn_idz_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
	// 		let buffer_offset = self.area_map.axn_idz(slc);
	// 		ocl::enqueue_write_buffer(sdr, area.axns.states.buf, area.ocl().queue(), buffer_offset);
	// 	}
	// }
	
