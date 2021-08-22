pub use crate::library::{self, LibraryItemFrontend, LibraryItemFrontendDetails, LibraryItemModule};
use serde::{Deserialize, Serialize};

// Frontend --> Core
#[derive(Debug, Deserialize, Serialize)]
pub enum FrontendActions {
	GetGameLibrary,
	GetFullGameLibrary,
	GetGameDetails(String),
	GameMethod(library::LibraryItemProviderMethods),
	RequestPreferenceDiag(String, library::preferences::DiagType),
	PreferenceDiagUpdate(
		(String, library::preferences::DiagType),
		std::collections::HashMap<String, library::preferences::FieldType>,
	),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FrontendEvent {
	pub version: String,
	pub uuid: Option<String>,
	pub action: FrontendActions,
}

// Core --> Frontend
#[derive(Debug, Serialize, Deserialize)]
pub enum CoreActions {
	SignalSuccessHandshake(String),
	ResponseGameLibrary(Vec<LibraryItemFrontend>),
	ResponseFullGameLibrary(Vec<LibraryItemFrontendDetails>),
	ResponseItemDetails(library::LibraryItemDetails),
	ResponseGameUpdate(LibraryItemFrontend),
	//SignalGameLaunch(String),
	PreferenceDiagResponse(
		(String, library::preferences::DiagType),
		library::preferences::PreferenceDiag,
	),
	Error(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreEvent {
	pub version: String,
	pub uuid: Option<String>,
	pub action: CoreActions,
}
impl CoreEvent {
	pub fn new(action: CoreActions, uuid: Option<String>) -> Self {
		CoreEvent {
			version: String::from("0.0.0"),
			uuid,
			action,
		}
	}
}
