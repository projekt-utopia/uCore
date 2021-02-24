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
    // uuid of game, uuid of provider
    RunRunner(String, String),
    // uuid of game, uuid of provider
    ChangeDefaultRunner(String, String),
    Close(String),
    GetPid(String),
    Kill(String)
}

#[derive(Debug, Serialize)]
pub struct LibraryItemRunner {
    pub name: String,
    pub uuid: String
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
    pub uuid: String,
    pub name: String,
    pub kind: LibraryItemKind,
    pub default_runner: LibraryItemRunner,
    pub available_runners: Vec<LibraryItemRunner>,
    //detais: LibraryItemDetails
}

#[derive(Debug, Deserialize)]
pub struct GameMethod {
    pub method: LibraryItemRunnerMethods
}
