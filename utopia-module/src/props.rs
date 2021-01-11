#[derive(Debug)]
pub enum CoreCommands {
    Reload,
    LaunchGame(String),
    SendChatMsg
}

#[derive(Debug)]
pub enum ModuleCommands {
    Refresh,
    Chat,
    Notification,
    AddGame
}

#[derive(Debug)]
pub enum ThreadDeathExcuse {
    HiracyDeath,
    Debug,
    Other(String)
}

pub struct ModuleInfo {
    pub name: String,
    pub url: Option<String>,
    pub developer: String,
    pub developer_url: Option<String>,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>
}
