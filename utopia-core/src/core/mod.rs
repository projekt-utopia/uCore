use utopia_module::com::{library, CoreCommands};
use crate::{modules::modules::ModuleManager, errors::LibraryItemNotAvailableError};

use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

pub struct LibraryItemSource {
    module: &'static str,
    item: library::LibraryItem
}
impl LibraryItemSource {
    pub fn launch(&self, mod_mgr: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
        mod_mgr.get(&self.module)?.send(CoreCommands::LaunchLibraryItem(self.item.uuid.clone()))?;
        Ok(())
    }
}

pub struct Library {
    inner: std::collections::HashMap<String, LibraryItemSource>
}
impl Library {
    pub fn new() -> Self {
        Library {
            inner: std::collections::HashMap::new()
        }
    }
    pub fn insert(&mut self, module: &'static str, item: library::LibraryItem) {
        println!("Added {}", item.uuid.clone());
        self.inner.insert(item.uuid.clone(), LibraryItemSource { module, item });
    }
    pub fn bulk_insert(&mut self, module: &'static str, items: Vec<library::LibraryItem>) {
        for item in items {
            self.insert(module, item);
        }
    }

    pub fn get(&self, uuid: String) -> Result<&LibraryItemSource, LibraryItemNotAvailableError> {
        match self.inner.get(&uuid) {
            Some(item) => Ok(item),
            None => Err(LibraryItemNotAvailableError::new(uuid))
        }
    }

    pub fn launch_library_item(&self, uuid: String, mod_mgr: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
        self.get(uuid.clone())?.launch(mod_mgr)?;
        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InternalCoreFutures {
    NewFrontendRegistered(String, tokio::net::UnixStream),
    Debug,
    Error(Box<dyn std::error::Error + Send>)
}

pub struct Core {
    pub library: Library,
    pub internal_futures: FuturesUnordered<JoinHandle<InternalCoreFutures>>
}
impl Core {
    pub fn new() -> Self {
        Core {
            library: Library::new(),
            internal_futures: FuturesUnordered::new()
        }
    }
}
