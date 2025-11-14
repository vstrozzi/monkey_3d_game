// This file contains reusable UI components for buttons, sliders, and other interactive elements.
use bevy::prelude::*;

use crate::utils::keyboard_navigation::{ButtonNavigationColors, MenuButton, SelectedButton};

/// Marker component for interactive buttons.
#[derive(Component)]
pub struct Button {
    pub action: ButtonAction,
}

/// Enum representing different button actions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonAction {
    StartGame,
    GoToInstructions,
    GoToSettings,
    BackToMenu,
    SaveSettings,
    ResetSettings,
    ExitGame,
}

/// Component for slider controls.
#[derive(Component)]
pub struct Slider {
    pub setting_type: SettingType,
    pub min_value: f32,
    pub max_value: f32,
    pub current_value: f32,
    pub step: f32,
}

/// Enum representing which setting a slider controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettingType {
    PyramidBaseRadiusMin,
    PyramidBaseRadiusMax,
    PyramidHeightMin,
    PyramidHeightMax,
    DecorationCountMin,
    DecorationCountMax,
    DecorationSizeMin,
    DecorationSizeMax,
    CameraSpeedRotation,
    CameraSpeedZoom,
    CameraMinRadius,
    CameraMaxRadius,
    AlignmentThreshold,
    RandomSeed,
}

/// Marker component for slider handles (the draggable part).
#[derive(Component)]
pub struct SliderHandle;

/// Marker component for slider tracks (the background bar).
#[derive(Component)]
pub struct SliderTrack;

/// Colors for button states.
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub text: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            normal: Color::srgba(0.15, 0.15, 0.2, 0.9),
            hovered: Color::srgba(0.25, 0.25, 0.35, 0.95),
            pressed: Color::srgba(0.35, 0.35, 0.45, 1.0),
            text: Color::srgb(0.9, 0.9, 0.95),
        }
    }
}

/// Spawns a styled button with text and action. Returns the Entity ID.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: ButtonAction,
    accent_color: Option<Color>,
    menu_index: Option<usize>,
) -> Entity {
    let colors = ButtonColors::default();
    let nav_colors = ButtonNavigationColors::default();

    // Use focused color for first button (index 0), otherwise use normal or accent color
    let button_color = if menu_index == Some(0) && accent_color.is_none() {
        nav_colors.focused
    } else {
        accent_color.unwrap_or(colors.normal)
    };

    let mut entity_commands = parent.spawn((
        Node {
            width: Val::Px(250.0),
            height: Val::Px(50.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(button_color),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.3)),
        Interaction::default(),
        Button { action },
    ));

    // Add MenuButton component if menu_index is provided
    if let Some(index) = menu_index {
        entity_commands.insert(MenuButton { index });

        // Mark first button (index 0) as selected by default
        if index == 0 {
            entity_commands.insert(SelectedButton);
        }
    }

    let entity_id = entity_commands
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(colors.text),
            ));
        })
        .id();

    entity_id
}

/// Spawns a slider control with label and current value display.
pub fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    setting_type: SettingType,
    min_value: f32,
    max_value: f32,
    current_value: f32,
    step: f32,
    is_integer: bool,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            margin: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(5.0),
            ..default()
        },))
        .with_children(|slider_container| {
            // Label and value display row
            slider_container
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },))
                .with_children(|label_row| {
                    // Label
                    label_row.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));

                    // Current value display
                    let value_text = if is_integer {
                        format!("{}", current_value as i32)
                    } else {
                        format!("{:.2}", current_value)
                    };

                    label_row.spawn((
                        Text::new(value_text),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.8, 1.0)),
                    ));
                });

            // Slider track and handle
            slider_container
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.8)),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                    SliderTrack,
                ))
                .with_children(|track| {
                    // Calculate handle position based on value
                    let normalized_value = (current_value - min_value) / (max_value - min_value);
                    let position_percent = normalized_value * 100.0;

                    track.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(position_percent),
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.8, 1.0)),
                        BorderColor::all(Color::srgb(1.0, 1.0, 1.0)),
                        SliderHandle,
                        Interaction::default(),
                        Slider {
                            setting_type,
                            min_value,
                            max_value,
                            current_value,
                            step,
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
    let colors = ButtonColors::default();

    for (interaction, mut bg_color, _button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(colors.pressed);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(colors.hovered);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(colors.normal);
            }
        }
    }
}

/// Helper to create a section header for settings groups.
pub fn spawn_section_header(parent: &mut ChildSpawnerCommands, text: &str) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                margin: UiRect {
                    top: Val::Px(20.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
                padding: UiRect {
                    bottom: Val::Px(5.0),
                    ..default()
                },
                border: UiRect {
                    bottom: Val::Px(2.0),
                    ..default()
                },
                ..default()
            },
            BorderColor {
                bottom: Color::srgba(1.0, 1.0, 1.0, 0.3),
                ..default()
            },
        ))
        .with_children(|header| {
            header.spawn((
                Text::new(text),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.8, 1.0)),
            ));
        });
}

/// Helper to create a color picker display (shows current color).
pub fn spawn_color_display(parent: &mut ChildSpawnerCommands, label: &str, color: Color) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },))
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
            ));

            // Color preview box
            row.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(30.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(color),
                BorderColor::all(Color::srgb(1.0, 1.0, 1.0)),
            ));
        });
}
