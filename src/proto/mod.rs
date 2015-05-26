
pub use self::regions::{ ProtoRegion, ProtoRegions, ProtoRegionKind };
pub use self::areas::{ ProtoAreas, ProtoAreasTrait, ProtoArea };
pub use self::layer::{ ProtoLayer };
	pub use self::layer::ProtoLayerKind::{ Cellular, Axonal };
	pub use self::layer::AxonKind::{ Spatial, Horizontal };
pub use self::cell::{ CellKind, Protocell, DendriteKind, CellFlags };

pub mod regions;
pub mod areas;
pub mod layer;
pub mod cell;
