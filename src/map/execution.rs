#![allow(dead_code, unused_variables)]

use std::ops::Range;
use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use ocl::{Event, Buffer, OclPrm};
use map::LayerAddress;
use cmn::{util, /*CmnError,*/ /*CmnResult*/};

type ExeGrResult<T> = Result<T, ExecutionGraphError>;

pub enum ExecutionGraphError {
    InvalidCommandIndex(usize),
    OrderInvalidCommandIndex(usize),
    InvalidRequisiteCommandIndex(usize, usize),
}

impl error::Error for ExecutionGraphError {
    fn description(&self) -> &str {
        match *self {
            ExecutionGraphError::InvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::OrderInvalidCommandIndex(_) => "Invalid command index.",
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
            ExecutionGraphError::OrderInvalidCommandIndex(cmd_idx) => {
                f.write_fmt(format_args!("Invalid command index while setting order \
                    (cmd_idx: {}).", cmd_idx))
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CorticalBuffer {
    // AxonLayer { buffer_id: u64, layer_addr: LayerAddress },
    // AxonLayerSubSlice { buffer_id: u64, layer_addr: LayerAddress, sub_slc_range: Range<u8> },
    AxonSlice { buffer_id: u64, layer_addr: LayerAddress, slc_id: u8 },
    DataCellSynapseTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize, },
    DataCellDendriteTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize },
    DataCellSomaTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize },
    DataCellSomaLayer { buffer_id: u64, layer_addr: LayerAddress },
    ControlCellSomaLayer { buffer_id: u64, area_id: usize, layer_id: usize },
}

impl CorticalBuffer {
    // pub fn axon_layer<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress) -> CorticalBuffer {
    //     CorticalBuffer::AxonLayer {
    //         buffer_id: util::buffer_uid(buf),
    //         layer_addr: layer_addr,
    //     }
    // }

    // pub fn axon_layer_sub_slice<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress,
    //         sub_slc_range: Range<u8>) -> CorticalBuffer
    // {
    //     CorticalBuffer::AxonLayerSubSlice {
    //         buffer_id: util::buffer_uid(buf),
    //         layer_addr: layer_addr,
    //         sub_slc_range: sub_slc_range,
    //     }
    // }

    pub fn axon_slice<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, slc_id: u8)
            -> CorticalBuffer
    {
        CorticalBuffer::AxonSlice {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            slc_id: slc_id,
        }
    }

    pub fn data_syn_tft<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, tuft_id: usize)
            -> CorticalBuffer
    {
        CorticalBuffer::DataCellSynapseTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            tuft_id: tuft_id,
        }
    }

    pub fn data_den_tft<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, tuft_id: usize)
            -> CorticalBuffer
    {
        CorticalBuffer::DataCellDendriteTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            tuft_id: tuft_id,
        }
    }

    pub fn data_soma_tft<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, tuft_id: usize)
            -> CorticalBuffer
    {
        CorticalBuffer::DataCellSomaTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            tuft_id: tuft_id,
        }
    }

    pub fn data_soma_lyr<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress)
            -> CorticalBuffer
    {
        CorticalBuffer::DataCellSomaLayer {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
        }
    }
}


/// A block of memory outside of the Cortex.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SubcorticalBuffer {
    AxonSlice { layer_addr: LayerAddress, sub_slc_range: Option<Range<u8>> },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl SubcorticalBuffer {
    pub fn axon_slice(layer_addr: LayerAddress, sub_slc_range: Option<Range<u8>>) -> SubcorticalBuffer {
        SubcorticalBuffer::AxonSlice {
            layer_addr: layer_addr,
            sub_slc_range: sub_slc_range,
        }
    }
}


/// A block of the thalamic tract.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ThalamicTract {
    Slice { src_layer_addr: LayerAddress, slc_id: u8 },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl ThalamicTract {
    pub fn axon_slice(src_layer_addr: LayerAddress, slc_id: u8) -> ThalamicTract {
        ThalamicTract::Slice {
            src_layer_addr: src_layer_addr,
            slc_id: slc_id,
        }
    }
}


/// A block of local or device memory.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MemoryBlock {
    CorticalBuffer(CorticalBuffer),
    SubcorticalBuffer(SubcorticalBuffer),
    ThalamicTract(ThalamicTract),
}


/// An execution command kind.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ExecutionCommandDetails {
    CorticalKernel { sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer> },
    CorticothalamicRead { sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract> },
    ThalamocorticalWrite { sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer> },
    SubcorticalCopy { source: MemoryBlock, target: MemoryBlock },
    SubGraph { sources: Vec<MemoryBlock>, target: Vec<MemoryBlock> },
}

impl ExecutionCommandDetails {
    fn sources(&self) -> Vec<MemoryBlock> {
        match *self {
            ExecutionCommandDetails::CorticalKernel { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::CorticalBuffer(src.clone())).collect()
            },
            ExecutionCommandDetails::CorticothalamicRead { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::CorticalBuffer(src.clone())).collect()
                // vec![MemoryBlock::CorticalBuffer(source.clone())]
            },
            ExecutionCommandDetails::ThalamocorticalWrite { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::ThalamicTract(src.clone())).collect()
                // vec![MemoryBlock::ThalamicTract(source.clone())]
            },
            ExecutionCommandDetails::SubcorticalCopy { ref source, .. } => vec![source.clone()],
            ExecutionCommandDetails::SubGraph { .. } => unimplemented!(),
        }
    }

    fn targets(&self) -> Vec<MemoryBlock> {
        match *self {
            ExecutionCommandDetails::CorticalKernel { ref targets, ..  } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            },
            ExecutionCommandDetails::CorticothalamicRead { ref targets, ..  } => {
                targets.iter().map(|tar| MemoryBlock::ThalamicTract(tar.clone())).collect()
                // vec![MemoryBlock::ThalamicTract(target.clone())]
            },
            ExecutionCommandDetails::ThalamocorticalWrite { ref targets, ..  } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
                // vec![MemoryBlock::CorticalBuffer(target.clone())]
            },
            ExecutionCommandDetails::SubcorticalCopy { ref target, ..  } => vec![target.clone()],
            ExecutionCommandDetails::SubGraph { .. } => unimplemented!(),
        }
    }
}


/// A memory accessing command.
///
#[derive(Debug, Clone)]
pub struct ExecutionCommand {
    details: ExecutionCommandDetails,
    event: Option<Event>,
    order_idx: Option<usize>,
}

impl ExecutionCommand {
    pub fn new(details: ExecutionCommandDetails) -> ExecutionCommand {
        ExecutionCommand {
            details: details,
            event: None,
            order_idx: None,
        }
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

    pub fn corticothalamic_read(sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract>) -> ExecutionCommand {
        ExecutionCommand::new(ExecutionCommandDetails::CorticothalamicRead {
            sources: sources, targets: targets })
    }

    pub fn thalamocortical_write(sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer>) -> ExecutionCommand {
        ExecutionCommand::new(ExecutionCommandDetails::ThalamocorticalWrite {
            sources: sources, targets: targets })
    }

    // pub fn local_copy() -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::ThalamicCopy)
    // }

    pub fn set_order_idx(&mut self, order_idx: usize) {
        self.order_idx = Some(order_idx);
    }

    #[inline] pub fn sources(&self) -> Vec<MemoryBlock> { self.details.sources() }
    #[inline] pub fn targets(&self) -> Vec<MemoryBlock> { self.details.targets() }
    #[inline] pub fn event(&self) -> Option<&Event> { self.event.as_ref() }
    #[inline] pub fn order_idx(&self) -> Option<usize> { self.order_idx.clone() }
}


type MemBlockRws = HashMap<MemoryBlock, (Vec<usize>, Vec<usize>)>;


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    commands: Vec<ExecutionCommand>,
    requisites: Vec<Vec<usize>>,
    locked: bool,
    next_order_idx: usize,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            commands: Vec::with_capacity(256),
            requisites: Vec::with_capacity(256),
            next_order_idx: 0,
            locked: false,
        }
    }

    /// Adds a new command.
    pub fn add_command(&mut self, command: ExecutionCommand) -> usize {
        let cmd_idx = self.commands.len();
        self.commands.push(command);
        self.requisites.push(Vec::with_capacity(16));
        cmd_idx
    }

    pub fn order_next(&mut self, cmd_idx: usize) -> ExeGrResult<usize> {
        let cmd = self.commands.get_mut(cmd_idx)
            .ok_or(ExecutionGraphError::OrderInvalidCommandIndex(cmd_idx))?;

        let order_idx = self.next_order_idx;
        cmd.set_order_idx(order_idx);
        self.next_order_idx += 1;
        Ok(order_idx)
    }


    // fn req_cmds_mut(&mut self, cmd_idx: usize) -> Result<&mut Vec<usize>, ExecutionGraphError>{
    //     self.requisites.get_mut(cmd_idx)
    //         .ok_or(CmnError::new(format!("ExecutionGraph::register_requisite: Invalid command index \
    //             (cmd_idx: {}).", cmd_idx)))
    // }

    // /// Registers a command as requisite to another.
    // pub fn register_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) -> Result<(), ExecutionGraphError> {
    //     let req_idxs = self.requisites.get_mut(cmd_idx)
    //         // .ok_or(CmnError::new(format!("ExecutionGraph::register_requisite: Invalid command index \
    //         //     (cmd_idx: {}).", cmd_idx)))?;
    //         .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

    //     // Ensure the requisite command index is within bounds and isn't the
    //     // same as the command index:
    //     if req_cmd_idx >= req_idxs.len() || cmd_idx == req_cmd_idx {
    //         // return CmnError::err(format!("ExecutionGraph::register_requisite: Invalid requisite command index \
    //         //     (req_cmd_idx: {}).", req_cmd_idx));
    //         return Err(CmnError::from(ExecutionGraphError::InvalidRequisiteCommandIndex(
    //             req_cmd_idx, cmd_idx)));
    //     }

    //     Ok(req_idxs.push(req_cmd_idx))
    // }


    /// Returns a memory block map which contains every command that reads
    /// from and every command that writes to each memory block.
    ///
    /// { MemBlockRws = HashMap<MemoryBlock, (Vec<usize>, Vec<usize>)> }
    ///
    fn readers_and_writers_by_mem_block(&self) -> MemBlockRws {
        let mut mem_block_rws = HashMap::with_capacity(self.commands.len() * 16);
        println!("\n##### Readers and Writers by Memory Block:");
        println!("#####");

        for (cmd_idx, cmd) in self.commands.iter().enumerate() {
            println!("##### Command [{}]:", cmd_idx);

            for cmd_src_block in cmd.sources().into_iter() {
                let & mut(_, ref mut readers) = mem_block_rws.entry(cmd_src_block.clone())
                    .or_insert((Vec::with_capacity(16), Vec::with_capacity(16)));

                readers.push(cmd_idx);

                println!("#####     Source Block [{}]: {:?}", readers.len() - 1, cmd_src_block);
            }

            for cmd_tar_block in cmd.targets().into_iter() {
                let & mut(ref mut writers, _) = mem_block_rws.entry(cmd_tar_block.clone())
                    .or_insert((Vec::with_capacity(16), Vec::with_capacity(16)));

                writers.push(cmd_idx);

                println!("#####     Target Block [{}]: {:?}", writers.len() - 1, cmd_tar_block);
            }

            // println!("##### Command [{}]: Sources: {:?}, Targets: {:?}", cmd_idx, cmd.sources(), cmd.targets());
        }

        mem_block_rws.shrink_to_fit();
        mem_block_rws
    }

    /// Returns a list of commands which both precede a command and which
    /// write to a block of memory which is read from by that command.
    fn preceding_writers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRws) -> BTreeMap<usize, usize> {
        let mut pre_writers = BTreeMap::new();

        for cmd_src_block in self.commands[cmd_idx].sources().iter() {
            let ref block_writers: Vec<usize> = mem_block_rws.get(cmd_src_block).unwrap().1;

            for &writer_cmd_idx in block_writers.iter()/*.filter(|&&wci| wci != cmd_idx)*/ {
                let cmd_order_idx = self.commands[writer_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                pre_writers.insert(cmd_order_idx, writer_cmd_idx);
            }
        }

        // println!("##### Command [{}]: Preceeding Writers: {:?}", cmd_idx, pre_writers);
        pre_writers
    }

    /// Returns a list of commands which both follow a command and which read
    /// from a block of memory which is written to by that command.
    fn following_readers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRws) -> BTreeMap<usize, usize> {
        let mut fol_readers = BTreeMap::new();

        for cmd_src_block in self.commands[cmd_idx].targets().iter() {
            let ref block_readers: Vec<usize> = mem_block_rws.get(cmd_src_block).unwrap().0;

            for &reader_cmd_idx in block_readers.iter()/*.filter(|&&rci| rci != cmd_idx)*/ {
                let cmd_order_idx = self.commands[reader_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                fol_readers.insert(cmd_order_idx, reader_cmd_idx);
            }
        }

        // println!("##### Command [{}]: Following Readers: {:?}", cmd_idx, fol_readers);
        fol_readers
    }

    /// Populates the list of requisite commands for each command.
    pub fn populate_requisites(&mut self) {
        let mem_block_rws = self.readers_and_writers_by_mem_block();

        // println!("\n########## Memory Block Reader/Writers: {:#?}\n", mem_block_rws);

        println!("\n##### Preceeding Writers and Following Readers:");
        println!("#####");

        for (cmd_idx, cmd) in self.commands.iter().enumerate() {
            let pre_writers = self.preceding_writers(cmd_idx, &mem_block_rws);
            println!("##### Command [{}]: Preceeding Writers: {:?}", cmd_idx, pre_writers);

            // for cmd_src_block in cmd.sources().into_iter() {
            //     let (ref src_block_writers, _) = mem_block_rws[&cmd_src_block];
            // }

            let fol_readers = self.following_readers(cmd_idx, &mem_block_rws);
            println!("##### Command [{}]: Following Readers: {:?}", cmd_idx, fol_readers);

            // for cmd_tar_block in cmd.targets().into_iter() {

            // }
        }
    }

    /// Returns the list of requisite events for a command.
    pub fn get_req_events(&self, cmd_idx: usize) -> ExeGrResult<Vec<Event>> {
        let req_idxs = self.requisites.get(cmd_idx)
            .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

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