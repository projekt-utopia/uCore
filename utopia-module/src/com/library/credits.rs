#[derive(Debug)]
pub struct Credits {
    pub developer: String,
    pub publisher: Option<String>,
    pub director: Option<String>,
    pub other: std::collections::HashMap<String, String>
}
