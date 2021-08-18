use futures::channel::mpsc;
use std::any::Any;
pub use tokio::runtime::Runtime; // reexport of tokio runtime, to use with macro
pub use utopia_common::module;

pub const MODULE_INTERFACE_VERSION: &'static str = "0.0.0";

pub trait Module: Any + Send + Sync {
	fn id(&self) -> &'static str;
	fn get_module_info(&self) -> module::ModuleInfo;
	fn init(&mut self) {}
	fn deinit(&self) {}

	fn thread(
		&self,
		mod_send: mpsc::UnboundedSender<(&'static str, module::ModuleCommands)>,
		core_recv: mpsc::UnboundedReceiver<module::CoreCommands>,
	) -> (
		&'static str,
		std::result::Result<module::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>,
	);

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
