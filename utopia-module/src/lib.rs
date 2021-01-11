pub mod props;
pub use tokio::runtime::Runtime; // reexport of tokio runtime, to use with macro
use std::any::Any;
use futures::channel::mpsc;

pub trait Module: Any + Send + Sync {
    fn id(&self) -> &'static str;
    fn get_module_info(&self) -> props::ModuleInfo;
    fn init(&mut self) {}
    fn deinit(&self) {}

    fn thread(&self, mod_send: mpsc::UnboundedSender<props::ModuleCommands>, core_recv: mpsc::UnboundedReceiver<props::CoreCommands>) -> (&'static str, std::result::Result<props::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>);
}


#[macro_export]
macro_rules! spawn_async_runtime {
    ($id:expr, $function:stmt) => {
        let rt = utopia_module::Runtime::new().unwrap();
        return ($id, rt.block_on(async { $function }));
    }
}

#[macro_export]
macro_rules! declare_module {
    ($module_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _module_create() -> *mut $crate::Module {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $module_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::Module> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
