#![allow(dead_code, unused_variables)]

// use std::ops::Range;
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
    Locked,
    Unlocked,
}

impl error::Error for ExecutionGraphError {
    fn description(&self) -> &str {
        match *self {
            ExecutionGraphError::InvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::OrderInvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::InvalidRequisiteCommandIndex(..) => "Invalid command index.",
            ExecutionGraphError::Locked => "Graph locked.",
            ExecutionGraphError::Unlocked => "Graph unlocked.",
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
            ExecutionGraphError::Locked => {
                f.write_str("Execution graph is locked.")
            }
            ExecutionGraphError::Unlocked => {
                f.write_str("Execution graph is unlocked. Lock using '::populate_requisites'")
            }
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
    AxonSlice { buffer_id: u64, area_id: usize, slc_id: u8 },
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

    pub fn axon_slice<T: OclPrm>(buf: &Buffer<T>, area_id: usize, slc_id: u8)
            -> CorticalBuffer
    {
        CorticalBuffer::AxonSlice {
            buffer_id: util::buffer_uid(buf),
            area_id: area_id,
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
    AxonSlice { area_id: usize, slc_id: u8 },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl SubcorticalBuffer {
    pub fn axon_slice(area_id: usize, slc_id: u8) -> SubcorticalBuffer {
        SubcorticalBuffer::AxonSlice {
            area_id: area_id,
            slc_id: slc_id,
        }
    }
}


/// A block of the thalamic tract.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ThalamicTract {
    Slice { area_id: usize, slc_id: u8 },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl ThalamicTract {
    pub fn axon_slice(area_id: usize, slc_id: u8) -> ThalamicTract {
        ThalamicTract::Slice {
            area_id: area_id,
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

struct MemBlockRwCmdIdxs {
    writers: Vec<usize>,
    readers: Vec<usize>,
}

impl MemBlockRwCmdIdxs {
    fn new() -> MemBlockRwCmdIdxs {
        MemBlockRwCmdIdxs { writers: Vec::with_capacity(16), readers: Vec::with_capacity(16) }
    }

    fn shrink_to_fit(&mut self) {
        self.writers.shrink_to_fit();
        self.readers.shrink_to_fit();
    }
}

type MemBlockRwsMap = HashMap<MemoryBlock, MemBlockRwCmdIdxs>;


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    commands: Vec<ExecutionCommand>,
    requisites: Vec<Vec<usize>>,
    order: BTreeMap<usize, usize>,
    locked: bool,
    next_order_idx: usize,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            commands: Vec::with_capacity(256),
            requisites: Vec::with_capacity(256),
            order: BTreeMap::new(),
            next_order_idx: 0,
            locked: false,
        }
    }

    /// Adds a new command.
    pub fn add_command(&mut self, command: ExecutionCommand) -> ExeGrResult<usize> {
        if self.locked { return Err(ExecutionGraphError::Locked); }

        let cmd_idx = self.commands.len();
        self.commands.push(command);
        self.requisites.push(Vec::with_capacity(16));
        Ok(cmd_idx)
    }

    pub fn order_next(&mut self, cmd_idx: usize) -> ExeGrResult<usize> {
        if self.locked { return Err(ExecutionGraphError::Locked); }

        let cmd = self.commands.get_mut(cmd_idx)
            .ok_or(ExecutionGraphError::OrderInvalidCommandIndex(cmd_idx))?;

        let order_idx = self.next_order_idx;
        cmd.set_order_idx(order_idx);
        self.order.insert(order_idx, cmd_idx);
        self.next_order_idx += 1;
        Ok(order_idx)
    }


    // fn req_cmds_mut(&mut self, cmd_idx: usize) -> Result<&mut Vec<usize>, ExecutionGraphError> {
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
    /// { MemBlockRwsMap = HashMap<MemoryBlock, (Vec<usize>, Vec<usize>)> }
    ///
    fn readers_and_writers_by_mem_block(&self) -> MemBlockRwsMap {
        let mut mem_block_rws = HashMap::with_capacity(self.commands.len() * 16);
        println!("\n##### Readers and Writers by Memory Block:");
        println!("#####");

        for (cmd_idx, cmd) in self.commands.iter().enumerate() {
            println!("##### Command [{}]:", cmd_idx);

            for cmd_src_block in cmd.sources().into_iter() {
                let rw_cmd_idxs = mem_block_rws.entry(cmd_src_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                rw_cmd_idxs.readers.push(cmd_idx);
                println!("#####     Source Block [{}]: {:?}", rw_cmd_idxs.readers.len() - 1, cmd_src_block);
            }

            println!("#####");

            for cmd_tar_block in cmd.targets().into_iter() {
                let rw_cmd_idxs = mem_block_rws.entry(cmd_tar_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                rw_cmd_idxs.writers.push(cmd_idx);
                println!("#####     Target Block [{}]: {:?}", rw_cmd_idxs.writers.len() - 1, cmd_tar_block);
            }

            println!("#####");
            println!("#####     Totals: Sources: {}, Targets: {}", cmd.sources().len(), cmd.targets().len());
            println!("#####");
        }

        mem_block_rws.shrink_to_fit();
        mem_block_rws
    }

    /// Returns a list of commands which both precede a command and which
    /// write to a block of memory which is read from by that command.
    ///
    /// [TODO]: Remove redundant, 'superseded', entries.
    ///
    fn preceding_writers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let mut pre_writers = BTreeMap::new();

        for (cmd_src_block_idx, cmd_src_block) in self.commands[cmd_idx].sources().iter().enumerate() {
            let ref block_writers: Vec<usize> = mem_block_rws.get(cmd_src_block).unwrap().writers;

            // TEMP:
                let ref block_readers: Vec<usize> = mem_block_rws.get(cmd_src_block).unwrap().readers;
            //

            // println!("##### Command [{}]: Source Block [{}]: Writers: {:?}, Readers: {:?}", cmd_idx,
            //     cmd_src_block_idx, block_writers, block_readers);

            for &writer_cmd_idx in block_writers.iter() {
                let cmd_order_idx = self.commands[writer_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                pre_writers.insert(cmd_order_idx, writer_cmd_idx);
            }
        }

        let cmd_order_idx = self.commands[cmd_idx].order_idx().unwrap();
        println!("##### Command [{}: {}]: Preceeding Writers: {:?}", cmd_order_idx, cmd_idx, pre_writers);
        // println!("#####");
        pre_writers
    }

    /// Returns a list of commands which both follow a command and which read
    /// from a block of memory which is written to by that command.
    ///
    /// [TODO]: Remove redundant, 'superseded', entries
    ///
    fn following_readers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let mut fol_readers = BTreeMap::new();

        for (cmd_tar_block_idx, cmd_tar_block) in self.commands[cmd_idx].targets().iter().enumerate() {
            // TEMP:
                let ref block_writers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().writers;
            //

            let ref block_readers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().readers;

                // println!("##### Command [{}]: Target Block [{}]: Writers: {:?}, Readers: {:?},", cmd_idx,
                //     cmd_tar_block_idx, block_writers, block_readers);

            for &reader_cmd_idx in block_readers.iter() {
                let cmd_order_idx = self.commands[reader_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                fol_readers.insert(cmd_order_idx, reader_cmd_idx);
            }
        }

        let cmd_order_idx = self.commands[cmd_idx].order_idx().unwrap();
        println!("##### Command [{}: {}]: Following Readers: {:?}", cmd_order_idx, cmd_idx, fol_readers);
        // println!("#####");
        fol_readers
    }

    /// Populates the list of requisite commands for each command.
    pub fn populate_requisites(&mut self) {
        assert!(self.commands.len() == self.requisites.len() &&
            self.commands.len() == self.order.len());

        let mem_block_rws = self.readers_and_writers_by_mem_block();

        // println!("\n########## Memory Block Reader/Writers: {:#?}\n", mem_block_rws);

        println!("\n##### Preceeding Writers and Following Readers:");
        println!("#####");

        for (_, &cmd_idx) in self.order.iter() {
            // println!("##### Command [{}]: ", cmd_idx);

            let pre_writers = self.preceding_writers(cmd_idx, &mem_block_rws);
            // println!("##### Command [{}]: Preceeding Writers: {:?}", cmd_idx, pre_writers);

            // for cmd_src_block in cmd.sources().into_iter() {
            //     let (ref src_block_writers, _) = mem_block_rws[&cmd_src_block];
            // }

            let fol_readers = self.following_readers(cmd_idx, &mem_block_rws);
            // println!("##### Command [{}]: Following Readers: {:?}", cmd_idx, fol_readers);

            // for cmd_tar_block in cmd.targets().into_iter() {

            // }
            println!("#####");
        }

        self.locked = true;
    }

    /// Returns the list of requisite events for a command.
    pub fn get_req_events(&self, cmd_idx: usize) -> ExeGrResult<Vec<Event>> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

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