use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum InputType {
    Keyboard,
    Xbox360,
    XboxOne,
    Switch,
    Ps3,
    Ps4
}

#[derive(Debug, Serialize)]
pub enum GameModes {
    Other(String)
}

#[derive(Debug, Serialize)]
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
