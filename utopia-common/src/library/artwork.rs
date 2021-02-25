use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtworkType {
    SquareCover,
    CaseCover,
    SteamCover, // please elaborate
    Logo,
    LandscapeCover,
    Background,
    Misc(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artwork {
    pub uuid: String,
    pub r#type: ArtworkType,
    pub data: Vec<u8>,
    pub mime: String,
    pub uri: Option<String>
}
