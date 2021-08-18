pub mod modules;
use futures::channel::mpsc;
use futures::stream;
use utopia_common::module;

pub use modules::ThreadHandle;

pub struct ModuleCore {
	pub mod_mgr: modules::ModuleManager,
	pub futures: stream::FuturesUnordered<ThreadHandle>,
}

impl ModuleCore {
	pub fn new() -> failure::Fallible<(
		ModuleCore,
		mpsc::UnboundedReceiver<(&'static str, module::ModuleCommands)>,
	)> {
		let mut mod_mgr = modules::ModuleManager::new();
		let (mod_send, mod_recv) = mpsc::unbounded::<(&'static str, module::ModuleCommands)>();
		let futures = stream::FuturesUnordered::new();
		unsafe {
			match mod_mgr.load_module(
				&std::ffi::OsStr::new("../utopia-dbgfiller-module/target/debug/libdbgfiller_steam_mod.so"),
				mod_send.clone(),
			) {
				Ok(handle) => futures.push(handle),
				Err(e) => eprintln!("Error loading module: {}", e),
			}
			match mod_mgr.load_module(
				&std::ffi::OsStr::new("../utopia-sample-module/target/debug/libsample_mod.so"),
				mod_send.clone(),
			) {
				Ok(handle) => futures.push(handle),
				Err(e) => eprintln!("Error loading module: {}", e),
			}
			match mod_mgr.load_module(
				&std::ffi::OsStr::new("../utopia-gog-dbgfiller-module/target/debug/libdbgfiller_gog_mod.so"),
				mod_send,
			) {
				Ok(handle) => futures.push(handle),
				Err(e) => eprintln!("Error loading module: {}", e),
			}
		}
		Ok((ModuleCore { mod_mgr, futures }, mod_recv))
	}

	pub fn get_modules(&self) {
		for (id, module) in &self.mod_mgr.modules {
			println!("Module: {} has name: {}", id, module.module.get_module_info().name)
		}
	}
}
