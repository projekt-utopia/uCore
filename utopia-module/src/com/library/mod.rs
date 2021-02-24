pub mod age_rating;
pub mod artwork;
pub mod credits;
pub mod item_meta;

#[derive(Debug, Clone, Copy)]
pub enum LibraryItemKind {
    Game,
    App
}

#[derive(Debug)]
pub struct LibraryItemDetails {
    pub age_rating: age_rating::AgeRating,
    pub description: String,
    pub genre: Vec<item_meta::Genre>,
    pub game_modes: Vec<item_meta::GameModes>,
    pub credits: credits::Credits,
    pub controller_support: Vec<item_meta::InputType>
}

#[derive(Debug)]
pub struct LibraryItem {
    pub uuid: String,
    pub name: String,
    pub kind: LibraryItemKind,
    pub details: LibraryItemDetails
}
