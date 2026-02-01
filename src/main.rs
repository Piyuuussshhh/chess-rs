use bevy::{prelude::*, window::WindowMode};

use crate::{board::BoardPlugin, resources::GameState, systems::GamePlugin, ui::UIPlugin};

mod board;
mod components;
mod resources;
mod systems;
mod ui;
mod events;
mod chess;

fn main() {
    App::new()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chess".into(),
                mode: WindowMode::Windowed,
                resolution: (1600, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UIPlugin)
        .add_plugins(BoardPlugin)
        .add_plugins(GamePlugin)
        .run();
}
