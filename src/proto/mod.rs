// PROTO: Not really a prototype but `WhateverBlueprint` is a bit cumbersome.
// Possible replacement names: `WhateverDraft`, `WhateverModel`.

//pub use self::layer_maps::{  };

pub use self::proto_layer_map::{ ProtolayerMaps, ProtolayerMap, RegionKind };
	pub use self::proto_layer_map::RegionKind::{ Associational, Sensory, Motor, Thalamic }; // SLATED FOR REDESIGN

pub use self::proto_area_map::{ ProtoareaMaps, ProtoareaMap };

pub use self::protolayer as layer;
pub use self::protolayer::{ Protolayer, ProtolayerKind, ProtoaxonKind };
	pub use self::protolayer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::protolayer::ProtoaxonKind::{ Spatial, Horizontal };

pub use self::protocell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };
	pub use self::protocell::ProtocellKind::{ Pyramidal, SpinyStellate, Inhibitory };

pub use self::protofilter::{ Protofilter };

pub use self::protoinput::{ Protoinput };

mod protofilter;
//pub mod regions;
mod proto_layer_map;
mod proto_area_map;
pub mod protolayer;
mod protocell;
mod protoinput;

