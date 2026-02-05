use std::fmt::Display;

use bevy::prelude::*;

#[derive(Component,Clone, Copy, Debug)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceKind,
    pub has_moved: bool,
    pub start_pos: (u8, u8),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceColor {
    White,
    Black
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            PieceColor::White => "White",
            PieceColor::Black => "Black",
        };
        write!(f, "{}", color)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            PieceKind::Pawn => "Pawn",
            PieceKind::Rook => "Rook",
            PieceKind::Knight => "Knight",
            PieceKind::Bishop => "Bishop",
            PieceKind::Queen => "Queen",
            PieceKind::King => "King",
        };
        write!(f, "{}", kind)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// This component will be attached to a single piece that is clicked on by the user.
#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct SelectedFilter;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct MovedFilter;

#[derive(Component)]
pub struct LegalMovesFilter;

#[derive(Component)]
pub struct InCheckHighlight;