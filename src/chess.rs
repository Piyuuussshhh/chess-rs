// The only Bevy-less module in this project (yet).

use crate::components::{Piece, PieceColor, PieceKind, Square};

type Board<'a> = &'a [(Piece, Square)];

pub fn is_legal_move(piece: &Piece, start: (u8, u8), end: (u8, u8), board: Board) -> bool {
    if !is_geometrically_valid_move(piece, start, end, board) {
        return false;
    }

    // Simulating the move to look for checks.

    let mut temp_board = board.to_vec();
    // Remove only the captured piece (if any).
    temp_board.retain(|(_, square)| square.x != end.0 || square.y != end.1);
    // Move the capturing (or moving) piece to the final location (end).
    if let Some((_, square)) = temp_board
        .iter_mut()
        .find(|(_, square)| square.x == start.0 && square.y == start.1)
    {
        square.x = end.0;
        square.y = end.1;
    }
    // Find our king's position.
    let king_position = if piece.kind == PieceKind::King {
        // If the piece that was moved is the king himself
        end
    } else {
        temp_board
            .iter()
            .find(|(p, _)| p.kind == PieceKind::King && p.color == piece.color)
            .map(|(_, s)| (s.x, s.y))
            .unwrap_or((0, 0))
    };

    // If our king is in check, then it's not a valid move.
    !is_king_in_check(king_position, piece.color, &temp_board)
}

pub fn get_legal_moves(piece: &Piece, start: (u8, u8), board: Board) -> Vec<(u8, u8)> {
    let mut legal_moves = Vec::new();

    for row in 0..8 {
        for col in 0..8 {
            if is_legal_move(piece, start, (row, col), board) {
                legal_moves.push((row, col));
            }
        }
    }

    legal_moves
}

fn is_geometrically_valid_move(
    piece: &Piece,
    start: (u8, u8),
    end: (u8, u8),
    board: Board,
) -> bool {
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
            board,
        ),
        PieceKind::Rook => (dx == 0 || dy == 0) && is_path_clear(start, end, board),
        PieceKind::Knight => (abs_dx == 1 && abs_dy == 2) || (abs_dx == 2 && abs_dy == 1),
        PieceKind::Bishop => (abs_dx == abs_dy) && is_path_clear(start, end, board),
        PieceKind::Queen => {
            ((dx == 0 || dy == 0) || (abs_dx == abs_dy)) && is_path_clear(start, end, board)
        }
        PieceKind::King => {
            if dx == 2 && dy == 0 {
                is_castling_possible(piece, CastleSide::KingSide, board)
            } else if dx == -2 && dy == 0 {
                is_castling_possible(piece, CastleSide::QueenSide, board)
            } else {
                abs_dx <= 1 && abs_dy <= 1
            }
        }
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
            square.x == start.0 && square.y == (start.1 as i8 + direction) as u8
        });
        return !is_blocked_by_another_piece && !target_square_has_piece;
    }

    // Diagonal Capture: dy == direction limits the capture to only happen diagonally forwards wrt the piece color.
    // The check for the enemy piece is already handled in input_system (if there is same color piece diagonal to a pawn, that piece will get selected instead of being captured)
    if dx.abs() == 1 && dy == direction {
        return target_square_has_piece;
    }

    false
}

pub fn is_king_in_check(king_position: (u8, u8), color: PieceColor, board: Board) -> bool {
    board
        .iter()
        .filter(|(enemy_piece, _)| enemy_piece.color != color)
        .any(|(enemy_piece, square)| {
            is_geometrically_valid_move(enemy_piece, (square.x, square.y), king_position, board)
        })
}

#[derive(Clone, Copy)]
enum CastleSide {
    KingSide,
    QueenSide,
}

impl CastleSide {
    fn get_rook_initial_pos(&self, color: PieceColor) -> (u8, u8) {
        match self {
            CastleSide::KingSide => match color {
                PieceColor::White => (7, 0),
                PieceColor::Black => (7, 7),
            },
            CastleSide::QueenSide => match color {
                PieceColor::White => (0, 0),
                PieceColor::Black => (0, 7),
            },
        }
    }

    fn squares_to_check(&self, color: PieceColor) -> [Option<(u8, u8)>; 4] {
        match self {
            CastleSide::KingSide => match color {
                PieceColor::White => [Some((4, 0)), Some((5, 0)), Some((6, 0)), None],
                PieceColor::Black => [Some((4, 7)), Some((5, 7)), Some((6, 7)), None],
            },
            CastleSide::QueenSide => match color {
                PieceColor::White => [Some((4, 0)), Some((3, 0)), Some((2, 0)), Some((1, 0))],
                PieceColor::Black => [Some((4, 7)), Some((3, 7)), Some((2, 7)), Some((1, 7))],
            },
        }
    }
}

fn is_castling_possible(king: &Piece, side: CastleSide, board: Board) -> bool {
    if king.has_moved {
        return false;
    }

    let checks = |side: CastleSide| -> bool {
        // To check if the rook has moved.
        if let Some((rook, _)) = board.iter().find(|(p, s)| {
            p.kind == PieceKind::Rook && (s.x, s.y) == side.get_rook_initial_pos(king.color)
        }) {
            if rook.has_moved {
                return false;
            }
        }

        // To check if the king or rook is blocked.
        let mut is_blocked = false;
        board.iter().for_each(|(_, sq)| {
            side.squares_to_check(king.color)
                .iter()
                // to skip the king's initial position.
                .skip(1)
                .for_each(|&s| {
                    if let Some((x, y)) = s {
                        // if there is a piece in between the rook and king, they are blocked.
                        if sq.x == x && sq.y == y {
                            is_blocked = true;
                        }
                    }
                });
        });
        if is_blocked {
            return false;
        }

        // To check if the king passes through a check.
        let possible_king_positions = side.squares_to_check(king.color);
        for possible_king_position in possible_king_positions {
            if possible_king_position == None {
                continue;
            }
            // the squares b1 and b8 need not be checked for king checks.
            if [(1, 0), (1, 7)].contains(&possible_king_position.unwrap()) {
                continue;
            }
            if is_king_in_check(possible_king_position.unwrap(), king.color, board) {
                return false;
            }
        }

        true
    };

    // Checking if the king passes through a check or if the king or the rook has moved or is blocked while castling.
    let castling_checks_passed = match side {
        CastleSide::KingSide => match king.color {
            PieceColor::White => checks(side),
            PieceColor::Black => checks(side),
        },
        CastleSide::QueenSide => match king.color {
            PieceColor::White => checks(side),
            PieceColor::Black => checks(side),
        },
    };

    if !castling_checks_passed {
        return false;
    }

    let rook_start_pos = side.get_rook_initial_pos(king.color);

    let is_rook_at_start_pos = board
        .iter()
        .any(|(p, s)| p.kind == PieceKind::Rook && (s.x, s.y) == rook_start_pos);

    is_rook_at_start_pos
}
