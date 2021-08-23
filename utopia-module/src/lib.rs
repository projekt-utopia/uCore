//! Definions for µtopia modules
//!
//! This create provides [Module](crate::Module), which is the trait
//! any module has to implement in order to be loaded by µCore.
//! Additionally it provides types and macros, that make writing such
//! a module a lot easier.
//!
//! ## Module definition
//! A module serves the main purpose of providing
//! [LibraryItems](crate::module::LibraryItem) to the
//! core, and running them when intructed to do so by it. The module
//! has to define it's runtime within
//! [Module::thread](crate::Module::thread), which will be called at
//! runtime within a [tokio task](tokio::task).
//!
//! In order to communicate with the core, the [module runtime
//! function (thread)](crate::Module::thread) provides a
//! [futures](crate::futures) [sender](crate::USend) and
//! [receiver](crate::URecv) with which the module can communicate
//! using prefedined messages
//! ([ModuleCommands](utopia_common::module::ModuleCommands) &
//! [CoreCommands](utopia_common::module::CoreCommands)).
//!
//! ### Example of adding a new library item
//! this shows how you can, with the help of [sender](crate::USend),
//! add a new library item to the core.
//! ```rust
//! use utopia_module::{module::LibraryItem, USend};
//!
//! // define a new library item
//! let item = LibraryItem {
//! 	[ ..snip.. ]
//! };
//!
//! let mod_send: USend = mod_send;
//! mod_send.unbounded_send((self.id(), module::ModuleCommands::AddLibraryItem(item)))?;
//! ```
//!
//! ## Sample module
//! The following example describes how to implement a very simple
//! module that does nothing:
//!
//! ```rust
//! use anyhow::bail;
//! use utopia_module::{Module, module, spawn_async_runtime, declare_module};
//! use utopia_module::{USend, URecv, URes};
//!
//! #[derive(Debug, Default)]
//! pub struct SampleMod;
//!
//! impl Module for SampleMod {
//! 	fn id(&self) -> &'static str {"com.github.projekt-utopia.sample_module"}
//!
//! 	fn get_module_info(&self) -> module::ModuleInfo {
//! 		module::ModuleInfo {
//! 			name: String::from("Sample µtopia module"),
//! 			url: None,
//! 			developer: String::from("sp1rit"),
//! 			developer_url: Some(String::from("https://sp1rit.ml")),
//! 			description: None,
//! 			icon: None
//! 		}
//! 	}
//!
//! 	fn thread(&self, _mod_send: USend, mut core_recv: URecv) -> URes {
//! 		spawn_async_runtime!(self.id(), {
//! 			loop {
//! 				match core_recv.next().await {
//! 					Some(msg) => println!("Received a message from core: {:?}", msg);
//! 					None => bail!("Channel to core died");
//! 				}
//! 			}
//! 			unreachable!();
//! 		})
//! 	}
//! }
//!
//! declare_module!(SampleMod, SampleMod::default);
//! ```

use std::any::Any;

use futures::channel::mpsc;
pub use tokio::runtime::Runtime; // reexport of tokio runtime, to use with macro
pub use redis;
pub use futures;
pub use utopia_common::module;

/// ABI version
///
/// string that is exported by a module and compared to the
/// string provided by [utopia_module](crate) µCore was built
/// against.
///
/// don't do anything with it
pub const MODULE_INTERFACE_VERSION: &'static str = "0.0.0";

/// the sender for the channel that facilitates messages from the
/// module to µCore
///
/// it's messages are a tuple containing the modules id and the
/// commond as defined in
/// [ModuleCommands](utopia_common::module::ModuleCommands).
///
/// ## usage in module runtime
/// ```rust
/// let mod_send: USend = mod_send;
/// let r = mod_send.unbounded_send((self.id(), module::ModuleCommands::Refresh));
/// if let Err(e) = r {
/// 	eprintln!("Unable to trigger a core refresh: {:?}", e);
/// }
/// ```
pub type USend = mpsc::UnboundedSender<(&'static str, module::ModuleCommands)>;
/// the receiver for the channel that facilitates messages from µCore
/// to a module
///
/// implements [StreamExt](futures::stream::StreamExt), so you'll
/// likely want to loop over the provided next().await method of it to
/// await new messages. For more complex modules having it as part of
/// a looped futures::select! is also possible.
///
/// ## Example
/// ```rust
/// use anyhow::bail;
/// use utopia_module::URecv;
///
/// let mut core_recv: URecv = core_recv;
/// loop {
/// 	match core_recv.next().await {
/// 		Some(msg) => println!("Received a message from core: {:?}", msg);
/// 		None => bail!("Channel to core died");
/// 	}
/// }
/// ```
pub type URecv = mpsc::UnboundedReceiver<module::CoreCommands>;
/// the return value of [Module's](crate::Module) [thread
/// function](crate::Module::thread)
///
/// it returns a tuple of a modules own id and a result of the
/// runtime. in theory the runtime should not die, however in edge
/// cases like errors or dead runtime deps it may die. See
/// [ThreadDeathExcuse](utopia_common::module::ThreadDeathExcuse) for
/// more info.
///
/// however you shoudn't have to worry about the id part, as it is
/// taken care of by [spawn_runtime!](crate::spawn_runtime) and
/// [spawn_aync_runtime!](crate::spawn_async_runtime). inside the
/// runtime you'd only have to return the result.
pub type URes = (
	&'static str,
	std::result::Result<module::ThreadDeathExcuse, Box<dyn std::error::Error + Send + Sync>>
);
/// RwLocked [redis::Connection](redis::Connection) to the database
///
/// this will be received by the [Module's](crate::Module) [init
/// function](crate::Module::init). See it for more details on
/// how to store it.
///
/// Plase make sure, that you **do not lock the db connection for
/// extended periods of time**, as it is shared between all modules.
/// Try wrapping the part where you handle the database in the
/// smallest block possible, that the writable lock guard can be
/// droped as soon as possible.
///
/// ## Example
/// ```rust
/// use std::ops::DerefMut;
/// use utopia_module::{UDb, redis::{Commands, Connection}};
///
/// let adbc: &UDb = &self.database.assume_init_ref();
///
/// // simple, high level operation
/// let val: isize = {
/// 	let key = format!("{}:some_db_key", self.id());
/// 	adbc.write()?.get(&key)?
/// }
/// println!("Value received from database operation: {}", val);
///
/// // more contrived example
/// {
/// 	let db: &Connection = adbc.write()?.deref_mut();
/// 	// do something complex with db, like redis::transaction or redis::pipe
/// }
/// ```
// TODO: wrap UDb in some kind of struct, that std::fmt::Debug can be
// implemented
pub type UDb = std::sync::Arc<std::sync::RwLock<redis::Connection>>;

/// The µtopia module definition.
///
/// See example from crate-level doc on for an implementation
pub trait Module: Any + Send + Sync {
	/// should return a reverse domain name that is unique to the
	/// module.
	fn id(&self) -> &'static str;

	/// should return the
	/// [ModuleInfo](utopia_common::module::ModuleInfo) struct that
	/// contains various information about the module, like it's human
	/// readable name, it's description and it's developer(s).
	fn get_module_info(&self) -> module::ModuleInfo;

	/// this is a blocking function that will be executed directly
	/// after the module was loaded. µCore will provide UDb as second
	/// parameter that the module may store if it wants to do database
	/// operations. You may use [MaybeUninit](std::mem::MaybeUninit)
	/// to store the Database connection.
	fn init(&mut self, _db: UDb) {}

	/// this is the function that will be executed directly before
	/// unloading the module.
	fn deinit(&self) {}

	/// this is the runtime function of a module. It is run in a
	/// sepeate tokio task and is not supposed to die. It is
	/// recommended to use [spawn_runtime!](crate::spawn_runtime) or
	/// [spawn_aync_runtime!](crate::spawn_async_runtime)
	/// in order take care of module-style return values. Inside the
	/// macro, you just need to return `Result<ThreadDeathExcuse,
	/// Box<std::error::Error>>`. See
	/// [ThreadDeathExcuse](utopia_common::module::ThreadDeathExcuse)
	/// for exceptions to let the thread function die.
	fn thread(&self, mod_send: USend, core_recv: URecv) -> URes;

	#[doc(hidden)]
	#[inline(always)]
	fn __abi_version(&self) -> &'static str {
		MODULE_INTERFACE_VERSION
	} // Prevent loading modules with newer/older ABI, to prevent segfaults.
}

/// spawns a synchronous runtime
///
/// it's likely you'll want an [asynronous
/// runtime](crate::spawn_async_runtime) instead, in order to
/// handle [USend](crate::USend) and [URecv](crate::URecv) properly,
/// as these channels are provided by [futures](futures) and require
/// .await in order to be handled. If however you want to spawn your
/// own, custom asnyc runtime, you can use this instead.
#[macro_export]
macro_rules! spawn_runtime {
	($id:expr, $function:stmt) => {
		return ($id, { $function });
	};
}

/// spawns an asynchronous runtime
///
/// to be placed inside the [thread function](crate::Module::thread)
/// of the [Module](crate::Module) trait.
///
/// It takes care of spawning a tokio runtime and returning
/// [URes](crate::URes) while you just need to return
/// `Result<ThreadDeathExcuse, Box<std::error::Error>>` inside the
/// macro wrapper.
///
/// ## Example
/// ```rust
/// use futures::future;
/// use utopia_module::{USend, URecv, URes, spawn_async_runtime};
/// use utopia_common::module::ThreadDeathExcuse;
///
/// fn thread(&self, _mod_send: USend, _core_recv: URecv) -> URes {
/// 	spawn_async_runtime!(self.id(), {
/// 		// do asynchronous task
/// 			let a = future::ready(1);
/// 			assert_eq!(a.await, 1);
/// 			Ok(ThreadDeathExcuse::Debug) // the actual thread function is not supposed to die
/// 	})
/// }
#[macro_export]
macro_rules! spawn_async_runtime {
	($id:expr, $function:stmt) => {
		let rt = utopia_module::Runtime::new().unwrap();
		return ($id, rt.block_on(async { $function }));
	};
}

/// Declare a module and its constructor
///
/// this macro creates a new c function called _module_create()
/// which returns a raw pointer to this Module invoking this macro
/// as created by the constructor. Consider implementing
/// [Default](std::default::Default) and using it's
/// [default() function](std::default::Default::default) like shown in
/// the example example.
///
/// ## Example
/// ```rust
/// use utopia_module::{Module, declare_module};
///
/// #[derive(Debug, Default)]
/// pub struct SampleMod;
///
/// impl Module for SampleMod {
/// 	[ ..snip.. ]
/// }
///
/// declare_module!(SampleMod, SampleMod::default);
/// ```
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
