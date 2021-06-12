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
pub enum ArtworkData {
    Data(Vec<u8>, bool, i32, i32, i32, i32 /* data, has_alpha, bits_per_sample, width, height, rowstride */),
    Uri(String),
    Path(std::path::PathBuf)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artwork {
    pub uuid: String,
    pub r#type: ArtworkType,
    pub mime: String,
    pub data: ArtworkData,
}
