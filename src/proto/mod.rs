
pub use self::regions::{ Protoregion, Protoregions, ProtoregionKind };
pub use self::areas::{ Protoareas, ProtoareasTrait, Protoarea };
pub use self::layer::{ Protolayer };
	pub use self::layer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::layer::ProtoaxonKind::{ Spatial, Horizontal };
pub use self::cell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };

pub mod regions;
pub mod areas;
pub mod layer;
pub mod cell;
