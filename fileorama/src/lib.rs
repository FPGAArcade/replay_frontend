use crossbeam_channel::unbounded;
use log::*;
use thiserror::Error;

use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
    sync::{Arc, RwLock},
    thread::{self},
};

mod ftp_fs;
mod local_fs;
mod zip_fs;

#[cfg(test)]
use std::println as trace;

// Used for keeping data a alive for a while. This is useful as some data may have been loaded and then it's
// common that another system wants to read the same data. We keep it alive for this amount of entries at the same time
const MAX_CACHE_COUNT: usize = 5;

#[derive(Default, Debug)]
pub struct FilesDirs {
    pub files: Vec<String>,
    pub dirs: Vec<String>,
}

impl FilesDirs {
    pub(crate) fn new(files: Vec<String>, dirs: Vec<String>) -> FilesDirs {
        FilesDirs { files, dirs }
    }
}

unsafe impl Sync for Data {}
unsafe impl Send for Data {}

/// This is used to pass cached data back to the main thread without having to copy it
pub struct Data {
    pub ptr: *const u8,
    pub size: usize,
}

impl Data {
    #[must_use]
    pub fn new(data: &[u8]) -> Data {
        Data {
            ptr: data.as_ptr(),
            size: data.len(),
        }
    }

    pub fn get(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
}

/// Used for sending messages back to the main thread
pub enum RecvMsg {
    /// Progress of the current read operation. Notice that this may not be fully accurate
    /// depending on the driver that is being used for the read operation and how many layers of
    /// indirection there are.
    ReadProgress(f32),
    /// Data after reading has finished.
    ReadDone(Data),
    /// Data after the reading has been completed, but does also include metadata about the data.
    ReadDoneMetadata(Data, Data),
    /// Error that occured during reading
    Error(SendError),
    /// When reading from a url a directory listing is returned
    Directory(FilesDirs),
    /// Nothing found at the given url
    NotFound,
}

#[derive(Debug)]
pub enum LoadStatus {
    // Data was loaded from the current node
    Data(Box<[u8]>),
    // directory.
    Directory,
    /// Requested node wasn't found
    NotFound,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("File Error")]
    FileDirNotFound,
    #[error("File Error")]
    FileError(#[from] std::io::Error),
    #[error("Parse Error")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Send Error")]
    SendError(#[from] crossbeam_channel::SendError<RecvMsg>),
    #[error("Walkdir Error")]
    WalkdirError(#[from] walkdir::Error),
    #[error("Ftp Error")]
    FtpError(#[from] ftp::FtpError),
    #[error("Generic")]
    Generic(String),
}

#[derive(Error, Debug)]
pub enum SendError {
    /// Errors from std::io::Error
    #[error("File Error)")]
    FileError(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct Progress<'a> {
    range: (f32, f32),
    step: f32,
    current: f32,
    msg: &'a crossbeam_channel::Sender<RecvMsg>,
}

/// File system implementations must implement this trait
pub trait IoDriver: std::fmt::Debug + Send + Sync {
    /// This indicates that the file system is remote (such as ftp, https) and has no local path
    fn is_remote(&self) -> bool;
    /// Name of the driver (only used for debug purposes)
    fn name(&self) -> &'static str;
    /// If the driver supports a certain url
    fn supports_url(&self, url: &str) -> bool;
    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> IoDriverType;
    /// Used when creating an instance of the driver with a path to load from
    fn create_from_url(&self, url: &str) -> Option<IoDriverType>;
    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, path: &str, progress: &mut Progress) -> Result<LoadStatus, Error>;
    // get a file/directory listing for the driver
    fn get_directory_list(
        &mut self,
        path: &str,
        progress: &mut Progress,
    ) -> Result<FilesDirs, Error>;
}

/// Memory Loader implementations must implement this trait
pub trait MemoryDriver: std::fmt::Debug + Send + Sync {
    /// Name of the driver (only used for debug purposes)
    fn name(&self) -> &'static str;
    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> MemoryDriverType;
    // Check if we can create a driver given some memory
    fn can_create_from_data(&self, data: &[u8]) -> bool;
    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(&self, data: Box<[u8]>) -> Option<MemoryDriverType>;
    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, local_path: &str, progress: &mut Progress) -> Result<LoadStatus, Error>;
    // get a file/directory listing for the driver. Default is that the loader doesn't
    // support directory listings
    fn get_directory_list(
        &mut self,
        _local_path: &str,
        _progress: &mut Progress,
    ) -> Result<FilesDirs, Error> {
        Ok(FilesDirs::default())
    }
}

#[derive(Clone)]
pub struct Handle {
    pub recv: crossbeam_channel::Receiver<RecvMsg>,
}

impl<'a> Progress<'a> {
    pub fn step(&mut self) -> Result<(), Error> {
        self.current += self.step;
        let f = self.current.clamp(0.0, 1.0);
        let res = self.range.0 + f * (self.range.1 - self.range.0);
        self.msg.send(RecvMsg::ReadProgress(res))?;
        Ok(())
    }

    pub fn set_step(&mut self, count: usize) {
        self.step = 1.0 / usize::max(1, count) as f32;
    }

    fn new(start: f32, end: f32, msg: &crossbeam_channel::Sender<RecvMsg>) -> Progress {
        Progress {
            range: (start, end),
            step: 0.1,
            current: 0.0,
            msg,
        }
    }
}

#[derive(PartialEq, Debug, Default)]
pub enum NodeType {
    #[default]
    Unknown,
    File,
    Directory,
    Archive,
    Other(usize),
}

#[derive(Debug, Copy, Clone)]
enum DriverIndex {
    /// Single driver such as a file loader
    Single(i32),
    /// Dual driver such as local file loader and memory zip loader
    Dual(i32, i32),
}

#[derive(Default, Debug)]
pub struct Node {
    node_type: NodeType,
    name: String,
    driver_index: Option<i32>,
    io_driver_index: Option<i32>,
    parent: u32,
    nodes: Vec<u32>,
}

impl Node {
    fn new_node(name: String, parent: u32, node_type: NodeType) -> Self {
        Node {
            node_type,
            parent,
            name,
            .. Default::default()
        }
    }

    fn new_directory_node(name: String, parent: u32) -> Node {
        Self::new_node(name, parent, NodeType::Directory)
    }

    fn new_unknown_node(name: String, parent: u32) -> Node {
        Self::new_node(name, parent, NodeType::Unknown)
    }

    fn new_file_node(name: String, parent: u32) -> Node {
        Self::new_node(name, parent, NodeType::File)
    }
}

pub type IoDriverType = Box<dyn IoDriver>;
pub type MemoryDriverType = Box<dyn MemoryDriver>;
type IoDrivers = Arc<RwLock<Vec<IoDriverType>>>;
type MemoryDrivers = Arc<RwLock<Vec<MemoryDriverType>>>;

enum NodeDriver {
    IoDriver(IoDriverType),
    MemoryDriver(MemoryDriverType),
}

struct CachedDataEntry {
    path: String,
    data: Box<[u8]>,
}

#[derive(Default)]
struct State {
    nodes: Vec<Node>,
    node_drivers: Vec<NodeDriver>,
    cached_data: Vec<CachedDataEntry>,
}

impl State {
    fn new() -> State {
        State {
            nodes: vec![Node::new_directory_node("root".into(), 0)],
            cached_data: Vec::with_capacity(MAX_CACHE_COUNT),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fileorama {
    /// for sending messages to the main-thread
    main_send: crossbeam_channel::Sender<SendMsg>,
}

impl Fileorama {
    /// Loads the first file given a url. If an known archive is encounterd the first file will be extracted
    /// (given if it has a file listing) and that will be returned until a file is encountered. If there are
    /// no files an error will/archive will be returned instead and the user code has to handle it
    pub fn load_url(&self, path: &str) -> Handle {
        let (thread_send, main_recv) = unbounded::<RecvMsg>();

        self.main_send
            .send(SendMsg::LoadUrl(path.into(), 0, thread_send))
            .unwrap();

        Handle { recv: main_recv }
    }

    pub fn add_io_driver(&self, driver: IoDriverType) {
        self.main_send.send(SendMsg::AddIoDriver(driver)).unwrap();
    }

    pub fn add_memory_driver(&self, driver: MemoryDriverType) {
        self.main_send
            .send(SendMsg::AddMemoryDriver(driver))
            .unwrap();
    }

    pub fn new(thread_count: usize) -> Self {
        let (main_send, thread_recv) = unbounded::<SendMsg>();

        let io_drivers: IoDrivers = Arc::new(RwLock::new(vec![
            Box::new(ftp_fs::FtpFs::new()),
            Box::new(local_fs::LocalFs::new()),
        ]));

        let memory_drivers: MemoryDrivers =
            Arc::new(RwLock::new(vec![Box::new(zip_fs::ZipFs::new())]));

        let thread_count = usize::max(1, thread_count);

        for i in 0..thread_count {
            let thread_recv = thread_recv.clone();
            let name = format!("vfs_worker_{}", i);
            let i_drivers = Arc::clone(&io_drivers);
            let mem_drivers = Arc::clone(&memory_drivers);

            thread::Builder::new()
                .name(name.to_owned())
                .spawn(move || {
                    let mut state = State::new();

                    while let Ok(msg) = thread_recv.recv() {
                        handle_msg(&mut state, &name, &msg, &i_drivers, &mem_drivers);
                    }
                })
                .unwrap();
        }

        Self { main_send }
    }
}

impl Default for Fileorama {
    fn default() -> Self {
        Self::new(2)
    }
}

pub enum SendMsg {
    LoadUrl(String, u32, crossbeam_channel::Sender<RecvMsg>),
    AddIoDriver(IoDriverType),
    AddMemoryDriver(MemoryDriverType),
}

fn handle_error(err: Error, msg: &crossbeam_channel::Sender<RecvMsg>) {
    if let Error::FileError(err) = err {
        let file_error = format!("{err:#?}");
        if let Err(send_err) = msg.send(RecvMsg::Error(err.into())) {
            error!(
                "evfs: Unable to send file error {:#?} to main thread due to {:#?}",
                file_error, send_err
            );
        }
    }
}

// TODO: uses a hashmap instead?
fn find_entry_in_node(node: &Node, nodes: &[Node], name: &str) -> Option<usize> {
    for n in &node.nodes {
        let index = *n as usize;
        let t = &nodes[index];
        if t.name == name {
            return Some(index);
        }
    }

    None
}

fn get_component_name<'a>(component: &'a Component, had_prefix: &mut bool) -> Cow<'a, str> {
    match component {
        Component::RootDir => {
            if !*had_prefix {
                "/".into()
            } else {
                "".into()
            }
        }

        Component::Prefix(t) => {
            *had_prefix = true;
            t.as_os_str().to_string_lossy()
        }
        Component::Normal(t) => t.to_string_lossy(),
        e => unimplemented!("{:?}", e),
    }
}

// Add a new node to the vfs at a specific index and get the new node index back
fn add_new_node(state: &mut State, index: usize, new_node: Node) -> usize {
    //let mut node = &state.nodes[index];
    let new_index = state.nodes.len();
    state.nodes.push(new_node);
    state.nodes[index].nodes.push(new_index as u32);
    new_index
}

fn add_path_to_vfs(vfs: &mut State, index: usize, path: &Path) -> (usize, usize) {
    let mut count = 0;
    let mut prefix = false;
    let mut current_index = index;

    for c in path.components() {
        let new_node = Node {
            node_type: NodeType::Unknown,
            parent: current_index as _,
            name: get_component_name(&c, &mut prefix).to_string(),
            ..Default::default()
        };

        current_index = add_new_node(vfs, current_index, new_node);
        count += 1;
    }

    (current_index, count)
}

fn add_files_dirs_to_vfs(
    vfs: &mut State,
    components: &[Component],
    in_index: usize,
    files_dirs: FilesDirs,
) -> usize {
    let mut index = in_index;
    let mut had_prefix = false;
    let mut search_nodes = true;

    // First loop over the path and see if we need create k
    for c in components.iter() {
        let node = &vfs.nodes[index];
        let component_name = get_component_name(c, &mut had_prefix);
        if search_nodes {
            if let Some(entry) = find_entry_in_node(node, &vfs.nodes, &component_name) {
                index = entry;
                continue;
            } else {
                search_nodes = false;
            }
        }

        // if we are here we need to add the remaining nodes to the vfs
        let new_node = Node::new_unknown_node(component_name.into(), index as _);
        index = add_new_node(vfs, index, new_node);
    }

    // if the last node isn't empty we assume that it has been filed already
    if !vfs.nodes[index].nodes.is_empty() {
        return index;
    }

    for name in files_dirs.dirs {
        let new_node = Node::new_unknown_node(name, index as _);
        add_new_node(vfs, index, new_node);
    }

    for name in files_dirs.files {
        let new_node = Node::new_file_node(name, index as _);
        add_new_node(vfs, index, new_node);
    }

    index
}

#[derive(Debug, PartialEq)]
enum LoadState {
    FindNode,
    FindDriverUrl,
    FindDriverData,
    LoadFromIoDriver,
    LoadFromDriver,
    LoadFromNode,
    UnsupportedPath,
    Done,
}

struct Loader<'a> {
    state: LoadState,
    path_components: Vec<Component<'a>>,
    path_str: String,
    component_index: usize,
    node_index: usize,
    driver_index: isize,
    had_prefix: bool,
    data: Option<Box<[u8]>>,
    msg: &'a crossbeam_channel::Sender<RecvMsg>,
}

/// Loading of urls works in the following way:
/// 1. First we start with the full urls for example: ftp://dir/foo.zip/test/bar.mod
/// 2. We try to load the url as is. In this case it will fail as only ftp://dir/foo.zip is present on the ftp server
/// 3. We now search backwards until we get get a file loaded (in this case ftp://dir/foo.zip)
/// 4. As we haven't fully resolved the full path yet, we will from this point search backwards again again with foo.zip
///    trying to resolve "foo.zip/test/bar.mod" which should succede in this case.
///    We repeat this process until everthing we are done.
impl<'a> Loader<'a> {
    fn new(path: &'a str, msg: &'a crossbeam_channel::Sender<RecvMsg>) -> Loader<'a> {
        Loader {
            state: LoadState::FindNode,
            path_components: Path::new(path).components().collect(),
            path_str: path.to_owned(),
            component_index: 0,
            node_index: 0,
            driver_index: -1,
            had_prefix: false,
            data: None,
            msg,
        }
    }

    // Search the vfs if we already have the path or parts of it to figure out how it should be loaded
    fn find_node(&mut self, vfs: &State) {
        let components = &self.path_components[self.component_index..];
        let mut has_local_parent_driver = false;
        let mut found_driver = false;

        trace!("Searching for node: {:?}", components);

        // Search for node in the vfs
        for c in components.iter() {
            let node = &vfs.nodes[self.node_index];
            let component_name = get_component_name(c, &mut self.had_prefix);
            if let Some(entry) = find_entry_in_node(node, &vfs.nodes, &component_name) {
                if let Some(index) = vfs.nodes[entry].driver_index {
                    let has_local_parent_driver = match vfs.node_drivers[index as usize] {
                        NodeDriver::IoDriver(ref driver) => driver.is_remote(),
                        _ => false,
                    };

                    found_driver = true;
                }

                trace!("found node {}", component_name);
                self.node_index = entry;
                self.component_index += 1;
                // If we don't have any driver yet and we didn't find a path we must search for a driver
            } else if self.component_index == 0 {
                trace!(
                    "Switching FindNode -> FindDriverUrl: {:?}",
                    &components[self.component_index..]
                );
                self.state = LoadState::FindDriverUrl;
                return;
            } else {
                // if the node has a local parent driver we try to open from the full path
                if has_local_parent_driver || !found_driver {
                    trace!("Switching FindNode -> FindDriverUrl: (root)");
                    self.component_index = 0;
                    self.node_index = 0;
                    self.state = LoadState::FindDriverUrl;
                } else {
                    trace!(
                        "Switching FindNode -> LoadFromNode: {} : {}",
                        component_name,
                        self.component_index
                    );
                    self.state = LoadState::LoadFromNode;
                }
                return;
            }
        }

        // if we didn't find any driver for the node we try to start from the begining instead
        if !found_driver {
            trace!("Switching FindNode -> FindDriverUrl: (root)");
            self.component_index = 0;
            self.node_index = 0;
            self.state = LoadState::FindDriverUrl;
        } else {
            trace!("Loading from node, index {}", self.component_index);
            // If we searched the whole tree and found the at last entry we try to load from it
            self.state = LoadState::LoadFromNode;
        }
    }

    // Walk the url backwards to find a driver
    fn find_driver_url(&mut self, vfs: &mut State, drivers: &IoDrivers) {
        let components = &self.path_components[self.component_index..];
        let mut p: PathBuf = components.iter().collect();
        let mut current_path: String = p.to_string_lossy().into();

        while !current_path.is_empty() {
            for d in &*drivers.read().unwrap() {
                if !d.supports_url(&current_path) {
                    continue;
                }

                if let Some(new_driver) = d.create_from_url(&current_path) {
                    let driver_name = new_driver.name();
                    // If we found a driver we mount it inside the vfs
                    self.driver_index = vfs.node_drivers.len() as _;
                    vfs.node_drivers.push(NodeDriver::IoDriver(new_driver));

                    trace!(
                        "Creating new driver: {} at {} - comp index {}",
                        driver_name,
                        current_path,
                        self.component_index
                    );

                    let res = add_path_to_vfs(vfs, self.node_index, &p);
                    self.node_index = res.0;
                    self.component_index += res.1;

                    vfs.nodes[self.node_index].driver_index = Some(self.driver_index as _);

                    self.state = LoadState::LoadFromIoDriver;

                    return;
                }
            }

            p.pop();
            current_path = p.to_string_lossy().into();
        }

        // Unable to find a driver to load
        self.state = LoadState::UnsupportedPath;
    }

    // Find a driver given input data at a node. If a driver is found we switch to state LoadFromDriver
    fn find_driver_data(&mut self, vfs: &mut State, drivers: &MemoryDrivers) -> Result<(), Error> {
        let node_data = self.data.as_ref().unwrap();

        trace!("Trying to find memory driver");

        for d in &*drivers.read().unwrap() {
            if !d.can_create_from_data(node_data) {
                continue;
            }

            // TODO: Fix this clone
            // Found a driver for this data. Updated the node index with the new driver
            // and switch state to load that from the new driver
            if let Some(new_driver) = d.create_from_data(node_data.clone()) {
                self.driver_index = vfs.node_drivers.len() as _;

                vfs.node_drivers.push(NodeDriver::MemoryDriver(new_driver));
                vfs.nodes[self.node_index].driver_index = Some(self.driver_index as _);

                self.state = LoadState::LoadFromDriver;
                return Ok(());
            }
        }

        trace!("No driver found, sending data as is {}", node_data.len());

        // No driver found data. So we just send it back here
        // TODO: Implement scanning here
        let t = node_data.clone();
        self.send_data(vfs, t)?;
        Ok(())
    }

    // Walk a path backwards and try to load the url given a driver
    fn load_from_driver(&mut self, vfs: &mut State) -> Result<(), Error> {
        let components = &self.path_components[self.component_index..];

        let mut p: PathBuf = components.iter().collect();
        let mut current_path: String = p.to_string_lossy().into();

        let _driver_name = match vfs.node_drivers[self.driver_index as usize] {
            NodeDriver::IoDriver(ref driver) => driver.name(),
            NodeDriver::MemoryDriver(ref driver) => driver.name(),
        };

        trace!(
            "Loading from driver {} : {} - type {}",
            &current_path,
            self.driver_index,
            _driver_name,
        );

        // walk backwards from the current path and try to load the data
        loop {
            // TODO: Fix range
            let driver = self.driver_index as usize;
            let mut progress = Progress::new(0.0, 1.0, self.msg);

            let load_msg = match vfs.node_drivers[driver] {
                NodeDriver::IoDriver(ref mut io_driver) => {
                    dbg!();
                    let msg = io_driver.load(&current_path, &mut progress)?;
                    dbg!(&msg);

                    match msg {
                        LoadStatus::Data(in_data) => {
                            trace!("Switching to find driver data");
                            self.data = Some(in_data);
                            self.state = LoadState::FindDriverData;
                            return Ok(());
                        },

                        _ => (),
                    }

                    msg
                }
                NodeDriver::MemoryDriver(ref mut mem_driver) => {
                    mem_driver.load(&current_path, &mut progress)?
                }
            };

            match load_msg {
                LoadStatus::Directory => {
                    return self.add_dir_to_vfs(
                        vfs,
                        self.component_index,
                        &current_path,
                        &mut progress,
                        driver,
                        self.node_index,
                    );
                }

                LoadStatus::Data(in_data) => {
                    trace!("Current path {:?}", current_path);
                    // if level is 0 then we are done, otherwise we have to continue
                    // TODO: If user has "scan" on data we need to continue here as well
                    //if current_path.is_empty() {
                    //    self.send_data(vfs, in_data)?;
                    //} else {
                        let res = add_path_to_vfs(vfs, self.node_index, &p);
                        self.node_index = res.0;
                        self.component_index += res.1;
                        // Add new nodes to the vfs
                        self.data = Some(in_data);
                        self.state = LoadState::FindDriverData;
                    //}

                    return Ok(());
                }

                _ => (),
            }

            p.pop();
            current_path = p.to_string_lossy().into();

            if current_path.is_empty() {
                break;
            }
        }

        if current_path.is_empty() && self.state == LoadState::LoadFromDriver {
            self.state = LoadState::UnsupportedPath;
        }

        Ok(())
    }

    fn load_from_io_driver(&mut self, vfs: &mut State) -> Result<(), Error> {
        let driver_index = self.driver_index as usize;
        match vfs.node_drivers[driver_index] {
            NodeDriver::IoDriver(ref mut driver) => {
                trace!("Loading from driver type {}", driver.name());
                let mut progress = Progress::new(0.0, 1.0, self.msg);
                let msg = driver.load("", &mut progress)?;

                match msg {
                    LoadStatus::Data(in_data) => {
                        trace!("Switching to find driver data");
                        self.data = Some(in_data);
                        self.state = LoadState::FindDriverData;
                    },

                    LoadStatus::Directory => {
                        return self.add_dir_to_vfs(
                            vfs,
                            self.component_index,
                            "",
                            &mut progress,
                            driver_index,
                            self.node_index,
                        );
                    }

                    LoadStatus::NotFound => return Err(Error::FileDirNotFound),
                }
            }

            _ => return Err(Error::Generic("Not a io driver".to_string())),
        };

        Ok(())
    }

    // When loading directly from a node we need to search backwards for a valid driver incase
    // the active node doesn't have one. This happens for example if we try to load from zip/file.bin
    // The current node would be "file.bin" but we need to load the data from the parent so the driver
    // will see the "file.bin" as input path
    fn load_from_node(&mut self, vfs: &mut State) -> Result<(), Error> {
        let mut node_index = self.node_index;

        // if we have travered to the end of the path and we know that it's a directory we don't need to ask the driver
        // to load any data and we can just return it back directly here.
        if self.component_index == self.path_components.len()
            && vfs.nodes[node_index].node_type == NodeType::Directory
        {
            trace!(
                "Sending cached directory for node {}",
                vfs.nodes[node_index].name
            );
            self.send_directory_for_node(vfs, node_index)?;
            self.state = LoadState::Done;
            return Ok(());
        }

        trace!("comp index {}", self.component_index);

        for i in (0..=self.component_index).rev() {
            let driver_index = vfs.nodes[node_index].driver_index;

            trace!("iter {}", i);

            // Search for a node that has a proper driver
            if let Some(driver_index) = driver_index {
                let components = &self.path_components[i..];
                let p: PathBuf = components.iter().collect();
                let current_path: String = p.to_string_lossy().into();

                trace!(
                    "loading from driver {} path {}",
                    driver_index,
                    &current_path
                );

                // construct the path to load from the driver
                let mut progress = Progress::new(0.0, 1.0, self.msg);
                let load_msg = match vfs.node_drivers[driver_index as usize] {
                    NodeDriver::IoDriver(ref mut driver) => {
                        driver.load(&current_path, &mut progress)?
                    }
                    NodeDriver::MemoryDriver(ref mut driver) => {
                        driver.load(&current_path, &mut progress)?
                    }
                };

                match load_msg {
                    LoadStatus::Directory => {
                        return self.add_dir_to_vfs(
                            vfs,
                            i,
                            &current_path,
                            &mut progress,
                            driver_index as usize,
                            self.node_index,
                        );
                    }
                    LoadStatus::Data(in_data) => self.send_data(vfs, in_data)?,
                    LoadStatus::NotFound => self.msg.send(RecvMsg::NotFound)?,
                }

                self.state = LoadState::Done;
                return Ok(());
            }

            node_index = vfs.nodes[node_index].parent as _;
        }

        self.msg.send(RecvMsg::NotFound)?;
        self.state = LoadState::Done;

        Ok(())
    }

    fn add_dir_to_vfs(
        &mut self,
        vfs: &mut State,
        comp_index: usize,
        current_path: &str,
        progress: &mut Progress,
        driver: usize,
        index: usize,
    ) -> Result<(), Error> {
        let mut node_index = index;
        let components = &self.path_components[comp_index..];

        trace!(
            "Found directory {} - {} - {:?}",
            vfs.nodes[index].name,
            current_path,
            components
        );

        if vfs.nodes[node_index].node_type != NodeType::Directory {
            // If the node type is unknown it means that we haven't fetched the dirs for
            // this node yet, so do that and update the node type
            //if vfs.nodes[node_index].node_type == NodeType::Unknown {
            let files_dirs = match vfs.node_drivers[driver] {
                NodeDriver::IoDriver(ref mut driver) => {
                    driver.get_directory_list(current_path, progress)?
                }
                NodeDriver::MemoryDriver(ref mut driver) => {
                    driver.get_directory_list(current_path, progress)?
                }
            };

            node_index = add_files_dirs_to_vfs(vfs, components, node_index, files_dirs);
            vfs.nodes[node_index].node_type = NodeType::Directory;
        }

        self.send_directory_for_node(vfs, node_index)?;
        self.state = LoadState::Done;

        Ok(())
    }

    // Traverses the children of a node, gets all the names and sents it back to the host
    fn send_directory_for_node(&mut self, vfs: &State, node_index: usize) -> Result<(), Error> {
        let source_node = &vfs.nodes[node_index];
        let mut files = Vec::with_capacity(source_node.nodes.len());
        let mut dirs = Vec::with_capacity(source_node.nodes.len());

        for i in &source_node.nodes {
            let node = &vfs.nodes[*i as usize];
            if node.node_type == NodeType::File {
                files.push(node.name.to_owned());
            } else {
                dirs.push(node.name.to_owned());
            }
        }

        self.msg
            .send(RecvMsg::Directory(FilesDirs::new(files, dirs)))?;
        Ok(())
    }

    fn send_data(&mut self, vfs: &mut State, data: Box<[u8]>) -> Result<(), Error> {
        // check if the cache is full, in that case remove the last entry
        if vfs.cached_data.len() >= MAX_CACHE_COUNT {
            vfs.cached_data.remove(0);
        }

        let ret_data = Data::new(&data);

        let cache_entry = CachedDataEntry {
            path: self.path_str.to_owned(),
            data,
        };

        vfs.cached_data.push(cache_entry);

        self.msg.send(RecvMsg::ReadDone(ret_data))?;
        self.state = LoadState::Done;

        Ok(())
    }
}

/*
fn print_tree(state: &State, index: u32, _parent: u32, indent: usize) {
    let node = &state.nodes[index as usize];

    println!(
        "{:indent$} {} driver {}",
        "",
        node.name,
        node.driver_index,
        indent = indent
    );

    for n in &node.nodes {
        print_tree(state, *n, node.parent, indent + 1);
    }
}
*/

pub(crate) fn load(
    vfs: &mut State,
    path: &str,
    msg: &crossbeam_channel::Sender<RecvMsg>,
    io_drivers: &IoDrivers,
    mem_drivers: &MemoryDrivers,
) -> Result<(), Error> {
    let mut loader = Loader::new(path, msg);

    // first we look in the cache if we have data there and then send that back
    for e in &vfs.cached_data {
        if e.path == path {
            trace!("Sending data for path {} as cached", path);
            msg.send(RecvMsg::ReadDone(Data::new(&e.data)))?;
            return Ok(());
        }
    }

    trace!("start processing {}", path);
    //print_tree(vfs, 0, 0, 0);

    loop {
        trace!("{:?}", loader.state);

        match loader.state {
            LoadState::FindNode => loader.find_node(vfs),
            LoadState::FindDriverUrl => loader.find_driver_url(vfs, io_drivers),
            LoadState::FindDriverData => loader.find_driver_data(vfs, mem_drivers)?,
            LoadState::LoadFromIoDriver => loader.load_from_io_driver(vfs)?,
            LoadState::LoadFromDriver => loader.load_from_driver(vfs)?,
            LoadState::LoadFromNode => loader.load_from_node(vfs)?,
            LoadState::Done => break,
            LoadState::UnsupportedPath => break,
        }
    }

    Ok(())
}

fn handle_msg(
    vfs: &mut State,
    _name: &str,
    msg: &SendMsg,
    io_drivers: &IoDrivers,
    mem_drivers: &MemoryDrivers,
) {
    match msg {
        SendMsg::LoadUrl(path, _node_index, msg) => {
            if let Err(e) = load(vfs, path, msg, io_drivers, mem_drivers) {
                handle_error(e, msg);
            }
        }
        // Add a io new driver. These drivers are pushed at the front of the list which
        // means that they will be tried first when looking for new drivers
        SendMsg::AddIoDriver(driver) => {
            let mut drivers = io_drivers.write().unwrap();
            drivers.insert(0, driver.create_instance());
        }
        // Add a memory new driver. These drivers are pushed at the front of the list which
        // means that they will be tried first when looking for new drivers
        SendMsg::AddMemoryDriver(driver) => {
            let mut drivers = mem_drivers.write().unwrap();
            drivers.insert(0, driver.create_instance());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfs_load_zip() {
        let mut path = std::fs::canonicalize("data/a.zip").unwrap();
        path = path.join("beat.zip/foo/6beat.mod");

        let vfs = Fileorama::new(1);
        let handle = vfs.load_url(&path.to_string_lossy());

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(_data)) = handle.recv.try_recv() {
                return;
            }

            thread::sleep(std::time::Duration::from_millis(10));
        }

        panic!();
    }

    #[test]
    fn vfs_local_directory() {
        let path = std::fs::canonicalize("data").unwrap();

        let vfs = Fileorama::new(1);
        let handle = vfs.load_url(&path.to_string_lossy());
        let mut found_first = false;

        for _ in 0..100 {
            if let Ok(RecvMsg::Directory(data)) = handle.recv.try_recv() {
                assert_eq!(data.files.len(), 2);
                assert_eq!(data.dirs.len(), 1);
                assert!(data.files.iter().any(|v| *v == "a.zip"));
                assert!(data.files.iter().any(|v| *v == "beat.zip"));
                assert!(data.dirs.iter().any(|v| *v == "test_dir"));
                found_first = true;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }
        
        assert!(found_first);

        let path = std::fs::canonicalize("data").unwrap();
        let handle = vfs.load_url(&path.to_string_lossy());

        for _ in 0..100 {
            if let Ok(RecvMsg::Directory(data)) = handle.recv.try_recv() {
                assert_eq!(data.files.len(), 2);
                assert_eq!(data.dirs.len(), 1);
                assert!(data.files.iter().any(|v| *v == "a.zip"));
                assert!(data.files.iter().any(|v| *v == "beat.zip"));
                assert!(data.dirs.iter().any(|v| *v == "test_dir"));
                return;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        panic!();
    }

    #[test]
    fn vfs_local_dir_zip_file() {
        let path = std::fs::canonicalize("data").unwrap();

        let vfs = Fileorama::new(1);
        let handle = vfs.load_url(&path.to_string_lossy());
        let mut found_first = false;

        for _ in 0..100 {
            if let Ok(RecvMsg::Directory(data)) = handle.recv.try_recv() {
                assert_eq!(data.files.len(), 2);
                assert_eq!(data.dirs.len(), 1);
                assert!(data.files.iter().any(|v| *v == "a.zip"));
                assert!(data.files.iter().any(|v| *v == "beat.zip"));
                assert!(data.dirs.iter().any(|v| *v == "test_dir"));
                found_first = true;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        assert!(found_first);

        let path = std::fs::canonicalize("data").unwrap();
        let path = path.join("beat.zip/foo/6beat.mod");

        let handle = vfs.load_url(&path.to_string_lossy());

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(data)) = handle.recv.try_recv() {
                assert!(data.get().len() > 2);
                return;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        panic!();
    }

    #[test]
    fn vfs_two_local_files() {
        let vfs = Fileorama::new(1);

        let path = std::fs::canonicalize(".").unwrap();
        let path = path.join("Cargo.toml");

        let handle = vfs.load_url(&path.to_string_lossy());
        let mut read_cargo_toml = false;

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(_data)) = handle.recv.try_recv() {
                read_cargo_toml = true;
                break;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        let path = std::fs::canonicalize(".").unwrap();
        let path = path.join("data/test_dir/dummy");

        let handle = vfs.load_url(&path.to_string_lossy());
        let mut read_dummy = false;

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(_data)) = handle.recv.try_recv() {
                read_dummy = true;
                break;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        assert!(read_cargo_toml);
        assert!(read_dummy);
    }

    #[test]
    fn vfs_read_same_file_twice() {
        let vfs = Fileorama::new(1);

        let path = std::fs::canonicalize(".").unwrap();
        let path = path.join("Cargo.toml");

        let handle = vfs.load_url(&path.to_string_lossy());
        let mut data_size = 1;
        let mut data_size_2 = 2;

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(data)) = handle.recv.try_recv() {
                data_size = data.get().len();
                break;
            }
            thread::sleep(std::time::Duration::from_millis(1));
        }

        let handle = vfs.load_url(&path.to_string_lossy());

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(data)) = handle.recv.try_recv() {
                data_size_2 = data.get().len();
                break;
            }

            thread::sleep(std::time::Duration::from_millis(1));
        }

        assert_eq!(data_size, data_size_2);
    }

    #[test]
    fn vfs_zip_dir() {
        let path = std::fs::canonicalize("data/a.zip").unwrap();

        let vfs = Fileorama::new(1);
        let handle = vfs.load_url(&path.to_string_lossy());

        for _ in 0..100 {
            if let Ok(RecvMsg::Directory(data)) = handle.recv.try_recv() {
                assert_eq!(data.files.len(), 1);
                assert_eq!(data.dirs.len(), 1);
                assert!(data.files.iter().any(|v| *v == "beat.zip"));
                assert!(data.dirs.iter().any(|v| *v == "foo"));
                return;
            }

            thread::sleep(std::time::Duration::from_millis(10));
        }

        panic!();
    }

    #[test]
    fn ftp_test_file() {
        let vfs = Fileorama::new(1);
        let handle = vfs.load_url("ftp://ftp.modland.com/readme_welcome.txt");

        for _ in 0..100 {
            if let Ok(RecvMsg::ReadDone(data)) = handle.recv.try_recv() {
                let welcome = std::str::from_utf8(data.get()).unwrap();
                assert!(welcome.contains("Welcome to Modland"));
                return;
            }

            thread::sleep(std::time::Duration::from_millis(22));
        }

        panic!();
    }

    #[test]
    fn ftp_test_large_file() {
        let vfs = Fileorama::new(1);
        let handle = vfs.load_url("ftp://ftp.modland.com/allmods.zip");

        for _ in 0..1000 {
            match handle.recv.try_recv() {
                Ok(RecvMsg::Directory(data)) => {
                    assert!(data.files.iter().any(|v| *v == "allmods.txt"));
                    return;
                }
                Ok(RecvMsg::ReadProgress(_data)) => {
                    //println!("Progress: {:?}", data);
                }
                _ => thread::sleep(std::time::Duration::from_millis(50)),
            }
        }

        panic!();
    }

    #[test]
    fn ftp_test_directory_1() {
        let vfs = Fileorama::new(1);
        let handle = vfs.load_url("ftp://ftp.modland.com");

        for _ in 0..100 {
            match handle.recv.try_recv() {
                Ok(RecvMsg::Directory(data)) => {
                    assert!(data.dirs.iter().any(|v| *v == "pub"));
                    assert!(data.dirs.iter().any(|v| *v == "incoming"));
                    return;
                }
                _ => thread::sleep(std::time::Duration::from_millis(50)),
            }

            thread::sleep(std::time::Duration::from_millis(50));
        }

        panic!();
    }
}
