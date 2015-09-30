/* src/cmn: Common: extra stuff I haven't found a better home for yet
	- Much of it is temporary
	- Some of it will be eventually moved to other modules
	- Some of it may remain and be renamed to utils or some such
*/

pub use self::cmn::*;
pub use self::cortical_dimensions::{ CorticalDimensions };
pub use self::slice_dimensions::{ SliceDimensions };
//pub use self::area_map::{ AreaMap, SliceMap };
pub use self::renderer::{ Renderer };
//pub use self::prediction::*;

mod cmn;
mod cortical_dimensions;
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
