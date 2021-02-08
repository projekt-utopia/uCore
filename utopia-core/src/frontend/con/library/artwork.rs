use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ArtworkType {
    SquareCover,
    CaseCover,
    SteamCover, // please elaborate
    Logo,
    LandscapeCover,
    Background,
    Misc(String)
}

#[derive(Debug, Serialize)]
pub struct Artwork {
    uuid: String,
    r#type: ArtworkType,
    data: Vec<u8>,
    mime: String,
    uri: Option<String>
}
