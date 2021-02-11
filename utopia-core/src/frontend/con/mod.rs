pub mod library;
use serde::{Serialize, Deserialize};

// Frontend --> Core
#[derive(Debug, Deserialize)]
pub enum FrontendActions {
    GetGameLibrary,
    GetGameDetails(String),
    GameMethod(library::GameMethod)
}

#[derive(Debug, Deserialize)]
pub struct FrontendEvent {
    pub version: String,
    pub action: FrontendActions
}

// Core --> Frontend
#[derive(Debug, Serialize)]
pub enum CoreActions {
    ResponseGameLibrary(Vec<library::LibraryItem>),
    SignalGameLaunch(String)
}

#[derive(Debug, Serialize)]
pub struct CoreEvent {
    pub version: String,
    pub action: CoreActions
}
