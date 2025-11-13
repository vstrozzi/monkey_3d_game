// This file contains the core game logic and UI functions.
use bevy::prelude::*;

use crate::utils::constants::game_constants::COSINE_ALIGNMENT_CAMERA_FACE_THRESHOLD;
use crate::utils::objects::{
    FaceMarker, GameEntity, GamePhase, GameState, Pyramid, RandomGen, UIEntity,
};
use crate::utils::setup::setup;

/// A plugin for handling game functions, including checking for face alignment and managing the game UI.
pub struct GameFunctionsPlugin;

impl Plugin for GameFunctionsPlugin {
    /// Builds the plugin by adding the `check_face_alignment` and `game_ui` systems to the app.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                crate::utils::game_functions::check_face_alignment,
                crate::utils::game_functions::game_ui,
            )
                .chain(),),
        );
    }
}

/// Spawns a semi-transparent dark overlay that covers the entire viewport.
pub fn spawn_overlay(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.)),
        UIEntity,
    ));
}

/// Spawns a styled card with centered text.
pub fn spawn_text_card(commands: &mut Commands, title: &str, content: &str, accent_color: Color) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UIEntity,
        ))
        .with_children(|parent| {
            // Card container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(3.0)),
                        max_width: Val::Px(700.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                    BorderColor::all(accent_color),
                ))
                .with_children(|card| {
                    // Title
                    card.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(accent_color),
                        TextLayout::new_with_justify(Justify::Center),
                    ));

                    // Divider
                    card.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                    ));

                    // Content
                    card.spawn((
                        Text::new(content),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            max_width: Val::Px(600.0),
                            ..default()
                        },
                    ));
                });
        });
}

/// Checks if the player has won the game by aligning the camera with the correct face of the pyramid.
pub fn check_face_alignment(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    camera_query: Query<&Transform, With<Camera3d>>,
    face_query: Query<(&Transform, &FaceMarker), With<Pyramid>>,
) {
    // Only check if the game is in Playing state
    if game_state.phase != GamePhase::Playing {
        return;
    }
    // Check for SPACE key press to check alignment
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.attempts += 1;

        let Ok(camera_transform) = camera_query.single() else {
            return;
        };
        // Get camera direction
        let camera_forward = camera_transform.local_z();

        // Check which face is most aligned with camera by getting the one with
        // the smallest dot product between camera dir and face dir towards origin
        // (i.e. face is facing camera)
        let mut best_alignment = 1.0;
        let mut best_face_index = None;

        for (face_transform, face_marker) in &face_query {
            // Get face normal in world space
            // The local normal is stored in `face_marker.normal`
            let face_normal = (face_transform.rotation * (face_marker.normal)).normalize();

            // Project down to XZ plane
            let face_normal_xz = Vec3::new(face_normal.x, 0.0, face_normal.z).normalize();
            // Calculate alignment (dot product) of camera direction and face normal
            let alignment = face_normal_xz.dot(*camera_forward);

            if alignment < best_alignment {
                best_alignment = alignment;
                best_face_index = Some(face_marker.face_index);
            }
        }

        // Check if aligned enough (within margin)
        if let Some(best_face_index) = best_face_index {
            // Check if the cosine alignment is good enough
            if best_alignment < COSINE_ALIGNMENT_CAMERA_FACE_THRESHOLD {
                // Check if the face is the correct one
                if best_face_index == game_state.pyramid_target_face_index {
                    // Transition to Won state
                    game_state.phase = GamePhase::Won;
                    game_state.end_time = Some(time.elapsed());
                    game_state.cosine_alignment = Some(best_alignment);
                    game_state.is_changed = true;
                }
            }
        }
    }
}

/// Manages the game's UI based on the current game state.
pub fn game_ui(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    entities: Query<Entity, With<GameEntity>>,
    query: Query<Entity, With<UIEntity>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    random_gen: ResMut<RandomGen>,
    time: Res<Time>,
) {
    // Check if the game state has changed from last frame before doing anything
    if !game_state.is_changed {
        return;
    }
    game_state.is_changed = false;

    // Clear all texts entities
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // State Machine using match pattern
    match game_state.phase {
        GamePhase::NotStarted => {
            if keyboard.just_pressed(KeyCode::Space) {
                // Transition: NotStarted -> Playing
                game_state.phase = GamePhase::Playing;
                game_state.start_time = Some(time.elapsed());
                game_state.attempts = 0;
                game_state.is_changed = true;
            } else {
                // Display start screen with overlay
                spawn_overlay(&mut commands);

                let title = "PYRAMID SEEKER";
                let content = "Find the correct orientation of the pyramid!\n\n\
                              Controls:\n\
                              Arrow Keys / WASD - Rotate camera\n\
                              SPACE - Check alignment\n\n\
                              Press SPACE to start";

                spawn_text_card(
                    &mut commands,
                    title,
                    content,
                    Color::srgb(1.0, 0.3, 0.3), // Red accent
                );
                game_state.is_changed = true;
            }
        }

        GamePhase::Playing => {
            // Display minimalist game HUD
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        left: Val::Px(20.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                    UIEntity,
                ))
                .with_children(|parent| {
                    // Target indicator
                    parent.spawn((
                        Text::new("Orientate yourself"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    ));

                    // Attempts counter
                    parent.spawn((
                        Text::new(format!("Attempts: {}", game_state.attempts)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.9)),
                    ));

                    // Controls hint
                    parent.spawn((
                        Text::new("WASD/Arrows | SPACE: Check"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                    ));
                });

            game_state.is_changed = true;
        }

        GamePhase::Won => {
            if keyboard.just_pressed(KeyCode::KeyR) {
                // Transition: Won -> NotStarted (restart)
                for entity in entities.iter() {
                    commands.entity(entity).despawn();
                }
                spawn_overlay(&mut commands);
                setup(commands, meshes, materials, random_gen, time);
            } else {
                // Display win screen with overlay
                spawn_overlay(&mut commands);

                let elapsed = game_state.end_time.unwrap().as_secs_f32()
                    - game_state.start_time.unwrap().as_secs_f32();
                let accuracy = game_state.cosine_alignment.unwrap().abs() * 100.0;

                let title = if game_state.attempts == 1 {
                    "PERFECT! FIRST TRY!"
                } else {
                    "VICTORY!"
                };

                let content = format!(
                    "You found the RED face!\n\n\
                     Time: {:.2}s\n\
                     Attempts: {}\n\
                     Accuracy: {:.1}%\n\n\
                     Press R to play again",
                    elapsed, game_state.attempts, accuracy
                );

                spawn_text_card(
                    &mut commands,
                    title,
                    &content,
                    Color::srgb(0.3, 1.0, 0.5), // Green accent for victory
                );
                game_state.is_changed = true;
            }
        }
    }
}
