pub mod age_rating;
pub mod artwork;
pub mod credits;
pub mod item_meta;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LibraryItemKind {
	Game,
	App,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LibraryItemStatus {
	Running(Option<u32>),
	Closing,
	Updatable,
	Updating,
	Installed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemDetails {
	pub age_rating: age_rating::AgeRating,
	pub artworks: Vec<artwork::Artwork>,
	pub description: String,
	pub genre: Vec<item_meta::Genre>,
	pub game_modes: Vec<item_meta::GameModes>,
	pub credits: credits::Credits,
	pub controller_support: Vec<item_meta::InputType>,
}

#[derive(Debug)]
pub struct LibraryItemModule {
	pub uuid: String,
	pub name: String,
	pub kind: LibraryItemKind,
	pub details: LibraryItemDetails,
	pub status: Vec<LibraryItemStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemFrontend {
	pub uuid: String,
	pub name: String,
	pub kind: LibraryItemKind,
	// (uuid, title, stati)
	pub active_provider: LibraryProvider,
	pub providers: HashMap<String, LibraryProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryProvider {
	pub uuid: String,
	pub name: String,
	pub icon: Option<String>,
	pub stati: Vec<LibraryItemStatus>,
}

impl LibraryProvider {
	pub fn new(uuid: String, name: String, icon: Option<String>, stati: Vec<LibraryItemStatus>) -> Self {
		Self {
			uuid,
			name,
			icon,
			stati,
		}
	}
}

impl PartialEq for LibraryProvider {
	fn eq(&self, other: &Self) -> bool {
		self.uuid == other.uuid
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemFrontendDetails {
	pub uuid: String,
	pub name: String,
	pub kind: LibraryItemKind,
	pub details: LibraryItemDetails,
	pub active_provider: LibraryProvider,
	pub providers: HashMap<String, LibraryProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LibraryItemProviderMethods {
	Launch(String),
	// uuid of game, uuid of provider
	LaunchViaProvider(String, String),
	// uuid of game, uuid of provider
	ChangeSelectedProvider(String, String),
	Close(String),
	GetPid(String),
	Kill(String),
	Update(String),
	Uninstall(String),
}
