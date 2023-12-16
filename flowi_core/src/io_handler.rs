use fileorama::{Fileorama, RecvMsg};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum LoadedData {
    Data(Box<[u8]>),
    Error(String),
}

pub type IoHandle = u64;

pub(crate) struct IoHandler {
    vfs: fileorama::Fileorama,
    /// Images that are currently being loaded (i.e async)
    pub(crate) inflight: Vec<(u64, fileorama::Handle)>,
    /// Images that have been loaded
    pub(crate) loaded: HashMap<u64, LoadedData>,
    id_counter: u64,
}

impl IoHandler {
    pub fn new(vfs: &Fileorama) -> Self {
        Self {
            vfs: vfs.clone(),
            inflight: Vec::new(),
            loaded: HashMap::new(),
            id_counter: 1,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_loaded(&self, id: u64) -> bool {
        self.loaded.contains_key(&id)
    }

    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.inflight.len() {
            let (id, handle) = &self.inflight[i];
            match handle.recv.try_recv() {
                //RecvMsg::Progress(Progress { loaded, total }) => { },
                Ok(RecvMsg::ReadDone(data)) => {
                    self.loaded.insert(*id, LoadedData::Data(data));
                    self.inflight.remove(i);
                }
                Ok(RecvMsg::Error(e)) => {
                    dbg!(&e);
                    self.loaded.insert(*id, LoadedData::Error(e));
                    self.inflight.remove(i);
                }

                Ok(RecvMsg::NotFound) => {
                    dbg!("NotFound");
                    self.loaded
                        .insert(*id, LoadedData::Error("File not found".to_string()));
                    self.inflight.remove(i);
                }

                _ => {}
            }

            i += 1;
        }
    }

    // Async loading
    #[allow(dead_code)]
    pub(crate) fn load(&mut self, url: &str) -> IoHandle {
        let id = self.id_counter;
        self.id_counter += 1;
        let handle = self.vfs.load_url(url);
        self.inflight.push((id, handle));
        id
    }

    /// Async load a url where the memory loader has to be a specific type. This is useful if you
    /// want to load a certain file type (such as a image) you can specifiy that the specific
    /// driver loading the data is an image driver. If the driver fails (with broken data for
    /// example) an error will be returned instead.
    pub fn load_with_driver(&mut self, url: &str, driver_name: &'static str) -> IoHandle {
        let id = self.id_counter;
        self.id_counter += 1;
        let handle = self.vfs.load_url_with_driver(url, driver_name);
        self.inflight.push((id, handle));
        id
    }
}
