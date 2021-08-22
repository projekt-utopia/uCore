// thanks to Michael-F-Bryan for his "Rust FFI Guide"
// https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html
// and to harmic on SO for telling me about Arc<T>
// https://stackoverflow.com/a/65621675/10890264

use std::{ffi::OsStr,
          fmt::{self, Debug, Formatter},
          sync::Arc};

use futures::channel::mpsc;
use libloading::{Library, Symbol};
use utopia_common::module;
pub use utopia_module::{Module, UDb, MODULE_INTERFACE_VERSION};

use crate::errors::{self, ModuleABIError, ModuleNotAvailableError};

pub type ThreadHandle = tokio::task::JoinHandle<(
	&'static str,
	Result<module::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>
)>;

pub struct IModule {
	pub module: Arc<std::boxed::Box<dyn Module>>,
	pub send: mpsc::UnboundedSender<module::CoreCommands>
}
impl IModule {
	pub fn new(
		module: Box<dyn Module>,
		mod_send: mpsc::UnboundedSender<(&'static str, module::ModuleCommands)>
	) -> (Self, ThreadHandle) {
		let (core_send, core_recv) = mpsc::unbounded::<module::CoreCommands>();
		let module = IModule {
			module: Arc::new(module),
			send: core_send
		};
		let module_ptr = module.module.clone();
		(
			module,
			tokio::spawn(async move { module_ptr.thread(mod_send, core_recv) })
		)
	}

	pub fn send(&self, cmd: module::CoreCommands) -> Result<(), mpsc::TrySendError<module::CoreCommands>> {
		self.send.unbounded_send(cmd)
	}
}

pub struct ModuleManager {
	pub modules: std::collections::HashMap<&'static str, IModule>,
	pub mod_lib: std::collections::HashMap<String, &'static str>,
	loaded_libraries: Vec<Library>,
	connection: UDb
}

impl ModuleManager {
	pub fn new(connection: UDb) -> ModuleManager {
		ModuleManager {
			modules: std::collections::HashMap::new(),
			mod_lib: std::collections::HashMap::new(),
			loaded_libraries: Vec::new(),
			connection
		}
	}

	pub unsafe fn load_module<P: AsRef<OsStr>>(
		&mut self,
		filename: P,
		mod_send: mpsc::UnboundedSender<(&'static str, module::ModuleCommands)>
	) -> Result<ThreadHandle, Box<dyn std::error::Error>> {
		type ModuleCreate = unsafe fn() -> *mut dyn Module;

		let lib = Library::new(filename.as_ref())?; /* (|| "Unable to load the plugin")?; */

		// We need to keep the library around otherwise our plugin's vtable
		// will point to garbage. We do this little dance to make sure the
		// library doesn't end up getting moved.
		self.loaded_libraries.push(lib);

		let lib = self.loaded_libraries.last().unwrap();

		let constructor: Symbol<ModuleCreate> = lib.get(b"_module_create")?;
		/* .chain_err(|| "The `_module_create` symbol wasn't found.")?; */
		let boxed_raw = constructor();

		let mut module = Box::from_raw(boxed_raw);
		if module.__abi_version() == MODULE_INTERFACE_VERSION {
			println!("Loaded module: {}", module.get_module_info().name);
			module.init(self.connection.clone());

			let (module, resolve) = IModule::new(module, mod_send);
			self.mod_lib.insert(module.module.id().to_string(), module.module.id());
			self.modules.insert(module.module.id(), module);
			Ok(resolve)
		} else {
			Err(Box::new(ModuleABIError::new(
				module.id(),
				module.__abi_version(),
				MODULE_INTERFACE_VERSION
			)))
		}
	}

	/// Unload all plugins and loaded plugin libraries, making sure to
	/// fire their `on_plugin_unload()` methods so they can do any
	/// necessary cleanup.
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

	pub fn get_owned(&self, uuid: &String) -> Result<&IModule, errors::ProvModuleNotAvailableError> {
		self.mod_lib
			.get(uuid)
			.map(|uuid| self.modules.get(uuid).unwrap())
			.ok_or(errors::ProvModuleNotAvailableError::new(uuid.to_string()))
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
		let modules: Vec<_> = self
			.modules
			.iter()
			.map(|(_id, module)| module.module.get_module_info().name)
			.collect();

		f.debug_struct("ModuleManager").field("modules", &modules).finish()
	}
}
