use bevy::prelude::*;

use crate::{
    board::{BoardPlugin, SCREEN_HEIGHT, SCREEN_WIDTH},
    systems::GamePlugin,
};

mod board;
mod components;
mod systems;

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
        .add_plugins(BoardPlugin)
        .add_plugins(GamePlugin)
        .run();
}
