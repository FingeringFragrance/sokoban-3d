use bevy::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    ModeSelect,
    Settings,
    Loading,
    Playing,
    Paused,
    LevelComplete,
}
