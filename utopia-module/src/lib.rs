use futures::channel::mpsc;
use std::any::Any;
pub use tokio::runtime::Runtime; // reexport of tokio runtime, to use with macro
pub use redis;
pub use futures;
pub use utopia_common::module;

pub const MODULE_INTERFACE_VERSION: &'static str = "0.0.0";

pub type USend = mpsc::UnboundedSender<(&'static str, module::ModuleCommands)>;
pub type URecv = mpsc::UnboundedReceiver<module::CoreCommands>;
pub type URes = (
	&'static str,
	std::result::Result<module::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>,
);
// TODO: wrap UDb in some kind of struct, that std::fmt::Debug can be implemented
pub type UDb = std::sync::Arc<std::sync::RwLock<redis::Connection>>;

pub trait Module: Any + Send + Sync {
	fn id(&self) -> &'static str;
	fn get_module_info(&self) -> module::ModuleInfo;
	fn init(&mut self, _db: UDb) {}
	fn deinit(&self) {}

	fn thread(&self, mod_send: USend, core_recv: URecv) -> URes;

	#[doc(hidden)]
	#[inline(always)]
	fn __abi_version(&self) -> &'static str {
		MODULE_INTERFACE_VERSION
	} // Prevent loading modules with newer/older ABI, to prevent segfaults.
}

#[macro_export]
macro_rules! spawn_async_runtime {
	($id:expr, $function:stmt) => {
		let rt = utopia_module::Runtime::new().unwrap();
		return ($id, rt.block_on(async { $function }));
	};
}

#[macro_export]
macro_rules! declare_module {
	($module_type:ty, $constructor:path) => {
		#[no_mangle]
		pub extern fn _module_create() -> *mut $crate::Module {
			// make sure the constructor is the correct type.
			let constructor: fn() -> $module_type = $constructor;

			let object = constructor();
			let boxed: Box<$crate::Module> = Box::new(object);
			Box::into_raw(boxed)
		}
	};
}
