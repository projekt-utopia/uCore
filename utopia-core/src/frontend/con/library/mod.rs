#![allow(dead_code)] // Allow unused stuff for now; TODO: Implement everything <3

mod age_rating;
mod artwork;
mod credits;
mod item_meta;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub enum LibraryItemKind {
    Game,
    App
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LibraryItemRunnerMethods {
    Run(String),
    Close(String),
    GetPid(String),
    Kill(String)
}

#[derive(Debug, Serialize)]
pub struct LibraryItemRunner {
    name: String,
    uuid: String,
    method: LibraryItemRunnerMethods
}

#[derive(Debug, Serialize)]
pub struct LibraryItemDetails {
    age_rating: age_rating::AgeRating,
    description: String,
    genre: Vec<item_meta::Genre>,
    game_modes: Vec<item_meta::GameModes>,
    credits: credits::Credits,
    controller_support: Vec<item_meta::InputType>
}

#[derive(Debug, Serialize)]
pub struct LibraryItem {
    uuid: String,
    name: String,
    kind: LibraryItemKind,
    default_runner: LibraryItemRunner,
    available_runners: Vec<LibraryItemRunner>,
    detais: LibraryItemDetails
}

#[derive(Debug, Deserialize)]
pub struct GameMethod {
    pub method: LibraryItemRunnerMethods
}
