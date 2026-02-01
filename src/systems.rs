use crate::{
    board::{OFFSET, TILE_SIZE, get_world_position},
    chess::is_valid_move,
    components::{MovedFilter, Piece, PieceColor, PieceKind, Selected, SelectedFilter, Square},
    events::MoveMadeEvent,
    resources::GameState,
};
use bevy::{prelude::*, window::PrimaryWindow};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                input_system,
                highlight_selected_piece_system.after(input_system),
                piece_movement_system,
            ),
        )
        .add_observer(on_move_made);
    }
}

fn input_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut piece_query: Query<(Entity, &Piece, &mut Square)>,
    selected_piece_query: Query<Entity, With<Selected>>,
    mut game_state: ResMut<GameState>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    else {
        return;
    };

    let x = ((world_position.x + OFFSET) / TILE_SIZE).floor() as u8;
    let y = ((world_position.y + OFFSET) / TILE_SIZE).floor() as u8;

    if !(0..8).contains(&x) || !(0..8).contains(&y) {
        return;
    }

    // Finding the piece we just clicked (if any).
    /*
        If clicked_piece = None, the player clicked a square with no piece on it.
        This could be the target square where the player wants a piece to go.
    */
    let mut clicked_piece: Option<(Entity, PieceKind, PieceColor)> = None;
    for (entity, piece, square) in piece_query.iter() {
        if square.x == x && square.y == y {
            clicked_piece = Some((entity, piece.kind, piece.color));
            break;
        }
    }

    // Finding the currently selected piece (it could be some other piece than the one found above).
    let mut selected_piece: Option<(Entity, PieceKind, PieceColor)> = None;
    if let Ok(selected_entity) = selected_piece_query.single() {
        if let Ok((_, piece, _)) = piece_query.get(selected_entity) {
            selected_piece = Some((selected_entity, piece.kind, piece.color))
        }
    }

    // Taking a snapshot of the board and saving it in an array.
    let board: Vec<(Piece, Square)> = piece_query.iter().map(|(_, p, s)| (*p, *s)).collect();

    // Decision & Execution
    match (selected_piece, clicked_piece) {
        // Case 1: Nothing selected yet -> player clicks a piece => Select the piece ONLY if its one of the player's pieces.
        (None, Some((entity, _, piece_color))) => {
            if piece_color != game_state.turn {
                return;
            }

            commands.entity(entity).insert(Selected);
        }
        // Case 2: A piece is selected -> player clicks an empty square => Move the piece to the empty square.
        (Some((entity, selected_piece_kind, selected_piece_color)), None) => {
            if selected_piece_color != game_state.turn {
                return;
            }

            let piece = Piece {
                kind: selected_piece_kind,
                color: selected_piece_color,
            };

            if let Ok((_, _, mut square)) = piece_query.get_mut(entity) {
                let (prev_x, prev_y) = (square.x, square.y);

                if !is_valid_move(&piece, (prev_x, prev_y), (x, y), &board) {
                    return;
                }

                square.x = x;
                square.y = y;

                game_state.turn = match game_state.turn {
                    PieceColor::White => PieceColor::Black,
                    PieceColor::Black => PieceColor::White,
                };

                commands.trigger(MoveMadeEvent {
                    piece: entity,
                    start: (prev_x, prev_y),
                    end: (x, y),
                });

                // Deselect after moving it.
                commands.entity(entity).remove::<Selected>();
            }
        }
        // Case 3: A piece is selected -> player clicks on a square with a piece => 3 possibilities.
        (
            Some((
                currently_selected_entity,
                currently_selected_piece_kind,
                currently_selected_piece_color,
            )),
            Some((target_entity, target_piece_kind, target_piece_color)),
        ) => {
            // If currently selected piece is not one of the player's pieces, do nothing.
            if currently_selected_piece_color != game_state.turn {
                return;
            }

            // Possibility 1: The player clicked on the same piece => Deselect the piece.
            if currently_selected_entity == target_entity {
                commands
                    .entity(currently_selected_entity)
                    .remove::<Selected>();
            }
            // Possibility 2: The player clicked on another piece from their own pieces => Deselect the currently selected piece and select the new piece.
            else if currently_selected_piece_color == target_piece_color {
                commands
                    .entity(currently_selected_entity)
                    .remove::<Selected>();
                commands.entity(target_entity).insert(Selected);
            }
            // Possibility 3: the player clicked on an enemy piece => Despawn (capture) the enemy piece, move the currently selected piece to the enemy piece's position then deselect it
            else {
                let currently_selected_piece = Piece {
                    kind: currently_selected_piece_kind,
                    color: currently_selected_piece_color,
                };

                if let Ok((_, _, square)) = piece_query.get(currently_selected_entity) {
                    if !is_valid_move(
                        &currently_selected_piece,
                        (square.x, square.y),
                        (x, y),
                        &board,
                    ) {
                        return;
                    }
                }

                commands.entity(target_entity).despawn();

                if let Ok((_, _, mut square)) = piece_query.get_mut(currently_selected_entity) {
                    let (prev_x, prev_y) = (square.x, square.y);
                    square.x = x;
                    square.y = y;

                    // Turn completed, switch turns.
                    game_state.turn = match game_state.turn {
                        PieceColor::White => PieceColor::Black,
                        PieceColor::Black => PieceColor::White,
                    };
                    // Send the move made event.
                    commands.trigger(MoveMadeEvent {
                        piece: currently_selected_entity,
                        start: (prev_x, prev_y),
                        end: (x, y),
                    });
                }
                commands
                    .entity(currently_selected_entity)
                    .remove::<Selected>();
            }
        }
        // TODO Case 4: Something is selected or not it doesn't matter -> Player clicked somewhere outside the board => Maybe clicked the UI, check.
        _ => {}
    }
}

fn highlight_selected_piece_system(
    mut commands: Commands,
    just_selected_square_query: Query<&Square, Added<Selected>>,
    previously_selected_square_query: Query<Entity, With<SelectedFilter>>,
    any_selected_square_query: Query<&Selected>,
) {
    // If the player just selected a square with a piece on it.
    if !just_selected_square_query.is_empty() {
        // Deleting old selected filter.
        for entity in previously_selected_square_query.iter() {
            commands.entity(entity).despawn();
        }

        for square in just_selected_square_query.iter() {
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.6, 0.1, 0.8, 0.5),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
                Transform::from_translation(get_world_position(
                    square.x as usize,
                    square.y as usize,
                    0.5,
                )),
                SelectedFilter,
            ));
        }
    }
    // If the player clicks outside the board or on a square with no piece on it.
    else if any_selected_square_query.is_empty() && !previously_selected_square_query.is_empty() {
        for entity in previously_selected_square_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn on_move_made(
    event: On<MoveMadeEvent>,
    moved_piece_query: Query<&Square, (With<Piece>, Changed<Square>)>,
    previously_moved_piece_query: Query<Entity, With<MovedFilter>>,
    mut commands: Commands,
) {
    // Remove the filter from the previous last move.
    for entity in previously_moved_piece_query.iter() {
        commands.entity(entity).despawn();
    }

    let (moved_entity, previous_position, new_position) = (event.piece, event.start, event.end);
    if let Ok(_) = moved_piece_query.get(moved_entity) {
        // Lighter shade for the start square.
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.89, 0.118, 0.61),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
            Transform::from_translation(get_world_position(
                previous_position.0 as usize,
                previous_position.1 as usize,
                0.5,
            )),
            MovedFilter,
        ));

        // Darker shade for the final square.
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.89, 0.118, 0.61),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
            Transform::from_translation(get_world_position(
                new_position.0 as usize,
                new_position.1 as usize,
                0.5,
            )),
            MovedFilter,
        ));
    }
}

fn piece_movement_system(
    mut movement_query: Query<(&Square, &mut Transform), (With<Piece>, Changed<Square>)>,
) {
    for (square, mut transform) in movement_query.iter_mut() {
        transform.translation = get_world_position(square.x as usize, square.y as usize, 1.0);
    }
}
