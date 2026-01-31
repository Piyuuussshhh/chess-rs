use bevy::prelude::*;
use crate::components::PieceColor;

#[derive(Resource)]
pub struct GameState {
    pub turn: PieceColor,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            turn: PieceColor::White,
        }
    }
}