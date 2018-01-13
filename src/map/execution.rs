// #![allow(dead_code, unused_variables)]

use std::collections::{HashMap, BTreeMap};
use std::error;
use std::fmt;
use ocl::{self, Event, Buffer, OclPrm, Error as OclError};
use ocl::core::CommandExecutionStatus;
use ocl::ffi::{cl_event, c_void};
use map::LayerAddress;
use cmn::{util};

const PRNT: bool = true;
const PRINT_EVENT_DEBUG: bool = false;
const PRNT_ALL: bool = false;

extern "C" fn __event_running(event: cl_event, _status: i32, cmd_idx: *mut c_void) {
    println!("##### [{}]   >>> Event running:    \t(event: {:?})",
        cmd_idx as usize, event);
}

extern "C" fn __event_complete(event: cl_event, _status: i32, cmd_idx: *mut c_void) {
    println!("##### [{}]   <<<   Event complete: \t(event: {:?})",
        cmd_idx as usize, event);
}

fn set_debug_callback_running(event: &Event, cmd_idx: usize) {
    if PRNT && PRINT_EVENT_DEBUG {
        unsafe { ocl::core::set_event_callback(event, CommandExecutionStatus::Running,
            Some(__event_running), cmd_idx as *mut c_void).unwrap(); }
    }
}

fn set_debug_callback_complete(event: &Event, cmd_idx: usize) {
    if PRNT && PRINT_EVENT_DEBUG {
        unsafe { ocl::core::set_event_callback(event, CommandExecutionStatus::Complete,
            Some(__event_complete), cmd_idx as *mut c_void).unwrap(); }
    }
}


struct MemBlockRwCmdIdxs {
    writers: Vec<usize>,
    readers: Vec<usize>,
}

impl MemBlockRwCmdIdxs {
    fn new() -> MemBlockRwCmdIdxs {
        MemBlockRwCmdIdxs { writers: Vec::with_capacity(16), readers: Vec::with_capacity(16) }
    }

    #[allow(dead_code)]
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

impl From<ocl::core::Error> for ExecutionGraphError {
    fn from(err: ocl::core::Error) -> ExecutionGraphError {
        ExecutionGraphError::OclError(err.into())
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
                    the commands were configured with `::order_command`. Ensure that each command is \
                    calling `::set_cmd_event` when enqueued.", expected_order, found_order))
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

    pub fn axon_slice(buf: &Buffer<u8>, area_id: usize, slc_id: u8)
            -> CorticalBuffer {
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
            -> CorticalBuffer {
        CorticalBuffer::DataCellSynapseTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            tuft_id: tuft_id,
        }
    }

    pub fn data_den_tft<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, tuft_id: usize)
            -> CorticalBuffer {
        CorticalBuffer::DataCellDendriteTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr: layer_addr,
            tuft_id: tuft_id,
        }
    }

    pub fn data_soma_tft<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress, tuft_id: usize)
            -> CorticalBuffer {
        CorticalBuffer::DataCellSomaTuft {
            buffer_id: util::buffer_uid(buf),
            layer_addr,
            tuft_id,
        }
    }

    pub fn data_soma_lyr<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress)
            -> CorticalBuffer {
        CorticalBuffer::DataCellSomaLayer {
            buffer_id: util::buffer_uid(buf),
            layer_addr,
        }
    }

    pub fn control_soma_lyr<T: OclPrm>(buf: &Buffer<T>, layer_addr: LayerAddress)
            -> CorticalBuffer {
        CorticalBuffer::ControlCellSomaLayer {
            buffer_id: util::buffer_uid(buf),
            layer_addr,
        }
    }
}


/// A block of memory outside of the Cortex.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SubcorticalBuffer {
    AxonSlice { mem_id: usize, area_id: usize, slc_id: u8 },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl SubcorticalBuffer {
    pub fn axon_slice(mem_id: usize, area_id: usize, slc_id: u8) -> SubcorticalBuffer {
        SubcorticalBuffer::AxonSlice { mem_id, area_id, slc_id, }
    }
}


/// A block of the thalamic tract.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ThalamicTract {
    AxonSlice { mem_id: usize, area_id: usize, slc_id: u8 },
    // SubCorticalLayerSource { area_id: usize, layer_id: usize },
}

impl ThalamicTract {
    pub fn axon_slice(mem_id: usize, area_id: usize, slc_id: u8) -> ThalamicTract {
        ThalamicTract::AxonSlice { mem_id, area_id, slc_id, }
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
    CorticalSample { sources: Vec<CorticalBuffer> },
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
    /// Returns a new command relation with `CommandRelationsKind::CorticalKernel`.
    pub fn cortical_kernel<S>(name: S, sources: Vec<CorticalBuffer>, targets: Vec<CorticalBuffer>)
            -> CommandRelations
            where S: Into<String> {
        CommandRelations {
            kind: CommandRelationsKind::CorticalKernel {
                name: name.into(),
                sources: sources,
                targets: targets,
            },
            cmd_idx: None,
        }
    }

    /// Returns a new command relation with `CommandRelationsKind::CorticothalamicRead`.
    pub fn corticothalamic_read(sources: Vec<CorticalBuffer>, targets: Vec<ThalamicTract>)
            -> CommandRelations {
        CommandRelations {
            kind: CommandRelationsKind::CorticothalamicRead {
                sources: sources,
                targets: targets
            },
            cmd_idx: None,
        }
    }

    /// Returns a new command relation with `CommandRelationsKind::ThalamocorticalWrite`.
    pub fn thalamocortical_write(sources: Vec<ThalamicTract>, targets: Vec<CorticalBuffer>)
            -> CommandRelations {
        CommandRelations {
            kind: CommandRelationsKind::ThalamocorticalWrite {
                sources: sources,
                targets: targets
            },
            cmd_idx: None,
        }
    }

    /// Returns a new command relation with `CommandRelationsKind::CorticalSample`.
    pub fn cortical_sample(sources: Vec<CorticalBuffer>) -> CommandRelations {
        CommandRelations {
            kind: CommandRelationsKind::CorticalSample {
                sources: sources,
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
            },
            CommandRelationsKind::ThalamocorticalWrite { ref sources, .. } => {
                sources.iter().map(|src| MemoryBlock::ThalamicTract(src.clone())).collect()
            },
            CommandRelationsKind::CorticalSample { ref sources } => {
                sources.iter().map(|src| MemoryBlock::CorticalBuffer(src.clone())).collect()
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
            },
            CommandRelationsKind::ThalamocorticalWrite { ref targets, .. } => {
                targets.iter().map(|tar| MemoryBlock::CorticalBuffer(tar.clone())).collect()
            },
            CommandRelationsKind::CorticalSample { .. } => {
                Vec::new()
            }
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
            CommandRelationsKind::CorticalSample { .. } => "CorticalSample",
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
    uid: CommandUid,
    event: Option<Event>,
    requisite_cmd_idxs: Vec<usize>,
    requisite_cmd_precedence: Vec<RequisitePrecedence>,
    stale_events: Vec<Event>,
}

impl ExecutionCommand {
    fn new(uid: CommandUid) -> ExecutionCommand {
        ExecutionCommand {
            uid,
            event: None,
            // [NOTE]: Sizing these vectors here could be delayed until
            // `::populate_requisites` to avoid creating canned sizes.
            requisite_cmd_idxs: Vec::with_capacity(8),
            requisite_cmd_precedence: Vec::with_capacity(8),
            stale_events: Vec::with_capacity(16),
        }
    }

    fn set_event(&mut self, event: Option<Event>) {
        self.event = event;
    }

    #[inline] pub fn event(&self) -> Option<&Event> { self.event.as_ref() }
}


/// A graph of memory accessing commands.
///
#[derive(Debug)]
pub struct ExecutionGraph {
    next_cmd_uid: usize,
    cmd_relations: BTreeMap<CommandUid, CommandRelations>,
    cmds: Vec<ExecutionCommand>,
    cmd_requisite_events: Vec<Vec<cl_event>>,
    next_cmd_idx: usize,
    locked: bool,
    // stale_events: HashMap<(Event, usize), usize>,
}

impl ExecutionGraph {
    /// Returns a new, empty, execution graph.
    pub fn new() -> ExecutionGraph {
        ExecutionGraph {
            next_cmd_uid: 0,
            cmd_relations: BTreeMap::new(),
            cmds: Vec::with_capacity(256),
            cmd_requisite_events: Vec::with_capacity(256),
            next_cmd_idx: 0,
            locked: false,
            // stale_events: HashMap::with_capacity(64),
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
                self.cmds.push(ExecutionCommand::new(cmd_uid));
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
        if PRNT { println!("\n##### Readers and Writers by Memory Block:"); }
        if PRNT { println!("#####"); }

        for (cmd_idx, cmd) in self.cmds.iter().enumerate() {
            let cmd_relations = self.cmd_relations.get(&cmd.uid).unwrap();

            if PRNT { println!("##### Command {}[idx:{}] ({}: '{}'):",
                cmd.uid, cmd_idx, cmd_relations.variant_string(), cmd_relations.kernel_name()); }
            if PRNT { println!("#####     [Sources:]"); }

            for cmd_src_block in cmd_relations.sources().into_iter() {
                let block_rw_cmd_idxs = mem_block_rws.entry(cmd_src_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());

                block_rw_cmd_idxs.readers.push(cmd_idx);
                if PRNT { println!("#####     [{}]: {:?}", block_rw_cmd_idxs.readers.len() - 1, cmd_src_block); }
            }

            if PRNT { println!("#####     [Targets:]"); }

            for cmd_tar_block in cmd_relations.targets().into_iter() {
                let block_rw_cmd_idxs = mem_block_rws.entry(cmd_tar_block.clone())
                    .or_insert(MemBlockRwCmdIdxs::new());
                block_rw_cmd_idxs.writers.push(cmd_idx);

                if PRNT { println!("#####     [{}]: {:?}", block_rw_cmd_idxs.writers.len() - 1, cmd_tar_block); }
            }

            if PRNT { println!("#####"); }
        }

        mem_block_rws
    }

    /// Returns a list of commands which both precede a command and which
    /// write to a block of memory which is read from by that command.
    ///
    /// * TODO: Remove redundant, 'superseded', entries (to save time).
    ///
    fn preceding_writers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> Vec<usize> {
        let cmd_relations = &self.cmd_relations[&self.cmds[cmd_idx].uid];
        let pre_writers = cmd_relations.sources().iter().flat_map(|cmd_src_block| {
                mem_block_rws.get(cmd_src_block).unwrap().writers.iter().cloned()
            }).collect();
        if PRNT { println!("##### {}:[{}]: Preceding Writers: {:?}",
            self.cmds[cmd_idx].uid, cmd_idx, pre_writers); }
        pre_writers
    }

    /// Returns a list of commands which both follow a command and which read
    /// from a block of memory which is written to by that command.
    ///
    /// * TODO: Remove redundant, 'superseded', entries (to save time).
    ///
    fn following_readers(&self, cmd_idx: usize, mem_block_rws: &MemBlockRwsMap) -> Vec<usize> {
        let cmd_relations = &self.cmd_relations[&self.cmds[cmd_idx].uid];
        let fol_readers = cmd_relations.targets().iter().flat_map(|cmd_tar_block| {
                mem_block_rws.get(cmd_tar_block).unwrap().readers.iter().cloned()
            }).collect();
        if PRNT { println!("##### {}:[{}]: Following Readers: {:?}",
            self.cmds[cmd_idx].uid, cmd_idx, fol_readers); }
        fol_readers
    }

    /// Adds a requisite command index and precedence to a command's list of
    /// requisites.
    fn add_requisite(&mut self, cmd_idx: usize, req_cmd_idx: usize) {
        // If a requisite command is the command itself, it contains a memory
        // block which is both read from and written to within the same
        // command and need not be specified:
        assert!(cmd_idx != req_cmd_idx, "Execution commands which both read from and write to the \
            same memory block should omit that block from the list of sources and specify only \
            within the list of targets. (cmd_idx: {}, req_cmd_idx: {})", cmd_idx, req_cmd_idx);

        // Add requisite command index if not a duplicate:
        if !self.cmds[cmd_idx].requisite_cmd_idxs.contains(&req_cmd_idx) {
            self.cmds[cmd_idx].requisite_cmd_idxs.push(req_cmd_idx);

            let req_cmd_precedence = if req_cmd_idx < cmd_idx {
                RequisitePrecedence::Preceding
            } else {
                RequisitePrecedence::Following
            };

            self.cmds[cmd_idx].requisite_cmd_precedence.push(req_cmd_precedence);
        }
    }

    /// Populates the list of requisite commands for each command and locks
    /// the graph, disallowing addition or removal of commands until unlocked
    /// with `::unlock`.
    pub fn lock(&mut self) {
        assert!(self.cmd_relations.len() == self.cmds.len(), "ExecutionGraph::lock \
            Not all commands have had their order properly set ({}/{}). Call '::order_command' to \
            include commands in the execution order.", self.cmds.len(), self.cmd_relations.len());
        assert!(!self.locked, "Cannot populate this graph while locked. Use '::unlock_clear' first.");

        let mem_block_rws = self.readers_and_writers_by_mem_block();

        if PRNT { println!("\n##### Preceding Writers and Following Readers (CommandUid:[idx]:)"); }
        if PRNT { println!("#####"); }

        // [NOTE]: Only using `self.order` instead of `self.commands` for
        // debug printing purposes. * TODO: Switch back at some point.
        for cmd_idx in 0..self.cmds.len() {
            if PRNT { println!("##### Command {}:[{}] ({}):", self.cmds[cmd_idx].uid,
                cmd_idx, self.cmd_relations[&self.cmds[cmd_idx].uid].variant_string()); }

            assert!(self.cmds[cmd_idx].requisite_cmd_idxs.is_empty() &&
                self.cmds[cmd_idx].requisite_cmd_precedence.is_empty());

            for pre_writer_cmd_idx in self.preceding_writers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, pre_writer_cmd_idx);
            }

            for fol_reader_cmd_idx in self.following_readers(cmd_idx, &mem_block_rws) {
                self.add_requisite(cmd_idx, fol_reader_cmd_idx);
            }

            self.cmds[cmd_idx].requisite_cmd_idxs.shrink_to_fit();
            self.cmds[cmd_idx].requisite_cmd_precedence.shrink_to_fit();

            debug_assert!(self.cmds[cmd_idx].requisite_cmd_idxs.len() ==
                self.cmds[cmd_idx].requisite_cmd_precedence.len());

            if PRNT { println!("#####"); }
        }

        self.locked = true;
        self.next_cmd_idx = 0;
    }

    /// Returns the list of requisite events for a command.
    pub fn get_req_events(&mut self, cmd_idx: usize) -> ExeGrResult<&[cl_event]> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }

        assert!(self.next_cmd_idx == cmd_idx,"{}", ExecutionGraphError::EventsRequestOutOfOrder(
            self.next_cmd_idx, cmd_idx));

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

        if PRNT && PRNT_ALL { println!("##### [{}] Getting event list: {:?}", cmd_idx,
            self.cmd_requisite_events.get(cmd_idx).unwrap()); }

        Ok(unsafe { self.cmd_requisite_events.get_unchecked_mut(cmd_idx).as_slice() })
    }

    /// Sets the event associated with the completion of a command.
    pub fn set_cmd_event(&mut self, cmd_idx: usize, event: Option<Event>) -> ExeGrResult<()> {
        if !self.locked { return Err(ExecutionGraphError::Unlocked); }
        let cmds_len = self.cmds.len();

        {
            let cmd = self.cmds.get_mut(cmd_idx)
                .ok_or(ExecutionGraphError::InvalidCommandIndex(cmd_idx))?;
            if self.next_cmd_idx != cmd_idx {
                return Err(ExecutionGraphError::EventsRequestOutOfOrder(self.next_cmd_idx, cmd_idx));
            }
            if PRNT && PRINT_EVENT_DEBUG {
                if PRNT_ALL {println!("##### [{}] Setting command event \t (event: {:?}", cmd_idx, event); }
                if let Some(ref ev) = event {
                    set_debug_callback_running(ev, cmd_idx);
                    set_debug_callback_complete(ev, cmd_idx);
                }
            }
            cmd.set_event(event);
        }

        if (self.next_cmd_idx + 1) == cmds_len {
            // self.eval_events();
            self.next_cmd_idx = 0;
        } else {
            self.next_cmd_idx += 1;
            // if PRNT && PRNT_ALL { println!("##### ExecutionGraph::set_cmd_event: \
            //     Setting event for [cmd_idx: {}].", cmd_idx,) }
        }
        Ok(())
    }

    /// Blocks until all events from all commands have completed.
    pub fn finish(&self) -> ExeGrResult<()> {
        for cmd in self.cmds.iter() {
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

    /// Determine which commands have events which are incomplete.
    fn _eval_events(&mut self) {
        println!("########################## Stale Events: ##########################");

        // Iterate over all events, adding incomplete events to the stale list:
        for (cmd_idx, cmd) in self.cmds.iter_mut().enumerate() {
            // Prune old:
            let mut pruned_stale = Vec::with_capacity(cmd.stale_events.capacity());
            for ev in cmd.stale_events.iter() {
                if !ev.is_complete().unwrap() {
                    pruned_stale.push(ev.clone());
                }
            }
            cmd.stale_events = pruned_stale;

            // Add new:
            if let Some(ev) = cmd.event.clone() {
                if !ev.is_complete().unwrap() {
                    cmd.stale_events.push(ev);
                }
            }
            println!("########################## [{}]:{}", cmd_idx, cmd.stale_events.len());
        }
        println!("###################################################################");
    }

    #[inline] pub fn is_locked(&self) -> bool { self.locked }
    #[inline] pub fn cmd(&self, cmd_idx: usize) -> Option<&ExecutionCommand> { self.cmds.get(cmd_idx) }
}

unsafe impl Send for ExecutionGraph {}

