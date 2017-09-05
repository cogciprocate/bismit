//! Control Cells (Interneurons)

#![allow(unused_imports)]


use std::fmt::Debug;
use ocl::Buffer;
use cortex::Dendrites;
use cmn::{CmnResult, CorticalDims};
use map::{CellScheme, ExecutionGraph, LayerAddress};

pub trait ControlCellLayer: 'static + Debug + Send {
    fn set_exe_order_pre(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn set_exe_order_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn cycle_pre(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn cycle_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn layer_name(&self) -> &'static str;
    fn layer_addr(&self) -> LayerAddress;
    fn host_layer_addr(&self) -> LayerAddress;
}