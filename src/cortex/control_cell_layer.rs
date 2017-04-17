#![allow(unused_imports)]

use ocl::Buffer;
use cortex::Dendrites;
use cmn::{CmnResult, CorticalDims};
use map::{CellScheme, ExecutionGraph};

pub trait ControlCellLayer {
    fn set_exe_order(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()>;
    fn cycle(&mut self, exe_graph: &mut ExecutionGraph, bypass: bool) -> CmnResult<()>;
    fn layer_name(&self) -> &'static str;
    fn layer_id(&self) -> usize;
}