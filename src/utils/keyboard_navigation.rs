// This file contains keyboard navigation functionality for menu buttons.
use bevy::prelude::*;

/// Component for menu buttons with keyboard navigation support.
#[derive(Component)]
pub struct MenuButton {
    pub index: usize,
}

/// Marker component for the currently selected button in keyboard navigation.
#[derive(Component)]
pub struct SelectedButton;

/// Resource to track button selection state and colors.
pub struct ButtonNavigationColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub focused: Color,
    pub text: Color,
}

impl Default for ButtonNavigationColors {
    fn default() -> Self {
        Self {
            normal: Color::srgba(0.15, 0.15, 0.2, 0.9),
            hovered: Color::srgba(0.25, 0.25, 0.35, 0.95),
            pressed: Color::srgba(0.35, 0.35, 0.45, 1.0),
            focused: Color::srgba(0.2, 0.5, 0.8, 0.95),
            text: Color::srgb(0.9, 0.9, 0.95),
        }
    }
}

/// System to handle keyboard navigation for menu buttons.
pub fn keyboard_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    buttons_query: Query<(Entity, &MenuButton), With<MenuButton>>,
    selected_query: Query<(Entity, &MenuButton), With<SelectedButton>>,
    mut button_query: Query<(&mut BackgroundColor, &mut Interaction)>,
) {
    // Get all menu buttons sorted by index
    let mut buttons: Vec<(Entity, &MenuButton)> = buttons_query.iter().collect();
    if buttons.is_empty() {
        return;
    }

    buttons.sort_by_key(|(_, menu_btn)| menu_btn.index);
    let button_count = buttons.len();

    // Get currently selected button
    let current_selected = selected_query.iter().next();

    let colors = ButtonNavigationColors::default();

    // Handle down/S key
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        if let Some((current_entity, current_button)) = current_selected {
            // Remove selection from current button
            commands.entity(current_entity).remove::<SelectedButton>();
            if let Ok((mut bg_color, _)) = button_query.get_mut(current_entity) {
                *bg_color = BackgroundColor(colors.normal);
            }

            // Move to next button (wrap around)
            let next_index = (current_button.index + 1) % button_count;
            if let Some((next_entity, _)) = buttons.iter().find(|(_, btn)| btn.index == next_index)
            {
                commands.entity(*next_entity).insert(SelectedButton);
                if let Ok((mut bg_color, _)) = button_query.get_mut(*next_entity) {
                    *bg_color = BackgroundColor(colors.focused);
                }
            }
        } else {
            // No button selected, select the first one
            if let Some((first_entity, _)) = buttons.first() {
                commands.entity(*first_entity).insert(SelectedButton);
                if let Ok((mut bg_color, _)) = button_query.get_mut(*first_entity) {
                    *bg_color = BackgroundColor(colors.focused);
                }
            }
        }
    }

    // Handle up/W key
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        if let Some((current_entity, current_button)) = current_selected {
            // Remove selection from current button
            commands.entity(current_entity).remove::<SelectedButton>();
            if let Ok((mut bg_color, _)) = button_query.get_mut(current_entity) {
                *bg_color = BackgroundColor(colors.normal);
            }

            // Move to previous button (wrap around)
            let prev_index = if current_button.index == 0 {
                button_count - 1
            } else {
                current_button.index - 1
            };
            if let Some((prev_entity, _)) = buttons.iter().find(|(_, btn)| btn.index == prev_index)
            {
                commands.entity(*prev_entity).insert(SelectedButton);
                if let Ok((mut bg_color, _)) = button_query.get_mut(*prev_entity) {
                    *bg_color = BackgroundColor(colors.focused);
                }
            }
        } else {
            // No button selected, select the last one
            if let Some((last_entity, _)) = buttons.last() {
                commands.entity(*last_entity).insert(SelectedButton);
                if let Ok((mut bg_color, _)) = button_query.get_mut(*last_entity) {
                    *bg_color = BackgroundColor(colors.focused);
                }
            }
        }
    }

    // Handle Enter/Space key
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        if let Some((current_entity, _current_button)) = current_selected {
            // Trigger the button by setting its interaction to Pressed
            if let Ok((mut bg_color, mut interaction)) = button_query.get_mut(current_entity) {
                *interaction = Interaction::Pressed;
                *bg_color = BackgroundColor(colors.pressed);
            }
        }
    }
}

/// System to update button colors based on selection state.
pub fn update_selected_button_visual(
    mut button_query: Query<
        (&mut BackgroundColor, &Interaction, Option<&SelectedButton>),
        (With<MenuButton>, Changed<Interaction>),
    >,
) {
    let colors = ButtonNavigationColors::default();

    for (mut bg_color, interaction, selected) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(colors.pressed);
            }
            Interaction::Hovered => {
                // Don't change color if selected (keep focused color)
                if selected.is_none() {
                    *bg_color = BackgroundColor(colors.hovered);
                }
            }
            Interaction::None => {
                // Use focused color if selected, otherwise normal
                if selected.is_some() {
                    *bg_color = BackgroundColor(colors.focused);
                } else {
                    *bg_color = BackgroundColor(colors.normal);
                }
            }
        }
    }
}

/// System to initialize the visual state of selected buttons when they spawn.
pub fn initialize_selected_button_color(
    mut button_query: Query<
        (&mut BackgroundColor, &SelectedButton),
        (With<MenuButton>, Added<SelectedButton>),
    >,
) {
    let colors = ButtonNavigationColors::default();

    for (mut bg_color, _) in &mut button_query {
        *bg_color = BackgroundColor(colors.focused);
    }
}
