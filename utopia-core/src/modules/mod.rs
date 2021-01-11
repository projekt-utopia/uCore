pub mod modules;
use futures::channel::mpsc;
use utopia_module::props;
use futures::stream;
//use std::sync::Arc;

pub struct ModuleCore {
    mod_mgr: modules::ModuleManager
}

impl ModuleCore {
    pub fn new() -> failure::Fallible<ModuleCore> {
        let mut mod_mgr = modules::ModuleManager::new();
        unsafe {
        match mod_mgr.load_module(&std::ffi::OsStr::new("../utopia-sample-module/target/debug/libsample_mod.so")) {
            Ok(()) => {},
            Err(e) => eprintln!("Error loading module: {}", e)
        }
        }
        Ok(ModuleCore {
            mod_mgr
        })
    }
    pub fn spawn_modules(&mut self) -> stream::FuturesUnordered<tokio::task::JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync >>)>> {
        let futures = stream::FuturesUnordered::new();
        for (_id, module) in &mut self.mod_mgr.modules {
            let (core_send, core_recv) = mpsc::unbounded::<props::CoreCommands>();
            let (mod_send, mod_recv) = mpsc::unbounded::<props::ModuleCommands>();

            module.send = Some(core_send);
            module.recv = Some(mod_recv);
            {
                let module = module.module.clone();
                futures.push(tokio::spawn(async move {
                    module.thread(mod_send, core_recv)
                }));
            }
        }
        futures
    }
    pub fn get_modules(&self) {
        for (id, module) in &self.mod_mgr.modules {
            println!("Module: {} has name: {}", id, module.module.get_module_info().name)
        }
    }
    pub fn get_mod_channel_receivers(&mut self) -> std::collections::hash_map::ValuesMut<&'static str, modules::IModule> {
        self.mod_mgr.modules.values_mut()
    }
}
