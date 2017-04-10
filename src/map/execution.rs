#![allow(dead_code, unused_variables)]

use std::num::Wrapping;
use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use ocl::{Event, Buffer, OclPrm, Error as OclError};
use ocl::core::Event as EventCore;
use ocl::ffi::cl_event;
use map::LayerAddress;
use cmn::{util};

const PRINT_DEBUG: bool = false;

type ExeGrResult<T> = Result<T, ExecutionGraphError>;

pub enum ExecutionGraphError {
    InvalidCommandIndex(usize),
    OrderInvalidCommandIndex(usize),
    InvalidRequisiteCommandIndex(usize, usize),
    Locked,
    Unlocked,
    OclError(OclError),
    EventsRequestOutOfOrder(usize, usize),
}

impl error::Error for ExecutionGraphError {
    fn description(&self) -> &str {
        match *self {
            ExecutionGraphError::InvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::OrderInvalidCommandIndex(_) => "Invalid command index.",
            ExecutionGraphError::InvalidRequisiteCommandIndex(..) => "Invalid command index.",
            ExecutionGraphError::Locked => "Graph locked.",
            ExecutionGraphError::Unlocked => "Graph unlocked.",
            ExecutionGraphError::OclError(_) => "OpenCL Error.",
            ExecutionGraphError::EventsRequestOutOfOrder(..) => "Events requested out of order.",
        }
    }
}

impl From<OclError> for ExecutionGraphError {
    fn from(err: OclError) -> ExecutionGraphError {
        ExecutionGraphError::OclError(err)
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
            ExecutionGraphError::OclError(ref ocl_error) => {
                f.write_fmt(format_args!("OpenCL Error: '{}'", ocl_error))
            }
            ExecutionGraphError::EventsRequestOutOfOrder(expected_order, found_order) => {
                f.write_fmt(format_args!("ExecutionGraph::get_req_events: Events requested out \
                    of order. Expected: <{}>, found: <{}>. Events must be requested in the order \
                    the commands were configured with `::order_next`", expected_order, found_order))
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
    AxonInputFilter { buffer_id: u64 },
    DataCellSynapseTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize, },
    DataCellDendriteTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize },
    DataCellSomaTuft { buffer_id: u64, layer_addr: LayerAddress, tuft_id: usize },
    DataCellSomaLayer { buffer_id: u64, layer_addr: LayerAddress },
    ControlCellSomaLayer { buffer_id: u64, layer_addr: LayerAddress },
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

    pub fn axon_input_filter<T: OclPrm>(buf: &Buffer<T>) -> CorticalBuffer {
        CorticalBuffer::AxonInputFilter {
            buffer_id: util::buffer_uid(buf),
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

    pub fn control_soma_lyr<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress)
            -> CorticalBuffer
    {
        CorticalBuffer::ControlCellSomaLayer {
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
    CorticalKernel { name: String, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer> },
    CorticothalamicRead { sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract> },
    ThalamocorticalWrite { sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer> },
    // InputFilterWrite { sources: Vec<ThalamicTract>, target: CorticalBuffer },
    SubcorticalCopy { source: MemoryBlock, target: MemoryBlock },
    Subgraph { sources: Vec<MemoryBlock>, target: Vec<MemoryBlock> },
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
            // ExecutionCommandDetails::InputFilterWrite { ref sources, .. } => {
            //     // vec![MemoryBlock::CorticalBuffer(source.clone())]
            //     sources.iter().map(|src| MemoryBlock::ThalamicTract(src.clone())).collect()
            // },
            ExecutionCommandDetails::SubcorticalCopy { ref source, .. } => vec![source.clone()],
            ExecutionCommandDetails::Subgraph { .. } => unimplemented!(),
        }
    }

    fn targets(&self) -> Vec<MemoryBlock> {
        match *self {
            ExecutionCommandDetails::CorticalKernel { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            },
            ExecutionCommandDetails::CorticothalamicRead { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::ThalamicTract(tar.clone())).collect()
                // vec![MemoryBlock::ThalamicTract(target.clone())]
            },
            ExecutionCommandDetails::ThalamocorticalWrite { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
                // vec![MemoryBlock::CorticalBuffer(target.clone())]
            },
            // ExecutionCommandDetails::InputFilterWrite { ref target, ..  } => {
            //     vec![MemoryBlock::CorticalBuffer(targets.clone())]
            //     // targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            // },
            ExecutionCommandDetails::SubcorticalCopy { ref target, .. } => vec![target.clone()],
            ExecutionCommandDetails::Subgraph { .. } => unimplemented!(),
        }
    }

    fn variant_string(&self) -> &'static str {
        match *self {
            ExecutionCommandDetails::CorticalKernel { .. } => "CorticalKernel",
            ExecutionCommandDetails::CorticothalamicRead { .. } => "CorticothalamicRead",
            ExecutionCommandDetails::ThalamocorticalWrite { .. } => "ThalamocorticalWrite",
            ExecutionCommandDetails::SubcorticalCopy { .. } => "SubcorticalCopy",
            ExecutionCommandDetails::Subgraph { .. } => "Subgraph",
        }
    }

    fn kernel_name<'a>(&'a self) -> &'a str {
        match *self {
            ExecutionCommandDetails::CorticalKernel { ref name, .. } => name,
            _ => "",
        }
    }
}


/// A memory accessing command.
///
//
//
#[derive(Debug, Clone)]
pub struct ExecutionCommand {
    details: ExecutionCommandDetails,
    event: Option<EventCore>,
    event_cycle_id: usize,
    order_idx: Option<usize>,
}

impl ExecutionCommand {
    pub fn new(details: ExecutionCommandDetails) -> ExecutionCommand {
        ExecutionCommand {
            details: details,
            event: None,
            event_cycle_id: 0,
            order_idx: None,
        }
    }

    pub fn cortical_kernel<S>(name: S, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer>)
            -> ExecutionCommand
            where S: Into<String>
    {
        ExecutionCommand::new(
            ExecutionCommandDetails::CorticalKernel {
                name: name.into(),
                sources: sources,
                targets: targets,
            }
        )
    }

    pub fn corticothalamic_read(sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract>)
            -> ExecutionCommand
    {
        ExecutionCommand::new(ExecutionCommandDetails::CorticothalamicRead {
            sources: sources, targets: targets
        })
    }

    pub fn thalamocortical_write(sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer>)
            -> ExecutionCommand
    {
        ExecutionCommand::new(ExecutionCommandDetails::ThalamocorticalWrite {
            sources: sources, targets: targets
        })
    }

    // pub fn input_filter_write(source: CorticalBuffer, targets: Vec<CorticalBuffer>) -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::InputFilterWrite {
    //         source: source, targets: targets
    //     })
    // }

    // pub fn local_copy() -> ExecutionCommand {
    //     ExecutionCommand::new(ExecutionCommandDetails::ThalamicCopy)
    // }

    pub fn set_order_idx(&mut self, order_idx: usize) {
        self.order_idx = Some(order_idx);
    }

    pub fn set_event(&mut self, event: Option<EventCore>) {
    // pub fn set_event(&mut self, event: EventCore) {
        self.event = event;
    }

    #[inline] pub fn sources(&self) -> Vec<MemoryBlock> { self.details.sources() }
    #[inline] pub fn targets(&self) -> Vec<MemoryBlock> { self.details.targets() }
    #[inline] pub fn event(&self) -> Option<&EventCore> { self.event.as_ref() }
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


#[derive(Debug)]
enum RequisitePrecedence {
    Preceding,
    Following,
}


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    commands: Vec<ExecutionCommand>,
    requisite_cmd_idxs: Vec<Vec<usize>>,
    requisite_cmd_precedence: Vec<Vec<RequisitePrecedence>>,
    requisite_cmd_events: Vec<Vec<cl_event>>,
    cycle_id: Wrapping<usize>,
    // requisite_cmd_event_cycle_ids: Vec<Vec<usize>>,
    order: BTreeMap<usize, usize>,
    locked: bool,
    next_order_idx: usize,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            commands: Vec::with_capacity(256),
            requisite_cmd_idxs: Vec::with_capacity(256),
            requisite_cmd_precedence: Vec::with_capacity(256),
            requisite_cmd_events: Vec::with_capacity(256),
            // requisite_cmd_event_cycle_ids: Vec::with_capacity(256),
            cycle_id: Wrapping(0),
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
        // [NOTE]: Pushing these vectors here could be delayed until
        // `::populate_requisites` and avoid creating canned sizes.
        self.requisite_cmd_idxs.push(Vec::with_capacity(16));
        self.requisite_cmd_precedence.push(Vec::with_capacity(16));
        self.requisite_cmd_events.push(Vec::with_capacity(16));
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

    /// Returns a memory block map which contains every command that reads
    /// from and every command that writes to each memory block.
    ///
    /// { MemBlockRwsMap = HashMap<MemoryBlock, (Vec<usize>, Vec<usize>)> }
    ///
    fn readers_and_writers_by_mem_block(&self) -> MemBlockRwsMap {
        let mut mem_block_rws = HashMap::with_capacity(self.commands.len() * 16);
        if PRINT_DEBUG { println!("\n##### Readers and Writers by Memory Block:"); }
        if PRINT_DEBUG { println!("#####"); }

        for (cmd_idx, cmd) in self.commands.iter().enumerate() {
            if PRINT_DEBUG { println!("##### Command [{}] ({}: '{}'):", cmd_idx,
                cmd.details.variant_string(), cmd.details.kernel_name()); }

            if PRINT_DEBUG { println!("#####     [Sources:]"); }

            for cmd_src_block in cmd.sources().into_iter() {
                let rw_cmd_idxs = mem_block_rws.entry(cmd_src_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                rw_cmd_idxs.readers.push(cmd_idx);
                // println!("#####     Source Block [{}]: {:?}", rw_cmd_idxs.readers.len() - 1, cmd_src_block);
                if PRINT_DEBUG { println!("#####     [{}]: {:?}", rw_cmd_idxs.readers.len() - 1, cmd_src_block); }
            }

            // println!("#####");
            if PRINT_DEBUG { println!("#####     [Targets:]"); }

            for cmd_tar_block in cmd.targets().into_iter() {
                let rw_cmd_idxs = mem_block_rws.entry(cmd_tar_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                rw_cmd_idxs.writers.push(cmd_idx);
                // println!("#####     Target Block [{}]: {:?}", rw_cmd_idxs.writers.len() - 1, cmd_tar_block);
                if PRINT_DEBUG { println!("#####     [{}]: {:?}", rw_cmd_idxs.writers.len() - 1, cmd_tar_block); }
            }

            // println!("#####");
            // println!("#####         Totals: Sources: {}, Targets: {}", cmd.sources().len(), cmd.targets().len());
            if PRINT_DEBUG { println!("#####"); }
        }

        mem_block_rws.shrink_to_fit();
        mem_block_rws
    }

    /// Returns a list of commands which both precede a command and which
    /// write to a block of memory which is read from by that command.
    ///
    /// * TODO: Remove redundant, 'superseded', entries.
    ///
    fn preceding_writers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let pre_writers = self.commands[cmd_idx].details.sources().iter().enumerate()
            .flat_map(|(cmd_src_block_idx, cmd_src_block)| {
                mem_block_rws.get(cmd_src_block).unwrap().writers.iter().map(|&writer_cmd_idx| {
                    let cmd_order_idx = self.commands[writer_cmd_idx].order_idx().expect(
                        "ExecutionGraph::preceeding_writers: Command order index not set.");
                    (cmd_order_idx, writer_cmd_idx)
                })
            })
            .collect();

        if PRINT_DEBUG { println!("##### <{}>:[{}]: Preceding Writers: {:?}",
            self.commands[cmd_idx].order_idx().unwrap(), cmd_idx, pre_writers); }

        pre_writers
    }

    /// Returns a list of commands which both follow a command and which read
    /// from a block of memory which is written to by that command.
    ///
    /// * TODO: Remove redundant, superfluous, entries
    ///
    fn following_readers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let mut fol_readers = BTreeMap::new();

        for (cmd_tar_block_idx, cmd_tar_block) in self.commands[cmd_idx].targets().iter().enumerate() {
            // // TEMP:
            //     let ref block_writers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().writers;
            // //

            let ref block_readers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().readers;

                // println!("##### Command [{}]: Target Block [{}]: Writers: {:?}, Readers: {:?},", cmd_idx,
                //     cmd_tar_block_idx, block_writers, block_readers);

            for &reader_cmd_idx in block_readers.iter() {
                let cmd_order_idx = self.commands[reader_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                fol_readers.insert(cmd_order_idx, reader_cmd_idx);
            }
        }

        if PRINT_DEBUG { println!("##### <{}>:[{}]: Following Readers: {:?}",
            self.commands[cmd_idx].order_idx().unwrap(), cmd_idx, fol_readers); }
        fol_readers
    }


    fn add_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) {
        let cmd_order = self.commands[cmd_idx].order_idx().expect(
                "ExecutionGraph::add_requisite: Command order index not set.");
        let req_cmd_order = self.commands[req_cmd_idx].order_idx().expect(
                "ExecutionGraph::add_requisite: Requisite command order index not set.");

        assert!(req_cmd_order != cmd_order);
        self.requisite_cmd_idxs[cmd_idx].push(req_cmd_idx);

        let req_cmd_precedence = if req_cmd_order < cmd_order {
            RequisitePrecedence::Preceding
        } else {
            RequisitePrecedence::Following
        };

        self.requisite_cmd_precedence[cmd_idx].push(req_cmd_precedence);
    }

    /// Populates the list of requisite commands for each command.
    ///
    pub fn populate_requisites(&mut self) {
        assert!(self.commands.len() == self.order.len(), "ExecutionGraph::populate_requisites \
            Not all commands have had their order properly set ({}/{}). Call '::order_next' to \
            include commands in the execution order.", self.order.len(), self.commands.len());

        let mem_block_rws = self.readers_and_writers_by_mem_block();

        // println!("\n########## Memory Block Reader/Writers: {:#?}\n", mem_block_rws);

        if PRINT_DEBUG { println!("\n##### Preceding Writers and Following Readers <order>:[cmd_idx]:"); }
        if PRINT_DEBUG { println!("#####"); }

        // [NOTE]: Only using `self.order` instead of `self.commands` for
        // debug printing purposes. * TODO: Switch back at some point.
        for (&cmd_order, &cmd_idx) in self.order.clone().iter() {
            if PRINT_DEBUG { println!("##### Command <{}>:[{}] ({}):", cmd_order, cmd_idx,
                self.commands[cmd_idx].details.variant_string()); }


            assert!(self.requisite_cmd_idxs[cmd_idx].is_empty() &&
                self.requisite_cmd_precedence[cmd_idx].is_empty());

            for (_, pre_writer_cmd_idx) in self.preceding_writers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, pre_writer_cmd_idx);
            }

            for (_, fol_reader_cmd_idx) in self.following_readers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, fol_reader_cmd_idx);
            }

            self.requisite_cmd_idxs[cmd_idx].shrink_to_fit();
            self.requisite_cmd_precedence[cmd_idx].shrink_to_fit();

            debug_assert!(self.requisite_cmd_idxs[cmd_idx].len() ==
                self.requisite_cmd_precedence[cmd_idx].len());

            // println!("##### <{}>:[{}]: Requisites: {:?}:{:?}",
            //     cmd_order, cmd_idx, self.requisite_cmd_idxs[cmd_idx],
            //     self.requisite_cmd_precedence[cmd_idx]);
            if PRINT_DEBUG { println!("#####"); }
        }

        self.requisite_cmd_idxs.shrink_to_fit();
        self.requisite_cmd_precedence.shrink_to_fit();

        debug_assert!(self.commands.len() == self.requisite_cmd_idxs.len() &&
            self.commands.len() == self.requisite_cmd_precedence.len());

        self.locked = true;
        self.next_order_idx = 0;
    }

    /// Returns the list of requisite events for a command.
    ///
    pub fn get_req_events(&mut self, cmd_idx: usize) -> ExeGrResult<&[cl_event]> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

        let req_idxs = self.requisite_cmd_idxs.get(cmd_idx)
            .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

        if self.next_order_idx != unsafe { self.commands.get_unchecked(cmd_idx).order_idx().unwrap() } {
            panic!("{}", ExecutionGraphError::EventsRequestOutOfOrder(self.next_order_idx,
                 self.commands[cmd_idx].order_idx().unwrap()));
            // return Err(ExecutionGraphError::EventsRequestOutOfOrder(self.next_order_idx,
            //     self.commands[cmd_idx].order_idx().unwrap()));
        }

        unsafe { self.requisite_cmd_events.get_unchecked_mut(cmd_idx).clear(); }

        for &req_idx in req_idxs.iter() {
            let cmd = unsafe { self.commands.get_unchecked(req_idx) };

            if let Some(event) = cmd.event() {
                unsafe {
                    self.requisite_cmd_events.get_unchecked_mut(cmd_idx).push(*event.as_ptr_ref());
                }
            }
        }

        Ok(unsafe { self.requisite_cmd_events.get_unchecked(cmd_idx).as_slice() })
    }

    /// Sets the event associated with the completion of a command.
    // pub fn set_cmd_event(&mut self, cmd_idx: usize, event: Event) -> ExeGrResult<()> {
    pub fn set_cmd_event(&mut self, cmd_idx: usize, event: Option<Event>) -> ExeGrResult<()> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

        let cmd = self.commands.get_mut(cmd_idx)
            .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

        if self.next_order_idx != cmd.order_idx().unwrap() {
            return Err(ExecutionGraphError::EventsRequestOutOfOrder(self.next_order_idx, cmd_idx));
        }

        cmd.set_event(event.map(|e| e.into())); // <--- Correct Version
        // cmd.set_event(event.map(|ev| ev.core().clone()));
        // cmd.set_event(event.core().clone());

        if (self.next_order_idx + 1) == self.order.len() {
            self.next_order_idx = 0;
        } else {
            self.next_order_idx += 1;
            if PRINT_DEBUG { println!("##### ExecutionGraph::set_cmd_event: (cmd_idx: {}): next_order_idx: {}",
                cmd_idx, self.next_order_idx) }
        }

        Ok(())
    }

    // pub fn _RESET(&mut self) {
    //     self.next_order_idx = 0;
    // }

    // #[inline] pub fn command_count(&self) -> usize { self.order.len() }
}





    // /// Returns the list of requisite events for a command.
    // ///
    // // pub fn get_req_events(&self, cmd_idx: usize, wait_list: &mut Vec<cl_event>) -> ExeGrResult<()> {
    // pub fn get_req_events(&self, cmd_idx: usize, wait_list: &mut EventList) -> ExeGrResult<()> {
    //     if !self.locked { return Err(ExecutionGraphError::Unlocked); }

    //     let req_idxs = self.requisite_cmd_idxs.get(cmd_idx)
    //         .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

    //     if self.next_order_idx != unsafe { self.commands.get_unchecked(cmd_idx).order_idx().unwrap() } {
    //         return Err(ExecutionGraphError::EventsRequestOutOfOrder(cmd_idx));
    //     }

    //     wait_list.clear()?;

    //     for &req_idx in req_idxs.iter() {
    //         let cmd = unsafe { self.commands.get_unchecked(req_idx) };

    //         if let Some(event) = cmd.event() {
    //             // unsafe { wait_list.push(*event.core()
    //             //     /* .expect(&format!("ExecutionGraph::get_req_events: Empty 'ocl::Event' found \
    //             //         for command [{}]", req_idx)) */
    //             //     .as_ptr_ref()); }
    //             wait_list.push(event.clone())
    //         }
    //     }

    //     Ok(())
    // }




    // fn req_cmds_mut(&mut self, cmd_idx: usize) -> Result<&mut Vec<usize>, ExecutionGraphError> {
    //     self.requisite_cmd_idxs.get_mut(cmd_idx)
    //         .ok_or(CmnError::new(format!("ExecutionGraph::register_requisite: Invalid command index \
    //             (cmd_idx: {}).", cmd_idx)))
    // }


    // /// Registers a command as requisite to another.
    // pub fn register_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) -> Result<(), ExecutionGraphError> {
    //     let req_idxs = self.requisite_cmd_idxs.get_mut(cmd_idx)
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
