
pub use self::regions::{ Protoregion, Protoregions, ProtoregionKind };
	pub use self::regions::ProtoregionKind::{ Associational, Sensory, Motor }; // SLATED FOR REDESIGN

pub use self::areas::{ Protoareas, ProtoareasTrait, Protoarea };

pub use self::layer::{ Protolayer, ProtolayerKind, ProtoaxonKind };
	pub use self::layer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::layer::ProtoaxonKind::{ Spatial, Horizontal };

pub use self::cell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };
	pub use self::cell::ProtocellKind::{ Pyramidal, SpinyStellate, Inhibitory };

pub mod filter;
pub mod regions;
pub mod areas;
pub mod layer;
pub mod cell;
