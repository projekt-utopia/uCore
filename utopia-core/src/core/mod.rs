//use utopia_module::com::{library, CoreCommands};
use std::collections::HashMap;

use utopia_common::{library, module::CoreCommands};
use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

use crate::{errors::{LibraryItemNotAvailableError, ModuleNotAvailableError, ProvModuleNotAvailableError},
            modules::modules::ModuleManager};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdStateAction {
	Add,
	Remove
}

#[derive(Clone)]
pub struct ItemProvider {
	pub title: String,
	pub icon: Option<String>,
	pub status: Vec<library::LibraryItemStatus>,
	module: &'static str
}

pub struct LibraryItem {
	uuid: String,
	name: String,
	kind: library::LibraryItemKind,
	pub details: library::LibraryItemDetails,
	active_provider: (String, ItemProvider),
	providers: HashMap<String, ItemProvider>
}
impl LibraryItem {
	pub fn new(provider: &'static str, title: String, icon: Option<String>, item: library::LibraryItemModule) -> Self {
		let iprovider = ItemProvider {
			title,
			icon,
			status: item.status,
			module: provider
		};
		let mut providers = HashMap::new();
		providers.insert(String::from(provider), iprovider.clone());
		LibraryItem {
			uuid: item.uuid,
			name: item.name,
			kind: item.kind,
			details: item.details,
			active_provider: (String::from(provider), iprovider),
			providers
		}
	}

	pub fn add_provider(
		&mut self,
		provider: &'static str,
		title: String,
		icon: Option<String>,
		status: Vec<library::LibraryItemStatus>
	) {
		self.providers.insert(String::from(provider), ItemProvider {
			title,
			icon,
			status,
			module: provider
		});
	}

	pub fn run_default(&self, mod_mgr: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
		mod_mgr
			.get(&self.active_provider.1.module)?
			.send(CoreCommands::LaunchLibraryItem(self.uuid.clone()))?;
		Ok(())
	}

	pub fn run_provider(&self, mod_mgr: &ModuleManager, provider: String) -> Result<(), Box<dyn std::error::Error>> {
		match self.providers.get(&provider) {
			Some(provider) => {
				mod_mgr
					.get(provider.module)?
					.send(CoreCommands::LaunchLibraryItem(self.uuid.clone()))?;
				Ok(())
			},
			None => Err(Box::new(ProvModuleNotAvailableError::new(provider)))
		}
	}

	pub fn change_default_provider(&mut self, provider: String) -> Result<(), ProvModuleNotAvailableError> {
		let title = self
			.providers
			.get(&provider)
			.ok_or(ProvModuleNotAvailableError::new(provider.clone()))?;
		self.active_provider = (provider, title.clone());
		Ok(())
	}

	pub fn update_state(
		&mut self,
		provider: String,
		action: UpdStateAction,
		state: library::LibraryItemStatus
	) -> Result<(), ProvModuleNotAvailableError> {
		let mut prov = self
			.providers
			.get_mut(&provider)
			.ok_or(ProvModuleNotAvailableError::new(provider.clone()))?;
		if action == UpdStateAction::Remove {
			prov.status = prov.status.clone().into_iter().filter(|&i| i != state).collect();
		} else if action == UpdStateAction::Add {
			if !prov.status.iter().any(|&i| i == state) {
				prov.status.push(state)
			}
		}
		if self.active_provider.0 == provider {
			self.active_provider.1 = prov.clone();
		}
		Ok(())
	}

	pub fn to_frontend(&self) -> library::LibraryItemFrontend {
		library::LibraryItemFrontend {
			uuid: self.uuid.clone(),
			name: self.name.clone(),
			kind: self.kind,
			active_provider: library::LibraryProvider::new(
				self.active_provider.0.clone(),
				self.active_provider.1.title.clone(),
				self.active_provider.1.icon.clone(),
				self.active_provider.1.status.clone()
			),
			providers: self
				.providers
				.iter()
				.map(|(k, v)| {
					(
						k.to_owned(),
						(library::LibraryProvider::new(
							k.to_owned(),
							v.title.clone(),
							v.icon.clone(),
							v.status.clone()
						))
					)
				})
				.collect()
		}
	}

	pub fn to_full(&self) -> library::LibraryItemFrontendDetails {
		library::LibraryItemFrontendDetails {
			uuid: self.uuid.clone(),
			name: self.name.clone(),
			kind: self.kind,
			details: self.details.clone(),
			active_provider: library::LibraryProvider::new(
				self.active_provider.0.clone(),
				self.active_provider.1.title.clone(),
				self.active_provider.1.icon.clone(),
				self.active_provider.1.status.clone()
			),
			providers: self
				.providers
				.iter()
				.map(|(k, v)| {
					(
						k.to_owned(),
						(library::LibraryProvider::new(
							k.to_owned(),
							v.title.clone(),
							v.icon.clone(),
							v.status.clone()
						))
					)
				})
				.collect()
		}
	}
}

pub struct Library {
	inner: HashMap<String, LibraryItem>
}
impl Library {
	pub fn new() -> Self {
		Library {
			inner: HashMap::new()
		}
	}

	pub fn insert(
		&mut self,
		module: &'static str,
		item: library::LibraryItemModule,
		mod_mgr: &ModuleManager
	) -> Result<(), ModuleNotAvailableError> {
		let modinfo = mod_mgr.get(&module)?.module.get_module_info();
		let status = item.status.clone();
		println!("Added {}", item.uuid.clone());
		self.inner
			.entry(item.uuid.clone())
			.and_modify(|item| item.add_provider(module, modinfo.name.clone(), modinfo.icon.clone(), status))
			.or_insert(LibraryItem::new(
				module,
				modinfo.name.clone(),
				modinfo.icon.clone(),
				item
			));
		Ok(())
	}

	pub fn bulk_insert(
		&mut self,
		module: &'static str,
		items: Vec<library::LibraryItemModule>,
		mod_mgr: &ModuleManager
	) -> Result<(), ModuleNotAvailableError> {
		for item in items {
			self.insert(module, item, mod_mgr)?;
		}
		Ok(())
	}

	pub fn get(&self, uuid: &String) -> Result<&LibraryItem, LibraryItemNotAvailableError> {
		match self.inner.get(uuid) {
			Some(item) => Ok(item),
			None => Err(LibraryItemNotAvailableError::new(uuid))
		}
	}

	pub fn get_mut(&mut self, uuid: &String) -> Result<&mut LibraryItem, LibraryItemNotAvailableError> {
		match self.inner.get_mut(uuid) {
			Some(item) => Ok(item),
			None => Err(LibraryItemNotAvailableError::new(uuid))
		}
	}

	pub fn launch_library_item(
		&self,
		uuid: &String,
		mod_mgr: &ModuleManager
	) -> Result<(), Box<dyn std::error::Error>> {
		self.get(uuid)?.run_default(mod_mgr)?;
		Ok(())
	}

	pub fn launch_library_item_from_provider(
		&self,
		uuid: &String,
		mod_mgr: &ModuleManager,
		provider: String
	) -> Result<(), Box<dyn std::error::Error>> {
		self.get(uuid)?.run_provider(mod_mgr, provider)?;
		Ok(())
	}

	pub fn change_default_provider(
		&mut self,
		uuid: &String,
		provider: String
	) -> Result<(), Box<dyn std::error::Error>> {
		self.get_mut(uuid)?.change_default_provider(provider)?;
		Ok(())
	}

	pub fn to_frontend(&self) -> Vec<library::LibraryItemFrontend> {
		self.inner.values().map(|item| item.to_frontend()).collect()
	}

	pub fn to_full(&self) -> Vec<library::LibraryItemFrontendDetails> {
		self.inner.values().map(|item| item.to_full()).collect()
	}
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InternalCoreFutures {
	NewFrontendRegistered(String, tokio::net::UnixStream),
	ProcessDied(u32, i32 /* old pid, POSIX return code */),
	DatabaseProcessDied(std::io::Result<std::process::ExitStatus>),
	Debug,
	Error(Box<dyn std::error::Error + Send>)
}

pub struct Core {
	pub library: Library,
	pub internal_futures: FuturesUnordered<JoinHandle<InternalCoreFutures>>,
	pub running: std::collections::HashMap<u32, (&'static str, String)>, /* pid, (module, uuid) */
	// <(module uuid, pot. item uuid), (frontend uuid, msg resp uuid)>
	pub open_preferences:
		std::collections::HashMap<(String, utopia_common::library::preferences::DiagType), (String, Option<String>)>
}
impl Core {
	pub fn new() -> Self {
		Core {
			library: Library::new(),
			internal_futures: FuturesUnordered::new(),
			running: std::collections::HashMap::new(),
			open_preferences: std::collections::HashMap::new()
		}
	}
}
