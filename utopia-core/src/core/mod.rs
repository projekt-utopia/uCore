mod modules;
use futures::channel::mpsc;
use utopia_module::props;
use futures::stream::{self, StreamExt, select_all};
//use std::sync::Arc;

pub struct Core {
    mod_mgr: modules::ModuleManager
}

impl Core {
    pub fn new() -> failure::Fallible<Core> {
        let mut mod_mgr = modules::ModuleManager::new();
        unsafe {
        match mod_mgr.load_module(&std::ffi::OsStr::new("../../utopia-sample_module/target/debug/libutopia-sample_module.so")) {
            Ok(()) => {},
            Err(e) => eprintln!("Error loading module: {}", e)
        }
        }
        Ok(Core {
            mod_mgr
        })
    }
    pub async fn spawn_modules(&mut self) {
        let mut futures = stream::FuturesUnordered::new();
        for (_id, module) in &mut self.mod_mgr.modules {
            let (core_send, core_recv) = mpsc::unbounded::<props::CoreCommands>();
            let (mod_send, mod_recv) = mpsc::unbounded::<props::ModuleCommands>();

            module.send = Some(core_send);
            module.recv = Some(mod_recv);
            {
                let module = module.module.clone();
                futures.push(tokio::spawn(async move {
                    module.thread(mod_send, core_recv);
                }));
            }
        }
        /* TODO: Move this into a new function, potentially seperate from this module, as it is basically the "main loop".
           After doing this, this function does not need to be async anymore. */
        let mut receivers = select_all(self.mod_mgr.modules.values_mut().map(|v| v.recv.as_mut().expect("A module had no channel")));
        loop {
            futures::select! {
                msg = receivers.next() => {
                    match msg {
                        Some(cmd) => {
                            println!("Command: {:?}", cmd);
                        },
                        None => eprintln!("Communication channel of a module died")
                    }
                }
                _ = futures.select_next_some() => {
                    eprintln!("A module died");
                },
                complete => break
            }
        }
    }
    pub fn get_modules(&self) {
        for (id, module) in &self.mod_mgr.modules {
            println!("Module: {} has name: {}", id, module.module.get_module_info().name)
        }
    }
}
