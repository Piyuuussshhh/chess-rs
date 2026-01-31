use bevy::prelude::*;

use crate::{
    components::{PieceColor, TurnText},
    resources::GameState,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_turn_text);
    }
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((Node {
            width: Val::Percent(40.0),
            height: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Px(1200.0),
            ..default()
        },))
        .with_children(|parent| {
            // Spawn text as a child of the container
            parent
                .spawn((
                    Text::new("White To Play"),
                    TextFont {
                        font_size: 50.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ))
                .insert(TurnText);
        });
}

fn update_turn_text(game_state: Res<GameState>, mut text_query: Query<&mut Text, With<TurnText>>) {
    for mut text in text_query.iter_mut() {
        let turn_str = match game_state.turn {
            PieceColor::White => "White To Play",
            PieceColor::Black => "Black To Play",
        };
        **text = turn_str.to_string();
    }
}
