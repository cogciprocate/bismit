// PROTO: More of a blueprint than a prototype

pub use self::regions::{ ProtolayerMaps };

pub use self::layer_map::{ ProtolayerMap, RegionKind };
	pub use self::layer_map::RegionKind::{ Associational, Sensory, Motor, Thalamic }; // SLATED FOR REDESIGN

pub use self::areas::{ Protoareas, Protoarea, Protoinput };

pub use self::layer::{ Protolayer, ProtolayerKind, ProtoaxonKind };
	pub use self::layer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::layer::ProtoaxonKind::{ Spatial, Horizontal };

pub use self::cell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };
	pub use self::cell::ProtocellKind::{ Pyramidal, SpinyStellate, Inhibitory };

pub use self::filter::{ Protofilter };

pub mod filter;
pub mod regions;
pub mod layer_map;
pub mod areas;
pub mod layer;
pub mod cell;
