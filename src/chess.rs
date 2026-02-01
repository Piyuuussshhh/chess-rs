// The only Bevy-less module in this project (yet).

use crate::components::{Piece, PieceColor, PieceKind, Square};

type Board<'a> = &'a [(Piece, Square)];

pub fn is_valid_move(piece: &Piece, start: (u8, u8), end: (u8, u8), board: Board) -> bool {
    if start == end {
        return false;
    }

    let potential_target_piece = board
        .iter()
        .find(|(_, square)| square.x == end.0 && square.y == end.1);
    // Cannot move to target square because path is blocked by same color piece.
    if let Some((target_piece, _)) = potential_target_piece {
        if target_piece.color == piece.color {
            return false;
        }
    }

    let dx = (end.0 as i8) - (start.0 as i8);
    let dy = (end.1 as i8) - (start.1 as i8);
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();

    match piece.kind {
        PieceKind::Pawn => is_valid_pawn_move(
            piece.color,
            start,
            end,
            potential_target_piece.is_some(),
            board,
        ),
        PieceKind::Rook => (dx == 0 || dy == 0) && is_path_clear(start, end, board),
        PieceKind::Knight => (abs_dx == 1 && abs_dy == 2) || (abs_dx == 2 && abs_dy == 1),
        PieceKind::Bishop => (abs_dx == abs_dy) && is_path_clear(start, end, board),
        PieceKind::Queen => {
            ((dx == 0 || dy == 0) || (abs_dx == abs_dy)) && is_path_clear(start, end, board)
        }
        PieceKind::King => abs_dx <= 1 && abs_dy <= 1,
    }
}

/// Returns true if the path between the start and end squares does not contain any other piece.
fn is_path_clear(start: (u8, u8), end: (u8, u8), board: Board) -> bool {
    let distance_x = (end.0 as i8) - (start.0 as i8);
    let distance_y = (end.1 as i8) - (start.1 as i8);

    let x_step = if distance_x == 0 {
        0
    } else {
        distance_x / distance_x.abs()
    };
    let y_step = if distance_y == 0 {
        0
    } else {
        distance_y / distance_y.abs()
    };

    let mut x = start.0 as i8 + x_step;
    let mut y = start.1 as i8 + y_step;

    let target_x = end.0 as i8;
    let target_y = end.1 as i8;

    while x != target_x || y != target_y {
        for (_, square) in board {
            if square.x == x as u8 && square.y == y as u8 {
                return false;
            }
        }

        x += x_step;
        y += y_step;
    }

    true
}

fn is_valid_pawn_move(
    color: PieceColor,
    start: (u8, u8),
    end: (u8, u8),
    is_target_occupied: bool,
    board: Board,
) -> bool {
    let dx = (end.0 as i8) - (start.0 as i8);
    let dy = (end.1 as i8) - (start.1 as i8);

    let direction = match color {
        PieceColor::White => 1i8,
        PieceColor::Black => -1i8,
    };

    let target_square_has_piece = board
        .iter()
        .any(|(_, square)| square.x == end.0 && square.y == end.1);

    // Forward move of the pawn by one square is valid only if there is no piece in front of it.
    if dx == 0 && dy == direction {
        return !target_square_has_piece;
    }

    // Initial two square move.
    let start_rank = match color {
        PieceColor::White => 1u8,
        PieceColor::Black => 6u8,
    };
    if dx == 0 && dy == 2 * direction && start.1 == start_rank {
        let is_blocked_by_another_piece = board.iter().any(|(_, square)| {
            square.x == (start.0 as i8 + direction) as u8
                && square.y == (start.1 as i8 + direction) as u8
        });
        return !is_blocked_by_another_piece && !target_square_has_piece;
    }

    // Diagonal Capture: dy == direction limits the capture to only happen diagonally forwards wrt the piece color.
    // The check for the enemy piece is already handled in input_system (if there is same color piece diagonal to a pawn, that piece will get selected instead of being captured)
    if dx.abs() == 1 && dy == direction {
        return is_target_occupied;
    }

    false
}

pub fn get_legal_moves(piece: &Piece, start: (u8, u8), board: Board) -> Vec<(u8, u8)> {
    let mut legal_moves = Vec::new();

    for row in 0..8 {
        for col in 0..8 {
            if is_valid_move(piece, start, (row, col), board) {
                legal_moves.push((row, col));
            }
        }
    }

    legal_moves
}
