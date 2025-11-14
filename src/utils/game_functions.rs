// This file contains the core game logic and UI functions.
use bevy::prelude::*;

use crate::utils::keyboard_navigation::{
    initialize_selected_button_color, keyboard_menu_navigation, update_selected_button_visual,
};
use crate::utils::objects::{FaceMarker, GamePhase, GameSettings, GameState, Pyramid, UIEntity};
use crate::utils::settings_io;

use crate::utils::ui_components::{
    Button, ButtonAction, SettingType, spawn_button, spawn_color_display, spawn_section_header,
    spawn_slider,
};

/// A plugin for handling game functions, including checking for face alignment and managing the game UI.
pub struct GameFunctionsPlugin;

impl Plugin for GameFunctionsPlugin {
    /// Builds the plugin by adding the game systems to the app.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initialize_selected_button_color,
                keyboard_menu_navigation,
                update_selected_button_visual,
                button_interaction_system,
                handle_button_clicks,
                handle_menu_shortcut,
                check_face_alignment,
                game_ui,
            )
                .chain(),
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

/// System to handle button interactions (hover and click effects).
pub fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Button),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let normal = Color::srgba(0.15, 0.15, 0.2, 0.9);
    let hovered = Color::srgba(0.25, 0.25, 0.35, 0.95);
    let pressed = Color::srgba(0.35, 0.35, 0.45, 1.0);

    for (interaction, mut bg_color, _button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(pressed);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(hovered);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(normal);
            }
        }
    }
}

/// System to handle button click actions.
pub fn handle_button_clicks(
    mut interaction_query: Query<(&Interaction, &Button), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    game_settings: Res<GameSettings>,
) {
    for (interaction, button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match button.action {
                ButtonAction::StartGame => {
                    game_state.phase = GamePhase::Playing;
                    game_state.is_changed = true;
                }
                ButtonAction::GoToInstructions => {
                    game_state.phase = GamePhase::Instructions;
                    game_state.is_changed = true;
                }
                ButtonAction::GoToSettings => {
                    game_state.phase = GamePhase::Settings;
                    game_state.is_changed = true;
                }
                ButtonAction::BackToMenu => {
                    game_state.phase = GamePhase::Menu;
                    game_state.is_changed = true;
                }
                ButtonAction::SaveSettings => {
                    if let Err(e) = settings_io::save_settings(&game_settings) {
                        eprintln!("Failed to save settings: {}", e);
                    } else {
                        println!("Settings saved successfully!");
                    }
                }
                ButtonAction::ResetSettings => {
                    // Will be implemented when we add the settings modification logic
                    println!("Reset settings to defaults");
                }
                ButtonAction::ExitGame => {
                    std::process::exit(0);
                }
            }
        }
    }
}

/// Handles the M key shortcut to return to the main menu from any screen.
pub fn handle_menu_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
) {
    // Press M to return to main menu (except when already in menu or playing)
    if keyboard.just_pressed(KeyCode::KeyM) {
        match game_state.phase {
            GamePhase::Menu | GamePhase::Playing => {
                // Don't allow returning to menu while playing or already in menu
            }
            _ => {
                // Return to menu from Instructions, Settings, Won screens
                game_state.phase = GamePhase::Menu;
                game_state.is_changed = true;
            }
        }
    }
}

/// Checks if the player has won the game by aligning the camera with the correct face of the pyramid.
pub fn check_face_alignment(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    game_settings: Res<GameSettings>,
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
            // Check if the cosine alignment is good enough (use settings value)
            if best_alignment < game_settings.alignment_threshold {
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
    query: Query<Entity, With<UIEntity>>,
    game_settings: Res<GameSettings>,
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
        GamePhase::Menu => {
            // Display main menu
            spawn_overlay(&mut commands);
            spawn_menu_screen(&mut commands);
        }

        GamePhase::Instructions => {
            // Display instructions screen
            spawn_overlay(&mut commands);
            spawn_instructions_screen(&mut commands);
        }

        GamePhase::Settings => {
            // Display settings screen
            spawn_overlay(&mut commands);
            spawn_settings_screen(&mut commands, &game_settings);
        }

        GamePhase::Playing => {
            // Handle keyboard shortcut to start game from old NotStarted logic

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
        }

        GamePhase::Won => {
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
                 Press R to play again\n\
                 Press M for menu",
                elapsed, game_state.attempts, accuracy
            );

            spawn_text_card(
                &mut commands,
                title,
                &content,
                Color::srgb(0.3, 1.0, 0.5), // Green accent for victory
            );
        }
    }

    /// Spawns the main menu screen.
    fn spawn_menu_screen(commands: &mut Commands) {
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
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(40.0)),
                            row_gap: Val::Px(15.0),
                            border: UiRect::all(Val::Px(3.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(0.3, 0.8, 1.0)),
                    ))
                    .with_children(|card| {
                        // Title
                        card.spawn((
                            Text::new("PYRAMID SEEKER"),
                            TextFont {
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.8, 1.0)),
                            TextLayout::new_with_justify(Justify::Center),
                        ));

                        // Subtitle
                        card.spawn((
                            Text::new("Find the RED face of the pyramid!"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                            TextLayout::new_with_justify(Justify::Center),
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));

                        // Buttons (with menu indices for keyboard navigation)
                        spawn_button(card, "Start Game", ButtonAction::StartGame, None, Some(0));
                        spawn_button(
                            card,
                            "Instructions",
                            ButtonAction::GoToInstructions,
                            None,
                            Some(1),
                        );
                        spawn_button(card, "Settings", ButtonAction::GoToSettings, None, Some(2));
                        spawn_button(card, "Exit", ButtonAction::ExitGame, None, Some(3));
                    });
            });
    }

    /// Spawns the instructions screen.
    fn spawn_instructions_screen(commands: &mut Commands) {
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
                        BorderColor::all(Color::srgb(0.3, 0.8, 1.0)),
                    ))
                    .with_children(|card| {
                        // Title
                        card.spawn((
                            Text::new("HOW TO PLAY"),
                            TextFont {
                                font_size: 42.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.8, 1.0)),
                            TextLayout::new_with_justify(Justify::Center),
                        ));

                        // Content
                        card.spawn((
                            Text::new(
                                "OBJECTIVE\n\
                                 Your goal is to find and face the RED pyramid face.\n\n\
                                 CONTROLS\n\
                                 Arrow Keys / WASD - Rotate camera around pyramid\n\
                                 SPACE - Check if you're facing the RED face\n\n\
                                 WINNING\n\
                                 Align your camera with the RED face and press SPACE.\n\
                                 Try to win in as few attempts as possible!\n\n\
                                 TIPS\n\
                                 • Take your time to explore all sides\n\
                                 • The pyramid has decorations to help identify faces\n\
                                 • Your accuracy matters - align carefully!",
                            ),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                            Node {
                                max_width: Val::Px(600.0),
                                ..default()
                            },
                        ));

                        // Keyboard shortcut hint
                        card.spawn((
                            Text::new("Press M to return to menu anytime"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            Node {
                                margin: UiRect::top(Val::Px(15.0)),
                                ..default()
                            },
                        ));

                        // Back button
                        spawn_button(card, "Back to Menu", ButtonAction::BackToMenu, None, None);
                    });
            });
    }

    /// Spawns the settings screen with sliders.
    fn spawn_settings_screen(commands: &mut Commands, settings: &GameSettings) {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                UIEntity,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(30.0)),
                            row_gap: Val::Px(10.0),
                            border: UiRect::all(Val::Px(3.0)),
                            max_width: Val::Px(800.0),
                            max_height: Val::Percent(90.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(0.3, 0.8, 1.0)),
                    ))
                    .with_children(|card| {
                        // Title
                        card.spawn((
                            Text::new("SETTINGS"),
                            TextFont {
                                font_size: 42.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.8, 1.0)),
                            TextLayout::new_with_justify(Justify::Center),
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));

                        // Pyramid Settings
                        card.spawn((
                            Text::new("🔺 Pyramid Dimensions"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.8, 1.0)),
                            Node {
                                margin: UiRect::top(Val::Px(20.0)),
                                ..default()
                            },
                        ));

                        card.spawn((
                            Text::new(format!(
                                "Base Radius: {:.1} - {:.1}",
                                settings.pyramid_base_radius_min, settings.pyramid_base_radius_max
                            )),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                        ));
                        card.spawn((
                            Text::new(format!(
                                "Height: {:.1} - {:.1}",
                                settings.pyramid_height_min, settings.pyramid_height_max
                            )),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                        ));

                        // Decoration Settings
                        card.spawn((
                            Text::new("✨ Decorations"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 0.8, 1.0)),
                            Node {
                                margin: UiRect::top(Val::Px(20.0)),
                                ..default()
                            },
                        ));
                        card.spawn((
                            Text::new(format!(
                                "Count: {} - {}",
                                settings.decoration_count_min, settings.decoration_count_max
                            )),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                        ));
                        card.spawn((
                            Text::new(format!(
                                "Size: {:.2} - {:.2}",
                                settings.decoration_size_min, settings.decoration_size_max
                            )),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.95)),
                        ));

                        // Camera Settings
                        spawn_section_header(card, "📷 Camera");
                        spawn_slider(
                            card,
                            "Rotation Speed",
                            SettingType::CameraSpeedRotation,
                            0.5,
                            8.0,
                            settings.camera_speed_rotation,
                            0.1,
                            false,
                        );
                        spawn_slider(
                            card,
                            "Zoom Speed",
                            SettingType::CameraSpeedZoom,
                            0.5,
                            8.0,
                            settings.camera_speed_zoom,
                            0.1,
                            false,
                        );

                        // Game Settings
                        spawn_section_header(card, "Game");
                        spawn_slider(
                            card,
                            "Alignment Threshold",
                            SettingType::AlignmentThreshold,
                            -1.0,
                            0.0,
                            settings.alignment_threshold,
                            0.05,
                            false,
                        );
                        spawn_slider(
                            card,
                            "Random Seed",
                            SettingType::RandomSeed,
                            0.0,
                            9999.0,
                            settings.random_seed as f32,
                            1.0,
                            true,
                        );

                        // Colors display
                        spawn_section_header(card, "🎨 Pyramid Colors");
                        spawn_color_display(card, "Target Color (Red)", settings.pyramid_colors[0]);
                        spawn_color_display(card, "Color 2 (Green)", settings.pyramid_colors[1]);
                        spawn_color_display(card, "Color 3 (Blue)", settings.pyramid_colors[2]);

                        // Action buttons
                        card.spawn((Node {
                            height: Val::Px(20.0),
                            ..default()
                        },));

                        card.spawn((Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        },))
                            .with_children(|button_row| {
                                spawn_button(
                                    button_row,
                                    "Save to File",
                                    ButtonAction::SaveSettings,
                                    Some(Color::srgb(0.3, 1.0, 0.5)),
                                    None,
                                );
                                spawn_button(
                                    button_row,
                                    "Reset Defaults",
                                    ButtonAction::ResetSettings,
                                    Some(Color::srgb(0.9, 0.5, 0.3)),
                                    None,
                                );
                            });

                        // Keyboard shortcut hint
                        card.spawn((
                            Text::new("Press M to return to menu anytime"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            Node {
                                margin: UiRect::top(Val::Px(15.0)),
                                ..default()
                            },
                        ));

                        spawn_button(card, "Back to Menu", ButtonAction::BackToMenu, None, None);
                    });
            });
    }
}
