/* src/cmn: Common: extra stuff I haven't found a better home for yet
	- Much of it is temporary
	- Some of it will be eventually moved to other modules
	- Some of it may remain and be renamed to utils or some such
*/
//use std::ops::{ Deref, DerefMut };

pub use self::cmn::*;
pub use self::cortical_dimensions::{ CorticalDimensions };
pub use self::slice_dimensions::{ SliceDimensions };
//pub use self::area_map::{ AreaMap, SliceMap };
pub use self::data_cell_layer::{ DataCellLayer };
pub use self::renderer::{ Renderer };
//pub use self::prediction::*;

#[cfg(test)]
pub use self::data_cell_layer::tests::{ CelCoords };

mod cmn;
mod cortical_dimensions;
mod data_cell_layer;
//mod area_map;
mod slice_dimensions;
mod renderer;
//pub mod input_source;
//mod prediction;


pub trait HexTilePlane {
	fn v_size(&self) -> u32;
	fn u_size(&self) -> u32;
	fn count(&self) -> u32;
}

pub type Sdr = [u8];



// THIS WORKS BUT HAVE TO ASSIGN ALL THE SLICES TO IT BEFORE USE
// pub struct Sdr([u8]);

// impl Deref for Sdr {
// 	type Target = [u8];
// 	fn deref(&self) -> &[u8] { &self.0 }
// }

// impl DerefMut for Sdr {
// 	fn deref_mut(&mut self) -> &mut [u8] { &mut self.0 }
// }


// struct Board([[Square; 8]; 8]);
// And to keep all base type's methods, impl Deref (and DerefMut):
// impl Deref<[[Square; 8]; 8]> for Board {
//     fn deref(&self) -> &[[Square, 8]; 8] { &self.0 }
// }
