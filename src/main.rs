use bevy::{prelude::*};

use monkey_3D_game::plugins::my_plugin::{MyPlugin};
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
        .run();
}




