// PROTO: More of a blueprint than a prototype

pub use self::regions::{ Protoregions };

pub use self::region::{ Protoregion, ProtoregionKind };
	pub use self::region::ProtoregionKind::{ Associational, Sensory, Motor, Thalamic }; // SLATED FOR REDESIGN

pub use self::areas::{ Protoareas, ProtoareasTrait, Protoarea };

pub use self::layer::{ Protolayer, ProtolayerKind, ProtoaxonKind };
	pub use self::layer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::layer::ProtoaxonKind::{ Spatial, Horizontal };

pub use self::cell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };
	pub use self::cell::ProtocellKind::{ Pyramidal, SpinyStellate, Inhibitory };

pub use self::filter::{ Protofilter };

pub mod filter;
pub mod regions;
pub mod region;
pub mod areas;
pub mod layer;
pub mod cell;
