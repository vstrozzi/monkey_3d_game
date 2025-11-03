use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use std::f32::consts::{FRAC_PI_2, TAU};
use crate::utils::constants::camera_3d_constants;

pub struct Camera3dFpovPlugin;

impl Plugin for Camera3dFpovPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, camera_3d_fpov_inputs);
    }
}

/// Orbiting 3D Camera System
/// Rotates around the origin with A/D and zooms in/out with W/S
pub fn camera_3d_fpov_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    _acc_mouse_motion_events: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // --- Static orbit parameters ---
    static mut THETA: f32 = 0.0;
    static mut RADIUS: f32 = 5.0;

    let speed = camera_3d_constants::CAMERA_3D_SPEED * time.delta_secs();
    let zoom_speed = camera_3d_constants::CAMERA_3D_SPEED * 2.0 * time.delta_secs();

    unsafe {
        // Handle input
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            THETA -= speed;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            THETA += speed;
        }

        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
            RADIUS -= zoom_speed;
        }
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            RADIUS += zoom_speed;
        }

        // --- Clamp zoom range ---
        const MIN_RADIUS: f32 = 5.0;
        const MAX_RADIUS: f32 = 20.0;
        RADIUS = RADIUS.clamp(MIN_RADIUS, MAX_RADIUS);

        // Wrap angle
        if THETA > TAU {
            THETA -= TAU;
        } else if THETA < 0.0 {
            THETA += TAU;
        }

        // Compute new position
        let new_x = RADIUS * THETA.cos();
        let new_z = RADIUS * THETA.sin();
        let new_y = transform.translation.y; // keep same height

        transform.translation = Vec3::new(new_x, new_y, new_z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
