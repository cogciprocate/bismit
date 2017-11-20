// PROTO: Not really a prototype but `WhateverBlueprint` is a bit cumbersome.
// Possible replacement names: `WhateverDraft`, `WhateverModel`.

//pub use self::layer_maps::{  };


mod filter;
mod layer_map;
mod area;
mod layer;
mod cell;
mod input;

pub use self::layer_map::{LayerMapSchemeList, LayerMapScheme};
    // pub use self::layer_scheme_map::LayerMapKind::{Cortical, Thalamic}; // SLATED FOR REDESIGN

pub use self::area::{AreaSchemeList, AreaScheme};

pub use self::layer::{LayerScheme, LayerKind};
    // pub use self::layer_map::LayerKind::{Cellular, Axonal};
    // pub use self::layer_map::AxonTopology::{Spatial, Horizontal};

pub use self::cell::{TuftScheme, TuftSourceLayer, CellScheme};
    // pub use self::cell::CellKind::{Pyramidal, SpinyStellate, Inhibitory, Complex};
    // pub use self::cell::CellClass::{Data, Control};

pub use self::filter::{FilterScheme};

pub use self::input::{EncoderScheme};
