//! Extra stuff I haven't found a better home for yet.
//!
//! - Much of it is temporary
//! - Some of it will be eventually moved to other modules
//! - Some of it may be moved to util


// #[macro_use] mod macros;
mod cortical_dims;
mod slice_dims;
mod tract_dims;
mod renderer;
mod error;
mod tract_frame;
mod map_store;
mod double_buffer_mutex;
// mod xorshiftrng;
pub mod util;
pub mod completion_pool;

use std;
use std::default::{Default};
use std::iter::{self};
use std::cmp::{self};
use std::io::{self, Write};
use std::collections::{BTreeMap, HashSet};
// use std::fmt::Debug;
// use std::ops::AddAssign;
use num::{FromPrimitive, };
// use num::{Num, NumCast};
use rand::{FromEntropy, rngs::SmallRng};
use rand::distributions::{Distribution, Range};
#[allow(unused_imports)]
use find_folder::Search;
use ocl::traits::OclScl;
use ocl::builders::ProgramBuilder;

pub use self::cortical_dims::{CorticalDims};
pub use self::slice_dims::SliceDims;
pub use self::tract_dims::TractDims;
// pub use self::data_cell_layer::{DataCellLayer};
pub use self::renderer::{Renderer};
pub use self::error::{CmnError, CmnResult};
pub use self::tract_frame::{TractFrame, TractFrameMut};

pub use self::map_store::MapStore;
pub use self::slice_dims::{calc_scale, scale};
pub use self::double_buffer_mutex::DoubleBufferMutex;

// // A clone of the counterpart types in the `rand` crate. Duplicated due to
// // some sort of bug with deriving `Debug`.
// pub use self::xorshiftrng::{weak_rng, SmallRng, Range, SampleRange};

// pub(crate) use self::completion_pool::UnparkMutex;


// pub trait ParaHexArray {
//     fn v_size(&self) -> u32;
//     fn u_size(&self) -> u32;
//     fn count(&self) -> u32;
// }

/// Type codes (synchronize with codes in 'cycle.py').
enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone)]
    pub enum TypeId {
        Float32 = 0,
        Float64 = 1,
        Int32   = 2,
        Int64   = 3,
    }
}


/// Types which can be represented as one or several stacked two-dimensional
/// parallelogram-shaped array containing hexagon-shaped elements.
pub trait ParaHexArray {
    fn depth(&self) -> u8;
    fn v_size(&self) -> u32;
    fn u_size(&self) -> u32;

    #[inline]
    fn len(&self) -> u32 {
        self.depth() as u32 * self.v_size() * self.u_size()
    }
}


//###### REENABLE WHEN SPECIALIZATION LANDS: ########
    // pub struct ParaHexArraySliceSize {
    //     v_size: u32,
    //     u_size: u32,
    // }

    // impl<T> From<T> for ParaHexArraySliceSize where T: ToPrimitive {
    //     default fn from(size: T) -> ParaHexArraySliceSize {
    //         ParaHexArraySliceSize {
    //             v_size: size.to_u32().unwrap(),
    //             u_size: size.to_u32().unwrap(),
    //         }
    //     }
    // }

    // impl<T> From<(T, T)> for ParaHexArraySliceSize where T: ToPrimitive {
    //     fn from(sizes: (T, T)) -> ParaHexArraySliceSize {
    //         ParaHexArraySliceSize {
    //             v_size: sizes.0.to_u32().unwrap(),
    //             u_size: sizes.1.to_u32().unwrap(),
    //         }
    //     }
    // }

    // impl<T> From<[T; 2]> for ParaHexArraySliceSize where T: ToPrimitive {
    //     fn from(sizes: [T; 2]) -> ParaHexArraySliceSize {
    //         ParaHexArraySliceSize {
    //             v_size: sizes[0].to_u32().unwrap(),
    //             u_size: sizes[1].to_u32().unwrap(),
    //         }
    //     }
    // }


pub type Sdr = [u8];

pub type AxnState = u8;
pub type SrcOfs = i8;
pub type SlcId = u8;



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

// pub static C_DEFAULT: &'static str = "\x1b[0m";
// pub static C_UNDER: &'static str = "\x1b[1m";
// pub static C_RED: &'static str = "\x1b[31m";
// pub static C_BRED: &'static str = "\x1b[1;31m";
// pub static C_GRN: &'static str = "\x1b[32m";
// pub static C_BGRN: &'static str = "\x1b[1;32m";
// pub static C_ORA: &'static str = "\x1b[33m";
// pub static C_DBL: &'static str = "\x1b[34m";
// pub static C_PUR: &'static str = "\x1b[35m";
// pub static C_CYA: &'static str = "\x1b[36m";
// pub static C_LGR: &'static str = "\x1b[37m";
// pub static C_DGR: &'static str = "\x1b[90m";
// pub static C_LRD: &'static str = "\x1b[91m";
// pub static C_YEL: &'static str = "\x1b[93m";
// pub static C_BLU: &'static str = "\x1b[94m";
// pub static C_MAG: &'static str = "\x1b[95m";
// pub static C_LBL: &'static str = "\x1b[94m";
// pub static BGC_DEFAULT: &'static str = "\x1b[49m";
// pub static BGC_GRN: &'static str = "\x1b[42m";
// pub static BGC_PUR: &'static str = "\x1b[45m";
// pub static BGC_LGR: &'static str = "\x1b[47m";
// pub static BGC_DGR: &'static str = "\x1b[100m";

// COLORS DISABLED (SWITCH TO ANOTHER METHOD):
pub static C_DEFAULT: &'static str = "";
pub static C_UNDER: &'static str = "";
pub static C_RED: &'static str = "";
pub static C_BRED: &'static str = "";
pub static C_GRN: &'static str = "";
pub static C_BGRN: &'static str = "";
pub static C_ORA: &'static str = "";
pub static C_DBL: &'static str = "";
pub static C_PUR: &'static str = "";
pub static C_CYA: &'static str = "";
pub static C_LGR: &'static str = "";
pub static C_DGR: &'static str = "";
pub static C_LRD: &'static str = "";
pub static C_YEL: &'static str = "";
pub static C_BLU: &'static str = "";
pub static C_MAG: &'static str = "";
pub static C_LBL: &'static str = "";
pub static BGC_DEFAULT: &'static str = "";
pub static BGC_GRN: &'static str = "";
pub static BGC_PUR: &'static str = "";
pub static BGC_LGR: &'static str = "";
pub static BGC_DGR: &'static str = "";


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
pub const DENDRITE_DEFAULT_INITIAL_THRESHOLD: u32 = 0;

//pub const LEARNING_ACTIVE: bool = true;
pub const SYNAPSE_REACH_MAX: i8 = std::i8::MAX;
pub const SYNAPSE_REACH_MIN: i8 = std::i8::MIN + 1;
pub const SYNAPSE_STRENGTH_FLOOR: i8 = -25;             // DIRECTLY AFFECTS LEARNING RATE
pub const SYNAPSE_REGROWTH_INTERVAL: usize = 200;         // DIRECTLY AFFECTS LEARNING RATE
pub const SYNAPSE_STRENGTH_INITIAL_DEVIATION: i8 = 5;
pub const DST_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;
pub const PRX_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;
pub const MAX_HRZ_DIM_SIZE: u32 = 255;

// Scaling coefficient. Higher values create more potential precision.
// 16 (2^4: 1 << 4) seems like plenty.
pub const SLC_SCL_COEFF_L2: i32 = 4;
pub const SLC_SCL_COEFF: usize = 1 << SLC_SCL_COEFF_L2 as usize;


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

pub const SYNAPSE_REACH: u32 = 8; // UNUSED?
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

pub const MCOL_IS_VATIC_FLAG: u8   = 0b00000001;

pub const CEL_PREV_CONCRETE_FLAG: u8   = 0b10000000;    // 128   (0x80)
pub const CEL_BEST_IN_COL_FLAG: u8     = 0b01000000;    // 64    (0x40)
pub const CEL_PREV_STPOT_FLAG: u8      = 0b00100000;    // 32    (0x20)
pub const CEL_PREV_VATIC_FLAG: u8      = 0b00010000;    // 16    (0x10)
pub const CEL_PREV_ACTIVE_FLAG: u8     = 0b00001000;    // 8     (0x08)

pub const SYN_STDEP_FLAG: u8        = 0b00000001;
pub const SYN_STPOT_FLAG: u8        = 0b00000010;
pub const SYN_CONCRETE_FLAG: u8     = 0b00001000;
pub const SYN_PREV_ACTIVE_FLAG: u8  = 0b00010000;

pub const DEN_BASAL_PROXIMAL_FLAG: u8   = 0b00000001;
pub const DEN_BASAL_DISTAL_FLAG: u8     = 0b00000010;
pub const DEN_APICAL_DISTAL_FLAG: u8    = 0b00000100;



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



// LOAD_BUILTIN_KERNEL_FILES(): MUST BE CALLED AFTER ANY CUSTOM KERNEL FILES ARE LOADED.
//        -Used by AreaMap
pub fn load_builtin_kernel_source<'b>(build_options: &mut ProgramBuilder<'b>) {
    // STATIC KERNEL SOURCE: Use normally:
    {
        pub static BUILTIN_OPENCL_PROGRAM_SOURCE: [&'static str; 5] = [
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/bismit.cl")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/syns.cl")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/control.cl")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/filters.cl")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cl/tests.cl")),
        ];

        for i in 0..BUILTIN_OPENCL_PROGRAM_SOURCE.len() {
            build_options.src(BUILTIN_OPENCL_PROGRAM_SOURCE[i]);
        }
    }

    // // DYNAMIC KERNEL SOURCE: Use for faster kernel iteration:
    // {
    //     // Loaded in reverse order:
    //     static BUILTIN_OPENCL_KERNEL_FILE_NAMES: [&'static str; 5] = [
    //         "bismit.cl",
    //         "syns.cl",
    //         "control.cl",
    //         "filters.cl",
    //         "tests.cl",
    //     ];
    //     let src_path_root = Search::ParentsThenKids(3, 3).for_folder("cl").unwrap();
    //     for file_name in BUILTIN_OPENCL_KERNEL_FILE_NAMES.iter() {
    //         build_options = build_options.src_file(src_path_root.clone().join(file_name));
    //     }
    // }

}

//    BASE_BUILD_OPTIONS():
//         -Used by AreaMap.
pub fn base_build_options<'b>() -> ProgramBuilder<'b> {

    //assert!(SENSORY_CHORD_COLUMNS % AXON_BUFFER_SIZE == 0);
    /*assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
    assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
    assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
    assert!(DENDRITES_PER_CELL_DISTAL <= 256);
    assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);*/

    let mut pb = ProgramBuilder::new();
    pb.cmplr_opt(OPENCL_BUILD_SWITCHES);
    // pb.cmplr_def("SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2", SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 as i32);
    // pb.cmplr_def("DENDRITES_PER_CELL_DISTAL_LOG2", DENDRITES_PER_CELL_DISTAL_LOG2 as i32);
    // pb.cmplr_def("DENDRITES_PER_CELL_DISTAL", DENDRITES_PER_CELL_DISTAL as i32);
    // pb.cmplr_def("DENDRITES_PER_CELL_PROXIMAL_LOG2", DENDRITES_PER_CELL_PROXIMAL_LOG2 as i32);

    // pb.cmplr_def("COLUMN_DOMINANCE_FLOOR", COLUMN_DOMINANCE_FLOOR as i32);
    // pb.cmplr_def("SYNAPSE_STRENGTH_FLOOR", SYNAPSE_STRENGTH_FLOOR as i32);
    // pb.cmplr_def("DENDRITE_INITIAL_THRESHOLD_PROXIMAL", DENDRITE_INITIAL_THRESHOLD_PROXIMAL as i32);
    // pb.cmplr_def("SYNAPSES_PER_CELL_PROXIMAL_LOG2", SYNAPSES_PER_CELL_PROXIMAL_LOG2 as i32);
    // pb.cmplr_def("ASPINY_REACH_LOG2", ASPINY_REACH_LOG2 as i32);
    // pb.cmplr_def("AXON_MARGIN_SIZE", AXON_MARGIN_SIZE as i32);
    // pb.cmplr_def("AXON_BUFFER_SIZE", AXON_BUFFER_SIZE as i32);
    // pb.cmplr_def("SYNAPSE_SPAN_RHOMBAL_AREA", SYNAPSE_SPAN_RHOMBAL_AREA as i32);
    // pb.cmplr_def("ASPINY_REACH", ASPINY_REACH as i32);
    // pb.cmplr_def("ASPINY_SPAN_LOG2", ASPINY_SPAN_LOG2 as i32);
    // pb.cmplr_def("ASPINY_SPAN", ASPINY_SPAN as i32);

    pb.cmplr_def("MCOL_IS_VATIC_FLAG", MCOL_IS_VATIC_FLAG as i32);
    pb.cmplr_def("CEL_PREV_CONCRETE_FLAG", CEL_PREV_CONCRETE_FLAG as i32);
    pb.cmplr_def("CEL_BEST_IN_COL_FLAG", CEL_BEST_IN_COL_FLAG as i32);
    pb.cmplr_def("CEL_PREV_STPOT_FLAG", CEL_PREV_STPOT_FLAG as i32);
    pb.cmplr_def("CEL_PREV_VATIC_FLAG", CEL_PREV_VATIC_FLAG as i32);
    pb.cmplr_def("CEL_PREV_ACTIVE_FLAG", CEL_PREV_ACTIVE_FLAG as i32);
    pb.cmplr_def("SYN_STPOT_FLAG", SYN_STPOT_FLAG as i32);
    pb.cmplr_def("SYN_STDEP_FLAG", SYN_STDEP_FLAG as i32);
    pb.cmplr_def("SYN_CONCRETE_FLAG", SYN_CONCRETE_FLAG as i32);
    pb.cmplr_def("SYN_PREV_ACTIVE_FLAG", SYN_PREV_ACTIVE_FLAG as i32);
    pb.cmplr_def("DEN_BASAL_PROXIMAL_FLAG", DEN_BASAL_PROXIMAL_FLAG as i32);
    pb.cmplr_def("DEN_BASAL_DISTAL_FLAG", DEN_BASAL_DISTAL_FLAG as i32);
    pb.cmplr_def("DEN_APICAL_DISTAL_FLAG", DEN_APICAL_DISTAL_FLAG as i32);
    pb
}




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
    debug_assert!(slc_id < depth);
    debug_assert!(v_id < v_size);
    debug_assert!(u_id < u_size);
    (slc_id as u32 * v_size * u_size) + (v_id * u_size) + u_id
}


// List of offsets to form a hexagon-shaped pattern of tiles.
pub fn hex_tile_offs(radius: SrcOfs) -> Vec<(SrcOfs, SrcOfs)> {
    assert!(radius >= 0);

    let tile_count = (3 * radius as usize) * (radius as usize + 1) + 1;
    let mut mold = Vec::with_capacity(tile_count);

    let radius: SrcOfs = radius as SrcOfs;

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

// List of offsets to form a hexagon-shaped pattern of tiles.
pub fn hex_tile_offs_scaled(radius: SrcOfs) -> Vec<(SrcOfs, SrcOfs)> {
    assert!(radius >= 0);

    let tile_count = (3 * radius as usize) * (radius as usize + 1) + 1;
    let mut mold = Vec::with_capacity(tile_count);

    let radius: SrcOfs = radius as SrcOfs;

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


/* SPARSE_VEC():

    sp_fctr_log2: sparsity factor (log2)
*/
pub fn sparse_vec<T: OclScl>(size: usize, min_val: T, max_val: T, sp_fctr_log2: usize) -> Vec<T> {
    let mut vec: Vec<T> = iter::repeat(min_val).cycle().take(size).collect();

    let len = vec.len();

    let notes = len >> sp_fctr_log2;

    let range_max: i64 = max_val.to_i64().expect("cmn::sparse_vec(): max_val.to_i64()") as i64 + 1;
    let range_min: i64 = min_val.to_i64().expect("cmn::sparse_vec(): min_val.to_i64()") as i64;

    let mut rng = SmallRng::from_entropy();
    let val_range = Range::new(range_min, range_max);
    let idx_range = Range::new(0, 1 << sp_fctr_log2);

    for i in 0..notes {
        vec[(i << sp_fctr_log2) + idx_range.sample(&mut rng)] = FromPrimitive::from_i64(val_range.sample(&mut rng)).expect("cmn::sparse_vec()");
        //vec[(i << sp_fctr_log2) + idx_range.sample(&mut rng)] = std::num::cast(val_range.sample(&mut rng)).expect("cmn.rs");
    }

    vec
}


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


/// Evenly distributed hexagon-tile-group center coordinates ([v, u]).
pub struct HexGroupCenters {
    centers: HashSet<[i32; 2]>,
    new_centers: HashSet<[i32; 2]>,
    // Hex group side length.
    radius: i32,
    // Determines whether surrounding groups are biased in the clockwise or
    // counterclockwise direction.
    clockwise_bias: bool,
    // Lower parallelogram bound ([v, u]) closed (inclusive).
    l_bound: [i32; 2],
    // Upper parallelogram bound ([v, u]) open (exclusive).
    u_bound: [i32; 2],
}

impl HexGroupCenters {
    /// Returns a new `HexGroupCenters`.
    ///
    /// `radius` sets the side length for each hexagon-tile group (in
    /// tiles). `l_bound` and `u_bound` set the lower and upper boundaries
    /// (creating a parallelogram).
    pub fn new(radius: i32, l_bound: [i32; 2], u_bound: [i32; 2]) -> HexGroupCenters {
        const CLOCKWISE_BIAS: bool = true;

        HexGroupCenters {
            centers: HashSet::new(),
            new_centers: HashSet::new(),
            radius,
            clockwise_bias: CLOCKWISE_BIAS,
            l_bound: l_bound,
            u_bound: u_bound,
        }
    }

    //
    // pub fn bounds(mut self, l_bound: [i32; 2], u_bound: [i32; 2]) -> HexGroupCenters {
    //     self.l_bound = Some(l_bound);
    //     self.u_bound = Some(u_bound);
    //     self
    // }

    /// Adds a coordinate to the `centers` set.
    ///
    /// If lower or upper boundaries are set, checks against them. If `center`
    /// is a new (unique) coordinate, adds to the `new_centers` set.
    pub fn add_center(&mut self, center: [i32; 2]) {
        if center[0] < self.l_bound[0] || center[1] < self.l_bound[1] ||
                center[0] >= self.u_bound[0] || center[1] >= self.u_bound[1]
        {
            return;
        }

        if self.centers.insert(center) {
            self.new_centers.insert(center);
        }
    }

    /// Adds all of the surrounds for a point to the `centers` set.
    ///
    /// If the surround coordinate is not already in the `centers` set, adds
    /// it to the `new_centers` set as well.
    pub fn add_surrounds(&mut self, center: [i32; 2]) {
        let (l, s) = if self.clockwise_bias {
            (self.radius + 1, self.radius)
        } else {
            (self.radius, self.radius + 1)
        };
        let ls = l + s;

        // u: 4, vi: 0, w: -3
        // v: -3, u: 7
        // new.push([center[0] - 3, center[1] + 7]);
        self.add_center([center[0] - s, center[1] + ls]);
        // u: 3, vi: -4, w: 0
        // v: 4, u: 3
        // new.push([center[0] + 4, center[1] + 3]);
        self.add_center([center[0] + l, center[1] + s]);
        // u: 0, vi: -3, w: 4
        // v: 7, u: -4
        // new.push([center[0] + 7, center[1] - 4]);
        self.add_center([center[0] + ls, center[1] - l]);
        // u: -4, vi: 0, w: 3
        // v: 3, u: -7
        // new.push([center[0] + 3, center[1] - 7]);
        self.add_center([center[0] + s, center[1] - ls]);
        // u: -3, vi: 4, w: 0
        // v: -4, u: -3
        // new.push([center[0] - 4, center[1] - 3]);
        self.add_center([center[0] - l, center[1] - s]);
        // u: 0, vi: 3, w: -4
        // v: -7, u: 4
        // new.push([center[0] - 7, center[1] + 4]);
        self.add_center([center[0] - ls, center[1] + l]);
    }

    /// Populates the set of group centers.
    ///
    /// If `center` is specified, that coordinate is used as the starting
    /// seed. If `center` is unspecified and no seed coordinates have been
    /// added, this function will return without doing anything.
    pub fn populate(&mut self, start: Option<[i32; 2]>) {
        if let Some(cntr) = start {
            self.add_center(cntr);
        }
        while self.new_centers.len() > 0 {
            let mut new_cntrs = HashSet::new();
            ::std::mem::swap(&mut self.new_centers, &mut new_cntrs);

            for cntr in new_cntrs {
                self.add_surrounds(cntr);
            }
        }
    }

    /// Converts the internal group centers list into a `Vec`.
    pub fn to_vec(&self) -> Vec<[i32; 2]> {
        self.centers.iter().cloned().collect()
    }

    /// Converts the internal group centers list into two `Vec`s, one for each
    /// coord.
    pub fn to_vecs(&self) -> (Vec<i32>, Vec<i32>) {
        let mut vcoords = Vec::with_capacity(self.centers.len());
        let mut ucoords = Vec::with_capacity(self.centers.len());

        for center in self.centers.iter() {
            vcoords.push(center[0]);
            ucoords.push(center[1]);
        }

        (vcoords, ucoords)
    }

    // Returns a reference to the centers set.
    pub fn set(&self) -> &HashSet<[i32; 2]> { &self.centers }
}


/// Populates a parallelogram-shaped (diamond-shaped) hex-tile area with the
/// centers of non-overlapping hexagonally shaped groups.
///
/// Useful for placing cells within a cortical layer without overlapping
/// nearby cells. This can provide small, evenly distributed regions (groups)
/// without having to keep track of arbitrarily addressed (positioned)
/// sources.
///
/// `radius` is the circumradius (= side length) of each hexagon group
/// measured in tiles.
///
pub fn populate_hex_tile_grps(radius: usize, dims: [i32; 2], start: [i32; 2], val: u8, sdr: &mut [u8]) {
    let dims = [dims[0] as i32, dims[1] as i32];

    let mut centers = HexGroupCenters::new(radius as i32, [0, 0], dims);
    centers.populate(Some(start));

    for cntr in centers.centers {
        let idx = (cntr[0] * dims[1] as i32) + cntr[1];
        assert!(idx >= 0);
        sdr[idx as usize] = val;
    }
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
