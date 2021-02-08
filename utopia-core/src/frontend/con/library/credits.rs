use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Credits {
    developer: String,
    publisher: Option<String>,
    director: Option<String>,
    other: std::collections::HashMap<String, String>
}
