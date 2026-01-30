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

#[derive(Clone, Copy, PartialEq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

#[derive(Component)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}