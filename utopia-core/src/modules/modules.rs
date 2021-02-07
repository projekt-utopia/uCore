// thanks to Michael-F-Bryan for his "Rust FFI Guide"
// https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html
// and to harmic on SO for telling me about Arc<T>
// https://stackoverflow.com/a/65621675/10890264

pub use utopia_module::{Module, props};
use std::sync::Arc;
use futures::channel::mpsc;
use std::ffi::OsStr;
use std::fmt::{self, Formatter, Debug};
use libloading::{Library, Symbol};

pub struct IModule {
    pub module: Arc<std::boxed::Box<dyn Module>>,
    pub send: Option<mpsc::UnboundedSender<props::CoreCommands>>,
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
    pub unsafe fn load_module<P: AsRef<OsStr>>(&mut self, filename: P) -> Result<(), Box<dyn std::error::Error>> {
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
        self.modules.insert(module.id(), IModule { module: Arc::new(module), send: None });

        Ok(())
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
