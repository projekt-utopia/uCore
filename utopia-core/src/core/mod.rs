use utopia_module::com::{library, CoreCommands};
use crate::{modules::modules::ModuleManager, errors::{ProvModuleNotAvailableError, ModuleNotAvailableError, LibraryItemNotAvailableError}};
use crate::frontend::con::library as FeLibrary;

use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

use std::collections::HashMap;

#[derive(Clone)]
pub struct ItemProvider {
    pub title: String,
    module: &'static str
}

pub struct LibraryItem {
    uuid: String,
    name: String,
    kind: library::LibraryItemKind,
    details: library::LibraryItemDetails,
    active_runner: (String, ItemProvider),
    runners: HashMap<String, ItemProvider>
}
impl LibraryItem {
    pub fn new(provider: &'static str, title: String, item: library::LibraryItem) -> Self {
        let iprovider = ItemProvider { title, module: provider };
        let mut runners = HashMap::new();
        runners.insert(String::from(provider), iprovider.clone());
        LibraryItem {
            uuid: item.uuid,
            name: item.name,
            kind: item.kind,
            details: item.details,
            active_runner: (String::from(provider), iprovider),
            runners
        }
    }
    pub fn add_runner(&mut self, provider: &'static str, title: String) {
        self.runners.insert(String::from(provider), ItemProvider { title, module: provider });
    }
    pub fn run_default(&self, mod_mgr: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
        mod_mgr.get(&self.active_runner.1.module)?.send(CoreCommands::LaunchLibraryItem(self.uuid.clone()))?;
        Ok(())
    }
    pub fn run_runner(&self, mod_mgr: &ModuleManager, runner: String) -> Result<(), Box<dyn std::error::Error>> {
        match self.runners.get(&runner) {
            Some(runner) => {
                mod_mgr.get(runner.module)?.send(CoreCommands::LaunchLibraryItem(self.uuid.clone()))?;
                Ok(())
            },
            None => Err(Box::new(ProvModuleNotAvailableError::new(runner)))
        }
    }
    pub fn change_default_runner(&mut self, provider: String) -> Result<(), ProvModuleNotAvailableError> {
        let title = self.runners.get(&provider).ok_or(ProvModuleNotAvailableError::new(provider.clone()))?;
        self.active_runner = (provider, title.clone());
        Ok(())
    }
    pub fn to_frontend(&self) -> FeLibrary::LibraryItem {
        FeLibrary::LibraryItem {
            uuid: self.uuid.clone(),
            name: self.name.clone(),
            kind: LibraryItem::kind_mod_to_fe(self.kind),
            default_runner: FeLibrary::LibraryItemRunner { uuid: self.active_runner.0.clone(), name: self.active_runner.1.title.clone() },
            available_runners: self.runners.iter().map(|(k, v)| FeLibrary::LibraryItemRunner { uuid: k.to_string(), name: v.title.clone() }).collect()
        }
    }

    // TODO: this should not be here, please move
    fn kind_mod_to_fe(kind: library::LibraryItemKind) -> FeLibrary::LibraryItemKind {
        match kind {
            library::LibraryItemKind::Game => FeLibrary::LibraryItemKind::Game,
            library::LibraryItemKind::App => FeLibrary::LibraryItemKind::App,
        }
    }
}

pub struct Library {
    inner: HashMap<String, LibraryItem>
}
impl Library {
    pub fn new() -> Self {
        Library {
            inner: HashMap::new(),
        }
    }
    pub fn insert(&mut self, module: &'static str, item: library::LibraryItem, mod_mgr: &ModuleManager) -> Result<(), ModuleNotAvailableError> {
        let title = mod_mgr.get(&module)?.module.get_module_info().name;
        println!("Added {}", item.uuid.clone());
        self.inner.entry(item.uuid.clone())
            .and_modify(|item| item.add_runner(module, title.clone()))
            .or_insert(LibraryItem::new(module, title, item));
        Ok(())
    }
    pub fn bulk_insert(&mut self, module: &'static str, items: Vec<library::LibraryItem>, mod_mgr: &ModuleManager) -> Result<(), ModuleNotAvailableError> {
        for item in items {
            self.insert(module, item, mod_mgr)?;
        }
        Ok(())
    }

    pub fn get(&self, uuid: String) -> Result<&LibraryItem, LibraryItemNotAvailableError> {
        match self.inner.get(&uuid) {
            Some(item) => Ok(item),
            None => Err(LibraryItemNotAvailableError::new(uuid))
        }
    }
    pub fn get_mut(&mut self, uuid: String) -> Result<&mut LibraryItem, LibraryItemNotAvailableError> {
        match self.inner.get_mut(&uuid) {
            Some(item) => Ok(item),
            None => Err(LibraryItemNotAvailableError::new(uuid))
        }
    }

    pub fn launch_library_item(&self, uuid: String, mod_mgr: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
        self.get(uuid.clone())?.run_default(mod_mgr)?;
        Ok(())
    }

    pub fn launch_library_item_from_provider(&self, uuid: String, mod_mgr: &ModuleManager, provider: String) -> Result<(), Box<dyn std::error::Error>> {
        self.get(uuid.clone())?.run_runner(mod_mgr, provider)?;
        Ok(())
    }

    pub fn change_default_provider(&mut self, uuid: String, provider: String) -> Result<(), Box<dyn std::error::Error>> {
        self.get_mut(uuid.clone())?.change_default_runner(provider)?;
        Ok(())
    }

    pub fn to_frontend(&self) -> Vec<FeLibrary::LibraryItem> {
        self.inner.values().map(|item| item.to_frontend()).collect()
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
