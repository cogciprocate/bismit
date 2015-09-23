use proto::{ ProtolayerMaps, ProtolayerMap, Protoarea };

pub struct AreaMap {
	protoarea: Protoarea,
	protoregion: ProtolayerMap,

	// Create maps for each aspect which have their own types and are queryable 
	// into sub-lists of the same type

	// layers: LayerMap
	// slices: SliceMap
	// etc...

	// other new types: TuftMap/CellMap

}

impl AreaMap {
	pub fn new(protoregions: &ProtolayerMaps, protoarea: &Protoarea) -> AreaMap {
		let protoarea = protoarea.clone();			
		let mut protoregion = protoregions[protoarea.region_name].clone();
		protoregion.freeze(&protoarea);	

		AreaMap {
			protoarea: protoarea,
			protoregion: protoregion,

		}

	}

	pub fn protoarea(&self) -> &Protoarea {
		&self.protoarea
	}

	pub fn protoregion(&self) -> &ProtolayerMap {
		&self.protoregion
	}

}
