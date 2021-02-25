pub mod modules;
use futures::channel::mpsc;
use utopia_common::module;
use futures::stream;

pub use modules::ThreadHandle;

pub struct ModuleCore {
    pub mod_mgr: modules::ModuleManager,
    pub futures: stream::FuturesUnordered<ThreadHandle>
}

impl ModuleCore {
    pub fn new() -> failure::Fallible<(ModuleCore, mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>)> {
        let mut mod_mgr = modules::ModuleManager::new();
        let (mod_send, mod_recv) = mpsc::unbounded::<(&'static str, module::ModuleCommands)>();
        let futures = stream::FuturesUnordered::new();
        unsafe {
            match mod_mgr.load_module(&std::ffi::OsStr::new("../utopia-sample-module/target/debug/libsample_mod.so"), mod_send) {
                Ok(handle) => futures.push(handle),
                Err(e) => eprintln!("Error loading module: {}", e)
            }
        }
        Ok((ModuleCore {
            mod_mgr,
            futures
        }, mod_recv))
    }

    pub fn get_modules(&self) {
        for (id, module) in &self.mod_mgr.modules {
            println!("Module: {} has name: {}", id, module.module.get_module_info().name)
        }
    }
}
