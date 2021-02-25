pub mod age_rating;
pub mod artwork;
pub mod credits;
pub mod item_meta;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LibraryItemKind {
    Game,
    App
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemDetails {
    pub age_rating: age_rating::AgeRating,
    pub description: String,
    pub genre: Vec<item_meta::Genre>,
    pub game_modes: Vec<item_meta::GameModes>,
    pub credits: credits::Credits,
    pub controller_support: Vec<item_meta::InputType>
}

#[derive(Debug)]
pub struct LibraryItemModule {
    pub uuid: String,
    pub name: String,
    pub kind: LibraryItemKind,
    pub details: LibraryItemDetails
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryItemFrontend {
    pub uuid: String,
    pub name: String,
    pub kind: LibraryItemKind,
    // (uuid, title)
    pub active_provider: (String, String),
    pub providers: HashMap<String, String>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LibraryItemProviderMethods {
    Run(String),
    // uuid of game, uuid of provider
    RunProvider(String, String),
    // uuid of game, uuid of provider
    ChangeDefaultProvider(String, String),
    Close(String),
    GetPid(String),
    Kill(String)
}
