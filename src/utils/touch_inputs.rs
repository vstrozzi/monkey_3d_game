//! Touch input handling for mobile/touchscreen support.
//! Implements natural iOS-like touch gestures with momentum, pinch-to-zoom, and rubber-banding.

use bevy::prelude::*;

use crate::utils::constants::camera_3d_constants::{
    CAMERA_3D_INITIAL_Y, CAMERA_3D_MAX_RADIUS, CAMERA_3D_MIN_RADIUS, CAMERA_3D_SPEED_X,
    CAMERA_3D_SPEED_Z,
};
use crate::utils::constants::touch_constants::{
    MAX_VELOCITY, MIN_VELOCITY_THRESHOLD, PINCH_SENSITIVITY, RUBBER_BAND_MAX_OVERSHOOT,
    RUBBER_BAND_SNAP_SPEED, RUBBER_BAND_STRENGTH, SWIPE_SENSITIVITY_X, TAP_MAX_DISTANCE,
    TAP_MAX_DURATION_SECS, VELOCITY_DECAY,
};
use crate::utils::objects::RotableComponent;

/// Gesture mode to lock direction once established
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GestureMode {
    #[default]
    None,
    Swipe, // Single finger swipe (rotation)
    Pinch, // Two finger pinch (zoom)
}

/// Resource to track touch state for gesture recognition with momentum
#[derive(Resource)]
pub struct TouchState {
    // Primary touch tracking
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
    pub active_touch_id: Option<u64>,
    pub touch_start_time: Option<f32>,
    pub is_potential_tap: bool,

    // Second touch for pinch gesture
    pub second_touch_id: Option<u64>,
    pub second_touch_position: Option<Vec2>,
    pub pinch_start_distance: Option<f32>,

    // Gesture mode locking
    pub gesture_mode: GestureMode,

    // Momentum tracking for natural iOS-like feel
    pub velocity: Vec2,              // Current velocity (x=rotation, y=zoom)
    pub is_momentum_active: bool,    // Whether momentum is being applied
    pub last_touch_position: Option<Vec2>,
    pub last_touch_time: f32,

    // Rubber-band state for zoom limits
    pub zoom_overshoot: f32,         // How much we've exceeded the limit
    pub is_rubber_banding: bool,     // Whether we're snapping back
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            start_position: None,
            current_position: None,
            active_touch_id: None,
            touch_start_time: None,
            is_potential_tap: true,
            second_touch_id: None,
            second_touch_position: None,
            pinch_start_distance: None,
            gesture_mode: GestureMode::None,
            velocity: Vec2::ZERO,
            is_momentum_active: false,
            last_touch_position: None,
            last_touch_time: 0.0,
            zoom_overshoot: 0.0,
            is_rubber_banding: false,
        }
    }
}

/// Plugin for touch input handling
pub struct TouchInputPlugin;

impl Plugin for TouchInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchState>()
            .add_message::<TouchTapEvent>()
            .add_systems(
                Update,
                (
                    track_touch_gestures,
                    process_touch_swipe,
                    process_pinch_zoom,
                    apply_touch_momentum,
                    apply_rubber_band_effect,
                )
                    .chain(),
            );
    }
}

/// Event fired when a tap is detected (using Message trait for Bevy 0.17)
#[derive(Message)]
pub struct TouchTapEvent;

/// System to track touch gestures and detect taps, including pinch detection
pub fn track_touch_gestures(
    touches: Res<Touches>,
    time: Res<Time>,
    mut touch_state: ResMut<TouchState>,
    mut tap_events: MessageWriter<TouchTapEvent>,
) {
    let current_time = time.elapsed_secs();

    // Handle new touch starts
    for touch in touches.iter_just_pressed() {
        if touch_state.active_touch_id.is_none() {
            // First touch
            touch_state.active_touch_id = Some(touch.id());
            touch_state.start_position = Some(touch.position());
            touch_state.current_position = Some(touch.position());
            touch_state.last_touch_position = Some(touch.position());
            touch_state.touch_start_time = Some(current_time);
            touch_state.last_touch_time = current_time;
            touch_state.is_potential_tap = true;
            touch_state.gesture_mode = GestureMode::None;
            touch_state.velocity = Vec2::ZERO;
            touch_state.is_momentum_active = false;
        } else if touch_state.second_touch_id.is_none() && Some(touch.id()) != touch_state.active_touch_id {
            // Second touch - initiate pinch
            touch_state.second_touch_id = Some(touch.id());
            touch_state.second_touch_position = Some(touch.position());
            touch_state.is_potential_tap = false;
            touch_state.gesture_mode = GestureMode::Pinch;

            // Calculate initial pinch distance
            if let (Some(pos1), Some(pos2)) = (touch_state.current_position, Some(touch.position())) {
                touch_state.pinch_start_distance = Some((pos1 - pos2).length());
            }
        }
    }

    // Track touch movement
    for touch in touches.iter() {
        let touch_id = touch.id();
        let new_position = touch.position();

        if Some(touch_id) == touch_state.active_touch_id {
            let old_position = touch_state.current_position;
            touch_state.current_position = Some(new_position);

            // Calculate velocity based on movement
            if let Some(old_pos) = old_position {
                let delta_time = current_time - touch_state.last_touch_time;
                if delta_time > 0.0 {
                    let delta = new_position - old_pos;
                    // Smooth velocity calculation with some damping
                    let instant_velocity = delta / delta_time.max(0.001);
                    touch_state.velocity = touch_state.velocity * 0.5 + instant_velocity * 0.5;
                    // Clamp velocity
                    touch_state.velocity = touch_state.velocity.clamp(
                        Vec2::splat(-MAX_VELOCITY),
                        Vec2::splat(MAX_VELOCITY),
                    );
                }
            }
            touch_state.last_touch_position = Some(new_position);
            touch_state.last_touch_time = current_time;

            // Check if moved too far to be a tap
            if let Some(start) = touch_state.start_position {
                let distance = (new_position - start).length();
                if distance > TAP_MAX_DISTANCE {
                    touch_state.is_potential_tap = false;

                    // Determine gesture mode if not yet set (and not pinching)
                    if touch_state.gesture_mode == GestureMode::None
                        && touch_state.second_touch_id.is_none()
                    {
                        // Single finger is always swipe for rotation
                        touch_state.gesture_mode = GestureMode::Swipe;
                    }
                }
            }
        } else if Some(touch_id) == touch_state.second_touch_id {
            touch_state.second_touch_position = Some(new_position);
        }
    }

    // Handle touch releases
    for touch in touches.iter_just_released() {
        let touch_id = touch.id();

        if Some(touch_id) == touch_state.second_touch_id {
            // Second touch released - end pinch, but keep primary touch
            touch_state.second_touch_id = None;
            touch_state.second_touch_position = None;
            touch_state.pinch_start_distance = None;
            // Don't reset gesture mode - let it continue as single touch
            if touch_state.active_touch_id.is_some() {
                touch_state.gesture_mode = GestureMode::None; // Will be re-determined
            }
        } else if Some(touch_id) == touch_state.active_touch_id {
            // Primary touch released
            // Check if it was a tap
            if touch_state.is_potential_tap && touch_state.second_touch_id.is_none() {
                if let Some(start_time) = touch_state.touch_start_time {
                    let duration = current_time - start_time;
                    if duration <= TAP_MAX_DURATION_SECS {
                        tap_events.write(TouchTapEvent);
                    }
                }
            }

            // Enable momentum if we had significant velocity
            if touch_state.velocity.length() > MIN_VELOCITY_THRESHOLD {
                touch_state.is_momentum_active = true;
            }

            // Reset primary touch state (but keep velocity for momentum)
            touch_state.active_touch_id = None;
            touch_state.start_position = None;
            touch_state.current_position = None;
            touch_state.touch_start_time = None;
            touch_state.is_potential_tap = true;
            touch_state.last_touch_position = None;

            // If second touch is still active, promote it to primary
            if let Some(second_id) = touch_state.second_touch_id {
                touch_state.active_touch_id = Some(second_id);
                touch_state.current_position = touch_state.second_touch_position;
                touch_state.start_position = touch_state.second_touch_position;
                touch_state.second_touch_id = None;
                touch_state.second_touch_position = None;
                touch_state.pinch_start_distance = None;
                touch_state.gesture_mode = GestureMode::None;
            } else {
                touch_state.gesture_mode = GestureMode::None;
            }
        }
    }

    // Handle cancelled touches
    for touch in touches.iter_just_canceled() {
        let touch_id = touch.id();

        if Some(touch_id) == touch_state.active_touch_id
            || Some(touch_id) == touch_state.second_touch_id
        {
            // Full reset on cancel
            *touch_state = TouchState::default();
        }
    }
}

/// System to process single-finger touch swipes for camera rotation
pub fn process_touch_swipe(
    touches: Res<Touches>,
    touch_state: Res<TouchState>,
    timer: Res<Time>,
    mut rot_entities: Query<&mut Transform, (With<RotableComponent>, Without<Camera3d>)>,
    gamestate: Res<crate::utils::objects::GameState>,
) {
    if gamestate.is_animating {
        return;
    }

    // Only process single-finger gestures (not pinch)
    if touch_state.active_touch_id.is_none()
        || touch_state.is_potential_tap
        || touch_state.gesture_mode == GestureMode::Pinch
    {
        return;
    }

    for touch in touches.iter() {
        if Some(touch.id()) == touch_state.active_touch_id {
            let delta = touch.delta();

            if delta.length() < 0.1 {
                continue;
            }

            if touch_state.gesture_mode == GestureMode::Swipe {
                // Rotate objects based on horizontal swipe
                let rotation_speed = CAMERA_3D_SPEED_X * timer.delta_secs();
                let rotation_amount = delta.x * SWIPE_SENSITIVITY_X * rotation_speed * 10.0;

                for mut rot_entity_transform in &mut rot_entities {
                    let (mut yaw, _, _) = rot_entity_transform.rotation.to_euler(EulerRot::YXZ);
                    yaw += rotation_amount;
                    rot_entity_transform.rotation = Quat::from_rotation_y(yaw);
                }
            }
        }
    }
}

/// System to process pinch-to-zoom gesture
pub fn process_pinch_zoom(
    mut touch_state: ResMut<TouchState>,
    timer: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    gamestate: Res<crate::utils::objects::GameState>,
) {
    if gamestate.is_animating {
        return;
    }

    // Only process if we have two active touches
    if touch_state.gesture_mode != GestureMode::Pinch {
        return;
    }

    let (Some(pos1), Some(pos2)) = (touch_state.current_position, touch_state.second_touch_position)
    else {
        return;
    };

    let current_distance = (pos1 - pos2).length();

    if let Some(start_distance) = touch_state.pinch_start_distance {
        let distance_delta = current_distance - start_distance;

        // Pinch in = zoom in (decrease radius), pinch out = zoom out
        // Invert the delta since smaller distance means zoom in
        let zoom_delta = -distance_delta * PINCH_SENSITIVITY * 0.1;

        apply_zoom_delta(zoom_delta, &timer, &mut camera_query, &mut touch_state);

        // Update start distance for continuous pinching
        touch_state.pinch_start_distance = Some(current_distance);
    }
}

/// Helper function to apply zoom with rubber-band effect
fn apply_zoom_delta(
    zoom_delta: f32,
    timer: &Res<Time>,
    camera_query: &mut Query<&mut Transform, With<Camera3d>>,
    touch_state: &mut ResMut<TouchState>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let zoom_speed = CAMERA_3D_SPEED_Z * timer.delta_secs();
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let mut radius = transform.translation.xz().length();

    // Apply zoom delta
    let zoom_change = zoom_delta * zoom_speed * 10.0;
    radius += zoom_change;

    // Rubber-band effect: allow slight overshoot with resistance
    let min_with_overshoot = CAMERA_3D_MIN_RADIUS - RUBBER_BAND_MAX_OVERSHOOT;
    let max_with_overshoot = CAMERA_3D_MAX_RADIUS + RUBBER_BAND_MAX_OVERSHOOT;

    if radius < CAMERA_3D_MIN_RADIUS {
        // Exceeded minimum - apply rubber band resistance
        let overshoot = CAMERA_3D_MIN_RADIUS - radius;
        touch_state.zoom_overshoot = -overshoot;
        touch_state.is_rubber_banding = true;
        // Reduce the effective overshoot with resistance
        radius = CAMERA_3D_MIN_RADIUS - (overshoot * RUBBER_BAND_STRENGTH);
        radius = radius.max(min_with_overshoot);
    } else if radius > CAMERA_3D_MAX_RADIUS {
        // Exceeded maximum - apply rubber band resistance
        let overshoot = radius - CAMERA_3D_MAX_RADIUS;
        touch_state.zoom_overshoot = overshoot;
        touch_state.is_rubber_banding = true;
        // Reduce the effective overshoot with resistance
        radius = CAMERA_3D_MAX_RADIUS + (overshoot * RUBBER_BAND_STRENGTH);
        radius = radius.min(max_with_overshoot);
    } else {
        touch_state.zoom_overshoot = 0.0;
        touch_state.is_rubber_banding = false;
    }

    transform.translation = Vec3::new(radius * yaw.sin(), CAMERA_3D_INITIAL_Y, radius * yaw.cos());
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

/// System to apply momentum after touch release for natural iOS-like feel
pub fn apply_touch_momentum(
    mut touch_state: ResMut<TouchState>,
    timer: Res<Time>,
    mut rot_entities: Query<&mut Transform, (With<RotableComponent>, Without<Camera3d>)>,
    gamestate: Res<crate::utils::objects::GameState>,
) {
    if gamestate.is_animating || !touch_state.is_momentum_active {
        return;
    }

    // Stop momentum if touch is active again
    if touch_state.active_touch_id.is_some() {
        touch_state.is_momentum_active = false;
        touch_state.velocity = Vec2::ZERO;
        return;
    }

    let velocity = touch_state.velocity;

    // Apply decay
    touch_state.velocity *= VELOCITY_DECAY;

    // Stop if velocity is too low
    if touch_state.velocity.length() < MIN_VELOCITY_THRESHOLD {
        touch_state.is_momentum_active = false;
        touch_state.velocity = Vec2::ZERO;
        return;
    }

    let dt = timer.delta_secs();

    // Apply rotation momentum (horizontal velocity)
    if velocity.x.abs() > MIN_VELOCITY_THRESHOLD {
        let rotation_speed = CAMERA_3D_SPEED_X * dt;
        let rotation_amount = velocity.x * SWIPE_SENSITIVITY_X * rotation_speed * 0.5;

        for mut rot_entity_transform in &mut rot_entities {
            let (mut yaw, _, _) = rot_entity_transform.rotation.to_euler(EulerRot::YXZ);
            yaw += rotation_amount;
            rot_entity_transform.rotation = Quat::from_rotation_y(yaw);
        }
    }

    // Note: Zoom momentum removed - zoom is pinch-only
}

/// System to apply rubber-band snap-back effect when zoom exceeds limits
pub fn apply_rubber_band_effect(
    mut touch_state: ResMut<TouchState>,
    timer: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    gamestate: Res<crate::utils::objects::GameState>,
) {
    if gamestate.is_animating {
        return;
    }

    // Only apply rubber band when not actively touching and when we have overshoot
    if touch_state.active_touch_id.is_some() || !touch_state.is_rubber_banding {
        return;
    }

    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let radius = transform.translation.xz().length();
    let dt = timer.delta_secs();

    // Calculate target radius (clamped to valid range)
    let target_radius = radius.clamp(CAMERA_3D_MIN_RADIUS, CAMERA_3D_MAX_RADIUS);

    // Smoothly interpolate back to valid range
    let new_radius = radius + (target_radius - radius) * RUBBER_BAND_SNAP_SPEED * dt;

    // Check if we've snapped back close enough
    if (new_radius - target_radius).abs() < 0.01 {
        touch_state.is_rubber_banding = false;
        touch_state.zoom_overshoot = 0.0;

        transform.translation =
            Vec3::new(target_radius * yaw.sin(), CAMERA_3D_INITIAL_Y, target_radius * yaw.cos());
    } else {
        transform.translation =
            Vec3::new(new_radius * yaw.sin(), CAMERA_3D_INITIAL_Y, new_radius * yaw.cos());
    }
    transform.look_at(Vec3::ZERO, Vec3::Y);
}
