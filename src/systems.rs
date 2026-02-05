use crate::{
    board::{OFFSET, TILE_SIZE, get_world_position},
    chess::{get_legal_moves, is_king_in_check, is_legal_move},
    components::{
        InCheckHighlight, LegalMovesFilter, MovedFilter, Piece, PieceColor, PieceKind, Selected,
        SelectedFilter, Square,
    },
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
                highlight_legal_moves_system,
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
    mut piece_query: Query<(Entity, &mut Piece, &mut Square)>,
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
    let mut clicked_piece: Option<(Entity, Piece)> = None;
    for (entity, piece, square) in piece_query.iter() {
        if square.x == x && square.y == y {
            clicked_piece = Some((entity, piece.clone()));
            break;
        }
    }

    // Finding a selected piece.
    let mut selected_piece: Option<(Entity, Piece)> = None;
    if let Ok(selected_entity) = selected_piece_query.single() {
        if let Ok((_, piece, _)) = piece_query.get(selected_entity) {
            selected_piece = Some((selected_entity, piece.clone()))
        }
    }

    // Saving the board.
    let board: Vec<(Piece, Square)> = piece_query.iter().map(|(_, p, s)| (*p, *s)).collect();

    // Decision Tree + Execution.
    match (selected_piece, clicked_piece) {
        // Case 1: Select a Piece
        (None, Some((entity, piece))) => {
            if piece.color != game_state.turn {
                return;
            }
            commands.entity(entity).insert(Selected);
        }

        // Case 2: Selected piece wants to move to Empty Square (Includes Castling)
        (Some((entity, selected_piece_data)), None) => {
            if selected_piece_data.color != game_state.turn {
                return;
            }

            // Identify if we need to move a Rook, BEFORE we borrow the query mutably.
            let mut castling_rook_task: Option<(Entity, (u8, u8))> = None;

            if selected_piece_data.kind == PieceKind::King {
                if let Ok((_, _, sq)) = piece_query.get(entity) {
                    let dx = x as i8 - sq.x as i8;

                    // If King moved 2 squares, it's a Castle attempt
                    if dx.abs() == 2 {
                        let rook_start_pos = match (x, y) {
                            (6, 0) => Some((7, 0)), // White King Side
                            (2, 0) => Some((0, 0)), // White Queen Side
                            (6, 7) => Some((7, 7)), // Black King Side
                            (2, 7) => Some((0, 7)), // Black Queen Side
                            _ => None,
                        };

                        if let Some(r_pos) = rook_start_pos {
                            let rook_dest = match r_pos {
                                (7, 0) => (5, 0),
                                (0, 0) => (3, 0),
                                (7, 7) => (5, 7),
                                (0, 7) => (3, 7),
                                _ => (0, 0), // Should not happen
                            };

                            // Search for the Rook Entity
                            if let Some((r_entity, _, _)) = piece_query
                                .iter()
                                .find(|(_, _, s)| s.x == r_pos.0 && s.y == r_pos.1)
                            {
                                castling_rook_task = Some((r_entity, rook_dest));
                            }
                        }
                    }
                }
            }

            if let Ok((_, mut piece, mut square)) = piece_query.get_mut(entity) {
                let (prev_x, prev_y) = (square.x, square.y);

                if !is_legal_move(&selected_piece_data, (prev_x, prev_y), (x, y), &board) {
                    return;
                }

                // Move.
                square.x = x;
                square.y = y;

                // Update has_moved flag.
                piece.has_moved = true;

                // Event.
                commands.trigger(MoveMadeEvent {
                    piece: entity,
                    start: (prev_x, prev_y),
                    end: (x, y),
                });
            }

            // Only runs if Phase A found a valid castling task
            if let Some((rook_entity, rook_dest)) = castling_rook_task {
                if let Ok((_, mut rook_piece, mut rook_square)) = piece_query.get_mut(rook_entity) {
                    if !rook_piece.has_moved {
                        let (r_prev_x, r_prev_y) = (rook_square.x, rook_square.y);
                        // Move Rook.
                        rook_square.x = rook_dest.0;
                        rook_square.y = rook_dest.1;

                        rook_piece.has_moved = true;

                        commands.trigger(MoveMadeEvent {
                            piece: rook_entity,
                            start: (r_prev_x, r_prev_y),
                            end: rook_dest,
                        });
                    }
                }
            }

            game_state.turn = match game_state.turn {
                PieceColor::White => PieceColor::Black,
                PieceColor::Black => PieceColor::White,
            };
            commands.entity(entity).remove::<Selected>();
        }

        // Case 3: Selected piece wants to move to a square that already has a piece on it => 3 sub-cases.
        (Some((curr_entity, curr_piece)), Some((target_entity, target_piece))) => {
            if curr_piece.color != game_state.turn {
                return;
            }

            // Sub-case 1: Clicked same piece -> Deselect
            if curr_entity == target_entity {
                commands.entity(curr_entity).remove::<Selected>();
            }
            // Sub-case 2: Clicked Friend -> Switch Selection
            else if curr_piece.color == target_piece.color {
                commands.entity(curr_entity).remove::<Selected>();
                commands.entity(target_entity).insert(Selected);
            }
            // Sub-case 3: Clicked Enemy -> CAPTURE
            else {
                // 1. Validate
                if let Ok((_, _, square)) = piece_query.get(curr_entity) {
                    if !is_legal_move(&curr_piece, (square.x, square.y), (x, y), &board) {
                        return;
                    }
                }

                // 2. Execute Capture
                commands.entity(target_entity).despawn();

                if let Ok((_, mut piece, mut square)) = piece_query.get_mut(curr_entity) {
                    let (prev_x, prev_y) = (square.x, square.y);
                    square.x = x;
                    square.y = y;

                    piece.has_moved = true;

                    game_state.turn = match game_state.turn {
                        PieceColor::White => PieceColor::Black,
                        PieceColor::Black => PieceColor::White,
                    };

                    commands.trigger(MoveMadeEvent {
                        piece: curr_entity,
                        start: (prev_x, prev_y),
                        end: (x, y),
                    });
                }
                commands.entity(curr_entity).remove::<Selected>();
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
        if let Ok(entity) = previously_selected_square_query.single() {
            commands.entity(entity).despawn();
        }

        if let Ok(square) = just_selected_square_query.single() {
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
        if let Ok(entity) = previously_selected_square_query.single() {
            commands.entity(entity).despawn();
        }
    }
}

fn highlight_legal_moves_system(
    mut commands: Commands,
    piece_query: Query<(Entity, &Piece, &Square)>,
    just_selected_square_query: Query<(&Piece, &Square), Added<Selected>>,
    previously_highlighted_legal_moves_query: Query<Entity, With<LegalMovesFilter>>,
    any_selected_query: Query<&Selected>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // CASE 1: A new piece was just selected
    if let Ok((piece, square)) = just_selected_square_query.single() {
        for entity in previously_highlighted_legal_moves_query.iter() {
            commands.entity(entity).despawn();
        }

        let board: Vec<(Piece, Square)> = piece_query.iter().map(|(_, p, s)| (*p, *s)).collect();
        let start = (square.x, square.y);
        let legal_moves = get_legal_moves(piece, start, &board);

        let color = Color::srgba(0.6, 0.1, 0.8, 0.5);
        for (x, y) in legal_moves {
            if let Some((_, _, sq)) = piece_query.iter().find(|(_, _, sq)| sq.x == x && sq.y == y) {
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.6, 0.1, 0.8, 0.5),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    Transform::from_translation(get_world_position(
                        sq.x as usize,
                        sq.y as usize,
                        0.5,
                    )),
                    LegalMovesFilter,
                ));
            }

            let highlight_circle = meshes.add(Circle::new(15.0));
            commands.spawn((
                Mesh2d(highlight_circle),
                MeshMaterial2d(materials.add(color)),
                Transform::from_translation(get_world_position(x as usize, y as usize, 0.5)),
                LegalMovesFilter,
            ));
        }
    }
    // CASE 2: Nothing is selected anymore, but highlights still exist.
    else if any_selected_query.is_empty() && !previously_highlighted_legal_moves_query.is_empty()
    {
        for entity in previously_highlighted_legal_moves_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn on_move_made(
    event: On<MoveMadeEvent>,
    mut commands: Commands,
    moved_piece_query: Query<&Square, (With<Piece>, Changed<Square>)>,
    previously_moved_piece_query: Query<Entity, With<MovedFilter>>,
    check_highlight_query: Query<Entity, With<InCheckHighlight>>,
    piece_query: Query<(Entity, &Piece, &Square)>,
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
) {
    // Remove the filter from the previous last move.
    for entity in previously_moved_piece_query.iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(entity) = check_highlight_query.single() {
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

    let board: Vec<(Piece, Square)> = piece_query.iter().map(|(_, p, s)| (*p, *s)).collect();
    for (_, piece, square) in piece_query.iter() {
        if piece.color == game_state.turn && piece.kind == PieceKind::King {
            if is_king_in_check((square.x, square.y), game_state.turn, &board) {
                let center_x = (square.x as f32 * TILE_SIZE) - OFFSET + (TILE_SIZE / 2.0);
                let center_y = (square.y as f32 * TILE_SIZE) - OFFSET + (TILE_SIZE / 2.0);

                commands.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        image: asset_server.load("effects/glow4.png"),
                        ..default()
                    },
                    Transform::from_xyz(center_x, center_y - 5.0, 0.9),
                    InCheckHighlight,
                ));
            }
        }
    }
}

fn piece_movement_system(
    mut movement_query: Query<(&Square, &mut Transform), (With<Piece>, Changed<Square>)>,
) {
    for (square, mut transform) in movement_query.iter_mut() {
        transform.translation = get_world_position(square.x as usize, square.y as usize, 1.0);
    }
}
