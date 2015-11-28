use std::ops::{ Range };
use std::collections::{ HashMap };
// use std::iter;

use cmn::{ self, Sdr, CmnError };
use map::{ AreaMap, LayerTags };
use ocl::{ self, EventList};
use cortical_area:: { CorticalAreas };
use proto::{ ProtoareaMaps, ProtolayerMaps, Thalamic };

use input_source::{ InputSource, InputSources };


//	THALAMUS:
//	- Input/Output is from a CorticalArea's point of view
// 		- input: to layer / area
// 		- output: from layer / area
pub struct Thalamus {
	tract: ThalamicTract,
	// input_sources: Vec<InputSource>,
	input_sources: InputSources,
	area_maps: HashMap<&'static str, AreaMap>,
}

impl Thalamus {
	pub fn new(plmaps: ProtolayerMaps,	mut pamaps: ProtoareaMaps) -> Thalamus {
		pamaps.freeze();

		let area_count = pamaps.maps().len();

		let mut tract = ThalamicTract::new();
		let mut input_sources = HashMap::new();
		let mut area_maps = HashMap::new();

		/*=============================================================================
		=================================== THALAMIC ==================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter().filter(|&(_, pa)| 
					&plmaps[pa.layer_map_name].kind == &Thalamic) 
		{			
			// input_sources.push(InputSource::new(pa, &plmaps[pa.layer_map_name]));
			let is = InputSource::new(pa, &plmaps[pa.layer_map_name]);
			input_sources.insert((is.area_name(), is.tags()), is);
		}

		/*=============================================================================
		====================================== ALL ====================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter()
			// .filter(|&(_, pa)| plmaps[pa.layer_map_name].kind != Thalamic)
		{	
			let area_map = AreaMap::new(pa, &plmaps, &pamaps, &input_sources);

			let layer_info = area_map.output_layer_info();

			for &(tags, cols) in layer_info.iter() {
				tract.add_area(area_name, tags, cols as usize);
			}

			println!("{mt}{mt}THALAMUS::NEW(): Area: '{}', output layer info: {:?}.", 
				area_name, layer_info, mt = cmn::MT);
			assert!(layer_info.len() > 0, "Areas must have at least one afferent or efferent area.");

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
		for (&(area_name, tags), src) in self.input_sources.iter_mut() {
			let (ganglion, events) = self.tract.ganglion_mut(src.area_name(), 
				src.tags()).expect("Thalamus::cycle_external_ganglions()");
			src.cycle(ganglion, events);
		}		
	}

	// WRITE_INPUT(): <<<<< TODO: CHECK SIZES AND SCALE WHEN NECESSARY >>>>>
	// pub fn write_input(&self, sdr: &Sdr, area: &mut CorticalArea) {		
	// 	area.write_input(sdr, map::FF_IN);
	// }

	pub fn ganglion(&mut self, src_area_name: &'static str, layer_mask: LayerTags
			) -> Result<(&EventList, &Sdr), CmnError> 
	{ 		
		self.tract.ganglion(src_area_name, layer_mask)
	}

	pub fn ganglion_mut(&mut self, src_area_name: &'static str, layer_mask: LayerTags
			) -> Result<(&mut Sdr, &mut EventList), CmnError>
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
	// tract_areas: Vec<TractArea>
	// tract_area_cache: HashMap<(&'static str, LayerTags), usize>,	
	tract_areas: TractAreaCache,
	ttl_len: usize,
}

impl ThalamicTract {
	fn new() -> ThalamicTract {
		let ganglion = Vec::with_capacity(0);
		let tract_areas = TractAreaCache::new();

		ThalamicTract {
			ganglion: ganglion,
			tract_areas: tract_areas,
			ttl_len: 0,
		}
	}

	fn add_area(&mut self, src_area_name: &'static str, layer_tags: LayerTags, len: usize) {
		self.tract_areas.insert(src_area_name, layer_tags, 
			TractArea::new(src_area_name, layer_tags, self.ttl_len..(self.ttl_len + len)));
		self.ttl_len += len;
	}

	fn init(mut self) -> ThalamicTract {
		self.ganglion.resize(self.ttl_len, 0);
		// println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
		self
	}


	fn ganglion(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<(&EventList, &Sdr), CmnError>
	{
		let ta = try!(self.tract_areas.get(src_area_name, layer_tags));
		let range = ta.range();
		let events = ta.events();

		// let (range, events) = {
		// 	let ta = try!(self.tract_areas.get(src_area_name, layer_tags));
		// 	(ta.range(), ta.events())
		// };

		// try!(self.verify_range(&range, src_area_name));
		// println!(" ### ThalamicTract::ganglion({}, {:?}): range: {:?}", src_area_name, 
		// 	 layer_tags, range);
		
		Ok((events, &self.ganglion[range]))
	}

	fn ganglion_mut(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<(&mut Sdr, &mut EventList), CmnError>
	{
		let ta = try!(self.tract_areas.get_mut(src_area_name, layer_tags));
		let range = ta.range();
		let events = ta.events_mut();

		// let (range, events) = {
		// 	let ta = try!(self.tract_areas.get_mut(src_area_name, layer_tags));
		// 	(ta.range(), ta.events_mut())
		// };

		// try!(self.verify_range(&range, src_area_name));
		// println!(" ### ThalamicTract::ganglion_mut({}, {:?}): range: {:?}", src_area_name, 
		// 	 layer_tags, range);
		
		Ok((&mut self.ganglion[range], events))
	}

	// fn verify_range(&self, range: &Range<usize>, area_name: &'static str) -> Result<(), CmnError> {
	// 	if range.end > self.ganglion.len() {
	// 		Err(CmnError::new(format!("ThalamicTract::ganglion_mut(): Index range for target area: '{}' \
	// 			exceeds the boundaries of the input tract (length: {})", area_name, 
	// 			self.ganglion.len())))
	// 	} else {
	// 		Ok(())
	// 	}
	// }
}

struct TractAreaCache {
	areas: Vec<TractArea>,
	index: HashMap<(&'static str, LayerTags), usize>,
}

impl TractAreaCache {
	fn new() -> TractAreaCache {
		TractAreaCache {
			areas: Vec::with_capacity(32),
			index: HashMap::with_capacity(48),
		}
	}

	fn insert(&mut self, src_area_name: &'static str, layer_tags: LayerTags, tract_area: TractArea)
	{
		self.areas.push(tract_area);
		let dup_area = self.index.insert((src_area_name, layer_tags), (self.areas.len() - 1));
		assert!(dup_area.is_none(), "TractAreaCache::insert(): Cannot add two areas \
			with the same name and tags");
	}

	fn get(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<&TractArea, CmnError> 
	{
		match self.area_search(src_area_name, layer_tags) {
			Ok(idx) => self.areas.get(idx).ok_or(CmnError::new(format!("Index '{}' not found for '{}' \
				with tags '{:?}'", idx, src_area_name, layer_tags))),

			Err(err) => Err(err),
		}

		// self.area_search(src_area_name, layer_tags).map(|idx|
		// 	self.areas.get(idx).ok_or(format!("Index '{}' not found for '{}' \
		// 	with tags '{:?}'", idx, src_area_name, layer_tags)))
	}

	fn get_mut(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<&mut TractArea, CmnError> 
	{
		match self.area_search(src_area_name, layer_tags) {
			Ok(idx) => self.areas.get_mut(idx).ok_or(CmnError::new(format!("Index '{}' not \
				found for '{}' with tags '{:?}'", idx, src_area_name, layer_tags))),

			Err(err) => {
				Err(err)
			},
		}

		// self.area_search(src_area_name, layer_tags).map(|idx|
		// 	self.areas.get_mut(idx).ok_or(format!("Index '{}' not found for '{}' \
		// 	with tags '{:?}'", idx, src_area_name, layer_tags)))
	}

	fn area_search(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<usize, CmnError> 
	{
		// let cleared_tags = clear_io_tags(layer_tags);
		// println!("TractAreaCache::area_search(): Searching for area: {}, tags: {:?}. ALL: {:?}", 
		// 	src_area_name, layer_tags, self.areas);
		let area_idx = self.index.get(&(src_area_name, layer_tags)).map(|&idx| idx);

		// println!("   area_idx: {:?}", area_idx);

		let mut matching_areas: Vec<usize> = Vec::with_capacity(4);

		match area_idx {
			Some(idx) => return Ok(idx),

			None => {
				for i in 0..self.areas.len() {
					if self.areas[i].layer_tags.meshes(layer_tags) 
						&& self.areas[i].src_area_name == src_area_name
					{
						matching_areas.push(i);
					}
				}

				match matching_areas.len() {
					0 => return Err(CmnError::new(format!("No areas found with name: '{}' and tags: '{:?}'",
						src_area_name, layer_tags))),

					1 => {
						self.index.insert((src_area_name, layer_tags), matching_areas[0]);
						return Ok(matching_areas[0]);
					},

					_ => Err(CmnError::new(format!("Multiple tract areas found for area: '{}' with tags: '{:?}'. \
						Please use additional tags to specify tract area more precisely", 
						src_area_name, layer_tags))),
				}
			}
		}
	}
}

#[derive(Debug)]
struct TractArea {
	src_area_name: &'static str,
	layer_tags: LayerTags,
	range: Range<usize>,
	events: EventList,
}

impl TractArea {
	fn new(src_area_name: &'static str, layer_tags: LayerTags, range: Range<usize>) -> TractArea {
		TractArea { 
			src_area_name: src_area_name,
			layer_tags: layer_tags,
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


// Remove input and output tags and return result.
// [FIXME]: TODO: Verify tags?
// fn clear_io_tags(layer_tags: LayerTags) -> LayerTags {
// 	layer_tags & !(map::OUTPUT | map::INPUT)
// }

#[cfg(test)]
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


	/*	FORWARD_FF_OUT(): Read afferent output from a cortical area and store it 
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

	// 	//println!("\n ##### FORWARDING FEEDFORWARD OUTPUT from: '{}' to: '{}'", src_area_name, tar_area_name);

	// 	//if self.area_maps[

	// 	areas.get(src_area_name).expect(&emsg_src).read_output(
	// 		self.afferent_tract.output_ganglion(src_area_name, tar_area_name),
	// 		map::FF_OUT, 
	// 	);		
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.afferent_tract.input_ganglion(tar_area_name),
	// 		map::FF_IN,
	// 	);

	// }

	// pub fn read_afferent_output(&mut self, src_area_name &str, 

	// BACKWARD_FB_OUT():  Cause an efferent frame to descend
	// pub fn backward_efferent_output(&mut self, src_area_name: &str, tar_area_name: &str,
	// 			 areas: &mut CorticalAreas) 
	// {
	// 	let emsg = "thalamus::Thalamus::backward_efferent_output(): Area not found: ";
	// 	let emsg_src = format!("{}'{}' ", emsg, src_area_name);
	// 	let emsg_tar = format!("{}'{}' ", emsg, tar_area_name);

	// 	match areas.get(tar_area_name) {
	// 		Some(area) => if self.area_maps[tar_area_name].plmap().kind == Thalamic { return; },
	// 		None => return,
	// 	}

	// 	//println!("\n ##### BACKWARDING FEEDBACK OUTPUT from: '{}' to: '{}'", src_area_name, tar_area_name);
		
	// 	areas.get(src_area_name).expect(&emsg_src).read_output(
	// 		self.efferent_tract.output_ganglion(src_area_name, tar_area_name), 
	// 		map::FB_OUT,
	// 	);
	
	// 	/* TEST */
	// 	//let test_vec = input_czar::sdr_stripes(512, false, &mut self.efferent_tract[slc_range.clone()]);
		
	// 	areas.get_mut(tar_area_name).expect(&emsg_tar).write_input(
	// 		self.efferent_tract.input_ganglion(tar_area_name), 
	// 		map::FB_IN,
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

	// 	//let ref region = self.plmaps[&LayerMapKind::Sensory];
	// 	let region = area.plmaps();
	// 	let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
	// 	for slc in axn_slcs { 
	// 		//let buffer_offset = cmn::axn_idz_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
	// 		let buffer_offset = self.area_map.axn_idz(slc);
	// 		ocl::enqueue_write_buffer(sdr, area.axns.states.buf, area.ocl().queue(), buffer_offset);
	// 	}
	// }
	
