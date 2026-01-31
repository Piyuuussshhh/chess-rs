use bevy::{prelude::*, window::WindowMode};

use crate::{
    board::BoardPlugin,
    resources::GameState,
    systems::GamePlugin,
};

mod board;
mod components;
mod resources;
mod systems;

fn main() {
    App::new()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chess".into(),
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoardPlugin)
        .add_plugins(GamePlugin)
        .run();
}
