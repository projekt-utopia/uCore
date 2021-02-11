// thanks to Michael-F-Bryan for his "Rust FFI Guide"
// https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html
// and to harmic on SO for telling me about Arc<T>
// https://stackoverflow.com/a/65621675/10890264

pub use utopia_module::{Module, com};
use crate::errors::ModuleNotAvailableError;
use std::sync::Arc;
use futures::channel::mpsc;
use std::ffi::OsStr;
use std::fmt::{self, Formatter, Debug};
use libloading::{Library, Symbol};

pub type ThreadHandle = tokio::task::JoinHandle<(&'static str, Result<com::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>;

pub struct IModule {
    pub module: Arc<std::boxed::Box<dyn Module>>,
    pub send: mpsc::UnboundedSender<com::CoreCommands>,
}
impl IModule {
    pub fn new(module: Box<dyn Module>, mod_send: mpsc::UnboundedSender<(&'static str, com::ModuleCommands)>) -> (Self, ThreadHandle) {
        let (core_send, core_recv) = mpsc::unbounded::<com::CoreCommands>();
        let module = IModule {
            module: Arc::new(module),
            send: core_send
        };
        let module_ptr = module.module.clone();
        (module, tokio::spawn(async move {
            module_ptr.thread(mod_send, core_recv)
        }))

    }
    pub fn send(&self, cmd: com::CoreCommands) -> Result<(), mpsc::TrySendError<com::CoreCommands>> {
        self.send.unbounded_send(cmd)
    }
}

pub struct ModuleManager {
    pub modules: std::collections::HashMap<&'static str, IModule>,
    loaded_libraries: Vec<Library>,
}

impl ModuleManager {
    pub fn new() -> ModuleManager {
        ModuleManager {
            modules: std::collections::HashMap::new(),
            loaded_libraries: Vec::new(),
        }
    }
    pub unsafe fn load_module<P: AsRef<OsStr>>(&mut self, filename: P, mod_send: mpsc::UnboundedSender<(&'static str, com::ModuleCommands)>) -> Result<ThreadHandle, Box<dyn std::error::Error>> {
        type ModuleCreate = unsafe fn() -> *mut dyn Module;

        let lib = Library::new(filename.as_ref())?; /* (|| "Unable to load the plugin")?; */

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<ModuleCreate> = lib.get(b"_module_create")?;
            /* .chain_err(|| "The `_module_create` symbol wasn't found.")?; */
        let boxed_raw = constructor();

        let mut module = Box::from_raw(boxed_raw);
        println!("Loaded module: {}", module.get_module_info().name);
        module.init();

        let (module, resolve) = IModule::new(module, mod_send);
        self.modules.insert(module.module.id(), module);
        Ok(resolve)
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn deinit(&mut self) {
        println!("Unloading plugins");

        for (id, module) in self.modules.drain() {
            println!("Unloading {}", id);
            module.module.deinit();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }

    pub fn get(&self, uuid: &'static str) -> Result<&IModule, ModuleNotAvailableError> {
        match self.modules.get(uuid) {
            Some(module) => Ok(module),
            None => Err(ModuleNotAvailableError::new(uuid))
        }
    }
}

impl Drop for ModuleManager {
    fn drop(&mut self) {
        if !self.modules.is_empty() || !self.loaded_libraries.is_empty() {
            self.deinit();
        }
    }
}

impl Debug for ModuleManager {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let modules: Vec<_> = self.modules.iter().map(|(_id, module)|module.module.get_module_info().name).collect();

        f.debug_struct("ModuleManager")
            .field("modules", &modules)
            .finish()
    }
}
