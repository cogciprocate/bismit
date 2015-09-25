/* src/cmn: Common: extra stuff I haven't found a better home for yet
	- Much of it is temporary
	- Some of it will be eventually moved to other modules
	- Some of it may remain and be renamed to utils or some such
*/

pub use self::cmn::*;
pub use self::cortical_dimensions::{ CorticalDimensions };
pub use self::area_map::{ AreaMap, SliceMap };
pub use self::renderer::{ Renderer };
//pub use self::prediction::*;

mod cmn;
mod cortical_dimensions;
mod area_map;
mod renderer;
//mod prediction;
