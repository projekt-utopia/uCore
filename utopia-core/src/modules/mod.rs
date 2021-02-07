pub mod modules;
use futures::channel::mpsc;
use utopia_module::props;
use futures::stream;
//use std::sync::Arc;

pub type ThreadHandle = tokio::task::JoinHandle<(&'static str, Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>)>;

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
    pub fn spawn_modules(&mut self) -> (stream::FuturesUnordered<ThreadHandle>, mpsc::UnboundedReceiver<(&'static str, props::ModuleCommands)>) {
        let (mod_send, mod_recv) = mpsc::unbounded::<(&'static str, props::ModuleCommands)>();
        let futures = stream::FuturesUnordered::new();
        for (_id, module) in &mut self.mod_mgr.modules {
            let (core_send, core_recv) = mpsc::unbounded::<props::CoreCommands>();
            //

            module.send = Some(core_send);
            //module.recv = Some(mod_recv);
            {
                let module = module.module.clone();
                let mod_send = mod_send.clone();
                futures.push(tokio::spawn(async move {
                    module.thread(mod_send, core_recv)
                }));
            }
        }
        (futures, mod_recv)
    }
    pub fn get_modules(&self) {
        for (id, module) in &self.mod_mgr.modules {
            println!("Module: {} has name: {}", id, module.module.get_module_info().name)
        }
    }
}
