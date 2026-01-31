use std::fmt::Display;

use bevy::prelude::*;

#[derive(Component)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceKind,
}

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Component)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

// This component will be attached to a single piece that is clicked on by the user.
#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct SelectedFilter;