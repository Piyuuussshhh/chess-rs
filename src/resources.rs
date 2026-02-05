use bevy::prelude::*;
use crate::components::PieceColor;

#[derive(Resource)]
pub struct GameState {
    pub turn: PieceColor,
    pub en_passant_target: Option<(u8, u8)>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            turn: PieceColor::White,
            en_passant_target: None,
        }
    }
}