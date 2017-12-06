//! Control Cells (Interneurons)


use std::fmt::Debug;
use std::collections::BTreeMap;
// use ocl::ProQue;
use cmn::CmnResult;
use map::{/*AreaMap,*/ LayerAddress, ExecutionGraph, /*CellScheme*/};
// use cortex::{AxonSpace, DataCellLayer, CorticalAreaSettings};

pub trait ControlCellLayer: 'static + Debug + Send {
    fn set_exe_order_pre(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn set_exe_order_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn cycle_pre(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn cycle_post(&mut self, exe_graph: &mut ExecutionGraph, host_lyr_addr: LayerAddress) -> CmnResult<()>;
    fn layer_name<'s>(&'s self) -> &'s str;
    fn layer_addr(&self) -> LayerAddress;
    fn host_layer_addr(&self) -> LayerAddress;
}

pub type ControlCellLayers = BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>;