use bevy::prelude::*;

use crate::components::Square;

mod components;

// Constants for positioning
const TILE_SIZE: f32 = 100.0;
const BOARD_SIZE: f32 = TILE_SIZE * 8.0;
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 800;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Chess".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, spawn_board, spawn_pieces))
        .run();
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
        let is_white = (x + y) % 2 == 0;
        let color = if is_white {white.clone()} else {black.clone()};

        commands.spawn((
            Mesh2d(square),
            MeshMaterial2d(color),
            Transform::from_xyz(
                x as f32 * TILE_SIZE - (SCREEN_WIDTH as f32 / 2.0) + (TILE_SIZE / 2.0),
                y as f32 * TILE_SIZE - (SCREEN_HEIGHT as f32 / 2.0) + (TILE_SIZE / 2.0),
                0.0,
            ),
            Square {x: x as u8, y: y as u8}
        ));
    }
}

fn spawn_pieces(mut commands: Commands) {}