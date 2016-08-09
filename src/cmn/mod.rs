/* src/cmn: Common: extra stuff I haven't found a better home for yet
    - Much of it is temporary
    - Some of it will be eventually moved to other modules
    - Some of it may remain and be renamed to utils or some such
*/


#[macro_use] mod macros;
mod cortical_dims;
mod slice_dims;
mod tract_dims;
mod renderer;
mod error;
mod tract_frame;
pub mod data_cell_layer;

use num::{FromPrimitive};
use std::default::{Default};
use std::iter::{self};
use std::cmp::{self};
use std::io::{self, Write};
use std::collections::{BTreeMap};
// use std::path::PathBuf;
use rand;
use rand::distributions::{IndependentSample, Range};
// use find_folder::Search;
use ocl::traits::OclScl;
use ocl::builders::ProgramBuilder;

pub use self::cortical_dims::{CorticalDims};
pub use self::slice_dims::SliceDims;
pub use self::tract_dims::TractDims;
pub use self::data_cell_layer::{DataCellLayer};
pub use self::renderer::{Renderer};
pub use self::error::{CmnError};
pub use self::tract_frame::{TractFrame, TractFrameMut};
#[cfg(test)] pub use self::data_cell_layer::tests::{CelCoords, DataCellLayerTest};

// pub trait ParaHexArray {
//     fn v_size(&self) -> u32;
//     fn u_size(&self) -> u32;
//     fn count(&self) -> u32;
// }

/// Types which can be represented as one or several stacked two-dimensional
/// parallelogram-shaped array containing hexagon-shaped elements.
pub trait ParaHexArray {
    fn v_size(&self) -> u32;
    fn u_size(&self) -> u32;
    fn depth(&self) -> u8;

    #[inline]
    fn len(&self) -> u32 {
        self.v_size() * self.u_size() * self.depth() as u32
    }
}


pub type Sdr = [u8];

pub type CmnResult<T> = Result<T, CmnError>;


/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
================================== COMMON =====================================
========================= YES, IT'S A MESS IN HERE ============================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/



/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
================================= CONSTANTS ===================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/


// MT: Mini-tab: 4 spaces ('mini' compared to the huge tab on certain terminals)
pub static MT: &'static str = "    ";

pub static C_DEFAULT: &'static str = "\x1b[0m";
pub static C_UNDER: &'static str = "\x1b[1m";

pub static C_RED: &'static str = "\x1b[31m";
pub static C_BRED: &'static str = "\x1b[1;31m";
pub static C_GRN: &'static str = "\x1b[32m";
pub static C_BGRN: &'static str = "\x1b[1;32m";
pub static C_ORA: &'static str = "\x1b[33m";
pub static C_DBL: &'static str = "\x1b[34m";
pub static C_PUR: &'static str = "\x1b[35m";
pub static C_CYA: &'static str = "\x1b[36m";
pub static C_LGR: &'static str = "\x1b[37m";
pub static C_DGR: &'static str = "\x1b[90m";
pub static C_LRD: &'static str = "\x1b[91m";
pub static C_YEL: &'static str = "\x1b[93m";
pub static C_BLU: &'static str = "\x1b[94m";
pub static C_MAG: &'static str = "\x1b[95m";
pub static C_LBL: &'static str = "\x1b[94m";


pub static BGC_DEFAULT: &'static str = "\x1b[49m";
pub static BGC_GRN: &'static str = "\x1b[42m";
pub static BGC_PUR: &'static str = "\x1b[45m";
pub static BGC_LGR: &'static str = "\x1b[47m";
pub static BGC_DGR: &'static str = "\x1b[100m";


// pub const DEFAULT_HORIZONTAL_SLICE_SIDE: u32 = 32;
pub const DEFAULT_OUTPUT_LAYER_DEPTH: u8 = 1;

// pub const SENSORY_CHORD_WIDTH_LOG2: usize = 5;
// pub const SENSORY_CHORD_WIDTH: u32 = 1 << SENSORY_CHORD_WIDTH_LOG2;
// pub const SENSORY_CHORD_HEIGHT_LOG2: usize = 5;
// pub const SENSORY_CHORD_HEIGHT: u32 = 1 << SENSORY_CHORD_HEIGHT_LOG2;
// pub const SENSORY_CHORD_COLUMNS_LOG2: usize = SENSORY_CHORD_WIDTH_LOG2 + SENSORY_CHORD_HEIGHT_LOG2;
// pub const SENSORY_CHORD_COLUMNS: u32 = 1 << SENSORY_CHORD_COLUMNS_LOG2;

/*pub const DENDRITES_PER_CELL_DISTAL_LOG2: u8 = 1;
pub const DENDRITES_PER_CELL_DISTAL: u32 = 1 << DENDRITES_PER_CELL_DISTAL_LOG2 as u32;
pub const SYNAPSES_PER_DENDRITE_DISTAL_LOG2: u8 = 3;
pub const SYNAPSES_PER_DENDRITE_DISTAL: u32 = 1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2 as u32;
pub const DENDRITES_PER_CELL_PROXIMAL_LOG2: u8 = 0;
pub const DENDRITES_PER_CELL_PROXIMAL: u32 = 1 << DENDRITES_PER_CELL_PROXIMAL_LOG2 as u32;
pub const SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2: u8 = 3;
pub const SYNAPSES_PER_DENDRITE_PROXIMAL: u32 = 1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 as u32;*/

//pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u32 = (128 * 4);
//pub const DENDRITE_INITIAL_THRESHOLD_DISTAL: u32 = (128 * 1);

//pub const LEARNING_ACTIVE: bool = true;
pub const SYNAPSE_STRENGTH_FLOOR: i8 = -25;             // DIRECTLY AFFECTS LEARNING RATE
pub const SYNAPSE_REGROWTH_INTERVAL: usize = 800;         // DIRECTLY AFFECTS LEARNING RATE
pub const SYNAPSE_STRENGTH_INITIAL_DEVIATION: i8 = 5;
pub const DST_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;
pub const PRX_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;

//pub const CORTICAL_SEGMENTS_TOTAL: usize = 1;
//pub const SENSORY_SEGMENTS_TOTAL: usize = 1;
//pub const MOTOR_SEGMENTS_TOTAL: usize = 1;
//pub const HYPERCOLUMNS_PER_SEGMENT: usize = 16;
//pub const COLUMNS_PER_HYPERCOLUMN: u32 = 64;

// pub const SYNAPSE_REACH_GEO_LOG2: u32 = 3;
// pub const SYNAPSE_REACH_GEO: u32 = 1 << SYNAPSE_REACH_GEO_LOG2;
// pub const SYNAPSE_SPAN_GEO: u32 = SYNAPSE_REACH_GEO << 1;
// pub const AXON_MAR__GIN_SIZE: u32 = (1 << (((SYNAPSE_REACH_GEO_LOG2 + 1) << 1) - 1));    // ((AXON_BUFFER_SIZE ^ 2) / 2)
// pub const AXON_BUF__FER_SIZE: u32 = (1 << ((SYNAPSE_REACH_GEO_LOG2 + 1) << 1));    // (AXON_BUFFER_SIZE ^ 2)

pub const SYNAPSE_REACH: u32 = 8;
pub const SYNAPSE_SPAN: u32 = SYNAPSE_REACH * 2; // TESTING PURPOSES
pub const SYNAPSE_SPAN_RHOMBAL_AREA: u32 = SYNAPSE_SPAN * SYNAPSE_SPAN; // TESTING PURPOSES
//pub const SYNAPSE_REACH_CELLS: u32 = (3 * (SYNAPSE_REACH * SYNAPSE_REACH)) + (3 * SYNAPSE_REACH) + 1;

//pub const AXON_MARGIN_SIZE: u32 = (SYNAPSE_REACH * SYNAPSE_REACH) + SYNAPSE_REACH;
//pub const AXON_BUFFER_SIZE: u32 = AXON_MARGIN_SIZE * 2;
pub const AXON_MARGIN_SIZE: u32 = 0; // DEPRICATE


pub const MAX_FEEDFORWARD_AREAS: usize = 256;

/*pub const AXON_MAR__GIN_SIZE_LOG2: u32 = 7;
pub const AXON_MAR__GIN_SIZE: u32 = 1 << AXON_MAR__GIN_SIZE_LOG2;
pub const AXON_BUF__FER_SIZE: u32 = AXON_MAR__GIN_SIZE << 1;
pub const AXON_MAR__GIN_SIZE: usize = AXON_MAR__GIN_SIZE as usize;*/


pub const SYNAPSE_ROW_POOL_SIZE: u32 = 256;

    /* GET RID OF THIS UNLESS CL NEEDS IT */
//pub const SYNAPSES_PER_CELL_PROXIMAL_LOG2: u32 = DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
//pub const SYNAPSES_PER_CELL_PROXIMAL: u32 = 1 << SYNAPSES_PER_CELL_PROXIMAL_LOG2;

//pub const AXONS_PER_CELL: usize = DENDRITES_PER_CELL * SYNAPSES_PER_DENDRITE;
//pub const SYNAPSES_PER_CELL: usize = SYNAPSES_PER_DENDRITE * DENDRITES_PER_CELL;

//pub const COLUMNS_PER_SEGMENT: usize = COLUMNS_PER_HYPERCOLUMN as usize * HYPERCOLUMNS_PER_SEGMENT;
//pub const COLUMN_AXONS_PER_SEGMENT: usize = AXONS_PER_CELL * COLUMNS_PER_SEGMENT;
//pub const COLUMN_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_CELL * COLUMNS_PER_SEGMENT;
//pub const COLUMN_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * COLUMN_DENDRITES_PER_SEGMENT;

//pub const CELLS_PER_SEGMENT: usize = LAYERS_PER_SEGMENT * COLUMNS_PER_SEGMENT;
//pub const CELL_AXONS_PER_SEGMENT: usize = AXONS_PER_CELL * CELLS_PER_SEGMENT;
//pub const CELL_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_CELL * CELLS_PER_SEGMENT;
//pub const CELL_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * CELL_DENDRITES_PER_SEGMENT;

//pub const LAYERS_PER_SEGMENT: usize = 16;
//pub const CELLS_PER_LAYER: usize = COLUMNS_PER_SEGMENT;
//pub const DENDRITES_PER_LAYER: usize = CELLS_PER_LAYER * DENDRITES_PER_CELL;
//pub const SYNAPSES_PER_LAYER: usize = CELLS_PER_LAYER * SYNAPSES_PER_CELL;

//pub const DST_DEN_BOOST_LOG2: u8 = 0;
//pub const PRX_DEN_BOOST_LOG2: u8 = 0;


// OVERWRITEN BY KERNEL CONSTANT - NEEDS UPDATE AND SYNTHESIS
// pub const ASPINY_REACH_LOG2: u8             = 2;
// pub const ASPINY_REACH:    u32                    = 1 << ASPINY_REACH_LOG2;
// pub const ASPINY_SPAN_LOG2: u8                 = ASPINY_REACH_LOG2 + 1;
// pub const ASPINY_SPAN: u32                     = 1 << ASPINY_SPAN_LOG2;
// pub const ASPINY_HEIGHT: u8 = 1;
pub const COLUMN_DOMINANCE_FLOOR: usize = 7;


pub const STATE_ZERO: u8 = 0;

pub const OPENCL_PREFERRED_VECTOR_MULTIPLE: u32 = 4;
pub const OPENCL_PREFERRED_WORKGROUP_SIZE: u32 = 256;
pub const OPENCL_MINIMUM_WORKGROUP_SIZE: u32 = 64;
pub const SYNAPSES_WORKGROUP_SIZE: u32 = OPENCL_PREFERRED_WORKGROUP_SIZE;
//pub const AXONS_WORKGROUP_SIZE: u32 = OPENCL_PREFERRED_WORKGROUP_SIZE;

pub const MCOL_IS_VATIC_FLAG: u8             = 0b00000001;

pub const CEL_PREV_CONCRETE_FLAG: u8         = 0b10000000;    // 128    (0x80)
pub const CEL_BEST_IN_COL_FLAG: u8             = 0b01000000;    // 64    (0x40)
pub const CEL_PREV_STPOT_FLAG: u8             = 0b00100000;    // 32    (0x20)
pub const CEL_PREV_VATIC_FLAG: u8            = 0b00010000;    // 16    (0x10)

pub const SYN_STDEP_FLAG: u8                = 0b00000001;
pub const SYN_STPOT_FLAG: u8                = 0b00000010;
pub const SYN_CONCRETE_FLAG: u8                = 0b00001000;




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/






//pub static BUILTIN_OPENCL_KERNEL_FILE_NAME: &'static str = "bismit.cl";
//pub static BUILTIN_FILTERS_CL_FILE_NAME: &'static str = "filters.cl";
static OPENCL_BUILD_SWITCHES: &'static str = "-cl-denorms-are-zero -cl-fast-relaxed-math";

// // BUILTIN_OPENCL_KERNEL_FILE_NAMES: Loaded in reverse order.
// pub static BUILTIN_OPENCL_KERNEL_FILE_NAMES: [&'static str; 4] = [
//     "tests.cl",
//     "filters.cl",
//     "syns.cl",
//     "bismit.cl",
// ];

// BUILTIN_OPENCL_KERNEL_FILE_NAMES: Loaded in reverse order.
pub static BUILTIN_OPENCL_PROGRAM_SOURCE: [&'static str; 4] = [
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/bismit.cl")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/syns.cl")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/filters.cl")),
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/tests.cl")),
];

// LOAD_BUILTIN_KERNEL_FILES(): MUST BE CALLED AFTER ANY CUSTOM KERNEL FILES ARE LOADED.
//        -Used by AreaMap
// [FIXME]: TEMPORARY: determine path non-retardedly...
pub fn load_builtin_kernel_source(mut build_options: ProgramBuilder) -> ProgramBuilder {
    // for i in 0..BUILTIN_OPENCL_KERNEL_FILE_NAMES.len() {
    //     build_options = build_options.src_file(
    //         cl_root_path().join(BUILTIN_OPENCL_KERNEL_FILE_NAMES[i]));
    // }
    for i in 0..BUILTIN_OPENCL_PROGRAM_SOURCE.len() {
        build_options = build_options.src(BUILTIN_OPENCL_PROGRAM_SOURCE[i]);
    }

    build_options
}

//    BASE_BUILD_OPTIONS():
//         -Used by AreaMap.
pub fn base_build_options() -> ProgramBuilder {

    //assert!(SENSORY_CHORD_COLUMNS % AXON_BUFFER_SIZE == 0);
    /*assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
    assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
    assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
    assert!(DENDRITES_PER_CELL_DISTAL <= 256);
    assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);*/

    ProgramBuilder::new()
        .cmplr_opt(OPENCL_BUILD_SWITCHES)
        // .cmplr_def("SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2", SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 as i32)
        // .cmplr_def("DENDRITES_PER_CELL_DISTAL_LOG2", DENDRITES_PER_CELL_DISTAL_LOG2 as i32)
        // .cmplr_def("DENDRITES_PER_CELL_DISTAL", DENDRITES_PER_CELL_DISTAL as i32)
        // .cmplr_def("DENDRITES_PER_CELL_PROXIMAL_LOG2", DENDRITES_PER_CELL_PROXIMAL_LOG2 as i32)

        // .cmplr_def("COLUMN_DOMINANCE_FLOOR", COLUMN_DOMINANCE_FLOOR as i32)
        // .cmplr_def("SYNAPSE_STRENGTH_FLOOR", SYNAPSE_STRENGTH_FLOOR as i32)
        // .cmplr_def("DENDRITE_INITIAL_THRESHOLD_PROXIMAL", DENDRITE_INITIAL_THRESHOLD_PROXIMAL as i32)
        // .cmplr_def("SYNAPSES_PER_CELL_PROXIMAL_LOG2", SYNAPSES_PER_CELL_PROXIMAL_LOG2 as i32)
        // .cmplr_def("ASPINY_REACH_LOG2", ASPINY_REACH_LOG2 as i32)
        // .cmplr_def("AXON_MARGIN_SIZE", AXON_MARGIN_SIZE as i32)
        // .cmplr_def("AXON_BUFFER_SIZE", AXON_BUFFER_SIZE as i32)
        // .cmplr_def("SYNAPSE_SPAN_RHOMBAL_AREA", SYNAPSE_SPAN_RHOMBAL_AREA as i32)
        // .cmplr_def("ASPINY_REACH", ASPINY_REACH as i32)
        // .cmplr_def("ASPINY_SPAN_LOG2", ASPINY_SPAN_LOG2 as i32)
        // .cmplr_def("ASPINY_SPAN", ASPINY_SPAN as i32)

        .cmplr_def("MCOL_IS_VATIC_FLAG", MCOL_IS_VATIC_FLAG as i32)
        .cmplr_def("CEL_PREV_CONCRETE_FLAG", CEL_PREV_CONCRETE_FLAG as i32)
        .cmplr_def("CEL_BEST_IN_COL_FLAG", CEL_BEST_IN_COL_FLAG as i32)
        .cmplr_def("CEL_PREV_STPOT_FLAG", CEL_PREV_STPOT_FLAG as i32)
        .cmplr_def("CEL_PREV_VATIC_FLAG", CEL_PREV_VATIC_FLAG as i32)
        .cmplr_def("SYN_STPOT_FLAG", SYN_STPOT_FLAG as i32)
        .cmplr_def("SYN_STDEP_FLAG", SYN_STDEP_FLAG as i32)
        .cmplr_def("SYN_CONCRETE_FLAG", SYN_CONCRETE_FLAG as i32)
}


// // [FIXME]: TEMPORARY
// pub fn cl_root_path() -> PathBuf {
//     // PathBuf::from("/home/nick/projects/bismit/cl")
//     Search::ParentsThenKids(3, 3).for_folder("cl")
//         .expect("bismit::cmn::cl_root_path()")
// }




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/


// CEL_IDX_3D(): Simple index resolution for a cell within a particular layer
pub fn cel_idx_3d(depth: u8, slc_id: u8, v_size: u32, v_id: u32, u_size: u32, u_id: u32) -> u32 {
    assert!(slc_id < depth);
    assert!(v_id < v_size);
    assert!(u_id < u_size);
    (slc_id as u32 * v_size * u_size) + (v_id * u_size) + u_id
}



pub fn hex_tile_offs(radius: u8) -> Vec<(i8, i8)> {
    assert!(radius <= 127);

    let tile_count = (3 * radius as usize) * (radius as usize + 1) + 1;
    let mut mold = Vec::with_capacity(tile_count);

    let radius: i8 = radius as i8;

    let v_ofs_z = 0 - radius;
    let v_ofs_n = radius + 1;

    for v_ofs in v_ofs_z..v_ofs_n {
        let v_ofs_inv = 0 - v_ofs;
        let u_ofs_z = cmp::max(0 - radius, v_ofs_inv - radius);
        let u_ofs_n = cmp::min(radius, v_ofs_inv + radius) + 1;
        //print!("[v_ofs:{}]", v_ofs);

        for u_ofs in u_ofs_z..u_ofs_n {
            mold.push((v_ofs, u_ofs));
        }
    }

    mold
}


// pub fn print_vec_simple<T: OclPrm>(vec: &[T]) {
//     print_vec(vec, 1, None, None, true);
// }


// pub fn print_vec<T: OclPrm>(
//             vec: &[T],
//             every: usize,
//             val_range: Option<(T, T)>,
//             idx_range: Option<(usize, usize)>,
//             show_zeros: bool,
// ) {


//     /*let val_range = match val_range {
//         Some(x) => x,
//         _ => 0,
//     }*/
//     let (ir_start, ir_end) = match idx_range {
//         Some(ir)    => ir,
//         None        => (0usize, 0usize),
//     };

//     let (vr_start, vr_end) = match val_range {
//         Some(vr)    => vr,
//         None        => (Default::default(), Default::default()),
//     };

//     let mut ttl_nz = 0usize;
//     let mut ttl_ir = 0usize;
//     let mut within_idx_range = true;
//     let mut within_val_range = true;
//     let mut hi: T = vr_start;
//     let mut lo: T = vr_end;
//     let mut sum: i64 = 0;
//     let mut ttl_prntd: usize = 0;
//     let len = vec.len();


//     let mut color: &'static str = C_DEFAULT;
//     let mut prnt: bool = false;

//     print!("{cdgr}[{cg}{}{cdgr}/{}", vec.len(), every, cg = C_GRN, cdgr = C_DGR);

//     if val_range.is_some() {
//         print!(";[{},{}]", vr_start, vr_end);
//     }

//     if idx_range.is_some() {
//                  // DUPLICATE
//         print!(";[{},{}]", ir_start, ir_end);
//     }
//     print!("]:{cd} ", cd = C_DEFAULT,);


//         /* Yes, this clusterfuck needs rewriting someday */
//     for i in 0..vec.len() {

//         prnt = false;

//         if every != 0 {
//             if i % every == 0 {
//                 prnt = true;
//             } else {
//                 prnt = false;
//             }
//         }

//         if idx_range.is_some() {
//             let ir = idx_range.as_ref().expect("cmn.rs");

//             if i < ir_start || i >= ir_end {
//                 prnt = false;
//                 within_idx_range = false;
//             } else {
//                 within_idx_range = true;
//             }
//         } else {
//             within_idx_range = true;
//         }

//         if val_range.is_some() {
//             if vec[i] < vr_start || vec[i] >= vr_end {
//                 prnt = false;
//                 within_val_range = false;
//             } else {
//                 if within_idx_range {
//                     if vec[i] == Default::default() {
//                         ttl_ir += 1;
//                     } else {
//                         ttl_ir += 1;
//                     }
//                 }

//                 within_val_range = true;
//             }
//         }

//         if within_idx_range && within_val_range {
//             sum += vec[i].to_i64().expect("ocl::fmt::print_vec(): vec[i]");

//             if vec[i] > hi { hi = vec[i] };

//             if vec[i] < lo { lo = vec[i] };

//             if vec[i] != Default::default() {
//                 ttl_nz += 1usize;
//                 color = C_ORA;
//             } else {
//                 if show_zeros {
//                     color = C_DEFAULT;
//                 } else {
//                     prnt = false;
//                 }
//             }
//         }

//         if prnt {
//             print!("{cg}[{cd}{}{cg}:{cc}{}{cg}]{cd}", i, vec[i], cc = color, cd = C_DEFAULT, cg = C_DGR);
//             ttl_prntd += 1;
//         }
//     }

//     let mut anz: f32 = 0f32;
//     let mut nz_pct: f32 = 0f32;

//     let mut ir_pct: f32 = 0f32;
//     let mut avg_ir: f32 = 0f32;

//     if ttl_nz > 0 {
//         anz = sum as f32 / ttl_nz as f32;
//         nz_pct = (ttl_nz as f32 / len as f32) * 100f32;
//         //print!("[ttl_nz: {}, nz_pct: {:.0}%, len: {}]", ttl_nz, nz_pct, len);
//     }

//     if ttl_ir > 0 {
//         avg_ir = sum as f32 / ttl_ir as f32;
//         ir_pct = (ttl_ir as f32 / len as f32) * 100f32;
//         //print!("[ttl_nz: {}, nz_pct: {:.0}%, len: {}]", ttl_nz, nz_pct, len);
//     }


//     println!("{cdgr}:(nz:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),\
//         ir:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),hi:{},lo:{},anz:{:.2},prntd:{}){cd} ",
//         ttl_nz, nz_pct, ttl_ir, ir_pct, hi, lo, anz, ttl_prntd, cd = C_DEFAULT, clbl = C_LBL, cdgr = C_DGR);
// }

// pub fn shuffled_vec<T: OclPrm>(size: usize, min_val: T, max_val: T) -> Vec<T> {

//     //println!("min_val: {}, max_val: {}", min_val, max_val);

//     //let min: i64 = num::cast(min_val).expect("cmn::shuffled_vec(), min");
//     //let max: i64 = num::cast::<T, i64>(max_val).expect("cmn::shuffled_vec(), max") + 1is;
//     //let size: usize = num::cast(max_val - min_val).expect("cmn::shuffled_vec(), size");
//     //let size: usize = num::from_int(max - min).expect("cmn::shuffled_vec(), size");

//     //assert!(max - min > 0, "Vector size must be greater than zero.");
//     let mut vec: Vec<T> = Vec::with_capacity(size);

//     assert!(size > 0, "\ncmn::shuffled_vec(): Vector size must be greater than zero.");
//     assert!(min_val < max_val, "\ncmn::shuffled_vec(): Minimum value must be less than maximum.");

//     let min = min_val.to_i64().expect("\ncmn::shuffled_vec(), min");
//     let max = max_val.to_i64().expect("\ncmn::shuffled_vec(), max") + 1;

//     let mut range = (min..max).cycle();

//     for i in (0..size) {
//         vec.push(FromPrimitive::from_i64(range.next().expect("\ncmn::shuffled_vec(), range")).expect("\ncmn::shuffled_vec(), from_usize"));
//     }

//     //let mut vec: Vec<T> = (min..max).cycle().take(size).collect();


//     /*let mut vec: Vec<T> = iter::range_inclusive::<T>(min_val, max_val).cycle().take(size).collect();*/


//     shuffle_vec(&mut vec);

//     vec

// }

// // Fisher-Yates
// pub fn shuffle_vec<T: OclPrm>(vec: &mut Vec<T>) {
//     let len = vec.len();
//     let mut rng = rand::weak_rng();

//     let mut ridx: usize;
//     let mut tmp: T;

//     for i in 0..len {
//         ridx = distributions::Range::new(i, len).ind_sample(&mut rng);
//         tmp = vec[i];
//         vec[i] = vec[ridx];
//         vec[ridx] = tmp;
//     }
// }

/* SPARSE_VEC():

    sp_fctr_log2: sparsity factor (log2)
*/
pub fn sparse_vec<T: OclScl>(size: usize, min_val: T, max_val: T, sp_fctr_log2: usize) -> Vec<T> {
    let mut vec: Vec<T> = iter::repeat(min_val).cycle().take(size).collect();

    let len = vec.len();

    let notes = len >> sp_fctr_log2;

    let range_max: i64 = max_val.to_i64().expect("cmn::sparse_vec(): max_val.to_i64()") as i64 + 1;
    let range_min: i64 = min_val.to_i64().expect("cmn::sparse_vec(): min_val.to_i64()") as i64;

    let mut rng = rand::weak_rng();
    let val_range = Range::new(range_min, range_max);
    let idx_range = Range::new(0, 1 << sp_fctr_log2);

    for i in 0..notes {
        vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = FromPrimitive::from_i64(val_range.ind_sample(&mut rng)).expect("cmn::sparse_vec()");
        //vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = std::num::cast(val_range.ind_sample(&mut rng)).expect("cmn.rs");
    }

    vec
}

// pub fn dup_check<T: OclPrm>(in_vec: &mut Vec<T>) -> (usize, usize) {


//     let mut vec = in_vec.clone();

//     vec.sort();


//     let mut dups = 0usize;
//     let mut unis = 0usize;
//     let mut prev_val = vec[vec.len() - 1];

//     for x in vec.iter() {
//         if prev_val == *x {
//             dups += 1;
//             //print!{"[{}]", *x};
//         } else {
//             unis += 1;
//         }
//         prev_val = *x;
//     }

//     println!("len: {}, dups: {}, unis: {}", vec.len(), dups, unis);
//     (dups, unis)
// }


/*pub fn log2(n: u32) -> u32 {
    let mut t = n;
    t = t | t >> 1;
    t = t | t >> 2;
    t = t | t >> 4;
    t = t | t >> 8;
    t = t | t >> 16;
    assert!((t - (t >> 1)).trailing_zeros() == t.trailing_zeros());

    (t - (t >> 1)).trailing_zeros()
}*/


pub fn log2(n: u32) -> u32 {
    if n > 0 {
        31 - n.leading_zeros()
    } else {
        0
    }
}

// RENDER_SDR_SQUARE(): Show SDR in a square grid -- DEPRICATE (hex version in tests/renderer)
pub fn render_sdr_square(
            vec_out: &Sdr,
            vec_ff_opt: Option<&Sdr>,
            vec_out_prev_opt: Option<&Sdr>,
            vec_ff_prev_opt: Option<&Sdr>,
            slc_map: &BTreeMap<u8, &'static str>,
            print: bool,
            sdr_len: u32,
) -> f32 {
    let vec_ff = match vec_ff_opt {
        Some(v) => v,
        None => vec_out.clone(),
    };

    let vec_out_prev = match vec_out_prev_opt {
        Some(v) => v,
        None => vec_out.clone(),
    };

    let vec_ff_prev = match vec_ff_prev_opt {
        Some(v) => v,
        None => vec_out.clone(),
    };

    //println!("vec_ff.len(): {}, vec_out.len(): {}", vec_ff.len(), vec_out.len());

    assert!(vec_ff.len() == vec_out.len(), "cmn::render_sdr(): vec_ff.len() != vec_out.len(), Input vectors must be of equal length.");
    assert!(vec_out.len() == vec_out_prev.len(), "cmn::render_sdr(): vec_out.len() != vec_out_prev.len(), Input vectors must be of equal length.");
    assert!(vec_out.len() == vec_ff_prev.len(), "cmn::render_sdr(): vec_out.len() != vec_ff_prev.len(), Input vectors must be of equal length.");


    let mut active_cols = 0usize;
    let mut failed_preds = 0usize;
    let mut corr_preds = 0usize;
    let mut anomalies = 0usize;
    let mut new_preds = 0usize;
    let mut ttl_active = 0usize;

    let cortical_area_per_line = 64;
    let line_character_width = (cortical_area_per_line * (4 + 4 + 2 + 4 + 4 + 1)) + 8;    // 8 extra for funsies

    //println!("\n[{}{}{}]:", C_GRN, vec_ff.len(), C_DEFAULT);

    let mut out_line: String = String::with_capacity(line_character_width);
    let mut i_line = 0usize;
    let mut i_global = 0usize;
    // let mut i_pattern = 0usize; // DEPRICATE
    let mut i_cort_area = 0u8;

    println!("");
    io::stdout().flush().unwrap();

    loop {
        if i_line >= vec_out.len() { break }

        out_line.clear();

        for i in i_line..(i_line + cortical_area_per_line) {
            let cur_active = vec_out[i] != Default::default();
            let col_active = vec_ff[i] != Default::default();
            let prediction = vec_out[i] != vec_ff[i];
            let new_prediction = prediction && (!col_active);

            //let prev_active = vec_ff_prev[i] != Default::default();
            let prev_prediction = new_pred(vec_out_prev[i], vec_ff_prev[i]);

            if col_active {
                active_cols += 1;
            }

            if new_prediction {
                new_preds += 1;
            }

            if (prev_prediction && !new_prediction) && !col_active {
                failed_preds += 1;
            } else if prev_prediction && col_active {
                corr_preds += 1;
            }

            if col_active && !prev_prediction {
                anomalies += 1;
            }

            if print {
                if cur_active {
                    if prediction {
                        out_line.push_str(BGC_DGR);
                    }

                    if new_prediction {
                        //assert!(new_pred(vec_out[i], vec_ff[i]));
                        out_line.push_str(C_MAG);
                    } else {
                        out_line.push_str(C_BLU);
                    }
                    /*if corr_pred(vec_out[i], vec_ff[i], vec_out_prev[i], vec_ff_prev[i]) {
                        corr_preds += 1;
                    }*/
                } else {
                    out_line.push_str(C_DEFAULT);
                }

                if cur_active {
                    out_line.push_str(&format!("{:02X}", vec_out[i]));
                    ttl_active += 1;
                } else {
                    if (i & 0x07) == 0 || (i_global & 0x07) == 0 {                // || ((i_global & 0x0F) == 7) || ((i_global & 0x0F) == 8)
                        out_line.push_str("  ");
                    } else {
                        out_line.push_str("--");
                    }
                }

                out_line.push_str(C_DEFAULT);
                out_line.push_str(BGC_DEFAULT);
                out_line.push_str(" ");
            }
        }


        if print {
            if ((i_line % sdr_len as usize) == 0) && (vec_ff.len() > sdr_len as usize) {
                let slc_id = (i_cort_area) as u8;

                let slc_name = match slc_map.get(&slc_id) {
                    Some(&name) => name,
                    None => "<render_sdr(): slc name not found in map>",
                };

                println!("\n[{}: {}]", slc_id, slc_name);
                i_cort_area += 1;
                // i_pattern = 0; // DEPRICATE
            } else {
                // i_pattern += 1; // DEPRICATE
            }

            println!("{}", out_line);
        }

        i_line += cortical_area_per_line;
        i_global += 1;
    }


    let preds_total = (corr_preds + failed_preds) as f32;

    let pred_accy = if preds_total > 0f32 {
        (corr_preds as f32 / preds_total) * 100f32
    } else {
        0f32
    };

    if print {
        if vec_out_prev_opt.is_some() {
            println!("\nprev preds:{} (correct:{}, incorrect:{}, accuracy:{:.1}%), anomalies:{}, cols active:{}, ttl active:{}, new_preds:{}",
                preds_total, corr_preds, failed_preds, pred_accy, anomalies, active_cols, ttl_active, new_preds,);
        }
    }

    pred_accy
}


pub fn corr_pred(
            out: u8,
            ff: u8,
            prev_out: u8,
            prev_ff: u8,
) -> Option<bool> {
    let prev_new_pred = new_pred(prev_out, prev_ff);
    let curr_new_pred = new_pred(out, ff);

    if prev_new_pred && (ff != 0) {
        Some(true)
    } else if prev_new_pred && curr_new_pred {
        None
    } else {
        Some(false)
    }
}


pub fn new_pred(
            out: u8,
            ff: u8,
) -> bool {
    // let out_active = out != 0;
    let ff_active = ff != 0;
    let pred = out != ff;
    let new_pred = pred && (!ff_active);

    new_pred
}


/*fn pred_accy<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex>(
            vec_out: &[T],
            vec_ff: &[T],
            prev_vec: &[T],
) -> f32 {
    assert!(vec_out.len() == vec_ff.len() && vec_out.len() == prev_vec.len());

    let len = vec_out.len();
    let mut corr_pred = 0usize;
    let mut icor_pred = 0usize;

    for i in 0..len {


    }

}*/


/* GEN_FRACT_SDR(): Generate simple SDR from integer seed
    - FUTURE IMPROVEMENTS:
        - Once the Rust API for wrapping integers is sorted out, use one of those instead of wrap_idx.
        - Create and store sdr as a "chord" or whatever else becomes the preferred SDR storage container

*/
pub fn gen_fract_sdr(seed: u8, len: usize) -> Vec<u8> {
    let mut vec: Vec<u8> = iter::repeat(0u8).take(len).collect();

    let mut idx = wrap_idx(seed as usize, len);
    let n = 1 + ((len >> 5) as f64).sqrt() as usize;

    for i in 0..n {
        for _ in 0..(n - i) {
            vec[idx] = 1;
            idx = wrap_idx(idx + 1, len)
        }
        idx = wrap_idx(idx + (i << 1) + 1, len);
    }

    vec
}

pub fn wrap_idx(idx: usize, len: usize) -> usize {
    let mut w_idx = idx;
    loop {
        if w_idx < len {
            break;
        } else {
            w_idx -= len;
        }
    }
    w_idx
}




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
================================ UNIT TESTS ===================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/




#[cfg(test)]
pub mod tests {
    use super::*;

    // #[test]
    // fn axn_idz_2d() {
    //     assert!(axn_idz_2d(1, 1024, 4) == 1024u32 + AXON_MARGIN_SIZE as u32);
    //     assert!(axn_idz_2d(5, 1024, 4) == 4096u32 + SYNAPSE_SPAN_RHOMBAL_AREA + AXON_MARGIN_SIZE as u32);
    //     assert!(axn_idz_2d(15, 1024, 4) == 4096u32 + (11 * SYNAPSE_SPAN_RHOMBAL_AREA) + AXON_MARGIN_SIZE as u32);

    // }

    #[test]
    fn test_wrap_idx() {
        assert!(wrap_idx(50, 40) == 10);
        assert!(wrap_idx(30, 40) == 30);
    }

    #[test]
    fn test_log2() {
        assert!(log2(126) == 6);
        assert!(log2(128) == 7);
        assert!(log2(129) == 7);
        assert!(log2(7) == 2);
        assert!(log2(8) == 3);
        assert!(log2(9) == 3);
    }
}




// THIS WORKS BUT HAVE TO ASSIGN ALL THE SLICES TO IT BEFORE USE
// pub struct Sdr([u8]);

// impl Deref for Sdr {
//     type Target = [u8];
//     fn deref(&self) -> &[u8] { &self.0 }
// }

// impl DerefMut for Sdr {
//     fn deref_mut(&mut self) -> &mut [u8] { &mut self.0 }
// }


// struct Board([[Square; 8]; 8]);
// And to keep all base type's methods, impl Deref (and DerefMut):
// impl Deref<[[Square; 8]; 8]> for Board {
//     fn deref(&self) -> &[[Square, 8]; 8] { &self.0 }
// }
