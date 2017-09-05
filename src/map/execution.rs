#![allow(dead_code, unused_variables)]

// use std::num::Wrapping;
use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use ocl::{Event, Buffer, OclPrm, Error as OclError};
// use ocl::core::Event as EventCore;
use ocl::ffi::cl_event;
use map::LayerAddress;
use cmn::{util};

const PRINT_DEBUG: bool = false;
const PRINT_DEBUG_ALL: bool = false;


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


#[derive(Debug, Clone)]
enum RequisitePrecedence {
    Preceding,
    Following,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandUid(usize);

impl fmt::Display for CommandUid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


pub enum ExecutionGraphError {
    InvalidCommandIndex(usize),
    OrderInvalidCommandUid(CommandUid),
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
            ExecutionGraphError::OrderInvalidCommandUid(_) => "Invalid command UID.",
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
            ExecutionGraphError::OrderInvalidCommandUid(cmd_uid) => {
                f.write_fmt(format_args!("Invalid command uid while setting order \
                    ({}).", cmd_uid))
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


type ExeGrResult<T> = Result<T, ExecutionGraphError>;


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
pub enum CommandRelationsKind {
    CorticalKernel { name: String, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer> },
    CorticothalamicRead { sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract> },
    ThalamocorticalWrite { sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer> },
    // InputFilterWrite { sources: Vec<ThalamicTract>, target: CorticalBuffer },
    SubcorticalCopy { source: MemoryBlock, target: MemoryBlock },
    Subgraph { sources: Vec<MemoryBlock>, target: Vec<MemoryBlock> },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CommandRelations {
    kind: CommandRelationsKind,
    cmd_idx: Option<usize>,
}

impl CommandRelations {
    pub fn cortical_kernel<S>(name: S, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer>)
            -> CommandRelations
            where S: Into<String>
    {
        CommandRelations {
            kind: CommandRelationsKind::CorticalKernel {
                name: name.into(),
                sources: sources,
                targets: targets,
            },
            cmd_idx: None,
        }
    }

    pub fn corticothalamic_read(sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract>)
            -> CommandRelations
    {
        CommandRelations {
            kind: CommandRelationsKind::CorticothalamicRead {
                sources: sources,
                targets: targets
            },
            cmd_idx: None,
        }
    }

    pub fn thalamocortical_write(sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer>)
            -> CommandRelations
    {
        CommandRelations {
            kind: CommandRelationsKind::ThalamocorticalWrite {
                sources: sources,
                targets: targets
            },
            cmd_idx: None,
        }
    }

    pub fn sources(&self) -> Vec<MemoryBlock> {
        match self.kind {
            CommandRelationsKind::CorticalKernel { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::CorticalBuffer(src.clone())).collect()
            },
            CommandRelationsKind::CorticothalamicRead { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::CorticalBuffer(src.clone())).collect()
                // vec![MemoryBlock::CorticalBuffer(source.clone())]
            },
            CommandRelationsKind::ThalamocorticalWrite { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::ThalamicTract(src.clone())).collect()
                // vec![MemoryBlock::ThalamicTract(source.clone())]
            },
            // CommandRelationsKind::InputFilterWrite { ref sources, .. } => {
            //     // vec![MemoryBlock::CorticalBuffer(source.clone())]
            //     sources.iter().map(|src| MemoryBlock::ThalamicTract(src.clone())).collect()
            // },
            CommandRelationsKind::SubcorticalCopy { ref source, .. } => vec![source.clone()],
            CommandRelationsKind::Subgraph { .. } => unimplemented!(),
        }
    }

    pub fn targets(&self) -> Vec<MemoryBlock> {
        match self.kind {
            CommandRelationsKind::CorticalKernel { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            },
            CommandRelationsKind::CorticothalamicRead { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::ThalamicTract(tar.clone())).collect()
                // vec![MemoryBlock::ThalamicTract(target.clone())]
            },
            CommandRelationsKind::ThalamocorticalWrite { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
                // vec![MemoryBlock::CorticalBuffer(target.clone())]
            },
            // CommandRelationsKind::InputFilterWrite { ref target, ..  } => {
            //     vec![MemoryBlock::CorticalBuffer(targets.clone())]
            //     // targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            // },
            CommandRelationsKind::SubcorticalCopy { ref target, .. } => vec![target.clone()],
            CommandRelationsKind::Subgraph { .. } => unimplemented!(),
        }
    }

    fn variant_string(&self) -> &'static str {
        match self.kind {
            CommandRelationsKind::CorticalKernel { .. } => "CorticalKernel",
            CommandRelationsKind::CorticothalamicRead { .. } => "CorticothalamicRead",
            CommandRelationsKind::ThalamocorticalWrite { .. } => "ThalamocorticalWrite",
            CommandRelationsKind::SubcorticalCopy { .. } => "SubcorticalCopy",
            CommandRelationsKind::Subgraph { .. } => "Subgraph",
        }
    }

    fn kernel_name<'a>(&'a self) -> &'a str {
        match self.kind {
            CommandRelationsKind::CorticalKernel { ref name, .. } => name,
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
    // details: CommandRelations,
    uid: CommandUid,
    event: Option<Event>,
    // event_cycle_id: usize,
    order_idx: Option<usize>,
    requisite_cmd_idxs: Vec<usize>,
    requisite_cmd_precedence: Vec<RequisitePrecedence>,
    // requisite_cmd_events: Vec<cl_event>,
}

impl ExecutionCommand {
    pub fn new(uid: CommandUid) -> ExecutionCommand {
        ExecutionCommand {
            // details: details,
            uid,
            event: None,
            // event_cycle_id: 0,
            order_idx: None,
            // [NOTE]: Sizing these vectors here could be delayed until
            // `::populate_requisites` to avoid creating canned sizes.
            requisite_cmd_idxs: Vec::with_capacity(8),
            requisite_cmd_precedence: Vec::with_capacity(8),
            // requisite_cmd_events: Vec::with_capacity(16),
        }
    }

    // pub fn cortical_kernel<S>(name: S, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer>)
    //         -> ExecutionCommand
    //         where S: Into<String>
    // {
    //     ExecutionCommand::new(
    //         CommandRelations::CorticalKernel {
    //             name: name.into(),
    //             sources: sources,
    //             targets: targets,
    //         }
    //     )
    // }

    // pub fn corticothalamic_read(sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract>)
    //         -> ExecutionCommand
    // {
    //     ExecutionCommand::new(CommandRelations::CorticothalamicRead {
    //         sources: sources, targets: targets
    //     })
    // }

    // pub fn thalamocortical_write(sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer>)
    //         -> ExecutionCommand
    // {
    //     ExecutionCommand::new(CommandRelations::ThalamocorticalWrite {
    //         sources: sources, targets: targets
    //     })
    // }

    // pub fn input_filter_write(source: CorticalBuffer, targets: Vec<CorticalBuffer>) -> ExecutionCommand {
    //     ExecutionCommand::new(CommandRelations::InputFilterWrite {
    //         source: source, targets: targets
    //     })
    // }

    // pub fn local_copy() -> ExecutionCommand {
    //     ExecutionCommand::new(CommandRelations::ThalamicCopy)
    // }

    pub fn set_order_idx(&mut self, order_idx: usize) {
        self.order_idx = Some(order_idx);
    }

    pub fn set_event(&mut self, event: Option<Event>) {
    // pub fn set_event(&mut self, event: EventCore) {
        self.event = event;
    }

    // #[inline] pub fn sources(&self) -> Vec<MemoryBlock> { self.details.sources() }
    // #[inline] pub fn targets(&self) -> Vec<MemoryBlock> { self.details.targets() }
    #[inline] pub fn event(&self) -> Option<&Event> { self.event.as_ref() }
    #[inline] pub fn order_idx(&self) -> Option<usize> { self.order_idx.clone() }
}


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    next_cmd_uid: usize,
    cmd_relations: BTreeMap<CommandUid, CommandRelations>,
    cmds: Vec<ExecutionCommand>,
    cmd_requisite_events: Vec<Vec<cl_event>>,
    // requisite_cmd_idxs: Vec<Vec<usize>>,
    // requisite_cmd_precedence: Vec<Vec<RequisitePrecedence>>,
    // cycle_id: Wrapping<usize>,
    // requisite_cmd_event_cycle_ids: Vec<Vec<usize>>,
    order: BTreeMap<usize, usize>,
    next_order_idx: usize,
    locked: bool,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            next_cmd_uid: 0,
            cmd_relations: BTreeMap::new(),
            cmds: Vec::with_capacity(256),
            cmd_requisite_events: Vec::with_capacity(256),
            // requisite_cmd_idxs: Vec::with_capacity(256),
            // requisite_cmd_precedence: Vec::with_capacity(256),
            // requisite_cmd_event_cycle_ids: Vec::with_capacity(256),
            // cycle_id: Wrapping(0),
            order: BTreeMap::new(),
            next_order_idx: 0,
            locked: false,
        }
    }

    fn next_cmd_uid(&mut self) -> CommandUid {
        let uid = self.next_cmd_uid;
        self.next_cmd_uid = self.next_cmd_uid.wrapping_add(1);
        CommandUid(uid)
    }

    /// Adds a new command and returns the command's unique identifier (uid).
    pub fn add_command(&mut self, cmd_relations: CommandRelations) -> ExeGrResult<CommandUid> {
        if self.locked { return Err(ExecutionGraphError::Locked); }

        // let cmd_idx = self.cmds.len();
        let cmd_uid = self.next_cmd_uid();
        self.cmd_relations.insert(cmd_uid, cmd_relations);
        // // [NOTE]: Pushing these vectors here could be delayed until
        // // `::populate_requisites` and avoid creating canned sizes.
        // self.requisite_cmd_idxs.push(Vec::with_capacity(16));
        // self.requisite_cmd_precedence.push(Vec::with_capacity(16));
        Ok(cmd_uid)
    }

    /// Specifies a command (by index) as the next in the loose sequence and
    /// returns the command's ordered index (idx).
    pub fn order_command(&mut self, cmd_uid: CommandUid) -> ExeGrResult<usize> {
        if self.locked { return Err(ExecutionGraphError::Locked); }

        match self.cmd_relations.get_mut(&cmd_uid) {
            Some(cmd_rel) => {
                let cmd_idx = self.cmds.len();
                cmd_rel.cmd_idx = Some(cmd_idx);
                let mut cmd = ExecutionCommand::new(cmd_uid);

                let order_idx = self.next_order_idx;
                cmd.set_order_idx(order_idx);
                self.order.insert(order_idx, cmd_idx);
                self.next_order_idx += 1;

                self.cmds.push(cmd);
                self.cmd_requisite_events.push(Vec::with_capacity(8));
                Ok(cmd_idx)
            },
            None => Err(ExecutionGraphError::OrderInvalidCommandUid(cmd_uid)),
        }
    }

    /// Returns a memory block map which contains every command that reads
    /// from and every command that writes to each memory block.
    ///
    /// { MemBlockRwsMap = HashMap<MemoryBlock, (Vec<usize>, Vec<usize>)> }
    ///
    fn readers_and_writers_by_mem_block(&self) -> MemBlockRwsMap {
        let mut mem_block_rws = HashMap::with_capacity(self.cmd_relations.len() * 16);
        if PRINT_DEBUG { println!("\n##### Readers and Writers by Memory Block:"); }
        if PRINT_DEBUG { println!("#####"); }

        for (cmd_idx, cmd) in self.cmds.iter().enumerate() {
            let cmd_relations = self.cmd_relations.get(&cmd.uid).unwrap();

            if PRINT_DEBUG { println!("##### Command [{}][idx:{}] ({}: '{}'):",
                cmd.uid, cmd_idx, cmd_relations.variant_string(), cmd_relations.kernel_name()); }
            if PRINT_DEBUG { println!("#####     [Sources:]"); }

            for cmd_src_block in cmd_relations.sources().into_iter() {
                let block_rw_cmd_idxs = mem_block_rws.entry(cmd_src_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                block_rw_cmd_idxs.readers.push(cmd_idx);
                if PRINT_DEBUG { println!("#####     [{}]: {:?}", block_rw_cmd_idxs.readers.len() - 1, cmd_src_block); }
            }

            if PRINT_DEBUG { println!("#####     [Targets:]"); }

            for cmd_tar_block in cmd_relations.targets().into_iter() {
                let block_rw_cmd_idxs = mem_block_rws.entry(cmd_tar_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());
                block_rw_cmd_idxs.writers.push(cmd_idx);

                if PRINT_DEBUG { println!("#####     [{}]: {:?}", block_rw_cmd_idxs.writers.len() - 1, cmd_tar_block); }
            }

            if PRINT_DEBUG { println!("#####"); }
        }

        // mem_block_rws.shrink_to_fit(); // Don't bother, it'll be dropped later.
        mem_block_rws
    }

    /// Returns a list of commands which both precede a command and which
    /// write to a block of memory which is read from by that command.
    ///
    /// * TODO: Remove redundant, 'superseded', entries.
    ///
    fn preceding_writers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let cmd_relations = &self.cmd_relations[&self.cmds[cmd_idx].uid];
        let pre_writers = cmd_relations.sources().iter().enumerate()
            .flat_map(|(cmd_src_block_idx, cmd_src_block)| {
                mem_block_rws.get(cmd_src_block).unwrap().writers.iter().map(|&writer_cmd_idx| {
                    let cmd_order_idx = self.cmds[writer_cmd_idx].order_idx().expect(
                        "ExecutionGraph::preceeding_writers: Command order index not set.");
                    (cmd_order_idx, writer_cmd_idx)
                })
            })
            .collect();

        if PRINT_DEBUG { println!("##### <{}>:[{}]: Preceding Writers: {:?}",
            self.cmds[cmd_idx].order_idx().unwrap(), cmd_idx, pre_writers); }

        pre_writers
    }

    /// Returns a list of commands which both follow a command and which read
    /// from a block of memory which is written to by that command.
    ///
    /// * TODO: Remove redundant, superfluous, entries
    ///
    fn following_readers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> BTreeMap<usize, usize> {
        let cmd_relations = &self.cmd_relations[&self.cmds[cmd_idx].uid];
        let mut fol_readers = BTreeMap::new();

        for (cmd_tar_block_idx, cmd_tar_block) in cmd_relations.targets().iter().enumerate() {
            // // TEMP:
            //     let ref block_writers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().writers;
            // //

            let ref block_readers: Vec<usize> = mem_block_rws.get(cmd_tar_block).unwrap().readers;

                // println!("##### Command [{}]: Target Block [{}]: Writers: {:?}, Readers: {:?},", cmd_idx,
                //     cmd_tar_block_idx, block_writers, block_readers);

            for &reader_cmd_idx in block_readers.iter() {
                let cmd_order_idx = self.cmds[reader_cmd_idx].order_idx().expect(
                    "ExecutionGraph::preceeding_writers: Command order index not set.");

                fol_readers.insert(cmd_order_idx, reader_cmd_idx);
            }
        }

        if PRINT_DEBUG { println!("##### <{}>:[{}]: Following Readers: {:?}",
            self.cmds[cmd_idx].order_idx().unwrap(), cmd_idx, fol_readers); }
        fol_readers
    }

    /// Adds a requisite command index and precedence to a command's list of
    /// requisites.
    fn add_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) {
        let cmd_order = self.cmds[cmd_idx].order_idx().expect(
                "ExecutionGraph::add_requisite: Command order index not set.");
        let req_cmd_order = self.cmds[req_cmd_idx].order_idx().expect(
                "ExecutionGraph::add_requisite: Requisite command order index not set.");

        // If a requisite command is the command itself, it must contain a
        // memory block which is both read from and written to:
        assert!(req_cmd_order != cmd_order, "Execution commands which both read from and write \
            to a memory block should omit that block from the list of sources and specify only \
            within the list of targets.");

        self.cmds[cmd_idx].requisite_cmd_idxs.push(req_cmd_idx);

        let req_cmd_precedence = if req_cmd_order < cmd_order {
            RequisitePrecedence::Preceding
        } else {
            RequisitePrecedence::Following
        };

        self.cmds[cmd_idx].requisite_cmd_precedence.push(req_cmd_precedence);
    }

    /// Populates the list of requisite commands for each command and locks
    /// the graph, disallowing addition or removal of commands until unlocked
    /// with `::unlock`.
    //
    // TODO: Consider renaming to `lock`.
    pub fn lock(&mut self) {
        assert!(self.cmd_relations.len() == self.cmds.len(), "ExecutionGraph::populate_requisites \
            Not all commands have had their order properly set ({}/{}). Call '::order_next' to \
            include commands in the execution order.", self.order.len(), self.cmds.len());
        assert!(!self.locked, "Cannot populate this graph while locked. Use '::unlock_clear' first.");

        let mem_block_rws = self.readers_and_writers_by_mem_block();

        // println!("\n########## Memory Block Reader/Writers: {:#?}\n", mem_block_rws);

        if PRINT_DEBUG { println!("\n##### Preceding Writers and Following Readers <order>:[cmd_idx]:"); }
        if PRINT_DEBUG { println!("#####"); }

        // [NOTE]: Only using `self.order` instead of `self.commands` for
        // debug printing purposes. * TODO: Switch back at some point.
        for (&cmd_order, &cmd_idx) in self.order.clone().iter() {
            if PRINT_DEBUG { println!("##### Command <{}>:[{}] ({}):", cmd_order, cmd_idx,
                self.cmd_relations[&self.cmds[cmd_idx].uid].variant_string()); }

            assert!(self.cmds[cmd_idx].requisite_cmd_idxs.is_empty() &&
                self.cmds[cmd_idx].requisite_cmd_precedence.is_empty());

            for (_, pre_writer_cmd_idx) in self.preceding_writers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, pre_writer_cmd_idx);
            }

            for (_, fol_reader_cmd_idx) in self.following_readers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, fol_reader_cmd_idx);
            }

            self.cmds[cmd_idx].requisite_cmd_idxs.shrink_to_fit();
            self.cmds[cmd_idx].requisite_cmd_precedence.shrink_to_fit();

            debug_assert!(self.cmds[cmd_idx].requisite_cmd_idxs.len() ==
                self.cmds[cmd_idx].requisite_cmd_precedence.len());

            // println!("##### <{}>:[{}]: Requisites: {:?}:{:?}",
            //     cmd_order, cmd_idx, self.requisite_cmd_idxs[cmd_idx],
            //     self.requisite_cmd_precedence[cmd_idx]);
            if PRINT_DEBUG { println!("#####"); }
        }

        // self.cmds[cmd_idx].requisite_cmd_idxs.shrink_to_fit();
        // self.cmds[cmd_idx].requisite_cmd_precedence.shrink_to_fit();

        // debug_assert!(self.cmds.len() == self.cmds[cmd_idx].requisite_cmd_idxs.len() &&
        //     self.cmds.len() == self.cmds[cmd_idx].requisite_cmd_precedence.len());

        self.locked = true;
        self.next_order_idx = 0;
    }

    /// Returns the list of requisite events for a command.
    pub fn get_req_events(&mut self, cmd_idx: usize) -> ExeGrResult<&[cl_event]> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

        if self.next_order_idx != unsafe { self.cmds.get_unchecked(cmd_idx).order_idx().unwrap() } {
            panic!("{}", ExecutionGraphError::EventsRequestOutOfOrder(self.next_order_idx,
                 self.cmds[cmd_idx].order_idx().unwrap()));
        }

        debug_assert!(self.cmd_requisite_events.len() > cmd_idx);
        unsafe { self.cmd_requisite_events.get_unchecked_mut(cmd_idx).clear(); }

        let req_idxs = &self.cmds.get(cmd_idx)
            .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?
            .requisite_cmd_idxs;

        for &req_idx in req_idxs.iter() {
            let cmd = unsafe { self.cmds.get_unchecked(req_idx) };

            if let Some(event) = cmd.event() {
                unsafe {
                    self.cmd_requisite_events.get_unchecked_mut(cmd_idx).push(*event.as_ptr_ref());
                }
            }
        }

        if PRINT_DEBUG && PRINT_DEBUG_ALL { println!("##### ExecutionGraph::get_req_events: Event \
            list for [cmd_idx: {}]: {:?}", cmd_idx, self.cmd_requisite_events.get(cmd_idx).unwrap()); }

        Ok(unsafe { self.cmd_requisite_events.get_unchecked_mut(cmd_idx).as_slice() })
    }

    /// Sets the event associated with the completion of a command.
    pub fn set_cmd_event(&mut self, cmd_idx: usize, event: Option<Event>) -> ExeGrResult<()> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

        let cmd = self.cmds.get_mut(cmd_idx)
            .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;

        if self.next_order_idx != cmd.order_idx().unwrap() {
            return Err(ExecutionGraphError::EventsRequestOutOfOrder(self.next_order_idx, cmd_idx));
        }

        cmd.set_event(event.map(|e| e.into()));

        if (self.next_order_idx + 1) == self.order.len() {
            self.next_order_idx = 0;
        } else {
            self.next_order_idx += 1;
            if PRINT_DEBUG && PRINT_DEBUG_ALL { println!("##### ExecutionGraph::set_cmd_event: \
                Setting event for [cmd_idx: {}].", cmd_idx,) }
        }

        Ok(())
    }

    /// Blocks until all events from all commands have completed.
    pub fn finish(&self) -> ExeGrResult<()> {
        // for cmd in self.commands.iter() {
        //     if let Some(ref ev) = cmd.event {
        //         ev.wait_for()?;
        //     }
        // }
        for cmd in self.order.iter().map(|(&cmd_idx, _)|
                unsafe { self.cmds.get_unchecked(cmd_idx) } )
        {
            if let Some(ref ev) = cmd.event {
                ev.wait_for()?;
            }
        }
        Ok(())
    }

    /// Unlocks this graph and allows commands to be added or removed.
    ///
    /// Graph must be locked with `::populate` to use.
    pub fn unlock(&mut self) {
        self.locked = false;
        self.cmds.clear();
        for (_, cmd_rel) in self.cmd_relations.iter_mut() {
            cmd_rel.cmd_idx = None;
        }
    }

    // #[inline] pub fn command_count(&self) -> usize { self.order.len() }
}

unsafe impl Send for ExecutionGraph {}




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
