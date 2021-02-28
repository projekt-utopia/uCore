pub use crate::library::{self, LibraryItemFrontend as LibraryItem};
use serde::{Serialize, Deserialize};

// Frontend --> Core
#[derive(Debug, Deserialize)]
pub enum FrontendActions {
    GetGameLibrary,
    GetGameDetails(String),
    GameMethod(library::LibraryItemProviderMethods)
}

#[derive(Debug, Deserialize)]
pub struct FrontendEvent {
    pub version: String,
    pub uuid: Option<String>,
    pub action: FrontendActions
}

// Core --> Frontend
#[derive(Debug, Serialize)]
pub enum CoreActions {
    SignalSuccessHandshake(String),
    ResponseGameLibrary(Vec<LibraryItem>),
    ResponseItemDetails(library::LibraryItemDetails),
    SignalGameLaunch(String),
    Error(String, String)
}

#[derive(Debug, Serialize)]
pub struct CoreEvent {
    pub version: String,
    pub uuid: Option<String>,
    pub action: CoreActions
}
impl CoreEvent {
    pub fn new(action: CoreActions, uuid: Option<String>) -> Self {
        CoreEvent {
            version: String::from("0.0.0"),
            uuid,
            action
        }
    }
}
