use std::ops::{ Range };
use std::collections::{ HashMap };

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
			let is = InputSource::new(pa, &plmaps[pa.layer_map_name]);
			input_sources.insert((is.area_name(), is.tags()), is).map(|is| panic!("Duplicate \
				'InputSource' keys: (area: \"{}\", tags: '{:?}')", is.area_name(), is.tags()));
		}

		/*=============================================================================
		====================================== ALL ====================================
		=============================================================================*/
		for (&area_name, pa) in pamaps.maps().iter() {	
			let area_map = AreaMap::new(pa, &plmaps, &pamaps, &input_sources);

			let layer_info = area_map.output_layer_info();

			for &(tags, cols) in layer_info.iter() {
				tract.add_area(area_name, tags, cols as usize);
			}

			println!("{mt}{mt}THALAMUS::NEW(): Area: \"{}\", output layer info: {:?}.", 
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

// THALAMICTRACT: A buffer for I/O between areas. Effectively analogous to the internal capsule.
pub struct ThalamicTract {
	ganglion: Vec<ocl::cl_uchar>,
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
		
		Ok((events, &self.ganglion[range]))
	}

	fn ganglion_mut(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<(&mut Sdr, &mut EventList), CmnError>
	{
		let ta = try!(self.tract_areas.get_mut(src_area_name, layer_tags));
		let range = ta.range();
		let events = ta.events_mut();
		
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

		self.index.insert((src_area_name, layer_tags), (self.areas.len() - 1))
			.map(|is| panic!("Duplicate 'TractAreaCache' keys: (area: \"{}\", tags: '{:?}')", 
				src_area_name, layer_tags));
	}

	fn get(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<&TractArea, CmnError> 
	{
		match self.area_search(src_area_name, layer_tags) {
			Ok(idx) => self.areas.get(idx).ok_or(CmnError::new(format!("Index '{}' not found for '{}' \
				with tags '{:?}'", idx, src_area_name, layer_tags))),

			Err(err) => Err(err),
		}
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
	}

	fn area_search(&mut self, src_area_name: &'static str, layer_tags: LayerTags
			) -> Result<usize, CmnError> 
	{
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

					_ => Err(CmnError::new(format!("Multiple tract areas found for area: '{}' \
						with tags: '{:?}'. Please use additional tags to specify tract area more \
						precisely", src_area_name, layer_tags))),
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


#[cfg(test)]
pub mod tests {
	
}
