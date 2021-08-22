pub use crate::library::{self, LibraryItemModule as LibraryItem};

/// Signals that the module sends to the core to notify about progress
#[derive(Debug)]
pub enum LibraryItemStatusSignals {
	/// A LibaryItem has launched (uuid, pid)
	Launched(String, u32),
	// (uuid)
	Closed(String),
	// (uuid)
	Crashed(String)
}

// Core --> Module
#[derive(Debug)]
pub enum CoreCommands {
	Reload,
	LaunchLibraryItem(String),
	RequestPreferenceDiag(library::preferences::DiagType),
	PreferenceDiagUpdate(
		library::preferences::DiagType,
		std::collections::HashMap<String, library::preferences::FieldType>
	)
}

// Module --> Core
#[derive(Debug)]
pub enum ModuleCommands {
	Refresh,
	AddLibraryItem(LibraryItem),
	AddLibraryItemBulk(Vec<LibraryItem>),
	ItemStatusSignal(LibraryItemStatusSignals),
	PreferenceDiagResponse(library::preferences::DiagType, library::preferences::PreferenceDiag)
}

/// The return value of the module thread
#[derive(Debug)]
pub enum ThreadDeathExcuse {
	/// If a runtime dependency dies and the module is unable to
	/// restart it
	HiracyDeath,
	/// Should be self-explaining. We will not accept any MR that
	/// actually throws this
	Debug,
	Other(String)
}

/// Human readable information about a module. Returned by the
/// get_module_info function (outside of the thread)
pub struct ModuleInfo {
	pub name: String,
	pub url: Option<String>,
	pub developer: String,
	pub developer_url: Option<String>,
	pub description: Option<String>,
	pub icon: Option<String>
}
