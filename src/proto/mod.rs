// PROTO: More of a blueprint than a prototype

//pub use self::layer_maps::{  };

pub use self::proto_layer_map::{ ProtoLayerMaps, ProtoLayerMap, RegionKind };
	pub use self::proto_layer_map::RegionKind::{ Associational, Sensory, Motor, Thalamic }; // SLATED FOR REDESIGN

pub use self::proto_area_map::{ ProtoAreaMaps, ProtoAreaMap };

pub use self::protolayer as layer;
pub use self::protolayer::{ Protolayer, ProtolayerKind, ProtoaxonKind, ProtolayerFlags };
	pub use self::protolayer::ProtolayerKind::{ Cellular, Axonal };
	pub use self::protolayer::ProtoaxonKind::{ Spatial, Horizontal };

pub use self::protocell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };
	pub use self::protocell::ProtocellKind::{ Pyramidal, SpinyStellate, Inhibitory };

pub use self::protofilter::{ Protofilter };

mod protofilter;
//pub mod regions;
mod proto_layer_map;
mod proto_area_map;
pub mod protolayer;
mod protocell;



#[derive(PartialEq, Debug, Clone)]
pub enum Protoinput {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader { file_name: &'static str, repeats: usize, scale: f64 },
	None,
}
