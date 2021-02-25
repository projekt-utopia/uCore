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
    pub action: FrontendActions
}

// Core --> Frontend
#[derive(Debug, Serialize)]
pub enum CoreActions {
    SignalSuccessHandshake(String),
    ResponseGameLibrary(Vec<LibraryItem>),
    ResponseItemDetails(library::LibraryItemDetails),
    SignalGameLaunch(String)
}

#[derive(Debug, Serialize)]
pub struct CoreEvent {
    pub version: String,
    pub action: CoreActions
}
impl CoreEvent {
    pub fn new(action: CoreActions) -> Self {
        CoreEvent {
            version: String::from("0.0.0"),
            action
        }
    }
}
