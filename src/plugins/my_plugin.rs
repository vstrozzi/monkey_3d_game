use bevy::prelude::*;

/// Plugins
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, crate::utils::setup::setup)
            .add_systems(
                Update,
                (
                    crate::camera::first_person_camera_look, // <-- Using the new system
                    crate::camera::mouse_look,
                    crate::utils::functions::check_face_alignment,
                    crate::utils::functions::game_ui,
                ),
            );
    }
}
