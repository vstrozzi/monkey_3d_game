use bevy::prelude::*;

/// Plugins
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, crate::utils::setup::setup)
        .add_systems(
            Update,
            (
                crate::utils::functions::check_face_alignment,
                crate::utils::functions::game_ui,
            ),
            );
    }
}
