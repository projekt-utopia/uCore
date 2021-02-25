use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputType {
    Keyboard,
    Xbox360,
    XboxOne,
    Switch,
    Ps3,
    Ps4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameModes {
    Other(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Genre {
    PointAndClick,
    Fighting,
    Shooter,
    Music,
    Platform,
    Puzzle,
    Racing,
    Rts,
    Rpg,
    Simulator,
    Sport,
    Strategy,
    TurnBased,
    Tactical,
    Quit,
    Hacknslash,
    Pinball,
    Adventure,
    Arcrade,
    VisualNovel,
    Indie,
    CardBoardGame,
    Moba,
    Other(String)
}
