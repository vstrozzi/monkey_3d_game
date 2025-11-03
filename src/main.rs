use bevy::{prelude::*};

use monkey_3d_game::plugins::my_plugin::{MyPlugin};
use monkey_3d_game::camera::Camera3dFpovPlugin;
/// Main application function
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pyramid Face Finder".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MyPlugin)
        .add_plugins(Camera3dFpovPlugin)
        .run();
}