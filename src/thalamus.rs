use std::ops::{ Range };
use std::collections::{ HashMap };
// use std::iter;

use cmn::{ self, Sdr };
use map::{ self, AreaMap, LayerFlags };
use ocl::{ self, EventList };
use cortical_area:: { CorticalAreas };
use proto::{ ProtoAreaMaps, ProtoLayerMaps, Thalamic };

use input_source::{ InputSource };



//	THALAMUS:
//	- Input/Output is from a CorticalArea's point of view
// 		- input: to layer / area
// 		- output: from layer / area
pub struct Thalamus {
	tract: ThalamicTract,
	input_sources: Vec<InputSource>,
	area_maps: HashMap<&'static str, AreaMap>,
}

impl Thalamus {
	pub fn new(plmaps: &ProtoLayerMaps,	pamaps: &ProtoAreaMaps) -> Thalamus {
		let area_count = pamaps.maps().len();

		let mut tract = ThalamicTract::new();
		let mut input_sources = Vec::new();
		let mut area_maps = HashMap::new();

		/*=============================================================================
		=================================== THALAMIC ==================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter().filter(|&(_, pa)| 
					plmaps[pa.region_name].kind == Thalamic) 
		{			
			input_sources.push(InputSource::new(pa));
		}

		/*=============================================================================
		================================= NON-THALAMIC ================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter()
			// .filter(|&(_, pa)| plmaps[pa.region_name].kind != Thalamic)
		{	
			let area_map = AreaMap::new(pa, plmaps, pamaps);

			let layer_info = area_map.output_layer_info_by_flag();

			for &(flags, cols) in layer_info.iter() {
				tract.add_area(area_name, flags, cols as usize);
			}

			println!("{}THALAMUS::NEW(): Area: '{}', source area info: {:?}.", cmn::MT, area_name, layer_info);
			assert!(layer_info.len() > 0, "Areas must have at least one afferent or efferent area.");

			area_map.slices().print_debug();

			area_maps.insert(area_name, area_map);	
			
		}

		Thalamus {
			tract: tract.init(),
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
			let src_area_name = src.area_name();
			let (ganglion, events) = self.tract.ganglion_mut(src_area_name, 
				map::AFFERENT_OUTPUT | map::EFFERENT_OUTPUT);
			src.next(ganglion, events);
		}		
	}

	// WRITE_INPUT(): <<<<< TODO: CHECK SIZES AND SCALE WHEN NECESSARY >>>>>
	// pub fn write_input(&self, sdr: &Sdr, area: &mut CorticalArea) {		
	// 	area.write_input(sdr, map::AFFERENT_INPUT);
	// }

	pub fn ganglion(&self, src_area_name: &'static str, layer_mask: LayerFlags) -> (&EventList, &Sdr) { 		
		self.tract.ganglion(src_area_name, layer_mask)
	}

	pub fn ganglion_mut(&mut self, src_area_name: &'static str, layer_mask: LayerFlags) 
			-> (&mut Sdr, &mut EventList) 
	{
		self.tract.ganglion_mut(src_area_name, layer_mask)
	}

 	pub fn area_maps(&self) -> &HashMap<&'static str, AreaMap> {
 		&self.area_maps
	}

 	pub fn area_map(&self, area_name: &'static str) -> &AreaMap {
 		&self.area_maps[area_name]
	}
}

// THALAMICTRACT: A buffer for I/O between areas
pub struct ThalamicTract {
	ganglion: Vec<ocl::cl_uchar>,				
	tract_areas: HashMap<(&'static str, LayerFlags), TractArea>,	
	ttl_len: usize,
}

impl ThalamicTract {
	fn new() -> ThalamicTract {
		let ganglion = Vec::with_capacity(0);
		let tract_areas = HashMap::new();

		ThalamicTract {
			ganglion: ganglion,
			tract_areas: tract_areas,
			ttl_len: 0,
		}
	}

	fn add_area(&mut self, src_area_name: &'static str, layer_flags: LayerFlags, len: usize) {
		self.tract_areas.insert((src_area_name, layer_flags), 
			TractArea::new(src_area_name, layer_flags, self.ttl_len..(self.ttl_len + len)));
		self.ttl_len += len;
	}

	fn init(mut self) -> ThalamicTract {
		self.ganglion.resize(self.ttl_len, 0);
		// println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
		self
	}


	fn ganglion(&self, src_area_name: &'static str, layer_flags: LayerFlags) -> (&EventList, &Sdr) {
		let ta = self.tract_areas.get(&(src_area_name, layer_flags)).expect(&format!(
			"ThalamicTract::ganglion(): Invalid source area name and/or flags: \
			('{}', '{:?}').", src_area_name, layer_flags));

		let range = ta.range();
		let events = ta.events();

		// println!(" ### ThalamicTract::ganglion({}, {:?}): range: {:?}", src_area_name, 
		// 	 layer_flags, range);
		debug_assert!(range.end <= self.ganglion.len(), "ThalamicTract::input_ganglion(): \
			Index range for target area: '{}' exceeds the boundaries of the input tract \
			(length: {}).", src_area_name, self.ganglion.len());
		
		(events, &self.ganglion[range])
	}

	fn ganglion_mut(&mut self, src_area_name: &'static str, layer_flags: LayerFlags)
			-> (&mut Sdr, &mut EventList) 
	{
		let ta = self.tract_areas.get_mut(&(src_area_name, layer_flags)).expect(&format!(
			"ThalamicTract::ganglion(): Invalid target area name and/or flags: \
			('{}', '{:?}').", src_area_name, layer_flags));

		let range = ta.range();
		let events = ta.events_mut();

		// println!(" ### ThalamicTract::ganglion_mut({}, {:?}): range: {:?}", src_area_name, 
		// 	 layer_flags, range);
		debug_assert!(range.end <= self.ganglion.len(), "ThalamicTract::ganglion_mut(): \
			Index range for target area: '{}' exceeds the boundaries of the input tract \
			(length: {}).", src_area_name, self.ganglion.len());
		
		(&mut self.ganglion[range], events)
	}
}

// #[derive(PartialEq, Debug, Clone, Eq)]
struct TractArea {
	src_area_name: &'static str,
	layer_flags: LayerFlags,
	range: Range<usize>,
	events: EventList,
}

impl TractArea {
	fn new(src_area_name: &'static str, layer_flags: LayerFlags, range: Range<usize>) -> TractArea {
		TractArea { 
			src_area_name: src_area_name,
			layer_flags: layer_flags,
			range: range,
			events: EventList::new(),
		}
	}

	fn range(&self) -> Range<usize> {
		self.range.clone()
	}

	fn len(&self) -> usize {
		self.range.len()
	}

	fn events(&self) -> &EventList {
		&self.events
	}

	fn events_mut(&mut self) -> &mut EventList {
		&mut self.events
	}
}


pub mod tests {
	
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
	// 		map::AFFERENT_OUTPUT, 
	// 	);		
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.afferent_tract.input_ganglion(tar_area_name),
	// 		map::AFFERENT_INPUT,
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
	// 		map::EFFERENT_OUTPUT,
	// 	);
	
	// 	/* TEST */
	// 	//let test_vec = input_czar::sdr_stripes(512, false, &mut self.efferent_tract[slc_range.clone()]);
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.efferent_tract.input_ganglion(tar_area_name), 
	// 		map::EFFERENT_INPUT,
	// 	);
 // 	}

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
	
