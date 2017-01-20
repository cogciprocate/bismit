// use std::ops::Range;
use std::error;
use std::fmt;
use ocl::{Event, Buffer, OclPrm};
use cmn::{util, CmnError, CmnResult};

pub enum ExecutionGraphError {
    InvalidCommandIndex(usize),
    InvalidRequisiteCommandIndex(usize, usize),
}

impl error::Error for ExecutionGraphError {
    fn description(&self) -> &str {
        match *self {
            ExecutionGraphError::InvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::InvalidRequisiteCommandIndex(..) => "Invalid command index.",
        }
    }
}

impl fmt::Display for ExecutionGraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExecutionGraphError::InvalidCommandIndex(cmd_idx) => {
                f.write_fmt(format_args!("Invalid command index (cmd_idx: {}).", cmd_idx))
            },
            ExecutionGraphError::InvalidRequisiteCommandIndex(req_cmd_idx, cmd_idx) => {
                f.write_fmt(format_args!("Invalid requisite command index (req_cmd_idx: {}, \
                    cmd_idx: {}).", req_cmd_idx, cmd_idx))
            },
        }
    }
}

impl fmt::Debug for ExecutionGraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}


/// A block of memory within the Cortex.
#[derive(Debug)]
pub enum CorticalBuffer {
    AxonSlice { buffer_id: u64, area_id: usize, slice_id: u8 },
    DataCellSynapseTuft { buffer_id: u64, area_id: usize, layer_id: usize, tuft_id: usize, },
    DataCellDendriteTuft { buffer_id: u64, area_id: usize, layer_id: usize, tuft_id: usize },
    DataCellSomaLayer { buffer_id: u64, area_id: usize, layer_id: usize, },
    ControlCellSomaLayer { buffer_id: u64, area_id: usize, layer_id: usize },
}

impl CorticalBuffer {
    pub fn axon_slice<T: OclPrm>(buf: &Buffer<T>, area_id: usize, slice_id: u8) -> CorticalBuffer {
        CorticalBuffer::AxonSlice {
            buffer_id: util::buffer_uid(buf),
            area_id: area_id,
            slice_id: slice_id,
        }
    }

    pub fn data_syn_tft<T: OclPrm>(buf: &Buffer<T>, area_id: usize, layer_id: usize, tuft_id: usize)
            -> CorticalBuffer
    {
        CorticalBuffer::DataCellSynapseTuft {
            buffer_id: util::buffer_uid(buf),
            area_id: area_id,
            layer_id: layer_id,
            tuft_id: tuft_id,
        }
    }
}



/// A block of memory outside of the Cortex.
#[derive(Debug)]
pub enum SubcorticalBuffer {
    SourceLayer { area_id: usize, layer_id: usize },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}


/// A block of local or device memory.
#[derive(Debug)]
pub enum MemoryBlock {
    CorticalBuffer(CorticalBuffer),
    SubcorticalBuffer(SubcorticalBuffer),
}


/// An execution command kind.
#[derive(Debug)]
pub enum ExecutionCommandDetails {
    CorticalKernel { sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer> },
    CorticalRead { source: CorticalBuffer, target: SubcorticalBuffer },
    CorticalWrite { source: SubcorticalBuffer, target: CorticalBuffer },
    SubcorticalCopy { source: MemoryBlock, target: MemoryBlock },
    SubGraph,
}


/// A memory accessing command.
///
#[derive(Debug)]
pub struct ExecutionCommand {
    kind: ExecutionCommandDetails,
    event: Option<Event>,
    // sources: Vec<MemoryBlock>,
    // targets: Vec<MemoryBlock>,
}

impl ExecutionCommand {
    pub fn new(kind: ExecutionCommandDetails) -> ExecutionCommand {
        ExecutionCommand { kind: kind, event: None }
    }

    pub fn cortical_kernel(sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer>)
            -> ExecutionCommand
    {
        ExecutionCommand::new(
            ExecutionCommandDetails::CorticalKernel {
                sources: sources,
                targets: targets,
            }
        )
    }

    // pub fn cortical_read() -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::CorticalRead)
    // }

    // pub fn cortical_write() -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::CorticalWrite)
    // }

    // pub fn local_copy() -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::ThalamicCopy)
    // }

    #[inline] pub fn event(&self) -> Option<&Event> { self.event.as_ref() }
}


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    commands: Vec<ExecutionCommand>,
    requisites: Vec<Vec<usize>>,
    locked: bool,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            commands: Vec::with_capacity(256),
            requisites: Vec::with_capacity(256),
            locked: false,
        }
    }

    /// Adds a new command.
    pub fn add_command(&mut self, command: ExecutionCommand) -> usize {
        self.commands.push(command);
        self.requisites.push(Vec::with_capacity(16));
        self.commands.len()
    }

    // fn req_cmds_mut(&mut self, cmd_idx: usize) -> CmnResult<&mut Vec<usize>> {
    //     self.requisites.get_mut(cmd_idx)
    //         .ok_or(CmnError::new(format!("ExecutionGraph::register_requisite: Invalid command index \
    //             (cmd_idx: {}).", cmd_idx)))
    // }

    /// Registers a command as requisite to another.
    pub fn register_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) -> CmnResult<()> {
        let req_idxs = self.requisites.get_mut(cmd_idx)
            // .ok_or(CmnError::new(format!("ExecutionGraph::register_requisite: Invalid command index \
            //     (cmd_idx: {}).", cmd_idx)))?;
            .ok_or(CmnError::from(ExecutionGraphError::InvalidCommandIndex(cmd_idx)))?;

        // Ensure the requisite command index is within bounds and isn't the
        // same as the command index:
        if req_cmd_idx >= req_idxs.len() || cmd_idx == req_cmd_idx {
            // return CmnError::err(format!("ExecutionGraph::register_requisite: Invalid requisite command index \
            //     (req_cmd_idx: {}).", req_cmd_idx));
            return Err(CmnError::from(ExecutionGraphError::InvalidRequisiteCommandIndex(
                req_cmd_idx, cmd_idx)));
        }

        Ok(req_idxs.push(req_cmd_idx))
    }

    /// Returns the list of requisite events for a command.
    pub fn get_req_events(&self, cmd_idx: usize) -> CmnResult<Vec<Event>> {
        let req_idxs = self.requisites.get(cmd_idx)
            .ok_or(CmnError::from(ExecutionGraphError::InvalidCommandIndex(cmd_idx)))?;

        let mut events = Vec::with_capacity(req_idxs.len());

        for &req_idx in req_idxs.iter() {
            let cmd = unsafe { self.commands.get_unchecked(req_idx) };

            if let Some(event) = cmd.event() {
                events.push(event.clone());
            }
        }

        Ok(events)
   }
}