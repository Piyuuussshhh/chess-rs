use crate::components::*;
use bevy::prelude::*;

// Constants for positioning
pub const TILE_SIZE: f32 = 100.0;
pub const BOARD_SIZE: f32 = TILE_SIZE * 8.0;
pub const OFFSET: f32 = BOARD_SIZE / 2.0;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_board, spawn_pieces));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Material (Square Paint)
    let white = materials.add(Color::srgb(0.9, 0.9, 0.8));
    let black = materials.add(Color::srgb(0.4, 0.4, 0.4));

    let mut board = Vec::new();

    for _ in 0..64 {
        board.push(meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)));
    }

    for (i, square) in board.into_iter().enumerate() {
        let x = i / 8;
        let y = i % 8;
        let is_black = (x + y) % 2 == 0;
        let color = if is_black {
            black.clone()
        } else {
            white.clone()
        };

        commands.spawn((
            Mesh2d(square),
            MeshMaterial2d(color),
            Transform::from_translation(get_world_position(y, x, 0.0)),
            Square {
                x: y as u8,
                y: x as u8,
            },
        ));
    }
}

fn spawn_pieces(mut commands: Commands, asset_server: Res<AssetServer>) {
    // initial setup -> (PieceType, Location)
    let white_last_rank = [
        (PieceKind::Rook, 0, 0),
        (PieceKind::Knight, 1, 0),
        (PieceKind::Bishop, 2, 0),
        (PieceKind::Queen, 3, 0),
        (PieceKind::King, 4, 0),
        (PieceKind::Bishop, 5, 0),
        (PieceKind::Knight, 6, 0),
        (PieceKind::Rook, 7, 0),
    ];

    let spawn_piece =
        |commands: &mut Commands, kind: PieceKind, color: PieceColor, x: u8, y: u8| {
            let color_str = match color {
                PieceColor::White => "white",
                PieceColor::Black => "black",
            };
            let kind_str = match kind {
                PieceKind::Pawn => "pawn",
                PieceKind::Rook => "rook",
                PieceKind::Knight => "knight",
                PieceKind::Bishop => "bishop",
                PieceKind::Queen => "queen",
                PieceKind::King => "king",
            };

            let path = format!("pieces/{color_str}-{kind_str}.png");

            commands.spawn((
                Sprite {
                    image: asset_server.load(path),
                    ..Default::default()
                },
                Transform {
                    translation: get_world_position(x as usize, y as usize, 1.0),
                    scale: Vec3::splat(0.8),
                    ..Default::default()
                },
                Piece { kind, color },
                Square {
                    x: x as u8,
                    y: y as u8,
                },
            ));
        };

    for (kind, x, y) in white_last_rank {
        spawn_piece(&mut commands, kind, PieceColor::White, x, y);
        spawn_piece(&mut commands, kind, PieceColor::Black, x, 7); // Mirror for black
    }

    // Spawn Pawns
    for i in 0..8 {
        spawn_piece(&mut commands, PieceKind::Pawn, PieceColor::White, i, 1);
        spawn_piece(&mut commands, PieceKind::Pawn, PieceColor::Black, i, 6);
    }
}

// Helper function to convert Grid Coordinates (0..8) to Pixel Coordinates (-400..400)
pub fn get_world_position(col: usize, row: usize, z: f32) -> Vec3 {
    Vec3::new(
        col as f32 * TILE_SIZE - OFFSET + TILE_SIZE / 2.0,
        row as f32 * TILE_SIZE - OFFSET + TILE_SIZE / 2.0,
        z,
    )
}
